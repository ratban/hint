//! Stack frame layout - Cranelift 0.110 compatible.

use cranelift_codegen::ir::{StackSlot, StackSlotData, StackSlotKind, Type};
use cranelift_frontend::FunctionBuilder;

/// Stack frame for a function
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub locals: Vec<LocalSlot>,
    pub spill_slots: Vec<SpillSlot>,
    pub frame_size: usize,
    pub alignment: usize,
}

#[derive(Debug, Clone)]
pub struct LocalSlot {
    pub name: String,
    pub slot: StackSlot,
    pub ty: Type,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct SpillSlot {
    pub slot: StackSlot,
    pub ty: Type,
    pub offset: i64,
}

impl StackFrame {
    pub fn new(alignment: usize) -> Self {
        Self {
            locals: Vec::new(),
            spill_slots: Vec::new(),
            frame_size: 0,
            alignment,
        }
    }
    
    pub fn allocate_local(&mut self, builder: &mut FunctionBuilder, name: &str, ty: Type) -> StackSlot {
        let size = ty.bytes() as u32;
        let slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, size, 0));
        
        self.locals.push(LocalSlot {
            name: name.to_string(),
            slot,
            ty,
            offset: 0,
        });
        
        slot
    }
    
    pub fn allocate_spill(&mut self, builder: &mut FunctionBuilder, ty: Type) -> StackSlot {
        let size = ty.bytes() as u32;
        // Use ExplicitSlot instead of SpillSlot (API changed in 0.110)
        let slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, size, 0));
        
        self.spill_slots.push(SpillSlot { slot, ty, offset: 0 });
        slot
    }
    
    pub fn finalize(&mut self) {
        let mut offset = 0i64;
        
        for spill in &mut self.spill_slots {
            spill.offset = offset;
            offset += spill.ty.bytes() as i64;
        }
        
        for local in &mut self.locals {
            local.offset = offset;
            offset += local.ty.bytes() as i64;
        }
        
        offset += 16; // Return address + saved rbp
        self.frame_size = ((offset as usize) + (self.alignment - 1)) & !(self.alignment - 1);
    }
}
