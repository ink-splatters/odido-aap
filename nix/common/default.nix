{
  imports = [
    ./rust-toolchain.nix
    ./crane-lib.nix
    ./args.nix
    ./artifacts.nix
  ];

  perSystem = {config, ...}: let
    inherit (config) craneLib commonArgs commonArgsNative cargoArtifacts cargoArtifactsNative;
  in {
    packages = {
      odido-aap = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
        });

      odido-aap-native = craneLib.buildPackage (commonArgsNative
        // {
          inherit cargoArtifactsNative;
        });
    };
  };
}
