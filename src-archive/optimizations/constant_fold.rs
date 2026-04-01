//! Constant Folding and Propagation
//! 
//! Evaluates constant expressions at compile time and propagates constants.

use crate::ir::{HIR, HirBlock, HirInstruction, HirConstant, HirValue, HirBinaryOp};
use crate::semantics::{HintType, IntSize, FloatSize};
use super::{OptimizationPass, OptimizationStats, OptimizationLevel};
use std::collections::HashMap;

/// Constant folder pass
pub struct ConstantFoldPass {
    stats: OptimizationStats,
}

impl ConstantFoldPass {
    pub fn new() -> Self {
        Self {
            stats: OptimizationStats::new(),
        }
    }
}

impl Default for ConstantFoldPass {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for ConstantFoldPass {
    fn name(&self) -> &'static str {
        "constant-fold"
    }
    
    fn run(&mut self, hir: &mut HIR) -> Result<OptimizationStats, String> {
        self.stats = OptimizationStats::new();
        
        let mut folder = ConstantFolder::new();
        
        // Fold constants in entry point
        if let Some(entry) = &mut hir.entry_point {
            folder.fold_block(entry, &mut self.stats);
        }
        
        // Fold constants in functions
        for func in &mut hir.functions {
            folder.fold_block(&mut func.body, &mut self.stats);
        }
        
        Ok(self.stats.clone())
    }
    
    fn should_run(&self, level: OptimizationLevel) -> bool {
        matches!(level, OptimizationLevel::Speed | OptimizationLevel::SpeedAndSize)
    }
}

/// Constant folder implementation
pub struct ConstantFolder {
    /// Known constant values: value_id -> constant
    constants: HashMap<usize, HirConstant>,
    /// Value use counts for dead code detection
    use_counts: HashMap<usize, usize>,
}

impl ConstantFolder {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            use_counts: HashMap::new(),
        }
    }
    
    /// Fold constants in a block
    pub fn fold_block(&mut self, block: &mut HirBlock, stats: &mut OptimizationStats) {
        // First pass: count value uses
        self.count_uses(block);
        
        // Second pass: fold constants
        self.fold_instructions(block, stats);
    }
    
    /// Count value uses
    fn count_uses(&mut self, block: &HirBlock) {
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
                    }
                }
                HirInstruction::Print { value } => {
                    *self.use_counts.entry(value.id).or_insert(0) += 1;
                }
                HirInstruction::Branch { condition, .. } => {
                    *self.use_counts.entry(condition.id).or_insert(0) += 1;
                }
                _ => {}
            }
        }
    }
    
    /// Fold constants in instructions
    fn fold_instructions(&mut self, block: &mut HirBlock, stats: &mut OptimizationStats) {
        let mut new_instructions = Vec::new();
        
        for instr in block.instructions.drain(..) {
            match self.fold_instruction(instr, stats) {
                FoldResult::Keep(new_instr) => {
                    new_instructions.push(new_instr);
                }
                FoldResult::Replace(new_instr) => {
                    new_instructions.push(new_instr);
                    stats.instructions_removed += 1;
                    stats.instructions_added += 1;
                }
                FoldResult::Remove => {
                    stats.instructions_removed += 1;
                }
                FoldResult::FoldToConst(const_val) => {
                    // Instruction was folded to a constant
                    stats.constants_folded += 1;
                }
            }
        }
        
        block.instructions = new_instructions;
    }
    
    /// Fold a single instruction
    fn fold_instruction(&mut self, instr: HirInstruction, stats: &mut OptimizationStats) -> FoldResult {
        match instr {
            HirInstruction::LoadConst { dest, value } => {
                // Track constant value
                self.constants.insert(dest.id, value.clone());
                FoldResult::Keep(HirInstruction::LoadConst { dest, value })
            }
            
            HirInstruction::BinaryOp { dest, op, left, right, result_type } => {
                // Check if both operands are known constants
                if let (Some(left_const), Some(right_const)) = (
                    self.constants.get(&left.id),
                    self.constants.get(&right.id),
                ) {
                    // Try to fold the operation
                    if let Some(result) = self.fold_binary_op(op, left_const, right_const, &result_type) {
                        self.constants.insert(dest.id, result.clone());
                        stats.constants_folded += 1;
                        return FoldResult::FoldToConst(result);
                    }
                }
                
                // Can't fold, keep instruction
                FoldResult::Keep(HirInstruction::BinaryOp { dest, op, left, right, result_type })
            }
            
            HirInstruction::LoadVar { dest, source } => {
                // If source variable has a known constant value, propagate it
                // (This would require tracking variable -> constant mappings)
                FoldResult::Keep(HirInstruction::LoadVar { dest, source })
            }
            
            HirInstruction::StoreVar { dest, source } => {
                // Check if source is unused (dead store)
                if self.use_counts.get(&source.id).copied().unwrap_or(0) == 0 {
                    // Dead store, remove
                    return FoldResult::Remove;
                }
                FoldResult::Keep(HirInstruction::StoreVar { dest, source })
            }
            
            _ => FoldResult::Keep(instr),
        }
    }
    
    /// Fold a binary operation on constants
    fn fold_binary_op(&self, op: HirBinaryOp, left: &HirConstant, right: &HirConstant, result_type: &HintType) -> Option<HirConstant> {
        match (left, right) {
            (HirConstant::Int(l), HirConstant::Int(r)) => {
                self.fold_int_op(op, *l, *r)
            }
            (HirConstant::Float(l), HirConstant::Float(r)) => {
                self.fold_float_op(op, *l, *r)
            }
            (HirConstant::Bool(l), HirConstant::Bool(r)) => {
                self.fold_bool_op(op, *l, *r)
            }
            _ => None,
        }
    }
    
    /// Fold integer binary operation
    fn fold_int_op(&self, op: HirBinaryOp, left: i64, right: i64) -> Option<HirConstant> {
        let result = match op {
            HirBinaryOp::Add => left.checked_add(right)?,
            HirBinaryOp::Sub => left.checked_sub(right)?,
            HirBinaryOp::Mul => left.checked_mul(right)?,
            HirBinaryOp::Div => {
                if right == 0 { return None; } // Division by zero
                left.checked_div(right)?
            }
            HirBinaryOp::Mod => {
                if right == 0 { return None; } // Modulo by zero
                left.checked_rem(right)?
            }
            HirBinaryOp::Eq => return Some(HirConstant::Bool(left == right)),
            HirBinaryOp::Ne => return Some(HirConstant::Bool(left != right)),
            HirBinaryOp::Lt => return Some(HirConstant::Bool(left < right)),
            HirBinaryOp::Le => return Some(HirConstant::Bool(left <= right)),
            HirBinaryOp::Gt => return Some(HirConstant::Bool(left > right)),
            HirBinaryOp::Ge => return Some(HirConstant::Bool(left >= right)),
            HirBinaryOp::And | HirBinaryOp::BitAnd => left & right,
            HirBinaryOp::Or | HirBinaryOp::BitOr => left | right,
            HirBinaryOp::BitXor => left ^ right,
            HirBinaryOp::Shl => left.checked_shl(right as u32)?,
            HirBinaryOp::Shr => left.checked_shr(right as u32)?,
        };
        
        Some(HirConstant::Int(result))
    }
    
    /// Fold float binary operation
    fn fold_float_op(&self, op: HirBinaryOp, left: f64, right: f64) -> Option<HirConstant> {
        let result = match op {
            HirBinaryOp::Add => left + right,
            HirBinaryOp::Sub => left - right,
            HirBinaryOp::Mul => left * right,
            HirBinaryOp::Div => left / right,
            HirBinaryOp::Mod => left % right,
            HirBinaryOp::Eq => return Some(HirConstant::Bool(left == right)),
            HirBinaryOp::Ne => return Some(HirConstant::Bool(left != right)),
            HirBinaryOp::Lt => return Some(HirConstant::Bool(left < right)),
            HirBinaryOp::Le => return Some(HirConstant::Bool(left <= right)),
            HirBinaryOp::Gt => return Some(HirConstant::Bool(left > right)),
            HirBinaryOp::Ge => return Some(HirConstant::Bool(left >= right)),
            _ => return None, // Bitwise ops not valid for floats
        };
        
        Some(HirConstant::Float(result))
    }
    
    /// Fold boolean binary operation
    fn fold_bool_op(&self, op: HirBinaryOp, left: bool, right: bool) -> Option<HirConstant> {
        let result = match op {
            HirBinaryOp::Eq => left == right,
            HirBinaryOp::Ne => left != right,
            HirBinaryOp::And | HirBinaryOp::BitAnd => left && right,
            HirBinaryOp::Or | HirBinaryOp::BitOr => left || right,
            _ => return None,
        };
        
        Some(HirConstant::Bool(result))
    }
    
    /// Clear constant tracking
    pub fn clear(&mut self) {
        self.constants.clear();
        self.use_counts.clear();
    }
}

impl Default for ConstantFolder {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of folding an instruction
enum FoldResult {
    /// Keep the instruction unchanged
    Keep(HirInstruction),
    /// Replace with a different instruction
    Replace(HirInstruction),
    /// Remove the instruction entirely
    Remove,
    /// Instruction folded to a constant (no instruction needed)
    FoldToConst(HirConstant),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fold_int_add() {
        let folder = ConstantFolder::new();
        let result = folder.fold_int_op(
            HirBinaryOp::Add,
            HirConstant::Int(2),
            HirConstant::Int(3),
        );
        assert_eq!(result, Some(HirConstant::Int(5)));
    }
    
    #[test]
    fn test_fold_int_div_by_zero() {
        let folder = ConstantFolder::new();
        let result = folder.fold_int_op(
            HirBinaryOp::Div,
            HirConstant::Int(10),
            HirConstant::Int(0),
        );
        assert_eq!(result, None); // Should return None for div by zero
    }
    
    #[test]
    fn test_fold_bool_and() {
        let folder = ConstantFolder::new();
        let result = folder.fold_bool_op(
            HirBinaryOp::And,
            HirConstant::Bool(true),
            HirConstant::Bool(false),
        );
        assert_eq!(result, Some(HirConstant::Bool(false)));
    }
    
    #[test]
    fn test_fold_float_add() {
        let folder = ConstantFolder::new();
        let result = folder.fold_float_op(
            HirBinaryOp::Add,
            HirConstant::Float(1.5),
            HirConstant::Float(2.5),
        );
        assert_eq!(result, Some(HirConstant::Float(4.0)));
    }
}
