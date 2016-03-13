{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };

  rust-nightly = pkgs.callPackage ./nix/rust-nightly {
    date = "2016-03-11";
    hash = "0s450rm51z9gywb4vnaradvy23cqyd19yk8j4swrr3v520f4dx6b";
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
