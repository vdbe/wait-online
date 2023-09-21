{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Devenv
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    nix2container.url = "github:nlewo/nix2container";
    nix2container.inputs.nixpkgs.follows = "devenv/nixpkgs";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "devenv/nixpkgs";
  };


  outputs = inputs@{ systems, devenv, fenix, ... }:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devenv.flakeModule
      ];
      systems = import systems;

      perSystem = { config, system, pkgs, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            fenix.overlays.default
          ];
        };

        checks = {
          pre-commit = config.devenv.shells.default.pre-commit.run;
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
            rust =
              let
                inherit (pkgs) fenix;

                fileToolChain = fenix.fromToolchainFile {
                  dir = ./..;
                };

                nightlyFileToolChain = fileToolChain // {
                  toolchain = fenix.fromToolchainOf { channel = "nightly"; };
                };

                toolchain = fenix.combine [ fileToolChain nightlyFileToolChain ];

              in
              {
                enable = true;
                inherit toolchain;
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

            # Makefile
            checkmake.enable = true;
          };
        };
      };
    };
}
