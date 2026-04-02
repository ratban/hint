//! Hint Compiler (hintc) - Zero-Dependency Native and WebAssembly Compiler
//!
//! A production-grade compiler for the Hint programming language.
//! Compiles conversational English to native executables and WebAssembly.
//!
//! # Usage
//!
//! ```text
//! hintc program.ht                    # Compile to native executable
//! hintc program.ht --target wasm32    # Compile to WebAssembly (wasm32)
//! hintc program.ht --target linux64   # Compile for Linux x86_64
//! hintc program.ht --target windows64 # Compile for Windows x86_64
//! hintc program.ht --target macos64   # Compile for macOS x86_64
//! hintc program.ht -o output          # Custom output name
//! hintc --ast program.ht              # Print AST
//! hintc --ir program.ht               # Print IR
//! hintc --tokens program.ht           # Print tokens
//! hintc --lsp                         # Start language server
//! hintc --repl                        # Start REPL (interactive mode)
//! ```
//!
//! # Targets
//!
//! Available compilation targets:
//! - `native` - Auto-detect host platform (default)
//! - `windows64` - Windows x86_64
//! - `linux64` - Linux x86_64
//! - `macos64` - macOS x86_64
//! - `wasm32` - WebAssembly 32-bit

pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod ir;
pub mod codegen;
pub mod target;
pub mod stdlib;
pub mod diagnostics;
pub mod compiler;
pub mod lsp;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process::exit;

use crate::compiler::Compiler;
use crate::target::CompilationTarget;

/// Hint Compiler - Zero-dependency native and WebAssembly compiler
#[derive(Parser, Debug)]
#[command(name = "hintc")]
#[command(author = "Hint Language Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Compiles Hint source files to native executables or WebAssembly", long_about = None)]
struct Cli {
    /// Input source file (.ht)
    #[arg(required_unless_present_any = ["lsp", "repl"])]
    input: Option<PathBuf>,

    /// Output file name (without extension)
    #[arg(short, long)]
    output: Option<String>,

    /// Compilation target
    #[arg(long, value_enum, default_value = "native")]
    target: TargetArg,

    /// Print tokens and exit
    #[arg(long)]
    tokens: bool,

    /// Print AST and exit
    #[arg(long)]
    ast: bool,

    /// Print IR and exit
    #[arg(long)]
    ir: bool,

    /// Keep intermediate files
    #[arg(long)]
    keep: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Start language server
    #[arg(long)]
    lsp: bool,

    /// Start REPL (interactive mode)
    #[arg(long)]
    repl: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TargetArg {
    /// Native executable (auto-detect platform)
    Native,
    /// Windows x86_64
    WindowsX64,
    /// Linux x86_64
    LinuxX64,
    /// macOS x86_64
    MacosX64,
    /// WebAssembly 32-bit
    Wasm32,
}

impl From<TargetArg> for CompilationTarget {
    fn from(arg: TargetArg) -> Self {
        match arg {
            TargetArg::Native => CompilationTarget::Native,
            TargetArg::WindowsX64 => CompilationTarget::WindowsX64,
            TargetArg::LinuxX64 => CompilationTarget::LinuxX64,
            TargetArg::MacosX64 => CompilationTarget::MacosX64,
            TargetArg::Wasm32 => CompilationTarget::Wasm32,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle LSP mode
    if cli.lsp {
        run_lsp();
        return;
    }

    // Handle REPL mode
    if cli.repl {
        run_repl();
        return;
    }

    // Get input file
    let input = match &cli.input {
        Some(p) => p,
        None => {
            eprintln!("Error: No input file specified");
            exit(1);
        }
    };

    // Validate input file exists
    if !input.exists() {
        eprintln!("Error: Input file not found: {}", input.display());
        exit(1);
    }

    // Read source file
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", input.display(), e);
            exit(1);
        }
    };

    // Handle --tokens
    if cli.tokens {
        print_tokens(&source);
        return;
    }

    // Handle --ast
    if cli.ast {
        print_ast(&source);
        return;
    }

    // Handle --ir
    if cli.ir {
        print_ir(&source);
        return;
    }

    // Determine output name
    let output = cli.output.unwrap_or_else(|| {
        input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string()
    });

    // Create compiler
    let target: CompilationTarget = cli.target.into();
    let compiler = Compiler::new(target)
        .with_verbose(cli.verbose)
        .with_keep_intermediates(cli.keep);

    // Compile
    match compiler.compile_source_to_file(&source, &output) {
        Ok(()) => {
            if cli.verbose {
                eprintln!("Compilation successful!");
            }
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            exit(1);
        }
    }
}

fn print_tokens(source: &str) {
    use crate::lexer::tokenize;
    
    match tokenize(source) {
        Ok(tokens) => {
            println!("=== Tokens ===");
            for (i, token) in tokens.iter().enumerate() {
                println!("{:4}: {}", i, token);
            }
        }
        Err(e) => {
            eprintln!("Lexical error: {}", e);
            exit(1);
        }
    }
}

fn print_ast(source: &str) {
    use crate::parser::parse;
    
    match parse(source) {
        Ok(program) => {
            println!("=== AST ===");
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("{:4}: {}", i, stmt);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            exit(1);
        }
    }
}

fn print_ir(source: &str) {
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::semantics::analyze_program;
    use crate::ir::lower_to_hir;
    
    let _tokens = match tokenize(source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexical error: {}", e);
            exit(1);
        }
    };
    
    let ast = match parse(source) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            exit(1);
        }
    };
    
    let typed = match analyze_program(&ast, source) {
        Ok(t) => t,
        Err(diags) => {
            eprintln!("Semantic errors:");
            diags.print("source");
            exit(1);
        }
    };
    
    let hir = lower_to_hir(&typed);
    
    println!("=== IR ===");
    println!("Globals: {}", hir.globals.len());
    for global in &hir.globals {
        println!("  {} : {:?}", global.name, global.var_type);
    }
    
    if let Some(entry) = &hir.entry_point {
        println!("\nEntry block ({} instructions):", entry.instructions.len());
        for (i, instr) in entry.instructions.iter().enumerate() {
            println!("  {:4}: {:?}", i, instr);
        }
    }
}

fn run_lsp() {
    use crate::lsp::run_language_server;
    
    eprintln!("Hint Language Server v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("Listening on stdin for JSON-RPC messages...");
    
    if let Err(e) = run_language_server() {
        eprintln!("LSP error: {}", e);
        exit(1);
    }
}

fn run_repl() {
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::semantics::analyze_program;
    
    println!("Hint REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type '.quit' to exit, '.help' for help.\n");
    
    loop {
        print!("hint> ");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
        
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == ".quit" {
            break;
        }
        
        if input == ".help" {
            println!("Commands:");
            println!("  .quit    - Exit REPL");
            println!("  .help    - Show this help");
            println!("  .tokens  - Show tokens for last input");
            println!("  .ast     - Show AST for last input");
            println!();
            continue;
        }
        
        // Try to parse and analyze
        match tokenize(input) {
            Ok(_tokens) => {
                match parse(input) {
                    Ok(ast) => {
                        match analyze_program(&ast, input) {
                            Ok(_) => println!("✓ Valid Hint code"),
                            Err(diags) => {
                                println!("Semantic errors:");
                                diags.print("input");
                            }
                        }
                    }
                    Err(e) => println!("Parse error: {}", e),
                }
            }
            Err(e) => println!("Lexical error: {}", e),
        }
    }
    
    println!("\nGoodbye!");
}
