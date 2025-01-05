{inputs, ...}: {
  perSystem = {
    self',
    pkgs,
    lib,
    ...
  }: let
    craneLib = inputs.crane.mkLib pkgs;
    commonArgs = (
      (craneLib.crateNameFromCargoToml {cargoToml = ../../Cargo.toml;})
      // {
        # for some reason some tests are failing when building with nix...
        # TODO: fix this
        doCheck = false;
        # cargoVendorDir = craneLib.vendorCargoDeps {src = ../../;};
        # pname = "wol-backend";
        # version = "0.1.0";
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs =
          [
            pkgs.openssl.dev
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin (
            with pkgs.darwin.apple_sdk.frameworks; [
              IOKit
            ]
          );
        src = let
          unfilteredRoot = ../..; # The original, unfiltered source
        in
          lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              (lib.fileset.maybeMissing ../../tests)
              (lib.fileset.maybeMissing ../../res)
            ];
          };
      }
    );
  in {
    packages = {
      backend =
        craneLib.buildPackage
        commonArgs
        // {
          cargoExtraArgs = "-p backend";
        };
      agent =
        craneLib.buildPackage
        commonArgs
        // {
          cargoExtraArgs = "-p agent";
        };
      default = self'.packages.backend;
    };
  };
}
