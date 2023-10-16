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

              # cargoBuildFlags = [ "--features tracing-journald"];

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
    };
}
