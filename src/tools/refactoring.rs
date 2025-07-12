use anyhow::Result;
use serde_json::{json, Value};
use crate::analyzer::RustAnalyzerClient;
use crate::tools::types::ToolResult;

pub async fn rename_symbol_impl(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args.get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
    let line = args.get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing line parameter"))?;
    let character = args.get("character")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing character parameter"))?;
    let new_name = args.get("new_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing new_name parameter"))?;

    // Implementation will use rust-analyzer LSP to rename symbol
    let result = analyzer.rename_symbol(file_path, line as u32, character as u32, new_name).await?;
    
    Ok(ToolResult {
        content: vec![json!({
            "type": "text",
            "text": result
        }).as_object().unwrap().clone()],
    })
}