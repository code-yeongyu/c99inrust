#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: tools/doom-validate-manual-transcript.sh <manual-transcript.txt>\n' >&2
}

if [ "$#" -ne 1 ]; then
	usage
	exit 2
fi

transcript="$1"
if [ ! -f "$transcript" ]; then
	printf 'error: transcript not found: %s\n' "$transcript" >&2
	exit 2
fi

field() {
	key="$1"
	awk -v key="$key" '
		index($0, key "=") == 1 {
			sub("^[^=]*=", "")
			if (key ~ /^(compile_ok|link_status|manual_run|final_status|window_visible|map_started|arrow_keys_move|strafe_fire_use_respond|commit)$/) {
				split($0, parts, /[ \t]/)
				print parts[1]
				exit
			}
			print
			exit
		}
		{
			for (i = 1; i <= NF; i++) {
				if (index($i, key "=") == 1) {
					sub("^[^=]*=", "", $i)
					print $i
					exit
				}
			}
		}
	' "$transcript"
}

failures=0

fail() {
	printf 'manual_transcript_error=%s\n' "$*" >&2
	failures=$((failures + 1))
}

require_nonempty() {
	key="$1"
	value="$(field "$key")"
	if [ -z "$value" ]; then
		fail "missing_or_empty:$key"
	fi
}

require_eq() {
	key="$1"
	expected="$2"
	actual="$(field "$key")"
	if [ "$actual" != "$expected" ]; then
		fail "expected:$key=$expected actual:$actual"
	fi
}

require_truthy() {
	key="$1"
	actual="$(field "$key")"
	normalized="$(printf '%s' "$actual" | tr '[:upper:]' '[:lower:]')"
	case "$normalized" in
		1|true|yes|y|ok|pass|passed) ;;
		*) fail "expected_truthy:$key actual:$actual" ;;
	esac
}

for key in operator date commit tmux_session out doom_source iwad display exit_method; do
	require_nonempty "$key"
done

require_eq compile_ok 62
require_eq compile_fail 0
require_eq link_status 0
require_eq manual_run finished
require_eq final_status 0
require_truthy window_visible
require_truthy map_started
require_truthy arrow_keys_move
require_truthy strafe_fire_use_respond

if [ "$(field commit)" = "unknown" ]; then
	fail "expected_real_commit"
fi

if [ "$failures" -ne 0 ]; then
	printf 'manual_transcript_status=invalid failures=%s\n' "$failures" >&2
	exit 1
fi

printf 'manual_transcript_status=valid\n'
printf 'commit=%s\n' "$(field commit)"
printf 'operator=%s\n' "$(field operator)"
