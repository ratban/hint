//! Runtime function helpers - Cranelift 0.110 compatible.

use cranelift_codegen::ir::{Value, types, InstBuilder, FuncRef};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::FuncId;

/// Runtime function registry
#[derive(Default)]
pub struct RuntimeFunctions {
    func_ids: std::collections::HashMap<String, FuncId>,
}

impl RuntimeFunctions {
    pub fn new() -> Self {
        Self { func_ids: std::collections::HashMap::new() }
    }
    
    pub fn register(&mut self, name: &str, id: FuncId) {
        self.func_ids.insert(name.to_string(), id);
    }
    
    pub fn get(&self, name: &str) -> Option<&FuncId> {
        self.func_ids.get(name)
    }
}

/// Helper for runtime calls
pub struct RuntimeCallBuilder<'a, 'b> {
    builder: &'a mut FunctionBuilder<'b>,
    runtime_funcs: &'a RuntimeFunctions,
    func_refs: std::collections::HashMap<FuncId, FuncRef>,
}

impl<'a, 'b> RuntimeCallBuilder<'a, 'b> {
    pub fn new(builder: &'a mut FunctionBuilder<'b>, runtime_funcs: &'a RuntimeFunctions) -> Self {
        Self { builder, runtime_funcs, func_refs: std::collections::HashMap::new() }
    }
    
    fn get_func_ref(&mut self, func_id: FuncId) -> FuncRef {
        if let Some(func_ref) = self.func_refs.get(&func_id) {
            return *func_ref;
        }
        let func_ref = self.builder.import_function(func_id);
        self.func_refs.insert(func_id, func_ref);
        func_ref
    }
    
    pub fn printf(&mut self, format_ptr: Value) -> Value {
        let id = self.runtime_funcs.get("printf").expect("printf not registered");
        let func_ref = self.get_func_ref(*id);
        let call = self.builder.ins().call(func_ref, &[format_ptr]);
        self.builder.inst_results(call)[0]
    }
    
    pub fn fputs(&mut self, str_ptr: Value, stream_ptr: Value) -> Value {
        let id = self.runtime_funcs.get("fputs").expect("fputs not registered");
        let func_ref = self.get_func_ref(*id);
        let call = self.builder.ins().call(func_ref, &[str_ptr, stream_ptr]);
        self.builder.inst_results(call)[0]
    }
    
    pub fn stdout(&mut self) -> Value {
        let id = self.runtime_funcs.get("stdout").expect("stdout not registered");
        let func_ref = self.get_func_ref(*id);
        let call = self.builder.ins().call(func_ref, &[]);
        self.builder.inst_results(call)[0]
    }
    
    pub fn malloc(&mut self, size: Value) -> Value {
        let id = self.runtime_funcs.get("malloc").expect("malloc not registered");
        let func_ref = self.get_func_ref(*id);
        let call = self.builder.ins().call(func_ref, &[size]);
        self.builder.inst_results(call)[0]
    }
    
    pub fn free(&mut self, ptr: Value) {
        let id = self.runtime_funcs.get("free").expect("free not registered");
        let func_ref = self.get_func_ref(*id);
        self.builder.ins().call(func_ref, &[ptr]);
    }
    
    pub fn memcpy(&mut self, dest: Value, src: Value, n: Value) -> Value {
        let id = self.runtime_funcs.get("memcpy").expect("memcpy not registered");
        let func_ref = self.get_func_ref(*id);
        let call = self.builder.ins().call(func_ref, &[dest, src, n]);
        self.builder.inst_results(call)[0]
    }
    
    pub fn memset(&mut self, ptr: Value, value: i32, n: Value) -> Value {
        let id = self.runtime_funcs.get("memset").expect("memset not registered");
        let func_ref = self.get_func_ref(*id);
        let val = self.builder.ins().iconst(types::I32, value as i64);
        let call = self.builder.ins().call(func_ref, &[ptr, val, n]);
        self.builder.inst_results(call)[0]
    }
    
    pub fn exit(&mut self, code: Value) {
        let id = self.runtime_funcs.get("exit").expect("exit not registered");
        let func_ref = self.get_func_ref(*id);
        self.builder.ins().call(func_ref, &[code]);
    }
    
    pub fn exit_with_code(&mut self, code: i32) {
        let val = self.builder.ins().iconst(types::I32, code as i64);
        self.exit(val);
    }
}

/// Print helper
pub fn print_string(builder: &mut FunctionBuilder, runtime_funcs: &RuntimeFunctions, string_ptr: Value) {
    let mut call = RuntimeCallBuilder::new(builder, runtime_funcs);
    let stdout = call.stdout();
    call.fputs(string_ptr, stdout);
}
