#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
	printf 'usage: tools/doom-movement-smoke.sh <official-doom-checkout> <doom1.wad> [output-dir]\n' >&2
	exit 2
fi

root="$1"
wad="$2"
out="${3:-/tmp/c99inrust-doom-movement-smoke}"
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
: >"$out/movement-smoke.log"

record() {
	printf '%s\n' "$*" | tee -a "$out/movement-smoke.log"
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
		>>"$out/movement-smoke.log" 2>&1; then
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
	-v "$asm:/asm:ro" \
	-v "$out:/out" \
	-v "$linuxdoom:/src:ro" \
	-v "$wad:/doom1.wad:ro" \
	ubuntu:24.04 bash -lc '
set -eu
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
    if xdpyinfo >/out/movement-xdpyinfo.log 2>&1; then
      return 0
    fi
    pause_for 0.2s
  done
  return 1
}

find_doom_window() {
  deadline=$((SECONDS + 15))
  while [ "$SECONDS" -lt "$deadline" ]; do
    if xwininfo -root -tree >/out/movement-window-tree.log 2>/out/movement-window-tree.err; then
      candidate=""
      while read -r window rest; do
        case "$rest" in
          *" 320x200+"*|*" 320x200-"*)
            candidate="$window"
            break
            ;;
        esac
      done </out/movement-window-tree.log
      if [ -n "$candidate" ]; then
        if xwininfo -id "$candidate" >/out/movement-window.log 2>/out/movement-window.err; then
          if rg -q "Map State: IsViewable" /out/movement-window.log; then
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

symbol_address() {
  target="$1"
  while read -r address kind name rest; do
    if [ "${name:-}" = "$target" ]; then
      printf "%s\n" "$address"
      return 0
    fi
  done </out/movement-symbols.txt
  return 1
}

: >/out/status.log
: >/out/movement-samples.log
apt-get update >/out/apt.log 2>&1 || exit $?
apt-get install -y --no-install-recommends \
  gcc libc6-dev libx11-dev libxext-dev libnsl-dev \
  xvfb xauth x11-utils xdotool ripgrep binutils \
  >/out/apt-install.log 2>&1 || exit $?

gcc -O2 -Wall -Wextra -x c - -o /tmp/read_movement <<'"'"'C_EOF'"'"'
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static void read_exact(int fd, uint64_t addr, void *dst, size_t size) {
  ssize_t got = pread(fd, dst, size, (off_t)addr);
  if (got != (ssize_t)size) {
    fprintf(stderr, "pread addr=0x%" PRIx64 " size=%zu got=%zd errno=%d\n", addr, size, got, errno);
    exit(3);
  }
}

static int32_t read_i32(int fd, uint64_t addr) {
  int32_t value = 0;
  read_exact(fd, addr, &value, sizeof(value));
  return value;
}

static uint64_t read_u64(int fd, uint64_t addr) {
  uint64_t value = 0;
  read_exact(fd, addr, &value, sizeof(value));
  return value;
}

static int8_t read_i8(int fd, uint64_t addr) {
  int8_t value = 0;
  read_exact(fd, addr, &value, sizeof(value));
  return value;
}

int main(int argc, char **argv) {
  if (argc != 8) {
    return 2;
  }
  int pid = atoi(argv[1]);
  uint64_t players = strtoull(argv[2], NULL, 16);
  uint64_t consoleplayer = strtoull(argv[3], NULL, 16);
  uint64_t thinkercap = strtoull(argv[4], NULL, 16);
  uint64_t p_mobj = strtoull(argv[5], NULL, 16);
  uint64_t gametic = strtoull(argv[6], NULL, 16);
  uint64_t leveltime = strtoull(argv[7], NULL, 16);
  char path[128];
  snprintf(path, sizeof(path), "/proc/%d/mem", pid);
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    perror(path);
    return 1;
  }

  int32_t cp = read_i32(fd, consoleplayer);
  uint64_t player = players + ((uint64_t)(uint32_t)cp * 320u);
  uint64_t mo = read_u64(fd, player);
  uint64_t cursor = read_u64(fd, thinkercap + 8u);
  int found = 0;
  int count = 0;
  while (cursor != thinkercap && cursor != 0 && count < 4096) {
    if (cursor == mo) {
      found = 1;
    }
    cursor = read_u64(fd, cursor + 8u);
    count++;
  }

  uint64_t func = read_u64(fd, mo + 16u);
  uint64_t state = read_u64(fd, mo + 152u);
  printf("%d %d %d %d %d %d %d %d %d %d %d\n",
    read_i32(fd, gametic),
    read_i32(fd, leveltime),
    found,
    func == p_mobj ? 1 : 0,
    state != 0 ? 1 : 0,
    read_i32(fd, mo + 24u),
    read_i32(fd, mo + 28u),
    read_i32(fd, mo + 112u),
    read_i32(fd, mo + 116u),
    read_i8(fd, player + 12u),
    read_i32(fd, mo + 144u));
  close(fd);
  return 0;
}
C_EOF

gcc -g -Wall -DNORMALUNIX -DLINUX -no-pie -I/src /asm/*.s \
  -o /out/linuxdoom-c99inrust -lXext -lX11 -lnsl -lm \
  >/out/link.log 2>&1
link_status=$?
append_status "link_status=$link_status"
if [ "$link_status" -ne 0 ]; then
  exit "$link_status"
fi

nm -g /out/linuxdoom-c99inrust >/out/movement-symbols.txt
players="$(symbol_address players)"
consoleplayer="$(symbol_address consoleplayer)"
thinkercap="$(symbol_address thinkercap)"
p_mobj="$(symbol_address P_MobjThinker)"
gametic="$(symbol_address gametic)"
leveltime="$(symbol_address leveltime)"

export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 640x480x8 -cc 3 >/out/movement-xvfb.log 2>&1 &
xvfb_pid=$!

display_status=0
wait_for_display || display_status=$?
append_status "display_status=$display_status"
if [ "$display_status" -ne 0 ]; then
  exit 20
fi

cd /
/out/linuxdoom-c99inrust -iwad /doom1.wad -warp 1 1 -nosound \
  >/out/movement-run.log 2>&1 &
doom_pid=$!

window_status=0
window_id="$(find_doom_window)" || window_status=$?
append_status "window_status=$window_status"
if [ "$window_status" -eq 0 ]; then
  append_status "window_id=$window_id"
else
  exit 21
fi

before_line="$(/tmp/read_movement "$doom_pid" "$players" "$consoleplayer" "$thinkercap" "$p_mobj" "$gametic" "$leveltime")"
printf "before %s\n" "$before_line" | tee -a /out/movement-samples.log
set -- $before_line
before_x="$6"
before_y="$7"

xdotool key --window "$window_id" Return >/out/movement-xdotool-return.log 2>&1 || true
pause_for 0.5s
input_status=0
xdotool keydown --window "$window_id" Up >/out/movement-xdotool-up.log 2>&1 || input_status=$?
append_status "input_status=$input_status"
if [ "$input_status" -ne 0 ]; then
  exit 22
fi

movement_status=1
for label in up_00 up_01 up_02 up_03 up_04 up_05 up_06 up_07 up_08 up_09 up_10 up_11; do
  line="$(/tmp/read_movement "$doom_pid" "$players" "$consoleplayer" "$thinkercap" "$p_mobj" "$gametic" "$leveltime")"
  printf "%s %s\n" "$label" "$line" | tee -a /out/movement-samples.log
  set -- $line
  found="$3"
  func_ok="$4"
  state_ok="$5"
  x="$6"
  y="$7"
  if [ "$found" -eq 1 ] && [ "$func_ok" -eq 1 ] && [ "$state_ok" -eq 1 ]; then
    if [ "$x" -ne "$before_x" ] || [ "$y" -ne "$before_y" ]; then
      movement_status=0
    fi
  fi
  pause_for 0.25s
done
xdotool keyup --window "$window_id" Up >/out/movement-xdotool-upup.log 2>&1 || true

append_status "movement_status=$movement_status"
if [ "$movement_status" -ne 0 ]; then
  exit 24
fi

set +e
timeout 2s tail --pid="$doom_pid" -f /dev/null >/dev/null 2>&1
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
' | tee -a "$out/movement-smoke.log"

cat "$out/status.log" >>"$out/movement-smoke.log" 2>/dev/null || true
record "binary=$out/linuxdoom-c99inrust"
record "run_log=$out/movement-run.log"
record "window_log=$out/movement-window.log"
record "samples_log=$out/movement-samples.log"
