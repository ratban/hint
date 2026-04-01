//! DOM Element Module
//! 
//! Provides element creation and manipulation abstractions.

use crate::stdlib::dom::DomIntrinsic;

/// DOM element wrapper
#[derive(Debug, Clone)]
pub struct DOMElement {
    /// Element ID (WASM handle)
    pub id: u32,
    /// Tag name
    pub tag: String,
    /// Parent element ID
    pub parent_id: Option<u32>,
    /// Child element IDs
    pub children: Vec<u32>,
}

impl DOMElement {
    pub fn new(id: u32, tag: &str) -> Self {
        Self {
            id,
            tag: tag.to_string(),
            parent_id: None,
            children: Vec::new(),
        }
    }
    
    pub fn is_text_node(&self) -> bool {
        self.tag == "#text"
    }
    
    pub fn is_element(&self) -> bool {
        !self.is_text_node()
    }
}

/// Builder for creating DOM elements
pub struct ElementBuilder {
    tag: String,
    id: Option<String>,
    classes: Vec<String>,
    attributes: Vec<(String, String)>,
    children: Vec<ChildElement>,
    text_content: Option<String>,
    styles: Vec<(String, String)>,
}

enum ChildElement {
    Element(ElementBuilder),
    Text(String),
}

impl ElementBuilder {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            id: None,
            classes: Vec::new(),
            attributes: Vec::new(),
            children: Vec::new(),
            text_content: None,
            styles: Vec::new(),
        }
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
    
    pub fn style(mut self, prop: &str, value: &str) -> Self {
        self.styles.push((prop.to_string(), value.to_string()));
        self
    }
    
    pub fn text(mut self, text: &str) -> Self {
        self.text_content = Some(text.to_string());
        self
    }
    
    pub fn child(mut self, child: ElementBuilder) -> Self {
        self.children.push(ChildElement::Element(child));
        self
    }
    
    pub fn text_child(mut self, text: &str) -> Self {
        self.children.push(ChildElement::Text(text.to_string()));
        self
    }
    
    /// Build element and return WASM handle
    pub fn build(self) -> Result<DOMElement, String> {
        // In real implementation, this would call WASM imports
        // For now, return a logical element
        
        let element = DOMElement::new(1, &self.tag);
        
        // Apply attributes, styles, children
        // This would generate WASM calls in real impl
        
        Ok(element)
    }
}

/// Registry for tracking DOM elements
pub struct ElementRegistry {
    elements: std::collections::HashMap<u32, DOMElement>,
    next_id: u32,
}

impl ElementRegistry {
    pub fn new() -> Self {
        Self {
            elements: std::collections::HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn register(&mut self, element: DOMElement) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.elements.insert(id, element);
        id
    }
    
    pub fn get(&self, id: u32) -> Option<&DOMElement> {
        self.elements.get(&id)
    }
    
    pub fn get_mut(&mut self, id: u32) -> Option<&mut DOMElement> {
        self.elements.get_mut(&id)
    }
    
    pub fn remove(&mut self, id: u32) -> Option<DOMElement> {
        self.elements.remove(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &DOMElement> {
        self.elements.values()
    }
}

impl Default for ElementRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Common HTML element creators
pub mod elements {
    use super::ElementBuilder;
    
    pub fn div() -> ElementBuilder {
        ElementBuilder::new("div")
    }
    
    pub fn span() -> ElementBuilder {
        ElementBuilder::new("span")
    }
    
    pub fn button() -> ElementBuilder {
        ElementBuilder::new("button")
    }
    
    pub fn input(input_type: &str) -> ElementBuilder {
        ElementBuilder::new("input").attr("type", input_type)
    }
    
    pub fn text_input() -> ElementBuilder {
        input("text")
    }
    
    pub fn checkbox() -> ElementBuilder {
        input("checkbox")
    }
    
    pub fn radio() -> ElementBuilder {
        input("radio")
    }
    
    pub fn link(href: &str) -> ElementBuilder {
        ElementBuilder::new("a").attr("href", href)
    }
    
    pub fn image(src: &str) -> ElementBuilder {
        ElementBuilder::new("img").attr("src", src)
    }
    
    pub fn heading(level: u8) -> ElementBuilder {
        let tag = format!("h{}", level.min(6).max(1));
        ElementBuilder::new(&tag)
    }
    
    pub fn paragraph() -> ElementBuilder {
        ElementBuilder::new("p")
    }
    
    pub fn list(ordered: bool) -> ElementBuilder {
        ElementBuilder::new(if ordered { "ol" } else { "ul" })
    }
    
    pub fn list_item() -> ElementBuilder {
        ElementBuilder::new("li")
    }
    
    pub fn container() -> ElementBuilder {
        div().class("container")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elements::*;
    
    #[test]
    fn test_element_builder() {
        let element = div()
            .id("main")
            .class("container")
            .class("flex")
            .style("display", "flex")
            .child(span().text("Hello"))
            .build();
        
        assert!(element.is_ok());
    }
    
    #[test]
    fn test_registry() {
        let mut registry = ElementRegistry::new();
        
        let element = DOMElement::new(1, "div");
        let id = registry.register(element);
        
        assert!(registry.get(id).is_some());
        assert_eq!(registry.get(id).unwrap().tag, "div");
    }
}
