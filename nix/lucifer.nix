self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.lucifer;
in {
  options.services.lucifer = {
    enable = lib.mkEnableOption "Lucifer DNS proxy";

    configFile = lib.mkOption {
      type = lib.types.str;
      default = "/lucifer/lucifer.toml";
      description = "Path to Confile relative to /etc";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 53;
      description = "Port to listen on";
    };

    nameservers = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = ["1.1.1.1"];
      description = "Upstream nameservers";
    };

    blocklist = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "Domains to block";
    };

    package = lib.mkOption {
      type = lib.types.package;
      description = "The Lucifer package";
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.lucifer;
    };
  };

  config = lib.mkIf cfg.enable {
    environment.etc.${cfg.configFile}.source = (pkgs.formats.toml {}).generate "lucifer.toml" {
      proxy = {
        inherit (cfg) port nameservers blocklist;
      };
    };

    networking.nameservers = ["127.0.0.1"];

    systemd.services.lucifer = {
      description = "Lucifer DNS proxy";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/lucifer --config /etc${cfg.configFile}";
        Restart = "always";
        RestartSec = "10";
        AmbientCapabilities = "CAP_NET_BIND_SERVICE";
        CapabilityBoundingSet = "CAP_NET_BIND_SERVICE";
        DynamicUser = true;
      };
    };
  };
}
