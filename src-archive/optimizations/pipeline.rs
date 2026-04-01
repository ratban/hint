//! Optimization Pipeline
//! 
//! Manages the sequence of optimization passes.

use super::{OptimizationPass, OptimizationStats};
use super::constant_fold::ConstantFoldPass;
use super::dce::DCEPass;
use super::inline::InlinePass;
use super::loop_opt::LoopOptPass;
use super::mem_opt::MemOptPass;
use super::inst_combine::InstCombinePass;
use crate::ir::HIR;

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimizations
    None,
    /// Basic optimizations (fast compile time)
    Speed,
    /// Aggressive optimizations (small code)
    SpeedAndSize,
    /// Maximum optimizations (slow compile time)
    Aggressive,
}

impl OptimizationLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" | "O0" => Some(OptimizationLevel::None),
            "speed" | "O1" => Some(OptimizationLevel::Speed),
            "size" | "Os" => Some(OptimizationLevel::SpeedAndSize),
            "aggressive" | "O2" | "O3" => Some(OptimizationLevel::Aggressive),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            OptimizationLevel::None => "none",
            OptimizationLevel::Speed => "speed",
            OptimizationLevel::SpeedAndSize => "size",
            OptimizationLevel::Aggressive => "aggressive",
        }
    }
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        OptimizationLevel::Speed
    }
}

/// Optimization pipeline
pub struct OptimizationPipeline {
    /// Optimization level
    level: OptimizationLevel,
    /// Passes to run
    pub passes: Vec<Box<dyn OptimizationPass>>,
}

impl OptimizationPipeline {
    pub fn new(level: OptimizationLevel) -> Self {
        let mut pipeline = Self {
            level,
            passes: Vec::new(),
        };
        
        pipeline.add_passes_for_level(level);
        pipeline
    }
    
    /// Add passes for optimization level
    fn add_passes_for_level(&mut self, level: OptimizationLevel) {
        match level {
            OptimizationLevel::None => {
                // No passes
            }
            OptimizationLevel::Speed => {
                self.add_pass(ConstantFoldPass::new());
                self.add_pass(DCEPass::new());
                self.add_pass(InstCombinePass::new());
            }
            OptimizationLevel::SpeedAndSize => {
                self.add_pass(ConstantFoldPass::new());
                self.add_pass(DCEPass::new());
                self.add_pass(InstCombinePass::new());
                self.add_pass(MemOptPass::new());
            }
            OptimizationLevel::Aggressive => {
                self.add_pass(ConstantFoldPass::new());
                self.add_pass(DCEPass::new());
                self.add_pass(InstCombinePass::new());
                self.add_pass(InlinePass::new());
                self.add_pass(LoopOptPass::new());
                self.add_pass(MemOptPass::new());
                // Run DCE again after inlining
                self.add_pass(DCEPass::new());
            }
        }
    }
    
    /// Add a pass to the pipeline
    pub fn add_pass<P: OptimizationPass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }
    
    /// Run the pipeline
    pub fn run(&mut self, hir: &mut HIR) -> Result<OptimizationStats, String> {
        let mut total_stats = OptimizationStats::new();
        
        for pass in &mut self.passes {
            if pass.should_run(self.level) {
                let stats = pass.run(hir)?;
                total_stats.merge(&stats);
            }
        }
        
        Ok(total_stats)
    }
    
    /// Get optimization level
    pub fn level(&self) -> OptimizationLevel {
        self.level
    }
    
    /// Get pass names
    pub fn pass_names(&self) -> Vec<&str> {
        self.passes.iter().map(|p| p.name()).collect()
    }
}

impl Default for OptimizationPipeline {
    fn default() -> Self {
        Self::new(OptimizationLevel::default())
    }
}

/// Create a standard optimization pipeline
pub fn create_pipeline(level: OptimizationLevel) -> OptimizationPipeline {
    OptimizationPipeline::new(level)
}

/// Run optimizations on HIR
pub fn optimize(hir: &mut HIR, level: OptimizationLevel) -> Result<OptimizationStats, String> {
    let mut pipeline = create_pipeline(level);
    pipeline.run(hir)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_level_from_str() {
        assert_eq!(OptimizationLevel::from_str("O0"), Some(OptimizationLevel::None));
        assert_eq!(OptimizationLevel::from_str("O1"), Some(OptimizationLevel::Speed));
        assert_eq!(OptimizationLevel::from_str("O2"), Some(OptimizationLevel::Aggressive));
        assert_eq!(OptimizationLevel::from_str("invalid"), None);
    }
    
    #[test]
    fn test_pipeline_creation() {
        let pipeline = OptimizationPipeline::new(OptimizationLevel::None);
        assert!(pipeline.passes.is_empty());
        
        let pipeline = OptimizationPipeline::new(OptimizationLevel::Speed);
        assert!(!pipeline.passes.is_empty());
    }
    
    #[test]
    fn test_pipeline_pass_names() {
        let pipeline = OptimizationPipeline::new(OptimizationLevel::Speed);
        let names = pipeline.pass_names();
        assert!(names.contains(&"constant-fold"));
        assert!(names.contains(&"dce"));
    }
}
