//! Type Context and Environment
//! 
//! Manages type information during type checking and inference.

use std::collections::HashMap;
use crate::types::types::{Type, TypeVar, Substitution, GenericParam};
use crate::diagnostics::Span;

/// Type environment: maps variable names to their types
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    bindings: HashMap<String, Type>,
    /// Parent environment (for nested scopes)
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    /// Create a new child environment
    pub fn extend(&self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }
    
    /// Insert a binding
    pub fn insert(&mut self, name: String, ty: Type) -> Option<Type> {
        self.bindings.insert(name, ty)
    }
    
    /// Look up a type by name
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        // Check current scope first
        if let Some(ty) = self.bindings.get(name) {
            return Some(ty);
        }
        
        // Check parent scope
        if let Some(parent) = &self.parent {
            return parent.lookup(name);
        }
        
        None
    }
    
    /// Look up a type, cloning the result
    pub fn get(&self, name: &str) -> Option<Type> {
        self.lookup(name).cloned()
    }
    
    /// Check if a name is bound
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }
    
    /// Get all bindings in current scope
    pub fn bindings(&self) -> impl Iterator<Item = (&String, &Type)> {
        self.bindings.iter()
    }
    
    /// Apply substitution to all types in environment
    pub fn apply_substitution(&mut self, subst: &Substitution) {
        for ty in self.bindings.values_mut() {
            *ty = ty.apply_substitution(subst);
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) -> ScopeGuard {
        ScopeGuard::new(self)
    }
    
    /// Get the depth of the environment
    pub fn depth(&self) -> usize {
        match &self.parent {
            Some(p) => 1 + p.depth(),
            None => 0,
        }
    }
}

/// RAII guard for scope management
pub struct ScopeGuard<'a> {
    env: &'a mut TypeEnv,
    old_bindings: Vec<String>,
}

impl<'a> ScopeGuard<'a> {
    fn new(env: &'a mut TypeEnv) -> Self {
        Self {
            env,
            old_bindings: Vec::new(),
        }
    }
    
    /// Add a binding that will be removed when the guard is dropped
    pub fn bind(&mut self, name: &str, ty: Type) {
        if self.env.bindings.contains_key(name) {
            self.old_bindings.push(name.to_string());
        }
        self.env.insert(name.to_string(), ty);
    }
}

impl<'a> Drop for ScopeGuard<'a> {
    fn drop(&mut self) {
        // Remove bindings added in this scope
        for name in &self.old_bindings {
            self.env.bindings.remove(name);
        }
    }
}

/// Type context for type checking
#[derive(Debug)]
pub struct TypeContext {
    /// Current type environment
    pub env: TypeEnv,
    /// Current substitution
    pub substitution: Substitution,
    /// Next fresh type variable ID
    pub next_var_id: u64,
    /// Generic type parameters in scope
    pub generics: HashMap<String, GenericParam>,
    /// Expected return type (for function bodies)
    pub expected_return: Option<Type>,
    /// Error recovery mode
    pub error_recovery: bool,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            substitution: Substitution::new(),
            next_var_id: 0,
            generics: HashMap::new(),
            expected_return: None,
            error_recovery: false,
        }
    }
    
    /// Create a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let var = TypeVar {
            id: self.next_var_id,
            name: None,
        };
        self.next_var_id += 1;
        Type::Var(var)
    }
    
    /// Create a named type variable
    pub fn fresh_var_named(&mut self, name: &str) -> Type {
        let var = TypeVar {
            id: self.next_var_id,
            name: Some(name.to_string()),
        };
        self.next_var_id += 1;
        Type::Var(var)
    }
    
    /// Apply current substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        ty.apply_substitution(&self.substitution)
    }
    
    /// Insert a type binding
    pub fn insert(&mut self, name: &str, ty: Type) {
        self.env.insert(name.to_string(), ty);
    }
    
    /// Look up a type
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.env.lookup(name)
    }
    
    /// Enter a new scope, returns a guard
    pub fn enter_scope(&mut self) -> ScopeGuard {
        self.env.enter_scope()
    }
    
    /// Add a generic parameter
    pub fn add_generic(&mut self, param: GenericParam) {
        self.generics.insert(param.name.clone(), param);
    }
    
    /// Check if a name is a generic parameter
    pub fn is_generic(&self, name: &str) -> bool {
        self.generics.contains_key(name)
    }
    
    /// Get a generic parameter
    pub fn get_generic(&self, name: &str) -> Option<&GenericParam> {
        self.generics.get(name)
    }
    
    /// Set expected return type
    pub fn set_expected_return(&mut self, ty: Type) {
        self.expected_return = Some(ty);
    }
    
    /// Clear expected return type
    pub fn clear_expected_return(&mut self) {
        self.expected_return = None;
    }
    
    /// Get expected return type
    pub fn expected_return(&self) -> Option<&Type> {
        self.expected_return.as_ref()
    }
    
    /// Enable error recovery mode
    pub fn enable_error_recovery(&mut self) {
        self.error_recovery = true;
    }
    
    /// Disable error recovery mode
    pub fn disable_error_recovery(&mut self) {
        self.error_recovery = false;
    }
    
    /// Check if in error recovery mode
    pub fn is_error_recovery(&self) -> bool {
        self.error_recovery
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Type constraint for solving
#[derive(Debug, Clone)]
pub struct Constraint {
    pub left: Type,
    pub right: Type,
    pub span: Span,
    pub reason: ConstraintReason,
}

#[derive(Debug, Clone)]
pub enum ConstraintReason {
    /// Type equality
    Equality,
    /// Subtype relationship
    Subtype,
    /// Trait bound
    TraitBound(String),
    /// Function argument
    Argument(usize),
    /// Function return
    Return,
    /// Assignment
    Assignment,
}

impl Constraint {
    pub fn equality(left: Type, right: Type, span: Span) -> Self {
        Self {
            left,
            right,
            span,
            reason: ConstraintReason::Equality,
        }
    }
    
    pub fn subtype(left: Type, right: Type, span: Span) -> Self {
        Self {
            left,
            right,
            span,
            reason: ConstraintReason::Subtype,
        }
    }
    
    pub fn trait_bound(left: Type, trait_name: &str, span: Span) -> Self {
        Self {
            left,
            right: Type::Unknown,
            span,
            reason: ConstraintReason::TraitBound(trait_name.to_string()),
        }
    }
}

/// Constraint solver
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    solved: Substitution,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            solved: Substitution::new(),
        }
    }
    
    /// Add a constraint
    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    
    /// Solve all constraints
    pub fn solve(&mut self) -> Result<Substitution, String> {
        let mut changed = true;
        
        while changed && !self.constraints.is_empty() {
            changed = false;
            let mut remaining = Vec::new();
            
            for constraint in self.constraints.drain(..) {
                if self.solve_constraint(constraint)? {
                    changed = true;
                } else {
                    remaining.push(constraint);
                }
            }
            
            self.constraints = remaining;
        }
        
        if !self.constraints.is_empty() {
            return Err(format!("Unsolved constraints: {}", self.constraints.len()));
        }
        
        Ok(self.solved.clone())
    }
    
    /// Solve a single constraint
    fn solve_constraint(&mut self, constraint: Constraint) -> Result<bool, String> {
        let left = self.apply(&constraint.left);
        let right = self.apply(&constraint.right);
        
        match constraint.reason {
            ConstraintReason::Equality => {
                self.unify(&left, &right, constraint.span)?;
                Ok(true)
            }
            ConstraintReason::Subtype => {
                // For now, subtype is same as equality
                self.unify(&left, &right, constraint.span)?;
                Ok(true)
            }
            ConstraintReason::TraitBound(_) => {
                // Trait bounds are checked separately
                Ok(false)
            }
            ConstraintReason::Argument(_) | ConstraintReason::Return | ConstraintReason::Assignment => {
                self.unify(&left, &right, constraint.span)?;
                Ok(true)
            }
        }
    }
    
    /// Unify two types
    fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> Result<(), String> {
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        
        if t1 == t2 {
            return Ok(());
        }
        
        match (&t1, &t2) {
            (Type::Var(var), _) => {
                self.bind_var(*var, &t2, span)?;
            }
            (_, Type::Var(var)) => {
                self.bind_var(*var, &t1, span)?;
            }
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(format!("Function arity mismatch"));
                }
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b, span)?;
                }
                self.unify(r1, r2, span)?;
            }
            (Type::Primitive(p1), Type::Primitive(p2)) => {
                if p1 != p2 {
                    return Err(format!("Primitive type mismatch: {} vs {}", p1, p2));
                }
            }
            _ => {
                return Err(format!("Cannot unify {} and {}", t1, t2));
            }
        }
        
        Ok(())
    }
    
    /// Bind a type variable
    fn bind_var(&mut self, var: TypeVar, ty: &Type, span: Span) -> Result<(), String> {
        // Occurs check
        if ty.contains_var(var) {
            return Err(format!("Occurs check failed"));
        }
        
        self.solved.insert(var, ty.clone());
        Ok(())
    }
    
    /// Apply current substitution
    fn apply(&self, ty: &Type) -> Type {
        ty.apply_substitution(&self.solved)
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_env() {
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), Type::int());
        
        assert!(env.contains("x"));
        assert_eq!(env.get("x"), Some(Type::int()));
        assert!(env.get("y").is_none());
    }
    
    #[test]
    fn test_type_env_extend() {
        let mut parent = TypeEnv::new();
        parent.insert("x".to_string(), Type::int());
        
        let mut child = parent.extend();
        child.insert("y".to_string(), Type::string());
        
        assert!(child.contains("x"));
        assert!(child.contains("y"));
        assert!(!parent.contains("y"));
    }
    
    #[test]
    fn test_scope_guard() {
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), Type::int());
        
        {
            let mut guard = env.enter_scope();
            guard.bind("y", Type::string());
            assert!(env.contains("y"));
        }
        
        // y should be removed after scope ends
        assert!(!env.contains("y"));
        assert!(env.contains("x"));
    }
    
    #[test]
    fn test_constraint_solver() {
        let mut solver = ConstraintSolver::new();
        
        let var = Type::var();
        solver.add(Constraint::equality(var.clone(), Type::int(), Span::default()));
        
        let result = solver.solve();
        assert!(result.is_ok());
    }
}
