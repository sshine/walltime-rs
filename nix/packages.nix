{ ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    rec {
      checks.walltime = packages.default;

      packages.default = pkgs.callPackage ./_package.nix { };

      apps.default = {
        type = "app";
        program = lib.getExe packages.default;
      };
    };
}
