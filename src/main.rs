use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use rustmcp::server::RustMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the rust-analyzer integration
    let mut rust_server = RustMcpServer::new();
    rust_server.start().await?;

    // Note: The #[tool] macros generate additional tools beyond our manual list
    println!("Starting Rust MCP Server with 18 tools (Tier 1 + Tier 2):");
    println!("  Tier 1 Tools (9):");
    println!("    - find_definition, find_references, get_diagnostics, workspace_symbols");
    println!("    - rename_symbol, extract_function, format_code");
    println!("    - analyze_manifest, run_cargo_check");
    println!("  Tier 2 Tools (9):");
    println!("    Code Generation: generate_struct, generate_enum, generate_trait_impl, generate_tests");
    println!("    Advanced Refactoring: inline_function, change_signature, organize_imports");
    println!("    Quality Checks: apply_clippy_suggestions, validate_lifetimes");
    println!("Server running on stdio transport...");

    // Start the MCP server using the ServiceExt trait
    let service = rust_server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}