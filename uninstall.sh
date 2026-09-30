#!/usr/bin/env bash
# Removes everything install.sh set up. Run with: sudo ./uninstall.sh
# Tunnels that are currently up are left running.
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "run with sudo" >&2
  exit 1
fi

rm -f /etc/sudoers.d/tunl-tui
rm -f /usr/local/bin/tunl-helper /usr/local/bin/tunl-tui

if getent group elsec-tunl >/dev/null; then
  groupdel elsec-tunl
fi

echo "Uninstalled. Existing sessions keep the elsec-tunl group until you log out."
