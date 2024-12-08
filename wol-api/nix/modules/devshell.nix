{inputs, ...}: {
  perSystem = {
    config,
    self',
    pkgs,
    lib,
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
        config.process-compose.cargo-doc-live.outputs.package
      ];
    };
  };
}
