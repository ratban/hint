//! Semantic analysis errors.

use crate::diagnostics::{Diagnostic, Span};

/// Semantic analysis error types
#[derive(Debug, Clone)]
pub enum SemanticError {
    /// Undefined variable
    UndefinedVariable {
        name: String,
        span: Span,
    },
    /// Variable already defined
    VariableAlreadyDefined {
        name: String,
        span: Span,
        previous_span: Span,
    },
    /// Type mismatch
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },
    /// Invalid operation
    InvalidOperation {
        operation: String,
        types: Vec<String>,
        span: Span,
    },
    /// Invalid assignment
    InvalidAssignment {
        name: String,
        span: Span,
    },
    /// Missing return statement
    MissingReturn {
        span: Span,
    },
    /// Unknown function
    UnknownFunction {
        name: String,
        span: Span,
    },
    /// Wrong number of arguments
    WrongArgumentCount {
        function: String,
        expected: usize,
        found: usize,
        span: Span,
    },
    /// Invalid type conversion
    InvalidConversion {
        from: String,
        to: String,
        span: Span,
    },
}

impl SemanticError {
    pub fn to_diagnostic(&self, source: &str) -> Diagnostic {
        match self {
            SemanticError::UndefinedVariable { name, span } => {
                Diagnostic::error()
                    .with_message(format!("undefined variable: `{}`", name))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
                    .with_help(format!("declare the variable with `Keep the number ... in mind as the {}.`", name))
            }

            SemanticError::VariableAlreadyDefined { name, span, previous_span } => {
                Diagnostic::error()
                    .with_message(format!("variable `{}` is already defined", name))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
                    .with_note(format!("previous definition at {:?}", previous_span))
            }

            SemanticError::TypeMismatch { expected, found, span } => {
                Diagnostic::error()
                    .with_message("type mismatch")
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
                    .with_note(format!("expected: {}", expected))
                    .with_note(format!("found: {}", found))
            }

            SemanticError::InvalidOperation { operation, types, span } => {
                Diagnostic::error()
                    .with_message(format!("invalid operation: {}", operation))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
                    .with_note(format!("types involved: {}", types.join(", ")))
            }

            SemanticError::InvalidAssignment { name, span } => {
                Diagnostic::error()
                    .with_message(format!("cannot assign to `{}`", name))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
            }

            SemanticError::MissingReturn { span } => {
                Diagnostic::error()
                    .with_message("missing return statement")
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
            }

            SemanticError::UnknownFunction { name, span } => {
                Diagnostic::error()
                    .with_message(format!("unknown function: `{}`", name))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
            }

            SemanticError::WrongArgumentCount { function, expected, found, span } => {
                Diagnostic::error()
                    .with_message(format!("wrong number of arguments for `{}`", function))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
                    .with_note(format!("expected {} arguments, found {}", expected, found))
            }

            SemanticError::InvalidConversion { from, to, span } => {
                Diagnostic::error()
                    .with_message(format!("cannot convert {} to {}", from, to))
                    .with_span(span.start, span.end)
                    .with_source(source.to_string())
            }
        }
    }
}

/// Result type for semantic analysis
pub type SemanticResult<T> = Result<T, SemanticError>;
