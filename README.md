# wait-online

`wait-online` is a standalone replacement for `systemd-networkd-wait-online`/`nm-online`.

## Installation

### Building from source

#### Dependencies

- make
- cargo
- rustc

```
make
sudo make install
sudo systemctl enable network-standalone-wait-online.service
```

## Usage in NixOS (Flakes)

### Install wait-online
```nix
inputs.standalone-network-wait-online = "github:vdbe/wait-online";
#inputs.standalone-network-wait-online.inputs.nixpkgs.follows = "nixpkgs";

outputs = { self, nixpkgs, standalone-network-wait-online, ... }: {
  nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
    system = "x86_64-linux";
    modules = [
      standalone-network-wait-online.nixosModules.default
      ./configuration.nix
    ];

    standalone-network-wait-online.enable = true;
  };
};
```

### Options

All options can be found under `standalone-network-wait-online`.

- enable
- requiredInterfaces
- ignoredInterfaces
- requireIpv4
- requireIpv6
- anyInterface
- timout
- interval

See the [module file](nix/modules/standalone-wait-online.nix) for more info
or `wait-online --help`.
