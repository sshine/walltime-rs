{ ... }:
{
  flake.overlays.default = final: _prev: {
    walltime = final.callPackage ./_package.nix { };

    # The binary is `wtime`; alias so downstream can reach it under either name.
    wtime = final.walltime;
  };
}
