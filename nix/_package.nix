# The build definition, shared by this flake's packages and by the overlay.
#
# Taking `pkgs` as an argument (rather than closing over this flake's own) is what
# lets the overlay build against the consumer's nixpkgs, so downstream can override
# and cross-compile it. That requires a committed Cargo.nix: crate2nix's
# appliedCargoNix generates at evaluation time, which would force IFD on consumers.
{ pkgs, ... }:
let
  cargoNix = import ../Cargo.nix { inherit pkgs; };
in
cargoNix.workspaceMembers.walltime.build.overrideAttrs (prev: {
  meta = (prev.meta or { }) // {
    description = "A library and CLI for measuring time spent in a process";
    mainProgram = "walltime";
  };
})
