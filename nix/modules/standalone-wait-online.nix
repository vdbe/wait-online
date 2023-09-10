{ config, options, lib, pkgs, utils, ... }:
let
  inherit (builtins) toString map;
  inherit (lib) types;
  inherit (lib.options) mkOption mkEnableOption;
  inherit (lib.modules) mkIf;
  inherit (lib.lists) optional;

  opt = options.standalone-network-wait-online;
  cfg = config.standalone-network-wait-online;
in
{
  options.standalone-network-wait-online = {
    enable = mkEnableOption "Enable the Standalone wait-online service";
    pkg = mkOption {
      type = types.package;
      default = pkgs.wait-online;
      description = lib.mDoc ''
        wait-online package to use.
      '';
    };
    requiredInterfaces = mkOption {
      description = lib.mdDoc ''
        Network interfaces to be required when deciding if the system is online.

        Can't be combined with `ignoredInterfaces`.
      '';
      type = with types; listOf str;
      default = [ ];
      example = [ "enp2s0" ];

    };
    ignoredInterfaces = mkOption {
      description = lib.mdDoc ''
        Network interfaces to be ignored when deciding if the system is online.

        Can't be combined with `requiredInterfaces`.
      '';
      type = with types; listOf str;
      default = [ ];
      example = [ "enp2s0" ];

    };
    requireIpv4 = mkOption {
      description = lib.mdDoc ''
        Whether to require an IPv4 address for an interface to be considered online.
      '';
      type = types.bool;
      default = false;
    };
    requireIpv6 = mkOption {
      description = lib.mdDoc ''
        Whether to require an IPv4 address for an interface to be considered online.
      '';
      type = types.bool;
      default = false;
    };
    anyInterface = mkOption {
      description = lib.mdDoc ''
        Whether to consider the network online when any interface is online, as opposed to all of them.
        This is useful on portable machines with a wired and a wireless interface, for example.
      '';
      type = types.bool;
      default = false;
    };
    timeout = mkOption {
      description = lib.mdDoc ''
        Time to wait for the network to come online, in seconds. Set to 0 to disable.
      '';
      type = types.ints.unsigned;
      default = 120;
      example = 0;
    };
    interval = mkOption {
      description = lib.mdDoc ''
        Time between checks to see if the network is online (in ms).
        Must be between 10 and 10000.
      '';
      type = types.ints.unsigned;
      example = 50;
    };

    extraArgs = mkOption {
      description = lib.mdDoc ''
        Extra command-line arguments to pass to standalone-network-wait-online.
      '';
      type = with types; listOf str;
      default = [ ];
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = !(cfg.requiredInterfaces != [ ] && cfg.ignoredInterfaces != [ ]);
        message = ''
          standalone-network-wait-online.ignoredInterfaces and standalone-network-wait-online.ignoredInterfaces
          can't be used at the same time
        '';
      }
      {
        assertion = !opt.interval.isDefined || (10 <= cfg.interval && cfg.interval >= 10000);
        message = ''
          standalone-network-wait-online.interval must be between 10 and 10000 milliseconds.
        '';
      }
    ];

    standalone-network-wait-online.extraArgs = [ "--timeout=${toString cfg.timeout}" ]
      ++ optional opt.interval.isDefined "--interval=${toString cfg.interval}"
      ++ optional cfg.anyInterface "--any"
      ++ optional cfg.requireIpv6 "--ipv6"
      ++ optional cfg.requireIpv4 "--ipv4"
      ++ map (i: "--ignore=${i}") cfg.ignoredInterfaces
      ++ map (i: "--interface=${i}") cfg.requiredInterfaces;

    systemd.services."network-standalone-wait-online" = {
      enable = true;

      # [Unit]
      description = "Wait for Network to be Configured";
      conflicts = [ "shutdown.target" ];
      bindsTo = [ "network.target" ];
      after = [ "network.target" ];
      before = [ "network-online.target" "shutdown.target" ];
      unitConfig = {
        DefaultDependencies = "no";
      };

      # [Service]
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${cfg.pkg}/bin/wait-online ${utils.escapeSystemdExecArgs cfg.extraArgs}";
        RemainAfterExit = "yes";
      };

      # [Install]
      wantedBy = [ "network-online.target" ];
    };
  };
}
