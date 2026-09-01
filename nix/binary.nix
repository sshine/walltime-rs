# Prebuilt release binaries, for consumers who want walltime without a Rust toolchain
# entering their build closure.
#
# The pinned version deliberately trails Cargo.toml: it names the last tag whose
# artifacts are published, not whatever the working tree happens to claim. Bump it
# once a tag's Release workflow has finished, with `just dist-hashes` for the values.
#
# An empty `hashes` means no tag has been released yet and `packages.walltime-bin` is
# absent; the same guard also keeps it off Darwin, which the release does not build.
{ ... }:
{
  perSystem =
    {
      system,
      pkgs,
      lib,
      ...
    }:
    let
      release = {
        version = "0.2.0";
        hashes = { };
      };

      triple = {
        "x86_64-linux" = "x86_64-unknown-linux-musl";
        "aarch64-linux" = "aarch64-unknown-linux-musl";
      };

      tag = "v${release.version}";

      tarball = pkgs.fetchurl {
        url = "https://github.com/sshine/walltime-rs/releases/download/${tag}/walltime-${tag}-${triple.${system}}.tar.gz";
        hash = release.hashes.${system};
      };
    in
    lib.optionalAttrs (release.hashes ? ${system}) {
      packages.walltime-bin =
        pkgs.runCommand "walltime-bin-${release.version}"
          {
            meta = {
              description = "CLI for measuring time spent in a process (prebuilt)";
              mainProgram = "walltime";
              license = with lib.licenses; [
                mit
                asl20
              ];
              platforms = lib.attrNames release.hashes;
              sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
            };
          }
          ''
            tar -xzf ${tarball}
            install -Dm755 walltime $out/bin/walltime
          '';
    };
}
