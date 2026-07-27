{ ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      cargoNix = import ../Cargo.nix { inherit pkgs; };
    in
    rec {
      # Building each member separately catches breakage in walltime-core that the
      # binary's own code paths happen not to exercise.
      checks = {
        walltime-core = cargoNix.workspaceMembers.walltime-core.build;
        walltime-cli = packages.default;
      };

      packages.default = pkgs.callPackage ./_package.nix { };

      apps.default = {
        type = "app";
        program = lib.getExe packages.default;
      };
    };
}
