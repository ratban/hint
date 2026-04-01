//! Standard library for Hint.
//! 
//! Provides built-in functionality.

pub mod core;
pub mod io;
pub mod net;

#[cfg(feature = "wasm")]
pub mod wasm;

use std::collections::HashMap;
use crate::semantics::HintType;

/// Standard library function
#[derive(Clone)]
pub struct StdlibFunction {
    pub name: String,
    pub params: Vec<HintType>,
    pub return_type: HintType,
    pub description: &'static str,
}

/// Intrinsic function IDs
#[derive(Clone, Copy)]
pub enum IntrinsicId {
    // Core
    Print,
    PrintLn,
    Len,
    
    // Math
    Abs,
    Min,
    Max,
    Sqrt,
    
    // I/O
    ReadFile,
    WriteFile,
    ReadLine,
    
    // Network
    HttpGet,
    HttpPost,
    
    // WASM
    #[cfg(feature = "wasm")]
    DomQuerySelector,
    #[cfg(feature = "wasm")]
    DomSetInnerHtml,
}

/// Standard library registry
pub struct StdlibRegistry {
    functions: HashMap<String, StdlibFunction>,
}

impl StdlibRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }
    
    fn register_builtins(&mut self) {
        // Core functions
        self.register(StdlibFunction {
            name: "print".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Void,
            description: "Print to console",
        });
        
        self.register(StdlibFunction {
            name: "println".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Void,
            description: "Print with newline",
        });
    }
    
    fn register(&mut self, func: StdlibFunction) {
        self.functions.insert(func.name.clone(), func);
    }
    
    pub fn get(&self, name: &str) -> Option<&StdlibFunction> {
        self.functions.get(name)
    }
}

impl Default for StdlibRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get standard library functions
pub fn get_stdlib() -> StdlibRegistry {
    StdlibRegistry::new()
}
