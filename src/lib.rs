//! Hint Compiler Library
//! 
//! A zero-dependency compiler for the Hint programming language.
//! Compiles conversational English to native executables and WebAssembly.

pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod ir;
pub mod codegen;
pub mod target;
pub mod stdlib;
pub mod diagnostics;
pub mod compiler;
pub mod lsp;

// Re-export main types
pub use lexer::{tokenize, Token};
pub use parser::{parse, Program as AstProgram};
pub use semantics::{SemanticAnalyzer, TypedProgram};
pub use ir::{HIR, HirBuilder, lower_to_hir};
pub use codegen::{CodeGenerator, CompilationTarget};
pub use target::TargetInfo;
pub use diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticsEngine};
pub use compiler::Compiler;

/// Compiler version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compile a Hint source file to the specified target
pub fn compile_file(
    input: &str,
    target: &CompilationTarget,
    output: &str,
) -> Result<(), String> {
    let compiler = Compiler::new(target.clone());
    compiler.compile_file(input, output)
}

/// Compile Hint source code string to bytes
pub fn compile_source(
    source: &str,
    target: &CompilationTarget,
) -> Result<Vec<u8>, String> {
    let compiler = Compiler::new(target.clone());
    compiler.compile_source(source)
}
