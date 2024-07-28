{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-formatter-pack = {
      url = "github:Gerschtli/nix-formatter-pack";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs =
    { self
    , nixpkgs
    , devenv
    , systems
    , nix-formatter-pack
    , ...
    } @ inputs:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });
      formatter = nixpkgs.lib.genAttrs supportedSystems (system:
        nix-formatter-pack.lib.mkFormatter {
          pkgs = nixpkgs.legacyPackages.${system};
          config.tools = {
            alejandra.enable = true;
            deadnix.enable = true;
            nixpkgs-fmt.enable = true;
            statix.enable = true;
          };
        });
      devShells =
        forEachSystem
          (system:
            let
              pkgs = nixpkgs.legacyPackages.${system};
            in
            {
              default = devenv.lib.mkShell {
                inherit inputs pkgs;
                modules = [
                  {
                    pre-commit.hooks = {
                      cargo-check.enable = true;
                      rustfmt.enable = true;
                      docker = {
                        enable = true;
                        name = "Docker linting";
                        entry = "ghcr.io/hadolint/hadolint hadolint ./Dockerfile";
                        files = "Dockerfile";
                        language = "docker_image";
                        pass_filenames = false;
                      };
                    };
                    # https://devenv.sh/reference/options/
                    packages = [ pkgs.openssl.dev pkgs.pkg-config pkgs.dbus.dev pkgs.postgresql pkgs.hadolint ];
                    dotenv.disableHint = true;
                    languages.rust = {
                      enable = true;
                      channel = "stable";
                      components = [ "rustfmt" "clippy" ];
                    };
                    env.OPENSSL_DEV = pkgs.openssl.dev;
                  }
                ];
              };
            });
    };
}
