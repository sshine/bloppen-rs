# The base shell: project management and the formatter wrapper. The tools
# specific to each phase are added by sibling modules (re-tools.nix, rust.nix)
# rather than growing this list.
{ inputs, ... }:
{
  imports = [ inputs.devshell.flakeModule ];
  perSystem =
    { config, pkgs, ... }:
    {
      devshells.default = {
        packages = [
          config.treefmt.build.wrapper
          pkgs.git
          pkgs.just
          pkgs.curl
          pkgs.wget
          pkgs.openssl
          pkgs.pkg-config
        ];

        env = [
          {
            name = "RUST_BACKTRACE";
            value = "1";
          }
        ];

        devshell.motd = "";
      };
    };
}
