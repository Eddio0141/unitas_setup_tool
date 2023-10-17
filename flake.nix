{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });
      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  packages = with pkgs; [ openssl ];

                  languages.rust.enable = true;

                  pre-commit = {
                    settings.clippy = {
                      allFeatures = true;
                      offline = false;
                    };
                    hooks = {
                      rustfmt.enable = true;
                      clippy.enable = true;
                      # cargo test
                      "cargo-test" = {
                        enable = true;
                        name = "cargo test";
                        description = "Run cargo test";
                        entry = "${pkgs.cargo}/bin/cargo test";
                        fail_fast = true;
                        pass_filenames = false;
                        stages = [ "manual" ];
                      };
                    };
                  };
                }
              ];
            };
          });
    };
}