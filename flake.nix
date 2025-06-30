{
  description = "Minimal development environment for muOS";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.fenix.url = "github:nix-community/fenix";

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        rustToolchain = fenix.packages.${system}.stable.toolchain;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            qemu
            rustToolchain
          ];

        };
      });
}
