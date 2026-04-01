//! Type system for the Hint language.

use std::fmt;

/// Hint type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintType {
    /// Void type (no value)
    Void,
    /// Signed integer types
    Int(IntSize),
    /// Unsigned integer types
    UInt(IntSize),
    /// Floating point types
    Float(FloatSize),
    /// String type
    String,
    /// Boolean type
    Bool,
    /// Array type
    Array(Box<HintType>, usize),
    /// Pointer type
    Pointer(Box<HintType>),
    /// Function type
    Function(Vec<HintType>, Box<HintType>),
    /// Unknown type (for error recovery)
    Unknown,
}

/// Integer size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSize {
    I8 = 1,
    I16 = 2,
    I32 = 4,
    I64 = 8,
}

impl IntSize {
    pub fn bytes(self) -> usize {
        self as usize
    }
    
    pub fn bits(self) -> usize {
        self.bytes() * 8
    }
}

/// Floating point size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSize {
    F32 = 4,
    F64 = 8,
}

impl FloatSize {
    pub fn bytes(self) -> usize {
        self as usize
    }
    
    pub fn bits(self) -> usize {
        self.bytes() * 8
    }
}

impl HintType {
    /// Get the default integer type (i64 for Hint)
    pub fn default_int() -> Self {
        HintType::Int(IntSize::I64)
    }
    
    /// Get the default float type (f64)
    pub fn default_float() -> Self {
        HintType::Float(FloatSize::F64)
    }
    
    /// Check if this is an integer type
    pub fn is_int(&self) -> bool {
        matches!(self, HintType::Int(_) | HintType::UInt(_))
    }
    
    /// Check if this is a float type
    pub fn is_float(&self) -> bool {
        matches!(self, HintType::Float(_))
    }
    
    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        self.is_int() || self.is_float()
    }
    
    /// Check if this is the void type
    pub fn is_void(&self) -> bool {
        matches!(self, HintType::Void)
    }
    
    /// Get the size in bytes of this type
    pub fn size(&self) -> Option<usize> {
        match self {
            HintType::Void => Some(0),
            HintType::Int(size) | HintType::UInt(size) => Some(size.bytes()),
            HintType::Float(size) => Some(size.bytes()),
            HintType::String => None, // Variable size
            HintType::Bool => Some(1),
            HintType::Array(elem, len) => elem.size().map(|s| s * len),
            HintType::Pointer(_) => Some(8), // 64-bit pointers
            HintType::Function(_, _) => None,
            HintType::Unknown => None,
        }
    }
    
    /// Get the alignment requirement in bytes
    pub fn alignment(&self) -> Option<usize> {
        self.size()
    }
    
    /// Check if two types are compatible
    pub fn is_compatible_with(&self, other: &HintType) -> bool {
        match (self, other) {
            // Same types are always compatible
            (a, b) if a == b => true,
            
            // Integers can be promoted to larger integers
            (HintType::Int(a), HintType::Int(b)) |
            (HintType::UInt(a), HintType::UInt(b)) => {
                a.bytes() <= b.bytes()
            }
            
            // Integers can be converted to floats
            (HintType::Int(_) | HintType::UInt(_), HintType::Float(_)) => true,
            
            // Unknown type is compatible with anything (for error recovery)
            (HintType::Unknown, _) | (_, HintType::Unknown) => true,
            
            _ => false,
        }
    }
}

impl fmt::Display for HintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HintType::Void => write!(f, "void"),
            HintType::Int(size) => write!(f, "i{}", size.bits()),
            HintType::UInt(size) => write!(f, "u{}", size.bits()),
            HintType::Float(size) => write!(f, "f{}", size.bits()),
            HintType::String => write!(f, "str"),
            HintType::Bool => write!(f, "bool"),
            HintType::Array(elem, len) => write!(f, "[{}; {}]", elem, len),
            HintType::Pointer(elem) => write!(f, "*{}", elem),
            HintType::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            HintType::Unknown => write!(f, "?"),
        }
    }
}

/// Type context for inference
#[derive(Debug, Default)]
pub struct TypeContext {
    pub expected: Option<HintType>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self { expected: None }
    }
    
    pub fn with_expected(mut self, ty: HintType) -> Self {
        self.expected = Some(ty);
        self
    }
}
