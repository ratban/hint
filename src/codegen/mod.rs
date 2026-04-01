//! Code generation backends.
//! 
//! Supports native and WebAssembly targets.

pub mod native;
pub mod wasm;

pub use crate::target::CompilationTarget;
use crate::ir::HIR;

/// Code generator trait
pub trait CodeGenerator {
    fn generate(&mut self, hir: &HIR) -> Result<Vec<u8>, String>;
    fn target(&self) -> &CompilationTarget;
}

/// Create code generator for target
pub fn create_generator(target: &CompilationTarget) -> Result<Box<dyn CodeGenerator>, String> {
    match target {
        CompilationTarget::Native |
        CompilationTarget::WindowsX64 |
        CompilationTarget::LinuxX64 |
        CompilationTarget::MacosX64 => {
            Ok(Box::new(native::NativeCodeGenerator::new(target.clone())))
        }
        CompilationTarget::Wasm32 => {
            Ok(Box::new(wasm::WasmCodeGenerator::new(target.clone())))
        }
        _ => Err(format!("Unsupported target: {:?}", target))
    }
}

pub use native::NativeCodeGenerator;
pub use wasm::WasmCodeGenerator;
