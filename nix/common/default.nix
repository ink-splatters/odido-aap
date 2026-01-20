{lib, ...}: {
  imports = [
    ./rust-toolchain.nix
    ./crane-lib.nix
    ./args.nix
    ./artifacts.nix
  ];

  perSystem = {config, ...}: let
    inherit (config) craneLib commonArgs commonArgsNative cargoArtifacts cargoArtifactsNative;

    meta = {
      description = "Odido AAP CLI tool";
      license = lib.licenses.mit;
      mainProgram = "odido";
    };
  in {
    packages = {
      odido-aap = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts meta;
        });

      odido-aap-native = craneLib.buildPackage (commonArgsNative
        // {
          cargoArtifacts = cargoArtifactsNative;
          inherit meta;
        });
    };
  };
}
