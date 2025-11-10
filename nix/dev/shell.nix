{
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) pre-commit craneLib;
  in {
    devShells.default = craneLib.devShell {
      inherit (config) checks;

      packages = [pkgs.mdformat] ++ pre-commit.settings.enabledPackages;

      shellHook = ''
        ${pre-commit.installationScript}
      '';
    };
  };
}
