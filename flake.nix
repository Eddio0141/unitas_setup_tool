{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = { flake-parts, ... } @ inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
      ];
      perSystem = { self', inputs', system, pkgs, ... }:
      let
        # check https://github.com/nix-community/fenix for what toolchains are available
        toolchain = inputs'.fenix.packages.stable;

        fenix = toolchain.withComponents [
          "rustc"
          "cargo"
          "rustfmt"
          "rust-analyzer"
          "clippy"
          "rust-src"
        ];

        rust-doc = pkgs.writeShellApplication {
          name = "rust-doc";
          text = ''
            xdg-open "${toolchain.rust-docs}/share/doc/rust/html/index.html"
          '';
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            fenix rust-doc
            openssl
            cmake
            pkg-config
          ];
        };
      };
    };
}
