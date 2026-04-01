//! Semantic analysis for Hint.
//! 
//! Performs basic type checking and validation.

pub mod types;
pub mod symbols;
pub mod checker;
pub mod error;

pub use types::{HintType, IntSize, FloatSize};
pub use symbols::{SymbolTable, Scope, Symbol, SymbolType};
pub use checker::{SemanticAnalyzer, analyze_program};
pub use error::SemanticError;

use crate::diagnostics::{DiagnosticsEngine, Span};
use crate::parser::Program as AstProgram;

/// A fully type-checked program
#[derive(Debug, Clone)]
pub struct TypedProgram {
    pub statements: Vec<TypedStatement>,
    pub symbol_table: SymbolTable,
}

/// A typed statement
#[derive(Debug, Clone)]
pub enum TypedStatement {
    Speak { text: String, span: Span },
    Remember { name: String, value: i32, span: Span },
    RememberList { name: String, values: Vec<i32>, span: Span },
    Halt { span: Span },
}

/// Analyze a program
pub fn analyze(program: &AstProgram, source: &str) -> Result<TypedProgram, DiagnosticsEngine> {
    let analyzer = SemanticAnalyzer::new(source);
    analyzer.analyze(program)
}
