//! C/C++ FFI Support
//! 
//! Provides interoperability with C and C++ code.

use super::types::{FFIType, FFISignature, FFIResult, FFIError};
use crate::ffi::FFIValue;
use std::collections::HashMap;
use std::path::Path;

/// C FFI manager
pub struct CFFI {
    /// Loaded modules (shared libraries)
    modules: HashMap<String, CModule>,
    /// Registered functions
    functions: HashMap<String, FFISignature>,
}

impl CFFI {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    
    /// Load a C module (shared library)
    pub fn load_module(&mut self, name: &str, path: &str) -> FFIResult<()> {
        let module = CModule::load(name, path)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }
    
    /// Register a C function
    pub fn register_function(&mut self, signature: FFISignature) {
        self.functions.insert(signature.name.clone(), signature);
    }
    
    /// Call a C function
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
        
        // In production, would use libloading or similar to call actual C function
        // For now, return a placeholder
        Ok(FFIValue::Null)
    }
    
    /// Generate C header bindings for a Hint module
    pub fn generate_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("/* Auto-generated C bindings for {} */\n\n", module_name));
        output.push_str("#ifndef HINT_{}_H\n", module_name.to_uppercase().replace('-', "_"));
        output.push_str("#define HINT_{}_H\n\n", module_name.to_uppercase().replace('-', "_"));
        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <stddef.h>\n\n");
        
        // Type definitions
        output.push_str("/* Hint types */\n");
        output.push_str("typedef int64_t hint_int_t;\n");
        output.push_str("typedef uint64_t hint_uint_t;\n");
        output.push_str("typedef double hint_float_t;\n");
        output.push_str("typedef const char* hint_string_t;\n\n");
        
        // Function declarations
        output.push_str("/* Function declarations */\n");
        for (name, sig) in &self.functions {
            output.push_str(&format!("{}\n", sig.as_c_declaration()));
        }
        
        output.push_str("\n#endif /* HINT_");
        output.push_str(&module_name.to_uppercase().replace('-', "_"));
        output.push_str("_H */\n");
        
        Ok(output)
    }
    
    /// Generate C++ header bindings
    pub fn generate_cpp_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("// Auto-generated C++ bindings for {}\n\n", module_name));
        output.push_str("#pragma once\n\n");
        output.push_str("#include <cstdint>\n");
        output.push_str("#include <string>\n");
        output.push_str("#include <vector>\n\n");
        output.push_str("extern \"C\" {\n\n");
        
        // C declarations
        for (name, sig) in &self.functions {
            output.push_str(&format!("    {}\n", sig.as_c_declaration()));
        }
        
        output.push_str("\n}\n\n");
        
        // C++ wrapper class
        output.push_str(&format!("class {} {{\n", module_name.to_string().to_uppercase().replace('-', "_")));
        output.push_str("public:\n");
        
        for (name, sig) in &self.functions {
            output.push_str(&self.generate_cpp_wrapper(name, sig));
        }
        
        output.push_str("};\n");
        
        Ok(output)
    }
    
    /// Generate C++ wrapper method
    fn generate_cpp_wrapper(&self, name: &str, sig: &FFISignature) -> String {
        let mut output = String::new();
        
        // Method signature
        let ret_type = sig.return_type.rust_type();
        output.push_str(&format!("    static {} {}(", 
            if ret_type == "()" { "void" } else { &ret_type },
            name
        ));
        
        let params: Vec<String> = sig.params.iter()
            .enumerate()
            .map(|(i, t)| format!("{} arg{}", t.rust_type(), i))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") {\n");
        
        // Call C function
        let call_args: Vec<String> = sig.params.iter()
            .enumerate()
            .map(|(i, _)| format!("arg{}", i))
            .collect();
        
        if sig.return_type == FFIType::Void {
            output.push_str(&format!("        ::{}({});\n", name, call_args.join(", ")));
        } else {
            output.push_str(&format!("        return ::{}({});\n", name, call_args.join(", ")));
        }
        
        output.push_str("    }\n\n");
        
        output
    }
    
    /// Get all registered functions
    pub fn functions(&self) -> &HashMap<String, FFISignature> {
        &self.functions
    }
}

impl Default for CFFI {
    fn default() -> Self {
        Self::new()
    }
}

/// C module information
#[derive(Debug, Clone)]
pub struct CModule {
    pub name: String,
    pub path: String,
    pub header_path: Option<String>,
    pub functions: Vec<FFISignature>,
    pub structs: Vec<CStruct>,
    pub defines: HashMap<String, String>,
}

impl CModule {
    pub fn load(name: &str, path: &str) -> FFIResult<Self> {
        // In production, would parse C header or use libloading
        Ok(Self {
            name: name.to_string(),
            path: path.to_string(),
            header_path: None,
            functions: Vec::new(),
            structs: Vec::new(),
            defines: HashMap::new(),
        })
    }
}

/// C struct definition
#[derive(Debug, Clone)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<(String, FFIType)>,
    pub packed: bool,
}

/// Parse C header file (simplified)
pub fn parse_c_header(content: &str) -> FFIResult<Vec<FFISignature>> {
    let mut functions = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and preprocessor
        if line.starts_with("//") || line.starts_with("#") || line.is_empty() {
            continue;
        }
        
        // Simple function declaration parsing
        if line.contains('(') && line.contains(')') && line.contains(';') {
            // This is a very simplified parser
            // In production, would use a proper C parser
        }
    }
    
    Ok(functions)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_c_ffi_creation() {
        let ffi = CFFI::new();
        assert!(ffi.modules.is_empty());
        assert!(ffi.functions.is_empty());
    }
    
    #[test]
    fn test_generate_c_header() {
        let mut ffi = CFFI::new();
        ffi.register_function(FFISignature::new(
            "add",
            vec![FFIType::Int, FFIType::Int],
            FFIType::Int,
        ));
        
        let header = ffi.generate_bindings("test").unwrap();
        assert!(header.contains("#ifndef HINT_TEST_H"));
        assert!(header.contains("int64_t add("));
    }
    
    #[test]
    fn test_generate_cpp_bindings() {
        let mut ffi = CFFI::new();
        ffi.register_function(FFISignature::new(
            "multiply",
            vec![FFIType::Int, FFIType::Int],
            FFIType::Int,
        ));
        
        let cpp = ffi.generate_cpp_bindings("math").unwrap();
        assert!(cpp.contains("extern \"C\""));
        assert!(cpp.contains("class MATH"));
    }
}
