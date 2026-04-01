//! Core standard library functions.

use crate::stdlib::StdlibFunction;
use crate::semantics::{HintType, IntSize};

/// Initialize core stdlib functions
pub fn init() -> Vec<StdlibFunction> {
    vec![
        StdlibFunction {
            name: "print".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Void,
            description: "Print to console",
        },
        StdlibFunction {
            name: "println".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Void,
            description: "Print with newline",
        },
        StdlibFunction {
            name: "len".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Int(IntSize::I64),
            description: "Get string length",
        },
    ]
}
