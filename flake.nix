{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-compat = {
      url = "github:NixOS/flake-compat";
      flake = false;
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = 
    inputs@{
      flake-parts,
      naersk,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      flake = {
        homeManagerModules.nixy = ./hm-module.nix;
      };

      perSystem =
        {
          lib,
          pkgs,
          system,
          ...
        }:
        let
          overlays = [ (import rust-overlay) ];
          pkgsWithRust = import nixpkgs { inherit system overlays; };

          rustToolchain = pkgsWithRust.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "clippy" "rustfmt" "rust-analyzer" ];
          };

          naersk' = pkgs.callPackage naersk {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
        in
        {
          packages.default = naersk'.buildPackage {
            src = ./.;
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = [
              rustToolchain
              pkgs.pkg-config
              pkgs.openssl
            ];

            # Create a stable symlink for IDEs like RustRover to follow
            shellHook = ''
              mkdir -p ./.rust-toolchain
              ln -sfn ${rustToolchain}/bin ./.rust-toolchain/bin
              ln -sfn ${rustToolchain}/lib ./.rust-toolchain/lib

              # Set source path for rust-analyzer/RustRover
              export RUST_SRC_PATH="./.rust-toolchain/lib/rustlib/src/rust/library"
              export RUSTUP_TOOLCHAIN="stable"
            '';
          };

          formatter = pkgs.nixfmt-tree;
        };
    };
}