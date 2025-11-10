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
            MD022 = false; # Don't require blank lines around headings
            MD026 = false; # Allow trailing punctuation in headings
            MD031 = false; # Don't require blank lines around fences
            MD033 = false; # Allow inline HTML
            MD040 = false; # Don't require language for code blocks
            MD041 = false; # First line doesn't need to be a heading
            MD012 = false; # Allow multiple blank lines
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
