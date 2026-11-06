# The Rust toolchain is only needed once Phase 5 starts, but pinning it now
# keeps the workspace reproducible and lets treefmt use the same rustfmt.
{ ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
    in
    {
      devshells.default.packages = [ rust-toolchain ];
    };
}
