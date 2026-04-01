//! Memory Management System for Hint
//! 
//! This module implements Automatic Reference Counting (ARC) for heap-allocated
//! objects, with stack allocation for primitive types.
//! 
//! # Memory Layout
//! 
//! Ref-counted heap objects:
//! ```text
//! +-------------+-------------+-------------+
//! | ref_count   | type_info   | data...     |
//! | (4 bytes)   | (4 bytes)   | (variable)  |
//! +-------------+-------------+-------------+
//! ```
//! 
//! String objects:
//! ```text
//! +-------------+-------------+-------------+-------------+
//! | ref_count   | length      | capacity    | data...     |
//! | (4 bytes)   | (4 bytes)   | (4 bytes)   | (variable)  |
//! +-------------+-------------+-------------+-------------+
//! ```
//! 
//! # Allocation Strategy
//! 
//! - **Stack**: i8, i16, i32, i64, f32, f64, bool (primitives)
//! - **Heap**: String, Array, Struct, Closure (complex types)

pub mod arc;
pub mod alloc;
pub mod gc_hooks;
pub mod stack;

pub use arc::{ArcRuntime, ArcObject, RcHeader};
pub use alloc::{AllocationStrategy, MemoryAllocator};
pub use gc_hooks::{CycleDetector, GCHooks};
pub use stack::StackAllocator;

use crate::semantics::HintType;
use crate::diagnostics::{Diagnostic, DiagnosticsEngine};

/// Memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Enable cycle detection
    pub enable_cycle_detection: bool,
    /// Enable memory tracking for debugging
    pub enable_memory_tracking: bool,
    /// Threshold for triggering cycle detection
    pub cycle_detection_threshold: usize,
    /// Stack size limit (bytes)
    pub stack_size_limit: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_cycle_detection: true,
            enable_memory_tracking: false,
            cycle_detection_threshold: 1000,
            stack_size_limit: 8 * 1024 * 1024, // 8 MB
        }
    }
}

/// Memory manager for Hint programs
pub struct MemoryManager {
    /// ARC runtime for ref-counted objects
    arc: ArcRuntime,
    /// Stack allocator for primitives
    stack: StackAllocator,
    /// Optional cycle detector
    cycle_detector: Option<CycleDetector>,
    /// Configuration
    config: MemoryConfig,
    /// Allocation statistics
    stats: MemoryStats,
}

/// Memory allocation statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub current_heap_usage: usize,
    pub peak_heap_usage: usize,
    pub stack_allocations: usize,
    pub heap_allocations: usize,
    pub ref_count_ops: usize,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: MemoryConfig) -> Self {
        let cycle_detector = if config.enable_cycle_detection {
            Some(CycleDetector::new())
        } else {
            None
        };
        
        Self {
            arc: ArcRuntime::new(),
            stack: StackAllocator::new(config.stack_size_limit),
            cycle_detector,
            config,
            stats: MemoryStats::default(),
        }
    }
    
    /// Get allocation strategy for a type
    pub fn allocation_strategy(&self, ty: &HintType) -> AllocationStrategy {
        match ty {
            // Stack-allocated primitives
            HintType::Int(_) | HintType::UInt(_) | 
            HintType::Float(_) | HintType::Bool => {
                AllocationStrategy::Stack
            }
            
            // Heap-allocated with ARC
            HintType::String | HintType::Array(_, _) | 
            HintType::Struct(_, _) | HintType::Function(_, _) => {
                AllocationStrategy::ArcHeap
            }
            
            // Pointers are just addresses
            HintType::Pointer(_) => AllocationStrategy::Stack,
            
            // Unknown defaults to heap for safety
            HintType::Void | HintType::Unknown => AllocationStrategy::ArcHeap,
        }
    }
    
    /// Allocate memory for a value
    pub fn allocate(&mut self, ty: &HintType, size: usize) -> Result<MemoryHandle, String> {
        self.stats.total_allocations += 1;
        
        match self.allocation_strategy(ty) {
            AllocationStrategy::Stack => {
                self.stats.stack_allocations += 1;
                let handle = self.stack.allocate(size)?;
                Ok(MemoryHandle::Stack(handle))
            }
            
            AllocationStrategy::ArcHeap => {
                self.stats.heap_allocations += 1;
                let handle = self.arc.allocate(size)?;
                self.stats.current_heap_usage += size;
                if self.stats.current_heap_usage > self.stats.peak_heap_usage {
                    self.stats.peak_heap_usage = self.stats.current_heap_usage;
                }
                Ok(MemoryHandle::Heap(handle))
            }
        }
    }
    
    /// Increment reference count
    pub fn incref(&mut self, handle: &MemoryHandle) {
        if let MemoryHandle::Heap(arc_handle) = handle {
            self.arc.incref(arc_handle);
            self.stats.ref_count_ops += 1;
        }
    }
    
    /// Decrement reference count, free if zero
    pub fn decref(&mut self, handle: &MemoryHandle) -> Result<(), String> {
        if let MemoryHandle::Heap(arc_handle) = handle {
            self.stats.ref_count_ops += 1;
            
            if let Some(freed_size) = self.arc.decref(arc_handle)? {
                self.stats.total_deallocations += 1;
                self.stats.current_heap_usage = self.stats.current_heap_usage.saturating_sub(freed_size);
            }
            
            // Check for cycles periodically
            if let Some(detector) = &mut self.cycle_detector {
                if self.stats.ref_count_ops % self.config.cycle_detection_threshold == 0 {
                    detector.check_for_cycles(&self.arc)?;
                }
            }
        }
        Ok(())
    }
    
    /// Get memory statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }
    
    /// Run garbage collection (cycle detection)
    pub fn gc(&mut self) -> Result<usize, String> {
        if let Some(detector) = &mut self.cycle_detector {
            let collected = detector.collect_cycles(&mut self.arc)?;
            self.stats.total_deallocations += collected;
            Ok(collected)
        } else {
            Ok(0)
        }
    }
    
    /// Print memory statistics (for debugging)
    pub fn print_stats(&self) {
        if self.config.enable_memory_tracking {
            eprintln!("=== Memory Statistics ===");
            eprintln!("Total allocations: {}", self.stats.total_allocations);
            eprintln!("Total deallocations: {}", self.stats.total_deallocations);
            eprintln!("Current heap usage: {} bytes", self.stats.current_heap_usage);
            eprintln!("Peak heap usage: {} bytes", self.stats.peak_heap_usage);
            eprintln!("Stack allocations: {}", self.stats.stack_allocations);
            eprintln!("Heap allocations: {}", self.stats.heap_allocations);
            eprintln!("Ref count operations: {}", self.stats.ref_count_ops);
            eprintln!("========================");
        }
    }
}

/// Handle to allocated memory
#[derive(Debug, Clone)]
pub enum MemoryHandle {
    /// Stack-allocated memory
    Stack(StackHandle),
    /// Heap-allocated ref-counted memory
    Heap(ArcHandle),
}

/// Stack handle (offset into stack frame)
#[derive(Debug, Clone, Copy)]
pub struct StackHandle {
    pub offset: i64,
    pub size: usize,
}

/// ARC handle (pointer to ref-counted object)
#[derive(Debug, Clone, Copy)]
pub struct ArcHandle {
    pub ptr: u64,
    pub size: usize,
}

/// Type-specific memory operations
pub trait MemoryOps {
    /// Type name for debugging
    fn type_name() -> &'static str;
    
    /// Size in bytes
    fn size() -> usize;
    
    /// Alignment requirement
    fn alignment() -> usize;
    
    /// Whether this type needs heap allocation
    fn needs_heap() -> bool;
}

impl MemoryOps for i64 {
    fn type_name() -> &'static str { "i64" }
    fn size() -> usize { 8 }
    fn alignment() -> usize { 8 }
    fn needs_heap() -> bool { false }
}

impl MemoryOps for String {
    fn type_name() -> &'static str { "String" }
    fn size() -> usize { 24 } // ptr + len + capacity
    fn alignment() -> usize { 8 }
    fn needs_heap() -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allocation_strategy() {
        let manager = MemoryManager::new(MemoryConfig::default());
        
        // Primitives should be stack-allocated
        assert_eq!(manager.allocation_strategy(&HintType::Int(IntSize::I64)), AllocationStrategy::Stack);
        assert_eq!(manager.allocation_strategy(&HintType::Float(FloatSize::F64)), AllocationStrategy::Stack);
        assert_eq!(manager.allocation_strategy(&HintType::Bool), AllocationStrategy::Stack);
        
        // Complex types should be heap-allocated
        assert_eq!(manager.allocation_strategy(&HintType::String), AllocationStrategy::ArcHeap);
    }
    
    #[test]
    fn test_memory_allocation() {
        let mut manager = MemoryManager::new(MemoryConfig::default());
        
        // Allocate stack memory
        let stack_handle = manager.allocate(&HintType::Int(IntSize::I64), 8).unwrap();
        assert!(matches!(stack_handle, MemoryHandle::Stack(_)));
        
        // Allocate heap memory
        let heap_handle = manager.allocate(&HintType::String, 24).unwrap();
        assert!(matches!(heap_handle, MemoryHandle::Heap(_)));
        
        // Check stats
        assert_eq!(manager.stats().total_allocations, 2);
        assert_eq!(manager.stats().stack_allocations, 1);
        assert_eq!(manager.stats().heap_allocations, 1);
    }
}

use crate::semantics::{IntSize, FloatSize};
