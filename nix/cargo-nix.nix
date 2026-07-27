# Regenerates Cargo.nix, or verifies it is in sync with Cargo.toml/Cargo.lock.
#
# Committing Cargo.nix is what keeps the overlay free of IFD, but it can drift
# whenever dependencies change, so pre-push and CI check it (see nix/hooks.nix).
{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      packages.cargo-nix = pkgs.writeShellApplication {
        name = "cargo-nix";
        runtimeInputs = [
          pkgs.crate2nix
          pkgs.git
          pkgs.diffutils
          pkgs.coreutils
        ];
        text = ''
          root="$(git rev-parse --show-toplevel)"
          cd "$root"

          if [ "''${1:-}" != "--check" ]; then
            crate2nix generate
            exit 0
          fi

          # Generate beside the real file, not in a temp dir: crate2nix writes crate
          # source paths relative to the output, so anywhere else yields absolute
          # ../../home/... paths that never match.
          tmp="$root/.Cargo.nix.check"
          hashes="$root/.crate-hashes.check.json"
          trap 'rm -f "$tmp" "$hashes"' EXIT
          crate2nix generate -o "$tmp" -h "$hashes" >/dev/null

          # crate2nix records its own argv in a 3-line header, so -o always differs there.
          if diff <(tail -n +4 Cargo.nix) <(tail -n +4 "$tmp"); then
            echo "Cargo.nix is up to date."
          else
            echo "Cargo.nix is out of date. Run 'just cargo-nix' and commit the result." >&2
            exit 1
          fi
        '';
      };
    };
}
