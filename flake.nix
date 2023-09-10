{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs, ... }:
    let
      nameValuePair = name: value: { inherit name value; };
      genAttrs = names: f: builtins.listToAttrs (map (n: nameValuePair n (f n)) names);

      pkgsFor = pkgs: system:
        import pkgs { inherit system; };

      allSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: genAttrs allSystems
        (system: f {
          inherit system;
          pkgs = pkgsFor nixpkgs system;
        });
    in
    {
      nixosModules = rec {
        default = standalone-wait-online;
        standalone-wait-online = { pkgs, ... }: {
          imports = [ ./nix/modules/standalone-wait-online.nix ];

          standalone-network-wait-online.pkg = self.packages.${pkgs.system}.default;
        };
      };
      packages = forAllSystems
        ({ pkgs, ... }: rec {
          wait-online = pkgs.callPackage ./nix/packages/wait-online.nix { };

          default = wait-online;
        });
    };
}
