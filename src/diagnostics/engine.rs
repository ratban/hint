//! Diagnostic Engine
//! 
//! Collects, filters, and reports diagnostics.

use super::diagnostic::{Diagnostic, DiagnosticLevel, Span};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticId(pub u64);

impl DiagnosticId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Filter for diagnostics
#[derive(Debug, Clone)]
pub struct DiagnosticFilter {
    /// Minimum level to report
    pub min_level: DiagnosticLevel,
    /// Specific error codes to deny
    pub deny_codes: Vec<String>,
    /// Specific error codes to allow
    pub allow_codes: Vec<String>,
    /// Specific error codes to warn
    pub warn_codes: Vec<String>,
}

impl Default for DiagnosticFilter {
    fn default() -> Self {
        Self {
            min_level: DiagnosticLevel::Warning,
            deny_codes: Vec::new(),
            allow_codes: Vec::new(),
            warn_codes: Vec::new(),
        }
    }
}

/// Diagnostic engine for collecting and reporting errors
#[derive(Debug)]
pub struct DiagnosticsEngine {
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Error count
    error_count: usize,
    /// Warning count
    warning_count: usize,
    /// Filter settings
    filter: DiagnosticFilter,
    /// Maximum errors before aborting
    max_errors: Option<usize>,
    /// Track emitted diagnostic codes to avoid duplicates
    emitted_codes: HashMap<String, usize>,
}

impl DiagnosticsEngine {
    /// Create a new diagnostics engine
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            filter: DiagnosticFilter::default(),
            max_errors: Some(10),
            emitted_codes: HashMap::new(),
        }
    }
    
    /// Create with custom filter
    pub fn with_filter(filter: DiagnosticFilter) -> Self {
        Self {
            filter,
            ..Self::new()
        }
    }
    
    /// Set maximum errors before aborting
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = Some(max);
        self
    }
    
    /// Emit a diagnostic
    pub fn emit(&mut self, diagnostic: Diagnostic) {
        // Check filter
        if !self.should_emit(&diagnostic) {
            return;
        }
        
        // Check max errors
        if let Some(max) = self.max_errors {
            if self.error_count >= max {
                return;
            }
        }
        
        // Track counts
        match diagnostic.level {
            DiagnosticLevel::Error | DiagnosticLevel::Bug => {
                self.error_count += 1;
            }
            DiagnosticLevel::Warning => {
                self.warning_count += 1;
            }
            _ => {}
        }
        
        // Track emitted codes
        if let Some(code) = &diagnostic.code {
            *self.emitted_codes.entry(code.clone()).or_insert(0) += 1;
        }
        
        self.diagnostics.push(diagnostic);
    }
    
    /// Check if a diagnostic should be emitted
    fn should_emit(&self, diagnostic: &Diagnostic) -> bool {
        // Check minimum level
        if diagnostic.level < self.filter.min_level {
            return false;
        }
        
        // Check deny codes
        if let Some(code) = &diagnostic.code {
            if self.filter.deny_codes.contains(code) {
                return true; // Always emit denied codes
            }
            
            if self.filter.allow_codes.contains(code) {
                return false; // Skip allowed codes
            }
            
            if self.filter.warn_codes.contains(code) {
                // Downgrade to warning
                return diagnostic.level >= DiagnosticLevel::Warning;
            }
        }
        
        true
    }
    
    /// Emit an error
    pub fn error(&mut self, message: impl Into<String>) {
        self.emit(Diagnostic::error().with_message(message));
    }

    /// Emit an error with span
    pub fn error_span(&mut self, message: impl Into<String>, span: Span) {
        self.emit(Diagnostic::error().with_message(message).with_span(span.start, span.end));
    }

    /// Emit a warning
    pub fn warning(&mut self, message: impl Into<String>) {
        self.emit(Diagnostic::warning().with_message(message));
    }

    /// Emit a warning with span
    pub fn warning_span(&mut self, message: impl Into<String>, span: Span) {
        self.emit(Diagnostic::warning().with_message(message).with_span(span.start, span.end));
    }
    
    /// Emit a note
    pub fn note(&mut self, message: impl Into<String>) {
        self.emit(Diagnostic::note().with_message(message));
    }
    
    /// Emit a help message
    pub fn help(&mut self, message: impl Into<String>) {
        self.emit(Diagnostic::help().with_message(message));
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
    
    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }
    
    /// Get error count
    pub fn error_count(&self) -> usize {
        self.error_count
    }
    
    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }
    
    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Get diagnostics by level
    pub fn diagnostics_by_level(&self, level: DiagnosticLevel) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.level == level)
            .collect()
    }
    
    /// Get diagnostics by code
    pub fn diagnostics_by_code(&self, code: &str) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.code.as_ref().map(|c| c == code).unwrap_or(false))
            .collect()
    }
    
    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
        self.emitted_codes.clear();
    }
    
    /// Get summary of emitted diagnostics
    pub fn summary(&self) -> DiagnosticSummary {
        DiagnosticSummary {
            total: self.diagnostics.len(),
            errors: self.error_count,
            warnings: self.warning_count,
            notes: self.diagnostics_by_level(DiagnosticLevel::Note).len(),
            helps: self.diagnostics_by_level(DiagnosticLevel::Help).len(),
            unique_codes: self.emitted_codes.len(),
        }
    }
    
    /// Check if we've reached max errors
    pub fn is_aborted(&self) -> bool {
        if let Some(max) = self.max_errors {
            self.error_count >= max
        } else {
            false
        }
    }
    
    /// Format all diagnostics as a string
    pub fn format(&self, source_name: &str) -> String {
        use super::render::TerminalRenderer;
        use super::DiagnosticRenderer;
        
        let renderer = TerminalRenderer::without_colors();
        let mut output = String::new();
        
        for diag in &self.diagnostics {
            output.push_str(&renderer.render(diag, source_name));
            output.push('\n');
        }
        
        output
    }
    
    /// Print diagnostics to stderr
    pub fn print(&self, source_name: &str) {
        eprintln!("{}", self.format(source_name));
    }
}

impl Default for DiagnosticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DiagnosticsEngine {
    fn clone(&self) -> Self {
        Self {
            diagnostics: self.diagnostics.clone(),
            error_count: self.error_count,
            warning_count: self.warning_count,
            filter: self.filter.clone(),
            max_errors: self.max_errors,
            emitted_codes: self.emitted_codes.clone(),
        }
    }
}

/// Summary of emitted diagnostics
#[derive(Debug, Clone)]
pub struct DiagnosticSummary {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub notes: usize,
    pub helps: usize,
    pub unique_codes: usize,
}

impl DiagnosticSummary {
    pub fn is_clean(&self) -> bool {
        self.errors == 0 && self.warnings == 0
    }
}

impl fmt::Display for DiagnosticSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} diagnostics", self.total)?;
        
        if self.errors > 0 {
            write!(f, " ({} errors", self.errors)?;
            if self.warnings > 0 {
                write!(f, ", {} warnings", self.warnings)?;
            }
            write!(f, ")")?;
        } else if self.warnings > 0 {
            write!(f, " ({} warnings)", self.warnings)?;
        }
        
        Ok(())
    }
}

/// Builder for creating diagnostics with fluent API
pub struct DiagnosticBuilder {
    diagnostic: Diagnostic,
}

impl DiagnosticBuilder {
    pub fn error() -> Self {
        Self {
            diagnostic: Diagnostic::error(),
        }
    }

    pub fn warning() -> Self {
        Self {
            diagnostic: Diagnostic::warning(),
        }
    }

    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.diagnostic.code = Some(code.into());
        self
    }
    
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.diagnostic.message = message.into();
        self
    }
    
    pub fn span(mut self, start: usize, end: usize) -> Self {
        self.diagnostic.span = Some(Span::new(start, end));
        self
    }
    
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.diagnostic.file = Some(file.into());
        self
    }
    
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.diagnostic.source = Some(source.into());
        self
    }
    
    pub fn label_primary(mut self, span: Span, label: impl Into<String>) -> Self {
        self.diagnostic.labels.push(DiagnosticLabel::primary(span).with_label(label));
        self
    }
    
    pub fn label_secondary(mut self, span: Span, label: impl Into<String>) -> Self {
        self.diagnostic.labels.push(DiagnosticLabel::secondary(span).with_label(label));
        self
    }
    
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.diagnostic.children.push(SubDiagnostic::note(note));
        self
    }
    
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.diagnostic.children.push(SubDiagnostic::help(help));
        self
    }
    
    pub fn suggestion(mut self, message: impl Into<String>, replacement: impl Into<String>) -> Self {
        self.diagnostic.suggestions.push(Suggestion {
            message: message.into(),
            style: SuggestionStyle::CodeBlock,
            span: self.diagnostic.span,
            replacement: Some(replacement.into()),
        });
        self
    }
    
    pub fn build(self) -> Diagnostic {
        self.diagnostic
    }
}

use super::diagnostic::{DiagnosticLabel, SubDiagnostic, Suggestion, SuggestionStyle};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_diagnostics_engine() {
        let mut engine = DiagnosticsEngine::new();
        
        engine.error("Something went wrong");
        engine.warning("This is a warning");
        
        assert!(engine.has_errors());
        assert!(engine.has_warnings());
        assert_eq!(engine.error_count(), 1);
        assert_eq!(engine.warning_count(), 1);
    }
    
    #[test]
    fn test_diagnostics_filter() {
        let filter = DiagnosticFilter {
            min_level: DiagnosticLevel::Error,
            deny_codes: vec!["E0001".to_string()],
            ..Default::default()
        };
        
        let mut engine = DiagnosticsEngine::with_filter(filter);
        
        // This should be emitted (error level)
        engine.emit(Diagnostic::error().with_message("Error").with_code("E0001"));
        
        // This should not be emitted (warning level, filter requires error)
        engine.emit(Diagnostic::warning().with_message("Warning"));
        
        assert_eq!(engine.error_count(), 1);
        assert_eq!(engine.warning_count(), 0);
    }
    
    #[test]
    fn test_diagnostic_summary() {
        let summary = DiagnosticSummary {
            total: 5,
            errors: 2,
            warnings: 3,
            notes: 0,
            helps: 0,
            unique_codes: 2,
        };
        
        assert!(!summary.is_clean());
        assert_eq!(summary.to_string(), "5 diagnostics (2 errors, 3 warnings)");
    }
    
    #[test]
    fn test_diagnostic_builder() {
        let diag = DiagnosticBuilder::error()
            .code("E0001")
            .message("Type mismatch")
            .span(10, 20)
            .help("Check your types")
            .build();
        
        assert_eq!(diag.code, Some("E0001".to_string()));
        assert_eq!(diag.message, "Type mismatch");
    }
}
