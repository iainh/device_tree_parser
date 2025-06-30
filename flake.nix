{
  description = "Development environment for device_tree_parser Rust library";

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
            # Rust toolchain with all components
            rustToolchain
            
            # Core development tools
            git
            
            # Device tree development
            qemu  # For generating DTB test data
            
            # Benchmark and report viewing
            # Note: bench.sh handles browser detection automatically
          ];

          # Environment setup
          shellHook = ''
            echo "🦀 Device Tree Parser Development Environment"
            echo "============================================="
            echo "Rust toolchain: $(rustc --version)"
            echo "Available commands:"
            echo "  cargo build    - Build the library"
            echo "  cargo test     - Run tests"
            echo "  cargo bench    - Run benchmarks"
            echo "  ./bench.sh     - Benchmark runner script"
            echo "  cargo clippy   - Run linter"
            echo "  cargo fmt      - Format code"
            echo ""
          '';
        };
      });
}
