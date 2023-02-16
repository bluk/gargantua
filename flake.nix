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

              # Use cargoLock above instead of manual hashes.
              # Use:
              # ```
              # cargoSha256 = lib.fakeSha256;
              # ```
              # initially and then build the package. Get the correct value from
              # the error message.
              #cargoSha256 =
              #  "sha256-a0Fe3GT9dR74W3R56hXDChOioVZWl2A57MbkhCkS/bE=";
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
            User = cfg.user;
            Group = cfg.group;
            StateDirectory = "gargantua";
            StateDirectoryMode = "0750";
            LogsDirectory = "gargantua";
            LogsDirectoryMode = "0750";
          };
        in {
          options.services.gargantua = {
            enable = mkEnableOption (lib.mdDoc "gargantua web app");

            package = lib.mkOption {
              type = lib.types.package;
              # default = pkgs.gargantua;
              # default = pkgs.legacyPackages.${pkgs.system}.gargantua;
              default = self.packages.${pkgs.system}.default;
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
              default = 8080;
              type = types.port;
              description = lib.mdDoc ''
                TCP port used to listen on.
              '';
            };

            user = lib.mkOption {
              default = "gargantua";
              type = lib.types.str;
              description = lib.mdDoc ''
                User which the service will run as. If it is set to "gargantua", that
                user will be created.
              '';
            };

            group = lib.mkOption {
              default = "gargantua";
              type = lib.types.str;
              description = lib.mdDoc ''
                Group which the service will run as. If it is set to "gargantua", that
                group will be created.
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.gargantua = {
              after = [ "network.target" ];

              environment = { PORT = "${toString cfg.port}"; };
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStart = "${cfg.package}/bin/gargantua";
              } // cfgService;
            };

            users.users = lib.mkMerge [
              (lib.mkIf (cfg.user == "gargantua") {
                gargantua = {
                  isSystemUser = true;
                  home = cfg.package;
                  inherit (cfg) group;
                };
              })
              (lib.attrsets.setAttrByPath [ cfg.user "packages" ]
                [ cfg.package ])
            ];
          };
        };
    };
}
