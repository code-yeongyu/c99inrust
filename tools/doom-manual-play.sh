#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: tools/doom-manual-play.sh <official-doom-checkout> <doom1.wad> [output-dir] [-- doom-args...]\n' >&2
}

if [ "$#" -lt 2 ]; then
	usage
	exit 2
fi

root="$1"
wad="$2"
shift 2

out="${DOOM_MANUAL_OUT:-/tmp/c99inrust-doom-manual-play}"
if [ "$#" -gt 0 ] && [ "${1:-}" != "--" ]; then
	out="$1"
	shift
fi
if [ "${1:-}" = "--" ]; then
	shift
fi
if [ "$#" -eq 0 ]; then
	set -- -warp 1 1 -nosound
fi

linuxdoom="$root/linuxdoom-1.10"
compiler="${C99INRUST_BIN:-target/debug/c99inrust}"
asm="$out/asm"
transcript="$out/manual-transcript.txt"

if [ ! -d "$linuxdoom" ]; then
	printf 'error: expected official id-Software/DOOM checkout with linuxdoom-1.10\n' >&2
	exit 2
fi

if [ ! -f "$wad" ]; then
	printf 'error: expected legal Doom IWAD file at %s\n' "$wad" >&2
	exit 2
fi

mkdir -p "$asm" "$out"
: >"$out/manual-play.log"

record() {
	printf '%s\n' "$*" | tee -a "$out/manual-play.log"
}

write_transcript() {
	manual_run="$1"
	final_status="$2"
	reason="${3:-}"
	commit="$(git rev-parse --short HEAD 2>/dev/null || printf 'unknown')"
	tmux_session="$(tmux display-message -p '#S' 2>/dev/null || printf '%s' "${TMUX:-}")"
	{
		printf 'operator=%s\n' "${DOOM_MANUAL_OPERATOR:-}"
		printf 'date=%s\n' "$(date '+%Y-%m-%d %H:%M %Z')"
		printf 'commit=%s\n' "$commit"
		printf 'tmux_session=%s\n' "$tmux_session"
		printf 'out=%s\n' "$out"
		printf 'doom_source=%s\n' "$root"
		printf 'iwad=%s\n' "$wad"
		printf 'display=%s\n' "${docker_display:-}"
		printf 'compile_ok=%s compile_fail=%s\n' "$compile_ok" "$compile_fail"
		printf 'link_status=0\n'
		printf 'manual_run=%s\n' "$manual_run"
		printf 'reason=%s\n' "$reason"
		printf 'window_visible=\n'
		printf 'map_started=\n'
		printf 'arrow_keys_move=\n'
		printf 'strafe_fire_use_respond=\n'
		printf 'exit_method=\n'
		printf 'final_status=%s\n' "$final_status"
		printf 'notes=\n'
	} >"$transcript"
	record "transcript=$transcript"
}

compile_ok=0
compile_fail=0

record "official-doom-root=$root"
record "linuxdoom=$linuxdoom"
record "iwad=$wad"
record "compiler=$compiler"
record "out=$out"

for file in "$linuxdoom"/*.c; do
	name="$(basename "$file" .c)"
	if "$compiler" compile -S --target x86_64-unknown-linux-gnu \
		-D NORMALUNIX -D LINUX -I "$linuxdoom" "$file" -o "$asm/$name.s" \
		>>"$out/manual-play.log" 2>&1; then
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
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update >/out/manual-apt.log 2>&1
apt-get install -y --no-install-recommends \
  gcc libc6-dev libx11-dev libxext-dev libnsl-dev \
  >/out/manual-apt-install.log 2>&1
gcc -g -Wall -DNORMALUNIX -DLINUX -no-pie -I/src /asm/*.s \
  -o /out/linuxdoom-c99inrust -lXext -lX11 -lnsl -lm \
  >/out/manual-link.log 2>&1
ldd /out/linuxdoom-c99inrust >/out/manual-ldd.log 2>&1 || true
'

record "link_status=0"
record "binary=$out/linuxdoom-c99inrust"

if [ "${DOOM_MANUAL_RUN:-1}" = "0" ]; then
	record "manual_run=skipped"
	record "reason=DOOM_MANUAL_RUN=0"
	write_transcript "skipped" "0" "DOOM_MANUAL_RUN=0"
	exit 0
fi

docker_display="${DOOM_DOCKER_DISPLAY:-${DISPLAY:-}}"
if [ -z "$docker_display" ]; then
	record "manual_run=blocked"
	record "reason=no DISPLAY or DOOM_DOCKER_DISPLAY"
	write_transcript "blocked" "20" "no DISPLAY or DOOM_DOCKER_DISPLAY"
	printf 'error: set DISPLAY or DOOM_DOCKER_DISPLAY, or use DOOM_MANUAL_RUN=0 for build-only mode\n' >&2
	exit 20
fi

x11_socket="${DOOM_X11_SOCKET:-/tmp/.X11-unix}"
docker_run_args=(--rm -it --platform linux/amd64 -e "DISPLAY=$docker_display")
if [ -d "$x11_socket" ]; then
	docker_run_args+=(-v "$x11_socket:$x11_socket:rw")
fi

record "manual_run=starting"
record "docker_display=$docker_display"
record "doom_args=$*"

if docker run "${docker_run_args[@]}" \
	-v "$out:/out" \
	-v "$wad:/doom1.wad:ro" \
	ubuntu:24.04 bash -lc '
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update >/out/manual-run-apt.log 2>&1
apt-get install -y --no-install-recommends libx11-6 libxext6 libnsl2 \
  >/out/manual-run-apt-install.log 2>&1
cd /
exec /out/linuxdoom-c99inrust -iwad /doom1.wad "$@"
' bash "$@"; then
	manual_status=0
else
	manual_status=$?
fi
record "manual_run=finished"
record "manual_status=$manual_status"
write_transcript "finished" "$manual_status"
exit "$manual_status"
