{
  description = "";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt.url = "github:numtide/treefmt-nix";
    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    advisory-db.url = "github:rustsec/advisory-db";
    advisory-db.flake = false;
  };

  outputs = inputs @ {
    flake-parts,
    treefmt,
    git-hooks,
    crane,
    advisory-db,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"];

      imports = [
        treefmt.flakeModule
        git-hooks.flakeModule
      ];

      perSystem = {
        config,
        pkgs,
        lib,
        ...
      }: let
        craneLib = crane.mkLib pkgs;
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources ./.)
            (lib.fileset.maybeMissing ./src/snapshots)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = [] ++ lib.optionals pkgs.stdenv.isDarwin [pkgs.libiconv];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        odata = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            meta.mainProgram = "odata";
          });
      in {
        checks = {
          inherit odata;

          odata-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          odata-doc = craneLib.cargoDoc (commonArgs // {inherit cargoArtifacts;});
          odata-audit = craneLib.cargoAudit {inherit src advisory-db;};
          odata-deny = craneLib.cargoDeny {inherit src;};

          odata-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages.default = odata;

        apps.default = {
          type = "app";
          program = lib.getExe odata;
          meta.description = "";
        };

        treefmt.programs = {
          alejandra.enable = true; # nix
          rustfmt.enable = true; # rust
          taplo.enable = true; # toml
        };

        pre-commit.settings.hooks = {
          treefmt.enable = true;
          treefmt.package = config.treefmt.build.wrapper;
        };

        devShells.default = craneLib.devShell {
          checks = config.checks;

          shellHook = ''
            ${config.pre-commit.installationScript}
          '';

          packages = with pkgs;
            [
              rust-analyzer # rust lsp
              nil # nix lsp
            ]
            ++ config.pre-commit.settings.enabledPackages
            ++ lib.attrValues config.treefmt.build.programs;
        };
      };
    };
}
