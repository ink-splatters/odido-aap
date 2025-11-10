{
  perSystem = {config, ...}: let
    inherit (config) craneLib src commonArgs cargoArtifacts;
  in {
    checks = {
      inherit (config.packages) odido-aap;

      odido-aap-clippy = craneLib.cargoClippy (
        commonArgs
        // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        }
      );

      odido-aap-doc = craneLib.cargoDoc (
        commonArgs
        // {
          inherit cargoArtifacts;
        }
      );

      odido-aap-fmt = craneLib.cargoFmt {
        inherit src;
      };

      # TODO: no tests to run for now
      # odido-aap-nextest = craneLib.cargoNextest (
      #   commonArgs
      #   // {
      #     inherit cargoArtifacts;
      #     partitions = 1;
      #     partitionType = "count";
      #   }
      # );
    };
  };
}
