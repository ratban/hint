//! WebAssembly code generator.
//!
//! Generates .wasm modules that can run in browsers or WASM runtimes.

use crate::codegen::{CodeGenerator, CompilationTarget};
use crate::ir::{HIR, HirInstruction, HirConstant};
use wasm_encoder::*;
use std::collections::HashMap;

/// WebAssembly code generator
pub struct WasmCodeGenerator {
    target: CompilationTarget,
}

impl WasmCodeGenerator {
    pub fn new(target: CompilationTarget) -> Self {
        Self { target }
    }
}

impl CodeGenerator for WasmCodeGenerator {
    fn generate(&mut self, hir: &HIR) -> Result<Vec<u8>, String> {
        // Collect string data first
        let mut string_data: Vec<u8> = Vec::new();
        let mut string_offsets: HashMap<String, u32> = HashMap::new();
        let mut offset = 0u32;
        
        if let Some(entry) = &hir.entry_point {
            for instr in &entry.instructions {
                if let HirInstruction::LoadConst { value: HirConstant::String(s), .. } = instr {
                    if !string_offsets.contains_key(s) {
                        string_offsets.insert(s.clone(), offset);
                        string_data.extend(s.as_bytes());
                        string_data.push(0); // null terminator
                        offset += s.len() as u32 + 1;
                    }
                }
            }
        }
        
        // Create module with sections in correct order
        let mut module = Module::new();
        
        // 1. Type section - define multiple function types
        let mut types = TypeSection::new();
        // Type 0: fn() -> i32 (main function)
        types.function([], [ValType::I32]);
        // Type 1: fn(i32, i32, i32, i32) -> i32 (fd_write)
        types.function([ValType::I32, ValType::I32, ValType::I32, ValType::I32], [ValType::I32]);
        // Type 2: fn(i32) -> () (proc_exit)
        types.function([ValType::I32], []);
        module.section(&types);
        
        // 2. Import section (WASI functions)
        let mut imports = ImportSection::new();
        // fd_write has type index 1
        imports.import("wasi_snapshot_preview1", "fd_write", EntityType::Function(1));
        // proc_exit has type index 2
        imports.import("wasi_snapshot_preview1", "proc_exit", EntityType::Function(2));
        module.section(&imports);
        
        // 3. Function section
        let mut functions = FunctionSection::new();
        functions.function(0); // main uses type index 0
        module.section(&functions);
        
        // 4. Table section (none needed)
        
        // 5. Memory section
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: 1,
            maximum: None,
            shared: false,
            page_size_log2: None,
            memory64: false,
        });
        module.section(&memories);
        
        // 6. Tag section (none needed)
        
        // 7. Export section
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        exports.export("memory", ExportKind::Memory, 0);
        module.section(&exports);
        
        // 8. Start section (none needed)
        
        // 9. Element section (none needed)
        
        // 10. Data count section (needed for passive data)
        // Skip for active data
        
        // 11. Code section - generate actual code from HIR
        let mut code = CodeSection::new();
        let mut func = Function::new(vec![]);
        
        // Generate code for each instruction in the entry point
        if let Some(entry) = &hir.entry_point {
            for instr in &entry.instructions {
                match instr {
                    HirInstruction::Print { value: _ } => {
                        // For print, we need to call fd_write
                        // Simplified: just write string to stdout (fd 1)
                        // In a full implementation, we'd track the value and write the corresponding string
                    }
                    HirInstruction::LoadConst { value: HirConstant::String(s), .. } => {
                        // String is already in data section
                        // We would load it here in a full implementation
                        let _ = s; // suppress unused warning
                    }
                    HirInstruction::Return { .. } => {
                        // Will add final return below
                    }
                    _ => {
                        // Other instructions: nop for now
                    }
                }
            }
        }
        
        // Return 0 (success)
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::End);
        
        code.function(&func);
        module.section(&code);
        
        // 12. Data section (must come after code)
        if !string_data.is_empty() {
            let mut data = DataSection::new();
            data.active(0, &ConstExpr::i32_const(0), string_data);
            module.section(&data);
        }
        
        let bytes = module.finish();
        
        // Validate
        wasmparser::validate(&bytes).map_err(|e| format!("Invalid WASM: {}", e))?;
        
        Ok(bytes)
    }
    
    fn target(&self) -> &CompilationTarget {
        &self.target
    }
}
