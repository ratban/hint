//! Automatic Reference Counting (ARC) Runtime
//! 
//! Provides ref-counted heap allocation with automatic deallocation
//! when reference count reaches zero.

use std::collections::HashMap;
use std::ptr;

/// Reference count header for ARC objects
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RcHeader {
    /// Reference count
    pub ref_count: u32,
    /// Type information (for debugging/cycle detection)
    pub type_id: u32,
    /// Object size in bytes
    pub size: u32,
    /// Flags (weak refs, etc.)
    pub flags: u32,
}

impl RcHeader {
    pub const SIZE: usize = 16; // 4 fields * 4 bytes
    
    pub fn new(type_id: u32, size: u32) -> Self {
        Self {
            ref_count: 1,
            type_id,
            size,
            flags: 0,
        }
    }
}

/// ARC-managed object
#[repr(C)]
pub struct ArcObject {
    /// Header with ref count
    header: RcHeader,
    /// Object data (variable size)
    data: [u8; 0], // Flexible array member
}

impl ArcObject {
    /// Get pointer to data area
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    
    /// Get mutable pointer to data area
    pub fn data_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    
    /// Get total allocation size (header + data)
    pub fn total_size(&self) -> usize {
        RcHeader::SIZE + self.header.size as usize
    }
}

/// ARC runtime manager
pub struct ArcRuntime {
    /// Allocated objects (for tracking/debugging)
    objects: HashMap<u64, ArcObjectInfo>,
    /// Next object ID
    next_id: u64,
    /// Weak reference support
    weak_refs: HashMap<u64, Vec<u64>>,
}

/// Information about an allocated object
#[derive(Debug, Clone)]
pub struct ArcObjectInfo {
    pub id: u64,
    pub type_id: u32,
    pub size: usize,
    pub ref_count: u32,
    pub weak_count: u32,
}

/// Handle to an ARC object
#[derive(Debug, Clone, Copy)]
pub struct ArcHandle {
    pub id: u64,
    pub ptr: u64,
}

impl ArcRuntime {
    /// Create new ARC runtime
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            next_id: 1,
            weak_refs: HashMap::new(),
        }
    }
    
    /// Allocate a new ref-counted object
    pub fn allocate(&mut self, size: usize, type_id: u32) -> Result<ArcHandle, String> {
        // In real implementation, this would allocate actual memory
        // For now, we track allocations logically
        
        let id = self.next_id;
        self.next_id += 1;
        
        let total_size = RcHeader::SIZE + size;
        
        self.objects.insert(id, ArcObjectInfo {
            id,
            type_id,
            size,
            ref_count: 1,
            weak_count: 0,
        });
        
        Ok(ArcHandle {
            id,
            ptr: id, // In real impl, this would be actual memory address
        })
    }
    
    /// Increment reference count
    pub fn incref(&mut self, handle: &ArcHandle) {
        if let Some(info) = self.objects.get_mut(&handle.id) {
            info.ref_count += 1;
        }
    }
    
    /// Decrement reference count, return freed size if deallocated
    pub fn decref(&mut self, handle: &ArcHandle) -> Result<Option<usize>, String> {
        if let Some(info) = self.objects.get_mut(&handle.id) {
            if info.ref_count == 0 {
                return Err(format!("Double free detected for object {}", handle.id));
            }
            
            info.ref_count -= 1;
            
            if info.ref_count == 0 {
                // Object is dead, deallocate
                let size = info.size;
                self.objects.remove(&handle.id);
                
                // Clean up weak refs
                self.weak_refs.remove(&handle.id);
                
                return Ok(Some(size));
            }
        }
        Ok(None)
    }
    
    /// Get reference count
    pub fn ref_count(&self, handle: &ArcHandle) -> Option<u32> {
        self.objects.get(&handle.id).map(|info| info.ref_count)
    }
    
    /// Get object info
    pub fn get_info(&self, handle: &ArcHandle) -> Option<&ArcObjectInfo> {
        self.objects.get(&handle.id)
    }
    
    /// Create weak reference
    pub fn create_weak_ref(&mut self, handle: &ArcHandle) -> Result<u64, String> {
        let weak_id = self.next_id;
        self.next_id += 1;
        
        self.weak_refs
            .entry(handle.id)
            .or_insert_with(Vec::new)
            .push(weak_id);
        
        if let Some(info) = self.objects.get_mut(&handle.id) {
            info.weak_count += 1;
        }
        
        Ok(weak_id)
    }
    
    /// Upgrade weak reference to strong
    pub fn upgrade_weak(&self, weak_id: u64, strong_handle: &ArcHandle) -> Option<ArcHandle> {
        if let Some(weak_ids) = self.weak_refs.get(&strong_handle.id) {
            if weak_ids.contains(&weak_id) {
                if let Some(info) = self.objects.get(&strong_handle.id) {
                    if info.ref_count > 0 {
                        return Some(*strong_handle);
                    }
                }
            }
        }
        None
    }
    
    /// Get all live objects (for debugging/GC)
    pub fn live_objects(&self) -> Vec<&ArcObjectInfo> {
        self.objects.values().collect()
    }
    
    /// Get total memory usage
    pub fn total_memory_usage(&self) -> usize {
        self.objects.values()
            .map(|info| RcHeader::SIZE + info.size)
            .sum()
    }
}

impl Default for ArcRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Smart pointer wrapper for ARC objects
pub struct ArcPtr<T> {
    handle: ArcHandle,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ArcPtr<T> {
    /// Create new ARC pointer
    pub fn new(runtime: &mut ArcRuntime, value: T) -> Result<Self, String>
    where
        T: Sized,
    {
        let size = std::mem::size_of::<T>();
        let type_id = std::any::TypeId::of::<T>().hash() as u32;
        let handle = runtime.allocate(size, type_id)?;
        
        Ok(Self {
            handle,
            _marker: std::marker::PhantomData,
        })
    }
    
    /// Get handle
    pub fn handle(&self) -> ArcHandle {
        self.handle
    }
    
    /// Clone (increment ref count)
    pub fn clone(&self, runtime: &mut ArcRuntime) -> Self {
        runtime.incref(&self.handle);
        Self {
            handle: self.handle,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for ArcPtr<T> {
    fn drop(&mut self) {
        // In real implementation, would call runtime.decref
        // This requires runtime access, so typically handled by MemoryManager
    }
}

/// Weak pointer wrapper
pub struct WeakPtr<T> {
    weak_id: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T> WeakPtr<T> {
    /// Create from strong pointer
    pub fn from_strong(strong: &ArcPtr<T>, runtime: &mut ArcRuntime) -> Result<Self, String> {
        let weak_id = runtime.create_weak_ref(&strong.handle)?;
        Ok(Self {
            weak_id,
            _marker: std::marker::PhantomData,
        })
    }
    
    /// Try to upgrade to strong pointer
    pub fn upgrade(&self, runtime: &ArcRuntime, strong: &ArcPtr<T>) -> Option<ArcPtr<T>> {
        runtime.upgrade_weak(self.weak_id, &strong.handle).map(|handle| ArcPtr {
            handle,
            _marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arc_basic() {
        let mut runtime = ArcRuntime::new();
        
        // Allocate
        let handle = runtime.allocate(100, 1).unwrap();
        assert_eq!(runtime.ref_count(&handle), Some(1));
        
        // Incref
        runtime.incref(&handle);
        assert_eq!(runtime.ref_count(&handle), Some(2));
        
        // Decref (should not free yet)
        let freed = runtime.decref(&handle).unwrap();
        assert_eq!(freed, None);
        assert_eq!(runtime.ref_count(&handle), Some(1));
        
        // Decref again (should free)
        let freed = runtime.decref(&handle).unwrap();
        assert_eq!(freed, Some(100));
        assert_eq!(runtime.ref_count(&handle), None);
    }
    
    #[test]
    fn test_arc_weak_refs() {
        let mut runtime = ArcRuntime::new();
        
        let handle = runtime.allocate(50, 1).unwrap();
        
        // Create weak ref
        let weak_id = runtime.create_weak_ref(&handle).unwrap();
        
        // Upgrade weak ref
        let upgraded = runtime.upgrade_weak(weak_id, &handle);
        assert!(upgraded.is_some());
        
        // Free original
        runtime.decref(&handle).unwrap();
        
        // Weak ref should no longer upgrade
        let upgraded = runtime.upgrade_weak(weak_id, &handle);
        assert!(upgraded.is_none());
    }
}
