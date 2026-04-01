//! Intermediate Representation (IR) for Hint.
//! 
//! This module defines the High-level IR (HIR) that sits between
//! the typed AST and the low-level code generation.

use crate::semantics::{HintType, IntSize, TypedProgram, TypedStatement};

/// High-level Intermediate Representation
#[derive(Debug, Clone)]
pub struct HIR {
    pub functions: Vec<HirFunction>,
    pub globals: Vec<HirGlobal>,
    pub entry_point: Option<HirBlock>,
}

/// A function in the IR
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: HintType,
    pub body: HirBlock,
    pub is_builtin: bool,
}

/// A function parameter
#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub param_type: HintType,
}

/// A global variable
#[derive(Debug, Clone)]
pub struct HirGlobal {
    pub name: String,
    pub var_type: HintType,
    pub initial_value: Option<HirConstant>,
}

/// A constant value
#[derive(Debug, Clone)]
pub enum HirConstant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// A block of instructions
#[derive(Debug, Clone, Default)]
pub struct HirBlock {
    pub instructions: Vec<HirInstruction>,
}

/// IR instructions
#[derive(Debug, Clone)]
pub enum HirInstruction {
    /// Load a constant value
    LoadConst {
        dest: HirValue,
        value: HirConstant,
    },
    
    /// Load a variable
    LoadVar {
        dest: HirValue,
        source: String,
    },
    
    /// Store to a variable
    StoreVar {
        dest: String,
        source: HirValue,
    },
    
    /// Binary operation
    BinaryOp {
        dest: HirValue,
        op: HirBinaryOp,
        left: HirValue,
        right: HirValue,
        result_type: HintType,
    },
    
    /// Function call
    Call {
        dest: Option<HirValue>,
        function: String,
        args: Vec<HirValue>,
    },
    
    /// Return from function
    Return {
        value: Option<HirValue>,
    },
    
    /// Print a string
    Print {
        value: HirValue,
    },
    
    /// Jump to a label
    Jump {
        target: String,
    },
    
    /// Conditional jump
    Branch {
        condition: HirValue,
        if_true: String,
        if_false: String,
    },
    
    /// Label for jumps
    Label {
        name: String,
    },
    
    /// No operation
    Nop,
}

/// A value in the IR (virtual register)
#[derive(Debug, Clone)]
pub struct HirValue {
    pub id: usize,
    pub value_type: HintType,
}

impl HirValue {
    pub fn new(id: usize, value_type: HintType) -> Self {
        Self { id, value_type }
    }
}

/// Binary operations in IR
#[derive(Debug, Clone, Copy)]
pub enum HirBinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Builder for constructing HIR
pub struct HirBuilder {
    next_value_id: usize,
    next_label_id: usize,
}

impl HirBuilder {
    pub fn new() -> Self {
        Self {
            next_value_id: 0,
            next_label_id: 0,
        }
    }
    
    /// Create a new virtual register
    pub fn new_value(&mut self, value_type: HintType) -> HirValue {
        let id = self.next_value_id;
        self.next_value_id += 1;
        HirValue::new(id, value_type)
    }
    
    /// Create a new label
    pub fn new_label(&mut self) -> String {
        let id = self.next_label_id;
        self.next_label_id += 1;
        format!("bb{}", id)
    }
}

impl Default for HirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a typed program to HIR
pub fn lower_to_hir(program: &TypedProgram) -> HIR {
    let mut builder = HirBuilder::new();
    let mut globals = Vec::new();
    let mut entry_block = HirBlock::default();
    
    // Process statements
    for stmt in &program.statements {
        lower_statement(stmt, &mut builder, &mut globals, &mut entry_block);
    }
    
    // Add implicit return if needed
    entry_block.instructions.push(HirInstruction::Return { value: None });
    
    HIR {
        functions: Vec::new(),
        globals,
        entry_point: Some(entry_block),
    }
}

fn lower_statement(
    stmt: &TypedStatement,
    builder: &mut HirBuilder,
    globals: &mut Vec<HirGlobal>,
    block: &mut HirBlock,
) {
    match stmt {
        TypedStatement::Speak { text, .. } => {
            // Create string constant and print
            let str_value = builder.new_value(HintType::String);
            block.instructions.push(HirInstruction::LoadConst {
                dest: str_value.clone(),
                value: HirConstant::String(text.clone()),
            });
            block.instructions.push(HirInstruction::Print {
                value: str_value.clone(),
            });
        }
        
        TypedStatement::Remember { name, value, .. } => {
            // Create global variable
            globals.push(HirGlobal {
                name: name.clone(),
                var_type: HintType::Int(IntSize::I64),
                initial_value: Some(HirConstant::Int(*value as i64)),
            });
        }

        TypedStatement::RememberList { name, values, .. } => {
            // Create global variable for each value in the list
            for (i, value) in values.iter().enumerate() {
                globals.push(HirGlobal {
                    name: format!("{}_{}", name, i),
                    var_type: HintType::Int(IntSize::I64),
                    initial_value: Some(HirConstant::Int(*value as i64)),
                });
            }
        }
        
        TypedStatement::Halt { .. } => {
            // Exit with code 0
            let zero = builder.new_value(HintType::Int(IntSize::I64));
            block.instructions.push(HirInstruction::LoadConst {
                dest: zero.clone(),
                value: HirConstant::Int(0),
            });
            block.instructions.push(HirInstruction::Return {
                value: Some(zero.clone()),
            });
        }
    }
}
