# Shell expression for the Nix package manager
#
# This nix expression creates an environment with necessary packages installed:
#
#  * `tockloader`
#  * arm-none-eabi toolchain
#  * elf2tab
#  * riscv32-embedded toolchain
#
# To use:
#
#  $ nix-shell

{ pkgs ? import <nixpkgs> {}, withUnfreePkgs ? false }:

with builtins;
let
  inherit (pkgs) stdenv stdenvNoCC lib;

  tockloader = import (pkgs.fetchFromGitHub {
    owner = "tock";
    repo = "tockloader";
    rev = "v1.12.0";
    sha256 = "sha256-VgbAKDY/7ZVINDkqSHF7C0zRzVgtk8YG6O/ZmUpsh/g=";
  }) { inherit pkgs withUnfreePkgs; };

  elf2tab = pkgs.rustPlatform.buildRustPackage rec {
    name = "elf2tab-${version}";
    version = "0.12.0";

    src = pkgs.fetchFromGitHub {
      owner = "tock";
      repo = "elf2tab";
      rev = "v${version}";
      sha256 = "sha256-+VeWLBI6md399Oaumt4pJrOkm0Nz7fmpXN2TjglUE34=";
    };

    cargoHash = "sha256-C1hg2/y557jRLkSBvFLxYKH+t8xEJudDvU72kO9sPug=";
  };


  rust_overlay = import "${pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "2da33335e40ca932b4c5ea632816eed573736fba";
    sha256 = "sha256-1HiKieYFvFi5Hw3x2/mptbbvAuL0QwlZQC9UIGNNb1w=";
  }}/overlay.nix";

  nixpkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

  # Get a custom cross-compile capable Rust install of a specific channel and
  # build. Tock expects a specific version of Rust with a selection of targets
  # and components to be present.
  rustBuild = (
    nixpkgs.fenix.fromToolchainFile { file = ./rust-toolchain.toml; }
  );
in
  pkgs.mkShell {
    name = "tock-dev";

    buildInputs = with pkgs; [
      rustBuild
      elf2tab
      qemu
      gcc-arm-embedded
      python3Full
      tockloader
      pkgsCross.riscv32-embedded.buildPackages.gcc
      openocd
    ];

    # Unfortunately, `segger-jlink` has been removed from Nixpkgs due to its
    # hard dependency in Qt4, which has multiple security issues and is
    # deprecated since a few years now. Efforts exist to bring the package back,
    # but for now we don't assume it's available. Once [1] is merged, we can add
    # the following back:
    #
    # buildInputs ++ (lib.optionals withUnfreePkgs [
    #   segger-jlink
    #   tockloader.nrf-command-line-tools
    # ])
    #
    # shellHook = ''
    #   # TODO: This should be patched into the rpath of the respective libraries!
    #   export LD_LIBRARY_PATH=${pkgs.libusb}/lib:${pkgs.segger-jlink}/lib:$LD_LIBRARY_PATH
    # '';
    #
    # [1]: https://github.com/NixOS/nixpkgs/pull/255185
  }
