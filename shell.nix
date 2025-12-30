# simple.nix
with (import <nixpkgs> {});
  mkShell {
    buildInputs = [
      perf
      protobuf
      zenoh
      clang
      rustPlatform.bindgenHook
      rerun
      opencv4
      pkg-config

      #needed for a temp package
      libx11
      wayland
    ];

    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  }
