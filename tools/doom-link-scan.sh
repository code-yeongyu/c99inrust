#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
	printf 'usage: tools/doom-link-scan.sh <official-doom-checkout> [output-dir]\n' >&2
	exit 2
fi

root="$1"
out="${2:-/tmp/c99inrust-doom-link-scan}"
linuxdoom="$root/linuxdoom-1.10"
compiler="${C99INRUST_BIN:-target/debug/c99inrust}"
asm="$out/asm"

if [ ! -d "$linuxdoom" ]; then
	printf 'error: expected official id-Software/DOOM checkout with linuxdoom-1.10\n' >&2
	exit 2
fi

mkdir -p "$asm" "$out"
: >"$out/link-scan.log"

record() {
	printf '%s\n' "$*" | tee -a "$out/link-scan.log"
}

compile_ok=0
compile_fail=0

record "official-doom-root=$root"
record "linuxdoom=$linuxdoom"
record "compiler=$compiler"

for file in "$linuxdoom"/*.c; do
	name="$(basename "$file" .c)"
	if "$compiler" compile -S --target x86_64-unknown-linux-gnu \
		-D NORMALUNIX -D LINUX -I "$linuxdoom" "$file" -o "$asm/$name.s" \
		>>"$out/link-scan.log" 2>&1; then
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
	ubuntu:24.04 bash -lc '
set -u
export DEBIAN_FRONTEND=noninteractive
: >/out/status.log
apt-get update >/out/apt.log 2>&1 || exit $?
apt-get install -y --no-install-recommends \
  gcc libc6-dev libx11-dev libxext-dev libnsl-dev file \
  >/out/apt-install.log 2>&1 || exit $?
gcc -g -Wall -DNORMALUNIX -DLINUX -no-pie -I/src /asm/*.s \
  -o /out/linuxdoom-c99inrust -lXext -lX11 -lnsl -lm \
  >/out/link.log 2>&1
link_status=$?
printf "link_status=%s\n" "$link_status" | tee -a /out/status.log
if [ "$link_status" -ne 0 ]; then
  exit "$link_status"
fi
file /out/linuxdoom-c99inrust | tee -a /out/status.log
'

cat "$out/status.log" >>"$out/link-scan.log"
record "binary=$out/linuxdoom-c99inrust"
