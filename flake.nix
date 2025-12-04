{
  inputs = {
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
  };

  nixConfig = {
    extra-substituters = [
      "https://aarch64-darwin.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "aarch64-darwin.cachix.org-1:mEz8A1jcJveehs/ZbZUEjXZ65Aukk9bg2kmb0zL9XDA="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} (let
      systems = import inputs.systems;
      flakeModules.default = import ./nix/flake-module.nix;
    in {
      imports = [
        flakeModules.default
        flake-parts.flakeModules.partitions
      ];

      inherit systems;

      partitionedAttrs = {
        apps = "dev";
        checks = "dev";
        devShells = "dev";
        formatter = "dev";
      };
      partitions.dev = {
        # directory containing inputs-only flake.nix
        extraInputsFlake = ./nix/dev;
        module = {
          imports = [./nix/dev];
        };
      };
      # this won't be exported
      perSystem = {
        config,
        lib,
        ...
      }: {
        options.src = lib.mkOption {
          default = builtins.path {
            path = ./.;
            name = "odido-aap";
          };
        };
        config.packages.default = config.packages.odido-aap-native;
      };

      flake = {
        inherit flakeModules;
      };
    });
}
