{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };

  rust-nightly = pkgs.callPackage ./nix/rust-nightly {
    date = "2016-09-28";
    hash = "1m3b29pwxpj7sw26rdq1kr1qzqkh2xv6gby8c131b8w05qyx5glg";
  };

in with pkgs; {
  bkernelEnv = stdenv.mkDerivation {
    name = "bkernel";
    buildInputs = [
      gnumake
      git
      rust-nightly
      gcc-arm-embedded
      minicom
      openocd
      expect
    ];
  };
}
