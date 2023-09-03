{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Devenv
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "devenv/nixpkgs";
  };


  outputs = inputs@{ systems, devenv, fenix, ... }:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devenv.flakeModule
      ];
      systems = import systems;

      perSystem = { system, pkgs, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            fenix.overlays.default
          ];
        };

        devenv.shells.default = {
          packages = with pkgs; [
            # Tools
            bacon
            cargo-expand
            cargo-hack
            cargo-msrv
            cargo-show-asm

            # Language servers
            yaml-language-server
            taplo
            nixpkgs-fmt
          ];

          languages = {
            rust = {
              enable = true;
              toolchain = {
                rustc = pkgs.fenix.fromToolchainFile {
                  dir = ./..;
                };
              };
            };
            nix.enable = true;
          };

          pre-commit.hooks = {
            # Rust
            cargo-check.enable = true;
            clippy.enable = true;
            rustfmt.enable = true;

            # Nix
            deadnix.enable = true;
            nixpkgs-fmt.enable = true;
            nil.enable = true;

            # Yaml
            yamllint.enable = true;
          };
        };
      };
    };
}
