//! Object file emission.

use cranelift_module::{DataId, FuncId};
use cranelift_object::{ObjectModule, ObjectProduct};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Object file emitter
pub struct ObjectEmitter {
    module: ObjectModule,
    functions: HashMap<String, FuncId>,
    data: HashMap<String, DataId>,
}

impl ObjectEmitter {
    pub fn new(module: ObjectModule) -> Self {
        Self { module, functions: HashMap::new(), data: HashMap::new() }
    }
    
    pub fn register_function(&mut self, name: String, id: FuncId) {
        self.functions.insert(name, id);
    }
    
    pub fn register_data(&mut self, name: String, id: DataId) {
        self.data.insert(name, id);
    }
    
    pub fn emit(self) -> Result<Vec<u8>, String> {
        let product = self.module.finish();
        let bytes = product.emit().map_err(|e| format!("Failed to emit: {:?}", e))?;
        Ok(bytes)
    }
    
    pub fn emit_to_file(self, path: &Path) -> Result<(), String> {
        let bytes = self.emit()?;
        fs::write(path, &bytes).map_err(|e| format!("Failed to write: {}", e))?;
        Ok(())
    }
    
    pub fn object_extension(triple: &target_lexicon::Triple) -> &'static str {
        match triple.operating_system {
            target_lexicon::OperatingSystem::Windows => "obj",
            _ => "o",
        }
    }
}

/// Finished object file
pub struct FinishedObject {
    pub bytes: Vec<u8>,
}

impl FinishedObject {
    pub fn from_product(product: ObjectProduct) -> Self {
        Self { bytes: product.emit().expect("Failed to emit") }
    }
    
    pub fn write_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        fs::write(path, &self.bytes)
    }
}

/// Linker configuration
pub struct LinkerConfig {
    pub linker: String,
    pub output: String,
    pub inputs: Vec<String>,
    pub libraries: Vec<String>,
    pub lib_paths: Vec<String>,
    pub flags: Vec<String>,
    pub entry: Option<String>,
    pub shared: bool,
    pub pie: bool,
}

impl LinkerConfig {
    pub fn new() -> Self {
        Self {
            linker: Self::default_linker(),
            output: "a.out".to_string(),
            inputs: Vec::new(),
            libraries: Vec::new(),
            lib_paths: Vec::new(),
            flags: Vec::new(),
            entry: None,
            shared: false,
            pie: false,
        }
    }
    
    pub fn default_linker() -> String {
        let triple = target_lexicon::Triple::host();
        match triple.operating_system {
            target_lexicon::OperatingSystem::Windows => "link.exe".to_string(),
            _ => "ld".to_string(),
        }
    }
    
    pub fn add_input(&mut self, path: &str) {
        self.inputs.push(path.to_string());
    }
    
    pub fn add_library(&mut self, name: &str) {
        self.libraries.push(name.to_string());
    }
    
    pub fn build_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.linker);
        cmd.arg(format!("-o={}", self.output));
        
        if let Some(entry) = &self.entry {
            cmd.arg(format!("-e={}", entry));
        }
        
        if self.shared {
            cmd.arg("-shared");
        }
        
        for path in &self.lib_paths {
            cmd.arg(format!("-L{}", path));
        }
        
        for lib in &self.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        
        for input in &self.inputs {
            cmd.arg(input);
        }
        
        for flag in &self.flags {
            cmd.arg(flag);
        }
        
        cmd
    }
    
    pub fn run(&self) -> Result<(), String> {
        let mut cmd = self.build_command();
        let output = cmd.output().map_err(|e| format!("Failed to run linker: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Linking failed:\n{}", stderr));
        }
        
        Ok(())
    }
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self::new()
    }
}
