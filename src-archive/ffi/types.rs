//! FFI Type Definitions

use std::fmt;

/// FFI type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FFIType {
    /// Void (no return value)
    Void,
    /// Boolean
    Bool,
    /// Signed integer
    Int,
    /// Unsigned integer
    UInt,
    /// Floating point
    Float,
    /// String
    String,
    /// Byte array
    Bytes,
    /// Array
    Array(Box<FFIType>),
    /// Pointer
    Pointer(Box<FFIType>),
    /// Function
    Function(Vec<FFIType>, Box<FFIType>),
    /// Opaque type
    Opaque,
    /// Custom type
    Custom(String),
}

impl FFIType {
    pub fn size(&self) -> Option<usize> {
        match self {
            FFIType::Void => Some(0),
            FFIType::Bool => Some(1),
            FFIType::Int | FFIType::UInt | FFIType::Pointer(_) => Some(8),
            FFIType::Float => Some(8),
            FFIType::String => None, // Variable size
            FFIType::Bytes => None,
            FFIType::Array(elem) => elem.size(), // Simplified
            FFIType::Function(_, _) => Some(8), // Function pointer
            FFIType::Opaque => Some(8),
            FFIType::Custom(_) => None,
        }
    }
    
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            FFIType::Bool | FFIType::Int | FFIType::UInt | FFIType::Float
        )
    }
    
    pub fn is_pointer(&self) -> bool {
        matches!(self, FFIType::Pointer(_))
    }
    
    pub fn c_type(&self) -> &'static str {
        match self {
            FFIType::Void => "void",
            FFIType::Bool => "bool",
            FFIType::Int => "int64_t",
            FFIType::UInt => "uint64_t",
            FFIType::Float => "double",
            FFIType::String => "const char*",
            FFIType::Bytes => "const uint8_t*",
            FFIType::Pointer(_) => "void*",
            FFIType::Array(_) => "void*",
            FFIType::Function(_, _) => "void*",
            FFIType::Opaque => "void*",
            FFIType::Custom(_) => "void*",
        }
    }
    
    pub fn rust_type(&self) -> String {
        match self {
            FFIType::Void => "()".to_string(),
            FFIType::Bool => "bool".to_string(),
            FFIType::Int => "i64".to_string(),
            FFIType::UInt => "u64".to_string(),
            FFIType::Float => "f64".to_string(),
            FFIType::String => "String".to_string(),
            FFIType::Bytes => "Vec<u8>".to_string(),
            FFIType::Pointer(inner) => format!("*mut {}", inner.rust_type()),
            FFIType::Array(inner) => format!("Vec<{}>", inner.rust_type()),
            FFIType::Function(params, ret) => {
                let params_str = params.iter().map(|p| p.rust_type()).collect::<Vec<_>>().join(", ");
                format!("Box<dyn Fn({}) -> {}>", params_str, ret.rust_type())
            }
            FFIType::Opaque => "*mut std::ffi::c_void".to_string(),
            FFIType::Custom(name) => name.clone(),
        }
    }
    
    pub fn js_type(&self) -> &'static str {
        match self {
            FFIType::Void => "void",
            FFIType::Bool => "boolean",
            FFIType::Int | FFIType::UInt => "number",
            FFIType::Float => "number",
            FFIType::String => "string",
            FFIType::Bytes => "Uint8Array",
            FFIType::Pointer(_) => "number",
            FFIType::Array(_) => "Array",
            FFIType::Function(_, _) => "Function",
            FFIType::Opaque => "object",
            FFIType::Custom(_) => "any",
        }
    }
    
    pub fn wasm_type(&self) -> &'static str {
        match self {
            FFIType::Bool | FFIType::Int | FFIType::UInt | FFIType::Pointer(_) => "i32",
            FFIType::Float => "f64",
            FFIType::String => "i32", // Pointer to string
            FFIType::Void => "()",
            _ => "i32",
        }
    }
}

impl fmt::Display for FFIType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FFIType::Void => write!(f, "void"),
            FFIType::Bool => write!(f, "bool"),
            FFIType::Int => write!(f, "int"),
            FFIType::UInt => write!(f, "uint"),
            FFIType::Float => write!(f, "float"),
            FFIType::String => write!(f, "str"),
            FFIType::Bytes => write!(f, "bytes"),
            FFIType::Pointer(inner) => write!(f, "*{}", inner),
            FFIType::Array(inner) => write!(f, "[{}]", inner),
            FFIType::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            FFIType::Opaque => write!(f, "opaque"),
            FFIType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// FFI function signature
#[derive(Debug, Clone)]
pub struct FFISignature {
    pub name: String,
    pub params: Vec<FFIType>,
    pub return_type: FFIType,
    pub variadic: bool,
}

impl FFISignature {
    pub fn new(name: &str, params: Vec<FFIType>, return_type: FFIType) -> Self {
        Self {
            name: name.to_string(),
            params,
            return_type,
            variadic: false,
        }
    }
    
    pub fn as_c_declaration(&self) -> String {
        let params_str = if self.params.is_empty() {
            "void".to_string()
        } else {
            self.params
                .iter()
                .enumerate()
                .map(|(i, p)| format!("{} arg{}", p.c_type(), i))
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        format!("{} {}({});", self.return_type.c_type(), self.name, params_str)
    }
    
    pub fn as_rust_declaration(&self, extern_block: &str) -> String {
        let params_str = self.params
            .iter()
            .enumerate()
            .map(|(i, p)| format!("arg{}: {}", i, p.rust_type()))
            .collect::<Vec<_>>()
            .join(", ");
        
        let ret_str = if matches!(self.return_type, FFIType::Void) {
            String::new()
        } else {
            format!(" -> {}", self.return_type.rust_type())
        };
        
        format!(
            "    #[{}]\n    pub fn {}({}){};",
            extern_block, self.name, params_str, ret_str
        )
    }
    
    pub fn as_js_declaration(&self) -> String {
        let params_str = self.params
            .iter()
            .enumerate()
            .map(|(i, _)| format!("arg{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        
        format!("function {}({}) {{ /* FFI call */ }}", self.name, params_str)
    }
}

/// FFI error types
#[derive(Debug, Clone)]
pub enum FFIError {
    /// Feature is disabled
    FeatureDisabled(String),
    /// Unknown language
    UnknownLanguage(String),
    /// Unknown target
    UnknownTarget(String),
    /// Function not found
    FunctionNotFound(String),
    /// Type mismatch
    TypeMismatch { expected: String, found: String },
    /// Invalid argument count
    InvalidArgCount { expected: usize, found: usize },
    /// Conversion error
    ConversionError(String),
    /// Module not loaded
    ModuleNotFound(String),
    /// Linking error
    LinkError(String),
    /// Runtime error
    RuntimeError(String),
}

impl std::fmt::Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FFIError::FeatureDisabled(feature) => write!(f, "Feature disabled: {}", feature),
            FFIError::UnknownLanguage(lang) => write!(f, "Unknown language: {}", lang),
            FFIError::UnknownTarget(target) => write!(f, "Unknown target: {}", target),
            FFIError::FunctionNotFound(func) => write!(f, "Function not found: {}", func),
            FFIError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            FFIError::InvalidArgCount { expected, found } => {
                write!(f, "Invalid argument count: expected {}, found {}", expected, found)
            }
            FFIError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            FFIError::ModuleNotFound(module) => write!(f, "Module not found: {}", module),
            FFIError::LinkError(msg) => write!(f, "Link error: {}", msg),
            FFIError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for FFIError {}

/// FFI result type
pub type FFIResult<T> = Result<T, FFIError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ffi_type_display() {
        assert_eq!(format!("{}", FFIType::Int), "int");
        assert_eq!(format!("{}", FFIType::Pointer(Box::new(FFIType::Int))), "*int");
        assert_eq!(format!("{}", FFIType::Array(Box::new(FFIType::Bool))), "[bool]");
    }
    
    #[test]
    fn test_ffi_type_c_type() {
        assert_eq!(FFIType::Int.c_type(), "int64_t");
        assert_eq!(FFIType::String.c_type(), "const char*");
        assert_eq!(FFIType::Pointer(Box::new(FFIType::Int)).c_type(), "void*");
    }
    
    #[test]
    fn test_ffi_type_rust_type() {
        assert_eq!(FFIType::Int.rust_type(), "i64");
        assert_eq!(FFIType::String.rust_type(), "String");
        assert_eq!(FFIType::Pointer(Box::new(FFIType::Int)).rust_type(), "*mut i64");
    }
    
    #[test]
    fn test_ffi_signature_c_declaration() {
        let sig = FFISignature::new(
            "add",
            vec![FFIType::Int, FFIType::Int],
            FFIType::Int,
        );
        
        let decl = sig.as_c_declaration();
        assert!(decl.contains("int64_t add("));
        assert!(decl.contains("int64_t arg0"));
        assert!(decl.contains("int64_t arg1"));
    }
}
