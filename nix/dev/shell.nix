{
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) pre-commit craneLib;
  in {
    devShells.default =
      craneLib.devShell.override {
        mkShell = pkgs.mkShell.override {
          inherit (pkgs.llvmPackages_latest) stdenv;
        };
      } ({
          inherit (config) checks;

          packages = [pkgs.mdformat] ++ pre-commit.settings.enabledPackages;

          shellHook = ''
            ${pre-commit.installationScript}
          '';
        }
        // (builtins.removeAttrs config.commonArgsNative ["src" "stdenv"]));
  };
}
