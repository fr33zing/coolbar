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

          devShell = pkgs.mkShell {
            inherit nativeBuildInputs buildInputs;
            packages = with pkgs; [
              (writeShellScriptBin "cargo" ''
                ${lib.getExe pkgs.nixgl.auto.nixGLDefault} ${toolchain}/bin/cargo $@
              '')
            ];
          };
        }
    );
}
