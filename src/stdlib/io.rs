//! I/O standard library functions.

use crate::stdlib::StdlibFunction;
use crate::semantics::HintType;

/// Initialize I/O stdlib functions
pub fn init() -> Vec<StdlibFunction> {
    vec![
        StdlibFunction {
            name: "read_file".to_string(),
            params: vec![HintType::String],
            return_type: HintType::String,
            description: "Read file contents",
        },
        StdlibFunction {
            name: "write_file".to_string(),
            params: vec![HintType::String, HintType::String],
            return_type: HintType::Void,
            description: "Write to file",
        },
    ]
}
