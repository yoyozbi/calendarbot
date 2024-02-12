{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
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
                  # https://devenv.sh/reference/options/
                  packages = [ pkgs.openssl.dev pkgs.pkgconfig pkgs.dbus.dev ];
                  languages.rust = {
                    enable = true;
                  };
                  services.postgres = {
                      enable = true;
                      package = pkgs.postgresql_15;
                      initialDatabases = [{ name = "mydb"; }];
                      extensions = extensions: [
                        extensions.postgis
                        extensions.timescaledb
                      ];
                      settings.shared_preload_libraries = "timescaledb";
                      initialScript = "CREATE EXTENSION IF NOT EXISTS timescaledb;";
                  };
                  env.OPENSSL_DEV=pkgs.openssl.dev;

                }
              ];
            };
          });
    };
}
