//! Lexer for the Hint programming language.
//!
//! Tokenizes conversational English input into a stream of tokens.
//! Case-insensitive for keywords, preserves string literals as-is.

use std::fmt;
use crate::diagnostics::Diagnostic;

/// A token produced by the lexer.
pub enum Token {
    /// A word/identifier (normalized to lowercase for matching).
    Word(String),
    /// A string literal including quotes, e.g., `"Hello, world!"`.
    String(String),
    /// A numeric literal, e.g., `42` or `-17`.
    Number(i32),
    /// A float literal, e.g., `3.14`.
    Float(f64),
    /// The equals sign `=`.
    Equals,
    /// The period punctuation `.`.
    Period,
    /// The comma punctuation `,`.
    Comma,
    /// The opening bracket `[`.
    OpenBracket,
    /// The closing bracket `]`.
    CloseBracket,
    /// The opening parenthesis `(`.
    OpenParen,
    /// The closing parenthesis `)`.
    CloseParen,
    /// The colon `:`.
    Colon,
    /// End of token stream.
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Word(s) => write!(f, "Word({})", s),
            Token::String(s) => write!(f, "String({})", s),
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Float(n) => write!(f, "Float({})", n),
            Token::Equals => write!(f, "="),
            Token::Period => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Colon => write!(f, ":"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Error type for lexical analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexical error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for LexError {}

impl LexError {
    pub fn to_diagnostic(&self, source: &str) -> crate::diagnostics::Diagnostic {
        Diagnostic::error()
            .with_message(&self.message)
            .with_span(self.position, self.position + 1)
            .with_source(source.to_string())
    }
}

/// The lexer for Hint source code.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    /// Create a new lexer for the given input string.
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    /// Tokenize the entire input, returning a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }


    fn advance(&mut self) -> Option<char> {
        let ch = self.current_char();
        if ch.is_some() {
            self.position += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let start_pos = self.position;
        let ch = self.current_char().ok_or_else(|| LexError {
            message: "Unexpected end of input".to_string(),
            position: start_pos,
        })?;

        match ch {
            '"' => self.read_string(start_pos),
            '.' => {
                self.advance();
                Ok(Token::Period)
            }
            '=' => {
                self.advance();
                Ok(Token::Equals)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '[' => {
                self.advance();
                Ok(Token::OpenBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::CloseBracket)
            }
            '(' => {
                self.advance();
                Ok(Token::OpenParen)
            }
            ')' => {
                self.advance();
                Ok(Token::CloseParen)
            }
            ':' => {
                self.advance();
                Ok(Token::Colon)
            }
            '-' | '0'..='9' => self.read_number(start_pos),
            _ => {
                if ch.is_alphabetic() || ch == '_' {
                    self.read_word(start_pos)
                } else {
                    Err(LexError {
                        message: format!("Unexpected character: '{}'", ch),
                        position: start_pos,
                    })
                }
            }
        }
    }

    fn read_string(&mut self, start_pos: usize) -> Result<Token, LexError> {
        // Consume opening quote
        self.advance();

        let mut content = String::new();

        while let Some(ch) = self.current_char() {
            if ch == '"' {
                // Consume closing quote
                self.advance();
                // Return the string with quotes included for AST convenience
                return Ok(Token::String(format!("\"{}\"", content)));
            }

            if ch == '\n' {
                return Err(LexError {
                    message: "Unterminated string literal".to_string(),
                    position: start_pos,
                });
            }

            content.push(ch);
            self.advance();
        }

        Err(LexError {
            message: "Unterminated string literal".to_string(),
            position: start_pos,
        })
    }

    fn read_number(&mut self, start_pos: usize) -> Result<Token, LexError> {
        let mut num_str = String::new();
        let is_negative = self.current_char() == Some('-');

        if is_negative {
            num_str.push(self.advance().unwrap());
        }

        // Must have at least one digit after optional minus
        if !matches!(self.current_char(), Some('0'..='9')) {
            return Err(LexError {
                message: "Invalid number format".to_string(),
                position: start_pos,
            });
        }

        // Read integer part
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for float (decimal point followed by digits)
        let mut is_float = false;
        if self.current_char() == Some('.') {
            if let Some(next_ch) = self.input.get(self.position + 1).copied() {
                if next_ch.is_ascii_digit() {
                    // It's a float
                    is_float = true;
                    num_str.push(self.advance().unwrap()); // consume '.'
                    while let Some(ch) = self.current_char() {
                        if ch.is_ascii_digit() {
                            num_str.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Check for scientific notation (e.g., 1e5, 1.0e5, 1E5)
        if let Some(ch) = self.current_char() {
            if ch == 'e' || ch == 'E' {
                is_float = true;
                num_str.push(ch);
                self.advance();
                
                // Optional sign after exponent
                if let Some(sign) = self.current_char() {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }
                
                // Must have at least one digit in exponent
                if !matches!(self.current_char(), Some('0'..='9')) {
                    return Err(LexError {
                        message: "Invalid scientific notation: expected digit after 'e' or 'E'".to_string(),
                        position: start_pos,
                    });
                }
                
                while let Some(ch) = self.current_char() {
                    if ch.is_ascii_digit() {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        if is_float {
            return num_str.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| LexError {
                    message: format!("Number out of range: {}", num_str),
                    position: start_pos,
                });
        }

        // It's an integer
        num_str
            .parse::<i32>()
            .map(Token::Number)
            .map_err(|_| LexError {
                message: format!("Number out of range: {}", num_str),
                position: start_pos,
            })
    }

    fn read_word(&mut self, start_pos: usize) -> Result<Token, LexError> {
        let mut word = String::new();

        while let Some(ch) = self.current_char() {
            // Allow alphabetic characters, underscore, and digits after first char
            if ch.is_alphabetic() || ch == '_' || (!word.is_empty() && ch.is_ascii_digit()) {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if word.is_empty() {
            return Err(LexError {
                message: "Expected identifier".to_string(),
                position: start_pos,
            });
        }

        // Normalize to lowercase for case-insensitive matching
        Ok(Token::Word(word.to_lowercase()))
    }
}

/// Tokenize input string into a vector of tokens.
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(input);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_say_statement() {
        let input = r#"Say "Hello, world!"."#;
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Word("say".to_string()));
        assert_eq!(tokens[1], Token::String("\"Hello, world!\"".to_string()));
        assert_eq!(tokens[2], Token::Period);
        assert_eq!(tokens[3], Token::Eof);
    }

    #[test]
    fn test_tokenize_keep_statement() {
        let input = "Keep the number 42 in mind as the answer.";
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens[0], Token::Word("keep".to_string()));
        assert_eq!(tokens[3], Token::Number(42));
        assert_eq!(tokens[8], Token::Word("answer".to_string()));
    }

    #[test]
    fn test_tokenize_halt_statement() {
        let input = "Stop the program.";
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens[0], Token::Word("stop".to_string()));
        assert_eq!(tokens[1], Token::Word("the".to_string()));
        assert_eq!(tokens[2], Token::Word("program".to_string()));
        assert_eq!(tokens[3], Token::Period);
    }

    #[test]
    fn test_case_insensitivity() {
        let input = r#"SAY "test"."#;
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens[0], Token::Word("say".to_string()));
    }

    #[test]
    fn test_negative_number() {
        let input = "Keep the number -17 in mind as x.";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens[3], Token::Number(-17));
    }

    #[test]
    fn test_unterminated_string() {
        let input = r#"Say "Hello"#;
        let result = tokenize(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_character() {
        let input = "Say @invalid.";
        let result = tokenize(input);
        assert!(result.is_err());
    }
}
