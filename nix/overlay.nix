{ ... }:
{
  flake.overlays.default = final: _prev: {
    walltime = final.callPackage ./_package.nix { };
  };
}
