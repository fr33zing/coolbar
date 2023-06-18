{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
    nixgl.url = "github:guibou/nixGL";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, nixpkgs-mozilla, nixgl }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import ./nix/overlay.nix)
            (import nixpkgs-mozilla)
            nixgl.overlay
          ];
        };

        toolchain = (pkgs.rustChannelOf {
          date = "2023-06-01";
          channel = "stable";
          sha256 = "sha256-gdYqng0y9iHYzYPAdkC/ka3DRny3La/S5G8ASj0Ayyc=";
        }).rust;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        nativeBuildInputs = with pkgs; [
          pkg-config
          gobject-introspection
        ] ++ [
          toolchain
        ];

        buildInputs = with pkgs; [
          glibcLocales
          glib
          gtk4
          gtk4-layer-shell
          libadwaita
          libpulseaudio
        ];
      in
        rec {
          defaultPackage = naersk'.buildPackage {
            inherit nativeBuildInputs buildInputs;
            src = ./.;
          };

          devShell = with pkgs; mkShell {
            inherit nativeBuildInputs buildInputs;

            packages = if lib.inPureEvalMode then [] else [
              (writeShellScriptBin "cargo" ''
                ${lib.getExe pkgs.nixgl.auto.nixGLDefault} ${toolchain}/bin/cargo $@
              '')
            ];

            shellHook = let
              pure = if lib.inPureEvalMode then "1" else "0";
              grep = lib.getExe gnugrep;
            in ''
              if ${grep} -q "ID=nixos" /etc/os-release; then
                if [ ${pure} -eq 0 ]; then
                  printf "\n%s\n\n" 'Run `nix develop` without `--impure`. Impure mode is unnecessary on NixOS.'
                  exit 1
                fi
              else
                if [ ${pure} -eq 1 ]; then
                  printf "\n%s\n\n" 'Run `nix develop --impure` to prevent fallback to software rendering.'
                  exit 1
                fi
              fi
            '';
          };
        }
    );
}
