//! DOM Standard Library for WebAssembly Targets
//! 
//! This module provides browser DOM manipulation capabilities for Hint programs
//! compiled to WebAssembly. It enables React-level UI development with plain
//! English syntax.
//! 
//! # Example Hint Program
//! 
//! ```hint
//! Say "Creating UI...".
//! Keep query_selector("#app") in mind as the container.
//! Keep create_element("button") in mind as the button.
//! Set the inner html of the button to "Click me!".
//! Set the style background-color of the button to "blue".
//! Append the button to the container.
//! Stop the program.
//! ```
//! 
//! # Architecture
//! 
//! ```text
//! Hint Source → WASM Module ←→ JS Glue Code ←→ Browser DOM
//! ```

pub mod elements;
pub mod events;
pub mod styles;
pub mod query;
pub mod render;

pub use elements::{DOMElement, ElementBuilder, ElementRegistry};
pub use events::{EventHandler, EventListener, EventRegistry};
pub use styles::{StyleBuilder, StyleRegistry};
pub use query::{DOMQuery, QueryResult};
pub use render::{DOMRenderer, RenderContext};

use crate::stdlib::{StdlibFunction, StdlibImpl, IntrinsicId};
use crate::semantics::HintType;

/// Initialize DOM stdlib functions
pub fn init() -> Vec<StdlibFunction> {
    vec![
        // Element Creation
        StdlibFunction {
            name: "create_element".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomCreateElement),
            description: "Create a DOM element by tag name",
        },
        
        StdlibFunction {
            name: "create_text".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomCreateText),
            description: "Create a text node",
        },
        
        StdlibFunction {
            name: "create_button".to_string(),
            params: vec![],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomCreateButton),
            description: "Create a button element",
        },
        
        StdlibFunction {
            name: "create_div".to_string(),
            params: vec![],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomCreateDiv),
            description: "Create a div element",
        },
        
        StdlibFunction {
            name: "create_input".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomCreateInput),
            description: "Create an input element with specified type",
        },
        
        // Element Manipulation
        StdlibFunction {
            name: "set_inner_html".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomSetInnerHtml),
            description: "Set element inner HTML content",
        },
        
        StdlibFunction {
            name: "set_text_content".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomSetTextContent),
            description: "Set element text content",
        },
        
        StdlibFunction {
            name: "get_inner_html".to_string(),
            params: vec![HintType::Pointer(Box::new(HintType::Void))],
            return_type: HintType::String,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetInnerHtml),
            description: "Get element inner HTML content",
        },
        
        StdlibFunction {
            name: "get_text_content".to_string(),
            params: vec![HintType::Pointer(Box::new(HintType::Void))],
            return_type: HintType::String,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetTextContent),
            description: "Get element text content",
        },
        
        // Style Manipulation
        StdlibFunction {
            name: "set_style".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomSetStyle),
            description: "Set CSS style property on element",
        },
        
        StdlibFunction {
            name: "get_style".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::String,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetStyle),
            description: "Get CSS style property value",
        },
        
        StdlibFunction {
            name: "set_class".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomSetClass),
            description: "Set CSS class on element",
        },
        
        StdlibFunction {
            name: "add_class".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomAddClass),
            description: "Add CSS class to element",
        },
        
        StdlibFunction {
            name: "remove_class".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomRemoveClass),
            description: "Remove CSS class from element",
        },
        
        StdlibFunction {
            name: "has_class".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Bool,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomHasClass),
            description: "Check if element has CSS class",
        },
        
        // Attribute Manipulation
        StdlibFunction {
            name: "set_attribute".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomSetAttribute),
            description: "Set HTML attribute on element",
        },
        
        StdlibFunction {
            name: "get_attribute".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::String,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetAttribute),
            description: "Get HTML attribute value",
        },
        
        StdlibFunction {
            name: "remove_attribute".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomRemoveAttribute),
            description: "Remove HTML attribute from element",
        },
        
        // DOM Tree Manipulation
        StdlibFunction {
            name: "append_child".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::Pointer(Box::new(HintType::Void)),
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomAppendChild),
            description: "Append child element to parent",
        },
        
        StdlibFunction {
            name: "prepend_child".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::Pointer(Box::new(HintType::Void)),
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomPrependChild),
            description: "Prepend child element to parent",
        },
        
        StdlibFunction {
            name: "remove_child".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::Pointer(Box::new(HintType::Void)),
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomRemoveChild),
            description: "Remove child element from parent",
        },
        
        StdlibFunction {
            name: "replace_child".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::Pointer(Box::new(HintType::Void)),
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomReplaceChild),
            description: "Replace child element with new element",
        },
        
        StdlibFunction {
            name: "remove".to_string(),
            params: vec![HintType::Pointer(Box::new(HintType::Void))],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomRemove),
            description: "Remove element from DOM",
        },
        
        // Query/Selection
        StdlibFunction {
            name: "query_selector".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomQuerySelector),
            description: "Query single DOM element by CSS selector",
        },
        
        StdlibFunction {
            name: "query_selector_all".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Array(Box::new(HintType::Pointer(Box::new(HintType::Void))), 0),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomQuerySelectorAll),
            description: "Query all DOM elements matching CSS selector",
        },
        
        StdlibFunction {
            name: "get_element_by_id".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetById),
            description: "Get element by ID",
        },
        
        StdlibFunction {
            name: "get_elements_by_class".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Array(Box::new(HintType::Pointer(Box::new(HintType::Void))), 0),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetByClass),
            description: "Get elements by class name",
        },
        
        StdlibFunction {
            name: "get_elements_by_tag".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Array(Box::new(HintType::Pointer(Box::new(HintType::Void))), 0),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetByTag),
            description: "Get elements by tag name",
        },
        
        // Event Handling
        StdlibFunction {
            name: "add_event_listener".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomAddEventListener),
            description: "Add event listener to element",
        },
        
        StdlibFunction {
            name: "remove_event_listener".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomRemoveEventListener),
            description: "Remove event listener from element",
        },
        
        StdlibFunction {
            name: "dispatch_event".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Bool,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomDispatchEvent),
            description: "Dispatch event from element",
        },
        
        // Form/Input Handling
        StdlibFunction {
            name: "get_value".to_string(),
            params: vec![HintType::Pointer(Box::new(HintType::Void))],
            return_type: HintType::String,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetValue),
            description: "Get input element value",
        },
        
        StdlibFunction {
            name: "set_value".to_string(),
            params: vec![
                HintType::Pointer(Box::new(HintType::Void)),
                HintType::String,
            ],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomSetValue),
            description: "Set input element value",
        },
        
        StdlibFunction {
            name: "focus".to_string(),
            params: vec![HintType::Pointer(Box::new(HintType::Void))],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomFocus),
            description: "Focus element",
        },
        
        StdlibFunction {
            name: "blur".to_string(),
            params: vec![HintType::Pointer(Box::new(HintType::Void))],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomBlur),
            description: "Blur element",
        },
        
        // Document/Window
        StdlibFunction {
            name: "get_document".to_string(),
            params: vec![],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetDocument),
            description: "Get document object",
        },
        
        StdlibFunction {
            name: "get_window".to_string(),
            params: vec![],
            return_type: HintType::Pointer(Box::new(HintType::Void)),
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomGetWindow),
            description: "Get window object",
        },
        
        StdlibFunction {
            name: "alert".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomAlert),
            description: "Show alert dialog",
        },
        
        StdlibFunction {
            name: "console_log".to_string(),
            params: vec![HintType::String],
            return_type: HintType::Void,
            implementation: StdlibImpl::Intrinsic(IntrinsicId::DomConsoleLog),
            description: "Log to browser console",
        },
    ]
}

/// WASM module imports for DOM access (WebAssembly Text Format)
pub const DOM_IMPORTS_WAT: &str = r#"
(module
  ;; Element Creation
  (import "hint_dom" "create_element" 
    (func $create_element (param i32 i32) (result i32)))
  (import "hint_dom" "create_text"
    (func $create_text (param i32 i32) (result i32)))
  (import "hint_dom" "create_button"
    (func $create_button () (result i32)))
  (import "hint_dom" "create_div"
    (func $create_div () (result i32)))
  (import "hint_dom" "create_input"
    (func $create_input (param i32 i32) (result i32)))
  
  ;; Element Manipulation
  (import "hint_dom" "set_inner_html"
    (func $set_inner_html (param i32 i32 i32 i32)))
  (import "hint_dom" "set_text_content"
    (func $set_text_content (param i32 i32 i32 i32)))
  (import "hint_dom" "get_inner_html"
    (func $get_inner_html (param i32) (result i32 i32)))
  (import "hint_dom" "get_text_content"
    (func $get_text_content (param i32) (result i32 i32)))
  
  ;; Style Manipulation
  (import "hint_dom" "set_style"
    (func $set_style (param i32 i32 i32 i32 i32 i32)))
  (import "hint_dom" "get_style"
    (func $get_style (param i32 i32 i32) (result i32 i32)))
  (import "hint_dom" "set_class"
    (func $set_class (param i32 i32 i32 i32)))
  (import "hint_dom" "add_class"
    (func $add_class (param i32 i32 i32 i32)))
  (import "hint_dom" "remove_class"
    (func $remove_class (param i32 i32 i32 i32)))
  (import "hint_dom" "has_class"
    (func $has_class (param i32 i32 i32 i32) (result i32)))
  
  ;; Attribute Manipulation
  (import "hint_dom" "set_attribute"
    (func $set_attribute (param i32 i32 i32 i32 i32 i32)))
  (import "hint_dom" "get_attribute"
    (func $get_attribute (param i32 i32 i32 i32) (result i32 i32)))
  (import "hint_dom" "remove_attribute"
    (func $remove_attribute (param i32 i32 i32 i32)))
  
  ;; DOM Tree Manipulation
  (import "hint_dom" "append_child"
    (func $append_child (param i32 i32)))
  (import "hint_dom" "prepend_child"
    (func $prepend_child (param i32 i32)))
  (import "hint_dom" "remove_child"
    (func $remove_child (param i32 i32)))
  (import "hint_dom" "replace_child"
    (func $replace_child (param i32 i32 i32)))
  (import "hint_dom" "remove"
    (func $remove (param i32)))
  
  ;; Query/Selection
  (import "hint_dom" "query_selector"
    (func $query_selector (param i32 i32) (result i32)))
  (import "hint_dom" "query_selector_all"
    (func $query_selector_all (param i32 i32) (result i32 i32)))
  (import "hint_dom" "get_element_by_id"
    (func $get_element_by_id (param i32 i32) (result i32)))
  
  ;; Event Handling
  (import "hint_dom" "add_event_listener"
    (func $add_event_listener (param i32 i32 i32 i32 i32)))
  (import "hint_dom" "remove_event_listener"
    (func $remove_event_listener (param i32 i32 i32 i32 i32)))
  (import "hint_dom" "dispatch_event"
    (func $dispatch_event (param i32 i32 i32 i32) (result i32)))
  
  ;; Form/Input
  (import "hint_dom" "get_value"
    (func $get_value (param i32) (result i32 i32)))
  (import "hint_dom" "set_value"
    (func $set_value (param i32 i32 i32 i32)))
  (import "hint_dom" "focus"
    (func $focus (param i32)))
  (import "hint_dom" "blur"
    (func $blur (param i32)))
  
  ;; Document/Window
  (import "hint_dom" "get_document"
    (func $get_document () (result i32)))
  (import "hint_dom" "get_window"
    (func $get_window () (result i32)))
  (import "hint_dom" "alert"
    (func $alert (param i32 i32)))
  (import "hint_dom" "console_log"
    (func $console_log (param i32 i32)))
  
  ;; Memory Management
  (import "hint_dom" "allocate_string"
    (func $allocate_string (param i32 i32) (result i32)))
  (import "hint_dom" "free_string"
    (func $free_string (param i32)))
  (import "hint_dom" "allocate_array"
    (func $allocate_array (param i32) (result i32)))
)"#;

/// JavaScript glue code for DOM interop
pub const DOM_GLUE_JS: &str = include_str!("dom_glue.js");

/// Get the complete DOM stdlib
pub fn get_dom_stdlib() -> Vec<StdlibFunction> {
    init()
}

/// DOM intrinsic function IDs
#[derive(Debug, Clone, Copy)]
pub enum DomIntrinsic {
    // Element Creation
    CreateElement,
    CreateText,
    CreateButton,
    CreateDiv,
    CreateInput,
    
    // Element Manipulation
    SetInnerHtml,
    SetTextContent,
    GetInnerHtml,
    GetTextContent,
    
    // Style
    SetStyle,
    GetStyle,
    SetClass,
    AddClass,
    RemoveClass,
    HasClass,
    
    // Attributes
    SetAttribute,
    GetAttribute,
    RemoveAttribute,
    
    // Tree
    AppendChild,
    PrependChild,
    RemoveChild,
    ReplaceChild,
    Remove,
    
    // Query
    QuerySelector,
    QuerySelectorAll,
    GetById,
    
    // Events
    AddEventListener,
    RemoveEventListener,
    DispatchEvent,
    
    // Form
    GetValue,
    SetValue,
    Focus,
    Blur,
    
    // Document/Window
    GetDocument,
    GetWindow,
    Alert,
    ConsoleLog,
}

impl DomIntrinsic {
    pub fn to_import_name(&self) -> &'static str {
        match self {
            DomIntrinsic::CreateElement => "create_element",
            DomIntrinsic::CreateText => "create_text",
            DomIntrinsic::CreateButton => "create_button",
            DomIntrinsic::CreateDiv => "create_div",
            DomIntrinsic::CreateInput => "create_input",
            DomIntrinsic::SetInnerHtml => "set_inner_html",
            DomIntrinsic::SetTextContent => "set_text_content",
            DomIntrinsic::GetInnerHtml => "get_inner_html",
            DomIntrinsic::GetTextContent => "get_text_content",
            DomIntrinsic::SetStyle => "set_style",
            DomIntrinsic::GetStyle => "get_style",
            DomIntrinsic::SetClass => "set_class",
            DomIntrinsic::AddClass => "add_class",
            DomIntrinsic::RemoveClass => "remove_class",
            DomIntrinsic::HasClass => "has_class",
            DomIntrinsic::SetAttribute => "set_attribute",
            DomIntrinsic::GetAttribute => "get_attribute",
            DomIntrinsic::RemoveAttribute => "remove_attribute",
            DomIntrinsic::AppendChild => "append_child",
            DomIntrinsic::PrependChild => "prepend_child",
            DomIntrinsic::RemoveChild => "remove_child",
            DomIntrinsic::ReplaceChild => "replace_child",
            DomIntrinsic::Remove => "remove",
            DomIntrinsic::QuerySelector => "query_selector",
            DomIntrinsic::QuerySelectorAll => "query_selector_all",
            DomIntrinsic::GetById => "get_element_by_id",
            DomIntrinsic::AddEventListener => "add_event_listener",
            DomIntrinsic::RemoveEventListener => "remove_event_listener",
            DomIntrinsic::DispatchEvent => "dispatch_event",
            DomIntrinsic::GetValue => "get_value",
            DomIntrinsic::SetValue => "set_value",
            DomIntrinsic::Focus => "focus",
            DomIntrinsic::Blur => "blur",
            DomIntrinsic::GetDocument => "get_document",
            DomIntrinsic::GetWindow => "get_window",
            DomIntrinsic::Alert => "alert",
            DomIntrinsic::ConsoleLog => "console_log",
        }
    }
}
