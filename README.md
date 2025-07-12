# Rust MCP Server

A comprehensive Model Context Protocol (MCP) server that provides rust-analyzer integration for LLM-assisted Rust development. This server enables AI tools like Claude to work with Rust code idiomatically through rust-analyzer's Language Server Protocol capabilities, avoiding string manipulation and providing intelligent code analysis and refactoring.

## Features

### Tier 1 Tools (Essential - 9 tools)
**Code Analysis**
- `find_definition` - Navigate to symbol definitions
- `find_references` - Find all symbol uses  
- `get_diagnostics` - Get compiler errors/warnings with fixes
- `workspace_symbols` - Search project symbols

**Basic Refactoring**
- `rename_symbol` - Rename with scope awareness
- `extract_function` - Extract code into functions
- `format_code` - Apply rustfmt formatting

**Project Management**
- `analyze_manifest` - Parse and analyze Cargo.toml
- `run_cargo_check` - Execute cargo check with error parsing

### Tier 2 Tools (High Value - 9 tools)
**Code Generation**
- `generate_struct` - Create structs with derives and constructors
- `generate_enum` - Create enums with variants
- `generate_trait_impl` - Generate trait implementations with stubs
- `generate_tests` - Create unit or integration test templates

**Advanced Refactoring**
- `inline_function` - Inline function calls
- `change_signature` - Modify function signatures
- `organize_imports` - Sort and organize use statements

**Quality Checks**
- `apply_clippy_suggestions` - Apply clippy automatic fixes
- `validate_lifetimes` - Check lifetime and borrow checker issues

## Prerequisites

- Rust toolchain (1.70+)
- rust-analyzer installed at `/Users/dex/.cargo/bin/rust-analyzer` (or update path in code)
- An MCP-compatible client (Claude Desktop, Roo, etc.)

## Installation

1. Clone this repository:
```bash
git clone <repository-url>
cd rust-mcp
```

2. Build the server:
```bash
cargo build --release
```

3. The server binary will be available at `target/release/rustmcp`

## Configuration

### Claude Desktop

Add the following to your Claude Desktop MCP configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "/path/to/rust-mcp/target/release/rustmcp",
      "args": []
    }
  }
}
```

### Roo Configuration

Add to your Roo configuration file (typically `~/.roo/config.json`):

```json
{
  "mcp_servers": [
    {
      "name": "rust-analyzer",
      "command": "/path/to/rust-mcp/target/release/rustmcp",
      "args": [],
      "env": {}
    }
  ]
}
```

### Other MCP Clients

For any MCP-compatible client, configure it to run:
```bash
/path/to/rust-mcp/target/release/rustmcp
```

The server uses stdio transport and will be ready to accept MCP protocol messages.

## Usage Examples

Once configured, you can use the tools through your AI assistant. Here are some example prompts:

### Code Analysis
```
"Find all references to the `Config` struct in this Rust project"
"Show me the definition of the `parse_args` function"
"Check for compiler errors in src/main.rs"
```

### Refactoring
```
"Rename the variable `data` to `user_input` throughout the codebase"
"Extract this code block into a separate function called `validate_input`"
"Format all the code in src/lib.rs"
```

### Code Generation
```
"Generate a struct called `User` with fields: name (String), age (u32), email (String), with Debug and Clone derives"
"Create an enum called `HttpStatus` with variants: Ok, NotFound, ServerError"
"Generate unit tests for the `calculate_total` function"
```

### Quality Checks
```
"Run clippy and apply all automatic fixes to improve code quality"
"Check for any lifetime or borrow checker issues in src/auth.rs"
```

## Architecture

The server is built with a modular architecture:

- **`src/main.rs`** - Entry point and server initialization
- **`src/server.rs`** - MCP server implementation using rmcp crate
- **`src/analyzer.rs`** - rust-analyzer LSP client integration
- **`src/tools.rs`** - Tool implementations and execution logic
- **`src/lib.rs`** - Module declarations

### Key Technologies
- **rmcp** - Official Rust SDK for MCP implementation
- **rust-analyzer** - Rust Language Server Protocol implementation
- **tokio** - Async runtime for handling concurrent operations
- **serde_json** - JSON serialization for LSP communication

## Development

### Running in Development
```bash
cargo run
```

### Testing Individual Tools
The server exposes all tools through the MCP protocol. For debugging, you can:

1. Run the server: `cargo run`
2. Send MCP messages via stdin (JSON-RPC format)
3. Check server logs and responses

### Adding New Tools

1. Implement the tool function in `src/tools.rs`
2. Add the tool to the `execute_tool` match statement
3. Add the corresponding `#[tool]` method to `RustMcpServer` in `src/server.rs`
4. Update the tool count in `src/main.rs`

## Troubleshooting

### rust-analyzer Not Found
Ensure rust-analyzer is installed and the path in `src/analyzer.rs` is correct:
```rust
const RUST_ANALYZER_PATH: &str = "/Users/dex/.cargo/bin/rust-analyzer";
```

Update this path to match your rust-analyzer installation.

### MCP Connection Issues
- Verify the server binary path in your MCP client configuration
- Check that the binary has execute permissions: `chmod +x target/release/rustmcp`
- Ensure no other processes are using the same MCP server name

### LSP Communication Errors
- Verify rust-analyzer works independently: `rust-analyzer --version`
- Check that your Rust project has a valid `Cargo.toml`
- Ensure the workspace path is correct when calling tools

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes with tests
4. Submit a pull request

## License
