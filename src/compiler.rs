//! Compiler orchestration module.
//! 
//! Coordinates all phases of compilation from source to executable.

use crate::lexer::tokenize;
use crate::parser::parse;
use crate::semantics::analyze;
use crate::ir::lower_to_hir;
use crate::codegen::{create_generator, CompilationTarget};
use crate::diagnostics::{Diagnostic, DiagnosticsEngine};
use std::fs;

/// Main compiler struct
pub struct Compiler {
    target: CompilationTarget,
    verbose: bool,
    keep_intermediates: bool,
}

impl Compiler {
    pub fn new(target: CompilationTarget) -> Self {
        Self {
            target,
            verbose: false,
            keep_intermediates: false,
        }
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    pub fn with_keep_intermediates(mut self, keep: bool) -> Self {
        self.keep_intermediates = keep;
        self
    }
    
    /// Compile source code string to a file
    pub fn compile_source_to_file(&self, source: &str, output: &str) -> Result<(), String> {
        // Compile source
        let bytes = self.compile_source(source)?;

        // Write output
        let output_path = if self.target.is_wasm() {
            format!("{}.wasm", output)
        } else if self.target.is_native() && cfg!(windows) {
            format!("{}.exe", output)
        } else {
            output.to_string()
        };

        fs::write(&output_path, &bytes)
            .map_err(|e| format!("Failed to write '{}': {}", output_path, e))?;

        if self.verbose {
            eprintln!("Successfully compiled -> {}", output_path);
        }

        Ok(())
    }

    /// Compile a source file to the target format
    pub fn compile_file(&self, input: &str, output: &str) -> Result<(), String> {
        // Read source file
        let source = fs::read_to_string(input)
            .map_err(|e| format!("Failed to read '{}': {}", input, e))?;

        // Compile source
        self.compile_source_to_file(&source, output)
    }
    
    /// Compile source code string to bytes
    pub fn compile_source(&self, source: &str) -> Result<Vec<u8>, String> {
        let mut diagnostics = DiagnosticsEngine::new();
        
        // Phase 1: Lexical Analysis
        if self.verbose {
            eprintln!("[hintc] Phase 1: Lexical analysis");
        }
        let tokens = tokenize(source).map_err(|e| {
            diagnostics.emit(e.to_diagnostic(""));
            diagnostics.format("source")
        })?;
        
        if self.verbose {
            eprintln!("[hintc]   Found {} tokens", tokens.len());
        }
        
        // Phase 2: Parsing
        if self.verbose {
            eprintln!("[hintc] Phase 2: Parsing");
        }
        let ast = parse(source).map_err(|e| {
            diagnostics.emit(e.to_diagnostic(source));
            diagnostics.format("source")
        })?;
        
        if self.verbose {
            eprintln!("[hintc]   Parsed {} statements", ast.statements.len());
        }
        
        // Phase 3: Semantic Analysis
        if self.verbose {
            eprintln!("[hintc] Phase 3: Semantic analysis");
        }
        let typed = analyze(&ast, source).map_err(|diags| {
            diags.format("source")
        })?;
        
        if self.verbose {
            eprintln!("[hintc]   Type checking passed");
        }
        
        // Phase 4: Lower to IR
        if self.verbose {
            eprintln!("[hintc] Phase 4: Lowering to IR");
        }
        let hir = lower_to_hir(&typed);
        
        if self.verbose {
            eprintln!("[hintc]   Generated {} globals", hir.globals.len());
        }
        
        // Phase 5: Code Generation
        if self.verbose {
            eprintln!("[hintc] Phase 5: Code generation ({})", self.target);
        }
        let mut generator = create_generator(&self.target)?;
        let bytes = generator.generate(&hir)?;
        
        if self.verbose {
            eprintln!("[hintc]   Generated {} bytes", bytes.len());
        }
        
        Ok(bytes)
    }
    
    /// Get the compilation target
    pub fn target(&self) -> &CompilationTarget {
        &self.target
    }
}

/// Compilation result
#[derive(Debug)]
pub struct CompilationResult {
    pub bytes: Vec<u8>,
    pub warnings: Vec<Diagnostic>,
}

/// Compile with full error reporting
pub fn compile(
    source: &str,
    target: &CompilationTarget,
) -> Result<CompilationResult, String> {
    let compiler = Compiler::new(target.clone()).with_verbose(false);
    
    let bytes = compiler.compile_source(source)?;
    
    Ok(CompilationResult {
        bytes,
        warnings: vec![],
    })
}
