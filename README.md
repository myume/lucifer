# lucifer

dns blocker

## Usage

### Nixos

For nixos just add this to your flake inputs:

```nix
lucifer.url = "github:myume/lucifer";
```

and import this in your configuration.nix:

```nix
imports = [
    inputs.lucifer.nixosModules.lucifer
]
```

You can then enable it in your configuration as follows

```nix
services.lucifer = {
  enable = true;
  nameservers = ["1.1.1.1"];
  blocklist = [
    "youtube.com"
  ];
};
```

### Linux

If you're not on nix, you're going to have to set it up manually, which is going
to be slightly more painful.

First off you need to build the app with `cargo build --release`. You then need
to set up a config file named `lucifer.toml` somewhere

```toml
[proxy]
nameservers = ["1.1.1.1"]
blocklist = [
  "youtube.com"
]
```

You can either run it manually with
`sudo /path/to/lucifer/bin --config /path/to/config` or set this up as a
service.

You can use the nix service as a guide

```nix
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
```

In addition to this, you need to add `nameserver 127.0.0.1` to your
`/etc/resolv.conf` file. It's not advised to do this manually as there is
usually a pre-existing service that handles the management of this file, figure
out what it is and set it up accordingly.

If this seems like a lot of work to you, you can just simply set up sinkholes in
your `/etc/hosts` file instead. Map `0.0.0.0` to whatever domain you want to
block.

Good luck!
