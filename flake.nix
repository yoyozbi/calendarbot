{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    fenix = {
        url = "github:nix-community/fenix";
        inputs.nixpkgs.follows = "nixpkgs";
    };
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
                  pre-commit.hooks = {
				    				clippy.enable = true;
										cargo-check.enable = true;
										rustfmt.enable = true;
										docker = {
											enable = true;
											name ="Docker linting";
											entry = "ghcr.io/hadolint/hadolint hadolint";
											files="Dockerfile";
											language="docker_image";
											pass_filenames = false;
										};
				  				};
                  # https://devenv.sh/reference/options/
                  packages = [ pkgs.openssl.dev pkgs.pkgconfig pkgs.dbus.dev pkgs.postgresql pkgs.hadolint ];
                  languages.rust = {
                    enable = true;
                    channel = "stable";
										components = ["rustfmt" "clippy"];
                  };
                  env.OPENSSL_DEV=pkgs.openssl.dev;
                }
              ];
            };
          });
    };
}
