{
  description = "Rust MCP Server - Model Context Protocol server for rust-analyzer integration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        naersk' = pkgs.callPackage naersk { };
      in
      {
        packages = {
          rust-mcp = naersk'.buildPackage {
            src = ./.;
          };
          default = self.packages.${system}.rust-mcp;
        };

        apps = {
          rust-mcp = {
            type = "app";
            program = "${self.packages.${system}.rust-mcp}/bin/rustmcp";
          };
          default = self.apps.${system}.rust-mcp;
        };
      }
    );
}