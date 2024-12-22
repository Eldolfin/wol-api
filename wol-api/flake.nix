{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    systems.url = "github:nix-systems/default";
    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    rust-flake.inputs.rust-overlay.follows = "rust-overlay";
    rust-overlay.url = "github:oxalica/rust-overlay";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
    omnix.url = "github:juspay/omnix";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import inputs.systems;

      # See ./nix/modules/*.nix for the modules that are imported here.
      imports = with builtins;
        map
        (fn: ./nix/modules/${fn})
        (attrNames (readDir ./nix/modules));
    };
}
