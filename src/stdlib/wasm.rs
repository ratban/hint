//! WebAssembly-specific standard library functions.
//!
//! Provides DOM manipulation and Web API bindings for browser-based Hint programs.

#![cfg(feature = "wasm")]

use crate::stdlib::StdlibFunction;
use crate::semantics::HintType;

/// Initialize WASM stdlib functions
pub fn init() -> Vec<StdlibFunction> {
    vec![
        // DOM functions
        StdlibFunction {
            name: "query_selector".to_string(),
            params: vec![HintType::String],
            return_type: HintType::String,
            description: "Query DOM element by selector",
        },
        StdlibFunction {
            name: "set_inner_html".to_string(),
            params: vec![HintType::String, HintType::String],
            return_type: HintType::Void,
            description: "Set element inner HTML",
        },
        StdlibFunction {
            name: "add_event_listener".to_string(),
            params: vec![HintType::String, HintType::String],
            return_type: HintType::Void,
            description: "Add DOM event listener",
        },
    ]
}

/// WASM module imports for DOM access
pub const DOM_IMPORTS: &str = r#"
(import "env" "console_log" (func $console_log (param i32 i32)))
(import "env" "query_selector" (func $query_selector (param i32 i32) (result i32)))
(import "env" "set_inner_html" (func $set_inner_html (param i32 i32 i32 i32)))
"#;
