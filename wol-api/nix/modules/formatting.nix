{inputs, ...}: {
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem = {
    config,
    self',
    pkgs,
    lib,
    ...
  }: {
    # Add your auto-formatters here.
    # cf. https://nixos.asia/en/treefmt
    treefmt.config = {
      projectRootFile = "flake.nix";
      programs = {
        alejandra.enable = true;
        rustfmt.enable = true;
      };
    };
  };
}
