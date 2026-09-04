{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        buildInputs = with pkgs; [
          git

          elmPackages.elm
          elmPackages.elm-format
          elmPackages.elm-language-server

          sqlite
          llvmPackages_22.libclang
          llvmPackages_22.libllvm

          (rust-bin.selectLatestNightlyWith (toolchain:
            toolchain.default.override {
              extensions = ["rust-src"];
            }))
          mold
          zlib
        ];
      in {
        devShells.default = pkgs.mkShell {
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          DATABASE_URL = "sqlite://dummy-database.db";
          buildInputs = buildInputs;
        };
      }
    );
}
