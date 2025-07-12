use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

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

        self.write_message(&notification).await
    }

    async fn send_request_internal(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        self.write_message(&request).await?;
        self.read_response().await
    }

    async fn write_message(&mut self, message: &Value) -> Result<()> {
        if let Some(ref mut child) = self.process {
            if let Some(ref mut stdin) = child.stdin.as_mut() {
                let content = message.to_string();
                let header = format!("Content-Length: {}\r\n\r\n", content.len());
                
                stdin.write_all(header.as_bytes()).await?;
                stdin.write_all(content.as_bytes()).await?;
                stdin.flush().await?;
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("rust-analyzer not running"))
    }

    async fn read_response(&mut self) -> Result<Value> {
        if let Some(ref mut child) = self.process {
            if let Some(ref mut stdout) = child.stdout.as_mut() {
                let mut reader = BufReader::new(stdout);
                
                // Read header
                let mut header_line = String::new();
                reader.read_line(&mut header_line).await?;
                
                if !header_line.starts_with("Content-Length:") {
                    return Err(anyhow::anyhow!("Invalid LSP header"));
                }
                
                let content_length: usize = header_line
                    .trim()
                    .strip_prefix("Content-Length:")
                    .unwrap()
                    .trim()
                    .parse()?;
                
                // Read empty line
                let mut empty_line = String::new();
                reader.read_line(&mut empty_line).await?;
                
                // Read content
                let mut content = vec![0u8; content_length];
                reader.read_exact(&mut content).await?;
                
                let response: Value = serde_json::from_slice(&content)?;
                return Ok(response);
            }
        }
        Err(anyhow::anyhow!("rust-analyzer not running"))
    }

    pub async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        if !self.initialized {
            return Err(anyhow::anyhow!("rust-analyzer not initialized"));
        }
        
        self.send_request_internal(method, params).await
    }

    pub async fn open_document(&mut self, file_path: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(file_path).await?;
        
        let params = json!({
            "textDocument": {
                "uri": format!("file://{}", file_path),
                "languageId": "rust",
                "version": 1,
                "text": content
            }
        });

        self.send_notification("textDocument/didOpen", params).await
    }
}