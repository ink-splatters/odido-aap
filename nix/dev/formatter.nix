{
  perSystem = {
    pkgs,
    config,
    ...
  }: {
    formatter = pkgs.writeShellScriptBin "fmt-all" ''
      ${pkgs.alejandra}/bin/alejandra .

      echo "Formatting Rust files..."
      ${config.rust-toolchain}/bin/cargo fmt --all

      echo "Formatting markdown files..."
      ${pkgs.fd}/bin/fd '\.md$' -x ${pkgs.mdformat}/bin/mdformat
    '';
  };
}
