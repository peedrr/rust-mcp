use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorData as McpError, *},
    schemars,
    tool, tool_handler, tool_router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::analyzer::RustAnalyzerClient;
use crate::tools::{execute_tool, get_tier1_tools};

// Parameter structs for tools
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindDefinitionParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindReferencesParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetDiagnosticsParams {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WorkspaceSymbolsParams {
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RenameSymbolParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
    pub new_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatCodeParams {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnalyzeManifestParams {
    pub manifest_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunCargoCheckParams {
    pub workspace_path: String,
}

#[derive(Clone)]
pub struct RustMcpServer {
    analyzer: Arc<Mutex<RustAnalyzerClient>>,
    tool_router: ToolRouter<RustMcpServer>,
}

#[tool_router]
impl RustMcpServer {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(Mutex::new(RustAnalyzerClient::new())),
            tool_router: Self::tool_router(),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut analyzer = self.analyzer.lock().await;
        analyzer.start().await
    }

    pub fn list_tools(&self) -> Vec<crate::tools::ToolDefinition> {
        get_tier1_tools()
    }

    pub async fn call_tool(&mut self, name: &str, args: Value) -> Result<crate::tools::ToolResult> {
        let mut analyzer = self.analyzer.lock().await;
        execute_tool(name, args, &mut analyzer).await
    }

    #[tool(description = "Find the definition of a symbol at a given position")]
    async fn find_definition(
        &self,
        Parameters(FindDefinitionParams { file_path, line, character }): Parameters<FindDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "file_path": file_path,
            "line": line,
            "character": character
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("find_definition", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("No definition found")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Find all references to a symbol at a given position")]
    async fn find_references(
        &self,
        Parameters(FindReferencesParams { file_path, line, character }): Parameters<FindReferencesParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "file_path": file_path,
            "line": line,
            "character": character
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("find_references", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("No references found")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Get compiler diagnostics for a file")]
    async fn get_diagnostics(
        &self,
        Parameters(GetDiagnosticsParams { file_path }): Parameters<GetDiagnosticsParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "file_path": file_path
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("get_diagnostics", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("No diagnostics found")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Search for symbols in the workspace")]
    async fn workspace_symbols(
        &self,
        Parameters(WorkspaceSymbolsParams { query }): Parameters<WorkspaceSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "query": query
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("workspace_symbols", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("No symbols found")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Rename a symbol with scope awareness")]
    async fn rename_symbol(
        &self,
        Parameters(RenameSymbolParams { file_path, line, character, new_name }): Parameters<RenameSymbolParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "file_path": file_path,
            "line": line,
            "character": character,
            "new_name": new_name
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("rename_symbol", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("Rename operation completed")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Apply rustfmt formatting to a file")]
    async fn format_code(
        &self,
        Parameters(FormatCodeParams { file_path }): Parameters<FormatCodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "file_path": file_path
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("format_code", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("Format operation completed")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Parse and analyze Cargo.toml file")]
    async fn analyze_manifest(
        &self,
        Parameters(AnalyzeManifestParams { manifest_path }): Parameters<AnalyzeManifestParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "manifest_path": manifest_path
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("analyze_manifest", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("Analysis completed")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Execute cargo check and parse errors")]
    async fn run_cargo_check(
        &self,
        Parameters(RunCargoCheckParams { workspace_path }): Parameters<RunCargoCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::json!({
            "workspace_path": workspace_path
        });
        
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool("run_cargo_check", args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("Cargo check completed")]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(&format!("Error: {}", e))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for RustMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Rust MCP Server providing rust-analyzer integration for idiomatic Rust development tools. Provides code analysis, refactoring, and project management capabilities.".to_string()),
        }
    }
}