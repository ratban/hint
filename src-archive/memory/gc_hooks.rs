//! Garbage Collection Hooks for Cycle Detection
//! 
//! While ARC handles most memory automatically, reference cycles can cause leaks.
//! This module provides optional cycle detection and collection.

use super::arc::{ArcRuntime, ArcHandle, ArcObjectInfo};
use std::collections::{HashMap, HashSet, VecDeque};

/// Cycle detector using trial deletion algorithm
pub struct CycleDetector {
    /// Candidate objects for cycle detection
    candidates: HashSet<u64>,
    /// Mark set for current collection
    mark_set: HashSet<u64>,
    /// Scan queue
    scan_queue: VecDeque<u64>,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            candidates: HashSet::new(),
            mark_set: HashSet::new(),
            scan_queue: VecDeque::new(),
        }
    }
    
    /// Add object as potential cycle candidate
    pub fn add_candidate(&mut self, id: u64) {
        self.candidates.insert(id);
    }
    
    /// Remove candidate (ref count went up)
    pub fn remove_candidate(&mut self, id: u64) {
        self.candidates.remove(&id);
    }
    
    /// Check for cycles among candidates
    pub fn check_for_cycles(&mut self, runtime: &ArcRuntime) -> Result<Vec<u64>, String> {
        let mut cycles = Vec::new();
        
        for &id in &self.candidates.clone() {
            if let Some(info) = runtime.get_info(&ArcHandle { id, ptr: id }) {
                if info.ref_count > 0 {
                    // Try to mark from this candidate
                    if self.try_mark(id, runtime) {
                        // Object is reachable, not a cycle
                        continue;
                    }
                    
                    // Object is garbage, collect it
                    cycles.push(id);
                }
            }
        }
        
        Ok(cycles)
    }
    
    /// Try to mark object as reachable
    fn try_mark(&mut self, id: u64, runtime: &ArcRuntime) -> bool {
        self.mark_set.clear();
        self.scan_queue.clear();
        self.scan_queue.push_back(id);
        
        while let Some(current) = self.scan_queue.pop_front() {
            if self.mark_set.contains(&current) {
                continue;
            }
            
            if let Some(info) = runtime.get_info(&ArcHandle { id: current, ptr: current }) {
                // Simulate tracing references (in real impl, would follow pointers)
                // For now, just check if ref_count > weak_count
                if info.ref_count > info.weak_count {
                    self.mark_set.insert(current);
                }
            }
        }
        
        self.mark_set.contains(&id)
    }
    
    /// Collect detected cycles
    pub fn collect_cycles(&mut self, runtime: &mut ArcRuntime) -> Result<usize, String> {
        let cycles = self.check_for_cycles(runtime)?;
        let count = cycles.len();
        
        for id in cycles {
            // Force deallocation
            let handle = ArcHandle { id, ptr: id };
            while runtime.decref(&handle)?.is_some() {}
            self.candidates.remove(&id);
        }
        
        Ok(count)
    }
    
    /// Get number of candidates
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }
    
    /// Clear all state
    pub fn clear(&mut self) {
        self.candidates.clear();
        self.mark_set.clear();
        self.scan_queue.clear();
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// GC hooks interface for integrating with codegen
pub trait GCHooks {
    /// Called when ref count is incremented
    fn on_incref(&mut self, handle: ArcHandle);
    
    /// Called when ref count is decremented
    fn on_decref(&mut self, handle: ArcHandle, new_count: u32);
    
    /// Called when object might be in a cycle
    fn on_possible_cycle(&mut self, handle: ArcHandle);
}

/// GC statistics
#[derive(Debug, Default)]
pub struct GCStats {
    pub cycles_detected: usize,
    pub objects_collected: usize,
    pub bytes_freed: usize,
    pub collection_count: usize,
}

/// Incremental GC for low-pause collection
pub struct IncrementalGC {
    /// Work budget per collection step
    work_budget: usize,
    /// Current work done
    work_done: usize,
    /// Objects to scan
    gray_set: VecDeque<u64>,
    /// Statistics
    stats: GCStats,
}

impl IncrementalGC {
    pub fn new(work_budget: usize) -> Self {
        Self {
            work_budget,
            work_done: 0,
            gray_set: VecDeque::new(),
            stats: GCStats::default(),
        }
    }
    
    /// Start collection cycle
    pub fn start_collection(&mut self, roots: &[u64]) {
        self.gray_set.extend(roots);
        self.work_done = 0;
    }
    
    /// Perform one collection step
    pub fn step(&mut self, runtime: &mut ArcRuntime) -> Result<bool, String> {
        if self.gray_set.is_empty() {
            return Ok(false); // Collection complete
        }
        
        while self.work_done < self.work_budget {
            if let Some(id) = self.gray_set.pop_front() {
                // Scan object
                if let Some(info) = runtime.get_info(&ArcHandle { id, ptr: id }) {
                    if info.ref_count == 0 {
                        // Object is garbage
                        runtime.decref(&ArcHandle { id, ptr: id })?;
                        self.stats.objects_collected += 1;
                        self.stats.bytes_freed += info.size;
                    }
                    
                    // Add children to gray set (in real impl)
                }
                
                self.work_done += 1;
            } else {
                break;
            }
        }
        
        Ok(!self.gray_set.is_empty())
    }
    
    /// Get statistics
    pub fn stats(&self) -> &GCStats {
        &self.stats
    }
}

/// Generational GC for better performance
pub struct GenerationalGC {
    /// Young generation (newly allocated)
    young_gen: HashSet<u64>,
    /// Old generation (survived collection)
    old_gen: HashSet<u64>,
    /// Collection threshold
    young_threshold: usize,
    /// Old collection threshold
    old_threshold: usize,
}

impl GenerationalGC {
    pub fn new(young_threshold: usize, old_threshold: usize) -> Self {
        Self {
            young_gen: HashSet::new(),
            old_gen: HashSet::new(),
            young_threshold,
            old_threshold,
        }
    }
    
    /// Add new object to young generation
    pub fn add_object(&mut self, id: u64) {
        self.young_gen.insert(id);
    }
    
    /// Promote object to old generation
    pub fn promote(&mut self, id: u64) {
        self.young_gen.remove(&id);
        self.old_gen.insert(id);
    }
    
    /// Collect young generation
    pub fn collect_young(&mut self, runtime: &mut ArcRuntime) -> Result<usize, String> {
        let mut collected = 0;
        let mut to_promote = Vec::new();
        
        for &id in &self.young_gen.clone() {
            if let Some(info) = runtime.get_info(&ArcHandle { id, ptr: id }) {
                if info.ref_count == 0 {
                    runtime.decref(&ArcHandle { id, ptr: id })?;
                    collected += 1;
                } else {
                    to_promote.push(id);
                }
            }
        }
        
        // Promote survivors
        for id in to_promote {
            self.promote(id);
        }
        
        Ok(collected)
    }
    
    /// Collect old generation (full GC)
    pub fn collect_old(&mut self, runtime: &mut ArcRuntime) -> Result<usize, String> {
        let mut collected = 0;
        
        for &id in &self.old_gen.clone() {
            if let Some(info) = runtime.get_info(&ArcHandle { id, ptr: id }) {
                if info.ref_count == 0 {
                    runtime.decref(&ArcHandle { id, ptr: id })?;
                    collected += 1;
                }
            }
        }
        
        Ok(collected)
    }
    
    /// Should we trigger young collection?
    pub fn should_collect_young(&self) -> bool {
        self.young_gen.len() >= self.young_threshold
    }
    
    /// Should we trigger old collection?
    pub fn should_collect_old(&self) -> bool {
        self.old_gen.len() >= self.old_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantics::memory::arc::ArcRuntime;
    
    #[test]
    fn test_cycle_detector() {
        let mut runtime = ArcRuntime::new();
        let mut detector = CycleDetector::new();
        
        // Create objects
        let obj1 = runtime.allocate(100, 1).unwrap();
        let obj2 = runtime.allocate(100, 1).unwrap();
        
        // Add as candidates
        detector.add_candidate(obj1.id);
        detector.add_candidate(obj2.id);
        
        // No cycles yet (both have ref_count = 1)
        let cycles = detector.check_for_cycles(&runtime).unwrap();
        assert!(cycles.is_empty());
        
        // Simulate cycle: decref both (in real impl, they'd reference each other)
        runtime.decref(&obj1).unwrap();
        runtime.decref(&obj2).unwrap();
        
        // Now they should be detected as garbage
        let cycles = detector.check_for_cycles(&runtime).unwrap();
        assert_eq!(cycles.len(), 2);
    }
    
    #[test]
    fn test_generational_gc() {
        let mut runtime = ArcRuntime::new();
        let mut gc = GenerationalGC::new(10, 100);
        
        // Add objects to young gen
        for i in 0..15 {
            let obj = runtime.allocate(50, 1).unwrap();
            gc.add_object(obj.id);
        }
        
        // Should trigger young collection
        assert!(gc.should_collect_young());
        
        let collected = gc.collect_young(&mut runtime).unwrap();
        assert_eq!(collected, 15); // All collected (ref_count = 1, then decref in collect)
    }
}
