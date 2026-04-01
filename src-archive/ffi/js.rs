//! JavaScript/TypeScript FFI Support
//! 
//! Provides interoperability with JavaScript and TypeScript code.
//! Works with Node.js, Deno, and browser environments.

use super::types::{FFIType, FFISignature, FFIResult, FFIError};
use crate::ffi::FFIValue;
use std::collections::HashMap;

/// JavaScript FFI manager
pub struct JSFFI {
    /// Loaded modules
    modules: HashMap<String, JSModule>,
    /// Registered functions
    functions: HashMap<String, FFISignature>,
    /// Runtime context (for WASM/Node)
    context: Option<JSContext>,
}

impl JSFFI {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
            context: None,
        }
    }
    
    /// Load a JavaScript module
    pub fn load_module(&mut self, name: &str, path: &str) -> FFIResult<()> {
        let module = JSModule::load(name, path)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }
    
    /// Set JavaScript runtime context
    pub fn set_context(&mut self, context: JSContext) {
        self.context = Some(context);
    }
    
    /// Register a JavaScript function
    pub fn register_function(&mut self, signature: FFISignature) {
        self.functions.insert(signature.name.clone(), signature);
    }
    
    /// Call a JavaScript function
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
        
        // In production, would use V8, Deno, or Node.js API
        // For now, return a placeholder
        Ok(FFIValue::Null)
    }
    
    /// Generate TypeScript bindings for a Hint module
    pub fn generate_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("// Auto-generated TypeScript bindings for {}\n\n", module_name));
        output.push_str("/* eslint-disable */\n\n");
        
        // Type definitions
        output.push_str("/* Hint types */\n");
        output.push_str("export type HintInt = number;\n");
        output.push_str("export type HintUInt = number;\n");
        output.push_str("export type HintFloat = number;\n");
        output.push_str("export type HintString = string;\n");
        output.push_str("export type HintBytes = Uint8Array;\n\n");
        
        // Module declaration
        output.push_str(&format!("export declare module '{}' {{\n", module_name));
        
        for (name, sig) in &self.functions {
            output.push_str(&self.generate_ts_function(name, sig, 2));
        }
        
        output.push_str("}\n\n");
        
        // WASM imports
        output.push_str("/* WASM imports */\n");
        output.push_str(&format!("export const {}Imports = {{\n", module_name));
        
        for (name, sig) in &self.functions {
            output.push_str(&format!("  {}: ({}) => {{ /* FFI call */ }},\n", 
                name,
                sig.params.iter()
                    .enumerate()
                    .map(|(i, _)| format!("arg{}", i))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        
        output.push_str("};\n");
        
        Ok(output)
    }
    
    /// Generate TypeScript function declaration
    fn generate_ts_function(&self, name: &str, sig: &FFISignature, indent: usize) -> String {
        let indent_str = " ".repeat(indent);
        
        let params: Vec<String> = sig.params.iter()
            .enumerate()
            .map(|(i, t)| format!("arg{}: {}", i, self.ffi_type_to_ts(t)))
            .collect();
        
        let ret_type = self.ffi_type_to_ts(&sig.return_type);
        
        format!(
            "{}export function {}({}): {};\n",
            indent_str, name, params.join(", "), ret_type
        )
    }
    
    /// Convert FFI type to TypeScript type
    fn ffi_type_to_ts(&self, ffi_type: &FFIType) -> String {
        match ffi_type {
            FFIType::Void => "void".to_string(),
            FFIType::Bool => "boolean".to_string(),
            FFIType::Int | FFIType::UInt => "number".to_string(),
            FFIType::Float => "number".to_string(),
            FFIType::String => "string".to_string(),
            FFIType::Bytes => "Uint8Array".to_string(),
            FFIType::Array(inner) => format!("{}[]", self.ffi_type_to_ts(inner)),
            FFIType::Pointer(_) => "number".to_string(),
            FFIType::Function(params, ret) => {
                let params_str = params.iter()
                    .map(|p| self.ffi_type_to_ts(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) => {}", params_str, self.ffi_type_to_ts(ret))
            }
            _ => "any".to_string(),
        }
    }
    
    /// Generate JavaScript bindings (for Node.js)
    pub fn generate_js_bindings(&self, module_name: &str) -> FFIResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("// Auto-generated JavaScript bindings for {}\n\n", module_name));
        output.push_str("'use strict';\n\n");
        
        // FFI bridge
        output.push_str("const ffi = require('ffi-napi');\n");
        output.push_str("const ref = require('ref-napi');\n\n");
        
        // Type definitions
        output.push_str("const types = {\n");
        output.push_str("  void: ref.types.void,\n");
        output.push_str("  int: ref.types.int64,\n");
        output.push_str("  uint: ref.types.uint64,\n");
        output.push_str("  float: ref.types.double,\n");
        output.push_str("  string: ref.types.CString,\n");
        output.push_str("  pointer: ref.refType(ref.types.void),\n");
        output.push_str("};\n\n");
        
        // Library binding
        output.push_str(&format!("const lib = ffi.Library('lib{}', {{\n", module_name));
        
        for (name, sig) in &self.functions {
            output.push_str(&format!("  '{}': [types.{}, [", name, sig.return_type.js_type()));
            
            let param_types: Vec<String> = sig.params.iter()
                .map(|t| format!("types.{}", t.js_type()))
                .collect();
            output.push_str(&param_types.join(", "));
            
            output.push_str("]],\n");
        }
        
        output.push_str("});\n\n");
        
        // Export functions
        output.push_str("module.exports = lib;\n");
        
        Ok(output)
    }
    
    /// Get all registered functions
    pub fn functions(&self) -> &HashMap<String, FFISignature> {
        &self.functions
    }
}

impl Default for JSFFI {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript runtime context
#[derive(Debug, Clone)]
pub struct JSContext {
    pub runtime: String,
    pub version: String,
}

impl JSContext {
    pub fn node(version: &str) -> Self {
        Self {
            runtime: "node".to_string(),
            version: version.to_string(),
        }
    }
    
    pub fn deno(version: &str) -> Self {
        Self {
            runtime: "deno".to_string(),
            version: version.to_string(),
        }
    }
    
    pub fn browser() -> Self {
        Self {
            runtime: "browser".to_string(),
            version: "es2020".to_string(),
        }
    }
}

/// JavaScript module information
#[derive(Debug, Clone)]
pub struct JSModule {
    pub name: String,
    pub path: String,
    pub exports: Vec<String>,
    pub is_esm: bool,
}

impl JSModule {
    pub fn load(name: &str, path: &str) -> FFIResult<Self> {
        // In production, would load actual JS module
        Ok(Self {
            name: name.to_string(),
            path: path.to_string(),
            exports: Vec::new(),
            is_esm: false,
        })
    }
}

/// Generate Node.js addon (native module)
pub fn generate_node_addon(functions: &[FFISignature]) -> String {
    let mut output = String::new();
    
    output.push_str("#include <node_api.h>\n");
    output.push_str("#include <string>\n\n");
    
    output.push_str("napi_value Init(napi_env env, napi_value exports) {\n");
    
    for sig in functions {
        output.push_str(&format!("  // Function: {}\n", sig.name));
        output.push_str(&format!("  {{\n"));
        output.push_str(&format!("    napi_value fn;\n"));
        output.push_str(&format!("    napi_create_function(env, \"{}\", NAPI_AUTO_LENGTH, NULL, NULL, &fn);\n", sig.name));
        output.push_str(&format!("    napi_set_named_property(env, exports, \"{}\", fn);\n", sig.name));
        output.push_str(&format!("  }}\n\n"));
    }
    
    output.push_str("  return exports;\n");
    output.push_str("}\n\n");
    
    output.push_str("NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)\n");
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_js_ffi_creation() {
        let ffi = JSFFI::new();
        assert!(ffi.modules.is_empty());
        assert!(ffi.functions.is_empty());
    }
    
    #[test]
    fn test_ffi_type_to_ts() {
        let ffi = JSFFI::new();
        
        assert_eq!(ffi.ffi_type_to_ts(&FFIType::Int), "number");
        assert_eq!(ffi.ffi_type_to_ts(&FFIType::String), "string");
        assert_eq!(ffi.ffi_type_to_ts(&FFIType::Bool), "boolean");
        assert_eq!(ffi.ffi_type_to_ts(&FFIType::Array(Box::new(FFIType::Int))), "number[]");
    }
    
    #[test]
    fn test_generate_ts_bindings() {
        let mut ffi = JSFFI::new();
        ffi.register_function(FFISignature::new(
            "add",
            vec![FFIType::Int, FFIType::Int],
            FFIType::Int,
        ));
        
        let ts = ffi.generate_bindings("math").unwrap();
        assert!(ts.contains("export function add("));
        assert!(ts.contains("arg0: number"));
        assert!(ts.contains("arg1: number"));
    }
    
    #[test]
    fn test_js_context() {
        let node = JSContext::node("18.0.0");
        assert_eq!(node.runtime, "node");
        
        let deno = JSContext::deno("1.30.0");
        assert_eq!(deno.runtime, "deno");
        
        let browser = JSContext::browser();
        assert_eq!(browser.runtime, "browser");
    }
}
