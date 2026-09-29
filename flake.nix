{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
    rust = pkgs.rust-bin.stable.latest.default.override {
      targets = [ "thumbv6m-none-eabi" "riscv32imc-unknown-none-elf" ];
      extensions = [ "rust-src" "llvm-tools" ];
    };
  in
  {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        rust
        espflash
        elf2uf2-rs
        picotool
      ];
    };
  };
}
