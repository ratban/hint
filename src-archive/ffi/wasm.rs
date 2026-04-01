//! WebAssembly FFI Support
//! 
//! Provides interoperability with WebAssembly modules.

use super::types::{FFIType, FFISignature, FFIResult, FFIError};
use crate::ffi::FFIValue;
use std::collections::HashMap;

/// WebAssembly FFI manager
pub struct WasmFFI {
    /// Loaded modules
    modules: HashMap<String, WasmModule>,
    /// Registered functions
    functions: HashMap<String, FFISignature>,
    /// Memory instance
    memory: Option<WasmMemory>,
}

impl WasmFFI {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
            memory: None,
        }
    }
    
    /// Load a WebAssembly module
    pub fn load_module(&mut self, name: &str, path: &str) -> FFIResult<()> {
        let module = WasmModule::load(name, path)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }
    
    /// Set WebAssembly memory
    pub fn set_memory(&mut self, memory: WasmMemory) {
        self.memory = Some(memory);
    }
    
    /// Register a WebAssembly function
    pub fn register_function(&mut self, signature: FFISignature) {
        self.functions.insert(signature.name.clone(), signature);
    }
    
    /// Call a WebAssembly function
    pub fn call(&self, function: &str, args: &[FFIValue]) -> FFIResult<FFIValue> {
        let signature = self.functions.get(function)
            .ok_or_else(|| FFIError::FunctionNotFound(function.to_string()))?;
        
        // Validate argument count
        if args.len() != signature.params.len() {
            return Err(FFIError::InvalidArgCount {
                expected: signature.params.len(),
                found: args.len(),
            });
        }
        
        // In production, would use wasmtime or similar
        // For now, return a placeholder
        Ok(FFIValue::Null)
    }
    
    /// Generate WebAssembly Text Format (WAT) bindings
    pub fn generate_wat_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!(";; Auto-generated WAT bindings for {}\n\n", module_name));
        output.push_str("(module\n");
        output.push_str("  ;; Imports\n");
        
        // Generate imports
        for (name, sig) in &self.functions {
            output.push_str(&format!("  (import \"hint\" \"{}\" (func ${} (", name, name));
            
            // Parameters
            for param in &sig.params {
                output.push_str(&format!("(param {}) ", param.wasm_type()));
            }
            
            // Result
            if sig.return_type != FFIType::Void {
                output.push_str(&format!("(result {}) ", sig.return_type.wasm_type()));
            }
            
            output.push_str(")))\n");
        }
        
        output.push_str(")\n");
        
        Ok(output)
    }
    
    /// Generate JavaScript WASM bindings
    pub fn generate_js_wasm_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("// Auto-generated JS/WASM bindings for {}\n\n", module_name));
        output.push_str("export async function load{}(wasmPath) {{\n", module_name.to_string().to_uppercase().replace('-', "_"));
        output.push_str("  const wasmBytes = await fetch(wasmPath).then(r => r.arrayBuffer());\n");
        output.push_str("  const {{ instance }} = await WebAssembly.instantiate(wasmBytes, {{\n");
        output.push_str("    hint: {{\n");
        
        for (name, sig) in &self.functions {
            output.push_str(&format!("      {}: ({}) => {{ /* FFI call */ }},\n", 
                name,
                (0..sig.params.len()).map(|i| format!("arg{}", i)).collect::<Vec<_>>().join(", ")
            ));
        }
        
        output.push_str("    }\n");
        output.push_str("  }});\n");
        output.push_str("  return instance.exports;\n");
        output.push_str("}\n");
        
        Ok(output)
    }
    
    /// Get all registered functions
    pub fn functions(&self) -> &HashMap<String, FFISignature> {
        &self.functions
    }
}

impl Default for WasmFFI {
    fn default() -> Self {
        Self::new()
    }
}

/// WebAssembly module information
#[derive(Debug, Clone)]
pub struct WasmModule {
    pub name: String,
    pub path: String,
    pub exports: Vec<WasmExport>,
    pub imports: Vec<WasmImport>,
    pub memory_pages: u32,
}

impl WasmModule {
    pub fn load(name: &str, path: &str) -> FFIResult<Self> {
        // In production, would parse actual WASM binary
        Ok(Self {
            name: name.to_string(),
            path: path.to_string(),
            exports: Vec::new(),
            imports: Vec::new(),
            memory_pages: 1,
        })
    }
}

/// WebAssembly export
#[derive(Debug, Clone)]
pub struct WasmExport {
    pub name: String,
    pub kind: WasmExportKind,
}

/// WebAssembly export kind
#[derive(Debug, Clone, Copy)]
pub enum WasmExportKind {
    Function,
    Table,
    Memory,
    Global,
}

/// WebAssembly import
#[derive(Debug, Clone)]
pub struct WasmImport {
    pub module: String,
    pub name: String,
    pub kind: WasmExportKind,
}

/// WebAssembly memory
#[derive(Debug, Clone)]
pub struct WasmMemory {
    pub pages: u32,
    pub max_pages: Option<u32>,
    pub data: Vec<u8>,
}

impl WasmMemory {
    pub fn new(pages: u32) -> Self {
        Self {
            pages,
            max_pages: None,
            data: vec![0; (pages * 65536) as usize],
        }
    }
    
    pub fn read_i32(&self, offset: usize) -> i32 {
        i32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }
    
    pub fn write_i32(&mut self, offset: usize, value: i32) {
        let bytes = value.to_le_bytes();
        self.data[offset..offset + 4].copy_from_slice(&bytes);
    }
    
    pub fn read_string(&self, offset: usize) -> String {
        let mut end = offset;
        while end < self.data.len() && self.data[end] != 0 {
            end += 1;
        }
        String::from_utf8_lossy(&self.data[offset..end]).to_string()
    }
    
    pub fn write_string(&mut self, offset: usize, s: &str) {
        let bytes = s.as_bytes();
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
        self.data[offset + bytes.len()] = 0; // Null terminator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_ffi_creation() {
        let ffi = WasmFFI::new();
        assert!(ffi.modules.is_empty());
        assert!(ffi.functions.is_empty());
    }
    
    #[test]
    fn test_wasm_memory() {
        let mut memory = WasmMemory::new(1);
        
        memory.write_i32(0, 42);
        assert_eq!(memory.read_i32(0), 42);
        
        memory.write_string(100, "hello");
        assert_eq!(memory.read_string(100), "hello");
    }
    
    #[test]
    fn test_generate_wat_bindings() {
        let mut ffi = WasmFFI::new();
        ffi.register_function(FFISignature::new(
            "add",
            vec![FFIType::Int, FFIType::Int],
            FFIType::Int,
        ));
        
        let wat = ffi.generate_wat_bindings("math").unwrap();
        assert!(wat.contains("(module"));
        assert!(wat.contains("(import \"hint\" \"add\""));
    }
}
