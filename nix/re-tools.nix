# APK acquisition, unpacking, decompilation, and binary inspection.
#
# This is deliberately a separate dendritic module: the reverse-engineering
# toolchain is useful before any Rust crate exists, and Phase 2 should not have
# to wait for the Rust toolchain to finish evaluating.
{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      devshells.default.packages = with pkgs; [
        # Acquisition (Google Play needs an AAS token; APKPure does not).
        apkeep

        # XAPK/APK unpacking and file census.
        unzip
        zip
        p7zip
        file
        tree
        findutils
        coreutils
        binwalk

        # Android resources, manifest, DEX, and signing.
        apktool
        jadx
        dex2jar
        android-tools
        aapt
        apksigner
        jdk

        # Native libraries and raw binaries. radare2 is the lightweight stand-in
        # for Ghidra; add Ghidra as a backlog task if the app turns out to have
        # substantial JNI code.
        radare2
        binutils
        util-linux

        # Text and structured data.
        ripgrep
        jq
        xmlstarlet

        # Extraction and analysis scripts. No pip; use nixpkgs packages.
        python3
        python3Packages.requests
        python3Packages.lxml
      ];
    };
}
