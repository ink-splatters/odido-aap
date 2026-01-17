{lib, ...}: {
  perSystem = {inputs', ...}: {
    options.rust-toolchain = lib.mkOption {
      type = lib.types.attrs;

      default = with inputs'.fenix.packages;
      with default;
        combine [
          cargo
          clippy
          rust-std
          rustc
          rustc-unwrapped
          rustfmt
        ];
    };
  };
}
