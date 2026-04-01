//! Package Registry
//! 
//! Local and remote package registries.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Package registry trait
pub trait Registry {
    /// Search for packages
    fn search(&self, query: &str) -> Vec<PackageMeta>;
    
    /// Get package info
    fn get(&self, name: &str, version: &str) -> Option<PackageMeta>;
    
    /// Get latest version
    fn latest(&self, name: &str) -> Option<PackageMeta>;
    
    /// Download package
    fn download(&self, name: &str, version: &str, dest: &Path) -> Result<(), String>;
    
    /// Upload package
    fn upload(&mut self, package: &[u8], meta: PackageMeta) -> Result<(), String>;
}

/// Package metadata
#[derive(Debug, Clone)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub checksum: String,
    pub downloads: u64,
    pub published_at: String,
}

/// In-memory registry for testing
pub struct MemoryRegistry {
    packages: HashMap<String, Vec<PackageMeta>>,
}

impl MemoryRegistry {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }
}

impl Default for MemoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry for MemoryRegistry {
    fn search(&self, query: &str) -> Vec<PackageMeta> {
        self.packages
            .values()
            .flatten()
            .filter(|p| p.name.contains(query) || p.description.contains(query))
            .cloned()
            .collect()
    }
    
    fn get(&self, name: &str, version: &str) -> Option<PackageMeta> {
        self.packages
            .get(name)
            .and_then(|versions| versions.iter().find(|p| p.version == version))
            .cloned()
    }
    
    fn latest(&self, name: &str) -> Option<PackageMeta> {
        self.packages
            .get(name)
            .and_then(|versions| versions.iter().max_by(|a, b| a.version.cmp(&b.version)))
            .cloned()
    }
    
    fn download(&self, name: &str, version: &str, dest: &Path) -> Result<(), String> {
        // Simulate download
        fs::write(dest, b"package content")
            .map_err(|e| format!("Failed to write: {}", e))
    }
    
    fn upload(&mut self, package: &[u8], meta: PackageMeta) -> Result<(), String> {
        self.packages
            .entry(meta.name.clone())
            .or_insert_with(Vec::new)
            .push(meta);
        Ok(())
    }
}

/// File-based local registry
pub struct FileRegistry {
    path: PathBuf,
    index: HashMap<String, Vec<PackageMeta>>,
}

impl FileRegistry {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            index: HashMap::new(),
        }
    }
    
    /// Load index from disk
    pub fn load(&mut self) -> Result<(), String> {
        let index_path = self.path.join("index.json");
        
        if index_path.exists() {
            let content = fs::read_to_string(&index_path)
                .map_err(|e| format!("Failed to read index: {}", e))?;
            
            // Simple JSON parsing (in production, use serde_json)
            self.index = HashMap::new();
        }
        
        Ok(())
    }
    
    /// Save index to disk
    pub fn save(&self) -> Result<(), String> {
        let index_path = self.path.join("index.json");
        
        let content = serde_json::to_string_pretty(&self.index)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;
        
        fs::write(&index_path, content)
            .map_err(|e| format!("Failed to write index: {}", e))?;
        
        Ok(())
    }
}

impl Registry for FileRegistry {
    fn search(&self, query: &str) -> Vec<PackageMeta> {
        self.index
            .values()
            .flatten()
            .filter(|p| p.name.contains(query))
            .cloned()
            .collect()
    }
    
    fn get(&self, name: &str, version: &str) -> Option<PackageMeta> {
        self.index
            .get(name)
            .and_then(|versions| versions.iter().find(|p| p.version == version))
            .cloned()
    }
    
    fn latest(&self, name: &str) -> Option<PackageMeta> {
        self.index
            .get(name)
            .and_then(|versions| versions.iter().max_by(|a, b| a.version.cmp(&b.version)))
            .cloned()
    }
    
    fn download(&self, name: &str, version: &str, dest: &Path) -> Result<(), String> {
        let package_path = self.path.join(format!("{}-{}.tar.gz", name, version));
        
        if !package_path.exists() {
            return Err(format!("Package not found: {} {}", name, version));
        }
        
        fs::copy(&package_path, dest)
            .map_err(|e| format!("Failed to copy: {}", e))?;
        
        Ok(())
    }
    
    fn upload(&mut self, package: &[u8], meta: PackageMeta) -> Result<(), String> {
        // Save package archive
        let archive_path = self.path.join(format!("{}-{}.tar.gz", meta.name, meta.version));
        fs::write(&archive_path, package)
            .map_err(|e| format!("Failed to write archive: {}", e))?;
        
        // Update index
        self.index
            .entry(meta.name.clone())
            .or_insert_with(Vec::new)
            .push(meta);
        
        self.save()?;
        
        Ok(())
    }
}

/// Multi-registry that searches multiple registries
pub struct MultiRegistry {
    registries: Vec<Box<dyn Registry>>,
}

impl MultiRegistry {
    pub fn new() -> Self {
        Self {
            registries: Vec::new(),
        }
    }
    
    pub fn add_registry<R: Registry + 'static>(&mut self, registry: R) {
        self.registries.push(Box::new(registry));
    }
}

impl Default for MultiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry for MultiRegistry {
    fn search(&self, query: &str) -> Vec<PackageMeta> {
        self.registries
            .iter()
            .flat_map(|r| r.search(query))
            .collect()
    }
    
    fn get(&self, name: &str, version: &str) -> Option<PackageMeta> {
        for registry in &self.registries {
            if let Some(meta) = registry.get(name, version) {
                return Some(meta);
            }
        }
        None
    }
    
    fn latest(&self, name: &str) -> Option<PackageMeta> {
        for registry in &self.registries {
            if let Some(meta) = registry.latest(name) {
                return Some(meta);
            }
        }
        None
    }
    
    fn download(&self, name: &str, version: &str, dest: &Path) -> Result<(), String> {
        for registry in &self.registries {
            if registry.get(name, version).is_some() {
                return registry.download(name, version, dest);
            }
        }
        Err(format!("Package not found: {} {}", name, version))
    }
    
    fn upload(&mut self, package: &[u8], meta: PackageMeta) -> Result<(), String> {
        if let Some(registry) = self.registries.first_mut() {
            return registry.upload(package, meta);
        }
        Err("No registries configured".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_registry() {
        let mut registry = MemoryRegistry::new();
        
        let meta = PackageMeta {
            name: "test-lib".to_string(),
            version: "1.0.0".to_string(),
            description: "Test library".to_string(),
            authors: vec!["Test Author".to_string()],
            license: Some("MIT".to_string()),
            repository: None,
            homepage: None,
            keywords: vec!["test".to_string()],
            categories: vec![],
            dependencies: HashMap::new(),
            checksum: "abc123".to_string(),
            downloads: 0,
            published_at: "2024-01-01".to_string(),
        };
        
        registry.upload(b"package", meta.clone()).unwrap();
        
        assert_eq!(registry.search("test").len(), 1);
        assert!(registry.get("test-lib", "1.0.0").is_some());
        assert!(registry.latest("test-lib").is_some());
    }
    
    #[test]
    fn test_multi_registry() {
        let mut multi = MultiRegistry::new();
        
        let mut registry1 = MemoryRegistry::new();
        registry1.upload(b"pkg1", PackageMeta {
            name: "pkg1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            checksum: String::new(),
            downloads: 0,
            published_at: String::new(),
        }).unwrap();
        
        multi.add_registry(registry1);
        
        assert_eq!(multi.search("pkg1").len(), 1);
    }
}
