{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };

  rust-nightly = pkgs.callPackage ./nix/rust-nightly {
    date = "2017-04-07";
    hash = "1f9ssvfgygxf2gl6ysxfl8cn758mhwz7q4ahjj8wpi5qz2h3mz14";
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
