{lib, ...}: {
  perSystem = {config, ...}: let
    inherit (config) craneLib commonArgs commonArgsNative;
  in {
    options = {
      # Production artifacts - minimal dependencies
      cargoArtifacts = lib.mkOption {
        type = lib.types.package;
        default = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "odido-aap";
          });
      };

      cargoArtifactsNative = lib.mkOption {
        type = lib.types.package;
        default = craneLib.buildDepsOnly (commonArgsNative
          // {
            pname = "odido-aap-native";
          });
      };

      # Development artifacts - includes test/bench deps for clippy offline mode
      cargoArtifactsDev = lib.mkOption {
        type = lib.types.package;
        default = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "odido-aap-dev";
            cargoCheckExtraArgs = "--all-targets";
          });
      };
    };
  };
}
