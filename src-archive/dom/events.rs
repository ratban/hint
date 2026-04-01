//! DOM Event Handling Module
//! 
//! Provides event listener registration and callback handling.

use std::collections::HashMap;

/// Event types supported by Hint DOM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    // Mouse Events
    Click,
    DblClick,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseOver,
    MouseOut,
    MouseEnter,
    MouseLeave,
    ContextMenu,
    
    // Keyboard Events
    KeyDown,
    KeyUp,
    KeyPress,
    
    // Form Events
    Input,
    Change,
    Submit,
    Reset,
    Focus,
    Blur,
    
    // Document Events
    Load,
    Unload,
    Resize,
    Scroll,
    
    // Drag Events
    Drag,
    DragStart,
    DragEnd,
    DragOver,
    Drop,
    
    // Clipboard Events
    Copy,
    Cut,
    Paste,
    
    // Touch Events
    TouchStart,
    TouchMove,
    TouchEnd,
    
    // Custom
    Custom,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Click => "click",
            EventType::DblClick => "dblclick",
            EventType::MouseDown => "mousedown",
            EventType::MouseUp => "mouseup",
            EventType::MouseMove => "mousemove",
            EventType::MouseOver => "mouseover",
            EventType::MouseOut => "mouseout",
            EventType::MouseEnter => "mouseenter",
            EventType::MouseLeave => "mouseleave",
            EventType::ContextMenu => "contextmenu",
            EventType::KeyDown => "keydown",
            EventType::KeyUp => "keyup",
            EventType::KeyPress => "keypress",
            EventType::Input => "input",
            EventType::Change => "change",
            EventType::Submit => "submit",
            EventType::Reset => "reset",
            EventType::Focus => "focus",
            EventType::Blur => "blur",
            EventType::Load => "load",
            EventType::Unload => "unload",
            EventType::Resize => "resize",
            EventType::Scroll => "scroll",
            EventType::Drag => "drag",
            EventType::DragStart => "dragstart",
            EventType::DragEnd => "dragend",
            EventType::DragOver => "dragover",
            EventType::Drop => "drop",
            EventType::Copy => "copy",
            EventType::Cut => "cut",
            EventType::Paste => "paste",
            EventType::TouchStart => "touchstart",
            EventType::TouchMove => "touchmove",
            EventType::TouchEnd => "touchend",
            EventType::Custom => "custom",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "click" => Some(EventType::Click),
            "dblclick" => Some(EventType::DblClick),
            "mousedown" => Some(EventType::MouseDown),
            "mouseup" => Some(EventType::MouseUp),
            "mousemove" => Some(EventType::MouseMove),
            "mouseover" => Some(EventType::MouseOver),
            "mouseout" => Some(EventType::MouseOut),
            "mouseenter" => Some(EventType::MouseEnter),
            "mouseleave" => Some(EventType::MouseLeave),
            "contextmenu" => Some(EventType::ContextMenu),
            "keydown" => Some(EventType::KeyDown),
            "keyup" => Some(EventType::KeyUp),
            "keypress" => Some(EventType::KeyPress),
            "input" => Some(EventType::Input),
            "change" => Some(EventType::Change),
            "submit" => Some(EventType::Submit),
            "reset" => Some(EventType::Reset),
            "focus" => Some(EventType::Focus),
            "blur" => Some(EventType::Blur),
            "load" => Some(EventType::Load),
            "unload" => Some(EventType::Unload),
            "resize" => Some(EventType::Resize),
            "scroll" => Some(EventType::Scroll),
            "drag" => Some(EventType::Drag),
            "dragstart" => Some(EventType::DragStart),
            "dragend" => Some(EventType::DragEnd),
            "dragover" => Some(EventType::DragOver),
            "drop" => Some(EventType::Drop),
            "copy" => Some(EventType::Copy),
            "cut" => Some(EventType::Cut),
            "paste" => Some(EventType::Paste),
            "touchstart" => Some(EventType::TouchStart),
            "touchmove" => Some(EventType::TouchMove),
            "touchend" => Some(EventType::TouchEnd),
            _ => None,
        }
    }
}

/// Event listener registration
#[derive(Debug)]
pub struct EventListener {
    pub element_id: u32,
    pub event_type: EventType,
    pub callback_id: u32,
    pub capture: bool,
}

/// Event handler callback
pub type EventCallback = Box<dyn Fn(EventData)>;

/// Event data passed to callbacks
#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: EventType,
    pub target_id: u32,
    pub timestamp: u64,
    pub data: Option<EventSpecificData>,
}

/// Event-specific data
#[derive(Debug, Clone)]
pub enum EventSpecificData {
    Mouse {
        x: i32,
        y: i32,
        button: u8,
        ctrl: bool,
        shift: bool,
        alt: bool,
    },
    Keyboard {
        key: String,
        code: String,
        ctrl: bool,
        shift: bool,
        alt: bool,
    },
    Input {
        value: String,
    },
    Drag {
        x: i32,
        y: i32,
        data: String,
    },
}

/// Event registry for tracking listeners
pub struct EventRegistry {
    listeners: HashMap<u32, Vec<EventListener>>,
    callbacks: HashMap<u32, EventCallback>,
    next_callback_id: u32,
}

impl EventRegistry {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            callbacks: HashMap::new(),
            next_callback_id: 1,
        }
    }
    
    /// Register event listener
    pub fn add_listener(
        &mut self,
        element_id: u32,
        event_type: EventType,
        callback: EventCallback,
    ) -> u32 {
        let callback_id = self.next_callback_id;
        self.next_callback_id += 1;
        
        self.callbacks.insert(callback_id, callback);
        
        let listener = EventListener {
            element_id,
            event_type,
            callback_id,
            capture: false,
        };
        
        self.listeners
            .entry(element_id)
            .or_insert_with(Vec::new)
            .push(listener);
        
        callback_id
    }
    
    /// Remove event listener
    pub fn remove_listener(&mut self, element_id: u32, callback_id: u32) {
        if let Some(listeners) = self.listeners.get_mut(&element_id) {
            listeners.retain(|l| l.callback_id != callback_id);
        }
        self.callbacks.remove(&callback_id);
    }
    
    /// Remove all listeners for an element
    pub fn remove_element_listeners(&mut self, element_id: u32) {
        if let Some(listeners) = self.listeners.remove(&element_id) {
            for listener in listeners {
                self.callbacks.remove(&listener.callback_id);
            }
        }
    }
    
    /// Get listeners for an element
    pub fn get_listeners(&self, element_id: u32) -> &[EventListener] {
        self.listeners.get(&element_id).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    /// Invoke callback
    pub fn invoke(&self, callback_id: u32, event: EventData) {
        if let Some(callback) = self.callbacks.get(&callback_id) {
            callback(event);
        }
    }
    
    /// Get callback count
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Event delegation helper
pub struct EventDelegation {
    registry: EventRegistry,
    delegated_events: HashMap<EventType, u32>,
}

impl EventDelegation {
    pub fn new() -> Self {
        Self {
            registry: EventRegistry::new(),
            delegated_events: HashMap::new(),
        }
    }
    
    /// Set up event delegation on a container
    pub fn delegate(&mut self, container_id: u32, event_type: EventType) {
        if !self.delegated_events.contains_key(&event_type) {
            let event_type_copy = event_type;
            self.registry.add_listener(
                container_id,
                event_type,
                Box::new(move |event| {
                    // Handle delegated event
                    // In real impl, would check target and bubble
                    eprintln!("Delegated {:?} event on {:?}", event_type_copy, event.target_id);
                }),
            );
            self.delegated_events.insert(event_type, container_id);
        }
    }
    
    /// Add delegated listener
    pub fn add_delegated(
        &mut self,
        selector: &str,
        event_type: EventType,
        callback: EventCallback,
    ) {
        // In real impl, would match selector against event targets
        let callback_id = self.registry.next_callback_id;
        self.registry.next_callback_id += 1;
        self.registry.callbacks.insert(callback_id, callback);
    }
}

impl Default for EventDelegation {
    fn default() -> Self {
        Self::new()
    }
}

/// Common event handler patterns
pub mod handlers {
    use super::*;
    
    /// Click handler with coordinates
    pub fn on_click<F>(callback: F) -> EventCallback
    where
        F: Fn(i32, i32) + 'static,
    {
        Box::new(move |event| {
            if let Some(EventSpecificData::Mouse { x, y, .. }) = event.data {
                callback(x, y);
            }
        })
    }
    
    /// Key handler
    pub fn on_key<F>(callback: F) -> EventCallback
    where
        F: Fn(String) + 'static,
    {
        Box::new(move |event| {
            if let Some(EventSpecificData::Keyboard { key, .. }) = event.data {
                callback(key);
            }
        })
    }
    
    /// Input handler
    pub fn on_input<F>(callback: F) -> EventCallback
    where
        F: Fn(String) + 'static,
    {
        Box::new(move |event| {
            if let Some(EventSpecificData::Input { value }) = event.data {
                callback(value);
            }
        })
    }
    
    /// Submit handler
    pub fn on_submit<F>(callback: F) -> EventCallback
    where
        F: Fn() + 'static,
    {
        Box::new(move |_| {
            callback();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_type_conversion() {
        assert_eq!(EventType::Click.as_str(), "click");
        assert_eq!(EventType::from_str("click"), Some(EventType::Click));
        assert_eq!(EventType::from_str("invalid"), None);
    }
    
    #[test]
    fn test_event_registry() {
        let mut registry = EventRegistry::new();
        
        let callback_id = registry.add_listener(
            1,
            EventType::Click,
            Box::new(|_| {}),
        );
        
        assert_eq!(registry.callback_count(), 1);
        assert_eq!(registry.get_listeners(1).len(), 1);
        
        registry.remove_listener(1, callback_id);
        assert_eq!(registry.callback_count(), 0);
    }
}
