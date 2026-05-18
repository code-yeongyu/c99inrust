# Zero-Argument Function Call QA Evidence

Date: 2026-05-18
Verified compiler commit: `4739729e42092cb023145ac42daf86ea64df3629`
Session: `c99-call-qa`
Raw capture: `/tmp/c99inrust-tmux-call-qa-4739729.txt`

## Manual tmux QA

The tmux session ran under `bash --noprofile --norc`, compiled a C99 program
with a zero-argument helper function call, assembled the `c99inrust` output with
`cc`, ran it, then compiled and ran the same source with the host C compiler.

Captured pass lines:

```text
compile_status=0
assemble_status=0
c99_status=42
host_compile_status=0
host_status=42
manual_zero_arg_call=ok
```

The session was closed with `exit`. `tmux kill-server` was not used.
