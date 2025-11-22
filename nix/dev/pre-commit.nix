{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    pre-commit = {
      check.enable = true;

      settings.hooks = {
        # TOML/Cargo files
        check-toml.enable = true;

        # Markdown
        markdownlint = {
          enable = true;
          settings.configuration = {
            MD013 = false; # Disable line length
            MD033 = false; # Allow inline HTML
            MD040 = false; # Don't require language for code blocks
          };
        };

        # Spell checking
        typos.enable = true;

        # Nix hooks
        deadnix.enable = true;
        nil.enable = true;
        alejandra.enable = true;
        statix.enable = true;
      };
    };

    apps.install-hooks = {
      type = "app";
      program = toString (pkgs.writeShellScript "install-hooks" ''
        ${config.pre-commit.installationScript}
        echo "Pre-commit hooks installed!"
      '');
      meta.description = "install pre-commit hooks";
    };
  };
}
