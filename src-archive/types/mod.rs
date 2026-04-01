//! Advanced Type System with Type Inference
//! 
//! This module provides a sophisticated type system for Hint including:
//! - Type inference with Hindley-Milner algorithm
//! - Generic types and type parameters
//! - Type traits and constraints
//! - Type unification
//! - Type checking with detailed error messages

pub mod types;
pub mod infer;
pub mod unify;
pub mod traits;
pub mod checker;
pub mod context;

pub use types::{Type, TypeVar, TypeConstructor, GenericParam};
pub use infer::{TypeInferencer, InferenceResult, InferenceError};
pub use unify::{Unifier, Substitution};
pub use traits::{Trait, TraitBound, TraitImpl, TraitRef};
pub use checker::{TypeChecker, TypeCheckResult};
pub use context::{TypeContext, TypeEnv};

use crate::diagnostics::{Diagnostic, DiagnosticsEngine, Span};
use crate::semantics::types::{HintType, TypedProgram, TypedStatement};

/// Type system configuration
#[derive(Debug, Clone)]
pub struct TypeSystemConfig {
    /// Enable type inference
    pub enable_inference: bool,
    /// Enable generic types
    pub enable_generics: bool,
    /// Enable trait bounds
    pub enable_traits: bool,
    /// Strict mode (no implicit coercions)
    pub strict_mode: bool,
    /// Maximum type inference depth
    pub max_inference_depth: usize,
}

impl Default for TypeSystemConfig {
    fn default() -> Self {
        Self {
            enable_inference: true,
            enable_generics: true,
            enable_traits: true,
            strict_mode: false,
            max_inference_depth: 100,
        }
    }
}

/// Type system manager
pub struct TypeSystem {
    config: TypeSystemConfig,
    context: TypeContext,
    inferencer: TypeInferencer,
    checker: TypeChecker,
    diagnostics: DiagnosticsEngine,
}

impl TypeSystem {
    /// Create new type system
    pub fn new(config: TypeSystemConfig) -> Self {
        Self {
            context: TypeContext::new(),
            inferencer: TypeInferencer::new(),
            checker: TypeChecker::new(),
            config,
            diagnostics: DiagnosticsEngine::new(),
        }
    }
    
    /// Check types in a program
    pub fn check_program(&mut self, program: &TypedProgram) -> TypeCheckResult {
        self.checker.check_program(program, &mut self.context)
    }
    
    /// Infer types in an expression
    pub fn infer_type(&mut self, expr: &crate::semantics::TypedStatement) -> InferenceResult {
        if self.config.enable_inference {
            self.inferencer.infer_statement(expr, &mut self.context)
        } else {
            Ok(Type::Void)
        }
    }
    
    /// Get type of a variable
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.context.lookup(name)
    }
    
    /// Add variable to context
    pub fn add_variable(&mut self, name: &str, ty: Type) {
        self.context.insert(name.to_string(), ty);
    }
    
    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsEngine {
        &self.diagnostics
    }
}

/// Type error with detailed information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub message: String,
    pub span: Span,
    pub expected: Option<Type>,
    pub found: Option<Type>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    /// Type mismatch between expected and found
    Mismatch,
    /// Unknown type
    Unknown,
    /// Type variable not resolved
    UnresolvedVar,
    /// Circular type (infinite type)
    CircularType,
    /// Missing trait implementation
    MissingTrait,
    /// Generic type parameter mismatch
    GenericMismatch,
    /// Function argument count mismatch
    ArgCountMismatch,
    /// Return type mismatch
    ReturnMismatch,
    /// Field not found in struct
    FieldNotFound,
    /// Method not found
    MethodNotFound,
    /// Cannot infer type
    CannotInfer,
}

impl TypeError {
    pub fn mismatch(expected: Type, found: Type, span: Span) -> Self {
        Self {
            kind: TypeErrorKind::Mismatch,
            message: format!("Type mismatch: expected {}, found {}", expected, found),
            span,
            expected: Some(expected),
            found: Some(found),
            notes: Vec::new(),
        }
    }
    
    pub fn unknown(name: &str, span: Span) -> Self {
        Self {
            kind: TypeErrorKind::Unknown,
            message: format!("Unknown type: {}", name),
            span,
            expected: None,
            found: None,
            notes: Vec::new(),
        }
    }
    
    pub fn cannot_infer(span: Span) -> Self {
        Self {
            kind: TypeErrorKind::CannotInfer,
            message: "Cannot infer type".to_string(),
            span,
            expected: None,
            found: None,
            notes: vec!["Consider adding a type annotation".to_string()],
        }
    }
    
    pub fn to_diagnostic(&self, source: &str) -> Diagnostic {
        let mut diag = Diagnostic::error(&self.message)
            .with_span(self.span)
            .with_source(source.to_string());
        
        for note in &self.notes {
            diag = diag.with_note(note);
        }
        
        if let Some(expected) = &self.expected {
            diag = diag.with_note(format!("Expected type: {}", expected));
        }
        
        if let Some(found) = &self.found {
            diag = diag.with_note(format!("Found type: {}", found));
        }
        
        diag
    }
}

/// Type check result
pub type TypeResult<T> = Result<T, TypeError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_system_creation() {
        let config = TypeSystemConfig::default();
        let ts = TypeSystem::new(config);
        
        assert!(ts.config.enable_inference);
        assert!(ts.config.enable_generics);
    }
    
    #[test]
    fn test_type_error() {
        let error = TypeError::mismatch(
            Type::Int,
            Type::String,
            Span::new(0, 10),
        );
        
        assert!(matches!(error.kind, TypeErrorKind::Mismatch));
        assert!(error.message.contains("Type mismatch"));
    }
}
