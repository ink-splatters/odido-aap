{
  perSystem = {config, ...}: let
    inherit (config) craneLib src commonArgs cargoArtifacts cargoArtifactsDev;
  in {
    checks = {
      inherit (config.packages) odido-aap;

      odido-aap-clippy = craneLib.cargoClippy (commonArgs
        // {
          cargoArtifacts = cargoArtifactsDev;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

      odido-aap-doc = craneLib.cargoDoc (commonArgs
        // {
          inherit cargoArtifacts;
        });

      odido-aap-fmt = craneLib.cargoFmt {
        inherit src;
      };

      odido-aap-nextest = craneLib.cargoNextest (commonArgs
        // {
          cargoArtifacts = cargoArtifactsDev;
          partitions = 1;
          partitionType = "count";
        });
    };
  };
}
