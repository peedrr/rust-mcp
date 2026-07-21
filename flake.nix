{
  description = "Rust MCP Server - Model Context Protocol server for rust-analyzer integration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    nix-wrapper-modules = {
      url = "github:BirdeeHub/nix-wrapper-modules";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, naersk, flake-utils, nix-wrapper-modules }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        naersk' = pkgs.callPackage naersk { };
        wlib = nix-wrapper-modules.lib;

        # Pinned rust-analyzer carried by the wrapper so the server is
        # self-contained — no global rust-analyzer install required and no
        # dependence on the launching shell's environment.
        rust-analyzer = pkgs.rust-analyzer;

        # Unwrapped server binary from naersk.
        rust-mcp-unwrapped = naersk'.buildPackage { src = ./.; };

        # Self-contained wrapper: puts rust-analyzer on PATH and pins
        # RUST_ANALYZER_PATH to it, so the server works regardless of which
        # devShell (or none) launched it.
        rust-mcp = wlib.wrapPackage {
          inherit pkgs;
          package = rust-mcp-unwrapped;
          # naersk's derivation has no meta.mainProgram/pname, so its name
          # resolves to "rustmcp-1.0.0" — pin both so the wrapper is named
          # `rustmcp` and execs the real binary at `bin/rustmcp`.
          binName = "rustmcp";
          exePath = "bin/rustmcp";
          runtimePkgs = [ rust-analyzer ];
          env.RUST_ANALYZER_PATH = "${rust-analyzer}/bin/rust-analyzer";
        };
      in
      {
        packages = {
          inherit rust-mcp rust-mcp-unwrapped;
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
