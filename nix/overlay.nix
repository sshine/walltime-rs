{ inputs, ... }:
{
  flake.overlays.default = final: prev: {
    walltime-cli = inputs.self.packages.${final.stdenv.hostPlatform.system}.default;
  };
}
