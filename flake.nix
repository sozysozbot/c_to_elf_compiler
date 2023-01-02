{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgsArgs = {
          inherit system;
        };
        pkgs = import nixpkgs pkgsArgs;
        linuxPkgs =
          if system == "x86_64-linux"
          then pkgs.pkgsStatic
          else (import nixpkgs {
            system = if system == "aarch64-darwin" then "x86_64-darwin" else system;
          }).pkgsCross.musl64.pkgsStatic;
      in
      rec {
        packages = {
        };
        devShell =
          with pkgs; mkShell {
            nativeBuildInputs = [
              gnumake
              pkg-config
              linuxPkgs.stdenv.cc.bintools.bintools_bin
              nasm
            ];
            buildInputs = lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.SystemConfiguration
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.Security
            ] ++ lib.optionals stdenv.isLinux [
              glibc
            ];
            LINUX_LIBC = linuxPkgs.stdenv.cc.libc;
            BINTOOLS_PREFIX = if system == "x86_64-linux" then "" else "x86_64-unknown-linux-musl-";
          };
        }
    );
}
