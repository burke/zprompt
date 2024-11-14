{
  description = "zprompt - A Rust binary";

  inputs = {
    nixpkgs.url = "github:NixPkgs/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "zprompt";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              # If you have git dependencies, add their output hashes here
            };
          };

          nativeBuildInputs = [ rustToolchain ];
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = "zprompt";
        };
      }
    );
} 