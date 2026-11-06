# One formatter set for every language this repository will contain. Generated
# reverse-engineering output is excluded because it is not source: it is
# regenerated from the APK, and formatting it would only create noisy diffs.
{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];
  perSystem =
    { pkgs, ... }:
    let
      rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
    in
    {
      treefmt = {
        projectRootFile = "flake.nix";

        settings.global.excludes = [
          # Backlog.md owns the generated block at the top and writes raw HTML
          # tags, which mdformat escapes into non-instructions.
          "AGENTS.md"
          # Backlog.md owns task, draft, document, and decision files; the CLI
          # is responsible for their layout.
          "backlog/**"
          "apk/**"
          "secrets/**"
          "re/decompiled/**"
          "re/extracted/**"
          "re/reports/**"
          ".direnv/**"
          "target/**"
          "result"
        ];

        programs.nixfmt.enable = true;
        programs.rustfmt = {
          enable = true;
          package = rust-toolchain;
        };
        programs.mdformat = {
          enable = true;
          # A skill's SKILL.md opens with YAML frontmatter; teach mdformat that
          # syntax rather than excluding every skill tree.
          plugins = ps: [ ps.mdformat-frontmatter ];
          settings.number = true;
        };
        programs.taplo.enable = true;
        programs.yamlfmt = {
          enable = true;
          settings.formatter.retain_line_breaks_single = true;
        };
        programs.shfmt.enable = true;
      };
    };
}
