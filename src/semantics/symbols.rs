//! Symbol table management for semantic analysis.

use std::collections::HashMap;
use crate::semantics::types::HintType;

/// A symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub span: crate::diagnostics::Span,
    pub is_mutable: bool,
}

/// Type of symbol
#[derive(Debug, Clone)]
pub enum SymbolType {
    /// A variable
    Variable(HintType),
    /// A constant
    Constant(HintType, ConstantValue),
    /// A function
    Function(Vec<HintType>, HintType),
    /// A builtin function
    Builtin(Vec<HintType>, HintType),
}

/// Constant values
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// A scope level in the symbol table
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

impl Scope {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            symbols: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(name: impl Into<String>, parent: usize) -> Self {
        Self {
            name: name.into(),
            symbols: HashMap::new(),
            parent: Some(parent),
        }
    }
    
    pub fn insert(&mut self, symbol: Symbol) -> Option<Symbol> {
        self.symbols.insert(symbol.name.clone(), symbol)
    }
    
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
    
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}

/// Symbol table for tracking declarations
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            scopes: Vec::new(),
            current_scope: 0,
        };
        // Create global scope
        table.scopes.push(Scope::new("global"));
        table
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self, name: &str) {
        let parent = self.current_scope;
        self.scopes.push(Scope::with_parent(name, parent));
        self.current_scope = self.scopes.len() - 1;
    }
    
    /// Exit current scope
    pub fn exit_scope(&mut self) -> Option<Scope> {
        if self.current_scope == 0 {
            return None;
        }
        
        let scope = self.scopes.pop();
        self.current_scope = self.scopes.len() - 1;
        scope
    }
    
    /// Insert a symbol into the current scope
    pub fn insert(&mut self, symbol: Symbol) -> Result<(), Symbol> {
        let scope = &mut self.scopes[self.current_scope];
        if let Some(existing) = scope.insert(symbol.clone()) {
            return Err(existing);
        }
        Ok(())
    }
    
    /// Look up a symbol by name
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = self.current_scope;
        
        loop {
            if let Some(symbol) = self.scopes[scope_idx].get(name) {
                return Some(symbol);
            }
            
            match self.scopes[scope_idx].parent {
                Some(parent) => scope_idx = parent,
                None => break,
            }
        }
        
        None
    }
    
    /// Check if a symbol exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }
    
    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        &self.scopes[self.current_scope]
    }
    
    /// Get all symbols in the global scope
    pub fn globals(&self) -> &HashMap<String, Symbol> {
        &self.scopes[0].symbols
    }
    
    /// Initialize with builtin functions
    pub fn init_builtins(&mut self) {
        self.enter_scope("builtins");
        
        // Add builtin functions here as needed
        // For now, the basic Hint language doesn't have builtins
        
        self.exit_scope();
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
