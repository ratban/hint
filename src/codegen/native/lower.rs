//! HIR to Cranelift IR lowering - Cranelift 0.110 compatible.

use std::collections::HashMap;

use cranelift_codegen::ir::{
    types, InstBuilder, Value, Type, condcodes::IntCC, FuncRef, ExtFuncData,
    GlobalValueData, ExternalName, UserExternalNameRef,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::{Module, FuncId, DataId};

use crate::ir::{HirInstruction, HirConstant, HirBinaryOp, HirBlock};
use crate::semantics::{HintType, IntSize, FloatSize};

/// Context for lowering a function
pub struct FunctionLowering<'a> {
    pub builder: &'a mut FunctionBuilder<'a>,
    pub variables: HashMap<String, (Variable, HintType)>,
    pub param_vars: Vec<(Variable, HintType)>,
    pub next_var: u32,
    pub value_stack: Vec<Value>,
    pub string_literals: &'a HashMap<String, DataId>,
    pub runtime_funcs: &'a HashMap<String, FuncId>,
    pub func_refs: HashMap<FuncId, FuncRef>,
    pub module: &'a mut dyn Module,
    pub call_conv: CallConv,
}

impl<'a> FunctionLowering<'a> {
    pub fn create_variable(&mut self, ty: &HintType) -> Variable {
        let var = Variable::from_u32(self.next_var);
        self.next_var += 1;
        let clif_type = self.hint_to_clif_type(ty);
        self.builder.declare_var(var, clif_type);
        var
    }
    
    pub fn hint_to_clif_type(&self, ty: &HintType) -> Type {
        match ty {
            HintType::Int(IntSize::I8) | HintType::UInt(IntSize::I8) => types::I8,
            HintType::Int(IntSize::I16) | HintType::UInt(IntSize::I16) => types::I16,
            HintType::Int(IntSize::I32) | HintType::UInt(IntSize::I32) => types::I32,
            HintType::Int(IntSize::I64) | HintType::UInt(IntSize::I64) => types::I64,
            HintType::Float(FloatSize::F32) => types::F32,
            HintType::Float(FloatSize::F64) => types::F64,
            HintType::Bool => types::I8,
            HintType::String | HintType::Pointer(_) | HintType::Array(_, _) | HintType::Function(_, _) => types::I64,
            HintType::Void => types::INVALID,
            HintType::Unknown => types::I64,
        }
    }
    
    pub fn lower_block(&mut self, block: &HirBlock) -> Result<(), String> {
        for instr in &block.instructions {
            self.lower_instruction(instr)?;
        }
        Ok(())
    }
    
    fn get_func_ref(&mut self, func_id: FuncId) -> FuncRef {
        if let Some(func_ref) = self.func_refs.get(&func_id) {
            return *func_ref;
        }
        
        let func_ref = self.builder.import_function(ExtFuncData::from_func_id(func_id));
        self.func_refs.insert(func_id, func_ref);
        func_ref
    }
    
    pub fn lower_instruction(&mut self, instr: &HirInstruction) -> Result<(), String> {
        match instr {
            HirInstruction::LoadConst { dest, value } => {
                self.lower_load_const(dest.id, value)?;
            }
            HirInstruction::LoadVar { dest, source } => {
                self.lower_load_var(dest.id, source)?;
            }
            HirInstruction::StoreVar { dest, source } => {
                self.lower_store_var(dest, source.id)?;
            }
            HirInstruction::BinaryOp { dest, op, left, right, result_type } => {
                self.lower_binary_op(dest.id, *op, left.id, right.id, result_type)?;
            }
            HirInstruction::Call { dest, function, args } => {
                self.lower_call(dest.as_ref().map(|d| d.id), function, args.iter().map(|a| a.id).collect())?;
            }
            HirInstruction::Return { value } => {
                self.lower_return(value.as_ref().map(|v| v.id))?;
            }
            HirInstruction::Print { value } => {
                self.lower_print(value.id)?;
            }
            HirInstruction::Jump { .. } => {
                return Err("Jump not implemented".to_string());
            }
            HirInstruction::Branch { condition, .. } => {
                self.lower_branch(condition.id)?;
            }
            HirInstruction::Label { .. } | HirInstruction::Nop => {}
        }
        Ok(())
    }
    
    fn lower_load_const(&mut self, dest_id: usize, value: &HirConstant) -> Result<(), String> {
        let clif_value = match value {
            HirConstant::Int(n) => self.builder.ins().iconst(types::I64, *n),
            HirConstant::Float(f) => {
                // For floats, use iconst with bits (Cranelift will interpret correctly)
                self.builder.ins().iconst(types::I64, f.to_bits() as i64)
            }
            HirConstant::Bool(b) => self.builder.ins().iconst(types::I8, if *b { 1 } else { 0 }),
            HirConstant::String(s) => {
                if let Some(data_id) = self.string_literals.get(s) {
                    let gv = self.builder.create_global_value(
                        GlobalValueData::Symbol {
                            name: ExternalName::user(UserExternalNameRef::from_u32(data_id.as_u32())),
                            colocated: false,
                            tls: false,
                            offset: 0.into(),
                        }
                    );
                    self.builder.ins().global_value(types::I64, gv)
                } else {
                    return Err(format!("String not found: {}", s));
                }
            }
        };
        
        while self.value_stack.len() <= dest_id {
            self.value_stack.push(self.builder.ins().iconst(types::I64, 0));
        }
        self.value_stack[dest_id] = clif_value;
        Ok(())
    }
    
    fn lower_load_var(&mut self, dest_id: usize, source: &str) -> Result<(), String> {
        let (var, _) = self.variables.get(source)
            .ok_or_else(|| format!("Variable not found: {}", source))?
            .clone();
        
        let value = self.builder.use_var(var);
        
        while self.value_stack.len() <= dest_id {
            self.value_stack.push(self.builder.ins().iconst(types::I64, 0));
        }
        self.value_stack[dest_id] = value;
        Ok(())
    }
    
    fn lower_store_var(&mut self, dest: &str, source_id: usize) -> Result<(), String> {
        let source_value = self.value_stack.get(source_id)
            .copied()
            .ok_or_else(|| format!("Value stack underflow at {}", source_id))?;
        
        let var = match self.variables.get(dest) {
            Some((v, _)) => *v,
            None => {
                let var = self.create_variable(&HintType::Int(IntSize::I64));
                self.variables.insert(dest.to_string(), (var, HintType::Int(IntSize::I64)));
                var
            }
        };
        
        self.builder.def_var(var, source_value);
        Ok(())
    }
    
    fn lower_binary_op(
        &mut self,
        dest_id: usize,
        op: HirBinaryOp,
        left_id: usize,
        right_id: usize,
        result_type: &HintType,
    ) -> Result<(), String> {
        let left = self.value_stack.get(left_id)
            .copied().ok_or_else(|| format!("Stack underflow at {}", left_id))?;
        let right = self.value_stack.get(right_id)
            .copied().ok_or_else(|| format!("Stack underflow at {}", right_id))?;
        
        let result = match op {
            HirBinaryOp::Add => if result_type.is_float() { self.builder.ins().fadd(left, right) } else { self.builder.ins().iadd(left, right) },
            HirBinaryOp::Sub => if result_type.is_float() { self.builder.ins().fsub(left, right) } else { self.builder.ins().isub(left, right) },
            HirBinaryOp::Mul => if result_type.is_float() { self.builder.ins().fmul(left, right) } else { self.builder.ins().imul(left, right) },
            HirBinaryOp::Div => if result_type.is_float() { self.builder.ins().fdiv(left, right) } else { self.builder.ins().sdiv(left, right) },
            HirBinaryOp::Mod => self.builder.ins().urem(left, right),
            HirBinaryOp::Eq => self.builder.ins().icmp(IntCC::Equal, left, right),
            HirBinaryOp::Ne => self.builder.ins().icmp(IntCC::NotEqual, left, right),
            HirBinaryOp::Lt => self.builder.ins().icmp(IntCC::SignedLessThan, left, right),
            HirBinaryOp::Le => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, left, right),
            HirBinaryOp::Gt => self.builder.ins().icmp(IntCC::SignedGreaterThan, left, right),
            HirBinaryOp::Ge => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left, right),
            HirBinaryOp::And | HirBinaryOp::BitAnd => self.builder.ins().band(left, right),
            HirBinaryOp::Or | HirBinaryOp::BitOr => self.builder.ins().bor(left, right),
            HirBinaryOp::BitXor => self.builder.ins().bxor(left, right),
            HirBinaryOp::Shl => self.builder.ins().ishl(left, right),
            HirBinaryOp::Shr => self.builder.ins().ushr(left, right),
        };
        
        while self.value_stack.len() <= dest_id {
            self.value_stack.push(self.builder.ins().iconst(types::I64, 0));
        }
        self.value_stack[dest_id] = result;
        Ok(())
    }
    
    fn lower_call(&mut self, dest_id: Option<usize>, function: &str, args: Vec<usize>) -> Result<(), String> {
        let arg_values: Vec<Value> = args.iter()
            .map(|id| self.value_stack.get(*id).copied().ok_or_else(|| format!("Stack underflow at {}", id)))
            .collect::<Result<Vec<_>, _>>()?;
        
        if let Some(func_id) = self.runtime_funcs.get(function) {
            let func_ref = self.get_func_ref(*func_id);
            let call = self.builder.ins().call(func_ref, &arg_values);
            if let Some(dest) = dest_id {
                let results = self.builder.inst_results(call);
                if !results.is_empty() {
                    let zero_val = self.builder.ins().iconst(types::I64, 0);
                    while self.value_stack.len() <= dest {
                        self.value_stack.push(zero_val);
                    }
                    self.value_stack[dest] = results[0];
                }
            }
            return Ok(());
        }
        
        Err(format!("Unknown function: {}", function))
    }
    
    fn lower_return(&mut self, value_id: Option<usize>) -> Result<(), String> {
        let return_values = if let Some(id) = value_id {
            let value = self.value_stack.get(id)
                .copied().ok_or_else(|| format!("Stack underflow at {}", id))?;
            vec![value]
        } else {
            vec![]
        };
        self.builder.ins().return_(&return_values);
        Ok(())
    }
    
    fn lower_print(&mut self, value_id: usize) -> Result<(), String> {
        let value = self.value_stack.get(value_id)
            .copied().ok_or_else(|| format!("Stack underflow at {}", value_id))?;
        
        let stdout_func = self.runtime_funcs.get("stdout").ok_or("stdout not found")?;
        let stdout_ref = self.get_func_ref(*stdout_func);
        let stdout_call = self.builder.ins().call(stdout_ref, &[]);
        let stdout_ptr = self.builder.inst_results(stdout_call)[0];
        
        let fputs_func = self.runtime_funcs.get("fputs").ok_or("fputs not found")?;
        let fputs_ref = self.get_func_ref(*fputs_func);
        self.builder.ins().call(fputs_ref, &[value, stdout_ptr]);
        Ok(())
    }
    
    fn lower_branch(&mut self, condition_id: usize) -> Result<(), String> {
        let condition = self.value_stack.get(condition_id)
            .copied().ok_or_else(|| format!("Stack underflow at {}", condition_id))?;
        
        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let join_block = self.builder.create_block();
        
        self.builder.ins().brif(condition, then_block, &[], else_block, &[]);
        
        self.builder.switch_to_block(then_block);
        self.builder.ins().jump(join_block, &[]);
        
        self.builder.switch_to_block(else_block);
        self.builder.ins().jump(join_block, &[]);
        
        self.builder.switch_to_block(join_block);
        self.builder.seal_block(then_block);
        self.builder.seal_block(else_block);
        self.builder.seal_block(join_block);
        
        Ok(())
    }
}
