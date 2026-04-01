//! Foreign Function Interface (FFI)
//! 
//! Provides interoperability with other languages:
//! - Rust
//! - C/C++
//! - JavaScript/TypeScript
//! - WebAssembly

pub mod rust;
pub mod c;
pub mod js;
pub mod wasm;
pub mod types;

pub use rust::{RustFFI, RustModule};
pub use c::{CFFI, CModule};
pub use js::{JSFFI, JSModule};
pub use wasm::{WasmFFI, WasmModule};
pub use types::{FFIType, FFIError, FFIResult, FFIValue};

use crate::diagnostics::{Diagnostic, DiagnosticsEngine};
use crate::semantics::HintType;

/// FFI configuration
#[derive(Debug, Clone)]
pub struct FFIConfig {
    /// Enable Rust FFI
    pub enable_rust: bool,
    /// Enable C FFI
    pub enable_c: bool,
    /// Enable JavaScript FFI
    pub enable_js: bool,
    /// Enable WebAssembly FFI
    pub enable_wasm: bool,
    /// FFI output directory
    pub output_dir: String,
    /// Generate bindings automatically
    pub auto_generate: bool,
}

impl Default for FFIConfig {
    fn default() -> Self {
        Self {
            enable_rust: true,
            enable_c: true,
            enable_js: true,
            enable_wasm: true,
            output_dir: "target/ffi".to_string(),
            auto_generate: true,
        }
    }
}

/// FFI manager for handling foreign function calls
pub struct FFIManager {
    config: FFIConfig,
    rust: RustFFI,
    c: CFFI,
    js: JSFFI,
    wasm: WasmFFI,
    diagnostics: DiagnosticsEngine,
}

impl FFIManager {
    pub fn new(config: FFIConfig) -> Self {
        Self {
            config,
            rust: RustFFI::new(),
            c: CFFI::new(),
            js: JSFFI::new(),
            wasm: WasmFFI::new(),
            diagnostics: DiagnosticsEngine::new(),
        }
    }
    
    /// Register a Rust module
    pub fn register_rust(&mut self, name: &str, path: &str) -> FFIResult<()> {
        if !self.config.enable_rust {
            return Err(FFIError::FeatureDisabled("Rust FFI".to_string()));
        }
        self.rust.load_module(name, path)?;
        Ok(())
    }
    
    /// Register a C module
    pub fn register_c(&mut self, name: &str, path: &str) -> FFIResult<()> {
        if !self.config.enable_c {
            return Err(FFIError::FeatureDisabled("C FFI".to_string()));
        }
        self.c.load_module(name, path)?;
        Ok(())
    }
    
    /// Register a JavaScript module
    pub fn register_js(&mut self, name: &str, path: &str) -> FFIResult<()> {
        if !self.config.enable_js {
            return Err(FFIError::FeatureDisabled("JavaScript FFI".to_string()));
        }
        self.js.load_module(name, path)?;
        Ok(())
    }
    
    /// Register a WebAssembly module
    pub fn register_wasm(&mut self, name: &str, path: &str) -> FFIResult<()> {
        if !self.config.enable_wasm {
            return Err(FFIError::FeatureDisabled("WebAssembly FFI".to_string()));
        }
        self.wasm.load_module(name, path)?;
        Ok(())
    }
    
    /// Call a foreign function
    pub fn call(&self, language: &str, function: &str, args: &[FFIValue]) -> FFIResult<FFIValue> {
        match language {
            "rust" => self.rust.call(function, args),
            "c" | "C" => self.c.call(function, args),
            "js" | "javascript" => self.js.call(function, args),
            "wasm" | "webassembly" => self.wasm.call(function, args),
            _ => Err(FFIError::UnknownLanguage(language.to_string())),
        }
    }
    
    /// Generate FFI bindings for a Hint module
    pub fn generate_bindings(&self, module_name: &str, target: &str) -> FFIResult<String> {
        match target {
            "rust" => self.rust.generate_bindings(module_name),
            "c" | "C" => self.c.generate_bindings(module_name),
            "js" | "javascript" => self.js.generate_bindings(module_name),
            "wasm" | "webassembly" => self.wasm.generate_bindings(module_name),
            _ => Err(FFIError::UnknownTarget(target.to_string())),
        }
    }
    
    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsEngine {
        &self.diagnostics
    }
}

/// FFI value for passing data between languages
#[derive(Debug, Clone)]
pub enum FFIValue {
    /// Null value
    Null,
    /// Boolean
    Bool(bool),
    /// Integer
    Int(i64),
    /// Float
    Float(f64),
    /// String
    String(String),
    /// Byte array
    Bytes(Vec<u8>),
    /// Array of values
    Array(Vec<FFIValue>),
    /// Object/map
    Object(std::collections::HashMap<String, FFIValue>),
    /// Pointer (for C interop)
    Pointer(u64),
}

impl FFIValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FFIValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FFIValue::Int(i) => Some(*i),
            FFIValue::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }
    
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FFIValue::Float(f) => Some(*f),
            FFIValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FFIValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            FFIValue::Bytes(b) => Some(b),
            FFIValue::String(s) => Some(s.as_bytes()),
            _ => None,
        }
    }
}

impl From<bool> for FFIValue {
    fn from(b: bool) -> Self {
        FFIValue::Bool(b)
    }
}

impl From<i64> for FFIValue {
    fn from(i: i64) -> Self {
        FFIValue::Int(i)
    }
}

impl From<i32> for FFIValue {
    fn from(i: i32) -> Self {
        FFIValue::Int(i as i64)
    }
}

impl From<f64> for FFIValue {
    fn from(f: f64) -> Self {
        FFIValue::Float(f)
    }
}

impl From<f32> for FFIValue {
    fn from(f: f32) -> Self {
        FFIValue::Float(f as f64)
    }
}

impl From<String> for FFIValue {
    fn from(s: String) -> Self {
        FFIValue::String(s)
    }
}

impl From<&str> for FFIValue {
    fn from(s: &str) -> Self {
        FFIValue::String(s.to_string())
    }
}

/// Convert Hint type to FFI type
pub fn hint_to_ffi_type(hint_type: &HintType) -> FFIType {
    match hint_type {
        HintType::Bool => FFIType::Bool,
        HintType::Int(_) | HintType::UInt(_) => FFIType::Int,
        HintType::Float(_) => FFIType::Float,
        HintType::String => FFIType::String,
        HintType::Array(elem, _) => FFIType::Array(Box::new(hint_to_ffi_type(elem))),
        _ => FFIType::Opaque,
    }
}

/// Convert FFI type to Hint type
pub fn ffi_to_hint_type(ffi_type: &FFIType) -> HintType {
    match ffi_type {
        FFIType::Bool => HintType::Bool,
        FFIType::Int => HintType::Int(crate::semantics::IntSize::I64),
        FFIType::Float => HintType::Float(crate::semantics::FloatSize::F64),
        FFIType::String => HintType::String,
        FFIType::Array(elem) => HintType::Array(
            Box::new(ffi_to_hint_type(elem)),
            None,
        ),
        _ => HintType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ffi_value_conversion() {
        let bool_val: FFIValue = true.into();
        assert_eq!(bool_val.as_bool(), Some(true));
        
        let int_val: FFIValue = 42i64.into();
        assert_eq!(int_val.as_int(), Some(42));
        
        let str_val: FFIValue = "hello".into();
        assert_eq!(str_val.as_string(), Some("hello"));
    }
    
    #[test]
    fn test_ffi_manager_creation() {
        let config = FFIConfig::default();
        let manager = FFIManager::new(config);
        
        assert!(manager.config.enable_rust);
        assert!(manager.config.enable_c);
        assert!(manager.config.enable_js);
        assert!(manager.config.enable_wasm);
    }
    
    #[test]
    fn test_type_conversion() {
        let hint_type = HintType::Int(crate::semantics::IntSize::I64);
        let ffi_type = hint_to_ffi_type(&hint_type);
        assert_eq!(ffi_type, FFIType::Int);
        
        let back = ffi_to_hint_type(&ffi_type);
        assert!(matches!(back, HintType::Int(_)));
    }
}
