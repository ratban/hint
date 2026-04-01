//! Package Manager and Build System
//! 
//! This module provides:
//! - Project configuration (Hint.toml)
//! - Dependency management
//! - Build configuration
//! - Package publishing

pub mod config;
pub mod deps;
pub mod build;
pub mod publish;
pub mod registry;

pub use config::{ProjectConfig, BuildConfig, PackageConfig};
pub use deps::DependencyResolver;
pub use config::{DependencySpec, ProjectConfig};
pub use build::{Builder, BuildMode, BuildResult};
pub use publish::{Publisher, PackageRegistry};

use std::path::{Path, PathBuf};
use std::fs;

/// Hint package manager
pub struct PackageManager {
    /// Project root directory
    root: PathBuf,
    /// Project configuration
    config: Option<ProjectConfig>,
    /// Build configuration
    build_config: BuildConfig,
}

impl PackageManager {
    /// Create a new package manager for a project
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            config: None,
            build_config: BuildConfig::default(),
        }
    }
    
    /// Load project configuration
    pub fn load_config(&mut self) -> Result<(), String> {
        let config_path = self.root.join("Hint.toml");
        
        if !config_path.exists() {
            return Err(format!("Hint.toml not found in {:?}", self.root));
        }
        
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read Hint.toml: {}", e))?;
        
        self.config = Some(ProjectConfig::parse(&content)?);
        Ok(())
    }
    
    /// Get project configuration
    pub fn config(&self) -> Option<&ProjectConfig> {
        self.config.as_ref()
    }
    
    /// Set build mode
    pub fn set_build_mode(&mut self, mode: BuildMode) {
        self.build_config.mode = mode;
    }
    
    /// Set target
    pub fn set_target(&mut self, target: &str) {
        self.build_config.target = target.to_string();
    }
    
    /// Set output directory
    pub fn set_output(&mut self, output: &Path) {
        self.build_config.output_dir = output.to_path_buf();
    }
    
    /// Build the project
    pub fn build(&mut self) -> Result<BuildResult, String> {
        // Load config if not loaded
        if self.config.is_none() {
            self.load_config()?;
        }
        
        let mut builder = Builder::new(&self.build_config);
        builder.build(&self.root, self.config.as_ref().unwrap())
    }
    
    /// Clean build artifacts
    pub fn clean(&self) -> Result<(), String> {
        let target_dir = self.root.join("target");
        
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("Failed to remove target directory: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Run tests
    pub fn test(&self) -> Result<TestResult, String> {
        let mut builder = Builder::new(&self.build_config);
        builder.test(&self.root, self.config.as_ref().unwrap())
    }
    
    /// Install dependencies
    pub fn install_deps(&mut self) -> Result<(), String> {
        if self.config.is_none() {
            self.load_config()?;
        }
        
        let resolver = deps::DependencyResolver::new();
        resolver.resolve(self.config.as_ref().unwrap())?;
        
        Ok(())
    }
    
    /// Publish package to registry
    pub fn publish(&self, registry_url: &str) -> Result<(), String> {
        if self.config.is_none() {
            return Err("Project configuration not loaded".to_string());
        }
        
        let publisher = Publisher::new(registry_url);
        publisher.publish(&self.root, self.config.as_ref().unwrap())
    }
}

/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub failures: Vec<TestFailure>,
}

/// Test failure information
#[derive(Debug, Clone)]
pub struct TestFailure {
    pub name: String,
    pub message: String,
    pub location: String,
}

impl TestResult {
    pub fn success(&self) -> bool {
        self.failed == 0
    }
    
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped
    }
}

/// Initialize a new Hint project
pub fn init_project(path: &Path, name: &str) -> Result<(), String> {
    // Create directory structure
    let src_dir = path.join("src");
    let tests_dir = path.join("tests");
    
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;
    fs::create_dir_all(&tests_dir)
        .map_err(|e| format!("Failed to create tests directory: {}", e))?;
    
    // Create Hint.toml
    let config_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
authors = [""]
edition = "2024"

[dependencies]

[dev-dependencies]

[build]
target = "native"
optimization = "speed"
"#,
        name
    );
    
    fs::write(path.join("Hint.toml"), config_content)
        .map_err(|e| format!("Failed to create Hint.toml: {}", e))?;
    
    // Create main.ht
    let main_content = r#"Say "Hello, World!".
Stop the program.
"#;
    
    fs::write(src_dir.join("main.ht"), main_content)
        .map_err(|e| format!("Failed to create main.ht: {}", e))?;
    
    Ok(())
}

/// Create a new library project
pub fn init_library(path: &Path, name: &str) -> Result<(), String> {
    init_project(path, name)?;
    
    // Update Hint.toml for library
    let config_path = path.join("Hint.toml");
    let mut config = fs::read_to_string(&config_path)?;
    config.push_str("\n[lib]\nname = \"");
    config.push_str(name);
    config.push_str("\"\n");
    
    fs::write(&config_path, config)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_package_manager_creation() {
        let pm = PackageManager::new(Path::new("/tmp/test"));
        assert_eq!(pm.root, Path::new("/tmp/test"));
        assert!(pm.config.is_none());
    }
    
    #[test]
    fn test_test_result() {
        let result = TestResult {
            passed: 10,
            failed: 2,
            skipped: 1,
            duration_ms: 100,
            failures: Vec::new(),
        };
        
        assert!(!result.success());
        assert_eq!(result.total(), 13);
    }
}
