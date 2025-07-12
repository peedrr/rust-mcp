use rmcp::schemars;

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