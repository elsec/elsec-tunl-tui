#!/usr/bin/env bash
# Installs elsec-tunl-tui and sets up its privileges.
# Build first as your user (cargo build --release), then run: sudo ./install.sh
set -euo pipefail
cd "$(dirname "$0")"

if [[ $EUID -ne 0 ]]; then
  echo "run with sudo" >&2
  exit 1
fi

if [[ ! -x target/release/elsec-tunl-tui ]]; then
  echo "target/release/elsec-tunl-tui not found; run 'cargo build --release' first (without sudo)" >&2
  exit 1
fi

groupadd -f elsec-tunl
if [[ -n ${SUDO_USER:-} ]]; then
  usermod -aG elsec-tunl "$SUDO_USER"
fi

install -o root -g root -m 0755 target/release/elsec-tunl-tui /usr/local/bin/elsec-tunl-tui
install -o root -g root -m 0755 priv/elsec-tunl-priv /usr/local/bin/elsec-tunl-priv

visudo -cf priv/sudoers
install -o root -g root -m 0440 priv/sudoers /etc/sudoers.d/elsec-tunl-tui

echo "Installed. Log out and back in (or run 'newgrp elsec-tunl') for the group to take effect, then run 'elsec-tunl-tui'."
