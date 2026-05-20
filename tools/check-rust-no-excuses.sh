#!/usr/bin/env bash
set -euo pipefail

src_paths=()
all_paths=()

if [ "$#" -eq 0 ]; then
	set -- src tests
fi

for path in "$@"; do
	all_paths+=("$path")
	case "$path" in
	tests | tests/*) ;;
	*) src_paths+=("$path") ;;
	esac
done

status=0

check_no_matches() {
	label="$1"
	pattern="$2"
	shift 2
	if [ "$#" -eq 0 ]; then
		return
	fi
	if rg -n "$pattern" "$@"; then
		printf 'error: %s\n' "$label" >&2
		status=1
	fi
}

check_no_matches "unsafe Rust is forbidden" 'unsafe\s*(fn|impl|\{|\()' "${all_paths[@]}"
check_no_matches "unwrap/expect are forbidden outside tests" '\.(unwrap|expect)\(' "${src_paths[@]}"
check_no_matches "placeholder macros are forbidden" '\b(todo|unimplemented|unreachable)!\(' "${src_paths[@]}"
check_no_matches "inline clippy allows need a documented exception" '#\[allow\(clippy::' "${src_paths[@]}"
check_no_matches "Box<dyn Error> is forbidden in source" 'Box<dyn Error' "${src_paths[@]}"
check_no_matches "panic! is forbidden in source" '\bpanic!\(' "${src_paths[@]}"

exit "$status"
