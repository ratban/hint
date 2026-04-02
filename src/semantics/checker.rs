//! Semantic analyzer and type checker.

use crate::diagnostics::{Diagnostic, DiagnosticsEngine, Span};
use crate::parser::{Program as AstProgram, AstNode};
use crate::semantics::{
    TypedProgram, TypedStatement,
    SymbolTable, Symbol, SymbolType,
    HintType, IntSize,
};
use crate::stdlib::get_stdlib;

/// Semantic analyzer for Hint programs
pub struct SemanticAnalyzer<'a> {
    source: &'a str,
    symbol_table: SymbolTable,
    diagnostics: DiagnosticsEngine,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut analyzer = Self {
            source,
            symbol_table: SymbolTable::new(),
            diagnostics: DiagnosticsEngine::new(),
        };
        analyzer.symbol_table.init_builtins();
        analyzer.init_stdlib();
        analyzer
    }

    /// Initialize standard library functions in symbol table
    fn init_stdlib(&mut self) {
        let stdlib = get_stdlib();
        // Register stdlib functions as builtins
        // For now, we just acknowledge they exist
        // A full implementation would add them to the symbol table
        let _ = stdlib; // suppress unused warning until fully integrated
    }
    
    /// Analyze a program and return a typed program
    pub fn analyze(mut self, program: &AstProgram) -> Result<TypedProgram, DiagnosticsEngine> {
        let mut statements = Vec::new();
        
        for node in &program.statements {
            match self.analyze_statement(node) {
                Ok(stmt) => statements.push(stmt),
                Err(_) => {
                    // Continue analyzing other statements even if one fails
                    // Add a placeholder statement to maintain structure
                    statements.push(TypedStatement::Halt { span: Span::default() });
                }
            }
        }
        
        if self.diagnostics.has_errors() {
            return Err(self.diagnostics);
        }
        
        Ok(TypedProgram {
            statements,
            symbol_table: self.symbol_table,
        })
    }
    
    fn analyze_statement(&mut self, node: &AstNode) -> Result<TypedStatement, ()> {
        match node {
            AstNode::Speak(text) => {
                let span = self.get_node_span(node);
                Ok(TypedStatement::Speak {
                    text: text.clone(),
                    span,
                })
            }
            
            AstNode::Remember { name, value } => {
                let span = self.get_node_span(node);
                
                // Check if variable already exists
                if self.symbol_table.contains(name) {
                    self.diagnostics.emit(
                        Diagnostic::error()
                            .with_message(format!("variable `{}` is already defined", name))
                            .with_span(span.start, span.end)
                            .with_source(self.source.to_string())
                            .with_help("use a different variable name")
                    );
                    return Err(());
                }

                // Insert variable into symbol table
                let symbol = Symbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable(HintType::Int(IntSize::I64)),
                    span,
                    is_mutable: true,
                };

                if let Err(_) = self.symbol_table.insert(symbol) {
                    self.diagnostics.emit(
                        Diagnostic::error()
                            .with_message(format!("failed to insert variable `{}`", name))
                            .with_span(span.start, span.end)
                            .with_source(self.source.to_string())
                    );
                    return Err(());
                }

                Ok(TypedStatement::Remember {
                    name: name.clone(),
                    value: *value,
                    span,
                })
            }
            
            AstNode::Halt => {
                let span = self.get_node_span(node);
                Ok(TypedStatement::Halt { span })
            }

            AstNode::RememberList { name, values } => {
                let span = self.get_node_span(node);

                // Check if variable already exists
                if self.symbol_table.contains(name) {
                    self.diagnostics.emit(
                        Diagnostic::error()
                            .with_message(format!("variable `{}` is already defined", name))
                            .with_span(span.start, span.end)
                            .with_source(self.source.to_string())
                            .with_help("use a different variable name")
                    );
                    return Err(());
                }

                // Insert variable into symbol table
                let symbol = Symbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable(HintType::Int(IntSize::I64)),
                    span,
                    is_mutable: true,
                };

                if let Err(_) = self.symbol_table.insert(symbol) {
                    self.diagnostics.emit(
                        Diagnostic::error()
                            .with_message(format!("failed to insert variable `{}`", name))
                            .with_span(span.start, span.end)
                            .with_source(self.source.to_string())
                    );
                    return Err(());
                }

                Ok(TypedStatement::RememberList {
                    name: name.clone(),
                    values: values.clone(),
                    span,
                })
            }
        }
    }
    
    fn get_node_span(&self, node: &AstNode) -> Span {
        // Track approximate spans based on node type
        // In a full implementation, we'd track exact spans in the AST
        match node {
            AstNode::Speak(text) => {
                // Approximate: find "say" or "print" keyword position
                Span::new(0, text.len() + 10)
            }
            AstNode::Remember { name, value: _ } => {
                // Approximate span for remember statement
                Span::new(0, name.len() + 20)
            }
            AstNode::RememberList { name, values } => {
                // Approximate span for list statement
                Span::new(0, name.len() + values.len() * 5 + 25)
            }
            AstNode::Halt => {
                Span::new(0, 20)
            }
        }
    }
}

/// Analyze a program (convenience function)
pub fn analyze_program(program: &AstProgram, source: &str) -> Result<TypedProgram, DiagnosticsEngine> {
    let analyzer = SemanticAnalyzer::new(source);
    analyzer.analyze(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    
    #[test]
    fn test_analyze_simple_program() {
        let source = r#"
Say "Hello".
Keep the number 42 in mind as the answer.
Stop the program.
"#;
        let ast = parse(source).unwrap();
        let typed = analyze_program(&ast, source);
        assert!(typed.is_ok());
    }
    
    #[test]
    fn test_analyze_duplicate_variable() {
        let source = r#"
Keep the number 1 in mind as the x.
Keep the number 2 in mind as the x.
Stop the program.
"#;
        let ast = parse(source).unwrap();
        let result = analyze_program(&ast, source);
        assert!(result.is_err());
    }
}
