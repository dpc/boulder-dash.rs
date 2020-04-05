let
  mozilla = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ mozilla ]; };
in

  with nixpkgs;

  mkShell {
    buildInputs = [
      alsaLib
      cmake
      freetype
      latest.rustChannels.stable.rust
      expat
      openssl
      pkgconfig
      python3
      vulkan-validation-layers
      xlibs.libX11
      musl
    ];

    APPEND_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
      vulkan-loader
      xlibs.libXcursor
      xlibs.libXi
      xlibs.libXrandr
    ];

    shellHook = ''
      export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
      export RUSTFLAGS="-C target-cpu=native"
    '';
  }
