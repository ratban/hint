//! Core Type Definitions
//! 
//! Defines the Type enum and related type system primitives.

use std::fmt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier generator for type variables
static TYPE_VAR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A type in the Hint type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Primitive types
    Primitive(PrimitiveType),
    
    /// Type variable (for inference)
    Var(TypeVar),
    
    /// Type constructor (generic types)
    Constructor(TypeConstructor, Vec<Type>),
    
    /// Function type
    Function(Vec<Type>, Box<Type>),
    
    /// Reference/Pointer type
    Reference(Box<Type>, Mutability),
    
    /// Array type
    Array(Box<Type>, Option<usize>),
    
    /// Tuple type
    Tuple(Vec<Type>),
    
    /// Struct/Record type
    Struct(String, Vec<Type>),
    
    /// Trait object type
    TraitObject(TraitRef),
    
    /// Never type (bottom type)
    Never,
    
    /// Unknown type (error type)
    Unknown,
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    // Integers
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    
    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    
    // Floating point
    F32,
    F64,
    
    // Other primitives
    Bool,
    Char,
    String,
    Unit,
}

/// Type variable for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: u64,
    pub name: Option<String>,
}

impl TypeVar {
    pub fn new() -> Self {
        Self {
            id: TYPE_VAR_COUNTER.fetch_add(1, Ordering::SeqCst),
            name: None,
        }
    }
    
    pub fn named(name: &str) -> Self {
        Self {
            id: TYPE_VAR_COUNTER.fetch_add(1, Ordering::SeqCst),
            name: Some(name.to_string()),
        }
    }
}

impl Default for TypeVar {
    fn default() -> Self {
        Self::new()
    }
}

/// Type constructor for generic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeConstructor {
    /// Option<T>
    Option,
    /// Result<T, E>
    Result,
    /// Vec<T>
    Vec,
    /// HashMap<K, V>
    HashMap,
    /// Box<T>
    Box,
    /// Rc<T>
    Rc,
    /// Arc<T>
    Arc,
    /// Fn(args) -> ret
    Fn,
    /// Custom type constructor
    Custom(&'static str),
}

/// Generic type parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
    pub default: Option<Type>,
}

/// Trait reference
#[derive(Debug, Clone)]
pub struct TraitRef {
    pub name: String,
    pub type_params: Vec<Type>,
}

/// Trait bound
#[derive(Debug, Clone)]
pub struct TraitBound {
    pub trait_name: String,
    pub type_params: Vec<Type>,
}

/// Mutability annotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Immutable,
    Mutable,
}

impl Type {
    /// Create a new type variable
    pub fn var() -> Self {
        Type::Var(TypeVar::new())
    }
    
    /// Create a named type variable
    pub fn var_named(name: &str) -> Self {
        Type::Var(TypeVar::named(name))
    }
    
    /// Create i64 type
    pub fn int() -> Self {
        Type::Primitive(PrimitiveType::I64)
    }
    
    /// Create f64 type
    pub fn float() -> Self {
        Type::Primitive(PrimitiveType::F64)
    }
    
    /// Create bool type
    pub fn bool() -> Self {
        Type::Primitive(PrimitiveType::Bool)
    }
    
    /// Create string type
    pub fn string() -> Self {
        Type::Primitive(PrimitiveType::String)
    }
    
    /// Create unit type
    pub fn unit() -> Self {
        Type::Primitive(PrimitiveType::Unit)
    }
    
    /// Create void type (alias for unit)
    pub fn void() -> Self {
        Type::unit()
    }
    
    /// Create function type
    pub fn function(params: Vec<Type>, ret: Type) -> Self {
        Type::Function(params, Box::new(ret))
    }
    
    /// Create array type
    pub fn array(elem: Type, size: Option<usize>) -> Self {
        Type::Array(Box::new(elem), size)
    }
    
    /// Create reference type
    pub fn reference(ty: Type, mutable: bool) -> Self {
        Type::Reference(Box::new(ty), if mutable { Mutability::Mutable } else { Mutability::Immutable })
    }
    
    /// Create Option type
    pub fn option(ty: Type) -> Self {
        Type::Constructor(TypeConstructor::Option, vec![ty])
    }
    
    /// Create Result type
    pub fn result(ok: Type, err: Type) -> Self {
        Type::Constructor(TypeConstructor::Result, vec![ok, err])
    }
    
    /// Create Vec type
    pub fn vec(ty: Type) -> Self {
        Type::Constructor(TypeConstructor::Vec, vec![ty])
    }
    
    /// Check if this is a type variable
    pub fn is_var(&self) -> bool {
        matches!(self, Type::Var(_))
    }
    
    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Primitive(_))
    }
    
    /// Check if this is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Primitive(PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | 
            PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::Isize |
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | 
            PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::Usize))
    }
    
    /// Check if this is a float type
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Primitive(PrimitiveType::F32 | PrimitiveType::F64))
    }
    
    /// Check if this is the unit type
    pub fn is_unit(&self) -> bool {
        matches!(self, Type::Primitive(PrimitiveType::Unit))
    }
    
    /// Check if this is the never type
    pub fn is_never(&self) -> bool {
        matches!(self, Type::Never)
    }
    
    /// Check if this is the unknown type
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }
    
    /// Get the inner type if this is an Option
    pub fn get_option_inner(&self) -> Option<&Type> {
        if let Type::Constructor(TypeConstructor::Option, types) = self {
            types.first()
        } else {
            None
        }
    }
    
    /// Get the inner types if this is a Result
    pub fn get_result_inner(&self) -> Option<(&Type, &Type)> {
        if let Type::Constructor(TypeConstructor::Result, types) = self {
            if types.len() == 2 {
                Some((&types[0], &types[1]))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Get function parameter and return types
    pub fn as_function(&self) -> Option<(&[Type], &Type)> {
        if let Type::Function(params, ret) = self {
            Some((params.as_slice(), ret.as_ref()))
        } else {
            None
        }
    }
    
    /// Get all type variables in this type
    pub fn get_type_vars(&self) -> Vec<TypeVar> {
        let mut vars = Vec::new();
        self.collect_type_vars(&mut vars);
        vars
    }
    
    fn collect_type_vars(&self, vars: &mut Vec<TypeVar>) {
        match self {
            Type::Var(v) => vars.push(*v),
            Type::Constructor(_, args) | Type::Function(args, _) | Type::Tuple(args) | Type::Struct(_, args) => {
                for arg in args {
                    arg.collect_type_vars(vars);
                }
            }
            Type::Reference(inner, _) | Type::Array(inner, _) | Type::Box(inner) => {
                inner.collect_type_vars(vars);
            }
            _ => {}
        }
    }
    
    /// Apply substitution to type
    pub fn apply_substitution(&self, subst: &Substitution) -> Type {
        match self {
            Type::Var(v) => {
                if let Some(replacement) = subst.get(v) {
                    replacement.clone()
                } else {
                    self.clone()
                }
            }
            Type::Constructor(constr, args) => {
                Type::Constructor(*constr, args.iter().map(|t| t.apply_substitution(subst)).collect())
            }
            Type::Function(params, ret) => {
                Type::Function(
                    params.iter().map(|t| t.apply_substitution(subst)).collect(),
                    Box::new(ret.apply_substitution(subst)),
                )
            }
            Type::Reference(inner, mutability) => {
                Type::Reference(Box::new(inner.apply_substitution(subst)), *mutability)
            }
            Type::Array(elem, size) => {
                Type::Array(Box::new(elem.apply_substitution(subst)), *size)
            }
            Type::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| t.apply_substitution(subst)).collect())
            }
            Type::Struct(name, args) => {
                Type::Struct(name.clone(), args.iter().map(|t| t.apply_substitution(subst)).collect())
            }
            _ => self.clone(),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Unknown
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", p),
            Type::Var(v) => {
                if let Some(name) = &v.name {
                    write!(f, "{}", name)
                } else {
                    write!(f, "?{}", v.id)
                }
            }
            Type::Constructor(constr, args) => {
                write!(f, "{}", constr)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Reference(inner, mutability) => {
                match mutability {
                    Mutability::Immutable => write!(f, "&{}", inner),
                    Mutability::Mutable => write!(f, "&mut {}", inner),
                }
            }
            Type::Array(elem, size) => {
                if let Some(size) = size {
                    write!(f, "[{}; {}]", elem, size)
                } else {
                    write!(f, "[{}]", elem)
                }
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Struct(name, args) => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::TraitObject(trait_ref) => {
                write!(f, "dyn {}", trait_ref)
            }
            Type::Never => write!(f, "!"),
            Type::Unknown => write!(f, "?"),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::I128 => write!(f, "i128"),
            PrimitiveType::Isize => write!(f, "isize"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::U128 => write!(f, "u128"),
            PrimitiveType::Usize => write!(f, "usize"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Char => write!(f, "char"),
            PrimitiveType::String => write!(f, "str"),
            PrimitiveType::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for TypeConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeConstructor::Option => write!(f, "Option"),
            TypeConstructor::Result => write!(f, "Result"),
            TypeConstructor::Vec => write!(f, "Vec"),
            TypeConstructor::HashMap => write!(f, "HashMap"),
            TypeConstructor::Box => write!(f, "Box"),
            TypeConstructor::Rc => write!(f, "Rc"),
            TypeConstructor::Arc => write!(f, "Arc"),
            TypeConstructor::Fn => write!(f, "Fn"),
            TypeConstructor::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for TraitRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}", param)?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

/// Substitution mapping type variables to types
#[derive(Debug, Clone, Default)]
pub struct Substitution {
    map: HashMap<u64, Type>,
}

impl Substitution {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }
    
    pub fn insert(&mut self, var: TypeVar, ty: Type) {
        self.map.insert(var.id, ty);
    }
    
    pub fn get(&self, var: &TypeVar) -> Option<&Type> {
        self.map.get(&var.id)
    }
    
    pub fn contains(&self, var: &TypeVar) -> bool {
        self.map.contains_key(&var.id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Type)> {
        self.map.iter()
    }
    
    pub fn extend(&mut self, other: Substitution) {
        self.map.extend(other.map);
    }
    
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::int()), "i64");
        assert_eq!(format!("{}", Type::string()), "str");
        assert_eq!(format!("{}", Type::bool()), "bool");
    }
    
    #[test]
    fn test_type_var() {
        let var = Type::var();
        assert!(var.is_var());
        
        let named = Type::var_named("T");
        assert!(named.is_var());
    }
    
    #[test]
    fn test_generic_types() {
        let option_int = Type::option(Type::int());
        assert!(format!("{}", option_int).contains("Option"));
        
        let vec_string = Type::vec(Type::string());
        assert!(format!("{}", vec_string).contains("Vec"));
    }
    
    #[test]
    fn test_function_type() {
        let func = Type::function(vec![Type::int(), Type::string()], Type::bool());
        assert!(func.as_function().is_some());
    }
    
    #[test]
    fn test_substitution() {
        let mut subst = Substitution::new();
        let var = TypeVar::named("T");
        subst.insert(var, Type::int());
        
        assert!(subst.contains(&var));
        assert_eq!(subst.get(&var), Some(&Type::int()));
    }
}
