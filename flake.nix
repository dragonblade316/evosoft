{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-25.11";
    robotics-scripts.url = "github:dragonblade316/robotics-scripts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    robotics-scripts,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
        config.allowUnfree = true;
      };
      onshape-to-robot = robotics-scripts.packages.${system}.onshape-to-robot;
      rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
    in
      with pkgs; rec {
        devShell = mkShell rec {
          buildInputs = [
            rust

            perf
            protobuf
            zenoh
            clang
            rustPlatform.bindgenHook
            rerun
            opencv4
            
            
            pkg-config
            mujoco
            unityhub

            #needed for a temp package
            libx11
            wayland

            libxkbcommon
            libGL
            zsh
            
            # WINIT_UNIX_BACKEND=wayland
            wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
            onshape-to-robot

            #for bevy
            alsa-lib
            #why is libudev in systemd?
            systemd
          ];

          shellHook = ''
            zsh
          '';

          env.MUJOCO_PATH = "${mujoco}";
          env.MUJOCO_PLUGIN_PATH = "${mujoco}/lib";
          env.MUJOCO_DYNAMIC_LINK_DIR = "${mujoco}/lib";

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
}
