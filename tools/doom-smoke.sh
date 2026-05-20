#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
	printf 'usage: tools/doom-smoke.sh <official-doom-checkout> <doom1.wad> [output-dir]\n' >&2
	exit 2
fi

root="$1"
wad="$2"
out="${3:-/tmp/c99inrust-doom-smoke}"
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
: >"$out/smoke.log"

record() {
	printf '%s\n' "$*" | tee -a "$out/smoke.log"
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
		>>"$out/smoke.log" 2>&1; then
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
set -u
export DEBIAN_FRONTEND=noninteractive
apt-get update >/out/apt.log 2>&1 || exit $?
apt-get install -y --no-install-recommends gcc libc6-dev libx11-dev libxext-dev libnsl-dev xvfb xauth >/out/apt-install.log 2>&1 || exit $?
gcc -g -Wall -DNORMALUNIX -DLINUX -no-pie -I/src /asm/*.s -o /out/linuxdoom-c99inrust -lXext -lX11 -lnsl -lm >/out/link.log 2>&1
link_status=$?
printf "link_status=%s\n" "$link_status" | tee /out/status.log
if [ "$link_status" -ne 0 ]; then
  exit "$link_status"
fi
timeout 25s xvfb-run -a -s "-screen 0 640x480x8 -cc 3" /out/linuxdoom-c99inrust -iwad /doom1.wad -warp 1 1 -nosound >/out/run.log 2>&1
run_status=$?
printf "run_status=%s\n" "$run_status" | tee -a /out/status.log
exit 0
' | tee -a "$out/smoke.log"

cat "$out/status.log" >>"$out/smoke.log" 2>/dev/null || true
record "binary=$out/linuxdoom-c99inrust"
record "run_log=$out/run.log"
