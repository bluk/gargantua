{
  description = "A test web server which returns 404s.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    # Rust flake functions
    naersk.url = "github:nmattia/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        naersk' = pkgs.callPackage naersk { };
      in rec {
        defaultPackage = naersk'.buildPackage { src = ./.; };

        defaultApp = let
          drv = self.defaultPackage."${system}";
          name = pkgs.lib.strings.removeSuffix ("-" + drv.version) drv.name;
        in flake-utils.lib.mkApp {
          inherit drv;
          # TODO: https://github.com/nix-community/naersk/issues/224
          exePath = "/bin/${name}";
        };

        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              cargo-watch
              rustc
              rust-analyzer
              rustfmt
              rustPackages.clippy

              pkg-config
              openssl
              cmake
            ];
            RUST_LOG = "debug";
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };

      }) // {
        nixosModule = { config, lib, pkgs, ... }:
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
                default = nixpkgs.legacyPackages.${pkgs.system}.gargantua;
                # default = pkgs.gargantua;
                # default = self.packages.${pkgs.system}.default;
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
