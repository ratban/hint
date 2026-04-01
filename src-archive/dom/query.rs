//! DOM Query Module
//! 
//! Provides CSS selector query abstractions.

use std::collections::HashMap;

/// Query result wrapper
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Element IDs matching query
    pub element_ids: Vec<u32>,
    /// Query selector used
    pub selector: String,
}

impl QueryResult {
    pub fn new(selector: &str) -> Self {
        Self {
            element_ids: Vec::new(),
            selector: selector.to_string(),
        }
    }
    
    pub fn with_elements(selector: &str, ids: Vec<u32>) -> Self {
        Self {
            element_ids: ids,
            selector: selector.to_string(),
        }
    }
    
    pub fn first(&self) -> Option<u32> {
        self.element_ids.first().copied()
    }
    
    pub fn len(&self) -> usize {
        self.element_ids.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.element_ids.is_empty()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &u32> {
        self.element_ids.iter()
    }
}

/// DOM query interface
pub struct DOMQuery {
    cache: HashMap<String, QueryResult>,
    cache_enabled: bool,
}

impl DOMQuery {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_enabled: true,
        }
    }
    
    /// Query single element
    pub fn query(&mut self, selector: &str) -> QueryResult {
        if self.cache_enabled {
            if let Some(cached) = self.cache.get(selector) {
                return cached.clone();
            }
        }
        
        // In real impl, would call WASM import
        let result = QueryResult::new(selector);
        
        if self.cache_enabled {
            self.cache.insert(selector.to_string(), result.clone());
        }
        
        result
    }
    
    /// Query all elements
    pub fn query_all(&mut self, selector: &str) -> QueryResult {
        if self.cache_enabled {
            if let Some(cached) = self.cache.get(selector) {
                return cached.clone();
            }
        }
        
        // In real impl, would call WASM import
        let result = QueryResult::new(selector);
        
        if self.cache_enabled {
            self.cache.insert(selector.to_string(), result.clone());
        }
        
        result
    }
    
    /// Query by ID
    pub fn by_id(&mut self, id: &str) -> QueryResult {
        self.query(&format!("#{}", id))
    }
    
    /// Query by class
    pub fn by_class(&mut self, class: &str) -> QueryResult {
        self.query(&format!(".{}", class))
    }
    
    /// Query by tag
    pub fn by_tag(&mut self, tag: &str) -> QueryResult {
        self.query(tag)
    }
    
    /// Invalidate cache for selector
    pub fn invalidate(&mut self, selector: &str) {
        self.cache.remove(selector);
    }
    
    /// Clear entire cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Enable/disable caching
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }
}

impl Default for DOMQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS selector builder
pub struct SelectorBuilder {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attributes: Vec<(String, String)>,
    pseudo: Vec<String>,
    combinator: Option<Combinator>,
}

#[derive(Debug, Clone)]
pub enum Combinator {
    Descendant,
    Child,
    AdjacentSibling,
    Sibling,
}

impl SelectorBuilder {
    pub fn new() -> Self {
        Self {
            tag: None,
            id: None,
            classes: Vec::new(),
            attributes: Vec::new(),
            pseudo: Vec::new(),
            combinator: None,
        }
    }
    
    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = Some(tag.to_string());
        self
    }
    
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }
    
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.attributes.push((name.to_string(), value.to_string()));
        self
    }
    
    pub fn pseudo(mut self, pseudo: &str) -> Self {
        self.pseudo.push(pseudo.to_string());
        self
    }
    
    pub fn child_of(mut self, parent: &SelectorBuilder) -> Self {
        self.combinator = Some(Combinator::Child);
        // In real impl, would chain selectors
        self
    }
    
    /// Build selector string
    pub fn build(&self) -> String {
        let mut selector = String::new();
        
        if let Some(tag) = &self.tag {
            selector.push_str(tag);
        }
        
        if let Some(id) = &self.id {
            selector.push('#');
            selector.push_str(id);
        }
        
        for class in &self.classes {
            selector.push('.');
            selector.push_str(class);
        }
        
        for (name, value) in &self.attributes {
            selector.push('[');
            selector.push_str(name);
            selector.push_str("=\"");
            selector.push_str(value);
            selector.push_str("\"]");
        }
        
        for pseudo in &self.pseudo {
            selector.push(':');
            selector.push_str(pseudo);
        }
        
        selector
    }
}

impl Default for SelectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common selector presets
pub mod selectors {
    use super::*;
    
    pub fn first_child() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("first-child")
    }
    
    pub fn last_child() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("last-child")
    }
    
    pub fn nth_child(n: u32) -> SelectorBuilder {
        SelectorBuilder::new().pseudo(&format!("nth-child({})", n))
    }
    
    pub fn hover() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("hover")
    }
    
    pub fn focus() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("focus")
    }
    
    pub fn active() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("active")
    }
    
    pub fn disabled() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("disabled")
    }
    
    pub fn enabled() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("enabled")
    }
    
    pub fn checked() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("checked")
    }
    
    pub fn empty() -> SelectorBuilder {
        SelectorBuilder::new().pseudo("empty")
    }
    
    pub fn not(selector: &str) -> SelectorBuilder {
        SelectorBuilder::new().pseudo(&format!("not({})", selector))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_result() {
        let result = QueryResult::with_elements(".test", vec![1, 2, 3]);
        
        assert_eq!(result.len(), 3);
        assert_eq!(result.first(), Some(1));
        assert!(!result.is_empty());
    }
    
    #[test]
    fn test_selector_builder() {
        let selector = SelectorBuilder::new()
            .tag("div")
            .id("main")
            .class("container")
            .class("flex")
            .pseudo("hover")
            .build();
        
        assert_eq!(selector, "div#main.container.flex:hover");
    }
    
    #[test]
    fn test_attribute_selector() {
        let selector = SelectorBuilder::new()
            .tag("input")
            .attr("type", "text")
            .attr("placeholder", "Enter name")
            .build();
        
        assert!(selector.contains("[type=\"text\"]"));
        assert!(selector.contains("[placeholder=\"Enter name\"]"));
    }
}
