//! Enhanced Diagnostics and Error Messages
//! 
//! This module provides rustc-quality error messages for the Hint compiler:
//! - Rich error formatting with source code snippets
//! - Error codes and explanations
//! - Suggestions for fixing errors
//! - Warning levels and lints
//! - Error recovery for multiple error reporting

pub mod diagnostic;
pub mod engine;
pub mod codes;
pub mod suggestions;
pub mod render;

pub use diagnostic::{Diagnostic, DiagnosticLevel, DiagnosticLabel, SubDiagnostic, Span};
pub use engine::{DiagnosticsEngine, DiagnosticId};
pub use codes::{ErrorCode, ErrorCategory};
pub use suggestions::{Suggestion, SuggestionStyle};
pub use render::{DiagnosticRenderer, TerminalRenderer};

use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::semantics::SemanticError;

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, DiagnosticsEngine>;

/// Create a lexer error diagnostic
pub fn lexer_error(error: LexError, source: &str) -> Diagnostic {
    Diagnostic::error()
        .with_code(ErrorCode::LexicalError.as_str())
        .with_message(&error.message)
        .with_span(error.position, error.position)
        .with_source(source)
        .with_help("Check for typos or invalid characters")
}

/// Create a parser error diagnostic
pub fn parser_error(error: ParseError, source: &str) -> Diagnostic {
    Diagnostic::error()
        .with_code(ErrorCode::UnexpectedToken.as_str())
        .with_message(&error.message)
        .with_span(error.position, error.position)
        .with_source(source)
        .with_help("Check the syntax of your statement")
}

/// Create a semantic error diagnostic
pub fn semantic_error(error: SemanticError, source: &str) -> Diagnostic {
    error.to_diagnostic(source)
}

/// Format a diagnostic for display
pub fn format_diagnostic(diag: &Diagnostic, source: &str) -> String {
    let renderer = TerminalRenderer::new();
    renderer.render(diag, source)
}

/// Print diagnostics to stderr
pub fn print_diagnostics(diagnostics: &[Diagnostic], source: &str) {
    let renderer = TerminalRenderer::new();
    for diag in diagnostics {
        eprintln!("{}", renderer.render(diag, source));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error()
            .with_code(ErrorCode::LexicalError)
            .with_message("Unexpected character");
        
        assert_eq!(diag.level, DiagnosticLevel::Error);
    }
    
    #[test]
    fn test_diagnostic_formatting() {
        let diag = Diagnostic::warning()
            .with_message("Unused variable");
        
        assert_eq!(diag.level, DiagnosticLevel::Warning);
    }
}
