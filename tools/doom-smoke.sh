#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
	printf 'usage: tools/doom-smoke.sh <official-doom-checkout>\n' >&2
	exit 2
fi

root="$1"

cargo run -- doom-audit "$root"

cat <<'MSG'

Manual Doom playability is not implemented yet.
Future gate:
  1. compile linuxdoom-1.10 with c99inrust
  2. link Linux/X11 executable
  3. set DOOMWADDIR to a legal IWAD directory
  4. run in tmux without tmux kill-server
  5. verify title loop and in-map keyboard movement
MSG
