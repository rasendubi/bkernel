{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };

  rust-nightly = pkgs.callPackage ./nix/rust-nightly {
    date = "2017-05-12";
    hash = "1xxcjmhqnzrd4ghvdby87vay95p3q4hfjlz5r6z0w1v40gx9ka3h";
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
