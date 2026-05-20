# Doom Visible Window And Input QA

Public Doom checkout: `id-Software/DOOM` at `/tmp/c99inrust-doom-src`.

This QA extends the earlier compile/link/Xvfb smoke with a visible-window probe
and scripted keyboard input. It proves X11 window creation, key dispatch, and
survival to the scripted timeout; it does not prove a human playthrough or
player movement yet.

## Regression Found

The first input smoke reproduced a real runtime crash after `Up` was sent to
the Doom window:

```text
out=/tmp/c99inrust-doom-input-smoke-1779263806
compile_ok=62 compile_fail=0
link_status=0
display_status=0
window_status=0
window_id=0x400002
input_status=0
run_status=139
```

A corrected input matrix showed the crash was specific to special-key input:

```text
keys=none   run_status=124
keys=Return run_status=124
keys=Up     run_status=139
```

Because ptrace/GDB register access failed under the local amd64 Docker
emulation, an in-process SIGSEGV backtrace shim was linked with the generated
assembly. It mapped the crash to `cht_CheckCheat` in `m_cheat.s` while
dereferencing `cht->p`.

Root cause: global Doom `cheatseq_t` objects such as
`cheatseq_t cheat_god = { cheat_god_seq, 0 };` were emitted as `.zero 16`, so
their `sequence` pointer was null. The fix preserves global struct-object
initializer values and also uses full row strides for byte-matrix row pointer
initializers such as `cheat_powerup_seq[6]`.

## Fixed Assembly Evidence

Focused `st_stuff.c` assembly generation now emits pointer data for the cheat
globals:

```text
cheat_god:
    .quad cheat_god_seq
    .quad 0

cheat_powerup:
    .quad cheat_powerup_seq
    .quad 0
    .quad cheat_powerup_seq+10
    .quad 0
    ...
    .quad cheat_powerup_seq+60
```

## Passing Input Smoke

The repaired input smoke was also run in tmux without `tmux kill-server`. The
session exited naturally after compiling all 62 official `linuxdoom-1.10`
translation units, linking them in an amd64 Ubuntu container, opening a
viewable `320x200` X11 child window under Xvfb, dispatching
`Return Up Up Left Right` with `xdotool`, and surviving to the scripted
timeout:

```text
tmux_session=c99inrust-doom-input-1779266316
out=/tmp/c99inrust-doom-input-1779266316-out
compile_ok=62 compile_fail=0
link_status=0
display_status=0
window_status=0
window_id=0x400002
input_status=0
tail_status=124
run_status=124
tmux_command_status=0
```

`run_status=124` is the scripted timeout, not a segfault.

Repeat with:

```bash
cargo build
tools/doom-input-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-input-smoke
```

## Remaining Gate

Interactive player movement is still not proven by this script. The next
playability gate should either be a human run in a visible X11 session or an
instrumented check that demonstrates movement/state changes after keyboard
input.
