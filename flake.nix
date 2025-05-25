{
  description = "Static Pointer Analyzer and Transpiler from C to Safe Rust";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllPkgs = f: lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      sonder =
        pkgs:
        let
          manifest = pkgs.lib.importTOML ./Cargo.toml;
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = manifest.package.name;
          version = manifest.package.version;

          src = lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            mainProgram = "sonder";
            license = lib.licenses.mit;
            description = "Static Pointer Analyzer and Transpiler from C to Safe Rust";
          };
        };
    in
    {
      packages = forAllPkgs (pkgs: rec {
        sonder = sonder pkgs;
        default = sonder;
      });
      overlays = rec {
        default = sonder;
        sonder = final: _prev: sonder final;
      };
      devShells = forAllPkgs (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ (sonder pkgs) ];
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            rustPackages.clippy
            rust-analyzer
          ];
        };
      });
    };
}
