{
  description = "A library and CLI for measuring time spent in a process";

  nixConfig = {
    extra-substituters = [ "https://crate2nix.cachix.org" ];
    extra-trusted-public-keys = [
      "crate2nix.cachix.org-1:bXMeMOBI39htMnFaFj5MkBczuNKDfTwBBzHbPmcJ+lE="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.devshell.follows = "devshell";
      inputs.flake-parts.follows = "flake-parts";
      inputs.cachix.follows = "crate2nix/crate2nix_stable/cachix";
      inputs.flake-compat.follows = "crate2nix/crate2nix_stable/flake-compat";
      inputs.nix-test-runner.follows = "crate2nix/crate2nix_stable/nix-test-runner";
      inputs.pre-commit-hooks.follows = "crate2nix/crate2nix_stable/pre-commit-hooks";
      inputs.crate2nix_stable.inputs.nixpkgs.follows = "nixpkgs";
      inputs.crate2nix_stable.inputs.devshell.follows = "devshell";
      inputs.crate2nix_stable.inputs.flake-parts.follows = "flake-parts";
      inputs.crate2nix_stable.inputs.cachix.inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    lefthook-nix = {
      url = "github:sudosubin/lefthook.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      imports = [
        inputs.devshell.flakeModule
        inputs.treefmt-nix.flakeModule
        ./nix/rust-overlay/flake-module.nix
        ./nix/treefmt/flake-module.nix
        ./nix/devshell/flake-module.nix
      ];

      flake.overlays.default = final: prev: {
        walltime-cli = inputs.self.packages.${final.stdenv.hostPlatform.system}.default;
      };

      perSystem =
        { pkgs, system, ... }:
        let
          cargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
            name = "walltime";
            src = ./.;
          };
        in
        {
          checks = {
            walltime-core = cargoNix.workspaceMembers.walltime-core.build;
            walltime-cli = cargoNix.workspaceMembers.walltime-cli.build;
          };

          packages = {
            default = cargoNix.workspaceMembers.walltime-cli.build;
          };

          apps.default = {
            type = "app";
            program = "${cargoNix.workspaceMembers.walltime-cli.build}/bin/wtime";
          };
        };
    };
}
