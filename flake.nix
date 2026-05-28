{

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/25.05";
    flake_utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake_utils,
      rust-overlay,
    }:
    flake_utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        # pkgs = import nixpkgs { inherit system; };
        rust-bin = pkgs.rust-bin.stable.latest.default; # pkgs.rust-bin.stable.latest.default.override {

        # targets = [ "thumbv6m-none-eabi" ];
        #};
      in
      rec {
        devShells.default = pkgs.mkShell rec {
          packages = [
            # packages.newOpenocd
            # pkgs.openocd
            # pkgs.cargo
            # pkgs.rustc

            rust-bin

            pkgs.libGL
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.vulkan-loader
            pkgs.mesa

            pkgs.xorg.libXcursor
            pkgs.xorg.libXrandr
            pkgs.xorg.libXi
            pkgs.xorg.libX11

            pkgs.renderdoc
            # pkgs.wayland
            # pkgs.libxkbcommon
            # pkgs.libGL
            # pkgs.rust-bin.stable.latest.default
            # pkgs.rust-bin.stable.latest.default.override
            # {
            #   targets = [ "thumbv6m-none-eabi" ];
            # }
          ];

          # env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${libPath}";
          RUST_LOG = "debug";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
        };
      }
    );

  # nixConfig = {
  #   boot.kernelModules = [ "usb-storage" ];
  #   boot.extraModprobeConfig = ''
  #     options usb-storage quirks=483:3744:i
  #   '';
  #
  # };

}
