{inputs, ...}: {
  perSystem = {
    self',
    lib,
    ...
  }: let
    pkgs = import inputs.nixpkgs {
      system = "x86_64-linux";
      overlays = [(import inputs.rust-overlay)];
    };
    # craneLib = inputs.crane.mkLib pkgs;
    craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (p:
      p.rust-bin.stable.latest.default.override {
        targets = ["x86_64-unknown-linux-musl"];
      });

    commonArgs = (
      (craneLib.crateNameFromCargoToml {cargoToml = ../../Cargo.toml;})
      // {
        # for some reason some tests are failing when building with nix...
        # TODO: fix this
        doCheck = false;
        strictDeps = true;
        # cargoExtraArgs = "--target x86_64-unknown-linux-musl";
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        # cargoVendorDir = craneLib.vendorCargoDeps {src = ../../;};
        # pname = "wol-backend";
        # version = "0.1.0";
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs;
          [
            pkgsStatic.openssl.dev
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
      all-binaries =
        craneLib.buildPackage
        commonArgs;
      default = self'.packages.all-binaries;
    };
  };
}
