{inputs, ...}: {
  perSystem = {
    config,
    self',
    pkgs,
    ...
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = inputs.nixpkgs.lib.genAttrs supportedSystems;
  in {
    checks = forAllSystems (system: {
      pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
        src = ./.;
        hooks = {
          alejandra.enable = true;
          clippy.enable = true;
          rustfmt.enable = true;
          cargo-check.enable = true;
        };
      };
    });
    devShells.default = pkgs.mkShell {
      name = "wol-relay-server-shell";
      inputsFrom = [
        self'.devShells.rust
        config.treefmt.build.devShell
      ];
      packages = with pkgs; [
        just
        nixd # Nix language server
        bacon
        entr
        rustfmt
        cargo-nextest

        # inputs.omnix.packages # TODO:
      ];
    };
  };
}
