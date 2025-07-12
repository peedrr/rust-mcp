use anyhow::Result;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::sync::Arc;

use crate::analyzer::RustAnalyzerClient;

pub struct ToolDefinition {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub input_schema: Arc<serde_json::Map<String, Value>>,
}

impl ToolDefinition {
    pub fn new(name: &'static str, description: &'static str, schema: Value) -> Self {
        let schema_map = match schema {
            Value::Object(map) => Arc::new(map),
            _ => Arc::new(serde_json::Map::new()),
        };

        Self {
            name: Cow::Borrowed(name),
            description: Cow::Borrowed(description),
            input_schema: schema_map,
        }
    }
}

pub struct ToolResult {
    pub content: Vec<serde_json::Map<String, Value>>,
}

pub async fn execute_tool(name: &str, args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    match name {
        "find_definition" => crate::tools::analysis::find_definition_impl(args, analyzer).await,
        "find_references" => crate::tools::analysis::find_references_impl(args, analyzer).await,
        "get_diagnostics" => crate::tools::analysis::get_diagnostics_impl(args, analyzer).await,
        "workspace_symbols" => crate::tools::navigation::workspace_symbols_impl(args, analyzer).await,
        "rename_symbol" => crate::tools::refactoring::rename_symbol_impl(args, analyzer).await,
        "format_code" => crate::tools::formatting::format_code_impl(args, analyzer).await,
        "analyze_manifest" => crate::tools::cargo::analyze_manifest_impl(args, analyzer).await,
        "run_cargo_check" => crate::tools::cargo::run_cargo_check_impl(args, analyzer).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}

pub fn get_tier1_tools() -> Vec<ToolDefinition> {
    vec![
        // Code Analysis
        ToolDefinition::new(
            "find_definition",
            "Find the definition of a symbol at a given position",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "number"},
                    "character": {"type": "number"}
                },
                "required": ["file_path", "line", "character"]
            }),
        ),
        ToolDefinition::new(
            "find_references",
            "Find all references to a symbol at a given position",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "number"},
                    "character": {"type": "number"}
                },
                "required": ["file_path", "line", "character"]
            }),
        ),
        ToolDefinition::new(
            "get_diagnostics",
            "Get compiler diagnostics for a file",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "workspace_symbols",
            "Search for symbols in the workspace",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }),
        ),
        ToolDefinition::new(
            "rename_symbol",
            "Rename a symbol with scope awareness",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "line": {"type": "number"},
                    "character": {"type": "number"},
                    "new_name": {"type": "string"}
                },
                "required": ["file_path", "line", "character", "new_name"]
            }),
        ),
        ToolDefinition::new(
            "format_code",
            "Apply rustfmt formatting to a file",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        ),
        ToolDefinition::new(
            "analyze_manifest",
            "Parse and analyze Cargo.toml file",
            json!({
                "type": "object",
                "properties": {
                    "manifest_path": {"type": "string"}
                },
                "required": ["manifest_path"]
            }),
        ),
        ToolDefinition::new(
            "run_cargo_check",
            "Execute cargo check and parse errors",
            json!({
                "type": "object",
                "properties": {
                    "workspace_path": {"type": "string"}
                },
                "required": ["workspace_path"]
            }),
        ),
    ]
}