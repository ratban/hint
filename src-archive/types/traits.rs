//! Type Traits and Bounds
//! 
//! Implements trait-based polymorphism for Hint.

use std::collections::HashMap;
use crate::types::types::{Type, TypeVar, GenericParam, TraitBound, TraitRef};

/// A trait definition
#[derive(Debug, Clone)]
pub struct Trait {
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub supertraits: Vec<TraitBound>,
    pub associated_types: Vec<String>,
    pub methods: Vec<TraitMethod>,
}

/// A trait method
#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub default_impl: Option<String>, // Source code of default implementation
}

/// Trait implementation for a type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_name: String,
    pub trait_params: Vec<Type>,
    pub impl_type: Type,
    pub impl_params: Vec<GenericParam>,
    pub methods: HashMap<String, String>, // method name -> implementation
}

/// Trait reference with resolved types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitRef {
    pub name: String,
    pub type_params: Vec<Type>,
}

impl Trait {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_params: Vec::new(),
            supertraits: Vec::new(),
            associated_types: Vec::new(),
            methods: Vec::new(),
        }
    }
    
    pub fn with_type_param(mut self, name: &str) -> Self {
        self.type_params.push(GenericParam {
            name: name.to_string(),
            bounds: Vec::new(),
            default: None,
        });
        self
    }
    
    pub fn with_supertrait(mut self, trait_name: &str) -> Self {
        self.supertraits.push(TraitBound {
            trait_name: trait_name.to_string(),
            type_params: Vec::new(),
        });
        self
    }
    
    pub fn with_method(mut self, name: &str, params: Vec<Type>, ret: Type) -> Self {
        self.methods.push(TraitMethod {
            name: name.to_string(),
            params,
            return_type: ret,
            default_impl: None,
        });
        self
    }
    
    pub fn with_default_method(mut self, name: &str, params: Vec<Type>, ret: Type, impl_code: &str) -> Self {
        self.methods.push(TraitMethod {
            name: name.to_string(),
            params,
            return_type: ret,
            default_impl: Some(impl_code.to_string()),
        });
        self
    }
}

impl TraitImpl {
    pub fn new(trait_name: &str, impl_type: Type) -> Self {
        Self {
            trait_name: trait_name.to_string(),
            trait_params: Vec::new(),
            impl_type,
            impl_params: Vec::new(),
            methods: HashMap::new(),
        }
    }
    
    pub fn with_method(mut self, name: &str, impl_code: &str) -> Self {
        self.methods.insert(name.to_string(), impl_code.to_string());
        self
    }
}

/// Trait registry for managing trait definitions and implementations
pub struct TraitRegistry {
    /// Defined traits
    traits: HashMap<String, Trait>,
    /// Trait implementations: (trait_name, impl_type) -> TraitImpl
    impls: HashMap<(String, String), TraitImpl>,
    /// Coherence check: ensure only one impl per (trait, type) pair
    coherence_checked: bool,
}

impl TraitRegistry {
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
            impls: HashMap::new(),
            coherence_checked: false,
        }
    }
    
    /// Register a trait definition
    pub fn register_trait(&mut self, trait_def: Trait) {
        self.traits.insert(trait_def.name.clone(), trait_def);
    }
    
    /// Register a trait implementation
    pub fn register_impl(&mut self, impl_def: TraitImpl) -> Result<(), String> {
        let key = (impl_def.trait_name.clone(), format!("{}", impl_def.impl_type));
        
        // Coherence check: ensure no duplicate implementations
        if self.impls.contains_key(&key) {
            return Err(format!(
                "Duplicate trait implementation for {} on {}",
                impl_def.trait_name, impl_def.impl_type
            ));
        }
        
        self.impls.insert(key, impl_def);
        Ok(())
    }
    
    /// Get a trait definition by name
    pub fn get_trait(&self, name: &str) -> Option<&Trait> {
        self.traits.get(name)
    }
    
    /// Find trait implementation for a type
    pub fn find_impl(&self, trait_name: &str, impl_type: &Type) -> Option<&TraitImpl> {
        let key = (trait_name.to_string(), format!("{}", impl_type));
        self.impls.get(&key)
    }
    
    /// Check if a type implements a trait
    pub fn implements(&self, trait_name: &str, impl_type: &Type) -> bool {
        self.find_impl(trait_name, impl_type).is_some()
    }
    
    /// Get all traits
    pub fn traits(&self) -> impl Iterator<Item = &Trait> {
        self.traits.values()
    }
    
    /// Get all implementations for a trait
    pub fn get_impls_for_trait(&self, trait_name: &str) -> impl Iterator<Item = &TraitImpl> {
        self.impls.values().filter(move |impl_def| impl_def.trait_name == trait_name)
    }
    
    /// Check coherence (one impl per trait/type pair)
    pub fn check_coherence(&self) -> Result<(), String> {
        // Already checked during registration
        Ok(())
    }
}

impl Default for TraitRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in traits for Hint
pub mod builtin_traits {
    use super::*;
    use crate::types::types::PrimitiveType;
    
    /// Create the Display trait
    pub fn display() -> Trait {
        Trait::new("Display")
            .with_method("to_string", vec![], Type::string())
    }
    
    /// Create the Debug trait
    pub fn debug() -> Trait {
        Trait::new("Debug")
            .with_method("debug", vec![], Type::string())
    }
    
    /// Create the Clone trait
    pub fn clone() -> Trait {
        let self_type = Type::var_named("Self");
        Trait::new("Clone")
            .with_type_param("Self")
            .with_method("clone", vec![], self_type)
    }
    
    /// Create the Copy trait
    pub fn copy() -> Trait {
        Trait::new("Copy")
            .with_supertrait("Clone")
    }
    
    /// Create the Eq trait
    pub fn eq() -> Trait {
        let self_type = Type::var_named("Self");
        Trait::new("Eq")
            .with_type_param("Self")
            .with_method("eq", vec![self_type.clone()], Type::bool())
            .with_method("ne", vec![self_type], Type::bool())
    }
    
    /// Create the Ord trait
    pub fn ord() -> Trait {
        let self_type = Type::var_named("Self");
        Trait::new("Ord")
            .with_type_param("Self")
            .with_supertrait("Eq")
            .with_method("cmp", vec![self_type.clone()], Type::int())
            .with_method("lt", vec![self_type.clone()], Type::bool())
            .with_method("le", vec![self_type.clone()], Type::bool())
            .with_method("gt", vec![self_type.clone()], Type::bool())
            .with_method("ge", vec![self_type], Type::bool())
    }
    
    /// Create the Add trait
    pub fn add() -> Trait {
        let self_type = Type::var_named("Self");
        let rhs_type = Type::var_named("Rhs");
        let output_type = Type::var_named("Output");
        
        Trait::new("Add")
            .with_type_param("Self")
            .with_type_param("Rhs")
            .with_type_param("Output")
            .with_method("add", vec![self_type.clone(), rhs_type], output_type)
    }
    
    /// Create the Sub trait
    pub fn sub() -> Trait {
        let self_type = Type::var_named("Self");
        let rhs_type = Type::var_named("Rhs");
        let output_type = Type::var_named("Output");
        
        Trait::new("Sub")
            .with_type_param("Self")
            .with_type_param("Rhs")
            .with_type_param("Output")
            .with_method("sub", vec![self_type.clone(), rhs_type], output_type)
    }
    
    /// Create the Mul trait
    pub fn mul() -> Trait {
        let self_type = Type::var_named("Self");
        let rhs_type = Type::var_named("Rhs");
        let output_type = Type::var_named("Output");
        
        Trait::new("Mul")
            .with_type_param("Self")
            .with_type_param("Rhs")
            .with_type_param("Output")
            .with_method("mul", vec![self_type.clone(), rhs_type], output_type)
    }
    
    /// Create the Div trait
    pub fn div() -> Trait {
        let self_type = Type::var_named("Self");
        let rhs_type = Type::var_named("Rhs");
        let output_type = Type::var_named("Output");
        
        Trait::new("Div")
            .with_type_param("Self")
            .with_type_param("Rhs")
            .with_type_param("Output")
            .with_method("div", vec![self_type.clone(), rhs_type], output_type)
    }
    
    /// Create the Iterator trait
    pub fn iterator() -> Trait {
        let item_type = Type::var_named("Item");
        Trait::new("Iterator")
            .with_type_param("Item")
            .with_method("next", vec![], Type::option(item_type))
    }
    
    /// Create the IntoIterator trait
    pub fn into_iterator() -> Trait {
        let item_type = Type::var_named("Item");
        let iter_type = Type::var_named("Iter");
        
        Trait::new("IntoIterator")
            .with_type_param("Item")
            .with_type_param("Iter")
            .with_method("into_iter", vec![], iter_type)
    }
    
    /// Create the Default trait
    pub fn default() -> Trait {
        let self_type = Type::var_named("Self");
        Trait::new("Default")
            .with_type_param("Self")
            .with_method("default", vec![], self_type)
    }
    
    /// Create the From trait
    pub fn from() -> Trait {
        let self_type = Type::var_named("Self");
        let from_type = Type::var_named("From");
        
        Trait::new("From")
            .with_type_param("Self")
            .with_type_param("From")
            .with_method("from", vec![from_type], self_type)
    }
}

/// Create a registry with all built-in traits
pub fn create_builtin_registry() -> TraitRegistry {
    use builtin_traits::*;
    
    let mut registry = TraitRegistry::new();
    
    registry.register_trait(display());
    registry.register_trait(debug());
    registry.register_trait(clone());
    registry.register_trait(copy());
    registry.register_trait(eq());
    registry.register_trait(ord());
    registry.register_trait(add());
    registry.register_trait(sub());
    registry.register_trait(mul());
    registry.register_trait(div());
    registry.register_trait(iterator());
    registry.register_trait(into_iterator());
    registry.register_trait(default());
    registry.register_trait(from());
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trait_creation() {
        let trait_def = Trait::new("Display")
            .with_method("to_string", vec![], Type::string());
        
        assert_eq!(trait_def.name, "Display");
        assert_eq!(trait_def.methods.len(), 1);
    }
    
    #[test]
    fn test_trait_impl() {
        let impl_def = TraitImpl::new("Display", Type::int())
            .with_method("to_string", "self.to_string()");
        
        assert_eq!(impl_def.trait_name, "Display");
        assert!(impl_def.methods.contains_key("to_string"));
    }
    
    #[test]
    fn test_trait_registry() {
        let mut registry = TraitRegistry::new();
        
        let trait_def = Trait::new("Test");
        registry.register_trait(trait_def);
        
        assert!(registry.get_trait("Test").is_some());
        assert!(registry.get_trait("NotFound").is_none());
    }
    
    #[test]
    fn test_coherence_check() {
        let mut registry = TraitRegistry::new();
        
        let impl1 = TraitImpl::new("Display", Type::int())
            .with_method("to_string", "int_to_string()");
        registry.register_impl(impl1).unwrap();
        
        let impl2 = TraitImpl::new("Display", Type::int())
            .with_method("to_string", "another_int_to_string()");
        let result = registry.register_impl(impl2);
        
        assert!(result.is_err()); // Should fail coherence check
    }
    
    #[test]
    fn test_builtin_traits() {
        let registry = create_builtin_registry();
        
        assert!(registry.get_trait("Display").is_some());
        assert!(registry.get_trait("Clone").is_some());
        assert!(registry.get_trait("Add").is_some());
    }
}
