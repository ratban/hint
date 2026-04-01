//! Diagnostic Suggestions
//!
//! Provides suggestions for fixing errors.

use crate::diagnostics::diagnostic::Span;

/// A suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Message describing the suggestion
    pub message: String,
    /// How to display the suggestion
    pub style: SuggestionStyle,
    /// Span to apply the suggestion to
    pub span: Option<Span>,
    /// Replacement text (if applicable)
    pub replacement: Option<String>,
}

impl Suggestion {
    /// Create a code block suggestion
    pub fn code_block(message: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            style: SuggestionStyle::CodeBlock,
            span: None,
            replacement: Some(replacement.into()),
        }
    }
    
    /// Create an inline code suggestion
    pub fn inline_code(message: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            style: SuggestionStyle::InlineCode,
            span: None,
            replacement: Some(replacement.into()),
        }
    }
    
    /// Create a command suggestion
    pub fn command(message: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            style: SuggestionStyle::Command,
            span: None,
            replacement: Some(command.into()),
        }
    }
    
    /// Create a plain text suggestion
    pub fn plain(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            style: SuggestionStyle::Plain,
            span: None,
            replacement: None,
        }
    }
    
    /// Set the span for this suggestion
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

/// Style for displaying a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionStyle {
    /// Show as inline code: `code`
    InlineCode,
    /// Show as a code block
    CodeBlock,
    /// Show as a shell command
    Command,
    /// Plain text
    Plain,
}

impl SuggestionStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            SuggestionStyle::InlineCode => "inline",
            SuggestionStyle::CodeBlock => "block",
            SuggestionStyle::Command => "command",
            SuggestionStyle::Plain => "plain",
        }
    }
}

/// Suggestion applicability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// The suggestion is definitely the right fix
    MachineApplicable,
    /// The suggestion might need manual adjustment
    HasPlaceholders,
    /// The suggestion is possibly correct
    MaybeIncorrect,
    /// The suggestion is probably incorrect
    Unspecified,
}

impl Applicability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Applicability::MachineApplicable => "machine-applicable",
            Applicability::HasPlaceholders => "has-placeholders",
            Applicability::MaybeIncorrect => "maybe-incorrect",
            Applicability::Unspecified => "unspecified",
        }
    }
}

/// A collection of suggestions
#[derive(Debug, Clone, Default)]
pub struct Suggestions {
    suggestions: Vec<Suggestion>,
}

impl Suggestions {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
        }
    }
    
    /// Add a suggestion
    pub fn push(&mut self, suggestion: Suggestion) {
        self.suggestions.push(suggestion);
    }
    
    /// Add a code block suggestion
    pub fn code(&mut self, message: impl Into<String>, replacement: impl Into<String>) {
        self.push(Suggestion::code_block(message, replacement));
    }
    
    /// Add an inline suggestion
    pub fn inline(&mut self, message: impl Into<String>, replacement: impl Into<String>) {
        self.push(Suggestion::inline_code(message, replacement));
    }
    
    /// Get all suggestions
    pub fn iter(&self) -> impl Iterator<Item = &Suggestion> {
        self.suggestions.iter()
    }
    
    /// Check if there are any suggestions
    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }
    
    /// Get the number of suggestions
    pub fn len(&self) -> usize {
        self.suggestions.len()
    }
}

/// Common suggestion patterns
pub mod patterns {
    use super::*;
    
    /// Suggest adding a missing period
    pub fn add_period(span: Span) -> Suggestion {
        Suggestion::inline_code("Add a period at the end", ".")
            .with_span(span)
    }
    
    /// Suggest removing unused variable
    pub fn remove_unused(name: &str, span: Span) -> Suggestion {
        Suggestion::code_block(
            format!("Remove unused variable '{}'", name),
            "",
        ).with_span(span)
    }
    
    /// Suggest type annotation
    pub fn add_type_annotation(name: &str, ty: &str, span: Span) -> Suggestion {
        Suggestion::code_block(
            "Add explicit type annotation",
            format!("Keep the number 0 in mind as the {}: {} = 0;", name, ty),
        ).with_span(span)
    }
    
    /// Suggest using correct keyword
    pub fn use_keyword(wrong: &str, correct: &str, span: Span) -> Suggestion {
        Suggestion::inline_code(
            format!("Use '{}' instead of '{}'", correct, wrong),
            correct.to_string(),
        ).with_span(span)
    }
    
    /// Suggest declaring variable first
    pub fn declare_variable(name: &str, span: Span) -> Suggestion {
        Suggestion::code_block(
            format!("Declare '{}' before using it", name),
            format!("Keep the number 0 in mind as the {}.", name),
        ).with_span(span)
    }
    
    /// Suggest matching types
    pub fn match_types(expected: &str, found: &str, span: Span) -> Suggestion {
        Suggestion::plain(format!(
            "Expected type '{}', found '{}'. Consider converting or using compatible types.",
            expected, found
        )).with_span(span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_suggestion_creation() {
        let sugg = Suggestion::code_block("Fix this", "replacement");
        assert_eq!(sugg.style, SuggestionStyle::CodeBlock);
        assert_eq!(sugg.replacement, Some("replacement".to_string()));
    }
    
    #[test]
    fn test_suggestions_collection() {
        let mut suggestions = Suggestions::new();
        suggestions.code("Fix", "fixed");
        suggestions.inline("Change", "changed");
        
        assert_eq!(suggestions.len(), 2);
        assert!(!suggestions.is_empty());
    }
    
    #[test]
    fn test_suggestion_patterns() {
        let span = Span::new(0, 10);
        let sugg = patterns::add_period(span);
        
        assert_eq!(sugg.style, SuggestionStyle::InlineCode);
        assert_eq!(sugg.replacement, Some(".".to_string()));
    }
}
