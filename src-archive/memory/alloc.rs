//! Memory Allocation Strategies

/// Allocation strategy for a type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Stack allocation (primitives)
    Stack,
    /// Heap allocation with ARC
    ArcHeap,
    /// Manual allocation (user manages)
    Manual,
    /// Arena allocation (batch deallocation)
    Arena,
}

/// Memory allocator interface
pub trait MemoryAllocator {
    /// Allocate memory
    fn allocate(&mut self, size: usize, align: usize) -> Result<u64, String>;
    
    /// Deallocate memory
    fn deallocate(&mut self, ptr: u64, size: usize) -> Result<(), String>;
    
    /// Reallocate memory
    fn reallocate(&mut self, ptr: u64, old_size: usize, new_size: usize) -> Result<u64, String>;
    
    /// Get memory usage
    fn usage(&self) -> MemoryUsage;
}

/// Memory usage statistics
#[derive(Debug, Default)]
pub struct MemoryUsage {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
}

/// Bump allocator for fast stack-like allocation
pub struct BumpAllocator {
    buffer: Vec<u8>,
    offset: usize,
    start_ptr: u64,
}

impl BumpAllocator {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0u8; size],
            offset: 0,
            start_ptr: 0x1000, // Fake base address
        }
    }
    
    pub fn reset(&mut self) {
        self.offset = 0;
    }
    
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.offset
    }
}

impl MemoryAllocator for BumpAllocator {
    fn allocate(&mut self, size: usize, align: usize) -> Result<u64, String> {
        // Align offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);
        
        if aligned_offset + size > self.buffer.len() {
            return Err("Out of memory".to_string());
        }
        
        let ptr = self.start_ptr + aligned_offset as u64;
        self.offset = aligned_offset + size;
        
        Ok(ptr)
    }
    
    fn deallocate(&mut self, _ptr: u64, _size: usize) -> Result<(), String> {
        // Bump allocator doesn't support individual deallocation
        Ok(())
    }
    
    fn reallocate(&mut self, ptr: u64, old_size: usize, new_size: usize) -> Result<u64, String> {
        // Simple case: if new size fits in remaining space after ptr
        let offset = (ptr - self.start_ptr) as usize;
        if offset + new_size <= self.buffer.len() {
            Ok(ptr)
        } else {
            // Need to allocate elsewhere
            let new_ptr = self.allocate(new_size, 8)?;
            // Copy data (in real impl)
            Ok(new_ptr)
        }
    }
    
    fn usage(&self) -> MemoryUsage {
        MemoryUsage {
            total_allocated: self.offset,
            total_freed: 0,
            current_usage: self.offset,
            peak_usage: self.offset,
            allocation_count: 1,
        }
    }
}

/// Slab allocator for fixed-size objects
pub struct SlabAllocator {
    slab_size: usize,
    slabs: Vec<Vec<u8>>,
    free_lists: HashMap<usize, Vec<u64>>,
    base_ptr: u64,
}

use std::collections::HashMap;

impl SlabAllocator {
    pub fn new(slab_size: usize) -> Self {
        Self {
            slab_size,
            slabs: Vec::new(),
            free_lists: HashMap::new(),
            base_ptr: 0x10000000,
        }
    }
    
    fn add_slab(&mut self) {
        let slab = vec![0u8; self.slab_size * 64]; // 64 objects per slab
        self.slabs.push(slab);
        
        // Add all objects to free list
        let slab_idx = self.slabs.len() - 1;
        let free_list = self.free_lists.entry(self.slab_size).or_insert_with(Vec::new);
        
        for i in 0..64 {
            let offset = i * self.slab_size;
            let ptr = self.base_ptr + (slab_idx * self.slab_size * 64) as u64 + offset as u64;
            free_list.push(ptr);
        }
    }
}

impl MemoryAllocator for SlabAllocator {
    fn allocate(&mut self, size: usize, _align: usize) -> Result<u64, String> {
        // Find appropriate slab size
        let slab_size = self.slab_size.max(size);
        
        let free_list = self.free_lists.entry(slab_size).or_insert_with(Vec::new);
        
        if free_list.is_empty() {
            self.add_slab();
        }
        
        free_list.pop().ok_or("Out of memory".to_string())
    }
    
    fn deallocate(&mut self, ptr: u64, size: usize) -> Result<(), String> {
        let slab_size = self.slab_size.max(size);
        let free_list = self.free_lists.entry(slab_size).or_insert_with(Vec::new);
        free_list.push(ptr);
        Ok(())
    }
    
    fn reallocate(&mut self, ptr: u64, old_size: usize, new_size: usize) -> Result<u64, String> {
        if new_size <= old_size {
            return Ok(ptr);
        }
        
        let new_ptr = self.allocate(new_size, 8)?;
        self.deallocate(ptr, old_size)?;
        Ok(new_ptr)
    }
    
    fn usage(&self) -> MemoryUsage {
        let total = self.slabs.len() * self.slab_size * 64;
        let mut used = 0;
        for (_, list) in &self.free_lists {
            used += list.len() * self.slab_size;
        }
        
        MemoryUsage {
            total_allocated: total,
            total_freed: total - used,
            current_usage: used,
            peak_usage: total,
            allocation_count: total / self.slab_size,
        }
    }
}

/// Region allocator for arena-style allocation
pub struct RegionAllocator {
    regions: Vec<Region>,
    current_region: usize,
}

struct Region {
    data: Vec<u8>,
    offset: usize,
}

impl Region {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            offset: 0,
        }
    }
}

impl RegionAllocator {
    pub fn new(region_size: usize) -> Self {
        Self {
            regions: vec![Region::new(region_size)],
            current_region: 0,
        }
    }
    
    /// Deallocate entire region
    pub fn dealloc_region(&mut self, region_idx: usize) {
        if region_idx < self.regions.len() {
            self.regions[region_idx].offset = 0;
        }
    }
    
    /// Deallocate all regions
    pub fn dealloc_all(&mut self) {
        for region in &mut self.regions {
            region.offset = 0;
        }
        self.current_region = 0;
    }
}

impl MemoryAllocator for RegionAllocator {
    fn allocate(&mut self, size: usize, align: usize) -> Result<u64, String> {
        loop {
            let region = &mut self.regions[self.current_region];
            let aligned_offset = (region.offset + align - 1) & !(align - 1);
            
            if aligned_offset + size <= region.data.len() {
                let ptr = region.offset as u64;
                region.offset = aligned_offset + size;
                return Ok(ptr);
            }
            
            // Need new region
            self.current_region += 1;
            if self.current_region >= self.regions.len() {
                self.regions.push(Region::new(1024 * 1024)); // 1MB regions
            }
        }
    }
    
    fn deallocate(&mut self, _ptr: u64, _size: usize) -> Result<(), String> {
        // Region allocator doesn't support individual deallocation
        Ok(())
    }
    
    fn reallocate(&mut self, ptr: u64, old_size: usize, new_size: usize) -> Result<u64, String> {
        if new_size <= old_size {
            return Ok(ptr);
        }
        
        let new_ptr = self.allocate(new_size, 8)?;
        Ok(new_ptr)
    }
    
    fn usage(&self) -> MemoryUsage {
        let total = self.regions.len() * self.regions.first().map(|r| r.data.len()).unwrap_or(0);
        let used: usize = self.regions.iter().map(|r| r.offset).sum();
        
        MemoryUsage {
            total_allocated: total,
            total_freed: total - used,
            current_usage: used,
            peak_usage: total,
            allocation_count: used,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bump_allocator() {
        let mut alloc = BumpAllocator::new(1024);
        
        let ptr1 = alloc.allocate(100, 8).unwrap();
        let ptr2 = alloc.allocate(200, 8).unwrap();
        
        assert!(ptr2 > ptr1);
        assert_eq!(alloc.usage().current_usage, 304); // 100 + 200 + alignment
    }
    
    #[test]
    fn test_slab_allocator() {
        let mut alloc = SlabAllocator::new(64);
        
        let ptr1 = alloc.allocate(50, 8).unwrap();
        let ptr2 = alloc.allocate(50, 8).unwrap();
        
        assert_ne!(ptr1, ptr2);
        
        alloc.deallocate(ptr1, 50).unwrap();
        
        // Reallocate should reuse freed slot
        let ptr3 = alloc.allocate(50, 8).unwrap();
        assert_eq!(ptr3, ptr1);
    }
}
