//! Calling convention and ABI handling for native codegen.

use cranelift_codegen::isa::CallConv;
use target_lexicon::{OperatingSystem, Triple};

/// Supported calling conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    /// System V AMD64 ABI (Linux, macOS, FreeBSD, etc.)
    SystemV,
    /// Windows x64 fastcall
    WindowsFastcall,
}

impl CallingConvention {
    /// Determine calling convention from target triple
    pub fn from_triple(triple: &Triple) -> Self {
        match triple.operating_system {
            OperatingSystem::Windows => CallingConvention::WindowsFastcall,
            _ => CallingConvention::SystemV,
        }
    }
    
    /// Convert to Cranelift CallConv
    pub fn to_cranelift(self) -> CallConv {
        match self {
            CallingConvention::SystemV => CallConv::SystemV,
            CallingConvention::WindowsFastcall => CallConv::WindowsFastcall,
        }
    }
    
    /// Get the number of register-passed integer arguments
    pub fn num_int_arg_regs(self) -> usize {
        match self {
            CallingConvention::SystemV => 6,
            CallingConvention::WindowsFastcall => 4,
        }
    }
    
    /// Get the number of register-passed float arguments
    pub fn num_float_arg_regs(self) -> usize {
        match self {
            CallingConvention::SystemV => 8,
            CallingConvention::WindowsFastcall => 4,
        }
    }
    
    /// Get the size of the shadow space (Windows only)
    pub fn shadow_space_size(self) -> usize {
        match self {
            CallingConvention::WindowsFastcall => 32,
            CallingConvention::SystemV => 0,
        }
    }
    
    /// Get the required stack alignment before a call
    pub fn stack_alignment(self) -> usize {
        16
    }
}
