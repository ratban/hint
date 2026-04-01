//! Stack Allocator for primitive types
//! 
//! Manages stack frame layout and local variable allocation.

use crate::semantics::HintType;

/// Stack frame manager
pub struct StackAllocator {
    /// Stack limit in bytes
    limit: usize,
    /// Current stack pointer offset
    sp_offset: usize,
    /// Allocated slots
    slots: Vec<StackSlot>,
    /// Frame pointer
    fp: usize,
}

/// Stack slot information
#[derive(Debug, Clone)]
pub struct StackSlot {
    pub name: String,
    pub var_type: HintType,
    pub offset: i64,
    pub size: usize,
}

impl StackAllocator {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            sp_offset: 0,
            slots: Vec::new(),
            fp: 0,
        }
    }
    
    /// Allocate a stack slot for a local variable
    pub fn allocate_slot(&mut self, name: &str, var_type: &HintType) -> Result<i64, String> {
        let size = self.type_size(var_type);
        let align = self.type_alignment(var_type);
        
        // Align stack pointer
        let aligned_sp = (self.sp_offset + align - 1) & !(align - 1);
        
        if aligned_sp + size > self.limit {
            return Err(format!("Stack overflow: need {} bytes, limit is {}", 
                aligned_sp + size, self.limit));
        }
        
        let offset = -(aligned_sp as i64) - (size as i64);
        
        self.slots.push(StackSlot {
            name: name.to_string(),
            var_type: var_type.clone(),
            offset,
            size,
        });
        
        self.sp_offset = aligned_sp + size;
        
        Ok(offset)
    }
    
    /// Get slot by name
    pub fn get_slot(&self, name: &str) -> Option<&StackSlot> {
        self.slots.iter().find(|s| s.name == name)
    }
    
    /// Get slot offset by name
    pub fn get_offset(&self, name: &str) -> Option<i64> {
        self.get_slot(name).map(|s| s.offset)
    }
    
    /// Begin new frame
    pub fn begin_frame(&mut self) {
        self.fp = self.sp_offset;
    }
    
    /// End current frame, deallocate all slots
    pub fn end_frame(&mut self) {
        self.sp_offset = self.fp;
        self.slots.retain(|s| s.offset.abs() as usize <= self.fp);
    }
    
    /// Get current stack usage
    pub fn usage(&self) -> usize {
        self.sp_offset
    }
    
    /// Get remaining stack space
    pub fn remaining(&self) -> usize {
        self.limit - self.sp_offset
    }
    
    /// Get all slots
    pub fn slots(&self) -> &[StackSlot] {
        &self.slots
    }
    
    fn type_size(&self, ty: &HintType) -> usize {
        match ty {
            HintType::Int(size) | HintType::UInt(size) => size.bytes(),
            HintType::Float(size) => size.bytes(),
            HintType::Bool => 1,
            HintType::Pointer(_) => 8,
            HintType::String => 24, // ptr + len + capacity
            HintType::Array(elem, len) => self.type_size(elem) * len,
            _ => 8,
        }
    }
    
    fn type_alignment(&self, ty: &HintType) -> usize {
        self.type_size(ty).min(8)
    }
}

/// Stack frame builder for codegen
pub struct StackFrameBuilder {
    slots: Vec<StackSlot>,
    frame_size: usize,
    has_frame_pointer: bool,
}

impl StackFrameBuilder {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            frame_size: 0,
            has_frame_pointer: true,
        }
    }
    
    pub fn with_frame_pointer(mut self, has: bool) -> Self {
        self.has_frame_pointer = has;
        self
    }
    
    pub fn add_slot(&mut self, name: &str, var_type: &HintType, size: usize, align: usize) -> i64 {
        let offset = -(self.frame_size as i64) - (size as i64);
        
        // Align
        let aligned_offset = (offset as usize / align) * align;
        
        self.slots.push(StackSlot {
            name: name.to_string(),
            var_type: var_type.clone(),
            offset: -(aligned_offset as i64) - (size as i64),
            size,
        });
        
        self.frame_size = aligned_offset + size;
        
        self.slots.last().unwrap().offset
    }
    
    pub fn frame_size(&self) -> usize {
        // Add space for saved frame pointer and return address
        self.frame_size + 16
    }
    
    pub fn slots(&self) -> &[StackSlot] {
        &self.slots
    }
    
    pub fn build(self) -> StackFrame {
        StackFrame {
            slots: self.slots,
            frame_size: self.frame_size(),
            has_frame_pointer: self.has_frame_pointer,
        }
    }
}

impl Default for StackFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Finalized stack frame
#[derive(Debug)]
pub struct StackFrame {
    pub slots: Vec<StackSlot>,
    pub frame_size: usize,
    pub has_frame_pointer: bool,
}

impl StackFrame {
    /// Get prologue instructions (as strings for codegen)
    pub fn prologue(&self) -> Vec<&'static str> {
        let mut prologue = Vec::new();
        
        if self.has_frame_pointer {
            prologue.push("push rbp");
            prologue.push("mov rbp, rsp");
        }
        
        if self.frame_size > 0 {
            prologue.push(&format!("sub rsp, {}", self.frame_size));
        }
        
        prologue
    }
    
    /// Get epilogue instructions
    pub fn epilogue(&self) -> Vec<&'static str> {
        let mut epilogue = Vec::new();
        
        if self.frame_size > 0 {
            epilogue.push(&format!("add rsp, {}", self.frame_size));
        }
        
        if self.has_frame_pointer {
            epilogue.push("pop rbp");
        }
        
        epilogue.push("ret");
        
        epilogue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantics::IntSize;
    
    #[test]
    fn test_stack_allocation() {
        let mut stack = StackAllocator::new(1024 * 1024); // 1MB stack
        
        let offset1 = stack.allocate_slot("x", &HintType::Int(IntSize::I64)).unwrap();
        let offset2 = stack.allocate_slot("y", &HintType::Int(IntSize::I64)).unwrap();
        
        // Stack grows downward
        assert!(offset2 < offset1);
        
        assert_eq!(stack.get_offset("x"), Some(offset1));
        assert_eq!(stack.get_offset("y"), Some(offset2));
    }
    
    #[test]
    fn test_stack_frame() {
        let mut builder = StackFrameBuilder::new();
        
        builder.add_slot("a", &HintType::Int(IntSize::I64), 8, 8);
        builder.add_slot("b", &HintType::Int(IntSize::I64), 8, 8);
        
        let frame = builder.build();
        
        assert_eq!(frame.slots.len(), 2);
        assert!(frame.frame_size >= 32); // 2 slots + saved rbp + return addr
    }
}
