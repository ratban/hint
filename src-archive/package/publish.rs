//! Package Publishing
//! 
//! Handles publishing packages to registries.

use std::path::{Path, PathBuf};
use std::fs;
use super::config::ProjectConfig;

/// Package publisher
pub struct Publisher {
    /// Registry URL
    registry_url: String,
    /// Authentication token
    token: Option<String>,
}

impl Publisher {
    pub fn new(registry_url: &str) -> Self {
        Self {
            registry_url: registry_url.to_string(),
            token: None,
        }
    }
    
    pub fn with_token(registry_url: &str, token: &str) -> Self {
        Self {
            registry_url: registry_url.to_string(),
            token: Some(token.to_string()),
        }
    }
    
    /// Publish a package
    pub fn publish(&self, root: &Path, config: &ProjectConfig) -> Result<(), String> {
        // Validate package
        self.validate_package(config)?;
        
        // Create package archive
        let archive = self.create_archive(root, config)?;
        
        // Upload to registry
        self.upload(&archive)?;
        
        Ok(())
    }
    
    /// Validate package before publishing
    fn validate_package(&self, config: &ProjectConfig) -> Result<(), String> {
        // Check required fields
        if config.package.name.is_empty() {
            return Err("Package name is required".to_string());
        }
        
        if config.package.version.is_empty() {
            return Err("Package version is required".to_string());
        }
        
        // Validate semver
        if !self.is_valid_semver(&config.package.version) {
            return Err(format!("Invalid version: {}. Must be valid semver.", config.package.version));
        }
        
        // Check for reserved names
        let reserved = ["hint", "std", "core", "test"];
        if reserved.contains(&config.package.name.as_str()) {
            return Err(format!("'{}' is a reserved package name", config.package.name));
        }
        
        Ok(())
    }
    
    /// Check if version is valid semver
    fn is_valid_semver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        for part in parts {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }
        
        true
    }
    
    /// Create package archive
    fn create_archive(&self, root: &Path, config: &ProjectConfig) -> Result<PathBuf, String> {
        let archive_name = format!("{}-{}.tar.gz", config.package.name, config.package.version);
        let archive_path = root.join("target").join(&archive_name);
        
        // Ensure target directory exists
        fs::create_dir_all(root.join("target"))
            .map_err(|e| format!("Failed to create target directory: {}", e))?;
        
        // In production, would create actual tar.gz archive
        // For now, create a placeholder
        fs::write(&archive_path, b"package archive placeholder")
            .map_err(|e| format!("Failed to create archive: {}", e))?;
        
        Ok(archive_path)
    }
    
    /// Upload package to registry
    fn upload(&self, archive: &Path) -> Result<(), String> {
        // In production, would make HTTP POST to registry
        // For now, just simulate success
        
        if self.token.is_none() {
            return Err("Authentication token required for publishing".to_string());
        }
        
        Ok(())
    }
}

/// Package registry
pub struct PackageRegistry {
    /// Registry URL
    url: String,
    /// Packages
    packages: Vec<PackageEntry>,
}

/// Package entry in registry
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub published_at: String,
    pub downloads: u64,
}

impl PackageRegistry {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            packages: Vec::new(),
        }
    }
    
    /// Add a package to the registry
    pub fn add_package(&mut self, entry: PackageEntry) {
        self.packages.push(entry);
    }
    
    /// Find packages by name
    pub fn find(&self, name: &str) -> Vec<&PackageEntry> {
        self.packages
            .iter()
            .filter(|p| p.name.contains(name))
            .collect()
    }
    
    /// Get specific version
    pub fn get(&self, name: &str, version: &str) -> Option<&PackageEntry> {
        self.packages
            .iter()
            .find(|p| p.name == name && p.version == version)
    }
    
    /// Get latest version of a package
    pub fn latest(&self, name: &str) -> Option<&PackageEntry> {
        self.packages
            .iter()
            .filter(|p| p.name == name)
            .max_by(|a, b| a.version.cmp(&b.version))
    }
    
    /// List all packages
    pub fn list(&self) -> &[PackageEntry] {
        &self.packages
    }
}

/// Local registry for testing
pub struct LocalRegistry {
    path: PathBuf,
}

impl LocalRegistry {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
    
    /// Initialize local registry
    pub fn init(&self) -> Result<(), String> {
        fs::create_dir_all(&self.path)
            .map_err(|e| format!("Failed to create registry: {}", e))?;
        
        let index_path = self.path.join("index.json");
        if !index_path.exists() {
            fs::write(&index_path, "[]")
                .map_err(|e| format!("Failed to create index: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Add package to local registry
    pub fn add(&self, archive: &Path, config: &ProjectConfig) -> Result<(), String> {
        let dest = self.path.join(format!("{}-{}.tar.gz", config.package.name, config.package.version));
        fs::copy(archive, &dest)
            .map_err(|e| format!("Failed to copy archive: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::PackageConfig;
    
    #[test]
    fn test_publisher_creation() {
        let publisher = Publisher::new("https://registry.hint-lang.org");
        assert_eq!(publisher.registry_url, "https://registry.hint-lang.org");
        assert!(publisher.token.is_none());
    }
    
    #[test]
    fn test_semver_validation() {
        let publisher = Publisher::new("https://example.com");
        
        assert!(publisher.is_valid_semver("1.0.0"));
        assert!(publisher.is_valid_semver("0.1.0"));
        assert!(publisher.is_valid_semver("1.2.3"));
        assert!(!publisher.is_valid_semver("1.0"));
        assert!(!publisher.is_valid_semver("1"));
        assert!(!publisher.is_valid_semver("a.b.c"));
    }
    
    #[test]
    fn test_package_registry() {
        let mut registry = PackageRegistry::new("https://example.com");
        
        registry.add_package(PackageEntry {
            name: "test-lib".to_string(),
            version: "1.0.0".to_string(),
            download_url: "https://example.com/test-lib-1.0.0.tar.gz".to_string(),
            checksum: "abc123".to_string(),
            published_at: "2024-01-01".to_string(),
            downloads: 100,
        });
        
        assert_eq!(registry.list().len(), 1);
        assert!(registry.find("test").len() > 0);
        assert!(registry.latest("test-lib").is_some());
    }
}
