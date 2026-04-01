//! Type Unification
//! 
//! Implements Robinson's unification algorithm for type inference.

use std::collections::HashMap;
use crate::types::types::{Type, TypeVar, Substitution};
use crate::diagnostics::Span;

/// Unification result
pub type UnifyResult = Result<Substitution, UnificationError>;

/// Unification error
#[derive(Debug, Clone)]
pub struct UnificationError {
    pub kind: UnificationErrorKind,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum UnificationErrorKind {
    /// Types cannot be unified
    Mismatch(Type, Type),
    /// Occurs check failed
    OccursCheck(TypeVar, Type),
    /// Variable not found
    VarNotFound(TypeVar),
}

/// Unifier for type inference
pub struct Unifier {
    /// Current substitution
    substitution: Substitution,
    /// Bindings (var -> type)
    bindings: HashMap<u64, Type>,
}

impl Unifier {
    pub fn new() -> Self {
        Self {
            substitution: Substitution::new(),
            bindings: HashMap::new(),
        }
    }
    
    /// Unify two types, returning a substitution
    pub fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> UnifyResult {
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        
        self.unify_types(&t1, &t2, span)?;
        Ok(self.substitution.clone())
    }
    
    /// Core unification algorithm
    fn unify_types(&mut self, t1: &Type, t2: &Type, span: Span) -> Result<(), UnificationError> {
        match (t1, t2) {
            // Same types
            (a, b) if a == b => Ok(()),
            
            // Type variable on left
            (Type::Var(var), _) => {
                self.unify_var(*var, t2, span)
            }
            
            // Type variable on right
            (_, Type::Var(var)) => {
                self.unify_var(*var, t1, span)
            }
            
            // Function types
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: format!("Function arity mismatch: {} vs {}", params1.len(), params2.len()),
                        span,
                    });
                }
                
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify_types(p1, p2, span)?;
                }
                self.unify_types(ret1, ret2, span)?;
                Ok(())
            }
            
            // Array types
            (Type::Array(elem1, size1), Type::Array(elem2, size2)) => {
                if size1 != size2 {
                    return Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: "Array size mismatch".to_string(),
                        span,
                    });
                }
                self.unify_types(elem1, elem2, span)
            }
            
            // Reference types
            (Type::Reference(inner1, mut1), Type::Reference(inner2, mut2)) => {
                if mut1 != mut2 {
                    return Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: "Mutability mismatch".to_string(),
                        span,
                    });
                }
                self.unify_types(inner1, inner2, span)
            }
            
            // Tuple types
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: "Tuple arity mismatch".to_string(),
                        span,
                    });
                }
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    self.unify_types(t1, t2, span)?;
                }
                Ok(())
            }
            
            // Constructor types
            (Type::Constructor(c1, args1), Type::Constructor(c2, args2)) => {
                if c1 != c2 {
                    return Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: format!("Type constructor mismatch: {} vs {}", c1, c2),
                        span,
                    });
                }
                if args1.len() != args2.len() {
                    return Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: "Type argument count mismatch".to_string(),
                        span,
                    });
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify_types(a1, a2, span)?;
                }
                Ok(())
            }
            
            // Primitive types
            (Type::Primitive(p1), Type::Primitive(p2)) => {
                if p1 == p2 {
                    Ok(())
                } else {
                    Err(UnificationError {
                        kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                        message: format!("Primitive type mismatch: {} vs {}", p1, p2),
                        span,
                    })
                }
            }
            
            // Cannot unify
            _ => Err(UnificationError {
                kind: UnificationErrorKind::Mismatch(t1.clone(), t2.clone()),
                message: format!("Cannot unify {} and {}", t1, t2),
                span,
            }),
        }
    }
    
    /// Unify a type variable with a type
    fn unify_var(&mut self, var: TypeVar, ty: &Type, span: Span) -> Result<(), UnificationError> {
        // Occurs check: prevent infinite types
        if ty.contains_var(var) {
            return Err(UnificationError {
                kind: UnificationErrorKind::OccursCheck(var, ty.clone()),
                message: format!("Occurs check failed: {} occurs in {}", var, ty),
                span,
            });
        }
        
        // Check if variable already has a binding
        if let Some(existing) = self.bindings.get(&var.id) {
            return self.unify_types(existing, ty, span);
        }
        
        // Create binding
        self.bindings.insert(var.id, ty.clone());
        self.substitution.insert(var, ty.clone());
        
        Ok(())
    }
    
    /// Apply current substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        ty.apply_substitution(&self.substitution)
    }
    
    /// Get the final substitution
    pub fn substitution(&self) -> &Substitution {
        &self.substitution
    }
    
    /// Clear the unifier state
    pub fn clear(&mut self) {
        self.substitution = Substitution::new();
        self.bindings.clear();
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Unify a list of type pairs
pub fn unify_all(pairs: &[(Type, Type)], span: Span) -> UnifyResult {
    let mut unifier = Unifier::new();
    
    for (t1, t2) in pairs {
        unifier.unify(t1, t2, span)?;
    }
    
    Ok(unifier.substitution())
}

/// Check if two types can be unified (without creating substitution)
pub fn can_unify(t1: &Type, t2: &Type) -> bool {
    let mut unifier = Unifier::new();
    unifier.unify(t1, t2, Span::default()).is_ok()
}

/// Compute the most general unifier (MGU) of two types
pub fn mgu(t1: &Type, t2: &Type, span: Span) -> UnifyResult {
    let mut unifier = Unifier::new();
    unifier.unify(t1, t2, span)?;
    Ok(unifier.substitution().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrimitiveType;
    
    #[test]
    fn test_unify_identical() {
        let mut unifier = Unifier::new();
        let result = unifier.unify(&Type::int(), &Type::int(), Span::default());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_unify_variables() {
        let mut unifier = Unifier::new();
        let var1 = Type::var();
        let var2 = Type::var();
        
        let result = unifier.unify(&var1, &var2, Span::default());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_unify_var_with_type() {
        let mut unifier = Unifier::new();
        let var = Type::var();
        
        let result = unifier.unify(&var, &Type::int(), Span::default());
        assert!(result.is_ok());
        
        let subst = result.unwrap();
        assert!(subst.contains(&TypeVar { id: var.get_id().unwrap(), name: None }));
    }
    
    #[test]
    fn test_unify_function_types() {
        let mut unifier = Unifier::new();
        let func1 = Type::function(vec![Type::int()], Type::string());
        let func2 = Type::function(vec![Type::var()], Type::var());
        
        let result = unifier.unify(&func1, &func2, Span::default());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_occurs_check_failure() {
        let mut unifier = Unifier::new();
        let var = Type::var();
        let array = Type::array(var.clone(), None);
        
        let result = unifier.unify(&var, &array, Span::default());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_unify_mismatch() {
        let mut unifier = Unifier::new();
        let result = unifier.unify(&Type::int(), &Type::string(), Span::default());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_mgu() {
        let var = Type::var();
        let result = mgu(&var, &Type::int(), Span::default());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_can_unify() {
        assert!(can_unify(&Type::int(), &Type::int()));
        assert!(!can_unify(&Type::int(), &Type::string()));
    }
}

// Helper method for Type to get var ID
impl Type {
    fn get_id(&self) -> Option<u64> {
        if let Type::Var(var) = self {
            Some(var.id)
        } else {
            None
        }
    }
}
