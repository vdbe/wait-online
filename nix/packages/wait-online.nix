{ pkgs ? import <nixpkgs> { }
, lib ? pkgs.lib
, nix-gitignore ? pkgs.nix-gitignore
, rustPlatform ? pkgs.rustPlatform
, ...
}:

let
  pname = "wait-online";
  version = "0.1.1";
in
rustPlatform.buildRustPackage rec {
  inherit pname version;

  # Files/directories not important to the build
  extraGitIgnore = [
    "dev/"
    "nix/"
    ".github/"
    ".gitignore"
    "bacon.toml"
    "rust-toolchain.toml"
    "flake.nix"
    "default.nix"
    "flake.lock"
  ];

  src = nix-gitignore.gitignoreSource extraGitIgnore ../../.;

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  meta = {
    homepage = "https://github.com/vdbe/wait-online/";
    description = "A program that waits untill all interfaces are up";
    platforms = lib.platforms.linux;
    license = lib.licenses.mit;
    maintainers = [ ];
  };
}
