{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = inputs.nixpkgs.lib.genAttrs supportedSystems;
  in {
    # checks = forAllSystems (system: {
    #   pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
    #     src = ./.;
    #     hooks = {
    #       alejandra.enable = true;
    #       clippy.enable = true;
    #       rustfmt.enable = true;
    #       cargo-check.enable = true;
    #     };
    #   };
    # });
    devShells.default = pkgs.mkShell {
      name = "wol-relay-server-shell";
      inputsFrom = [
        config.treefmt.build.devShell
        config.packages.default
      ];
      packages = with pkgs; [
        just
        nixd # Nix language server
        bacon
        entr
        rustfmt
        cargo-nextest
        cargo-flamegraph
      ];
      env = {
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
      };
    };
  };
}
