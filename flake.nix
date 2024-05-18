{
  description = "TaskUI flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs { inherit system overlays; };

        toolchain = pkgs.rust-bin.stable."1.78.0".default.override {
          targets = [
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
            "x86_64-unknown-linux-gnu"
            "aarch64-unknown-linux-gnu"
          ];
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

      in rec {
        packages = {
          default = naersk'.buildPackage {
            src = ./.;
          };
          test = naersk'.buildPackage {
            src = ./.;
            mode = "test";
          };
		  clippy = naersk'.buildPackage {
            src = ./.;
            mode = "clippy";
          };
        };

        devShell = pkgs.mkShell { nativeBuildInputs = [ toolchain pkgs.rust-analyzer ]; };
      });
}
