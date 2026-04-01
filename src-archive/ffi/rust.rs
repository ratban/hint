//! Rust FFI Support
//! 
//! Provides interoperability with Rust code.

use super::types::{FFIType, FFISignature, FFIResult, FFIError};
use crate::ffi::FFIValue;
use std::collections::HashMap;
use std::path::Path;

/// Rust FFI manager
pub struct RustFFI {
    /// Loaded modules
    modules: HashMap<String, RustModule>,
    /// Registered functions
    functions: HashMap<String, FFISignature>,
}

impl RustFFI {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    
    /// Load a Rust module
    pub fn load_module(&mut self, name: &str, path: &str) -> FFIResult<()> {
        let module = RustModule::load(name, path)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }
    
    /// Register a Rust function
    pub fn register_function(&mut self, signature: FFISignature) {
        self.functions.insert(signature.name.clone(), signature);
    }
    
    /// Call a Rust function
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
        
        // In production, would use dlopen/ffi to call actual Rust function
        // For now, return a placeholder
        Ok(FFIValue::Null)
    }
    
    /// Generate Rust bindings for a Hint module
    pub fn generate_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("// Auto-generated Rust bindings for {}\n\n", module_name));
        output.push_str("use std::ffi::CStr;\n");
        output.push_str("use std::os::raw::c_char;\n\n");
        
        output.push_str("#[link(name = \"hint_")
            .push_str(module_name)
            .push_str("\")]\n");
        output.push_str("extern \"C\" {\n");
        
        for (name, sig) in &self.functions {
            output.push_str(&format!("    {}\n", sig.as_rust_declaration("link_name")));
        }
        
        output.push_str("}\n\n");
        
        // Generate safe wrapper functions
        for (name, sig) in &self.functions {
            output.push_str(&self.generate_safe_wrapper(name, sig));
        }
        
        Ok(output)
    }
    
    /// Generate safe wrapper for a function
    fn generate_safe_wrapper(&self, name: &str, sig: &FFISignature) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("/// Safe wrapper for {}\n", name));
        output.push_str(&format!("pub fn {}_safe(", name));
        
        let params: Vec<String> = sig.params.iter()
            .enumerate()
            .map(|(i, t)| format!("arg{}: {}", i, t.rust_type()))
            .collect();
        output.push_str(&params.join(", "));
        
        if matches!(sig.return_type, FFIType::Void) {
            output.push_str(") {\n");
        } else {
            output.push_str(&format!(") -> {} {{\n", sig.return_type.rust_type()));
        }
        
        // Generate unsafe block
        output.push_str("    unsafe {\n");
        
        let call_args: Vec<String> = sig.params.iter()
            .enumerate()
            .map(|(i, _)| format!("arg{}", i))
            .collect();
        
        if matches!(sig.return_type, FFIType::Void) {
            output.push_str(&format!("        {}({});\n", name, call_args.join(", ")));
        } else {
            output.push_str(&format!("        {}({})\n", name, call_args.join(", ")));
        }
        
        output.push_str("    }\n");
        output.push_str("}\n\n");
        
        output
    }
    
    /// Get all registered functions
    pub fn functions(&self) -> &HashMap<String, FFISignature> {
        &self.functions
    }
}

impl Default for RustFFI {
    fn default() -> Self {
        Self::new()
    }
}

/// Rust module information
#[derive(Debug, Clone)]
pub struct RustModule {
    pub name: String,
    pub path: String,
    pub functions: Vec<FFISignature>,
    pub structs: Vec<RustStruct>,
    pub enums: Vec<RustEnum>,
}

impl RustModule {
    pub fn load(name: &str, path: &str) -> FFIResult<Self> {
        // In production, would parse Rust source or use cargo metadata
        Ok(Self {
            name: name.to_string(),
            path: path.to_string(),
            functions: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        })
    }
}

/// Rust struct definition
#[derive(Debug, Clone)]
pub struct RustStruct {
    pub name: String,
    pub fields: Vec<(String, FFIType)>,
}

/// Rust enum definition
#[derive(Debug, Clone)]
pub struct RustEnum {
    pub name: String,
    pub variants: Vec<String>,
}

/// Generate Rust FFI header file
pub fn generate_rust_header(functions: &[FFISignature]) -> String {
    let mut output = String::new();
    
    output.push_str("/* Auto-generated FFI header for Rust */\n\n");
    output.push_str("#ifndef HINT_FFI_H\n");
    output.push_str("#define HINT_FFI_H\n\n");
    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdbool.h>\n\n");
    output.push_str("#ifdef __cplusplus\n");
    output.push_str("extern \"C\" {\n");
    output.push_str("#endif\n\n");
    
    for sig in functions {
        output.push_str(&format!("{}\n", sig.as_c_declaration()));
    }
    
    output.push_str("\n#ifdef __cplusplus\n");
    output.push_str("}\n");
    output.push_str("#endif\n\n");
    output.push_str("#endif /* HINT_FFI_H */\n");
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_ffi_creation() {
        let ffi = RustFFI::new();
        assert!(ffi.modules.is_empty());
        assert!(ffi.functions.is_empty());
    }
    
    #[test]
    fn test_generate_rust_header() {
        let functions = vec![
            FFISignature::new("add", vec![FFIType::Int, FFIType::Int], FFIType::Int),
        ];
        
        let header = generate_rust_header(&functions);
        assert!(header.contains("HINT_FFI_H"));
        assert!(header.contains("int64_t add("));
    }
    
    #[test]
    fn test_safe_wrapper_generation() {
        let mut ffi = RustFFI::new();
        ffi.register_function(FFISignature::new(
            "multiply",
            vec![FFIType::Int, FFIType::Int],
            FFIType::Int,
        ));
        
        let bindings = ffi.generate_bindings("test").unwrap();
        assert!(bindings.contains("multiply_safe"));
        assert!(bindings.contains("unsafe"));
    }
}
