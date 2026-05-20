#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
	printf 'usage: tools/doom-input-smoke.sh <official-doom-checkout> <doom1.wad> [output-dir]\n' >&2
	exit 2
fi

root="$1"
wad="$2"
out="${3:-/tmp/c99inrust-doom-input-smoke}"
linuxdoom="$root/linuxdoom-1.10"
compiler="${C99INRUST_BIN:-target/debug/c99inrust}"
asm="$out/asm"

if [ ! -d "$linuxdoom" ]; then
	printf 'error: expected official id-Software/DOOM checkout with linuxdoom-1.10\n' >&2
	exit 2
fi

if [ ! -f "$wad" ]; then
	printf 'error: expected legal Doom IWAD file at %s\n' "$wad" >&2
	exit 2
fi

mkdir -p "$asm" "$out"
: >"$out/input-smoke.log"

record() {
	printf '%s\n' "$*" | tee -a "$out/input-smoke.log"
}

compile_ok=0
compile_fail=0

record "official-doom-root=$root"
record "linuxdoom=$linuxdoom"
record "iwad=$wad"
record "compiler=$compiler"

for file in "$linuxdoom"/*.c; do
	name="$(basename "$file" .c)"
	if "$compiler" compile -S --target x86_64-unknown-linux-gnu \
		-D NORMALUNIX -D LINUX -I "$linuxdoom" "$file" -o "$asm/$name.s" \
		>>"$out/input-smoke.log" 2>&1; then
		compile_ok=$((compile_ok + 1))
	else
		compile_fail=$((compile_fail + 1))
		record "compile_fail $name"
	fi
done

record "compile_ok=$compile_ok compile_fail=$compile_fail"
if [ "$compile_fail" -ne 0 ]; then
	exit 10
fi

docker run --rm --platform linux/amd64 \
	-e DOOM_INPUT_TIMEOUT="${DOOM_INPUT_TIMEOUT:-20s}" \
	-v "$asm:/asm:ro" \
	-v "$out:/out" \
	-v "$linuxdoom:/src:ro" \
	-v "$wad:/doom1.wad:ro" \
	ubuntu:24.04 bash -lc '
set -u
export DEBIAN_FRONTEND=noninteractive

pause_for() {
  timeout "$1" tail -f /dev/null >/dev/null 2>&1 || true
}

append_status() {
  printf "%s\n" "$*" | tee -a /out/status.log
}

cleanup() {
  if [ -n "${doom_pid:-}" ] && kill -0 "$doom_pid" 2>/dev/null; then
    kill "$doom_pid" 2>/dev/null || true
    wait "$doom_pid" 2>/dev/null || true
  fi
  if [ -n "${xvfb_pid:-}" ] && kill -0 "$xvfb_pid" 2>/dev/null; then
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

wait_for_display() {
  deadline=$((SECONDS + 10))
  while [ "$SECONDS" -lt "$deadline" ]; do
    if xdpyinfo >/out/input-xdpyinfo.log 2>&1; then
      return 0
    fi
    pause_for 0.2s
  done
  return 1
}

find_doom_window() {
  deadline=$((SECONDS + 15))
  while [ "$SECONDS" -lt "$deadline" ]; do
    if xwininfo -root -tree >/out/input-window-tree.log 2>/out/input-window-tree.err; then
      candidate=""
      while read -r window rest; do
        case "$rest" in
          *" 320x200+"*|*" 320x200-"*)
            candidate="$window"
            break
            ;;
        esac
      done </out/input-window-tree.log
      if [ -n "$candidate" ]; then
        if xwininfo -id "$candidate" >/out/input-window.log 2>/out/input-window.err; then
          if rg -q "Map State: IsViewable" /out/input-window.log; then
            printf "%s\n" "$candidate"
            return 0
          fi
        fi
      fi
    fi
    pause_for 0.25s
  done
  return 1
}

: >/out/status.log
apt-get update >/out/apt.log 2>&1 || exit $?
apt-get install -y --no-install-recommends \
  gcc libc6-dev libx11-dev libxext-dev libnsl-dev \
  xvfb xauth x11-utils xdotool ripgrep \
  >/out/apt-install.log 2>&1 || exit $?

gcc -g -Wall -DNORMALUNIX -DLINUX -no-pie -I/src /asm/*.s \
  -o /out/linuxdoom-c99inrust -lXext -lX11 -lnsl -lm \
  >/out/link.log 2>&1
link_status=$?
append_status "link_status=$link_status"
if [ "$link_status" -ne 0 ]; then
  exit "$link_status"
fi

export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 640x480x8 -cc 3 >/out/input-xvfb.log 2>&1 &
xvfb_pid=$!

display_status=0
wait_for_display || display_status=$?
append_status "display_status=$display_status"
if [ "$display_status" -ne 0 ]; then
  exit 20
fi

cd /
/out/linuxdoom-c99inrust -iwad /doom1.wad -warp 1 1 -nosound \
  >/out/input-run.log 2>&1 &
doom_pid=$!

window_status=0
window_id="$(find_doom_window)" || window_status=$?
append_status "window_status=$window_status"
if [ "$window_status" -eq 0 ]; then
  append_status "window_id=$window_id"
else
  exit 21
fi

input_status=0
xdotool key --window "$window_id" Return Up Up Left Right \
  >/out/input-xdotool.log 2>&1 || input_status=$?
append_status "input_status=$input_status"
if [ "$input_status" -ne 0 ]; then
  exit 22
fi

pause_for 1s
set +e
timeout "$DOOM_INPUT_TIMEOUT" tail --pid="$doom_pid" -f /dev/null >/dev/null 2>&1
tail_status=$?
if kill -0 "$doom_pid" 2>/dev/null; then
  kill "$doom_pid" 2>/dev/null || true
  wait "$doom_pid" 2>/dev/null
  run_status=124
else
  wait "$doom_pid" 2>/dev/null
  run_status=$?
fi
set -e

append_status "tail_status=$tail_status"
append_status "run_status=$run_status"
if [ "$run_status" -ne 124 ]; then
  exit 23
fi
exit 0
' | tee -a "$out/input-smoke.log"

cat "$out/status.log" >>"$out/input-smoke.log" 2>/dev/null || true
record "binary=$out/linuxdoom-c99inrust"
record "run_log=$out/input-run.log"
record "window_log=$out/input-window.log"
record "xdotool_log=$out/input-xdotool.log"
