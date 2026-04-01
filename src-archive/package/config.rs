//! Project Configuration (Hint.toml)
//! 
//! Defines the structure of Hint.toml configuration files.

use std::collections::HashMap;
use std::path::PathBuf;

/// Complete project configuration
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Package metadata
    pub package: PackageConfig,
    /// Build configuration
    pub build: BuildConfig,
    /// Library configuration (if applicable)
    pub lib: Option<LibraryConfig>,
    /// Runtime dependencies
    pub dependencies: HashMap<String, DependencySpec>,
    /// Development dependencies
    pub dev_dependencies: HashMap<String, DependencySpec>,
    /// Build dependencies
    pub build_dependencies: HashMap<String, DependencySpec>,
    /// Test configuration
    pub test: Option<TestConfig>,
    /// Profile configurations
    pub profiles: HashMap<String, ProfileConfig>,
}

impl ProjectConfig {
    /// Parse configuration from TOML string
    pub fn parse(content: &str) -> Result<Self, String> {
        // Simple TOML parser (in production, use toml crate)
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Section headers
            if line.starts_with('[') && line.ends_with(']') {
                continue;
            }
            
            // Key-value pairs
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim().trim_matches('"');
                
                match key {
                    "name" => config.package.name = value.to_string(),
                    "version" => config.package.version = value.to_string(),
                    "authors" => config.package.authors = value.split(',').map(|s| s.trim().to_string()).collect(),
                    "edition" => config.package.edition = value.to_string(),
                    "description" => config.package.description = Some(value.to_string()),
                    "license" => config.package.license = Some(value.to_string()),
                    "repository" => config.package.repository = Some(value.to_string()),
                    "target" => config.build.target = value.to_string(),
                    "optimization" => config.build.optimization = value.to_string(),
                    "output" => config.build.output = Some(value.to_string()),
                    _ => {} // Ignore unknown keys
                }
            }
        }
        
        Ok(config)
    }
    
    /// Get the project name
    pub fn name(&self) -> &str {
        &self.package.name
    }
    
    /// Get the project version
    pub fn version(&self) -> &str {
        &self.package.version
    }
    
    /// Get the main source file
    pub fn main_source(&self) -> PathBuf {
        PathBuf::from("src").join("main.ht")
    }
    
    /// Get the library source file (if library)
    pub fn lib_source(&self) -> Option<PathBuf> {
        self.lib.as_ref().map(|lib| {
            PathBuf::from("src").join(format!("{}.ht", lib.name))
        })
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            package: PackageConfig::default(),
            build: BuildConfig::default(),
            lib: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build_dependencies: HashMap::new(),
            test: None,
            profiles: HashMap::new(),
        }
    }
}

/// Package metadata
#[derive(Debug, Clone)]
pub struct PackageConfig {
    /// Package name
    pub name: String,
    /// Package version (semver)
    pub version: String,
    /// Authors
    pub authors: Vec<String>,
    /// Edition (e.g., "2024")
    pub edition: String,
    /// Description
    pub description: Option<String>,
    /// License
    pub license: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Keywords
    pub keywords: Vec<String>,
    /// Categories
    pub categories: Vec<String>,
}

impl PackageConfig {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            authors: Vec::new(),
            edition: "2024".to_string(),
            description: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: Vec::new(),
            categories: Vec::new(),
        }
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self::new("untitled", "0.1.0")
    }
}

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Target platform
    pub target: String,
    /// Optimization level
    pub optimization: String,
    /// Output name
    pub output: Option<String>,
    /// Output directory
    pub output_dir: Option<PathBuf>,
    /// Enable debug info
    pub debug: bool,
    /// Enable LTO
    pub lto: bool,
    /// Features to enable
    pub features: Vec<String>,
    /// Default features
    pub default_features: bool,
}

impl BuildConfig {
    pub fn new() -> Self {
        Self {
            target: "native".to_string(),
            optimization: "speed".to_string(),
            output: None,
            output_dir: None,
            debug: false,
            lto: false,
            features: Vec::new(),
            default_features: true,
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Library configuration
#[derive(Debug, Clone)]
pub struct LibraryConfig {
    /// Library name
    pub name: String,
    /// Library type
    pub lib_type: LibraryType,
    /// Source file
    pub source: Option<PathBuf>,
}

/// Library type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryType {
    /// Static library
    Static,
    /// Dynamic library
    Dynamic,
    /// WebAssembly module
    Wasm,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            lib_type: LibraryType::Static,
            source: None,
        }
    }
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Test harness to use
    pub harness: String,
    /// Test source directory
    pub source_dir: Option<PathBuf>,
    /// Parallel test execution
    pub parallel: bool,
    /// Fail fast
    pub fail_fast: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            harness: "default".to_string(),
            source_dir: None,
            parallel: true,
            fail_fast: false,
        }
    }
}

/// Profile configuration
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    /// Inherit from another profile
    pub inherits: Option<String>,
    /// Optimization level
    pub optimization: Option<String>,
    /// Debug info
    pub debug: Option<bool>,
    /// LTO
    pub lto: Option<bool>,
}

/// Dependency specification
#[derive(Debug, Clone)]
pub struct DependencySpec {
    /// Version requirement (semver)
    pub version: String,
    /// Git repository
    pub git: Option<String>,
    /// Git branch
    pub branch: Option<String>,
    /// Git tag
    pub tag: Option<String>,
    /// Git revision
    pub rev: Option<String>,
    /// Local path
    pub path: Option<PathBuf>,
    /// Features to enable
    pub features: Vec<String>,
    /// Use default features
    pub default_features: bool,
    /// Optional dependency
    pub optional: bool,
}

impl DependencySpec {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: None,
            features: Vec::new(),
            default_features: true,
            optional: false,
        }
    }
    
    pub fn from_git(git: &str) -> Self {
        Self {
            version: String::new(),
            git: Some(git.to_string()),
            branch: None,
            tag: None,
            rev: None,
            path: None,
            features: Vec::new(),
            default_features: true,
            optional: false,
        }
    }
    
    pub fn from_path(path: &str) -> Self {
        Self {
            version: String::new(),
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: Some(PathBuf::from(path)),
            features: Vec::new(),
            default_features: true,
            optional: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_config() {
        let content = r#"
[package]
name = "my-project"
version = "1.0.0"
authors = ["John Doe"]
edition = "2024"
description = "My Hint project"

[build]
target = "native"
optimization = "speed"
"#;
        
        let config = ProjectConfig::parse(content).unwrap();
        assert_eq!(config.package.name, "my-project");
        assert_eq!(config.package.version, "1.0.0");
        assert_eq!(config.build.target, "native");
    }
    
    #[test]
    fn test_dependency_spec() {
        let dep = DependencySpec::new("1.0.0");
        assert_eq!(dep.version, "1.0.0");
        assert!(dep.git.is_none());
        
        let git_dep = DependencySpec::from_git("https://github.com/user/repo");
        assert!(git_dep.git.is_some());
        
        let path_dep = DependencySpec::from_path("../local-lib");
        assert!(path_dep.path.is_some());
    }
}
