//! Error Codes and Categories
//! 
//! Defines error codes for all compiler errors with explanations.

use std::fmt;

/// Error category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Lexical analysis errors
    Lexical,
    /// Syntax/parsing errors
    Syntax,
    /// Type checking errors
    Type,
    /// Semantic analysis errors
    Semantic,
    /// Code generation errors
    Codegen,
    /// Linking errors
    Linking,
    /// I/O errors
    IO,
    /// Internal compiler errors
    Internal,
}

impl ErrorCategory {
    pub fn prefix(&self) -> &'static str {
        match self {
            ErrorCategory::Lexical => "L",
            ErrorCategory::Syntax => "S",
            ErrorCategory::Type => "T",
            ErrorCategory::Semantic => "M",
            ErrorCategory::Codegen => "C",
            ErrorCategory::Linking => "K",
            ErrorCategory::IO => "I",
            ErrorCategory::Internal => "ICE",
        }
    }
}

/// Error code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode {
    pub category: ErrorCategory,
    pub number: u32,
}

impl ErrorCode {
    pub const fn new(category: ErrorCategory, number: u32) -> Self {
        Self { category, number }
    }
    
    /// Format as string (e.g., "E0001")
    pub fn as_str(&self) -> String {
        format!("{}{:04}", self.category.prefix(), self.number)
    }
    
    /// Get explanation for this error code
    pub fn explanation(&self) -> &'static str {
        match (self.category, self.number) {
            // Lexical errors
            (ErrorCategory::Lexical, 1) => "Unexpected character in source",
            (ErrorCategory::Lexical, 2) => "Unterminated string literal",
            (ErrorCategory::Lexical, 3) => "Invalid number format",
            (ErrorCategory::Lexical, 4) => "Invalid escape sequence",
            
            // Syntax errors
            (ErrorCategory::Syntax, 1) => "Expected statement keyword",
            (ErrorCategory::Syntax, 2) => "Missing period at end of statement",
            (ErrorCategory::Syntax, 3) => "Unexpected token",
            (ErrorCategory::Syntax, 4) => "Invalid statement structure",
            (ErrorCategory::Syntax, 5) => "Expected identifier",
            (ErrorCategory::Syntax, 6) => "Expected string literal",
            (ErrorCategory::Syntax, 7) => "Expected number",
            
            // Type errors
            (ErrorCategory::Type, 1) => "Type mismatch in expression",
            (ErrorCategory::Type, 2) => "Cannot infer type",
            (ErrorCategory::Type, 3) => "Unknown type",
            (ErrorCategory::Type, 4) => "Invalid type conversion",
            (ErrorCategory::Type, 5) => "Type does not implement trait",
            (ErrorCategory::Type, 6) => "Generic type parameter mismatch",
            
            // Semantic errors
            (ErrorCategory::Semantic, 1) => "Undefined variable",
            (ErrorCategory::Semantic, 2) => "Variable already defined",
            (ErrorCategory::Semantic, 3) => "Unknown function",
            (ErrorCategory::Semantic, 4) => "Wrong number of arguments",
            (ErrorCategory::Semantic, 5) => "Return type mismatch",
            (ErrorCategory::Semantic, 6) => "Missing return statement",
            (ErrorCategory::Semantic, 7) => "Unreachable code",
            (ErrorCategory::Semantic, 8) => "Unused variable",
            (ErrorCategory::Semantic, 9) => "Mutable variable used immutably",
            
            // Codegen errors
            (ErrorCategory::Codegen, 1) => "Failed to generate code",
            (ErrorCategory::Codegen, 2) => "Invalid IR",
            (ErrorCategory::Codegen, 3) => "Register allocation failed",
            
            // Linking errors
            (ErrorCategory::Linking, 1) => "Undefined symbol",
            (ErrorCategory::Linking, 2) => "Duplicate symbol",
            (ErrorCategory::Linking, 3) => "Incompatible object files",
            
            // I/O errors
            (ErrorCategory::IO, 1) => "Failed to read file",
            (ErrorCategory::IO, 2) => "Failed to write file",
            (ErrorCategory::IO, 3) => "File not found",
            (ErrorCategory::IO, 4) => "Permission denied",
            
            // Internal errors
            (ErrorCategory::Internal, 1) => "Compiler bug: unreachable code reached",
            (ErrorCategory::Internal, 2) => "Compiler bug: assertion failed",
            (ErrorCategory::Internal, 3) => "Compiler bug: unexpected state",
            
            _ => "Unknown error",
        }
    }
    
    /// Get help message for this error
    pub fn help(&self) -> &'static str {
        match (self.category, self.number) {
            (ErrorCategory::Lexical, 1) => "Remove or escape the invalid character",
            (ErrorCategory::Lexical, 2) => "Add a closing quote to terminate the string",
            (ErrorCategory::Lexical, 3) => "Check that the number is a valid integer",
            (ErrorCategory::Syntax, 2) => "Add a period (.) at the end of the statement",
            (ErrorCategory::Syntax, 3) => "Check the syntax of your statement",
            (ErrorCategory::Type, 1) => "Ensure both sides of the operation have compatible types",
            (ErrorCategory::Type, 4) => "Use an explicit type conversion",
            (ErrorCategory::Semantic, 1) => "Declare the variable before using it",
            (ErrorCategory::Semantic, 2) => "Use a different variable name",
            (ErrorCategory::Semantic, 4) => "Check the function signature",
            (ErrorCategory::IO, 3) => "Check that the file path is correct",
            _ => "See the error message for details",
        }
    }
    
    /// Get URL for more information
    pub fn documentation_url(&self) -> String {
        format!("https://hint-lang.org/errors/{}", self.as_str())
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.as_str())
    }
}

// Predefined error codes
impl ErrorCode {
    // Lexical errors (L0001-L0099)
    pub const LexicalError: Self = Self::new(ErrorCategory::Lexical, 1);
    pub const UnterminatedString: Self = Self::new(ErrorCategory::Lexical, 2);
    pub const InvalidNumber: Self = Self::new(ErrorCategory::Lexical, 3);
    pub const InvalidEscape: Self = Self::new(ErrorCategory::Lexical, 4);
    
    // Syntax errors (S0001-S0099)
    pub const ExpectedKeyword: Self = Self::new(ErrorCategory::Syntax, 1);
    pub const MissingPeriod: Self = Self::new(ErrorCategory::Syntax, 2);
    pub const UnexpectedToken: Self = Self::new(ErrorCategory::Syntax, 3);
    pub const InvalidStructure: Self = Self::new(ErrorCategory::Syntax, 4);
    pub const ExpectedIdentifier: Self = Self::new(ErrorCategory::Syntax, 5);
    pub const ExpectedString: Self = Self::new(ErrorCategory::Syntax, 6);
    pub const ExpectedNumber: Self = Self::new(ErrorCategory::Syntax, 7);
    
    // Type errors (T0001-T0099)
    pub const TypeMismatch: Self = Self::new(ErrorCategory::Type, 1);
    pub const CannotInfer: Self = Self::new(ErrorCategory::Type, 2);
    pub const UnknownType: Self = Self::new(ErrorCategory::Type, 3);
    pub const InvalidConversion: Self = Self::new(ErrorCategory::Type, 4);
    pub const TraitNotImplemented: Self = Self::new(ErrorCategory::Type, 5);
    pub const GenericMismatch: Self = Self::new(ErrorCategory::Type, 6);
    
    // Semantic errors (M0001-M0099)
    pub const UndefinedVariable: Self = Self::new(ErrorCategory::Semantic, 1);
    pub const DuplicateVariable: Self = Self::new(ErrorCategory::Semantic, 2);
    pub const UnknownFunction: Self = Self::new(ErrorCategory::Semantic, 3);
    pub const WrongArgCount: Self = Self::new(ErrorCategory::Semantic, 4);
    pub const ReturnTypeMismatch: Self = Self::new(ErrorCategory::Semantic, 5);
    pub const MissingReturn: Self = Self::new(ErrorCategory::Semantic, 6);
    pub const UnreachableCode: Self = Self::new(ErrorCategory::Semantic, 7);
    pub const UnusedVariable: Self = Self::new(ErrorCategory::Semantic, 8);
    
    // Codegen errors (C0001-C0099)
    pub const CodegenFailed: Self = Self::new(ErrorCategory::Codegen, 1);
    pub const InvalidIR: Self = Self::new(ErrorCategory::Codegen, 2);
    
    // I/O errors (I0001-I0099)
    pub const FileNotFound: Self = Self::new(ErrorCategory::IO, 3);
    pub const ReadFailed: Self = Self::new(ErrorCategory::IO, 1);
    pub const WriteFailed: Self = Self::new(ErrorCategory::IO, 2);
    
    // Internal errors (ICE0001-ICE0099)
    pub const ICE: Self = Self::new(ErrorCategory::Internal, 1);
}

/// Error explanation with full details
#[derive(Debug, Clone)]
pub struct ErrorExplanation {
    pub code: ErrorCode,
    pub title: &'static str,
    pub explanation: &'static str,
    pub example: &'static str,
    pub solution: &'static str,
}

impl ErrorExplanation {
    pub fn get(code: ErrorCode) -> Option<Self> {
        Some(match code {
            ErrorCode::TypeMismatch => Self {
                code,
                title: "Type mismatch",
                explanation: "The types of two expressions are incompatible for the operation being performed.",
                example: r#"Keep the number 42 in mind as the answer.
Keep the text "hello" in mind as the greeting.
Keep the answer + the greeting in mind as the result.
// Error: cannot add str to i64"#,
                solution: "Ensure both operands have compatible types, or use explicit conversion.",
            },
            ErrorCode::UndefinedVariable => Self {
                code,
                title: "Undefined variable",
                explanation: "A variable is used before it has been declared.",
                example: r#"Say the answer.
// Error: 'answer' is not defined"#,
                solution: "Declare the variable with 'Keep' before using it.",
            },
            ErrorCode::MissingPeriod => Self {
                code,
                title: "Missing period",
                explanation: "Hint statements must end with a period.",
                example: r#"Say "Hello"
// Error: expected '.' at end of statement"#,
                solution: "Add a period at the end of the statement.",
            },
            ErrorCode::DuplicateVariable => Self {
                code,
                title: "Duplicate variable",
                explanation: "A variable is declared more than once in the same scope.",
                example: r#"Keep the number 1 in mind as the x.
Keep the number 2 in mind as the x.
// Error: 'x' is already defined"#,
                solution: "Use a different variable name or remove the duplicate declaration.",
            },
            _ => return None,
        })
    }
}

impl fmt::Display for ErrorExplanation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", self.code, self.title)?;
        writeln!(f)?;
        writeln!(f, "Explanation:")?;
        writeln!(f, "  {}", self.explanation)?;
        writeln!(f)?;
        writeln!(f, "Example:")?;
        for line in self.example.lines() {
            writeln!(f, "  {}", line)?;
        }
        writeln!(f)?;
        writeln!(f, "Solution:")?;
        writeln!(f, "  {}", self.solution)?;
        writeln!(f)?;
        writeln!(f, "For more information: {}", self.code.documentation_url())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_code_format() {
        let code = ErrorCode::TypeMismatch;
        assert_eq!(code.as_str(), "T0001");
        assert_eq!(format!("{}", code), "[T0001]");
    }
    
    #[test]
    fn test_error_explanation() {
        let explanation = ErrorExplanation::get(ErrorCode::TypeMismatch).unwrap();
        assert_eq!(explanation.title, "Type mismatch");
        assert!(explanation.explanation.contains("incompatible"));
    }
    
    #[test]
    fn test_error_category_prefix() {
        assert_eq!(ErrorCategory::Lexical.prefix(), "L");
        assert_eq!(ErrorCategory::Syntax.prefix(), "S");
        assert_eq!(ErrorCategory::Type.prefix(), "T");
        assert_eq!(ErrorCategory::Internal.prefix(), "ICE");
    }
}
