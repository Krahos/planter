{ pkgs, lib, config, inputs, ... }:

{
  languages.rust.enable = true;

  packages = with pkgs; [
    bacon
    cargo-edit
    rust-analyzer
    cargo-deny

    pkg-config
    dbus
    libxkbcommon
    expat
    fontconfig
    freetype
    freetype.dev
    libGL
  ] ++ lib.optionals stdenv.isLinux [
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  env.LD_LIBRARY_PATH = lib.makeLibraryPath config.packages;
}
