//! Diagnostic Rendering
//! 
//! Renders diagnostics for terminal and other output formats.

use super::diagnostic::{Diagnostic, DiagnosticLevel, DiagnosticLabel, Span, Suggestion, SuggestionStyle};

/// Diagnostic renderer trait
pub trait DiagnosticRenderer {
    fn render(&self, diagnostic: &Diagnostic, source: &str) -> String;
}

/// Terminal renderer with ANSI colors
pub struct TerminalRenderer {
    use_colors: bool,
    tab_width: usize,
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self {
            use_colors: true,
            tab_width: 4,
        }
    }
    
    pub fn without_colors() -> Self {
        Self {
            use_colors: false,
            tab_width: 4,
        }
    }
    
    pub fn with_tab_width(mut self, width: usize) -> Self {
        self.tab_width = width;
        self
    }
    
    /// Reset ANSI color
    fn reset(&self) -> &'static str {
        if self.use_colors { "\x1b[0m" } else { "" }
    }
    
    /// Get color code for level
    fn level_color(&self, level: DiagnosticLevel) -> &'static str {
        if !self.use_colors {
            return "";
        }
        level.color_code()
    }
    
    /// Format a span label
    fn format_label(&self, label: &DiagnosticLabel, source: &str) -> String {
        let mut output = String::new();
        
        if let Some(source) = source.get(label.span.start..label.span.end.min(source.len())) {
            output.push_str(&format!("  | {}\n", source));
        }
        
        if let Some(text) = &label.label {
            let marker = if label.is_primary { "^^^" } else { "---" };
            output.push_str(&format!("  | {} {}\n", marker, text));
        }
        
        output
    }
    
    /// Format source snippet with line numbers
    fn format_snippet(&self, source: &str, span: Span) -> String {
        let mut output = String::new();
        
        let (start_line, start_col) = self.position_to_line_col(source, span.start);
        let (end_line, end_col) = self.position_to_line_col(source, span.end.min(source.len()));
        
        let lines: Vec<&str> = source.lines().collect();
        
        // Show context: 2 lines before and after
        let first_line = start_line.saturating_sub(2);
        let last_line = (end_line + 3).min(lines.len());
        
        // Calculate line number width
        let line_width = format!("{}", last_line + 1).len();
        
        for (i, line) in lines.iter().enumerate().skip(first_line).take(last_line - first_line) {
            let line_num = i + 1;
            
            // Line number
            output.push_str(&format!("{:width$} | ", line_num, width = line_width));
            
            // Line content (truncate long lines)
            let display_line = if line.len() > 120 {
                line[..117].to_string() + "..."
            } else {
                line.to_string()
            };
            output.push_str(&format!("{}\n", display_line));
            
            // Underline for error span
            if i >= start_line && i <= end_line {
                output.push_str(&format!("{:width$} | ", "", width = line_width));
                
                let start_pos = if i == start_line { start_col } else { 0 };
                let end_pos = if i == end_line { end_col } else { line.len().min(120) };
                
                if end_pos > start_pos {
                    let underline = "^".repeat((end_pos - start_pos).max(1));
                    output.push_str(&format!("{}{}\n", " ".repeat(start_pos), underline));
                }
            }
        }
        
        output
    }
    
    /// Convert byte position to line/column
    fn position_to_line_col(&self, source: &str, pos: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;
        let mut current = 0;
        
        for ch in source.chars() {
            if current >= pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            current += 1;
        }
        
        (line, col)
    }
    
    /// Format a suggestion
    fn format_suggestion(&self, suggestion: &Suggestion) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("help: {}\n", suggestion.message));
        
        if let Some(replacement) = &suggestion.replacement {
            match suggestion.style {
                SuggestionStyle::InlineCode => {
                    output.push_str(&format!("  `{}`\n", replacement));
                }
                SuggestionStyle::CodeBlock => {
                    output.push_str(&format!("  ```\n  {}\n  ```\n", replacement));
                }
                SuggestionStyle::Command => {
                    output.push_str(&format!("  $ {}\n", replacement));
                }
                SuggestionStyle::Plain => {
                    output.push_str(&format!("  {}\n", replacement));
                }
            }
        }
        
        output
    }
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticRenderer for TerminalRenderer {
    fn render(&self, diagnostic: &Diagnostic, source: &str) -> String {
        let mut output = String::new();
        
        // Header: level and message
        output.push_str(&format!(
            "{}{}{}: {}",
            self.level_color(diagnostic.level),
            diagnostic.level,
            self.reset(),
            diagnostic.message
        ));
        
        // Error code
        if let Some(code) = &diagnostic.code {
            output.push_str(&format!(" [{}]", code));
        }
        output.push('\n');
        
        // Source location
        if let Some(span) = diagnostic.span {
            let file = diagnostic.file.as_deref().unwrap_or("<input>");
            let (line, col) = self.position_to_line_col(source, span.start);
            output.push_str(&format!(" --> {}:{}:{}\n", file, line + 1, col + 1));
        }
        
        // Source snippet
        if let Some(source) = &diagnostic.source {
            if let Some(span) = diagnostic.span {
                output.push_str(&self.format_snippet(source, span));
            }
        }
        
        // Labels
        for label in &diagnostic.labels {
            if let Some(source) = &diagnostic.source {
                output.push_str(&self.format_label(label, source));
            }
        }
        
        // Children (notes, help)
        for child in &diagnostic.children {
            let color = self.level_color(child.level);
            output.push_str(&format!("  {}{}{}: {}\n", color, child.level, self.reset(), child.message));
            
            if let Some(span) = child.span {
                if let Some(source) = &diagnostic.source {
                    output.push_str(&self.format_snippet(source, span));
                }
            }
        }
        
        // Suggestions
        for suggestion in &diagnostic.suggestions {
            output.push_str(&self.format_suggestion(suggestion));
        }
        
        output
    }
}

/// JSON renderer for machine-readable output
pub struct JsonRenderer;

impl JsonRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticRenderer for JsonRenderer {
    fn render(&self, diagnostic: &Diagnostic, _source: &str) -> String {
        let mut json = String::from("{\n");
        
        json.push_str(&format!("  \"level\": \"{}\",\n", diagnostic.level.as_str()));
        json.push_str(&format!("  \"message\": \"{}\",\n", escape_json(&diagnostic.message)));
        
        if let Some(code) = &diagnostic.code {
            json.push_str(&format!("  \"code\": \"{}\",\n", code));
        }
        
        if let Some(span) = diagnostic.span {
            json.push_str(&format!("  \"span\": {{\"start\": {}, \"end\": {}}},\n", span.start, span.end));
        }
        
        if let Some(file) = &diagnostic.file {
            json.push_str(&format!("  \"file\": \"{}\",\n", escape_json(file)));
        }
        
        json.push_str("  \"labels\": [\n");
        for (i, label) in diagnostic.labels.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str(&format!("    {{\"span\": {{\"start\": {}, \"end\": {}}}, \"label\": {:?}, \"primary\": {}}}",
                label.span.start, label.span.end, label.label, label.is_primary));
        }
        json.push_str("\n  ],\n");
        
        json.push_str("  \"children\": [\n");
        for (i, child) in diagnostic.children.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str(&format!("    {{\"level\": \"{}\", \"message\": \"{}\"}}",
                child.level.as_str(), escape_json(&child.message)));
        }
        json.push_str("\n  ]\n");
        
        json.push_str("}");
        json
    }
}

/// Escape special characters for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Render diagnostics to HTML
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn render_multiple(diagnostics: &[Diagnostic], _source: &str) -> String {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<style>\n");
        html.push_str(".error { color: red; }\n");
        html.push_str(".warning { color: orange; }\n");
        html.push_str(".note { color: blue; }\n");
        html.push_str(".help { color: green; }\n");
        html.push_str("pre { background: #f4f4f4; padding: 10px; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        
        for diag in diagnostics {
            html.push_str(&format!("<div class=\"{}\">\n", diag.level.as_str()));
            html.push_str(&format!("<strong>{}</strong>: {}\n", diag.level, diag.message));
            
            if let Some(code) = &diag.code {
                html.push_str(&format!(" <code>[{}]</code>", code));
            }
            
            if let Some(source) = &diag.source {
                html.push_str("<pre>");
                html.push_str(&escape_html(source));
                html.push_str("</pre>");
            }
            
            html.push_str("</div>\n");
        }
        
        html.push_str("</body>\n</html>");
        html
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::diagnostic::Diagnostic;
    
    #[test]
    fn test_terminal_renderer() {
        let renderer = TerminalRenderer::new();
        let diag = Diagnostic::error()
            .with_message("Test error")
            .with_code("E0001");
        
        let output = renderer.render(&diag, "");
        assert!(output.contains("error"));
        assert!(output.contains("Test error"));
        assert!(output.contains("E0001"));
    }
    
    #[test]
    fn test_json_renderer() {
        let renderer = JsonRenderer::new();
        let diag = Diagnostic::warning()
            .with_message("Test warning");
        
        let output = renderer.render(&diag, "");
        assert!(output.contains("\"level\": \"warning\""));
        assert!(output.contains("\"message\": \"Test warning\""));
    }
    
    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
    }
}
