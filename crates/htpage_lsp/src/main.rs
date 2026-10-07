use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use htpage_core::ast::{get_tag_attrs, BOOLEAN_ATTRS, GLOBAL_ATTRS, HTML_ONLY_ATTRS, Node};
use htpage_core::parser::{KNOWN_HTML_TAGS, PSEUDO_CLASSES};
use htpage_core::span::LineIndex;
use htpage_core::{parse_with_diagnostics, DiagnosticSeverity, ParseOutput};

struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn on_change(&self, uri: Url, text: String) {
        let parse_output = parse_with_diagnostics(&text);
        let lsp_diagnostics = self.convert_diagnostics(&parse_output);
        self.documents.write().await.insert(uri.clone(), text);
        self.client.publish_diagnostics(uri, lsp_diagnostics, None).await;
    }

    fn convert_diagnostics(&self, output: &ParseOutput) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        output
            .diagnostics
            .iter()
            .map(|d| {
                let (start_line, start_col) = output.line_index.line_col(d.span.start);
                let (end_line, end_col) = output.line_index.line_col(d.span.end);
                let range = Range {
                    start: Position {
                        line: start_line as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: end_line as u32,
                        character: end_col as u32,
                    },
                };
                let severity = match d.severity {
                    DiagnosticSeverity::Error => tower_lsp::lsp_types::DiagnosticSeverity::ERROR,
                    DiagnosticSeverity::Warning => tower_lsp::lsp_types::DiagnosticSeverity::WARNING,
                    DiagnosticSeverity::Information => tower_lsp::lsp_types::DiagnosticSeverity::INFORMATION,
                    DiagnosticSeverity::Hint => tower_lsp::lsp_types::DiagnosticSeverity::HINT,
                };
                tower_lsp::lsp_types::Diagnostic {
                    range,
                    severity: Some(severity),
                    code: None,
                    code_description: None,
                    source: Some("htpage".to_string()),
                    message: d.message.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![":".to_string(), " ".to_string(), "{".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "htpage-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "htpage language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().await.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let docs = self.documents.read().await;
        let text = match docs.get(&uri) {
            Some(t) => t,
            None => return Ok(None),
        };

        let output = parse_with_diagnostics(text);
        let offset = match output.line_index.offset(pos.line as usize, pos.character as usize) {
            Some(o) => o,
            None => 0,
        };

        let mut items = Vec::new();

        // 1. Tag name completions
        for tag in KNOWN_HTML_TAGS {
            items.push(CompletionItem {
                label: tag.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(format!("HTML tag <{tag}>")),
                documentation: Some(Documentation::String(format!("htpage element block for <{tag}>"))),
                insert_text: Some(format!("{tag} {{ $0 }}")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // 2. Global HTML attributes
        for attr in GLOBAL_ATTRS {
            items.push(CompletionItem {
                label: attr.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Global HTML attribute".to_string()),
                insert_text: Some(format!("{attr}: ")),
                ..Default::default()
            });
        }

        // 3. Boolean attributes
        for b in BOOLEAN_ATTRS {
            items.push(CompletionItem {
                label: b.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Boolean HTML attribute".to_string()),
                insert_text: Some(format!("{b}: true")),
                ..Default::default()
            });
        }

        // 4. Pseudo-classes
        for pseudo in PSEUDO_CLASSES {
            items.push(CompletionItem {
                label: pseudo.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Pseudo-class :{pseudo}")),
                insert_text: Some(format!("{pseudo} {{ $0 }}")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // 5. CSS property suggestions
        let common_css = [
            "color", "background-color", "font-size", "font-weight", "font-family",
            "padding", "margin", "border", "border-radius", "display", "flex",
            "flex-direction", "justify-content", "align-items", "width", "height",
            "max-width", "min-height", "gap", "box-shadow", "opacity", "cursor",
        ];
        for css in common_css {
            items.push(CompletionItem {
                label: css.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("CSS property (auto-bundled into style)".to_string()),
                insert_text: Some(format!("{css}: ")),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let text = match docs.get(&uri) {
            Some(t) => t,
            None => return Ok(None),
        };

        let output = parse_with_diagnostics(text);
        let offset = match output.line_index.offset(pos.line as usize, pos.character as usize) {
            Some(o) => o,
            None => return Ok(None),
        };

        // Find matching node at cursor
        for node in &output.ast {
            if let Some(hover) = find_hover_in_node(node, offset) {
                return Ok(Some(hover));
            }
        }

        Ok(None)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let docs = self.documents.read().await;
        let text = match docs.get(&uri) {
            Some(t) => t,
            None => return Ok(None),
        };

        let output = parse_with_diagnostics(text);
        let mut symbols = Vec::new();

        for node in &output.ast {
            if let Some(sym) = node_to_symbol(node, &output.line_index) {
                symbols.push(sym);
            }
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }
}

fn find_hover_in_node(node: &Node, offset: usize) -> Option<Hover> {
    match node {
        Node::Element(el) => {
            // Check tag name
            if el.name_span.contains(offset) {
                let markdown = format!("### `<{}>` (htpage element)\n\nCompiled into standards-compliant HTML5.", el.name);
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: markdown,
                    }),
                    range: None,
                });
            }

            // Check attributes
            for attr in &el.attrs {
                if attr.key_span.contains(offset) {
                    let is_html = GLOBAL_ATTRS.contains(&attr.key.as_str())
                        || get_tag_attrs(&el.name).contains(&attr.key.as_str());
                    let desc = if is_html {
                        format!("**HTML Attribute**: `{}` on `<{}>`", attr.key, el.name)
                    } else if HTML_ONLY_ATTRS.contains(&attr.key.as_str()) {
                        format!("**Invalid HTML Attribute**: `{}` cannot be used as CSS", attr.key)
                    } else {
                        format!("**CSS Property**: `{}` (bundled into inline `style`)", attr.key)
                    };
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: desc,
                        }),
                        range: None,
                    });
                }
            }

            // Check children
            for child in &el.children {
                if let Some(h) = find_hover_in_node(child, offset) {
                    return Some(h);
                }
            }
        }
        Node::Text(_) => {}
    }
    None
}

fn node_to_symbol(node: &Node, line_index: &LineIndex) -> Option<DocumentSymbol> {
    match node {
        Node::Element(el) => {
            let (start_line, start_col) = line_index.line_col(el.span.start);
            let (end_line, end_col) = line_index.line_col(el.span.end);
            let (name_s_line, name_s_col) = line_index.line_col(el.name_span.start);
            let (name_e_line, name_e_col) = line_index.line_col(el.name_span.end);

            let range = Range {
                start: Position {
                    line: start_line as u32,
                    character: start_col as u32,
                },
                end: Position {
                    line: end_line as u32,
                    character: end_col as u32,
                },
            };
            let selection_range = Range {
                start: Position {
                    line: name_s_line as u32,
                    character: name_s_col as u32,
                },
                end: Position {
                    line: name_e_line as u32,
                    character: name_e_col as u32,
                },
            };

            let mut children = Vec::new();
            for c in &el.children {
                if let Some(cs) = node_to_symbol(c, line_index) {
                    children.push(cs);
                }
            }

            #[allow(deprecated)]
            Some(DocumentSymbol {
                name: el.name.clone(),
                detail: None,
                kind: SymbolKind::FIELD,
                tags: None,
                deprecated: None,
                range,
                selection_range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }
        Node::Text(_) => None,
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
