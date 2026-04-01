//! Type Inference Engine
//! 
//! Implements Hindley-Milner type inference with extensions.

use std::collections::HashMap;
use crate::types::types::{Type, TypeVar, Substitution, PrimitiveType};
use crate::semantics::{TypedStatement, TypedExpression, HintType};
use crate::diagnostics::Span;

/// Type inference result
pub type InferenceResult = Result<Type, InferenceError>;

/// Type inference error
#[derive(Debug, Clone)]
pub struct InferenceError {
    pub kind: InferenceErrorKind,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum InferenceErrorKind {
    /// Cannot unify two types
    UnificationFailed(Type, Type),
    /// Occurs check failed (infinite type)
    OccursCheck(TypeVar, Type),
    /// Unknown variable
    UnknownVariable(String),
    /// Cannot infer type
    CannotInfer,
    /// Constraint unsatisfiable
    ConstraintUnsatisfiable(String),
}

impl InferenceError {
    pub fn unification_failed(expected: Type, found: Type, span: Span) -> Self {
        Self {
            kind: InferenceErrorKind::UnificationFailed(expected, found),
            message: format!("Cannot unify types: {} and {}", expected, found),
            span,
        }
    }
    
    pub fn occurs_check(var: TypeVar, ty: Type, span: Span) -> Self {
        Self {
            kind: InferenceErrorKind::OccursCheck(var, ty),
            message: format!("Occurs check failed: {} appears in {}", var, ty),
            span,
        }
    }
    
    pub fn unknown_variable(name: &str, span: Span) -> Self {
        Self {
            kind: InferenceErrorKind::UnknownVariable(name.to_string()),
            message: format!("Unknown variable: {}", name),
            span,
        }
    }
}

/// Type inference context
#[derive(Debug, Default)]
pub struct InferenceContext {
    /// Type environment (variable -> type)
    pub env: HashMap<String, Type>,
    /// Current substitution
    pub substitution: Substitution,
    /// Next fresh type variable ID
    pub next_var: u64,
    /// Constraint stack
    pub constraints: Vec<(Type, Type, Span)>,
}

impl InferenceContext {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            substitution: Substitution::new(),
            next_var: 0,
            constraints: Vec::new(),
        }
    }
    
    /// Create a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let var = TypeVar {
            id: self.next_var,
            name: None,
        };
        self.next_var += 1;
        Type::Var(var)
    }
    
    /// Add a constraint
    pub fn add_constraint(&mut self, t1: Type, t2: Type, span: Span) {
        self.constraints.push((t1, t2, span));
    }
    
    /// Apply current substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        ty.apply_substitution(&self.substitution)
    }
    
    /// Unify two types
    pub fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> InferenceResult<()> {
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        
        match (&t1, &t2) {
            // Same types
            (a, b) if a == b => Ok(()),
            
            // Type variable on left
            (Type::Var(var), _) => {
                self.unify_var(*var, &t2, span)
            }
            
            // Type variable on right
            (_, Type::Var(var)) => {
                self.unify_var(*var, &t1, span)
            }
            
            // Function types
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(InferenceError::unification_failed(t1, t2, span));
                }
                
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2, span)?;
                }
                self.unify(ret1, ret2, span)?;
                Ok(())
            }
            
            // Array types
            (Type::Array(elem1, size1), Type::Array(elem2, size2)) => {
                if size1 != size2 {
                    return Err(InferenceError::unification_failed(t1, t2, span));
                }
                self.unify(elem1, elem2, span)
            }
            
            // Reference types
            (Type::Reference(inner1, mut1), Type::Reference(inner2, mut2)) => {
                if mut1 != mut2 {
                    return Err(InferenceError::unification_failed(t1, t2, span));
                }
                self.unify(inner1, inner2, span)
            }
            
            // Tuple types
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(InferenceError::unification_failed(t1, t2, span));
                }
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    self.unify(t1, t2, span)?;
                }
                Ok(())
            }
            
            // Primitive types
            (Type::Primitive(p1), Type::Primitive(p2)) => {
                if p1 == p2 {
                    Ok(())
                } else {
                    Err(InferenceError::unification_failed(t1, t2, span))
                }
            }
            
            // Constructor types
            (Type::Constructor(c1, args1), Type::Constructor(c2, args2)) => {
                if c1 != c2 || args1.len() != args2.len() {
                    return Err(InferenceError::unification_failed(t1, t2, span));
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2, span)?;
                }
                Ok(())
            }
            
            // Cannot unify
            _ => Err(InferenceError::unification_failed(t1, t2, span)),
        }
    }
    
    /// Unify a type variable with a type
    fn unify_var(&mut self, var: TypeVar, ty: &Type, span: Span) -> InferenceResult<()> {
        // Occurs check
        if ty.contains_var(var) {
            return Err(InferenceError::occurs_check(var, ty.clone(), span));
        }
        
        // If variable already has a substitution, unify with that
        if let Some(existing) = self.substitution.get(&var) {
            return self.unify(existing, ty, span);
        }
        
        // Substitute variable with type
        self.substitution.insert(var, ty.clone());
        Ok(())
    }
    
    /// Solve all constraints
    pub fn solve_constraints(&mut self) -> InferenceResult<()> {
        while let Some((t1, t2, span)) = self.constraints.pop() {
            self.unify(&t1, &t2, span)?;
        }
        Ok(())
    }
}

/// Type inferencer
pub struct TypeInferencer {
    context: InferenceContext,
}

impl TypeInferencer {
    pub fn new() -> Self {
        Self {
            context: InferenceContext::new(),
        }
    }
    
    /// Infer type of a statement
    pub fn infer_statement(&mut self, stmt: &TypedStatement, span: Span) -> InferenceResult {
        match stmt {
            TypedStatement::Speak { text, .. } => {
                // Say statements return unit
                Ok(Type::unit())
            }
            
            TypedStatement::Remember { name, value, value_type, .. } => {
                // Keep statements: infer type from value
                let ty = self.infer_expression(value, span)?;
                self.context.env.insert(name.clone(), ty.clone());
                Ok(ty)
            }
            
            TypedStatement::Halt { .. } => {
                // Halt has never type (doesn't return)
                Ok(Type::Never)
            }
            
            _ => Ok(Type::unit()),
        }
    }
    
    /// Infer type of an expression
    pub fn infer_expression(&mut self, expr: &TypedExpression, span: Span) -> InferenceResult {
        match expr {
            TypedExpression::Int(n, _) => {
                // Integer literals are i64 by default
                Ok(Type::Primitive(PrimitiveType::I64))
            }
            
            TypedExpression::Float(f, _) => {
                // Float literals are f64 by default
                Ok(Type::Primitive(PrimitiveType::F64))
            }
            
            TypedExpression::String(s) => {
                // String literals are str
                Ok(Type::Primitive(PrimitiveType::String))
            }
            
            TypedExpression::Bool(b) => {
                // Boolean literals are bool
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
            
            TypedExpression::Var(name, ty) => {
                // Look up variable in environment
                if let Some(ty) = self.context.env.get(name) {
                    Ok(ty.clone())
                } else {
                    Err(InferenceError::unknown_variable(name, span))
                }
            }
            
            TypedExpression::Binary { op, left, right, result_type } => {
                // Infer operand types
                let left_ty = self.infer_expression(left, span)?;
                let right_ty = self.infer_expression(right, span)?;
                
                // Operands must have same type
                self.context.add_constraint(left_ty, right_ty.clone(), span);
                
                // Result type depends on operator
                Ok(result_type.clone().into())
            }
            
            TypedExpression::Call { name, args, return_type } => {
                // Infer argument types
                for arg in args {
                    self.infer_expression(arg, span)?;
                }
                
                // Return type is known
                Ok(return_type.clone().into())
            }
        }
    }
    
    /// Solve constraints and return final substitution
    pub fn solve(&mut self) -> InferenceResult<Substitution> {
        self.context.solve_constraints()?;
        Ok(self.context.substitution.clone())
    }
    
    /// Get the current type environment
    pub fn env(&self) -> &HashMap<String, Type> {
        &self.context.env
    }
    
    /// Add a variable to the environment
    pub fn add_to_env(&mut self, name: &str, ty: Type) {
        self.context.env.insert(name.to_string(), ty);
    }
}

impl Default for TypeInferencer {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be inferred
pub trait Infer {
    fn infer(&self, inferencer: &mut TypeInferencer, span: Span) -> InferenceResult;
}

/// Helper to convert HintType to Type
impl From<HintType> for Type {
    fn from(hint_ty: HintType) -> Self {
        use crate::semantics::{IntSize, FloatSize};
        
        match hint_ty {
            HintType::Int(IntSize::I8) => Type::Primitive(PrimitiveType::I8),
            HintType::Int(IntSize::I16) => Type::Primitive(PrimitiveType::I16),
            HintType::Int(IntSize::I32) => Type::Primitive(PrimitiveType::I32),
            HintType::Int(IntSize::I64) => Type::Primitive(PrimitiveType::I64),
            HintType::UInt(IntSize::I8) => Type::Primitive(PrimitiveType::U8),
            HintType::UInt(IntSize::I16) => Type::Primitive(PrimitiveType::U16),
            HintType::UInt(IntSize::I32) => Type::Primitive(PrimitiveType::U32),
            HintType::UInt(IntSize::I64) => Type::Primitive(PrimitiveType::U64),
            HintType::Float(FloatSize::F32) => Type::Primitive(PrimitiveType::F32),
            HintType::Float(FloatSize::F64) => Type::Primitive(PrimitiveType::F64),
            HintType::Bool => Type::Primitive(PrimitiveType::Bool),
            HintType::String => Type::Primitive(PrimitiveType::String),
            HintType::Void => Type::unit(),
            _ => Type::Unknown,
        }
    }
}

/// Extension for Type to check if it contains a type variable
impl Type {
    pub fn contains_var(&self, var: TypeVar) -> bool {
        match self {
            Type::Var(v) => *v == var,
            Type::Function(params, ret) => {
                params.iter().any(|t| t.contains_var(var)) || ret.contains_var(var)
            }
            Type::Reference(inner, _) | Type::Array(inner, _) => {
                inner.contains_var(var)
            }
            Type::Constructor(_, args) | Type::Tuple(args) | Type::Struct(_, args) => {
                args.iter().any(|t| t.contains_var(var))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fresh_var() {
        let mut ctx = InferenceContext::new();
        let var1 = ctx.fresh_var();
        let var2 = ctx.fresh_var();
        
        assert!(var1.is_var());
        assert!(var2.is_var());
        assert_ne!(var1, var2);
    }
    
    #[test]
    fn test_unify_same_type() {
        let mut ctx = InferenceContext::new();
        let result = ctx.unify(&Type::int(), &Type::int(), Span::default());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_unify_different_types() {
        let mut ctx = InferenceContext::new();
        let result = ctx.unify(&Type::int(), &Type::string(), Span::default());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_unify_var() {
        let mut ctx = InferenceContext::new();
        let var = ctx.fresh_var();
        let result = ctx.unify(&var, &Type::int(), Span::default());
        assert!(result.is_ok());
        
        // Variable should now be substituted
        let applied = ctx.apply(&var);
        assert_eq!(applied, Type::int());
    }
    
    #[test]
    fn test_occurs_check() {
        let mut ctx = InferenceContext::new();
        let var = ctx.fresh_var();
        let array = Type::array(var.clone(), None);
        
        let result = ctx.unify(&var, &array, Span::default());
        assert!(result.is_err()); // Should fail occurs check
    }
}
