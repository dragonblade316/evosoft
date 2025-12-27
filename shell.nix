# simple.nix
with (import <nixpkgs> {});
mkShell {
  buildInputs = [
    protobuf
    zenoh
    clang
    rustPlatform.bindgenHook
  ];
}

