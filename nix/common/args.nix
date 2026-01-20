{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config) craneLib;
    inherit (pkgs.llvmPackages_latest) clang bintools stdenv libcxx;

    mkFlags = flags: lib.concatStringsSep " " (map (x: "-C ${x}") flags);

    flags = [
      "linker=${clang}/bin/cc"
      "link-args=-fuse-ld=lld"
      "embed-bitcode=yes"
      "lto=thin"
    ];

    mkCommonArgs = args @ {flags, ...}:
      {
        src = craneLib.cleanCargoSource config.src;
        stdenv = _: stdenv;
        strictDeps = true;
        enableParallelBuilding = true;
        RUSTFLAGS = "-Zdylib-lto " + (mkFlags flags);

        buildInputs = [
          libcxx
        ];

        nativeBuildInputs = [
          clang
          bintools
        ];
      }
      // (builtins.removeAttrs args ["flags"]);
  in {
    options = {
      commonArgs = lib.mkOption {
        type = lib.types.attrs;
        default = mkCommonArgs {inherit flags;};
      };

      commonArgsNative = lib.mkOption {
        type = lib.types.attrs;

        default = mkCommonArgs {
          flags = flags ++ ["target-cpu=native"];
          NIX_ENFORCE_NO_NATIVE = 0;
        };
      };
    };
  };
}
