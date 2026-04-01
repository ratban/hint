//! Parser for the Hint programming language.
//!
//! Uses pattern matching on token sequences to recognize the intent
//! of conversational English statements and produce an AST.

use crate::lexer::{tokenize, LexError, Token};
use crate::diagnostics::Diagnostic;
use std::fmt;

/// Abstract Syntax Tree nodes for Hint programs.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Output a string: `Say "text".`
    Speak(String),
    /// Store a number in memory: `Keep the number N in mind as the NAME.`
    Remember { name: String, value: i32 },
    /// Store a list in memory: `Keep the list [1, 2, 3] in mind as the items.`
    RememberList { name: String, values: Vec<i32> },
    /// Terminate execution: `Stop the program.`
    Halt,
}

impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Speak(s) => write!(f, "Speak({})", s),
            AstNode::Remember { name, value } => {
                write!(f, "Remember({} = {})", name, value)
            }
            AstNode::RememberList { name, values } => {
                write!(f, "RememberList({} = {:?})", name, values)
            }
            AstNode::Halt => write!(f, "Halt"),
        }
    }
}

/// A complete Hint program is a sequence of AST nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub statements: Vec<AstNode>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

/// Error type for parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError {
            message: err.message,
            position: err.position,
        }
    }
}

impl ParseError {
    pub fn to_diagnostic(&self, source: &str) -> crate::diagnostics::Diagnostic {
        Diagnostic::error()
            .with_message(&self.message)
            .with_span(self.position, self.position + 1)
            .with_source(source.to_string())
    }
}

/// The parser for Hint source code.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser from a vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    /// Parse the entire token stream into a Program AST.
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Check if we've reached the EOF token
            if let Some(Token::Eof) = self.current_token() {
                break;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(Program { statements })
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), ParseError> {
        match self.current_token() {
            Some(Token::Word(w)) if w == expected => {
                self.advance();
                Ok(())
            }
            Some(other) => Err(ParseError {
                message: format!("Expected word '{}', found '{}'", expected, other),
                position: self.position,
            }),
            None => Err(ParseError {
                message: format!("Expected word '{}', found EOF", expected),
                position: self.position,
            }),
        }
    }

    fn expect_period(&mut self) -> Result<(), ParseError> {
        match self.current_token() {
            Some(Token::Period) => {
                self.advance();
                Ok(())
            }
            Some(other) => Err(ParseError {
                message: format!("Expected '.', found '{}'", other),
                position: self.position,
            }),
            None => Err(ParseError {
                message: "Expected '.', found EOF".to_string(),
                position: self.position,
            }),
        }
    }

    fn parse_statement(&mut self) -> Result<AstNode, ParseError> {
        // Pattern match on the token sequence to determine intent
        match self.current_token() {
            // New concise syntax (Hint v2.0)
            Some(Token::Word(word)) if word == "print" => self.parse_print_statement(),
            Some(Token::Word(word)) if word == "let" => self.parse_let_statement(),
            Some(Token::Word(word)) if word == "exit" => self.parse_exit_statement(),
            // Legacy syntax (backward compatible)
            Some(Token::Word(word)) if word == "say" => self.parse_say_statement(),
            Some(Token::Word(word)) if word == "keep" => self.parse_keep_statement(),
            Some(Token::Word(word)) if word == "stop" => self.parse_stop_statement(),
            Some(other) => Err(ParseError {
                message: format!(
                    "Unexpected token '{}'. Expected 'print', 'let', 'exit', 'say', 'keep', or 'stop'.",
                    other
                ),
                position: self.position,
            }),
            None => Err(ParseError {
                message: "Unexpected end of input".to_string(),
                position: self.position,
            }),
        }
    }

    /// Parse: `Print "[text]".`
    fn parse_print_statement(&mut self) -> Result<AstNode, ParseError> {
        // Consume "print"
        self.expect_word("print")?;

        // Expect a string literal
        let content = match self.current_token() {
            Some(Token::String(s)) => {
                let cloned = s.clone();
                self.advance();
                cloned
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected string after 'print', found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected string after 'print', found EOF".to_string(),
                    position: self.position,
                });
            }
        };

        // Expect terminating period
        self.expect_period()?;

        // Strip quotes from the string for the AST
        let unquoted = content
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(&content)
            .to_string();

        Ok(AstNode::Speak(unquoted))
    }

    /// Parse: `Let [name] = [value].`
    fn parse_let_statement(&mut self) -> Result<AstNode, ParseError> {
        // Consume "let"
        self.expect_word("let")?;

        // Expect variable name
        let name = match self.current_token() {
            Some(Token::Word(w)) => {
                let cloned = w.clone();
                self.advance();
                cloned
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected variable name after 'let', found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected variable name after 'let', found EOF".to_string(),
                    position: self.position,
                });
            }
        };

        // Expect "="
        match self.current_token() {
            Some(Token::Word(w)) if w == "=" => {
                self.advance();
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected '=' after variable name, found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected '=' after variable name, found EOF".to_string(),
                    position: self.position,
                });
            }
        };

        // Expect a numeric literal or list
        match self.current_token() {
            Some(Token::Number(n)) => {
                let val = *n;
                self.advance();
                self.expect_period()?;
                Ok(AstNode::Remember { name, value: val })
            }
            Some(Token::Float(f)) => {
                // For floats, store as int * 1000 (simplified)
                let val = (f * 1000.0) as i32;
                self.advance();
                self.expect_period()?;
                Ok(AstNode::Remember { name, value: val })
            }
            Some(Token::OpenBracket) => {
                // List syntax: Let items = [1, 2, 3].
                self.advance(); // consume "["
                let mut values = Vec::new();
                
                // Parse list elements
                loop {
                    match self.current_token() {
                        Some(Token::CloseBracket) => {
                            self.advance(); // consume "]"
                            break;
                        }
                        Some(Token::Number(n)) => {
                            values.push(*n);
                            self.advance();
                        }
                        Some(Token::Comma) => {
                            self.advance(); // consume ","
                        }
                        Some(other) => {
                            return Err(ParseError {
                                message: format!("Expected number or ']' in list, found '{}'", other),
                                position: self.position,
                            });
                        }
                        None => {
                            return Err(ParseError {
                                message: "Expected ']' in list, found EOF".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
                
                self.expect_period()?;
                Ok(AstNode::RememberList { name, values })
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected number or list after '=', found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected number or list after '=', found EOF".to_string(),
                    position: self.position,
                });
            }
        }
    }

    /// Parse: `Exit.`
    fn parse_exit_statement(&mut self) -> Result<AstNode, ParseError> {
        // Consume "exit"
        self.expect_word("exit")?;
        // Expect terminating period
        self.expect_period()?;
        Ok(AstNode::Halt)
    }

    /// Parse: `Say "[text]".`
    fn parse_say_statement(&mut self) -> Result<AstNode, ParseError> {
        // Consume "say"
        self.expect_word("say")?;

        // Expect a string literal
        let content = match self.current_token() {
            Some(Token::String(s)) => {
                let cloned = s.clone();
                self.advance();
                cloned
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected string after 'say', found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected string after 'say', found EOF".to_string(),
                    position: self.position,
                });
            }
        };

        // Expect terminating period
        self.expect_period()?;

        // Strip quotes from the string for the AST
        let unquoted = content
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(&content)
            .to_string();

        Ok(AstNode::Speak(unquoted))
    }

    /// Parse: `Keep the number [number] in mind as the [name].`
    /// Or: `Keep the list [1, 2, 3] in mind as the items.`
    fn parse_keep_statement(&mut self) -> Result<AstNode, ParseError> {
        // Consume "keep"
        self.expect_word("keep")?;

        // Expect "the"
        self.expect_word("the")?;

        // Check if it's "list" or "number"
        if let Some(Token::Word(w)) = self.current_token() {
            if w == "list" {
                // It's a list: Keep the list [...] in mind as the name.
                self.advance(); // consume "list"

                // Expect "in"
                self.expect_word("in")?;
                // Expect "mind"
                self.expect_word("mind")?;
                // Expect "as"
                self.expect_word("as")?;
                // Expect "the"
                self.expect_word("the")?;

                // Expect the variable name
                let name = match self.current_token() {
                    Some(Token::Word(w)) => {
                        let cloned = w.clone();
                        self.advance();
                        cloned
                    }
                    Some(other) => {
                        return Err(ParseError {
                            message: format!("Expected identifier for variable name, found '{}'", other),
                            position: self.position,
                        });
                    }
                    None => {
                        return Err(ParseError {
                            message: "Expected identifier for variable name, found EOF".to_string(),
                            position: self.position,
                        });
                    }
                };

                // Expect period
                self.expect_period()?;

                // For now, return empty list (full list parsing to be implemented)
                return Ok(AstNode::RememberList { name, values: vec![] });
            }
        }
        
        // It's a number: Keep the number N in mind as the name.
        // Expect "number"
        self.expect_word("number")?;

        // Expect a numeric literal
        let value = match self.current_token() {
            Some(Token::Number(n)) => {
                let val = *n;
                self.advance();
                val
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected number, found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected number, found EOF".to_string(),
                    position: self.position,
                });
            }
        };

        // Expect "in"
        self.expect_word("in")?;

        // Expect "mind"
        self.expect_word("mind")?;

        // Expect "as"
        self.expect_word("as")?;

        // Expect "the"
        self.expect_word("the")?;

        // Expect the variable name (identifier)
        let name = match self.current_token() {
            Some(Token::Word(w)) => {
                let cloned = w.clone();
                self.advance();
                cloned
            }
            Some(other) => {
                return Err(ParseError {
                    message: format!("Expected identifier for variable name, found '{}'", other),
                    position: self.position,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected identifier for variable name, found EOF".to_string(),
                    position: self.position,
                });
            }
        };

        // Expect terminating period
        self.expect_period()?;

        Ok(AstNode::Remember { name, value })
    }

    /// Parse: `Stop the program.`
    fn parse_stop_statement(&mut self) -> Result<AstNode, ParseError> {
        // Consume "stop"
        self.expect_word("stop")?;

        // Expect "the"
        self.expect_word("the")?;

        // Expect "program"
        self.expect_word("program")?;

        // Expect terminating period
        self.expect_period()?;

        Ok(AstNode::Halt)
    }
}

/// Parse input string into a Program AST.
pub fn parse(input: &str) -> Result<Program, ParseError> {
    let tokens = tokenize(input).map_err(ParseError::from)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_say_statement() {
        let input = r#"Say "Hello, world!"."#;
        let program = parse(input).unwrap();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(
            program.statements[0],
            AstNode::Speak("Hello, world!".to_string())
        );
    }

    #[test]
    fn test_parse_keep_statement() {
        let input = "Keep the number 42 in mind as the answer.";
        let program = parse(input).unwrap();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(
            program.statements[0],
            AstNode::Remember {
                name: "answer".to_string(),
                value: 42
            }
        );
    }

    #[test]
    fn test_parse_halt_statement() {
        let input = "Stop the program.";
        let program = parse(input).unwrap();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.statements[0], AstNode::Halt);
    }

    #[test]
    fn test_parse_multiple_statements() {
        let input = r#"Say "Starting.".
Keep the number 10 in mind as the counter.
Stop the program."#;
        let program = parse(input).unwrap();

        assert_eq!(program.statements.len(), 3);
        assert_eq!(program.statements[0], AstNode::Speak("Starting.".to_string()));
        assert_eq!(
            program.statements[1],
            AstNode::Remember {
                name: "counter".to_string(),
                value: 10
            }
        );
        assert_eq!(program.statements[2], AstNode::Halt);
    }

    #[test]
    fn test_case_insensitive_parsing() {
        let input = r#"SAY "test".
KEEP THE NUMBER 5 IN MIND AS THE X.
STOP THE PROGRAM."#;
        let program = parse(input).unwrap();

        assert_eq!(program.statements.len(), 3);
        assert_eq!(program.statements[0], AstNode::Speak("test".to_string()));
        assert_eq!(
            program.statements[1],
            AstNode::Remember {
                name: "x".to_string(),
                value: 5
            }
        );
        assert_eq!(program.statements[2], AstNode::Halt);
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let input = "Hello world.";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_missing_period() {
        let input = r#"Say "Hello""#;
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_negative_number() {
        let input = "Keep the number -100 in mind as the temperature.";
        let program = parse(input).unwrap();

        assert_eq!(
            program.statements[0],
            AstNode::Remember {
                name: "temperature".to_string(),
                value: -100
            }
        );
    }
}
