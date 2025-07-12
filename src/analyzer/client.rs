use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

use crate::analyzer::protocol::*;

const RUST_ANALYZER_PATH: &str = "/Users/dex/.cargo/bin/rust-analyzer";

pub struct RustAnalyzerClient {
    process: Option<Child>,
    request_id: u64,
    initialized: bool,
}

impl RustAnalyzerClient {
    pub fn new() -> Self {
        Self {
            process: None,
            request_id: 0,
            initialized: false,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let child = tokio::process::Command::new(RUST_ANALYZER_PATH)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);
        self.initialize().await?;
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        // Get current working directory
        let current_dir = std::env::current_dir()?;
        let root_uri = format!("file://{}", current_dir.display());
        
        // Send initialize request
        let init_params = json!({
            "processId": null,
            "clientInfo": {
                "name": "rust-mcp-server",
                "version": "0.1.0"
            },
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "definition": {
                        "dynamicRegistration": false
                    },
                    "references": {
                        "dynamicRegistration": false
                    },
                    "publishDiagnostics": {
                        "relatedInformation": true
                    }
                },
                "workspace": {
                    "symbol": {
                        "dynamicRegistration": false
                    }
                }
            }
        });

        let _response = self.send_request_internal("initialize", init_params).await?;
        
        // Send initialized notification
        self.send_notification("initialized", json!({})).await?;
        
        self.initialized = true;
        Ok(())
    }

    async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&notification).await
    }

    async fn send_request_internal(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        self.send_message(&request).await?;
        self.read_response(self.request_id).await
    }

    async fn send_message(&mut self, message: &Value) -> Result<()> {
        let content = message.to_string();
        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        
        if let Some(child) = &mut self.process {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(header.as_bytes()).await?;
                stdin.write_all(content.as_bytes()).await?;
                stdin.flush().await?;
            }
        }
        
        Ok(())
    }

    async fn read_response(&mut self, expected_id: u64) -> Result<Value> {
        if let Some(child) = &mut self.process {
            if let Some(stdout) = child.stdout.as_mut() {
                let mut reader = BufReader::new(stdout);
                
                loop {
                    // Read headers
                    let mut content_length: Option<usize> = None;
                    loop {
                        let mut line = String::new();
                        reader.read_line(&mut line).await?;
                        
                        if line == "\r\n" {
                            break;
                        }
                        
                        if line.starts_with("Content-Length:") {
                            let length_str = line["Content-Length:".len()..].trim();
                            content_length = Some(length_str.parse()?);
                        }
                    }
                    
                    if let Some(length) = content_length {
                        let mut content = vec![0u8; length];
                        reader.read_exact(&mut content).await?;
                        
                        let response: Value = serde_json::from_slice(&content)?;
                        
                        if let Some(id) = response.get("id") {
                            if id.as_u64() == Some(expected_id) {
                                return Ok(response);
                            }
                        }
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Failed to read response"))
    }

    // Tool implementation methods
    pub async fn find_definition(&mut self, file_path: &str, line: u32, character: u32) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_text_document_position_params(file_path, line, character);
        let response = self.send_request_internal("textDocument/definition", params).await?;
        
        Ok(format!("Definition response: {}", response))
    }

    pub async fn find_references(&mut self, file_path: &str, line: u32, character: u32) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_references_params(file_path, line, character);
        let response = self.send_request_internal("textDocument/references", params).await?;
        
        Ok(format!("References response: {}", response))
    }

    pub async fn get_diagnostics(&mut self, file_path: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        // For diagnostics, we typically receive them via notifications
        // This is a simplified implementation
        Ok(format!("Diagnostics for file: {}", file_path))
    }

    pub async fn workspace_symbols(&mut self, query: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_workspace_symbol_params(query);
        let response = self.send_request_internal("workspace/symbol", params).await?;
        
        Ok(format!("Workspace symbols response: {}", response))
    }

    pub async fn rename_symbol(&mut self, file_path: &str, line: u32, character: u32, new_name: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_rename_params(file_path, line, character, new_name);
        let response = self.send_request_internal("textDocument/rename", params).await?;
        
        Ok(format!("Rename response: {}", response))
    }

    pub async fn format_code(&mut self, file_path: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }

        let params = create_formatting_params(file_path);
        let response = self.send_request_internal("textDocument/formatting", params).await?;
        
        Ok(format!("Formatting response: {}", response))
    }

    pub async fn analyze_manifest(&mut self, manifest_path: &str) -> Result<String> {
        // This would analyze Cargo.toml file
        Ok(format!("Manifest analysis for: {}", manifest_path))
    }

    pub async fn run_cargo_check(&mut self, workspace_path: &str) -> Result<String> {
        // This would run cargo check and parse results
        Ok(format!("Cargo check results for: {}", workspace_path))
    }
}