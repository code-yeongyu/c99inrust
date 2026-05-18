# Logical Short-Circuit QA Evidence

Date: 2026-05-18
Verified compiler commit: `a30c5e5bc7fe7f1b221019813bb8c5fb1d2d4093`
Session: `c99-logical-qa`
Raw capture: `/tmp/c99inrust-tmux-logical-qa-a30c5e5.txt`

## Manual tmux QA

The tmux session ran under `bash --noprofile --norc`, compiled a C99 snippet
that would divide by zero under eager `&&`/`||` evaluation, assembled the
`c99inrust` output with `cc`, ran it, then compiled and ran the same source with
the host C compiler.

Captured pass lines:

```text
compile_status=0
assemble_status=0
c99_status=42
host_compile_status=0
host_status=42
manual_logical_short_circuit=ok
```

The session was closed with `exit`. `tmux kill-server` was not used.
