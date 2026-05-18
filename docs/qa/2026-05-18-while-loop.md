# While Loop QA Evidence

Date: 2026-05-18
Verified compiler head: `a0f03f3d2af8eabee20fc70d5ff71b0033b07f31`
Session: `c99-while-qa3`
Raw capture: `/tmp/c99inrust-tmux-while-qa-a0f03f3.txt`

## Manual tmux QA

The tmux session ran under `bash --noprofile --norc`, compiled a C99 `while`
loop snippet with `c99inrust`, assembled it with `cc`, ran the produced binary,
then compiled and ran the same source with the host C compiler.

Captured pass lines:

```text
compile_status=0
assemble_status=0
c99_status=10
host_compile_status=0
host_status=10
manual_while_compile=ok
```

The session was closed with `exit`. `tmux kill-server` was not used.

## CI QA

GitHub Actions run `26013566593` passed for:

- `rust (ubuntu-24.04)`, check-run `76458925160`
- `rust (macos-15)`, check-run `76458925166`
- `rust (windows-2025-vs2026)`, check-run `76458925162`

The raw log was saved at `/tmp/c99inrust-ci-26013566593.log`.
Warning scan over `warning|deprecated|deprecation|redirect|executable stack|GNU-stack`
returned no matches, and all three check-runs reported zero annotations.
