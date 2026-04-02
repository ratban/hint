//! Language Server Protocol implementation for Hint.

use lsp_types::*;
use crate::parser::{parse, AstNode};
use crate::semantics::{HintType, IntSize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use serde_json::{json, Value};

/// The Hint Language Server
pub struct HintLanguageServer {
    documents: HashMap<String, String>,
    variables: HashMap<String, Vec<(String, u32, HintType)>>,
}

impl HintLanguageServer {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn get_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![".".to_string(), "\"".to_string()]),
                work_done_progress_options: Default::default(),
                all_commit_characters: None,
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                DiagnosticOptions {
                    identifier: Some("hint".to_string()),
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                    work_done_progress_options: Default::default(),
                }
            )),
            ..ServerCapabilities::default()
        }
    }

    pub fn on_did_open(&mut self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let content = params.text_document.text;
        self.variables.insert(uri.clone(), self.extract_variables(&content));
        self.documents.insert(uri, content);
    }

    pub fn on_did_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some(current_content) = self.documents.get(&uri).cloned() {
            let mut new_content = current_content;

            for change in params.content_changes {
                if let Some(range) = change.range {
                    // Apply incremental change using line-based approach
                    let mut lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
                    
                    let start_line = range.start.line as usize;
                    let end_line = range.end.line as usize;
                    let start_col = range.start.character as usize;
                    let end_col = range.end.character as usize;

                    if start_line < lines.len() && end_line <= lines.len() {
                        if start_line == end_line {
                            // Single line change
                            let line = &mut lines[start_line];
                            let chars: Vec<char> = line.chars().collect();
                            if start_col <= chars.len() && end_col <= chars.len() {
                                let mut new_line: String = chars[..start_col].iter().collect();
                                new_line.push_str(&change.text);
                                new_line.push_str(&chars[end_col..].iter().collect::<String>());
                                lines[start_line] = new_line;
                            }
                        } else {
                            // Multi-line change
                            let first_line = &lines[start_line];
                            let last_line = &lines[end_line];
                            let first_chars: Vec<char> = first_line.chars().collect();
                            let last_chars: Vec<char> = last_line.chars().collect();
                            
                            let prefix: String = first_chars[..start_col.min(first_chars.len())].iter().collect();
                            let suffix: String = last_chars[end_col.min(last_chars.len())..].iter().collect();
                            
                            let mut new_lines = Vec::new();
                            new_lines.push(prefix + &change.text + &suffix);
                            
                            // Add any lines between start and end that should be preserved
                            for (_i, line) in lines.iter().enumerate().take(end_line).skip(start_line + 1) {
                                new_lines.push(line.clone());
                            }
                            new_lines.push(suffix);
                            
                            lines.splice(start_line..=end_line, new_lines);
                        }
                        new_content = lines.join("\n");
                    } else {
                        // Fallback: replace entire content
                        new_content = change.text;
                    }
                } else {
                    // Full document change
                    new_content = change.text;
                }
            }

            self.variables.insert(uri.clone(), self.extract_variables(&new_content));
            self.documents.insert(uri.clone(), new_content);
        }
    }

    pub fn on_did_close(&mut self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
        self.variables.remove(&uri);
    }

    pub fn on_hover(&self, params: HoverParams) -> Option<Hover> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let content = self.documents.get(&uri)?;
        let hover_text = self.get_hover_at_position(content, position)?;
        
        Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_text)),
            range: None,
        })
    }

    pub fn on_definition(&self, params: GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let content = self.documents.get(&uri)?;
        let var_name = self.get_word_at_position(content, position)?;

        if let Some(vars) = self.variables.get(&uri) {
            for (name, line, _) in vars {
                if name == &var_name {
                    let line_content = content.lines().nth(*line as usize).unwrap_or("");
                    let line_len = line_content.len() as u32;
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: params.text_document_position_params.text_document.uri,
                        range: Range {
                            start: Position { line: *line, character: 0 },
                            end: Position { line: *line, character: line_len },
                        },
                    }));
                }
            }
        }
        None
    }

    pub fn on_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let mut completions = Vec::new();
        
        // Statement completions
        completions.push(CompletionItem {
            label: "Say".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Output text to console".to_string()),
            insert_text: Some("Say \"\".".to_string()),
            ..CompletionItem::default()
        });
        
        completions.push(CompletionItem {
            label: "Keep".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Store a number in memory".to_string()),
            insert_text: Some("Keep the number 0 in mind as the name.".to_string()),
            ..CompletionItem::default()
        });
        
        completions.push(CompletionItem {
            label: "Stop".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Terminate the program".to_string()),
            insert_text: Some("Stop the program.".to_string()),
            ..CompletionItem::default()
        });
        
        // Stdlib completions (simplified for now)
        // In a full implementation, we'd get these from the stdlib registry
        
        // Variable completions
        if let Some(vars) = self.variables.get(&uri) {
            for (name, _, ty) in vars {
                completions.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some(format!("{}", ty)),
                    ..CompletionItem::default()
                });
            }
        }

        Some(completions)
    }

    pub fn on_document_symbol(&self, params: DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        let uri = params.text_document.uri.to_string();
        let content = self.documents.get(&uri)?;
        let mut symbols = Vec::new();

        if let Ok(program) = parse(content) {
            // Track actual line positions by scanning source
            let lines: Vec<&str> = content.lines().collect();
            let mut current_line = 0u32;

            for stmt in &program.statements {
                // Find the line where this statement appears
                // Simplified: increment line for each statement
                match stmt {
                    AstNode::Speak(text) => {
                        let line_len = lines.get(current_line as usize).map(|s| s.len() as u32).unwrap_or(100);
                        symbols.push(DocumentSymbol {
                            name: format!("Say \"{}\"", text.chars().take(30).collect::<String>()),
                            detail: Some("Output statement".to_string()),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: Some(false),
                            range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: line_len },
                            },
                            selection_range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: 3 },
                            },
                            children: None,
                        });
                    }
                    AstNode::Remember { name, value } => {
                        let line_len = lines.get(current_line as usize).map(|s| s.len() as u32).unwrap_or(100);
                        symbols.push(DocumentSymbol {
                            name: name.clone(),
                            detail: Some(format!("Variable = {}", value)),
                            kind: SymbolKind::VARIABLE,
                            tags: None,
                            deprecated: Some(false),
                            range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: line_len },
                            },
                            selection_range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: 4 },
                            },
                            children: None,
                        });
                    }
                    AstNode::RememberList { name, values } => {
                        let line_len = lines.get(current_line as usize).map(|s| s.len() as u32).unwrap_or(100);
                        symbols.push(DocumentSymbol {
                            name: format!("{} (list)", name),
                            detail: Some(format!("List with {} items", values.len())),
                            kind: SymbolKind::VARIABLE,
                            tags: None,
                            deprecated: Some(false),
                            range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: line_len },
                            },
                            selection_range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: 4 },
                            },
                            children: None,
                        });
                    }
                    AstNode::Halt => {
                        let line_len = lines.get(current_line as usize).map(|s| s.len() as u32).unwrap_or(100);
                        symbols.push(DocumentSymbol {
                            name: "Stop".to_string(),
                            detail: Some("Program termination".to_string()),
                            kind: SymbolKind::OPERATOR,
                            tags: None,
                            deprecated: Some(false),
                            range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: line_len },
                            },
                            selection_range: Range {
                                start: Position { line: current_line, character: 0 },
                                end: Position { line: current_line, character: 4 },
                            },
                            children: None,
                        });
                    }
                }
                current_line += 1;
            }
        }

        Some(DocumentSymbolResponse::Nested(symbols))
    }

    fn extract_variables(&self, content: &str) -> Vec<(String, u32, HintType)> {
        let mut vars = Vec::new();
        if let Ok(program) = parse(content) {
            for (line_num, stmt) in program.statements.iter().enumerate() {
                match stmt {
                    AstNode::Remember { name, .. } => {
                        vars.push((name.clone(), line_num as u32, HintType::Int(IntSize::I64)));
                    }
                    AstNode::RememberList { name, values } => {
                        // Use Array type for lists
                        vars.push((name.clone(), line_num as u32, HintType::Array(Box::new(HintType::Int(IntSize::I64)), values.len())));
                    }
                    _ => {}
                }
            }
        }
        vars
    }

    fn get_hover_at_position(&self, content: &str, position: Position) -> Option<String> {
        let word = self.get_word_at_position(content, position)?;
        
        match word.to_lowercase().as_str() {
            "say" => Some("**Say** statement\n\nOutputs text to the console.\n\n```hint\nSay \"Hello, world!\".\n```".to_string()),
            "keep" => Some("**Keep** statement\n\nStores a number in memory with a named variable.\n\n```hint\nKeep the number 42 in mind as the answer.\n```".to_string()),
            "stop" => Some("**Stop** statement\n\nTerminates the program execution.\n\n```hint\nStop the program.\n```".to_string()),
            _ => {
                if let Ok(program) = parse(content) {
                    for stmt in &program.statements {
                        if let AstNode::Remember { name, value, .. } = stmt {
                            if name == &word {
                                return Some(format!("**Variable**: `{}`\n\nValue: `{}`\n\nType: `i64`", name, value));
                            }
                        }
                    }
                }
                // Check stdlib (simplified for now)
                // In a full implementation, we'd check the stdlib registry
                None
            }
        }
    }

    fn get_word_at_position(&self, content: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let line = lines.get(position.line as usize)?;
        let chars: Vec<char> = line.chars().collect();
        
        let col = position.character as usize;
        if col >= chars.len() {
            return None;
        }
        
        let mut start = col;
        let mut end = col;
        
        while start > 0 && (chars[start - 1].is_alphabetic() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }
}

impl Default for HintLanguageServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the language server
pub fn run_language_server() -> Result<(), String> {
    let mut server = HintLanguageServer::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() || !line.trim().starts_with('{') {
            continue;
        }
        
        if let Ok(response) = handle_json_rpc(&line, &mut server) {
            let output = format!("Content-Length: {}\r\n\r\n{}", response.len(), response);
            stdout.write_all(output.as_bytes()).map_err(|e| e.to_string())?;
            stdout.flush().map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

fn handle_json_rpc(content: &str, server: &mut HintLanguageServer) -> Result<String, String> {
    let value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let obj = value.as_object().ok_or("Expected JSON object")?;

    let method = obj.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let id = obj.get("id").cloned();
    let params = obj.get("params").cloned().unwrap_or(Value::Null);

    let result = match method {
        "initialize" => {
            let capabilities = server.get_capabilities();
            Ok::<Value, String>(json!({
                "capabilities": capabilities,
                "serverInfo": {
                    "name": "hintc-lsp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }))
        }
        "initialized" | "shutdown" => Ok(Value::Null),
        "exit" => Ok(Value::Null),
        "textDocument/didOpen" => {
            if let Ok(p) = serde_json::from_value::<DidOpenTextDocumentParams>(params.clone()) {
                server.on_did_open(p);
            }
            Ok(Value::Null)
        }
        "textDocument/didChange" => {
            if let Ok(p) = serde_json::from_value::<DidChangeTextDocumentParams>(params.clone()) {
                server.on_did_change(p);
            }
            Ok(Value::Null)
        }
        "textDocument/didClose" => {
            if let Ok(p) = serde_json::from_value::<DidCloseTextDocumentParams>(params.clone()) {
                server.on_did_close(p);
            }
            Ok(Value::Null)
        }
        "textDocument/hover" => {
            if let Ok(p) = serde_json::from_value::<HoverParams>(params.clone()) {
                Ok(server.on_hover(p)
                    .and_then(|h| serde_json::to_value(h).ok())
                    .unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/definition" => {
            if let Ok(p) = serde_json::from_value::<GotoDefinitionParams>(params.clone()) {
                Ok(server.on_definition(p)
                    .and_then(|d| serde_json::to_value(d).ok())
                    .unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/completion" => {
            if let Ok(p) = serde_json::from_value::<CompletionParams>(params.clone()) {
                Ok(server.on_completion(p)
                    .and_then(|c| serde_json::to_value(c).ok())
                    .unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/documentSymbol" => {
            if let Ok(p) = serde_json::from_value::<DocumentSymbolParams>(params.clone()) {
                Ok(server.on_document_symbol(p)
                    .and_then(|s| serde_json::to_value(s).ok())
                    .unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "$/cancelRequest" | "$/setTrace" | "workspace/didChangeConfiguration" => Ok(Value::Null),
        _ => {
            eprintln!("Unknown method: {}", method);
            Ok(Value::Null)
        }
    };

    match (result, id) {
        (Ok(res), Some(msg_id)) => Ok(json!({
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": res
        }).to_string()),
        (Ok(_), None) => Ok(Value::Null.to_string()),
        (Err(e), msg_id) => Ok(json!({
            "jsonrpc": "2.0",
            "id": msg_id,
            "error": {
                "code": -32603,
                "message": e
            }
        }).to_string()),
    }
}
