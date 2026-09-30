#!/usr/bin/env bash
# One-time setup for tunl-tui privileges. Run with: sudo ./install.sh
set -euo pipefail
cd "$(dirname "$0")"

if [[ $EUID -ne 0 ]]; then
  echo "run with sudo" >&2
  exit 1
fi

groupadd -f wireguard
if [[ -n ${SUDO_USER:-} ]]; then
  usermod -aG wireguard "$SUDO_USER"
fi

install -o root -g root -m 0755 helper/tunl-helper /usr/local/bin/tunl-helper

visudo -cf helper/sudoers
install -o root -g root -m 0440 helper/sudoers /etc/sudoers.d/tunl-tui

echo "Installed. Log out and back in (or run 'newgrp wireguard') for the group to take effect."
