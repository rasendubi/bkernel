{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };

in with pkgs; {
  bkernelEnv = mkShell {
    name = "bkernel";
    buildInputs = [
      gdb-multitarget
      gnumake
      git
      rustup
      gcc-arm-embedded
      minicom
      openocd
      expect
    ];
  };
}
