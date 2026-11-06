# Applies the pinned Rust overlay to the nixpkgs instance every perSystem
# module sees, so `pkgs.rust-bin` resolves consistently in the devshell and
# in treefmt.
{ inputs, ... }:
{
  perSystem =
    { system, ... }:
    {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };
    };
}
