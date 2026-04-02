//! Hint CLI - NPM-like package manager and build tool for Hint Language
//!
//! Provides commands like:
//! - hint run <file>     - Run a Hint program
//! - hint build          - Build the project
//! - hint build:windows  - Build for Windows
//! - hint build:linux    - Build for Linux
//! - hint build:macos    - Build for macOS
//! - hint build:wasm     - Build for WebAssembly
//! - hint dev            - Development mode with hot reload
//! - hint create <name>  - Create a new project
//! - hint init           - Initialize a new project in current directory
//! - hint check          - Type check without building
//! - hint clean          - Clean build artifacts
//! - hint --version      - Show version
//! - hint --help         - Show help

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::time::Duration;
use std::io::{self, Write};

/// Hint CLI - The all-in-one tool for Hint Language development
#[derive(Parser, Debug)]
#[command(name = "hint")]
#[command(author = "Hint Language Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Hint Language CLI - Build, run, and manage Hint projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a Hint program
    Run {
        /// File to run (default: src/main.ht)
        file: Option<PathBuf>,
        
        /// Target platform
        #[arg(long, default_value = "native")]
        target: String,
        
        /// Additional arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Build the project
    Build {
        /// File to build (default: src/main.ht)
        file: Option<PathBuf>,
        
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        out_dir: String,
        
        /// Build target
        #[arg(long, default_value = "native")]
        target: String,
        
        /// Release mode
        #[arg(long)]
        release: bool,
    },
    
    /// Build for Windows (cross-compile)
    #[command(hide = true)]
    "Build:windows" {
        #[arg(short, long, default_value = "dist")]
        out_dir: String,
    },
    
    /// Build for Linux (cross-compile)
    #[command(hide = true)]
    "Build:linux" {
        #[arg(short, long, default_value = "dist")]
        out_dir: String,
    },
    
    /// Build for macOS (cross-compile)
    #[command(hide = true)]
    "Build:macos" {
        #[arg(short, long, default_value = "dist")]
        out_dir: String,
    },
    
    /// Build for WebAssembly
    #[command(hide = true)]
    "Build:wasm" {
        #[arg(short, long, default_value = "dist")]
        out_dir: String,
    },
    
    /// Development mode with hot reload
    Dev {
        /// File to watch (default: src/main.ht)
        file: Option<PathBuf>,
        
        /// Port for dev server (for web projects)
        #[arg(long, default_value = "3000")]
        port: u16,
    },
    
    /// Create a new Hint project
    Create {
        /// Project name
        name: String,
        
        /// Project template
        #[arg(long, default_value = "default")]
        template: String,
    },
    
    /// Initialize a Hint project in current directory
    Init {
        /// Project name (default: current directory name)
        name: Option<String>,
    },
    
    /// Type check without building
    Check {
        /// File to check (default: src/main.ht)
        file: Option<PathBuf>,
    },
    
    /// Clean build artifacts
    Clean {
        /// Directory to clean (default: dist)
        #[arg(default_value = "dist")]
        dir: String,
    },
    
    /// Show project information
    Info,
    
    /// Install dependencies (placeholder for future package manager)
    Install {
        /// Package name
        package: Option<String>,
        
        /// Install as dev dependency
        #[arg(long)]
        dev: bool,
    },
    
    /// Start the language server
    Lsp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HintPackage {
    name: String,
    version: String,
    description: Option<String>,
    main: Option<String>,
    #[serde(rename = "type")]
    project_type: Option<String>,
    scripts: Option<std::collections::HashMap<String, String>>,
    dependencies: Option<std::collections::HashMap<String, String>>,
    dev_dependencies: Option<std::collections::HashMap<String, String>>,
    author: Option<String>,
    license: Option<String>,
}

impl Default for HintPackage {
    fn default() -> Self {
        Self {
            name: String::from("my-hint-app"),
            version: String::from("1.0.0"),
            description: Some(String::from("A Hint Language application")),
            main: Some(String::from("src/main.ht")),
            project_type: Some(String::from("application")),
            scripts: None,
            dependencies: None,
            dev_dependencies: None,
            author: None,
            license: Some(String::from("MIT")),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Run { file, target, args }) => {
            cmd_run(&file, &target, &args);
        }
        Some(Commands::Build { file, out_dir, target, release }) => {
            cmd_build(&file, &out_dir, &target, release);
        }
        Some(Commands::"Build:windows" { out_dir }) => {
            cmd_build(None, &out_dir, "windows64", false);
        }
        Some(Commands::"Build:linux" { out_dir }) => {
            cmd_build(None, &out_dir, "linux64", false);
        }
        Some(Commands::"Build:macos" { out_dir }) => {
            cmd_build(None, &out_dir, "macos64", false);
        }
        Some(Commands::"Build:wasm" { out_dir }) => {
            cmd_build(None, &out_dir, "wasm32", false);
        }
        Some(Commands::Dev { file, port }) => {
            cmd_dev(&file, port);
        }
        Some(Commands::Create { name, template }) => {
            cmd_create(&name, &template);
        }
        Some(Commands::Init { name }) => {
            cmd_init(&name);
        }
        Some(Commands::Check { file }) => {
            cmd_check(&file);
        }
        Some(Commands::Clean { dir }) => {
            cmd_clean(&dir);
        }
        Some(Commands::Info) => {
            cmd_info();
        }
        Some(Commands::Install { package, dev }) => {
            cmd_install(&package, dev);
        }
        Some(Commands::Lsp) => {
            cmd_lsp();
        }
        None => {
            // No command - show help
            println!("Hint Language CLI v{}", env!("CARGO_PKG_VERSION"));
            println!();
            println!("Usage: hint <command> [options]");
            println!();
            println!("Common commands:");
            println!("  hint run [file]        Run a Hint program");
            println!("  hint build             Build the project");
            println!("  hint dev               Development mode");
            println!("  hint create <name>     Create a new project");
            println!("  hint init              Initialize project in current directory");
            println!("  hint check             Type check without building");
            println!("  hint clean             Clean build artifacts");
            println!();
            println!("Build targets:");
            println!("  hint build --target native    Build for current platform");
            println!("  hint build --target wasm32    Build for WebAssembly");
            println!("  hint build --target windows64 Build for Windows");
            println!("  hint build --target linux64   Build for Linux");
            println!("  hint build --target macos64   Build for macOS");
            println!();
            println!("Run 'hint --help' for more information.");
        }
    }
}

fn cmd_run(file: &Option<PathBuf>, target: &str, _args: &[String]) {
    let file_path = file.clone().unwrap_or_else(|| PathBuf::from("src/main.ht"));
    
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        exit(1);
    }
    
    println!("🚀 Running {}...", file_path.display());
    
    // Get the directory where hintc is located
    let hintc_path = find_hintc();
    
    // Compile and run
    let temp_output = format!(".hint_temp_{}", std::process::id());
    
    let compile_result = Command::new(&hintc_path)
        .arg(&file_path)
        .arg("-o")
        .arg(&temp_output)
        .arg("--target")
        .arg(target)
        .output();
    
    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Compilation failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                exit(1);
            }
            
            // Determine the output file name
            let exe_path = if target == "wasm32" || target == "wasm" {
                format!("{}.wasm", temp_output)
            } else if cfg!(windows) {
                format!("{}.exe", temp_output)
            } else {
                temp_output.clone()
            };
            
            // Run the compiled program
            let run_result = Command::new(&exe_path)
                .status();
            
            // Clean up
            let _ = fs::remove_file(&exe_path);
            let _ = fs::remove_file(&temp_output);
            
            match run_result {
                Ok(status) => {
                    if !status.success() {
                        exit(status.code().unwrap_or(1));
                    }
                }
                Err(e) => {
                    eprintln!("Error running program: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to run compiler: {}", e);
            exit(1);
        }
    }
}

fn cmd_build(file: &Option<PathBuf>, out_dir: &str, target: &str, release: bool) {
    let file_path = file.clone().unwrap_or_else(|| {
        // Check for package.json-like config
        if Path::new("hint.json").exists() {
            if let Ok(pkg) = read_package_json() {
                if let Some(main) = pkg.main {
                    return PathBuf::from(main);
                }
            }
        }
        PathBuf::from("src/main.ht")
    });
    
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        exit(1);
    }
    
    // Create output directory
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("Error creating output directory: {}", e);
        exit(1);
    }
    
    let output_name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    
    let output_path = Path::new(out_dir).join(output_name);
    
    println!("🔨 Building {} for {}...", file_path.display(), target);
    
    let hintc_path = find_hintc();
    
    let compile_result = Command::new(&hintc_path)
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--target")
        .arg(target)
        .output();
    
    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Build failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                exit(1);
            }
            
            let exe_name = if target == "wasm32" || target == "wasm" {
                format!("{}.wasm", output_name)
            } else if cfg!(windows) {
                format!("{}.exe", output_name)
            } else {
                output_name.to_string()
            };
            
            println!("✅ Build successful: {}/{}", out_dir, exe_name);
            
            if release {
                println!("📦 Release build completed");
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to run compiler: {}", e);
            exit(1);
        }
    }
}

fn cmd_dev(file: &Option<PathBuf>, port: u16) {
    let file_path = file.clone().unwrap_or_else(|| PathBuf::from("src/main.ht"));
    
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        exit(1);
    }
    
    println!("🔧 Starting development mode...");
    println!("📁 Watching: {}", file_path.display());
    println!("🌐 Dev server port: {}", port);
    println!();
    println!("Press Ctrl+C to stop");
    println!();
    
    let hintc_path = find_hintc();
    let mut last_modified = fs::metadata(&file_path)
        .map(|m| m.modified().unwrap())
        .unwrap_or_else(|_| std::time::SystemTime::now());
    
    loop {
        std::thread::sleep(Duration::from_millis(500));
        
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Ok(modified) = metadata.modified() {
                if modified > last_modified {
                    last_modified = modified;
                    println!("📝 Change detected, rebuilding...");
                    
                    let compile_result = Command::new(&hintc_path)
                        .arg(&file_path)
                        .arg("-o")
                        .arg(".hint_dev_output")
                        .output();
                    
                    match compile_result {
                        Ok(output) => {
                            if output.status.success() {
                                println!("✅ Build successful");
                                // In a full implementation, we'd trigger a reload here
                            } else {
                                println!("❌ Build failed:");
                                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }
        }
    }
}

fn cmd_create(name: &str, template: &str) {
    let project_dir = PathBuf::from(name);
    
    if project_dir.exists() {
        eprintln!("Error: Directory '{}' already exists", name);
        exit(1);
    }
    
    println!("📁 Creating project '{}' with template '{}'...", name, template);
    
    // Create directory structure
    fs::create_dir_all(&project_dir.join("src")).unwrap();
    fs::create_dir_all(&project_dir.join("dist")).unwrap();
    
    // Create hint.json
    let package = HintPackage {
        name: name.to_string(),
        ..Default::default()
    };
    let package_json = serde_json::to_string_pretty(&package).unwrap();
    fs::write(project_dir.join("hint.json"), package_json).unwrap();
    
    // Create main.ht based on template
    let main_content = match template {
        "web" => r#"// Web Application
Say "Hello from Hint Web App!".
Stop the program.
"#,
        "cli" => r#"// CLI Application
Say "Hello from Hint CLI!".
Stop the program.
"#,
        _ => r#"// Hint Application
Say "Hello from Hint!".
Stop the program.
"#,
    };
    
    fs::write(project_dir.join("src").join("main.ht"), main_content).unwrap();
    
    // Create .gitignore
    let gitignore = r#"# Hint build artifacts
dist/*
!dist/.gitkeep
.hint_*
*.wasm
*.exe

# Dependencies
node_modules/
.hint-packages/

# Editor
.vscode/
.idea/
*.swp
*.swo
"#;
    fs::write(project_dir.join(".gitignore"), gitignore).unwrap();
    
    // Create dist/.gitkeep
    fs::write(project_dir.join("dist").join(".gitkeep"), "").unwrap();
    
    // Create README.md
    let readme = format!(r#"# {}

A Hint Language project.

## Getting Started

```bash
# Run the application
hint run

# Build for current platform
hint build

# Build for WebAssembly
hint build --target wasm32

# Development mode
hint dev
```

## Project Structure

```
{}
├── src/
│   └── main.ht    # Main application
├── dist/          # Build output
├── hint.json      # Project configuration
└── README.md
```
"#, name, name);
    fs::write(project_dir.join("README.md"), readme).unwrap();
    
    println!("✅ Project created successfully!");
    println!();
    println!("To get started:");
    println!("  cd {}", name);
    println!("  hint run");
}

fn cmd_init(name: &Option<String>) {
    let project_name = name.clone().unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-hint-app".to_string())
    });
    
    if Path::new("hint.json").exists() {
        eprintln!("Error: hint.json already exists in this directory");
        exit(1);
    }
    
    println!("📁 Initializing Hint project '{}'...", project_name);
    
    // Create directory structure
    let _ = fs::create_dir_all("src");
    let _ = fs::create_dir_all("dist");
    
    // Create hint.json
    let package = HintPackage {
        name: project_name,
        ..Default::default()
    };
    let package_json = serde_json::to_string_pretty(&package).unwrap();
    fs::write("hint.json", package_json).unwrap();
    
    // Create main.ht if it doesn't exist
    if !Path::new("src/main.ht").exists() {
        let main_content = r#"// Hint Application
Say "Hello from Hint!".
Stop the program.
"#;
        fs::write("src/main.ht", main_content).unwrap();
    }
    
    // Create .gitignore if it doesn't exist
    if !Path::new(".gitignore").exists() {
        let gitignore = r#"# Hint build artifacts
dist/*
!dist/.gitkeep
.hint_*
*.wasm
*.exe

# Dependencies
node_modules/
.hint-packages/

# Editor
.vscode/
.idea/
*.swp
*.swo
"#;
        fs::write(".gitignore", gitignore).unwrap();
    }
    
    // Create dist/.gitkeep
    let _ = fs::write("dist/.gitkeep", "");
    
    println!("✅ Project initialized!");
    println!();
    println!("To get started:");
    println!("  hint run");
}

fn cmd_check(file: &Option<PathBuf>) {
    let file_path = file.clone().unwrap_or_else(|| PathBuf::from("src/main.ht"));
    
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        exit(1);
    }
    
    println!("🔍 Checking {}...", file_path.display());
    
    let hintc_path = find_hintc();
    
    // Use --ast flag to parse and check without generating code
    let check_result = Command::new(&hintc_path)
        .arg(&file_path)
        .arg("--ast")
        .output();
    
    match check_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ No errors found");
            } else {
                eprintln!("Check failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

fn cmd_clean(dir: &str) {
    let clean_dir = Path::new(dir);
    
    if !clean_dir.exists() {
        println!("Nothing to clean - {} doesn't exist", dir);
        return;
    }
    
    println!("🧹 Cleaning {}...", dir);
    
    match fs::remove_dir_all(clean_dir) {
        Ok(_) => {
            // Recreate the directory
            let _ = fs::create_dir_all(clean_dir);
            println!("✅ Cleaned successfully");
        }
        Err(e) => {
            eprintln!("Error cleaning: {}", e);
            exit(1);
        }
    }
}

fn cmd_info() {
    println!("📦 Hint Language Project Info");
    println!();
    
    if let Ok(pkg) = read_package_json() {
        println!("Name:        {}", pkg.name);
        println!("Version:     {}", pkg.version);
        if let Some(desc) = pkg.description {
            println!("Description: {}", desc);
        }
        if let Some(main) = pkg.main {
            println!("Main:        {}", main);
        }
        if let Some(author) = pkg.author {
            println!("Author:      {}", author);
        }
        if let Some(license) = pkg.license {
            println!("License:     {}", license);
        }
    } else {
        println!("No hint.json found - not a Hint project");
        println!();
        println!("Run 'hint init' to initialize a project");
    }
    
    println!();
    println!("Hint CLI Version: {}", env!("CARGO_PKG_VERSION"));
}

fn cmd_install(_package: &Option<String>, _dev: bool) {
    // Placeholder for future package manager
    println!("⚠️  Package manager is not yet implemented");
    println!();
    println!("The package manager is coming in a future release.");
    println!("For now, you can:");
    println!("  - Use hint.json to track dependencies");
    println!("  - Manually download and place packages in a 'packages' folder");
}

fn cmd_lsp() {
    println!("🔧 Starting Hint Language Server...");
    
    let hintc_path = find_hintc();
    
    let result = Command::new(&hintc_path)
        .arg("--lsp")
        .status();
    
    match result {
        Ok(status) => {
            if !status.success() {
                exit(status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to start LSP: {}", e);
            exit(1);
        }
    }
}

fn find_hintc() -> PathBuf {
    // Try to find hintc in the same directory as this binary
    if let Ok(exe) = std::env::current_exe() {
        let hintc = exe.parent()
            .map(|p| p.join("hintc"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| {
                // Fallback: try PATH
                PathBuf::from("hintc")
            });
        
        // On Windows, check for .exe
        if cfg!(windows) && !hintc.exists() {
            let hintc_exe = exe.parent()
                .map(|p| p.join("hintc.exe"))
                .filter(|p| p.exists())
                .unwrap_or_else(|| PathBuf::from("hintc.exe"));
            return hintc_exe;
        }
        
        return hintc;
    }
    
    PathBuf::from("hintc")
}

fn read_package_json() -> Result<HintPackage, String> {
    let content = fs::read_to_string("hint.json")
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&content)
        .map_err(|e| e.to_string())
}
