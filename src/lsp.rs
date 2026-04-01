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
                trigger_characters: Some(vec![".".to_string(), "\"".to_string(), " ".to_string()]),
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
        for change in params.content_changes {
            self.variables.insert(uri.clone(), self.extract_variables(&change.text));
            self.documents.insert(uri.clone(), change.text);
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
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: params.text_document_position_params.text_document.uri,
                        range: Range {
                            start: Position { line: *line, character: 0 },
                            end: Position { line: *line, character: 100 },
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
            let mut line = 0u32;
            for stmt in &program.statements {
                match stmt {
                    AstNode::Speak(text) => {
                        symbols.push(DocumentSymbol {
                            name: format!("Say \"{}\"", text.chars().take(30).collect::<String>()),
                            detail: Some("Output statement".to_string()),
                            kind: SymbolKind::STRING,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 100 },
                            },
                            selection_range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 3 },
                            },
                            children: None,
                        });
                    }
                    AstNode::Remember { name, value } => {
                        symbols.push(DocumentSymbol {
                            name: name.clone(),
                            detail: Some(format!("Variable = {}", value)),
                            kind: SymbolKind::VARIABLE,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 100 },
                            },
                            selection_range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 4 },
                            },
                            children: None,
                        });
                    }
                    AstNode::RememberList { name, values } => {
                        symbols.push(DocumentSymbol {
                            name: format!("{} (list)", name),
                            detail: Some(format!("List with {} items", values.len())),
                            kind: SymbolKind::VARIABLE,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 100 },
                            },
                            selection_range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 4 },
                            },
                            children: None,
                        });
                    }
                    AstNode::Halt => {
                        symbols.push(DocumentSymbol {
                            name: "Stop".to_string(),
                            detail: Some("Program termination".to_string()),
                            kind: SymbolKind::OPERATOR,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 100 },
                            },
                            selection_range: Range {
                                start: Position { line, character: 0 },
                                end: Position { line, character: 4 },
                            },
                            children: None,
                        });
                    }
                }
                line += 1;
            }
        }
        
        Some(DocumentSymbolResponse::Nested(symbols))
    }

    fn extract_variables(&self, content: &str) -> Vec<(String, u32, HintType)> {
        let mut vars = Vec::new();
        if let Ok(program) = parse(content) {
            for (line_num, stmt) in program.statements.iter().enumerate() {
                if let AstNode::Remember { name, value: _value, .. } = stmt {
                    vars.push((name.clone(), line_num as u32, HintType::Int(IntSize::I64)));
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
                Ok(server.on_hover(p).map(|h| serde_json::to_value(h).unwrap()).unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/definition" => {
            if let Ok(p) = serde_json::from_value::<GotoDefinitionParams>(params.clone()) {
                Ok(server.on_definition(p).map(|d| serde_json::to_value(d).unwrap()).unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/completion" => {
            if let Ok(p) = serde_json::from_value::<CompletionParams>(params.clone()) {
                Ok(server.on_completion(p).map(|c| serde_json::to_value(c).unwrap()).unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/documentSymbol" => {
            if let Ok(p) = serde_json::from_value::<DocumentSymbolParams>(params.clone()) {
                Ok(server.on_document_symbol(p).map(|s| serde_json::to_value(s).unwrap()).unwrap_or(Value::Null))
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
