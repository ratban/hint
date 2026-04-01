//! DOM Rendering Module
//! 
//! Provides virtual DOM and rendering abstractions for efficient updates.

use std::collections::HashMap;

/// Virtual DOM node
#[derive(Debug, Clone)]
pub enum VNode {
    /// Element node
    Element {
        tag: String,
        props: HashMap<String, String>,
        children: Vec<VNode>,
        key: Option<String>,
    },
    /// Text node
    Text(String),
    /// Comment node
    Comment(String),
    /// Fragment (group of nodes)
    Fragment(Vec<VNode>),
}

impl VNode {
    pub fn element(tag: &str) -> VElementBuilder {
        VElementBuilder::new(tag)
    }
    
    pub fn text(content: &str) -> Self {
        VNode::Text(content.to_string())
    }
    
    pub fn comment(content: &str) -> Self {
        VNode::Comment(content.to_string())
    }
    
    pub fn fragment(children: Vec<VNode>) -> Self {
        VNode::Fragment(children)
    }
}

/// Virtual element builder
pub struct VElementBuilder {
    tag: String,
    props: HashMap<String, String>,
    children: Vec<VNode>,
    key: Option<String>,
}

impl VElementBuilder {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            props: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }
    
    pub fn prop(mut self, name: &str, value: &str) -> Self {
        self.props.insert(name.to_string(), value.to_string());
        self
    }
    
    pub fn id(mut self, id: &str) -> Self {
        self.prop("id", id)
    }
    
    pub fn class(mut self, class: &str) -> Self {
        self.prop("class", class)
    }
    
    pub fn style(mut self, prop: &str, value: &str) -> Self {
        self.prop(&format!("style-{}", prop), value)
    }
    
    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }
    
    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }
    
    pub fn text_child(mut self, text: &str) -> Self {
        self.children.push(VNode::Text(text.to_string()));
        self
    }
    
    pub fn build(self) -> VNode {
        VNode::Element {
            tag: self.tag,
            props: self.props,
            children: self.children,
            key: self.key,
        }
    }
}

/// Virtual DOM tree
#[derive(Debug, Clone)]
pub struct VTree {
    pub root: VNode,
}

impl VTree {
    pub fn new(root: VNode) -> Self {
        Self { root }
    }
}

/// DOM diff algorithm
pub struct DiffAlgorithm;

impl DiffAlgorithm {
    /// Compute diff between two VNodes
    pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
        let mut patches = Vec::new();
        Self::diff_recursive(old, new, &mut patches, 0);
        patches
    }
    
    fn diff_recursive(old: &VNode, new: &VNode, patches: &mut Vec<Patch>, index: usize) {
        match (old, new) {
            (VNode::Text(old_text), VNode::Text(new_text)) => {
                if old_text != new_text {
                    patches.push(Patch::ReplaceText(index, new_text.clone()));
                }
            }
            
            (
                VNode::Element { tag: old_tag, props: old_props, children: old_children, .. },
                VNode::Element { tag: new_tag, props: new_props, children: new_children, .. },
            ) => {
                if old_tag != new_tag {
                    patches.push(Patch::ReplaceElement(index, new_tag.clone()));
                    return;
                }
                
                // Diff props
                Self::diff_props(old_props, new_props, &mut patches, index);
                
                // Diff children
                Self::diff_children(old_children, new_children, &mut patches, index);
            }
            
            _ => {
                patches.push(Patch::ReplaceNode(index, new.clone()));
            }
        }
    }
    
    fn diff_props(
        old_props: &HashMap<String, String>,
        new_props: &HashMap<String, String>,
        patches: &mut Vec<Patch>,
        index: usize,
    ) {
        // Removed props
        for (key, _) in old_props {
            if !new_props.contains_key(key) {
                patches.push(Patch::RemoveProp(index, key.clone()));
            }
        }
        
        // Added/changed props
        for (key, value) in new_props {
            if old_props.get(key) != Some(value) {
                patches.push(Patch::SetProp(index, key.clone(), value.clone()));
            }
        }
    }
    
    fn diff_children(
        old_children: &[VNode],
        new_children: &[VNode],
        patches: &mut Vec<Patch>,
        parent_index: usize,
    ) {
        let max_len = old_children.len().max(new_children.len());
        
        for i in 0..max_len {
            match (old_children.get(i), new_children.get(i)) {
                (Some(old), Some(new)) => {
                    Self::diff_recursive(old, new, patches, parent_index * 100 + i + 1);
                }
                (Some(_), None) => {
                    patches.push(Patch::RemoveChild(parent_index * 100 + i + 1));
                }
                (None, Some(new)) => {
                    patches.push(Patch::InsertChild(parent_index * 100 + i + 1, new.clone()));
                }
            }
        }
    }
}

/// Patch operation for DOM updates
#[derive(Debug, Clone)]
pub enum Patch {
    /// Replace entire node
    ReplaceNode(usize, VNode),
    /// Replace element tag
    ReplaceElement(usize, String),
    /// Replace text content
    ReplaceText(usize, String),
    /// Set property
    SetProp(usize, String, String),
    /// Remove property
    RemoveProp(usize, String),
    /// Insert child
    InsertChild(usize, VNode),
    /// Remove child
    RemoveChild(usize),
    /// Reorder children
    ReorderChildren(usize, Vec<usize>),
}

/// DOM renderer
pub struct DOMRenderer {
    /// Element registry (WASM handles)
    element_registry: HashMap<u32, VNode>,
    /// Next available ID
    next_id: u32,
}

impl DOMRenderer {
    pub fn new() -> Self {
        Self {
            element_registry: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Render virtual tree to DOM
    pub fn render(&mut self, tree: &VTree, container_id: u32) -> Result<(), String> {
        // In real impl, would generate WASM calls
        // For now, just track the rendered tree
        self.element_registry.insert(container_id, tree.root.clone());
        Ok(())
    }
    
    /// Patch existing DOM
    pub fn patch(&mut self, element_id: u32, patches: &[Patch]) -> Result<(), String> {
        if let Some(vnode) = self.element_registry.get_mut(&element_id) {
            for patch in patches {
                self.apply_patch(vnode, patch);
            }
        }
        Ok(())
    }
    
    fn apply_patch(&self, vnode: &mut VNode, patch: &Patch) {
        match patch {
            Patch::ReplaceNode(_, new_vnode) => {
                *vnode = new_vnode.clone();
            }
            Patch::ReplaceElement(_, new_tag) => {
                if let VNode::Element { tag, .. } = vnode {
                    *tag = new_tag.clone();
                }
            }
            Patch::ReplaceText(_, new_text) => {
                *vnode = VNode::Text(new_text.clone());
            }
            _ => {
                // Other patches would modify children/props
            }
        }
    }
    
    /// Get rendered node
    pub fn get_node(&self, element_id: u32) -> Option<&VNode> {
        self.element_registry.get(&element_id)
    }
}

impl Default for DOMRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render context for component rendering
pub struct RenderContext {
    /// Current component state
    state: HashMap<String, String>,
    /// Event handlers
    handlers: HashMap<String, u32>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            handlers: HashMap::new(),
        }
    }
    
    pub fn set_state(&mut self, key: &str, value: &str) {
        self.state.insert(key.to_string(), value.to_string());
    }
    
    pub fn get_state(&self, key: &str) -> Option<&str> {
        self.state.get(key).map(|s| s.as_str())
    }
    
    pub fn register_handler(&mut self, event: &str, callback_id: u32) {
        self.handlers.insert(event.to_string(), callback_id);
    }
    
    pub fn get_handler(&self, event: &str) -> Option<u32> {
        self.handlers.get(event).copied()
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Component trait for reusable UI components
pub trait Component {
    type Props;
    type State;
    
    fn init(props: Self::Props) -> Self::State;
    fn render(state: &Self::State, ctx: &mut RenderContext) -> VNode;
    fn update(state: &mut Self::State, action: &str, payload: &str);
}

/// Simple functional component helper
pub fn functional_component<F, P>(props: P, render_fn: F) -> VNode
where
    F: Fn(P) -> VNode,
{
    render_fn(props)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vnode_builder() {
        let vnode = VNode::element("div")
            .id("app")
            .class("container")
            .text_child("Hello, World!")
            .child(
                VNode::element("button")
                    .class("btn")
                    .text_child("Click me")
                    .build()
            )
            .build();
        
        if let VNode::Element { children, .. } = vnode {
            assert_eq!(children.len(), 2);
        }
    }
    
    #[test]
    fn test_diff_algorithm() {
        let old = VNode::element("div")
            .id("app")
            .text_child("Old")
            .build();
        
        let new = VNode::element("div")
            .id("app")
            .text_child("New")
            .build();
        
        let patches = DiffAlgorithm::diff(&old, &new);
        
        assert!(!patches.is_empty());
    }
}
