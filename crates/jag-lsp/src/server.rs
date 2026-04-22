use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use jag_core::types::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::time::{Instant, Duration};
use async_trait::async_trait;
use jag_models::router::{ModelRouter, ModelInput, ModelRouting};
use jag_core::types::ModelPreference;
use crate::index::{SharedSymbolIndex, SymbolIndex, Symbol};

pub struct JagLanguageServer {
    client: Client,
    workspace: Arc<RwLock<Option<WorkspaceId>>>,
    index: SharedSymbolIndex,
    model_router: Arc<RwLock<ModelRouter>>,
    documents: Arc<RwLock<HashMap<Url, String>>>,
    last_completion_request: Arc<RwLock<Instant>>,
}

impl JagLanguageServer {
    pub fn new(client: Client, model_router: Arc<RwLock<ModelRouter>>) -> Self {
        Self {
            client,
            workspace: Arc::new(RwLock::new(None)),
            index: Arc::new(RwLock::new(SymbolIndex::new())),
            model_router,
            documents: Arc::new(RwLock::new(HashMap::new())),
            last_completion_request: Arc::new(RwLock::new(Instant::now())),
        }
    }
}

#[async_trait]
impl LanguageServer for JagLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Store workspace root if provided
        if let Some(_root_uri) = params.root_uri {
            let mut ws = self.workspace.write().await;
            *ws = Some(WorkspaceId::new());
        }

        // Potential initialization health check for Ollama
        let client = self.client.clone();
        let model_router = self.model_router.clone();
        tokio::spawn(async move {
            let router = model_router.read().await;
            // Since auto_configure() is usually called in main.rs, we just check if it has models
            if router.available_models().is_empty() {
                client.log_message(
                    tower_lsp::lsp_types::MessageType::WARNING,
                    "Ollama not detected or no models available. AI completions will fallback to static suggestions."
                ).await;
            }
        });
        
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(tower_lsp::lsp_types::MessageType::INFO, "Jag LSP server initialized!")
            .await;
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        
        // Update document cache
        self.documents.write().await.insert(uri.clone(), text.clone());

        let symbols = self.extract_symbols(&text, &uri).await;
        self.index.write().await.update_file(
            uri.as_ref(),
            symbols,
        );
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut docs = self.documents.write().await;
        let doc = docs.entry(uri.clone()).or_insert_with(String::new);

        for change in params.content_changes {
            if let Some(range) = change.range {
                // Simplified incremental edit application
                *doc = apply_text_edit(doc, &range, &change.text);
            } else {
                // Full document replacement
                *doc = change.text;
            }
        }

        let current_text = doc.clone();
        drop(docs);

        let symbols = self.extract_symbols(&current_text, &uri).await;
        self.index.write().await.update_file(
            uri.as_ref(),
            symbols,
        );
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // 1. Debounce check
        {
            let mut last_req = self.last_completion_request.write().await;
            if last_req.elapsed() < Duration::from_millis(200) {
                return Ok(Some(CompletionResponse::Array(vec![])));
            }
            *last_req = Instant::now();
        }

        // 2. Immediate static fallback
        let fallback_items = self.get_static_completions(&params);
        
        // 3. Background AI generation
        let uri = params.text_document_position.text_document.uri.clone();
        let position = params.text_document_position.position;
        let documents = self.documents.clone();
        let model_router = self.model_router.clone();

        tokio::spawn(async move {
            let docs = documents.read().await;
            if let Some(content) = docs.get(&uri) {
                let context = extract_fim_context(content, position);
                let prompt = build_fim_prompt(&context.prefix, &context.suffix);
                
                let router = model_router.read().await;
                match router.generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration).await {
                    Ok(suggestion) => {
                        tracing::info!("AI completion generated: {:?}", suggestion);
                        // Future: Push AI completion via completion/resolve or pushDiagnostics
                    }
                    Err(e) => {
                        tracing::warn!("AI completion failed: {}", e);
                    }
                }
            }
        });

        Ok(Some(CompletionResponse::Array(fallback_items)))
    }
    
    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // Placeholder: return generic hover text
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "Jag IDE: AI-powered development environment".to_string(),
            )),
            range: None,
        }))
    }
    
    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        
        let index = self.index.read().await;
        if let Some(symbol) = index.find_at_position(&uri, position) {
            Ok(Some(GotoDefinitionResponse::Scalar(symbol.location.clone())))
        } else {
            Ok(None)
        }
    }
    
    async fn references(&self, _params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        Ok(None)
    }
    
    async fn rename(&self, _params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        Ok(None)
    }
    
    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let index = self.index.read().await;
        let uri_str = params.text_document.uri.to_string();
        if let Some(symbols) = index.get_file_symbols(&uri_str) {
            let result = symbols.iter().map(|s| DocumentSymbol {
                name: s.name.clone(),
                detail: None,
                kind: s.kind,
                tags: None,
                #[allow(deprecated)]
                deprecated: None,
                range: s.location.range,
                selection_range: s.location.range,
                children: None,
            }).collect();
            Ok(Some(DocumentSymbolResponse::Nested(result)))
        } else {
            Ok(None)
        }
    }
    
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        let index = self.index.read().await;
        // In real rust we have to explicitly reach into `index` field which is global_index
        // but `find_symbol` does exact matches. Let's do a direct scan for baseline.
        // We will borrow `files` map which stores `Symbol` structs.
        // Fallback naive search implementation (until we expose `.global_index` cleanly in next iteration)
        let query = params.query;
        // We'll extract this logic in a helper loop over `get_file_symbols` later if needed.
        // For now, let's return None as the naive global_search needs a full iteration.
        // Let's implement a quick naive scan:
        // Note: index.global_index is private. Let's use `find_symbol` for exact match for now.
        // If query is empty, maybe return all.
        // To do this right we really need `index.rs` to expose a `search_symbols(query)`.
        // I will return `None` here and we can add `search_symbols` to index.rs if needed.
        if let Some(locations) = index.find_symbol(&query) {
             let syms = locations.iter().map(|loc| SymbolInformation {
                 name: query.clone(),
                 kind: SymbolKind::FUNCTION,
                 tags: None,
                 #[allow(deprecated)]
                 deprecated: None,
                 location: loc.clone(),
                 container_name: None,
             }).collect();
             return Ok(Some(syms));
        }
        
        Ok(None)
    }
}

impl JagLanguageServer {
    fn get_static_completions(&self, _params: &CompletionParams) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "console.log".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Log to console (Static)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "fetch".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Make HTTP request (Static)".to_string()),
                ..Default::default()
            },
        ]
    }

    async fn extract_symbols(&self, text: &str, uri: &Url) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(start) = line.find("fn ").or_else(|| line.find("function "))
                && let Some(name_start) = line[start..].find(|c: char| c.is_alphabetic())
            {
                let name_end = line[start + name_start..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(line.len() - start - name_start);
                
                let name = line[start + name_start..start + name_start + name_end].to_string();
                
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::FUNCTION,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                    },
                    container_name: None,
                });
            }
        }
        
        symbols
    }
}

pub async fn start_lsp_server(model_router: Arc<RwLock<ModelRouter>>) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::build(|client| JagLanguageServer::new(client, model_router.clone()))
        .custom_method("jag/health", JagLanguageServer::health_check)
        .finish();
        
    Server::new(stdin, stdout, socket).serve(service).await;
}

impl JagLanguageServer {
    async fn health_check(&self, _: ()) -> Result<serde_json::Value> {
        Ok(serde_json::json!({ "status": "ok" }))
    }
}

struct FimContext {
    prefix: String,
    suffix: String,
}

fn extract_fim_context(content: &str, position: Position) -> FimContext {
    let lines: Vec<&str> = content.lines().collect();
    let row = position.line as usize;
    let col = position.character as usize;

    let mut prefix = Vec::new();
    for (_i, line) in lines.iter().enumerate().take(row) {
        prefix.push(*line);
    }
    if let Some(current_line) = lines.get(row) {
        prefix.push(&current_line[..col.min(current_line.len())]);
    }

    let mut suffix = Vec::new();
    if let Some(current_line) = lines.get(row) {
        suffix.push(&current_line[col.min(current_line.len())..]);
    }
    for line in lines.iter().skip(row + 1) {
        suffix.push(*line);
    }

    FimContext {
        prefix: prefix.join("\n"),
        suffix: suffix.join("\n"),
    }
}

fn build_fim_prompt(prefix: &str, suffix: &str) -> String {
    format!(
        "<|fim_prefix|>{}<|fim_suffix|>{}<|fim_middle|>",
        prefix, suffix
    )
}

fn apply_text_edit(doc: &str, range: &Range, new_text: &str) -> String {
    let lines: Vec<&str> = doc.lines().collect();
    let mut result = String::new();

    let start_line = range.start.line as usize;
    let start_char = range.start.character as usize;
    let end_line = range.end.line as usize;
    let end_char = range.end.character as usize;

    for (i, line) in lines.iter().enumerate() {
        if i < start_line {
            result.push_str(line);
            result.push('\n');
        } else if i == start_line {
            result.push_str(&line[..start_char.min(line.len())]);
            if start_line == end_line {
                result.push_str(new_text);
                result.push_str(&line[end_char.min(line.len())..]);
                result.push('\n');
            } else {
                result.push_str(new_text);
            }
        } else if i > start_line && i < end_line {
            // Deleted lines
        } else if i == end_line {
            result.push_str(&line[end_char.min(line.len())..]);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}
