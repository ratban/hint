//! Network standard library functions.

use crate::stdlib::StdlibFunction;
use crate::semantics::HintType;

/// Initialize network stdlib functions
pub fn init() -> Vec<StdlibFunction> {
    vec![
        StdlibFunction {
            name: "http_get".to_string(),
            params: vec![HintType::String],
            return_type: HintType::String,
            description: "HTTP GET request",
        },
        StdlibFunction {
            name: "http_post".to_string(),
            params: vec![HintType::String, HintType::String],
            return_type: HintType::String,
            description: "HTTP POST request",
        },
    ]
}
