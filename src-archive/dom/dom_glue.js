/**
 * Hint WASM DOM Glue Code
 * 
 * This JavaScript module provides the bridge between Hint WASM modules
 * and the browser DOM API. It handles:
 * - Element creation and manipulation
 * - Event handling and callbacks
 * - Memory management for strings/arrays
 * - Style and attribute operations
 * 
 * Usage:
 * ```javascript
 * import { createHintDOMBridge } from './hint_dom.js';
 * 
 * const imports = createHintDOMBridge(wasmMemory);
 * const instance = await WebAssembly.instantiate(wasmModule, imports);
 * ```
 */

// ============================================================================
// Memory Management
// ============================================================================

/** @type {WebAssembly.Memory} */
let wasmMemory;

/** @type {Map<number, string>} */
const stringCache = new Map();

/** @type {Map<number, any[]>} */
const arrayCache = new Map();

/** @type {Map<number, Element>} */
const elementRegistry = new Map();

let nextStringId = 1;
let nextArrayId = 1;
let nextElementId = 1;

/**
 * Initialize the DOM bridge with WASM memory
 * @param {WebAssembly.Memory} memory 
 */
export function initializeWasmMemory(memory) {
    wasmMemory = memory;
}

/**
 * Get Uint8Array view of WASM memory
 * @param {number} ptr 
 * @param {number} len 
 * @returns {Uint8Array}
 */
function getMemoryView(ptr, len) {
    return new Uint8Array(wasmMemory.buffer, ptr, len);
}

/**
 * Read string from WASM memory
 * @param {number} ptr 
 * @param {number} len 
 * @returns {string}
 */
function readString(ptr, len) {
    const bytes = getMemoryView(ptr, len);
    return new TextDecoder().decode(bytes);
}

/**
 * Write string to WASM memory, return pointer
 * @param {string} str 
 * @returns {{ptr: number, len: number}}
 */
function writeString(str) {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);
    const ptr = allocate_memory(bytes.length);
    getMemoryView(ptr, bytes.length).set(bytes);
    return { ptr, len: bytes.length };
}

/**
 * Allocate memory in WASM heap
 * @param {number} size 
 * @returns {number}
 */
export function allocate_memory(size) {
    // In production, this would call the WASM export
    // For now, use a simple bump allocator
    if (!wasmMemory) {
        throw new Error('WASM memory not initialized');
    }
    
    // Simple allocation: start at 64KB (after stack)
    const heapStart = 64 * 1024;
    const allocPtr = heapStart + (nextStringId * 1024);
    nextStringId++;
    return allocPtr;
}

// ============================================================================
// Element Registry
// ============================================================================

/**
 * Register an element and return its ID
 * @param {Element} element 
 * @returns {number}
 */
function registerElement(element) {
    const id = nextElementId++;
    elementRegistry.set(id, element);
    return id;
}

/**
 * Get element by ID
 * @param {number} id 
 * @returns {Element|undefined}
 */
function getElement(id) {
    return elementRegistry.get(id);
}

/**
 * Unregister element
 * @param {number} id 
 */
function unregisterElement(id) {
    elementRegistry.delete(id);
}

// ============================================================================
// Element Creation
// ============================================================================

/**
 * Create DOM element by tag name
 * @param {number} tagPtr 
 * @param {number} tagLen 
 * @returns {number} Element ID
 */
export function create_element(tagPtr, tagLen) {
    const tag = readString(tagPtr, tagLen);
    const element = document.createElement(tag);
    return registerElement(element);
}

/**
 * Create text node
 * @param {number} textPtr 
 * @param {number} textLen 
 * @returns {number} Element ID
 */
export function create_text(textPtr, textLen) {
    const text = readString(textPtr, textLen);
    const node = document.createTextNode(text);
    return registerElement(node);
}

/**
 * Create button element
 * @returns {number} Element ID
 */
export function create_button() {
    const btn = document.createElement('button');
    return registerElement(btn);
}

/**
 * Create div element
 * @returns {number} Element ID
 */
export function create_div() {
    const div = document.createElement('div');
    return registerElement(div);
}

/**
 * Create input element
 * @param {number} typePtr 
 * @param {number} typeLen 
 * @returns {number} Element ID
 */
export function create_input(typePtr, typeLen) {
    const type = readString(typePtr, typeLen);
    const input = document.createElement('input');
    input.type = type;
    return registerElement(input);
}

// ============================================================================
// Element Content Manipulation
// ============================================================================

/**
 * Set element inner HTML
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} contentPtr 
 * @param {number} contentLen 
 */
export function set_inner_html(elemId, _elemLen, contentPtr, contentLen) {
    const element = getElement(elemId);
    if (element) {
        const content = readString(contentPtr, contentLen);
        element.innerHTML = content;
    }
}

/**
 * Set element text content
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} contentPtr 
 * @param {number} contentLen 
 */
export function set_text_content(elemId, _elemLen, contentPtr, contentLen) {
    const element = getElement(elemId);
    if (element) {
        const content = readString(contentPtr, contentLen);
        element.textContent = content;
    }
}

/**
 * Get element inner HTML
 * @param {number} elemId 
 * @returns {{ptr: number, len: number}} String pointer/length
 */
export function get_inner_html(elemId) {
    const element = getElement(elemId);
    if (element) {
        return writeString(element.innerHTML);
    }
    return writeString('');
}

/**
 * Get element text content
 * @param {number} elemId 
 * @returns {{ptr: number, len: number}} String pointer/length
 */
export function get_text_content(elemId) {
    const element = getElement(elemId);
    if (element) {
        return writeString(element.textContent || '');
    }
    return writeString('');
}

// ============================================================================
// Style Manipulation
// ============================================================================

/**
 * Set CSS style property
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} propPtr 
 * @param {number} propLen 
 * @param {number} valuePtr 
 * @param {number} valueLen 
 */
export function set_style(elemId, _elemLen, propPtr, propLen, valuePtr, valueLen) {
    const element = getElement(elemId);
    if (element) {
        const prop = readString(propPtr, propLen);
        const value = readString(valuePtr, valueLen);
        element.style[prop] = value;
    }
}

/**
 * Get CSS style property
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} propPtr 
 * @param {number} propLen 
 * @returns {{ptr: number, len: number}} String pointer/length
 */
export function get_style(elemId, _elemLen, propPtr, propLen) {
    const element = getElement(elemId);
    if (element) {
        const prop = readString(propPtr, propLen);
        const value = getComputedStyle(element)[prop] || element.style[prop] || '';
        return writeString(value);
    }
    return writeString('');
}

/**
 * Set CSS class
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} classPtr 
 * @param {number} classLen 
 */
export function set_class(elemId, _elemLen, classPtr, classLen) {
    const element = getElement(elemId);
    if (element) {
        const className = readString(classPtr, classLen);
        element.className = className;
    }
}

/**
 * Add CSS class
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} classPtr 
 * @param {number} classLen 
 */
export function add_class(elemId, _elemLen, classPtr, classLen) {
    const element = getElement(elemId);
    if (element) {
        const className = readString(classPtr, classLen);
        element.classList.add(className);
    }
}

/**
 * Remove CSS class
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} classPtr 
 * @param {number} classLen 
 */
export function remove_class(elemId, _elemLen, classPtr, classLen) {
    const element = getElement(elemId);
    if (element) {
        const className = readString(classPtr, classLen);
        element.classList.remove(className);
    }
}

/**
 * Check if element has CSS class
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} classPtr 
 * @param {number} classLen 
 * @returns {number} 1 if has class, 0 otherwise
 */
export function has_class(elemId, _elemLen, classPtr, classLen) {
    const element = getElement(elemId);
    if (element) {
        const className = readString(classPtr, classLen);
        return element.classList.contains(className) ? 1 : 0;
    }
    return 0;
}

// ============================================================================
// Attribute Manipulation
// ============================================================================

/**
 * Set HTML attribute
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} namePtr 
 * @param {number} nameLen 
 * @param {number} valuePtr 
 * @param {number} valueLen 
 */
export function set_attribute(elemId, _elemLen, namePtr, nameLen, valuePtr, valueLen) {
    const element = getElement(elemId);
    if (element) {
        const name = readString(namePtr, nameLen);
        const value = readString(valuePtr, valueLen);
        element.setAttribute(name, value);
    }
}

/**
 * Get HTML attribute
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} namePtr 
 * @param {number} nameLen 
 * @returns {{ptr: number, len: number}} String pointer/length
 */
export function get_attribute(elemId, _elemLen, namePtr, nameLen) {
    const element = getElement(elemId);
    if (element) {
        const name = readString(namePtr, nameLen);
        const value = element.getAttribute(name) || '';
        return writeString(value);
    }
    return writeString('');
}

/**
 * Remove HTML attribute
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} namePtr 
 * @param {number} nameLen 
 */
export function remove_attribute(elemId, _elemLen, namePtr, nameLen) {
    const element = getElement(elemId);
    if (element) {
        const name = readString(namePtr, nameLen);
        element.removeAttribute(name);
    }
}

// ============================================================================
// DOM Tree Manipulation
// ============================================================================

/**
 * Append child element
 * @param {number} parentId 
 * @param {number} childId 
 */
export function append_child(parentId, childId) {
    const parent = getElement(parentId);
    const child = getElement(childId);
    if (parent && child) {
        parent.appendChild(child);
    }
}

/**
 * Prepend child element
 * @param {number} parentId 
 * @param {number} childId 
 */
export function prepend_child(parentId, childId) {
    const parent = getElement(parentId);
    const child = getElement(childId);
    if (parent && child) {
        parent.insertBefore(child, parent.firstChild);
    }
}

/**
 * Remove child element
 * @param {number} parentId 
 * @param {number} childId 
 */
export function remove_child(parentId, childId) {
    const parent = getElement(parentId);
    const child = getElement(childId);
    if (parent && child) {
        parent.removeChild(child);
        unregisterElement(childId);
    }
}

/**
 * Replace child element
 * @param {number} parentId 
 * @param {number} oldChildId 
 * @param {number} newChildId 
 */
export function replace_child(parentId, oldChildId, newChildId) {
    const parent = getElement(parentId);
    const oldChild = getElement(oldChildId);
    const newChild = getElement(newChildId);
    if (parent && oldChild && newChild) {
        parent.replaceChild(newChild, oldChild);
        unregisterElement(oldChildId);
    }
}

/**
 * Remove element from DOM
 * @param {number} elemId 
 */
export function remove(elemId) {
    const element = getElement(elemId);
    if (element && element.parentNode) {
        element.parentNode.removeChild(element);
        unregisterElement(elemId);
    }
}

// ============================================================================
// Query/Selection
// ============================================================================

/**
 * Query single element by CSS selector
 * @param {number} selectorPtr 
 * @param {number} selectorLen 
 * @returns {number} Element ID or 0
 */
export function query_selector(selectorPtr, selectorLen) {
    const selector = readString(selectorPtr, selectorLen);
    const element = document.querySelector(selector);
    return element ? registerElement(element) : 0;
}

/**
 * Query all elements by CSS selector
 * @param {number} selectorPtr 
 * @param {number} selectorLen 
 * @returns {{ptr: number, len: number}} Array pointer/length
 */
export function query_selector_all(selectorPtr, selectorLen) {
    const selector = readString(selectorPtr, selectorLen);
    const elements = document.querySelectorAll(selector);
    const ids = Array.from(elements).map(el => registerElement(el));
    return allocateElementArray(ids);
}

/**
 * Get element by ID
 * @param {number} idPtr 
 * @param {number} idLen 
 * @returns {number} Element ID or 0
 */
export function get_element_by_id(idPtr, idLen) {
    const id = readString(idPtr, idLen);
    const element = document.getElementById(id);
    return element ? registerElement(element) : 0;
}

/**
 * Allocate array of element IDs in WASM memory
 * @param {number[]} ids 
 * @returns {{ptr: number, len: number}}
 */
function allocateElementArray(ids) {
    const arrayId = nextArrayId++;
    arrayCache.set(arrayId, ids);
    
    // Store array ID as pointer (simplified)
    return { ptr: arrayId, len: ids.length };
}

// ============================================================================
// Event Handling
// ============================================================================

/** @type {Map<number, Map<string, Function>>} */
const eventListeners = new Map();

/** @type {Map<number, Function>} */
const hintEventCallbacks = new Map();

/**
 * Add event listener to element
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} eventPtr 
 * @param {number} eventLen 
 * @param {number} callbackId 
 */
export function add_event_listener(elemId, _elemLen, eventPtr, eventLen, callbackId) {
    const element = getElement(elemId);
    if (!element) return;
    
    const eventType = readString(eventPtr, eventLen);
    
    // Store callback reference
    hintEventCallbacks.set(callbackId, () => {
        // Call WASM callback
        const instance = getWasmInstance();
        if (instance && instance.exports.handle_event) {
            instance.exports.handle_event(callbackId);
        }
    });
    
    const listener = (e) => {
        const callback = hintEventCallbacks.get(callbackId);
        if (callback) {
            callback();
        }
    };
    
    element.addEventListener(eventType, listener);
    
    // Track listeners for cleanup
    if (!eventListeners.has(elemId)) {
        eventListeners.set(elemId, new Map());
    }
    eventListeners.get(elemId).set(`${eventType}_${callbackId}`, listener);
}

/**
 * Remove event listener
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} eventPtr 
 * @param {number} eventLen 
 * @param {number} callbackId 
 */
export function remove_event_listener(elemId, _elemLen, eventPtr, eventLen, callbackId) {
    const element = getElement(elemId);
    if (!element) return;
    
    const eventType = readString(eventPtr, eventLen);
    const listenerKey = `${eventType}_${callbackId}`;
    
    const listeners = eventListeners.get(elemId);
    if (listeners) {
        const listener = listeners.get(listenerKey);
        if (listener) {
            element.removeEventListener(eventType, listener);
            listeners.delete(listenerKey);
        }
    }
    
    hintEventCallbacks.delete(callbackId);
}

/**
 * Dispatch event from element
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} eventPtr 
 * @param {number} eventLen 
 * @returns {number} 1 if event was dispatched, 0 otherwise
 */
export function dispatch_event(elemId, _elemLen, eventPtr, eventLen) {
    const element = getElement(elemId);
    if (!element) return 0;
    
    const eventType = readString(eventPtr, eventLen);
    const event = new Event(eventType, { bubbles: true, cancelable: true });
    return element.dispatchEvent(event) ? 1 : 0;
}

// ============================================================================
// Form/Input Handling
// ============================================================================

/**
 * Get input element value
 * @param {number} elemId 
 * @returns {{ptr: number, len: number}} String pointer/length
 */
export function get_value(elemId) {
    const element = getElement(elemId);
    if (element && 'value' in element) {
        return writeString(element.value);
    }
    return writeString('');
}

/**
 * Set input element value
 * @param {number} elemId 
 * @param {number} _elemLen 
 * @param {number} valuePtr 
 * @param {number} valueLen 
 */
export function set_value(elemId, _elemLen, valuePtr, valueLen) {
    const element = getElement(elemId);
    if (element && 'value' in element) {
        const value = readString(valuePtr, valueLen);
        element.value = value;
    }
}

/**
 * Focus element
 * @param {number} elemId 
 */
export function focus(elemId) {
    const element = getElement(elemId);
    if (element && typeof element.focus === 'function') {
        element.focus();
    }
}

/**
 * Blur element
 * @param {number} elemId 
 */
export function blur(elemId) {
    const element = getElement(elemId);
    if (element && typeof element.blur === 'function') {
        element.blur();
    }
}

// ============================================================================
// Document/Window
// ============================================================================

/**
 * Get document object
 * @returns {number} Element ID
 */
export function get_document() {
    return registerElement(document);
}

/**
 * Get window object (wrapped)
 * @returns {number} Element ID
 */
export function get_window() {
    // Window is not an Element, so we create a wrapper
    const wrapper = { _isWindow: true, windowObject: window };
    return registerElement(wrapper);
}

/**
 * Show alert dialog
 * @param {number} msgPtr 
 * @param {number} msgLen 
 */
export function alert(msgPtr, msgLen) {
    const msg = readString(msgPtr, msgLen);
    window.alert(msg);
}

/**
 * Log to browser console
 * @param {number} msgPtr 
 * @param {number} msgLen 
 */
export function console_log(msgPtr, msgLen) {
    const msg = readString(msgPtr, msgLen);
    console.log('[Hint]', msg);
}

// ============================================================================
// WASM Instance Access
// ============================================================================

/** @type {WebAssembly.Instance|null} */
let wasmInstance = null;

/**
 * Set WASM instance reference
 * @param {WebAssembly.Instance} instance 
 */
export function setWasmInstance(instance) {
    wasmInstance = instance;
}

/**
 * Get WASM instance
 * @returns {WebAssembly.Instance|null}
 */
export function getWasmInstance() {
    return wasmInstance;
}

// ============================================================================
// Bridge Creation
// ============================================================================

/**
 * Create complete WASM import object for Hint DOM
 * @param {WebAssembly.Memory} memory 
 * @returns {WebAssembly.Imports}
 */
export function createHintDOMBridge(memory) {
    initializeWasmMemory(memory);
    
    return {
        hint_dom: {
            // Element Creation
            create_element,
            create_text,
            create_button,
            create_div,
            create_input,
            
            // Element Manipulation
            set_inner_html,
            set_text_content,
            get_inner_html,
            get_text_content,
            
            // Style
            set_style,
            get_style,
            set_class,
            add_class,
            remove_class,
            has_class,
            
            // Attributes
            set_attribute,
            get_attribute,
            remove_attribute,
            
            // Tree
            append_child,
            prepend_child,
            remove_child,
            replace_child,
            remove,
            
            // Query
            query_selector,
            query_selector_all,
            get_element_by_id,
            
            // Events
            add_event_listener,
            remove_event_listener,
            dispatch_event,
            
            // Form
            get_value,
            set_value,
            focus,
            blur,
            
            // Document/Window
            get_document,
            get_window,
            alert,
            console_log,
            
            // Memory
            allocate_memory,
        }
    };
}

// Export all functions for direct import
export {
    create_element,
    create_text,
    create_button,
    create_div,
    create_input,
    set_inner_html,
    set_text_content,
    get_inner_html,
    get_text_content,
    set_style,
    get_style,
    set_class,
    add_class,
    remove_class,
    has_class,
    set_attribute,
    get_attribute,
    remove_attribute,
    append_child,
    prepend_child,
    remove_child,
    replace_child,
    remove,
    query_selector,
    query_selector_all,
    get_element_by_id,
    add_event_listener,
    remove_event_listener,
    dispatch_event,
    get_value,
    set_value,
    focus,
    blur,
    get_document,
    get_window,
    alert,
    console_log,
    allocate_memory,
};
