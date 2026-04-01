//! Compiler Optimization Passes
//! 
//! This module implements various optimization passes for the Hint compiler:
//! - Constant folding and propagation
//! - Dead code elimination
//! - Function inlining
//! - Loop optimizations
//! - Memory optimizations
//! - Instruction combining

pub mod constant_fold;
pub mod dce;
pub mod inline;
pub mod loop_opt;
pub mod mem_opt;
pub mod inst_combine;
pub mod pipeline;

pub use constant_fold::{ConstantFolder, ConstantFoldPass};
pub use dce::{DeadCodeEliminator, DCEPass};
pub use inline::{Inliner, InlinePass};
pub use loop_opt::{LoopOptimizer, LoopOptPass};
pub use mem_opt::{MemoryOptimizer, MemOptPass};
pub use inst_combine::{InstructionCombiner, InstCombinePass};
pub use pipeline::{OptimizationPipeline, OptimizationLevel};

use crate::ir::{HIR, HirFunction, HirBlock, HirInstruction};
use crate::diagnostics::{Diagnostic, DiagnosticsEngine};

/// Optimization statistics
#[derive(Debug, Default, Clone)]
pub struct OptimizationStats {
    /// Number of instructions removed
    pub instructions_removed: usize,
    /// Number of instructions added
    pub instructions_added: usize,
    /// Number of functions inlined
    pub functions_inlined: usize,
    /// Number of loops optimized
    pub loops_optimized: usize,
    /// Number of constants folded
    pub constants_folded: usize,
    /// Number of dead code blocks removed
    pub dead_blocks_removed: usize,
    /// Compilation time in milliseconds
    pub time_ms: u64,
}

impl OptimizationStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Merge statistics from another run
    pub fn merge(&mut self, other: &OptimizationStats) {
        self.instructions_removed += other.instructions_removed;
        self.instructions_added += other.instructions_added;
        self.functions_inlined += other.functions_inlined;
        self.loops_optimized += other.loops_optimized;
        self.constants_folded += other.constants_folded;
        self.dead_blocks_removed += other.dead_blocks_removed;
        self.time_ms += other.time_ms;
    }
    
    /// Get net instruction change
    pub fn net_instruction_change(&self) -> i64 {
        self.instructions_added as i64 - self.instructions_removed as i64
    }
}

/// Optimization pass trait
pub trait OptimizationPass {
    /// Pass name
    fn name(&self) -> &'static str;
    
    /// Run the optimization pass
    fn run(&mut self, hir: &mut HIR) -> Result<OptimizationStats, String>;
    
    /// Check if the pass should run
    fn should_run(&self, level: OptimizationLevel) -> bool;
}

/// Optimization result
#[derive(Debug)]
pub struct OptimizationResult {
    /// Optimized HIR
    pub hir: HIR,
    /// Statistics
    pub stats: OptimizationStats,
    /// Warnings generated during optimization
    pub warnings: Vec<Diagnostic>,
}

/// Main optimizer
pub struct Optimizer {
    pipeline: OptimizationPipeline,
    stats: OptimizationStats,
    diagnostics: DiagnosticsEngine,
}

impl Optimizer {
    pub fn new(level: OptimizationLevel) -> Self {
        Self {
            pipeline: OptimizationPipeline::new(level),
            stats: OptimizationStats::new(),
            diagnostics: DiagnosticsEngine::new(),
        }
    }
    
    /// Create optimizer with custom pipeline
    pub fn with_pipeline(pipeline: OptimizationPipeline) -> Self {
        Self {
            pipeline,
            stats: OptimizationStats::new(),
            diagnostics: DiagnosticsEngine::new(),
        }
    }
    
    /// Optimize a HIR program
    pub fn optimize(&mut self, mut hir: HIR) -> Result<OptimizationResult, String> {
        let start_time = std::time::Instant::now();
        
        // Run all passes in the pipeline
        for pass in self.pipeline.passes.iter_mut() {
            match pass.run(&mut hir) {
                Ok(pass_stats) => {
                    self.stats.merge(&pass_stats);
                }
                Err(e) => {
                    self.diagnostics.emit(
                        Diagnostic::warning(format!("Optimization pass '{}' failed: {}", pass.name(), e))
                    );
                    // Continue with other passes
                }
            }
        }
        
        self.stats.time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(OptimizationResult {
            hir,
            stats: self.stats.clone(),
            warnings: self.diagnostics.diagnostics().to_vec(),
        })
    }
    
    /// Get optimization statistics
    pub fn stats(&self) -> &OptimizationStats {
        &self.stats
    }
    
    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsEngine {
        &self.diagnostics
    }
}

/// Helper function to check if an instruction has side effects
pub fn has_side_effects(instr: &HirInstruction) -> bool {
    match instr {
        HirInstruction::Call { function, .. } => {
            // Built-in functions may have side effects
            matches!(function.as_str(), "print" | "println" | "exit" | "abort")
        }
        HirInstruction::StoreVar { .. } => true,
        HirInstruction::Return { .. } => true,
        HirInstruction::Print { .. } => true,
        HirInstruction::Jump { .. } => true,
        HirInstruction::Branch { .. } => true,
        _ => false,
    }
}

/// Helper function to check if a value is used
pub fn is_value_used(value_id: usize, block: &HirBlock) -> bool {
    for instr in &block.instructions {
        if instruction_uses_value(instr, value_id) {
            return true;
        }
    }
    false
}

/// Check if an instruction uses a specific value
pub fn instruction_uses_value(instr: &HirInstruction, value_id: usize) -> bool {
    match instr {
        HirInstruction::LoadVar { source, .. } => false, // Uses variable, not value
        HirInstruction::StoreVar { source, .. } => source.id == value_id,
        HirInstruction::BinaryOp { left, right, .. } => left.id == value_id || right.id == value_id,
        HirInstruction::Call { args, .. } => args.iter().any(|a| a.id == value_id),
        HirInstruction::Return { value } => value.as_ref().map(|v| v.id == value_id).unwrap_or(false),
        HirInstruction::Print { value } => value.id == value_id,
        HirInstruction::Branch { condition, .. } => condition.id == value_id,
        _ => false,
    }
}

/// Get the definition of a value (which instruction produced it)
pub fn find_value_definition(value_id: usize, block: &HirBlock) -> Option<usize> {
    for (i, instr) in block.instructions.iter().enumerate() {
        if instruction_defines_value(instr, value_id) {
            return Some(i);
        }
    }
    None
}

/// Check if an instruction defines a specific value
pub fn instruction_defines_value(instr: &HirInstruction, value_id: usize) -> bool {
    match instr {
        HirInstruction::LoadConst { dest, .. } => dest.id == value_id,
        HirInstruction::LoadVar { dest, .. } => dest.id == value_id,
        HirInstruction::BinaryOp { dest, .. } => dest.id == value_id,
        HirInstruction::Call { dest: Some(dest), .. } => dest.id == value_id,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{HirValue, HirConstant};
    use crate::semantics::{HintType, IntSize};
    
    #[test]
    fn test_optimization_stats() {
        let mut stats = OptimizationStats::new();
        stats.instructions_removed = 10;
        stats.instructions_added = 5;
        
        assert_eq!(stats.net_instruction_change(), -5);
    }
    
    #[test]
    fn test_has_side_effects() {
        let print_instr = HirInstruction::Print {
            value: HirValue::new(0, HintType::String),
        };
        assert!(has_side_effects(&print_instr));
        
        let const_instr = HirInstruction::LoadConst {
            dest: HirValue::new(0, HintType::Int(IntSize::I64)),
            value: HirConstant::Int(42),
        };
        assert!(!has_side_effects(&const_instr));
    }
    
    #[test]
    fn test_stats_merge() {
        let mut stats1 = OptimizationStats::new();
        stats1.instructions_removed = 10;
        
        let stats2 = OptimizationStats {
            instructions_removed: 5,
            instructions_added: 3,
            ..Default::default()
        };
        
        stats1.merge(&stats2);
        assert_eq!(stats1.instructions_removed, 15);
        assert_eq!(stats1.instructions_added, 3);
    }
}
