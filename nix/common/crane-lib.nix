{
  lib,
  inputs,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    options.craneLib = lib.mkOption {
      type = lib.types.attrs;
      default = (inputs.crane.mkLib pkgs).overrideToolchain config.rust-toolchain;
    };
  };
}
