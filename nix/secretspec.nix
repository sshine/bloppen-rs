# Credentials for Phase 4 live validation.
#
# secretspec.toml declares the names; where the values come from is each
# developer's own choice. The global default here is the system keyring, so
# `just secrets-set` stores the account password outside the repository. CI can
# set SECRETSPEC_PROVIDER=env instead, exactly as the nemlig-rs live workflow
# does.
{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      devshells.default.packages = [ pkgs.secretspec ];
    };
}
