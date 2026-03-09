{ inputs, ... }:
{
  perSystem = { pkgs, ... }:
    let
      rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;
    in
    {
      devshells.default = {
        packages = [
          rust-toolchain
          pkgs.just
          pkgs.cargo-watch
          pkgs.cargo-insta
        ];

        env = [
          { name = "RUST_BACKTRACE"; value = "1"; }
        ];
      };
    };
}
