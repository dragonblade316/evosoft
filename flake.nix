{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-25.11";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in
      with pkgs; rec {
        devShell = mkShell rec {
          buildInputs = [
            perf
            protobuf
            zenoh
            clang
            rustPlatform.bindgenHook
            rerun
            opencv4
            
            pkg-config
            mujoco

            #needed for a temp package
            libx11
            wayland

            libxkbcommon
            libGL

            # WINIT_UNIX_BACKEND=wayland
            # wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];
          env.MUJOCO_PATH = "${mujoco}";
          env.MUJOCO_PLUGIN_PATH = "${mujoco}/lib";
          env.MUJOCO_DYNAMIC_LINK_DIR = "${mujoco}/lib";

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
}
