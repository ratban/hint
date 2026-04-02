//! Target abstraction for cross-platform compilation.
//! 
//! Supports native targets (Windows, macOS, Linux) and WebAssembly.

use std::fmt;
use std::str::FromStr;
use target_lexicon::{Architecture, OperatingSystem, Environment, Vendor};

/// Compilation target specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilationTarget {
    /// Native target (auto-detect host)
    Native,
    /// Windows x86_64
    WindowsX64,
    /// Windows ARM64
    WindowsArm64,
    /// Linux x86_64
    LinuxX64,
    /// Linux ARM64
    LinuxArm64,
    /// macOS x86_64
    MacosX64,
    /// macOS ARM64 (Apple Silicon)
    MacosArm64,
    /// WebAssembly 32-bit
    Wasm32,
    /// WebAssembly 64-bit (experimental)
    Wasm64,
}

impl CompilationTarget {
    /// Cached host triple to avoid repeated allocations
    fn host_triple() -> &'static str {
        use std::sync::OnceLock;
        static HOST_TRIPLE: OnceLock<String> = OnceLock::new();
        HOST_TRIPLE.get_or_init(|| target_lexicon::Triple::host().to_string()).as_str()
    }

    /// Get the target triple string
    pub fn triple(&self) -> &'static str {
        match self {
            CompilationTarget::Native => Self::host_triple(),
            CompilationTarget::WindowsX64 => "x86_64-pc-windows-msvc",
            CompilationTarget::WindowsArm64 => "aarch64-pc-windows-msvc",
            CompilationTarget::LinuxX64 => "x86_64-unknown-linux-gnu",
            CompilationTarget::LinuxArm64 => "aarch64-unknown-linux-gnu",
            CompilationTarget::MacosX64 => "x86_64-apple-darwin",
            CompilationTarget::MacosArm64 => "aarch64-apple-darwin",
            CompilationTarget::Wasm32 => "wasm32-unknown-unknown",
            CompilationTarget::Wasm64 => "wasm64-unknown-unknown",
        }
    }
    
    /// Check if this is a WebAssembly target
    pub fn is_wasm(&self) -> bool {
        matches!(self, CompilationTarget::Wasm32 | CompilationTarget::Wasm64)
    }
    
    /// Check if this is a native target
    pub fn is_native(&self) -> bool {
        !self.is_wasm()
    }
    
    /// Get the file extension for the output
    pub fn output_extension(&self) -> &'static str {
        match self {
            CompilationTarget::Wasm32 | CompilationTarget::Wasm64 => "wasm",
            CompilationTarget::WindowsX64 | CompilationTarget::WindowsArm64 => "exe",
            _ => "", // Unix executables have no extension
        }
    }
    
    /// Get the object file extension
    pub fn object_extension(&self) -> &'static str {
        match self {
            CompilationTarget::WindowsX64 | CompilationTarget::WindowsArm64 => "obj",
            _ => "o",
        }
    }
    
    /// Parse from a target triple string
    pub fn from_triple(triple: &str) -> Result<Self, String> {
        match triple {
            "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => Ok(CompilationTarget::WindowsX64),
            "aarch64-pc-windows-msvc" => Ok(CompilationTarget::WindowsArm64),
            "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => Ok(CompilationTarget::LinuxX64),
            "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => Ok(CompilationTarget::LinuxArm64),
            "x86_64-apple-darwin" => Ok(CompilationTarget::MacosX64),
            "aarch64-apple-darwin" => Ok(CompilationTarget::MacosArm64),
            "wasm32-unknown-unknown" => Ok(CompilationTarget::Wasm32),
            "wasm64-unknown-unknown" => Ok(CompilationTarget::Wasm64),
            _ => Err(format!("Unknown target triple: {}", triple)),
        }
    }
}

impl fmt::Display for CompilationTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.triple())
    }
}

impl Default for CompilationTarget {
    fn default() -> Self {
        CompilationTarget::Native
    }
}

/// Detailed target information
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub architecture: Architecture,
    pub vendor: Vendor,
    pub operating_system: OperatingSystem,
    pub environment: Environment,
}

impl TargetInfo {
    /// Get target info for the host system
    pub fn host() -> Self {
        let triple = target_lexicon::Triple::host();
        Self {
            architecture: triple.architecture,
            vendor: triple.vendor,
            operating_system: triple.operating_system,
            environment: triple.environment,
        }
    }
    
    /// Get target info for a specific target
    pub fn from_target(target: &CompilationTarget) -> Result<Self, String> {
        let triple = target_lexicon::Triple::from_str(target.triple())
            .unwrap_or_else(|_| target_lexicon::Triple::host());
        
        Ok(Self {
            architecture: triple.architecture,
            vendor: triple.vendor,
            operating_system: triple.operating_system,
            environment: triple.environment,
        })
    }
    
    /// Check if the target is Windows
    pub fn is_windows(&self) -> bool {
        matches!(self.operating_system, OperatingSystem::Windows)
    }
    
    /// Check if the target is macOS
    pub fn is_macos(&self) -> bool {
        matches!(self.operating_system, OperatingSystem::Darwin)
    }
    
    /// Check if the target is Linux
    pub fn is_linux(&self) -> bool {
        matches!(self.operating_system, OperatingSystem::Linux)
    }
    
    /// Check if the target uses ELF format
    pub fn is_elf(&self) -> bool {
        self.is_linux()
    }
    
    /// Check if the target uses PE/COFF format
    pub fn is_pe(&self) -> bool {
        self.is_windows()
    }
    
    /// Check if the target uses Mach-O format
    pub fn is_macho(&self) -> bool {
        self.is_macos()
    }
}

impl fmt::Display for TargetInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}-{}", 
            self.architecture,
            self.vendor,
            self.operating_system,
            self.environment
        )
    }
}
