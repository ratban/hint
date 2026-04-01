//! Instruction Combining
//! 
//! Combines multiple instructions into fewer, more efficient instructions.

use crate::ir::{HIR, HirBlock, HirInstruction, HirValue, HirBinaryOp, HirConstant};
use crate::semantics::{HintType, IntSize};
use super::{OptimizationPass, OptimizationStats, OptimizationLevel};

/// Instruction combining pass
pub struct InstCombinePass {
    stats: OptimizationStats,
}

impl InstCombinePass {
    pub fn new() -> Self {
        Self {
            stats: OptimizationStats::new(),
        }
    }
}

impl Default for InstCombinePass {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for InstCombinePass {
    fn name(&self) -> &'static str {
        "inst-combine"
    }
    
    fn run(&mut self, hir: &mut HIR) -> Result<OptimizationStats, String> {
        self.stats = OptimizationStats::new();
        
        let mut combiner = InstructionCombiner::new();
        
        // Combine instructions in entry point
        if let Some(entry) = &mut hir.entry_point {
            combiner.combine_block(entry, &mut self.stats);
        }
        
        // Combine instructions in functions
        for func in &mut hir.functions {
            combiner.combine_block(&mut func.body, &mut self.stats);
        }
        
        Ok(self.stats.clone())
    }
    
    fn should_run(&self, level: OptimizationLevel) -> bool {
        matches!(level, OptimizationLevel::Speed | OptimizationLevel::SpeedAndSize)
    }
}

/// Instruction combiner implementation
pub struct InstructionCombiner {
    /// Track value definitions for pattern matching
    definitions: std::collections::HashMap<usize, HirInstruction>,
}

impl InstructionCombiner {
    pub fn new() -> Self {
        Self {
            definitions: std::collections::HashMap::new(),
        }
    }
    
    /// Combine instructions in a block
    pub fn combine_block(&mut self, block: &mut HirBlock, stats: &mut OptimizationStats) {
        self.definitions.clear();
        
        // Build definition map
        for instr in &block.instructions {
            match instr {
                HirInstruction::LoadConst { dest, value } => {
                    self.definitions.insert(dest.id, instr.clone());
                }
                HirInstruction::BinaryOp { dest, op, left, right, result_type } => {
                    self.definitions.insert(dest.id, instr.clone());
                }
                _ => {}
            }
        }
        
        // Apply combining patterns
        self.combine_patterns(block, stats);
    }
    
    /// Apply instruction combining patterns
    fn combine_patterns(&mut self, block: &mut HirBlock, stats: &mut OptimizationStats) {
        let mut new_instructions = Vec::new();
        
        for instr in block.instructions.drain(..) {
            match self.try_combine(&instr) {
                CombineResult::Combined(new_instrs) => {
                    stats.instructions_removed += 1;
                    stats.instructions_added += new_instrs.len();
                    new_instructions.extend(new_instrs);
                }
                CombineResult::Keep => {
                    new_instructions.push(instr);
                }
            }
        }
        
        block.instructions = new_instructions;
    }
    
    /// Try to combine an instruction
    fn try_combine(&self, instr: &HirInstruction) -> CombineResult {
        match instr {
            // Pattern: (x + c1) + c2 -> x + (c1 + c2)
            HirInstruction::BinaryOp { dest, op: HirBinaryOp::Add, left, right, result_type } => {
                self.try_combine_add(*dest, *left, *right, result_type.clone())
            }
            
            // Pattern: x * 2 -> x << 1
            HirInstruction::BinaryOp { dest, op: HirBinaryOp::Mul, left, right, result_type } => {
                self.try_combine_mul(*dest, *left, *right, result_type.clone())
            }
            
            // Pattern: x / 2 -> x >> 1
            HirInstruction::BinaryOp { dest, op: HirBinaryOp::Div, left, right, result_type } => {
                self.try_combine_div(*dest, *left, *right, result_type.clone())
            }
            
            // Pattern: x - x -> 0
            HirInstruction::BinaryOp { dest, op: HirBinaryOp::Sub, left, right, result_type } => {
                if left.id == right.id {
                    return CombineResult::Combined(vec![
                        HirInstruction::LoadConst {
                            dest: *dest,
                            value: HirConstant::Int(0),
                        }
                    ]);
                }
                CombineResult::Keep
            }
            
            // Pattern: x * 0 -> 0
            HirInstruction::BinaryOp { dest, op: HirBinaryOp::Mul, left, right, result_type } => {
                if let Some(const_val) = self.get_constant(left.id) {
                    if let HirConstant::Int(0) = const_val {
                        return CombineResult::Combined(vec![
                            HirInstruction::LoadConst {
                                dest: *dest,
                                value: HirConstant::Int(0),
                            }
                        ]);
                    }
                }
                if let Some(const_val) = self.get_constant(right.id) {
                    if let HirConstant::Int(0) = const_val {
                        return CombineResult::Combined(vec![
                            HirInstruction::LoadConst {
                                dest: *dest,
                                value: HirConstant::Int(0),
                            }
                        ]);
                    }
                }
                CombineResult::Keep
            }
            
            _ => CombineResult::Keep,
        }
    }
    
    /// Combine addition patterns
    fn try_combine_add(&self, dest: HirValue, left: HirValue, right: HirValue, result_type: HintType) -> CombineResult {
        // Check for (x + c1) + c2 pattern
        if let Some(HirInstruction::BinaryOp { op: HirBinaryOp::Add, left: inner_left, right: inner_right, .. }) = self.get_definition(left.id) {
            if let Some(HirConstant::Int(c1)) = self.get_constant(inner_right.id) {
                if let Some(HirConstant::Int(c2)) = self.get_constant(right.id) {
                    // Combine: x + (c1 + c2)
                    let combined = c1 + c2;
                    return CombineResult::Combined(vec![
                        HirInstruction::BinaryOp {
                            dest,
                            op: HirBinaryOp::Add,
                            left: *inner_left,
                            right: HirValue::new(1000, result_type.clone()),
                            result_type,
                        },
                        HirInstruction::LoadConst {
                            dest: HirValue::new(1000, result_type.clone()),
                            value: HirConstant::Int(combined),
                        },
                    ]);
                }
            }
        }
        
        CombineResult::Keep
    }
    
    /// Combine multiplication patterns
    fn try_combine_mul(&self, dest: HirValue, left: HirValue, right: HirValue, result_type: HintType) -> CombineResult {
        // x * 2 -> x << 1
        if let Some(HirConstant::Int(2)) = self.get_constant(right.id) {
            return CombineResult::Combined(vec![
                HirInstruction::BinaryOp {
                    dest,
                    op: HirBinaryOp::Shl,
                    left,
                    right: HirValue::new(1001, result_type.clone()),
                    result_type,
                },
                HirInstruction::LoadConst {
                    dest: HirValue::new(1001, result_type.clone()),
                    value: HirConstant::Int(1),
                },
            ]);
        }
        
        // 2 * x -> x << 1
        if let Some(HirConstant::Int(2)) = self.get_constant(left.id) {
            return CombineResult::Combined(vec![
                HirInstruction::BinaryOp {
                    dest,
                    op: HirBinaryOp::Shl,
                    left: right,
                    right: HirValue::new(1001, result_type.clone()),
                    result_type,
                },
                HirInstruction::LoadConst {
                    dest: HirValue::new(1001, result_type.clone()),
                    value: HirConstant::Int(1),
                },
            ]);
        }
        
        CombineResult::Keep
    }
    
    /// Combine division patterns
    fn try_combine_div(&self, dest: HirValue, left: HirValue, right: HirValue, result_type: HintType) -> CombineResult {
        // x / 2 -> x >> 1 (for unsigned)
        if let Some(HirConstant::Int(2)) = self.get_constant(right.id) {
            return CombineResult::Combined(vec![
                HirInstruction::BinaryOp {
                    dest,
                    op: HirBinaryOp::Shr,
                    left,
                    right: HirValue::new(1001, result_type.clone()),
                    result_type,
                },
                HirInstruction::LoadConst {
                    dest: HirValue::new(1001, result_type.clone()),
                    value: HirConstant::Int(1),
                },
            ]);
        }
        
        CombineResult::Keep
    }
    
    /// Get definition of a value
    fn get_definition(&self, value_id: usize) -> Option<&HirInstruction> {
        self.definitions.get(&value_id)
    }
    
    /// Get constant value if known
    fn get_constant(&self, value_id: usize) -> Option<&HirConstant> {
        if let Some(HirInstruction::LoadConst { value, .. }) = self.get_definition(value_id) {
            Some(value)
        } else {
            None
        }
    }
}

/// Result of combine attempt
enum CombineResult {
    /// Instruction was combined into new instructions
    Combined(Vec<HirInstruction>),
    /// Keep original instruction
    Keep,
}

/// Peephole optimizer
pub struct PeepholeOptimizer;

impl PeepholeOptimizer {
    /// Apply peephole optimizations
    pub fn optimize(block: &mut HirBlock, stats: &mut super::OptimizationStats) {
        let mut new_instructions = Vec::new();
        let mut prev: Option<HirInstruction> = None;
        
        for instr in block.instructions.drain(..) {
            if let Some(prev_instr) = prev.take() {
                match Self::try_peephole(&prev_instr, &instr) {
                    PeepholeResult::Combined(combined) => {
                        stats.instructions_removed += 2;
                        stats.instructions_added += combined.len();
                        new_instructions.extend(combined);
                    }
                    PeepholeResult::KeepBoth => {
                        new_instructions.push(prev_instr);
                        prev = Some(instr);
                    }
                }
            } else {
                prev = Some(instr);
            }
        }
        
        if let Some(last) = prev {
            new_instructions.push(last);
        }
        
        block.instructions = new_instructions;
    }
    
    /// Try peephole optimization on two consecutive instructions
    fn try_peephole(first: &HirInstruction, second: &HirInstruction) -> PeepholeResult {
        // Pattern: LoadConst x, StoreVar v -> StoreVar v with constant
        // This is a simplified implementation
        PeepholeResult::KeepBoth
    }
}

enum PeepholeResult {
    Combined(Vec<HirInstruction>),
    KeepBoth,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inst_combine_pass() {
        let pass = InstCombinePass::new();
        assert_eq!(pass.name(), "inst-combine");
    }
    
    #[test]
    fn test_combiner_creation() {
        let combiner = InstructionCombiner::new();
        assert!(combiner.definitions.is_empty());
    }
}
