{
  inputs = {
    pkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, utils, ... }@inputs:
    utils.lib.eachDefaultSystem (system:
      let
        lock = builtins.fromJSON (builtins.readFile ./flake.lock);
        overlays = [
          inputs.rust-overlay.overlay ];
        pkgs = import inputs.pkgs { inherit system overlays; };

        # Get the latest rust nightly
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = [ "rust-src" "rust-analyzer-preview" ];
            targets = [ "wasm32-unknown-unknown" ];
          });

        update-artifacts = pkgs.writeScriptBin "update-artifacts" ''
          ${pkgs.trunk}/bin/trunk build --release --dist docs --public-url /cellular-automaton
        '';

        serve = pkgs.writeScriptBin "serve" ''
          ${pkgs.trunk}/bin/trunk serve
        '';

      in {
        # `nix develop`
        devShell = pkgs.mkShell rec {
          # supply the specific rust version
          nativeBuildInputs = [
            pkgs.cargo-readme
            pkgs.gcc
            pkgs.miniserve
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.wasm-pack
            rust
            serve
            update-artifacts
          ];
          RUST_SRC_PATH = "${rust}";
          RUST_ANALYZER = "${rust}/bin/rust-analyzer";
        };
      });
}
