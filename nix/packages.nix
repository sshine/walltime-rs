{ inputs, ... }:
{
  perSystem =
    { system, ... }:
    let
      cargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
        name = "walltime";
        src = ../.;
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
}
