# tunl-tui

A minimal TUI for bringing `wg-quick` tunnels up/down and viewing their status.

## Setup

```sh
sudo ./install.sh        # creates `wireguard` group, installs helper + sudoers rule
newgrp wireguard         # or log out/in
cargo build --release
./target/release/tunl-tui
```

`install.sh` installs `/usr/local/bin/tunl-helper` and `/etc/sudoers.d/tunl-tui`, which lets
members of the `wireguard` group run only that helper as root without a password. The helper
only accepts tunnel names that exist in `/etc/wireguard` and strips private/preshared keys from
status output.

## Running without installing

Set `TUNL_HELPER` to use the helper from the repo, running as root:

```sh
sudo TUNL_HELPER=./helper/tunl-helper ./target/release/tunl-tui
```

## Keys

`j`/`k` or arrows move · `enter`/`space` toggle up/down · `r` refresh · `q`/`esc` quit
