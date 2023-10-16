# Flake definition:
# https://nixos.wiki/wiki/Flakes

{
  description = "A test web server which returns 404s.";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; };

  outputs = { self, nixpkgs }:
    let
      # Explicit list of supported systems. Add systems when necessary.
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];

      # Function which generates an attribute set: '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (system:
        import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        });
    in {
      # Used to add the package to an existing nixpkgs as an overlay.
      overlays.default = final: prev: {
        gargantua = with final;
          final.callPackage ({ inShell ? false }:
            final.rustPlatform.buildRustPackage rec {
              pname = "gargantua";
              # Version of this package (not necessarily the same version as in Cargo.toml but probably should be).
              version = "0.5.0";

              # Ignore the source code if used in 'nix develop'.
              src = if inShell then null else ./.;

              cargoLock = { lockFile = ./Cargo.lock; };

              postInstall = if inShell then null else lib.optionalString stdenv.isLinux ''
                cp -rf $src/resources $out/resources
              '';
            }) { };
      };

      legacyPackages =
        forAllSystems (system: { inherit (nixpkgsFor.${system}) gargantua; });

      packages = forAllSystems (system: {
        inherit (nixpkgsFor.${system}) gargantua;

        default = self.packages.${system}.gargantua;
      });

      # 'nix develop' environment
      devShells = forAllSystems (system: {
        default =
          self.packages.${system}.gargantua.override { inShell = true; };
      });

      nixosModules.gargantua = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.services.gargantua;

          cfgService = {
            DynamicUser = true;
            ConfigurationDirectory = "gargantua";
            ConfigurationDirectoryMode = "0750";
            RuntimeDirectory = "gargantua";
            RuntimeDirectoryMode = "0750";
            StateDirectory = "gargantua";
            StateDirectoryMode = "0750";
            LogsDirectory = "gargantua";
            LogsDirectoryMode = "0750";
            CacheDirectory = "gargantua";
            CacheDirectoryMode = "0750";
          };
        in {
          options.services.gargantua = {
            enable = mkEnableOption (lib.mdDoc "gargantua web app");

            package = lib.mkOption {
              type = lib.types.package;
              default = pkgs.gargantua;
              defaultText = lib.literalExpression "pkgs.gargantua";
              description = lib.mdDoc "Package to use.";
            };

            address = mkOption {
              type = types.str;
              default = "127.0.0.1";
              description = lib.mdDoc ''
                IPv4 address to bind to.
              '';
            };

            port = mkOption {
              type = types.port;
              default = 8080;
              description = lib.mdDoc ''
                TCP port used to listen on.
              '';
            };

            configDir = mkOption {
              type = types.path;
              description = lib.mdDoc ''
                Path to config file directory.
              '';
            };
          };

          config = lib.mkIf cfg.enable {

            environment.etc = {
              gargantua.source = cfg.configDir;
            };

            systemd.services.gargantua = {
              after = [ "network.target" ];

              environment = {
                  PORT = "${toString cfg.port}";
                  RUST_LOG = "debug";
              };
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStartPre = [
                 "-${pkgs.coreutils}/bin/chmod -R 0750 /run/gargantua"
                 "-${pkgs.coreutils}/bin/chmod -R 0750 /var/lib/gargantua"
                 "-${pkgs.coreutils}/bin/chmod -R 0750 /var/cache/gargantua"
                 "-${pkgs.coreutils}/bin/chmod -R 0750 /var/log/gargantua"
                 "-${pkgs.coreutils}/bin/rm -rf /var/lib/gargantua/static"
                 "${pkgs.coreutils}/bin/cp -r ${cfg.package}/resources/state/static /var/lib/gargantua/"
                 "${pkgs.coreutils}/bin/chmod -R 0750 /run/gargantua"
                 "${pkgs.coreutils}/bin/chmod -R 0750 /var/lib/gargantua"
                 "${pkgs.coreutils}/bin/chmod -R 0750 /var/cache/gargantua"
                 "${pkgs.coreutils}/bin/chmod -R 0750 /var/log/gargantua"
                ];
                ExecStart = "${cfg.package}/bin/gargantua";
              } // cfgService;
            };
          };
        };
    };
}
