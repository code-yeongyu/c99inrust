#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
	printf 'usage: tools/doom-compile-scan.sh <official-doom-checkout> [output.txt]\n' >&2
	exit 2
fi

root="$1"
output="${2:-/tmp/c99inrust-doom-compile-scan.txt}"
linuxdoom="$root/linuxdoom-1.10"
compiler="${C99INRUST_BIN:-target/debug/c99inrust}"
tmpdir="${TMPDIR:-/tmp}"

if [ ! -d "$linuxdoom" ]; then
	printf 'error: expected official id-Software/DOOM checkout with linuxdoom-1.10\n' >&2
	exit 2
fi

mkdir -p "$(dirname "$output")"
: > "$output"

record() {
	printf '%s\n' "$*" | tee -a "$output"
}

ok=0
fail=0

record "official-doom-root=$root"
record "linuxdoom=$linuxdoom"
record "compiler=$compiler"

for file in "$linuxdoom"/*.c; do
	name="$(basename "$file")"
	assembly="$tmpdir/c99inrust-doom-$name.s"
	stderr_log="$tmpdir/c99inrust-doom-$name.err"
	if "$compiler" compile -S -D NORMALUNIX -D LINUX -I "$linuxdoom" "$file" -o "$assembly" >"$stderr_log" 2>&1; then
		ok=$((ok + 1))
		record "OK $name"
	else
		fail=$((fail + 1))
		record "FAIL $name"
		sed 's/^/  /' "$stderr_log" | tee -a "$output"
	fi
done

record "ok=$ok"
record "fail=$fail"
