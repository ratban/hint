//! Core Diagnostic Types
//! 
//! Defines the diagnostic data structures used throughout the compiler.

use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    /// Internal compiler note
    Note,
    /// Helpful suggestion
    Help,
    /// Code warning (non-fatal)
    Warning,
    /// Compilation error (fatal)
    Error,
    /// Internal compiler error (bug)
    Bug,
}

impl DiagnosticLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticLevel::Note => "note",
            DiagnosticLevel::Help => "help",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Bug => "error: internal compiler error",
        }
    }
    
    pub fn color_code(&self) -> &'static str {
        match self {
            DiagnosticLevel::Note => "\x1b[36m",      // Cyan
            DiagnosticLevel::Help => "\x1b[32m",      // Green
            DiagnosticLevel::Warning => "\x1b[33m",   // Yellow
            DiagnosticLevel::Error => "\x1b[31m",     // Red
            DiagnosticLevel::Bug => "\x1b[35m",       // Magenta
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A source code span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    
    pub fn point(pos: usize) -> Self {
        Self { start: pos, end: pos + 1 }
    }
    
    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }
    
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }
    
    pub fn contains_span(&self, other: Span) -> bool {
        self.contains(other.start) && self.contains(other.end.saturating_sub(1))
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::empty()
    }
}

/// A labeled span within a diagnostic
#[derive(Debug, Clone)]
pub struct DiagnosticLabel {
    pub span: Span,
    pub label: Option<String>,
    pub is_primary: bool,
}

impl DiagnosticLabel {
    pub fn primary(span: Span) -> Self {
        Self {
            span,
            label: None,
            is_primary: true,
        }
    }
    
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    
    pub fn secondary(span: Span) -> Self {
        Self {
            span,
            label: None,
            is_primary: false,
        }
    }
}

/// A sub-diagnostic (note or help message with optional span)
#[derive(Debug, Clone)]
pub struct SubDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<Span>,
    pub label: Option<String>,
}

impl SubDiagnostic {
    pub fn note(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Note,
            message: message.into(),
            span: None,
            label: None,
        }
    }
    
    pub fn help(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Help,
            message: message.into(),
            span: None,
            label: None,
        }
    }
    
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_span_usize(mut self, start: usize, end: usize) -> Self {
        self.span = Some(Span::new(start, end));
        self
    }
    
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A compiler diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    pub level: DiagnosticLevel,
    /// Main error message
    pub message: String,
    /// Error code (e.g., "E0001")
    pub code: Option<String>,
    /// Primary source location
    pub span: Option<Span>,
    /// Source file name
    pub file: Option<String>,
    /// Source code snippet
    pub source: Option<String>,
    /// Additional labeled spans
    pub labels: Vec<DiagnosticLabel>,
    /// Additional notes and help messages
    pub children: Vec<SubDiagnostic>,
    /// Suggestions for fixing the error
    pub suggestions: Vec<Suggestion>,
}

/// A suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub style: SuggestionStyle,
    pub span: Option<Span>,
    pub replacement: Option<String>,
}

/// Style of a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionStyle {
    /// Show as inline code
    InlineCode,
    /// Show as a code block
    CodeBlock,
    /// Show as a shell command
    Command,
    /// Plain text suggestion
    Plain,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error() -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: String::new(),
            code: None,
            span: None,
            file: None,
            source: None,
            labels: Vec::new(),
            children: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    /// Create a new warning diagnostic
    pub fn warning() -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: String::new(),
            code: None,
            span: None,
            file: None,
            source: None,
            labels: Vec::new(),
            children: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    /// Create a new note diagnostic
    pub fn note() -> Self {
        Self {
            level: DiagnosticLevel::Note,
            message: String::new(),
            code: None,
            span: None,
            file: None,
            source: None,
            labels: Vec::new(),
            children: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    /// Create a new help diagnostic
    pub fn help() -> Self {
        Self {
            level: DiagnosticLevel::Help,
            message: String::new(),
            code: None,
            span: None,
            file: None,
            source: None,
            labels: Vec::new(),
            children: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    /// Set the error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
    
    /// Set the main message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
    
    /// Set the primary span
    pub fn with_span(mut self, start: usize, end: usize) -> Self {
        self.span = Some(Span::new(start, end));
        self
    }
    
    /// Set the source file name
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }
    
    /// Set the source code
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
    
    /// Add a primary label
    pub fn with_primary_label(mut self, span: Span, label: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel::primary(span).with_label(label));
        self
    }
    
    /// Add a secondary label
    pub fn with_secondary_label(mut self, span: Span, label: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel::secondary(span).with_label(label));
        self
    }
    
    /// Add a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.children.push(SubDiagnostic::note(note));
        self
    }
    
    /// Add a note with span
    pub fn with_note_span(mut self, span: Span, note: impl Into<String>) -> Self {
        self.children.push(SubDiagnostic::note(note).with_span(span));
        self
    }
    
    /// Add a help message
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.children.push(SubDiagnostic::help(help));
        self
    }
    
    /// Add a help message with span
    pub fn with_help_span(mut self, span: Span, help: impl Into<String>) -> Self {
        self.children.push(SubDiagnostic::help(help).with_span(span));
        self
    }
    
    /// Add a suggestion
    pub fn with_suggestion(mut self, message: impl Into<String>, replacement: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            style: SuggestionStyle::CodeBlock,
            span: self.span,
            replacement: Some(replacement.into()),
        });
        self
    }
    
    /// Add a suggestion with specific span
    pub fn with_suggestion_span(
        mut self,
        span: Span,
        message: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            style: SuggestionStyle::CodeBlock,
            span: Some(span),
            replacement: Some(replacement.into()),
        });
        self
    }
    
    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Error | DiagnosticLevel::Bug)
    }
    
    /// Check if this is a warning
    pub fn is_warning(&self) -> bool {
        self.level == DiagnosticLevel::Warning
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.level, self.message)?;
        
        if let Some(code) = &self.code {
            write!(f, " [{}]", code)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
    }
    
    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20);
        assert!(span.contains(15));
        assert!(!span.contains(5));
        assert!(!span.contains(25));
    }
    
    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error()
            .with_message("Something went wrong")
            .with_code("E0001")
            .with_span(10, 20);
        
        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert_eq!(diag.message, "Something went wrong");
        assert_eq!(diag.code, Some("E0001".to_string()));
    }
    
    #[test]
    fn test_diagnostic_labels() {
        let span1 = Span::new(10, 15);
        let span2 = Span::new(20, 25);
        
        let diag = Diagnostic::error()
            .with_message("Type mismatch")
            .with_primary_label(span1, "expected i64")
            .with_secondary_label(span2, "found str");
        
        assert_eq!(diag.labels.len(), 2);
        assert!(diag.labels[0].is_primary);
        assert!(!diag.labels[1].is_primary);
    }
    
    #[test]
    fn test_diagnostic_suggestions() {
        let diag = Diagnostic::warning()
            .with_message("Unused variable")
            .with_suggestion("Remove the variable", "");
        
        assert_eq!(diag.suggestions.len(), 1);
    }
}
