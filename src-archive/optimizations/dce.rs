//! Dead Code Elimination (DCE)
//! 
//! Removes unreachable code and unused computations.

use crate::ir::{HIR, HirBlock, HirInstruction, HirValue};
use crate::semantics::HintType;
use super::{OptimizationPass, OptimizationStats, OptimizationLevel, has_side_effects};
use std::collections::{HashMap, HashSet};

/// Dead code elimination pass
pub struct DCEPass {
    stats: OptimizationStats,
}

impl DCEPass {
    pub fn new() -> Self {
        Self {
            stats: OptimizationStats::new(),
        }
    }
}

impl Default for DCEPass {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for DCEPass {
    fn name(&self) -> &'static str {
        "dce"
    }
    
    fn run(&mut self, hir: &mut HIR) -> Result<OptimizationStats, String> {
        self.stats = OptimizationStats::new();
        
        let mut eliminator = DeadCodeEliminator::new();
        
        // Eliminate dead code in entry point
        if let Some(entry) = &mut hir.entry_point {
            eliminator.eliminate_block(entry, &mut self.stats);
        }
        
        // Eliminate dead code in functions
        for func in &mut hir.functions {
            eliminator.eliminate_block(&mut func.body, &mut self.stats);
        }
        
        Ok(self.stats.clone())
    }
    
    fn should_run(&self, level: OptimizationLevel) -> bool {
        matches!(level, OptimizationLevel::Speed | OptimizationLevel::SpeedAndSize)
    }
}

/// Dead code eliminator implementation
pub struct DeadCodeEliminator {
    /// Value use counts
    use_counts: HashMap<usize, usize>,
    /// Values that are live (used outside the block)
    live_values: HashSet<usize>,
}

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self {
            use_counts: HashMap::new(),
            live_values: HashSet::new(),
        }
    }
    
    /// Eliminate dead code in a block
    pub fn eliminate_block(&mut self, block: &mut HirBlock, stats: &mut OptimizationStats) {
        // First pass: count all uses
        self.compute_use_counts(block);
        
        // Second pass: identify live values (return values, side effects)
        self.identify_live_values(block);
        
        // Third pass: remove dead instructions
        self.remove_dead_instructions(block, stats);
    }
    
    /// Compute use counts for all values
    fn compute_use_counts(&mut self, block: &HirBlock) {
        self.use_counts.clear();
        
        for instr in &block.instructions {
            match instr {
                HirInstruction::StoreVar { source, .. } => {
                    *self.use_counts.entry(source.id).or_insert(0) += 1;
                }
                HirInstruction::BinaryOp { left, right, .. } => {
                    *self.use_counts.entry(left.id).or_insert(0) += 1;
                    *self.use_counts.entry(right.id).or_insert(0) += 1;
                }
                HirInstruction::Call { args, .. } => {
                    for arg in args {
                        *self.use_counts.entry(arg.id).or_insert(0) += 1;
                    }
                }
                HirInstruction::Return { value } => {
                    if let Some(v) = value {
                        *self.use_counts.entry(v.id).or_insert(0) += 1;
                        self.live_values.insert(v.id);
                    }
                }
                HirInstruction::Print { value } => {
                    *self.use_counts.entry(value.id).or_insert(0) += 1;
                    self.live_values.insert(value.id);
                }
                HirInstruction::Branch { condition, .. } => {
                    *self.use_counts.entry(condition.id).or_insert(0) += 1;
                    self.live_values.insert(condition.id);
                }
                _ => {}
            }
        }
    }
    
    /// Identify live values
    fn identify_live_values(&mut self, block: &HirBlock) {
        self.live_values.clear();
        
        // Values used in terminating instructions are live
        for instr in &block.instructions {
            match instr {
                HirInstruction::Return { value } => {
                    if let Some(v) = value {
                        self.live_values.insert(v.id);
                    }
                }
                HirInstruction::Print { value } => {
                    self.live_values.insert(value.id);
                }
                HirInstruction::Branch { condition, .. } => {
                    self.live_values.insert(condition.id);
                }
                HirInstruction::StoreVar { dest, .. } => {
                    // Variables stored might be used later
                    // Mark as potentially live
                }
                _ => {}
            }
        }
        
        // Propagate liveness backwards
        self.propagate_liveness(block);
    }
    
    /// Propagate liveness backwards through the block
    fn propagate_liveness(&mut self, block: &HirBlock) {
        let mut changed = true;
        
        while changed {
            changed = false;
            
            for instr in &block.instructions {
                match instr {
                    HirInstruction::LoadConst { dest, .. } => {
                        if self.live_values.contains(&dest.id) {
                            // Constant is live, but doesn't depend on anything
                        }
                    }
                    HirInstruction::BinaryOp { dest, left, right, .. } => {
                        if self.live_values.contains(&dest.id) {
                            if !self.live_values.contains(&left.id) {
                                self.live_values.insert(left.id);
                                changed = true;
                            }
                            if !self.live_values.contains(&right.id) {
                                self.live_values.insert(right.id);
                                changed = true;
                            }
                        }
                    }
                    HirInstruction::Call { dest, args, .. } => {
                        if dest.as_ref().map(|d| self.live_values.contains(&d.id)).unwrap_or(false) {
                            for arg in args {
                                if !self.live_values.contains(&arg.id) {
                                    self.live_values.insert(arg.id);
                                    changed = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Remove dead instructions
    fn remove_dead_instructions(&mut self, block: &mut HirBlock, stats: &mut OptimizationStats) {
        let mut new_instructions = Vec::new();
        
        for instr in block.instructions.drain(..) {
            if self.is_instruction_dead(&instr) {
                stats.instructions_removed += 1;
                stats.dead_blocks_removed += 1;
            } else {
                new_instructions.push(instr);
            }
        }
        
        block.instructions = new_instructions;
    }
    
    /// Check if an instruction is dead (can be removed)
    fn is_instruction_dead(&self, instr: &HirInstruction) -> bool {
        // Instructions with side effects are never dead
        if has_side_effects(instr) {
            return false;
        }
        
        // Check if the defined value is used
        match instr {
            HirInstruction::LoadConst { dest, .. } => {
                !self.live_values.contains(&dest.id) && 
                self.use_counts.get(&dest.id).copied().unwrap_or(0) == 0
            }
            HirInstruction::BinaryOp { dest, .. } => {
                !self.live_values.contains(&dest.id) && 
                self.use_counts.get(&dest.id).copied().unwrap_or(0) == 0
            }
            HirInstruction::LoadVar { dest, .. } => {
                !self.live_values.contains(&dest.id) && 
                self.use_counts.get(&dest.id).copied().unwrap_or(0) == 0
            }
            HirInstruction::Call { dest, function, .. } => {
                // Built-in functions might have side effects
                if matches!(function.as_str(), "print" | "println" | "exit" | "abort") {
                    return false;
                }
                
                if let Some(d) = dest {
                    !self.live_values.contains(&d.id) && 
                    self.use_counts.get(&d.id).copied().unwrap_or(0) == 0
                } else {
                    false // No return value, might have side effects
                }
            }
            HirInstruction::StoreVar { dest, source } => {
                // Dead store if the variable is never read
                // This requires more sophisticated analysis
                false
            }
            _ => false,
        }
    }
    
    /// Clear state
    pub fn clear(&mut self) {
        self.use_counts.clear();
        self.live_values.clear();
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple dead code detection for single instructions
pub fn is_dead_instruction(instr: &HirInstruction) -> bool {
    // Pure computations with no users are dead
    match instr {
        HirInstruction::LoadConst { .. } => false, // Constants might be used
        HirInstruction::BinaryOp { .. } => false, // Results might be used
        HirInstruction::LoadVar { .. } => false, // Variables might be used
        HirInstruction::StoreVar { .. } => false, // Stores have side effects
        HirInstruction::Call { function, dest, .. } => {
            // Pure function calls with unused results are dead
            if matches!(function.as_str(), "print" | "println" | "exit" | "abort") {
                return false; // Has side effects
            }
            dest.is_none() // No result = potentially dead
        }
        HirInstruction::Return { .. } => false, // Terminating
        HirInstruction::Print { .. } => false, // Has side effects
        HirInstruction::Jump { .. } => false, // Control flow
        HirInstruction::Branch { .. } => false, // Control flow
        HirInstruction::Label { .. } => false, // Label
        HirInstruction::Nop => true, // Nops are always dead
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::HirConstant;
    
    #[test]
    fn test_dce_pass() {
        let mut pass = DCEPass::new();
        assert_eq!(pass.name(), "dce");
    }
    
    #[test]
    fn test_is_dead_instruction() {
        let nop = HirInstruction::Nop;
        assert!(is_dead_instruction(&nop));
        
        let print = HirInstruction::Print {
            value: HirValue::new(0, HintType::String),
        };
        assert!(!is_dead_instruction(&print));
    }
    
    #[test]
    fn test_eliminator_creation() {
        let eliminator = DeadCodeEliminator::new();
        assert!(eliminator.use_counts.is_empty());
        assert!(eliminator.live_values.is_empty());
    }
}
