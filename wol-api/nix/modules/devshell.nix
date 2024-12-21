{inputs, ...}: {
  perSystem = {
    config,
    self',
    pkgs,
    ...
  }: {
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

        # inputs.omnix.packages # TODO:
      ];
    };
  };
}
