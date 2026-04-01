//! Type Checker
//! 
//! Performs type checking on typed Hint programs.

use crate::types::types::{Type, TypeContext, TypeEnv, Constraint, ConstraintSolver, Substitution};
use crate::semantics::{TypedProgram, TypedStatement, TypedExpression, HintType};
use crate::diagnostics::{Diagnostic, DiagnosticsEngine, Span};
use std::collections::HashMap;

/// Type check result
pub type TypeCheckResult = Result<(), TypeCheckError>;

/// Type check error
#[derive(Debug, Clone)]
pub struct TypeCheckError {
    pub kind: TypeCheckErrorKind,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TypeCheckErrorKind {
    /// Type mismatch
    Mismatch { expected: Type, found: Type },
    /// Unknown variable
    UnknownVariable(String),
    /// Unknown function
    UnknownFunction(String),
    /// Wrong argument count
    WrongArgCount { expected: usize, found: usize },
    /// Return type mismatch
    ReturnTypeMismatch { expected: Type, found: Type },
    /// Cannot infer type
    CannotInfer,
    /// Constraint unsatisfiable
    ConstraintUnsatisfiable(String),
    /// Circular reference
    CircularReference(String),
}

impl TypeCheckError {
    pub fn mismatch(expected: Type, found: Type, span: Span) -> Self {
        Self {
            kind: TypeCheckErrorKind::Mismatch { expected, found },
            message: format!("Type mismatch: expected {}, found {}", expected, found),
            span,
            notes: Vec::new(),
        }
    }
    
    pub fn unknown_variable(name: &str, span: Span) -> Self {
        Self {
            kind: TypeCheckErrorKind::UnknownVariable(name.to_string()),
            message: format!("Unknown variable: {}", name),
            span,
            notes: Vec::new(),
        }
    }
    
    pub fn unknown_function(name: &str, span: Span) -> Self {
        Self {
            kind: TypeCheckErrorKind::UnknownFunction(name.to_string()),
            message: format!("Unknown function: {}", name),
            span,
            notes: Vec::new(),
        }
    }
    
    pub fn wrong_arg_count(expected: usize, found: usize, span: Span) -> Self {
        Self {
            kind: TypeCheckErrorKind::WrongArgCount { expected, found },
            message: format!("Wrong number of arguments: expected {}, found {}", expected, found),
            span,
            notes: Vec::new(),
        }
    }
    
    pub fn to_diagnostic(&self, source: &str) -> Diagnostic {
        let mut diag = Diagnostic::error(&self.message)
            .with_span(self.span)
            .with_source(source.to_string());
        
        for note in &self.notes {
            diag = diag.with_note(note);
        }
        
        match &self.kind {
            TypeCheckErrorKind::Mismatch { expected, found } => {
                diag = diag.with_note(format!("Expected: {}", expected));
                diag = diag.with_note(format!("Found: {}", found));
            }
            _ => {}
        }
        
        diag
    }
}

/// Type checker for Hint programs
pub struct TypeChecker {
    context: TypeContext,
    diagnostics: DiagnosticsEngine,
    /// Function signatures
    functions: HashMap<String, FunctionSig>,
    /// Check mode
    strict: bool,
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
            diagnostics: DiagnosticsEngine::new(),
            functions: HashMap::new(),
            strict: false,
        }
    }
    
    /// Enable strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
    
    /// Register a function signature
    pub fn register_function(&mut self, sig: FunctionSig) {
        self.functions.insert(sig.name.clone(), sig);
    }
    
    /// Check a complete program
    pub fn check_program(&mut self, program: &TypedProgram) -> TypeCheckResult {
        // First pass: collect function signatures
        self.collect_signatures(program);
        
        // Second pass: type check all statements
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        
        Ok(())
    }
    
    /// Collect function signatures
    fn collect_signatures(&mut self, program: &TypedProgram) {
        // Built-in functions
        self.register_function(FunctionSig {
            name: "print".to_string(),
            params: vec![Type::string()],
            return_type: Type::unit(),
        });
        
        self.register_function(FunctionSig {
            name: "println".to_string(),
            params: vec![Type::string()],
            return_type: Type::unit(),
        });
    }
    
    /// Check a single statement
    pub fn check_statement(&mut self, stmt: &TypedStatement) -> TypeCheckResult {
        match stmt {
            TypedStatement::Speak { text, span } => {
                // Say statements are always valid (string literal)
                Ok(())
            }
            
            TypedStatement::Remember { name, value, value_type, span } => {
                // Check the value type matches the declared type
                let expr_type = self.check_expression(value)?;
                
                // Convert HintType to Type
                let declared_type: Type = value_type.clone().into();
                
                // Check compatibility
                if !self.types_compatible(&expr_type, &declared_type) {
                    self.diagnostics.emit(
                        TypeCheckError::mismatch(declared_type, expr_type, *span)
                            .to_diagnostic("")
                    );
                    if self.strict {
                        return Err(TypeCheckError::mismatch(declared_type, expr_type, *span));
                    }
                }
                
                // Add variable to context
                self.context.insert(name, declared_type);
                Ok(())
            }
            
            TypedStatement::Halt { span } => {
                // Halt is always valid
                Ok(())
            }
            
            _ => Ok(()),
        }
    }
    
    /// Check an expression and return its type
    pub fn check_expression(&mut self, expr: &TypedExpression) -> Result<Type, TypeCheckError> {
        match expr {
            TypedExpression::Int(n, ty) => {
                Ok(Type::int())
            }
            
            TypedExpression::Float(f, ty) => {
                Ok(Type::float())
            }
            
            TypedExpression::String(s) => {
                Ok(Type::string())
            }
            
            TypedExpression::Bool(b) => {
                Ok(Type::bool())
            }
            
            TypedExpression::Var(name, ty) => {
                // Look up variable in context
                if let Some(ty) = self.context.lookup(name) {
                    Ok(ty.clone())
                } else {
                    Err(TypeCheckError::unknown_variable(name, Span::default()))
                }
            }
            
            TypedExpression::Binary { op, left, right, result_type } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                
                // Check operand types are compatible
                if !self.types_compatible(&left_type, &right_type) {
                    self.diagnostics.emit(
                        TypeCheckError::mismatch(left_type, right_type, Span::default())
                            .to_diagnostic("")
                    );
                }
                
                Ok(result_type.clone().into())
            }
            
            TypedExpression::Call { name, args, return_type } => {
                // Check function exists
                if let Some(sig) = self.functions.get(name) {
                    // Check argument count
                    if args.len() != sig.params.len() {
                        return Err(TypeCheckError::wrong_arg_count(
                            sig.params.len(),
                            args.len(),
                            Span::default(),
                        ));
                    }
                    
                    // Check argument types
                    for (i, (arg, param_type)) in args.iter().zip(&sig.params).enumerate() {
                        let arg_type = self.check_expression(arg)?;
                        if !self.types_compatible(&arg_type, param_type) {
                            self.diagnostics.emit(
                                TypeCheckError::mismatch(param_type.clone(), arg_type, Span::default())
                                    .to_diagnostic("")
                            );
                        }
                    }
                    
                    Ok(sig.return_type.clone())
                } else {
                    Err(TypeCheckError::unknown_function(name, Span::default()))
                }
            }
        }
    }
    
    /// Check if two types are compatible
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        // Same types are compatible
        if t1 == t2 {
            return true;
        }
        
        // Integers can be coerced to floats
        if t1.is_integer() && t2.is_float() {
            return true;
        }
        
        // Unknown type is compatible with anything (for error recovery)
        if t1.is_unknown() || t2.is_unknown() {
            return true;
        }
        
        false
    }
    
    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsEngine {
        &self.diagnostics
    }
    
    /// Get the type context
    pub fn context(&self) -> &TypeContext {
        &self.context
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Type check a program (convenience function)
pub fn check(program: &TypedProgram) -> Result<(), DiagnosticsEngine> {
    let mut checker = TypeChecker::new();
    
    match checker.check_program(program) {
        Ok(()) => {
            if checker.diagnostics().has_errors() {
                Err(checker.diagnostics().clone())
            } else {
                Ok(())
            }
        }
        Err(e) => {
            checker.diagnostics.emit(e.to_diagnostic(""));
            Err(checker.diagnostics().clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantics::{IntSize, FloatSize};
    
    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new();
        assert!(!checker.strict);
    }
    
    #[test]
    fn test_types_compatible() {
        let checker = TypeChecker::new();
        
        // Same types
        assert!(checker.types_compatible(&Type::int(), &Type::int()));
        
        // Int to float coercion
        assert!(checker.types_compatible(&Type::int(), &Type::float()));
        
        // Different types
        assert!(!checker.types_compatible(&Type::int(), &Type::string()));
    }
    
    #[test]
    fn test_function_signature() {
        let sig = FunctionSig {
            name: "test".to_string(),
            params: vec![Type::int(), Type::string()],
            return_type: Type::bool(),
        };
        
        assert_eq!(sig.name, "test");
        assert_eq!(sig.params.len(), 2);
    }
}
