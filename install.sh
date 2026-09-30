#!/usr/bin/env bash
# Installs tunl-tui and sets up its privileges.
# Build first as your user (cargo build --release), then run: sudo ./install.sh
set -euo pipefail
cd "$(dirname "$0")"

if [[ $EUID -ne 0 ]]; then
  echo "run with sudo" >&2
  exit 1
fi

if [[ ! -x target/release/tunl-tui ]]; then
  echo "target/release/tunl-tui not found; run 'cargo build --release' first (without sudo)" >&2
  exit 1
fi

groupadd -f elsec-tunl
if [[ -n ${SUDO_USER:-} ]]; then
  usermod -aG elsec-tunl "$SUDO_USER"
fi

install -o root -g root -m 0755 target/release/tunl-tui /usr/local/bin/tunl-tui
install -o root -g root -m 0755 helper/tunl-helper /usr/local/bin/tunl-helper

visudo -cf helper/sudoers
install -o root -g root -m 0440 helper/sudoers /etc/sudoers.d/tunl-tui

echo "Installed. Log out and back in (or run 'newgrp elsec-tunl') for the group to take effect, then run 'tunl-tui'."
