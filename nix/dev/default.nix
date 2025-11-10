{inputs, ...}: {
  imports = [
    inputs.git-hooks.flakeModule
    ./checks.nix
    ./formatter.nix
    ./pre-commit.nix
    ./shell.nix
  ];
}
