//! Function Inlining
//! 
//! Replaces function calls with the function body for performance.

use crate::ir::{HIR, HirFunction, HirBlock, HirInstruction, HirValue, HirConstant};
use crate::semantics::{HintType, IntSize};
use super::{OptimizationPass, OptimizationStats, OptimizationLevel};
use std::collections::HashMap;

/// Function inlining pass
pub struct InlinePass {
    stats: OptimizationStats,
    config: InlineConfig,
}

impl InlinePass {
    pub fn new() -> Self {
        Self {
            stats: OptimizationStats::new(),
            config: InlineConfig::default(),
        }
    }
    
    pub fn with_config(config: InlineConfig) -> Self {
        Self {
            stats: OptimizationStats::new(),
            config,
        }
    }
}

impl Default for InlinePass {
    fn default() -> Self {
        Self::new()
    }
}

/// Inlining configuration
#[derive(Debug, Clone)]
pub struct InlineConfig {
    /// Maximum function size (instructions) to inline
    pub max_function_size: usize,
    /// Maximum recursion depth for inlining
    pub max_recursion_depth: usize,
    /// Always inline these functions
    pub always_inline: Vec<String>,
    /// Never inline these functions
    pub never_inline: Vec<String>,
}

impl Default for InlineConfig {
    fn default() -> Self {
        Self {
            max_function_size: 20,
            max_recursion_depth: 3,
            always_inline: Vec::new(),
            never_inline: Vec::new(),
        }
    }
}

impl OptimizationPass for InlinePass {
    fn name(&self) -> &'static str {
        "inline"
    }
    
    fn run(&mut self, hir: &mut HIR) -> Result<OptimizationStats, String> {
        self.stats = OptimizationStats::new();
        
        let mut inliner = Inliner::new(self.config.clone());
        
        // Inline functions in entry point
        if let Some(entry) = &mut hir.entry_point {
            inliner.inline_block(entry, hir, &mut self.stats, 0)?;
        }
        
        // Inline functions in functions
        for func in &mut hir.functions {
            inliner.inline_block(&mut func.body, hir, &mut self.stats, 0)?;
        }
        
        Ok(self.stats.clone())
    }
    
    fn should_run(&self, level: OptimizationLevel) -> bool {
        matches!(level, OptimizationLevel::Speed)
    }
}

/// Function inliner implementation
pub struct Inliner {
    config: InlineConfig,
    /// Function cache for quick lookup
    functions: HashMap<String, HirFunction>,
}

impl Inliner {
    pub fn new(config: InlineConfig) -> Self {
        Self {
            config,
            functions: HashMap::new(),
        }
    }
    
    /// Inline function calls in a block
    pub fn inline_block(&mut self, block: &mut HirBlock, hir: &HIR, stats: &mut OptimizationStats, depth: usize) -> Result<(), String> {
        if depth > self.config.max_recursion_depth {
            return Ok(()); // Prevent infinite recursion
        }
        
        // Cache functions
        for func in &hir.functions {
            self.functions.insert(func.name.clone(), func.clone());
        }
        
        let mut new_instructions = Vec::new();
        
        for instr in block.instructions.drain(..) {
            match instr {
                HirInstruction::Call { dest, function, args } => {
                    // Check if we should inline this function
                    if let Some(func) = self.functions.get(&function) {
                        if self.should_inline(&function, func) {
                            // Inline the function
                            let inlined = self.inline_function(func, &args, dest.as_ref(), stats)?;
                            new_instructions.extend(inlined);
                            stats.functions_inlined += 1;
                            continue;
                        }
                    }
                    
                    // Keep the call
                    new_instructions.push(HirInstruction::Call { dest, function, args });
                }
                _ => {
                    new_instructions.push(instr);
                }
            }
        }
        
        block.instructions = new_instructions;
        Ok(())
    }
    
    /// Check if a function should be inlined
    fn should_inline(&self, name: &str, func: &HirFunction) -> bool {
        // Check never_inline list
        if self.config.never_inline.contains(&name.to_string()) {
            return false;
        }
        
        // Check always_inline list
        if self.config.always_inline.contains(&name.to_string()) {
            return true;
        }
        
        // Check function size
        let func_size = func.body.instructions.len();
        if func_size > self.config.max_function_size {
            return false;
        }
        
        // Small functions are good candidates
        func_size <= 5
    }
    
    /// Inline a function call
    fn inline_function(
        &self,
        func: &HirFunction,
        args: &[HirValue],
        dest: Option<&HirValue>,
        stats: &mut OptimizationStats,
    ) -> Result<Vec<HirInstruction>, String> {
        let mut inlined = Vec::new();
        
        // Create parameter bindings
        let mut param_bindings: HashMap<String, HirValue> = HashMap::new();
        for (i, param) in func.params.iter().enumerate() {
            if i < args.len() {
                param_bindings.insert(param.name.clone(), args[i]);
            }
        }
        
        // Copy function body with parameter substitution
        for instr in &func.body.instructions {
            let substituted = self.substitute_instruction(instr, &param_bindings, dest, stats)?;
            inlined.push(substituted);
        }
        
        Ok(inlined)
    }
    
    /// Substitute parameters in an instruction
    fn substitute_instruction(
        &self,
        instr: &HirInstruction,
        bindings: &HashMap<String, HirValue>,
        dest: Option<&HirValue>,
        stats: &mut OptimizationStats,
    ) -> Result<HirInstruction, String> {
        match instr {
            HirInstruction::LoadConst { dest, value } => {
                Ok(HirInstruction::LoadConst { dest: *dest, value: value.clone() })
            }
            
            HirInstruction::LoadVar { dest, source } => {
                // Check if source is a parameter
                if let Some(param_value) = bindings.get(source) {
                    // Replace with constant load
                    stats.instructions_added += 1;
                    Ok(HirInstruction::LoadConst { dest: *dest, value: HirConstant::Int(0) })
                } else {
                    Ok(HirInstruction::LoadVar { dest: *dest, source: source.clone() })
                }
            }
            
            HirInstruction::StoreVar { dest, source } => {
                Ok(HirInstruction::StoreVar { dest: dest.clone(), source: *source })
            }
            
            HirInstruction::BinaryOp { dest, op, left, right, result_type } => {
                Ok(HirInstruction::BinaryOp {
                    dest: *dest,
                    op: *op,
                    left: *left,
                    right: *right,
                    result_type: result_type.clone(),
                })
            }
            
            HirInstruction::Return { value } => {
                // Replace return with assignment to dest
                if let (Some(v), Some(d)) = (value, dest) {
                    stats.instructions_added += 1;
                    Ok(HirInstruction::StoreVar { dest: d.id.to_string(), source: *v })
                } else {
                    Ok(HirInstruction::Return { value: *value })
                }
            }
            
            _ => Ok(instr.clone()),
        }
    }
}

/// Simple inlining heuristic
pub fn should_inline_function(func_size: usize, call_frequency: usize) -> bool {
    // Small functions are always good candidates
    if func_size <= 3 {
        return true;
    }
    
    // Frequently called small functions
    if func_size <= 10 && call_frequency >= 5 {
        return true;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inline_config() {
        let config = InlineConfig::default();
        assert_eq!(config.max_function_size, 20);
        assert_eq!(config.max_recursion_depth, 3);
    }
    
    #[test]
    fn test_should_inline_heuristic() {
        assert!(should_inline_function(2, 1)); // Small function
        assert!(should_inline_function(5, 10)); // Frequently called
        assert!(!should_inline_function(50, 1)); // Large, rarely called
    }
    
    #[test]
    fn test_inliner_creation() {
        let inliner = Inliner::new(InlineConfig::default());
        assert!(inliner.functions.is_empty());
    }
}
