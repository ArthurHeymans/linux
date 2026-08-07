# shell.nix
#
# Provides all build tools directly in the shell environment for use with:
#   direnv (.envrc with `use nix`)
#   nix-shell --run "make ..."
#   nix-shell (interactive - auto-enters FHS env for crossgcc compat)
#
# The FHS environment (coreboot-env) is also built and available for cases
# where crossgcc binaries require standard /lib paths. When running under
# direnv or non-interactively, the packages are available directly without
# entering the FHS namespace.
#
# Usage:
#   direnv: add `use nix` to .envrc
#   interactive: nix-shell  (auto-enters FHS env)
#   scripted:    nix-shell --run "make -j4"

{
  pkgs ? import <nixpkgs> { },
}:

let
  corebootPkgs = with pkgs; [
    gnat15 # gcc with ada
    ncurses
    ncurses.dev
    m4
    flex
    bison # Generate flashmap descriptor parser
    zlib
    zlib.dev # zlib headers for building gcc
    pkg-config
    qemu # test the image
    libuuid
    nasm
    acpica-tools

    # EDK2
    imagemagick

    # Rust toolchain (nightly required for SMM handler)
    rustup

    # Dependencies for building crossgcc toolchain
    gnumake
    gnutar
    patch
    wget
    curl
    xz
    bzip2
    coreutils # sha1sum, etc.
    gzip
    texinfo
    gmp
    gmp.dev
    mpfr
    mpfr.dev
    libmpc
    nss

    # Standard build tools
    binutils

    # Allwinner FEL USB flashing tool
    xfel
  ];

  # Keep glibc out of mkShell inputs. Adding glibc.dev there injects its
  # include directory with -isystem before libstdc++ headers, which breaks
  # GCC's #include_next <stdlib.h> while crossgcc builds host C++ tools.
  corebootFHSPkgs =
    with pkgs;
    corebootPkgs
    ++ [
      glibc
      glibc.dev
      glibc.static
    ];

  corebootFHS = pkgs.buildFHSEnv {
    name = "coreboot-env";

    targetPkgs = _: corebootFHSPkgs;

    profile = ''
      # Note: NIX_LDFLAGS with -lncurses was previously set here but it leaks
      # into cross-compiler linker invocations (smmstub, sipi_vector) and breaks
      # the build. ncurses is available via pkg-config for host tools that need it.

      # The crossgcc toolchain binaries (built by Nix's GCC) have Nix store
      # paths baked into their ELF interpreter and RUNPATH. The Nix ld-linux
      # does not search /lib by default, so libraries like libncursesw.so.6
      # (provided by the FHS env at /lib/) are not found at runtime. Setting
      # LD_LIBRARY_PATH bridges this gap.
      export LD_LIBRARY_PATH="/lib:/usr/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

      # Rust/rustup: use the user's existing toolchain directory
      export RUSTUP_HOME="''${RUSTUP_HOME:-$HOME/.rustup}"
      export CARGO_HOME="''${CARGO_HOME:-$HOME/.cargo}"
      export PATH="$CARGO_HOME/bin:$PATH"

      # Ensure nightly toolchain with rust-src is available (needed for -Z build-std)
      if command -v rustup &>/dev/null; then
        if ! rustup toolchain list 2>/dev/null | grep -q '^nightly-'; then
          echo "Installing Rust nightly toolchain (required for SMM handler)..."
          rustup toolchain install nightly --component rust-src
        elif ! rustup component list --toolchain nightly --installed 2>/dev/null | grep -q rust-src; then
          echo "Adding rust-src component to nightly toolchain..."
          rustup component add --toolchain nightly rust-src
        fi
      fi
    '';

    runScript = "bash";
  };
in
pkgs.mkShell {
  # All packages are included directly so direnv and nix-shell --run have
  # full access to build tools without entering the FHS namespace.
  nativeBuildInputs = corebootPkgs ++ [ corebootFHS ];

  shellHook = ''
    # Set LD_LIBRARY_PATH so crossgcc binaries built in this Nix environment
    # can find runtime libraries (ncurses, etc.) from the Nix store.
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath corebootPkgs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

    # Rust/rustup: use the user's existing toolchain directory
    export RUSTUP_HOME="''${RUSTUP_HOME:-$HOME/.rustup}"
    export CARGO_HOME="''${CARGO_HOME:-$HOME/.cargo}"
    export PATH="$CARGO_HOME/bin:$PATH"

    # For interactive nix-shell sessions, enter the FHS env so crossgcc binaries
    # compiled with standard /lib paths work. Skipped under direnv and --run.
    if [ -z "$IN_COREBOOT_FHS" ]; then
      export IN_COREBOOT_FHS=1
      if [[ -t 0 ]] && [[ "$-" == *i* ]]; then
        exec coreboot-env
      fi
    fi
  '';
}
