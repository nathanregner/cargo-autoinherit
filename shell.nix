{
  pkgs ? import <nixpkgs> { },
}:
let
  inherit (pkgs)
    cargo
    clippy
    mkShell
    rust-analyzer
    rustfmt
    rustPlatform
    ;
in
mkShell {
  RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  packages = [
    cargo
    clippy
    rust-analyzer
    rustfmt
  ];
}
