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
    fn new(name: &'static str, description: &'static str, schema: Value) -> Self {
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
        // Basic Refactoring
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
            "extract_function",
            "Extract selected code into a new function",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "start_line": {"type": "number"},
                    "start_character": {"type": "number"},
                    "end_line": {"type": "number"},
                    "end_character": {"type": "number"},
                    "function_name": {"type": "string"}
                },
                "required": ["file_path", "start_line", "start_character", "end_line", "end_character", "function_name"]
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
        // Project Management
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

pub struct ToolResult {
    pub content: Vec<Value>,
    pub is_error: bool,
}

pub async fn execute_tool(
    name: &str,
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    match name {
        // Code Analysis
        "find_definition" => find_definition(args, analyzer).await,
        "find_references" => find_references(args, analyzer).await,
        "get_diagnostics" => get_diagnostics(args, analyzer).await,
        "workspace_symbols" => workspace_symbols(args, analyzer).await,
        // Basic Refactoring
        "rename_symbol" => rename_symbol(args, analyzer).await,
        "extract_function" => extract_function(args, analyzer).await,
        "format_code" => format_code(args, analyzer).await,
        // Project Management
        "analyze_manifest" => analyze_manifest(args, analyzer).await,
        "run_cargo_check" => run_cargo_check(args, analyzer).await,
        // Code Generation
        "generate_struct" => generate_struct(args, analyzer).await,
        "generate_enum" => generate_enum(args, analyzer).await,
        "generate_trait_impl" => generate_trait_impl(args, analyzer).await,
        "generate_tests" => generate_tests(args, analyzer).await,
        // Advanced Refactoring
        "inline_function" => inline_function(args, analyzer).await,
        "change_signature" => change_signature(args, analyzer).await,
        "organize_imports" => organize_imports(args, analyzer).await,
        // Quality Checks
        "apply_clippy_suggestions" => apply_clippy_suggestions(args, analyzer).await,
        "validate_lifetimes" => validate_lifetimes(args, analyzer).await,
        _ => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Unknown tool: {}", name)
            })],
            is_error: true,
        }),
    }
}

async fn find_definition(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();
    let line = args["line"].as_u64().unwrap() as u32;
    let character = args["character"].as_u64().unwrap() as u32;

    // Open the document first
    analyzer.open_document(file_path).await?;

    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "position": {
            "line": line,
            "character": character
        }
    });

    match analyzer.send_request("textDocument/definition", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() {
                    "No definition found".to_string()
                } else if let Some(locations) = result.as_array() {
                    if locations.is_empty() {
                        "No definition found".to_string()
                    } else {
                        format!("Found {} definition(s):\n{}", locations.len(), 
                            serde_json::to_string_pretty(result)?)
                    }
                } else {
                    format!("Definition found:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error finding definition: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn find_references(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();
    let line = args["line"].as_u64().unwrap() as u32;
    let character = args["character"].as_u64().unwrap() as u32;

    // Open the document first
    analyzer.open_document(file_path).await?;

    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "position": {
            "line": line,
            "character": character
        },
        "context": {
            "includeDeclaration": true
        }
    });

    match analyzer.send_request("textDocument/references", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() {
                    "No references found".to_string()
                } else if let Some(locations) = result.as_array() {
                    if locations.is_empty() {
                        "No references found".to_string()
                    } else {
                        format!("Found {} reference(s):\n{}", locations.len(), 
                            serde_json::to_string_pretty(result)?)
                    }
                } else {
                    format!("References found:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error finding references: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn get_diagnostics(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();

    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        }
    });

    let response = analyzer
        .send_request("textDocument/publishDiagnostics", params)
        .await?;

    Ok(ToolResult {
        content: vec![json!({
            "type": "text",
            "text": format!("Diagnostics result: {}", response)
        })],
        is_error: false,
    })
}

async fn workspace_symbols(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let query = args["query"].as_str().unwrap();

    let params = json!({
        "query": query
    });

    match analyzer.send_request("workspace/symbol", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() {
                    "No symbols found".to_string()
                } else if let Some(symbols) = result.as_array() {
                    if symbols.is_empty() {
                        "No symbols found".to_string()
                    } else {
                        format!("Found {} symbol(s):\n{}", symbols.len(), 
                            serde_json::to_string_pretty(result)?)
                    }
                } else {
                    format!("Symbols found:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error searching symbols: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn rename_symbol(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();
    let line = args["line"].as_u64().unwrap() as u32;
    let character = args["character"].as_u64().unwrap() as u32;
    let new_name = args["new_name"].as_str().unwrap();

    // Open the document first
    analyzer.open_document(file_path).await?;

    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "position": {
            "line": line,
            "character": character
        },
        "newName": new_name
    });

    match analyzer.send_request("textDocument/rename", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() {
                    "Cannot rename symbol at this position".to_string()
                } else if let Some(workspace_edit) = result.as_object() {
                    if let Some(changes) = workspace_edit.get("changes") {
                        let change_count = changes.as_object()
                            .map(|obj| obj.len())
                            .unwrap_or(0);
                        format!("Rename operation would affect {} file(s):\n{}", 
                            change_count, serde_json::to_string_pretty(result)?)
                    } else {
                        format!("Rename result:\n{}", serde_json::to_string_pretty(result)?)
                    }
                } else {
                    format!("Rename result:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error renaming symbol: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn extract_function(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();
    let start_line = args["start_line"].as_u64().unwrap() as u32;
    let start_character = args["start_character"].as_u64().unwrap() as u32;
    let end_line = args["end_line"].as_u64().unwrap() as u32;
    let end_character = args["end_character"].as_u64().unwrap() as u32;
    let _function_name = args["function_name"].as_str().unwrap();

    // Open the document first
    analyzer.open_document(file_path).await?;

    // Extract function is typically implemented via code actions
    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "range": {
            "start": {
                "line": start_line,
                "character": start_character
            },
            "end": {
                "line": end_line,
                "character": end_character
            }
        },
        "context": {
            "diagnostics": []
        }
    });

    match analyzer.send_request("textDocument/codeAction", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() || (result.is_array() && result.as_array().unwrap().is_empty()) {
                    "No extract function refactoring available for this selection".to_string()
                } else if let Some(actions) = result.as_array() {
                    let extract_actions: Vec<_> = actions.iter()
                        .filter(|action| {
                            if let Some(title) = action.get("title").and_then(|t| t.as_str()) {
                                title.to_lowercase().contains("extract")
                            } else {
                                false
                            }
                        })
                        .collect();
                    
                    if extract_actions.is_empty() {
                        format!("Available code actions (no extract function found):\n{}", 
                            serde_json::to_string_pretty(result)?)
                    } else {
                        format!("Found {} extract-related action(s):\n{}", 
                            extract_actions.len(), 
                            serde_json::to_string_pretty(&json!(extract_actions))?)
                    }
                } else {
                    format!("Code actions result:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error getting code actions: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn format_code(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();

    // Use rustfmt directly instead of LSP for formatting
    match tokio::process::Command::new("rustfmt")
        .arg("--check")
        .arg(file_path)
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("File {} is already properly formatted", file_path)
                    })],
                    is_error: false,
                })
            } else {
                // File needs formatting, let's format it
                match tokio::process::Command::new("rustfmt")
                    .arg(file_path)
                    .output()
                    .await
                {
                    Ok(format_output) => {
                        if format_output.status.success() {
                            Ok(ToolResult {
                                content: vec![json!({
                                    "type": "text",
                                    "text": format!("Successfully formatted {}", file_path)
                                })],
                                is_error: false,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&format_output.stderr);
                            Ok(ToolResult {
                                content: vec![json!({
                                    "type": "text",
                                    "text": format!("Error formatting file: {}", error_msg)
                                })],
                                is_error: true,
                            })
                        }
                    }
                    Err(e) => Ok(ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Error running rustfmt: {}", e)
                        })],
                        is_error: true,
                    }),
                }
            }
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error checking format status: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn analyze_manifest(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let manifest_path = args["manifest_path"].as_str().unwrap();

    match tokio::fs::read_to_string(manifest_path).await {
        Ok(content) => {
            match toml::from_str::<toml::Value>(&content) {
                Ok(parsed_toml) => {
                    let mut analysis = Vec::new();
                    
                    // Analyze package section
                    if let Some(package) = parsed_toml.get("package") {
                        if let Some(name) = package.get("name") {
                            analysis.push(format!("Package: {}", name));
                        }
                        if let Some(version) = package.get("version") {
                            analysis.push(format!("Version: {}", version));
                        }
                        if let Some(edition) = package.get("edition") {
                            analysis.push(format!("Edition: {}", edition));
                        }
                        if let Some(description) = package.get("description") {
                            analysis.push(format!("Description: {}", description));
                        }
                    }
                    
                    // Analyze dependencies
                    if let Some(deps) = parsed_toml.get("dependencies") {
                        if let Some(deps_table) = deps.as_table() {
                            analysis.push(format!("Dependencies ({}):", deps_table.len()));
                            for (name, version) in deps_table {
                                let version_str = match version {
                                    toml::Value::String(v) => v.clone(),
                                    toml::Value::Table(t) => {
                                        if let Some(v) = t.get("version") {
                                            format!("{} ({})", v, 
                                                if t.contains_key("features") { "with features" } else { "default" })
                                        } else if t.contains_key("git") {
                                            "git dependency".to_string()
                                        } else if t.contains_key("path") {
                                            "local path".to_string()
                                        } else {
                                            "complex dependency".to_string()
                                        }
                                    },
                                    _ => "unknown".to_string(),
                                };
                                analysis.push(format!("  - {}: {}", name, version_str));
                            }
                        }
                    }
                    
                    // Analyze dev-dependencies
                    if let Some(dev_deps) = parsed_toml.get("dev-dependencies") {
                        if let Some(dev_deps_table) = dev_deps.as_table() {
                            analysis.push(format!("Dev Dependencies ({}): {}", 
                                dev_deps_table.len(), 
                                dev_deps_table.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                        }
                    }
                    
                    // Analyze features
                    if let Some(features) = parsed_toml.get("features") {
                        if let Some(features_table) = features.as_table() {
                            analysis.push(format!("Features ({}): {}", 
                                features_table.len(),
                                features_table.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                        }
                    }
                    
                    let analysis_text = analysis.join("\n");
                    
                    Ok(ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Cargo.toml Analysis:\n{}", analysis_text)
                        })],
                        is_error: false,
                    })
                }
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error parsing TOML: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error reading manifest file: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn run_cargo_check(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let workspace_path = args["workspace_path"].as_str().unwrap();

    match tokio::process::Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .current_dir(workspace_path)
        .output()
        .await
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            let mut messages = Vec::new();
            let mut error_count = 0;
            let mut warning_count = 0;
            
            // Parse JSON messages from cargo
            for line in stdout.lines() {
                if let Ok(json_msg) = serde_json::from_str::<Value>(line) {
                    if let Some(reason) = json_msg.get("reason") {
                        if reason == "compiler-message" {
                            if let Some(message) = json_msg.get("message") {
                                if let Some(level) = message.get("level") {
                                    if let Some(rendered) = message.get("rendered") {
                                        let level_str = level.as_str().unwrap_or("unknown");
                                        match level_str {
                                            "error" => error_count += 1,
                                            "warning" => warning_count += 1,
                                            _ => {}
                                        }
                                        messages.push(format!("[{}] {}", level_str.to_uppercase(), 
                                            rendered.as_str().unwrap_or("No message")));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            let summary = if output.status.success() {
                if warning_count > 0 {
                    format!("✅ Cargo check completed with {} warning(s)", warning_count)
                } else {
                    "✅ Cargo check completed successfully - no issues found".to_string()
                }
            } else {
                format!("❌ Cargo check failed with {} error(s) and {} warning(s)", 
                    error_count, warning_count)
            };
            
            let result_text = if messages.is_empty() {
                format!("{}\n\nStderr: {}", summary, stderr)
            } else {
                format!("{}\n\nMessages:\n{}", summary, messages.join("\n"))
            };
            
            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: !output.status.success(),
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error running cargo check: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn generate_struct(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let struct_name = args["struct_name"].as_str().unwrap();
    let fields = args["fields"].as_array().unwrap();
    let derives = args.get("derives").and_then(|d| d.as_array());
    let visibility = args.get("visibility").and_then(|v| v.as_str()).unwrap_or("pub");
    let file_path = args["file_path"].as_str().unwrap();

    let mut struct_code = String::new();
    
    // Add derives if specified
    if let Some(derives_array) = derives {
        if !derives_array.is_empty() {
            let derive_list: Vec<String> = derives_array
                .iter()
                .filter_map(|d| d.as_str())
                .map(|s| s.to_string())
                .collect();
            struct_code.push_str(&format!("#[derive({})]\n", derive_list.join(", ")));
        }
    }
    
    // Add struct declaration
    struct_code.push_str(&format!("{} struct {} {{\n", visibility, struct_name));
    
    // Add fields
    for field in fields {
        if let Some(field_obj) = field.as_object() {
            let field_name = field_obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let field_type = field_obj.get("type").and_then(|t| t.as_str()).unwrap_or("String");
            let field_vis = field_obj.get("visibility").and_then(|v| v.as_str()).unwrap_or("");
            
            if field_vis.is_empty() {
                struct_code.push_str(&format!("    {}: {},\n", field_name, field_type));
            } else {
                struct_code.push_str(&format!("    {} {}: {},\n", field_vis, field_name, field_type));
            }
        }
    }
    
    struct_code.push_str("}\n");
    
    // Add basic impl block with new() constructor
    struct_code.push_str(&format!("\nimpl {} {{\n", struct_name));
    
    // Generate constructor parameters
    let constructor_params: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            if let Some(field_obj) = field.as_object() {
                let field_name = field_obj.get("name").and_then(|n| n.as_str())?;
                let field_type = field_obj.get("type").and_then(|t| t.as_str())?;
                Some(format!("{}: {}", field_name, field_type))
            } else {
                None
            }
        })
        .collect();
    
    let constructor_assignments: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            if let Some(field_obj) = field.as_object() {
                let field_name = field_obj.get("name").and_then(|n| n.as_str())?;
                Some(format!("            {}", field_name))
            } else {
                None
            }
        })
        .collect();
    
    struct_code.push_str(&format!(
        "    pub fn new({}) -> Self {{\n        Self {{\n{},\n        }}\n    }}\n",
        constructor_params.join(", "),
        constructor_assignments.join(",\n")
    ));
    
    struct_code.push_str("}\n");
    
    // Write to file
    match tokio::fs::read_to_string(file_path).await {
        Ok(existing_content) => {
            let new_content = format!("{}\n{}", existing_content, struct_code);
            match tokio::fs::write(file_path, new_content).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully generated struct {} in {}\n\nGenerated code:\n{}", 
                            struct_name, file_path, struct_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error writing to file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
        Err(_) => {
            // File doesn't exist, create it
            match tokio::fs::write(file_path, &struct_code).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully created file {} with struct {}\n\nGenerated code:\n{}", 
                            file_path, struct_name, struct_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error creating file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
    }
}

async fn generate_enum(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let enum_name = args["enum_name"].as_str().unwrap();
    let variants = args["variants"].as_array().unwrap();
    let derives = args.get("derives").and_then(|d| d.as_array());
    let visibility = args.get("visibility").and_then(|v| v.as_str()).unwrap_or("pub");
    let file_path = args["file_path"].as_str().unwrap();

    let mut enum_code = String::new();
    
    // Add derives if specified
    if let Some(derives_array) = derives {
        if !derives_array.is_empty() {
            let derive_list: Vec<String> = derives_array
                .iter()
                .filter_map(|d| d.as_str())
                .map(|s| s.to_string())
                .collect();
            enum_code.push_str(&format!("#[derive({})]\n", derive_list.join(", ")));
        }
    }
    
    // Add enum declaration
    enum_code.push_str(&format!("{} enum {} {{\n", visibility, enum_name));
    
    // Add variants
    for variant in variants {
        if let Some(variant_str) = variant.as_str() {
            enum_code.push_str(&format!("    {},\n", variant_str));
        }
    }
    
    enum_code.push_str("}\n");
    
    // Write to file
    match tokio::fs::read_to_string(file_path).await {
        Ok(existing_content) => {
            let new_content = format!("{}\n{}", existing_content, enum_code);
            match tokio::fs::write(file_path, new_content).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully generated enum {} in {}\n\nGenerated code:\n{}", 
                            enum_name, file_path, enum_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error writing to file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
        Err(_) => {
            match tokio::fs::write(file_path, &enum_code).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully created file {} with enum {}\n\nGenerated code:\n{}", 
                            file_path, enum_name, enum_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error creating file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
    }
}

async fn generate_trait_impl(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let trait_name = args["trait_name"].as_str().unwrap();
    let target_type = args["target_type"].as_str().unwrap();
    let file_path = args["file_path"].as_str().unwrap();

    let mut impl_code = format!("impl {} for {} {{\n", trait_name, target_type);
    
    // Generate stub methods for common traits
    match trait_name {
        "Display" => {
            impl_code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            impl_code.push_str(&format!("        write!(f, \"{}\")\n", target_type));
            impl_code.push_str("    }\n");
        }
        "Debug" => {
            impl_code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            impl_code.push_str(&format!("        f.debug_struct(\"{}\")\n", target_type));
            impl_code.push_str("            .finish()\n");
            impl_code.push_str("    }\n");
        }
        "Default" => {
            impl_code.push_str("    fn default() -> Self {\n");
            impl_code.push_str("        todo!(\"Implement default constructor\")\n");
            impl_code.push_str("    }\n");
        }
        _ => {
            impl_code.push_str("    // TODO: Implement trait methods\n");
        }
    }
    
    impl_code.push_str("}\n");
    
    // Write to file
    match tokio::fs::read_to_string(file_path).await {
        Ok(existing_content) => {
            let new_content = format!("{}\n{}", existing_content, impl_code);
            match tokio::fs::write(file_path, new_content).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully generated {} implementation for {} in {}\n\nGenerated code:\n{}", 
                            trait_name, target_type, file_path, impl_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error writing to file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
        Err(_) => {
            match tokio::fs::write(file_path, &impl_code).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully created file {} with {} implementation for {}\n\nGenerated code:\n{}", 
                            file_path, trait_name, target_type, impl_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error creating file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
    }
}

async fn generate_tests(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let target_function = args.get("target_function").and_then(|f| f.as_str());
    let target_struct = args.get("target_struct").and_then(|s| s.as_str());
    let test_type = args.get("test_type").and_then(|t| t.as_str()).unwrap_or("unit");
    let file_path = args["file_path"].as_str().unwrap();
    
    let mut test_code = String::new();
    
    match test_type {
        "unit" => {
            test_code.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
            
            if let Some(function_name) = target_function {
                test_code.push_str(&format!("    #[test]\n    fn test_{}() {{\n", function_name));
                test_code.push_str("        // TODO: Add test implementation\n");
                test_code.push_str("    }\n\n");
            }
            
            if let Some(struct_name) = target_struct {
                test_code.push_str(&format!("    #[test]\n    fn test_{}_new() {{\n", struct_name.to_lowercase()));
                test_code.push_str("        // TODO: Test struct creation\n");
                test_code.push_str("    }\n\n");
            }
            
            if target_function.is_none() && target_struct.is_none() {
                test_code.push_str("    #[test]\n    fn test_example() {\n");
                test_code.push_str("        assert_eq!(2 + 2, 4);\n");
                test_code.push_str("    }\n\n");
            }
            
            test_code.push_str("}\n");
        }
        "integration" => {
            test_code.push_str("// Integration test - place in tests/ directory\n\n");
            test_code.push_str("#[test]\nfn integration_test() {\n");
            test_code.push_str("    // TODO: Add integration test\n");
            test_code.push_str("}\n");
        }
        _ => {
            return Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Unknown test type: {}. Supported: unit, integration", test_type)
                })],
                is_error: true,
            });
        }
    }
    
    // Write to file
    match tokio::fs::read_to_string(file_path).await {
        Ok(existing_content) => {
            let new_content = format!("{}\n{}", existing_content, test_code);
            match tokio::fs::write(file_path, new_content).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully generated {} tests in {}\n\nGenerated code:\n{}", 
                            test_type, file_path, test_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error writing to file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
        Err(_) => {
            match tokio::fs::write(file_path, &test_code).await {
                Ok(_) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Successfully created file {} with {} tests\n\nGenerated code:\n{}", 
                            file_path, test_type, test_code)
                    })],
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Error creating file: {}", e)
                    })],
                    is_error: true,
                }),
            }
        }
    }
}

// =============================================================================
// Tier 2: Advanced Refactoring Functions
// =============================================================================

async fn inline_function(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();
    let line = args["line"].as_u64().unwrap() as u32;
    let character = args["character"].as_u64().unwrap() as u32;

    // Open the document first
    analyzer.open_document(file_path).await?;

    // Use code actions to find inline function refactoring
    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "range": {
            "start": {
                "line": line,
                "character": character
            },
            "end": {
                "line": line,
                "character": character
            }
        },
        "context": {
            "diagnostics": [],
            "only": ["refactor.inline"]
        }
    });

    match analyzer.send_request("textDocument/codeAction", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() || (result.is_array() && result.as_array().unwrap().is_empty()) {
                    "No inline function refactoring available at this position".to_string()
                } else if let Some(actions) = result.as_array() {
                    let inline_actions: Vec<_> = actions.iter()
                        .filter(|action| {
                            if let Some(title) = action.get("title").and_then(|t| t.as_str()) {
                                title.to_lowercase().contains("inline")
                            } else {
                                false
                            }
                        })
                        .collect();
                    
                    if inline_actions.is_empty() {
                        format!("Available code actions (no inline function found):\n{}", 
                            serde_json::to_string_pretty(result)?)
                    } else {
                        format!("Found {} inline function action(s):\n{}", 
                            inline_actions.len(), 
                            serde_json::to_string_pretty(&json!(inline_actions))?)
                    }
                } else {
                    format!("Inline function result:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error getting inline function actions: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn change_signature(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();
    let line = args["line"].as_u64().unwrap() as u32;
    let character = args["character"].as_u64().unwrap() as u32;
    let new_signature = args["new_signature"].as_str().unwrap();

    // Open the document first
    analyzer.open_document(file_path).await?;

    // Use code actions to find signature change refactoring
    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "range": {
            "start": {
                "line": line,
                "character": character
            },
            "end": {
                "line": line,
                "character": character
            }
        },
        "context": {
            "diagnostics": [],
            "only": ["refactor.rewrite"]
        }
    });

    match analyzer.send_request("textDocument/codeAction", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() || (result.is_array() && result.as_array().unwrap().is_empty()) {
                    format!("No signature change refactoring available. Note: Manual signature change needed to: {}", new_signature)
                } else if let Some(actions) = result.as_array() {
                    let signature_actions: Vec<_> = actions.iter()
                        .filter(|action| {
                            if let Some(title) = action.get("title").and_then(|t| t.as_str()) {
                                let title_lower = title.to_lowercase();
                                title_lower.contains("signature") || title_lower.contains("parameter") || title_lower.contains("argument")
                            } else {
                                false
                            }
                        })
                        .collect();
                    
                    if signature_actions.is_empty() {
                        format!("Available code actions (no signature change found):\n{}\n\nRequested signature: {}", 
                            serde_json::to_string_pretty(result)?, new_signature)
                    } else {
                        format!("Found {} signature-related action(s):\n{}\n\nRequested signature: {}", 
                            signature_actions.len(), 
                            serde_json::to_string_pretty(&json!(signature_actions))?, 
                            new_signature)
                    }
                } else {
                    format!("Signature change result:\n{}\n\nRequested signature: {}", 
                        serde_json::to_string_pretty(result)?, new_signature)
                }
            } else {
                format!("Raw response: {}\n\nRequested signature: {}", response, new_signature)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error getting signature change actions: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn organize_imports(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();

    // Open the document first
    analyzer.open_document(file_path).await?;

    // Use code actions to organize imports
    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        },
        "range": {
            "start": {
                "line": 0,
                "character": 0
            },
            "end": {
                "line": 0,
                "character": 0
            }
        },
        "context": {
            "diagnostics": [],
            "only": ["source.organizeImports"]
        }
    });

    match analyzer.send_request("textDocument/codeAction", params).await {
        Ok(response) => {
            let result_text = if let Some(result) = response.get("result") {
                if result.is_null() || (result.is_array() && result.as_array().unwrap().is_empty()) {
                    "No organize imports action available (imports may already be organized)".to_string()
                } else if let Some(actions) = result.as_array() {
                    let organize_actions: Vec<_> = actions.iter()
                        .filter(|action| {
                            if let Some(title) = action.get("title").and_then(|t| t.as_str()) {
                                let title_lower = title.to_lowercase();
                                title_lower.contains("organize") || title_lower.contains("sort") || title_lower.contains("import")
                            } else {
                                false
                            }
                        })
                        .collect();
                    
                    if organize_actions.is_empty() {
                        format!("Available code actions (no organize imports found):\n{}", 
                            serde_json::to_string_pretty(result)?)
                    } else {
                        format!("Found {} organize imports action(s):\n{}", 
                            organize_actions.len(), 
                            serde_json::to_string_pretty(&json!(organize_actions))?)
                    }
                } else {
                    format!("Organize imports result:\n{}", serde_json::to_string_pretty(result)?)
                }
            } else {
                format!("Raw response: {}", response)
            };

            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: false,
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error organizing imports: {}", e)
            })],
            is_error: true,
        }),
    }
}

// =============================================================================
// Tier 2: Quality Checks Functions
// =============================================================================

async fn apply_clippy_suggestions(args: Value, _analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let workspace_path = args["workspace_path"].as_str().unwrap();

    match tokio::process::Command::new("cargo")
        .args(["clippy", "--fix", "--allow-dirty", "--all-targets", "--", "-W", "clippy::all"])
        .current_dir(workspace_path)
        .output()
        .await
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            let mut suggestions_applied = 0;
            let mut warnings = Vec::new();
            
            // Parse clippy output for applied fixes and remaining warnings
            for line in stdout.lines().chain(stderr.lines()) {
                if line.contains("Fixed") || line.contains("fixed") {
                    suggestions_applied += 1;
                } else if line.contains("warning:") {
                    warnings.push(line.to_string());
                }
            }
            
            let result_text = if output.status.success() {
                if suggestions_applied > 0 {
                    format!("✅ Clippy applied {} automatic fix(es)\n\nRemaining warnings: {}\n\nOutput:\n{}", 
                        suggestions_applied, warnings.len(), stdout)
                } else {
                    format!("✅ Clippy completed - no automatic fixes applied\n\nWarnings found: {}\n\nOutput:\n{}", 
                        warnings.len(), stdout)
                }
            } else {
                format!("❌ Clippy failed to run\n\nStderr: {}\nStdout: {}", stderr, stdout)
            };
            
            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: !output.status.success(),
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error running clippy: {}", e)
            })],
            is_error: true,
        }),
    }
}

async fn validate_lifetimes(args: Value, analyzer: &mut RustAnalyzerClient) -> Result<ToolResult> {
    let file_path = args["file_path"].as_str().unwrap();

    // Open the document first
    analyzer.open_document(file_path).await?;

    // Get diagnostics to check for lifetime issues
    let params = json!({
        "textDocument": {
            "uri": format!("file://{}", file_path)
        }
    });

    match analyzer.send_request("textDocument/publishDiagnostics", params).await {
        Ok(response) => {
            let mut lifetime_issues = Vec::new();
            let mut borrow_checker_issues = Vec::new();
            
            if let Some(diagnostics) = response.get("params")
                .and_then(|p| p.get("diagnostics"))
                .and_then(|d| d.as_array()) 
            {
                for diagnostic in diagnostics {
                    if let Some(message) = diagnostic.get("message").and_then(|m| m.as_str()) {
                        let message_lower = message.to_lowercase();
                        if message_lower.contains("lifetime") {
                            lifetime_issues.push(message.to_string());
                        } else if message_lower.contains("borrow") || message_lower.contains("borrowed") 
                            || message_lower.contains("move") || message_lower.contains("moved") {
                            borrow_checker_issues.push(message.to_string());
                        }
                    }
                }
            }
            
            let result_text = if lifetime_issues.is_empty() && borrow_checker_issues.is_empty() {
                "✅ No lifetime or borrow checker issues found".to_string()
            } else {
                let mut result = String::new();
                
                if !lifetime_issues.is_empty() {
                    result.push_str(&format!("⚠️  Found {} lifetime issue(s):\n", lifetime_issues.len()));
                    for (i, issue) in lifetime_issues.iter().enumerate() {
                        result.push_str(&format!("  {}. {}\n", i + 1, issue));
                    }
                    result.push('\n');
                }
                
                if !borrow_checker_issues.is_empty() {
                    result.push_str(&format!("⚠️  Found {} borrow checker issue(s):\n", borrow_checker_issues.len()));
                    for (i, issue) in borrow_checker_issues.iter().enumerate() {
                        result.push_str(&format!("  {}. {}\n", i + 1, issue));
                    }
                }
                
                result
            };
            
            Ok(ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": result_text
                })],
                is_error: !lifetime_issues.is_empty() || !borrow_checker_issues.is_empty(),
            })
        }
        Err(e) => Ok(ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Error validating lifetimes: {}", e)
            })],
            is_error: true,
        }),
    }
}