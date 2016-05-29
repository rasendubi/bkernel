{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };

  rust-nightly = pkgs.callPackage ./nix/rust-nightly {
    date = "2016-05-28";
    hash = "0f9rx672v97f5bn6mnb1dgyczyf5f8vcjp55yvasflvln1w64krv";
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
