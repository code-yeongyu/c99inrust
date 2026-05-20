# Doom Movement QA

Public Doom checkout: `id-Software/DOOM` at `/tmp/c99inrust-doom-src`.

This QA extends the visible-window input smoke with live player-state sampling.
It proves scripted keyboard input changes the player coordinates while the
player mobj remains linked to the Doom thinker list.

## Regression Found

The earlier input smoke proved keyboard delivery but not movement. A live
`/proc/<pid>/mem` probe showed why:

```text
before_0 level=5 func=0x441392 found_player=1 x=69206016 y=-236978176
up_1     level=52 func=0xffffffffffffffff found_player=0 x=69206016 y=-236978176 momy=153597
```

Input reached `ticcmd` and `P_Thrust` changed momentum, but the player mobj was
removed from the thinker list. The root cause was the global Doom `states[]`
table: `state_t states[NUMSTATES]` was emitted as `.zero 54152`. When movement
called `P_SetMobjState(player->mo, S_PLAY_RUN1)`, the zeroed row supplied
`tics=0` and `nextstate=S_NULL`, which removed the player mobj.

## Fixed Assembly Evidence

Focused `info.c` assembly generation now emits action function pointers instead
of zeroing the whole state table:

```text
states:
    ...
    .quad A_Pain
    .long 149
    ...
    .quad A_Fall
```

## Passing Movement Smoke

`tools/doom-movement-smoke.sh` recompiles all 62 official Doom C translation
units, links them into a Linux/X11 ELF, opens a viewable `320x200` X11 child
window under Xvfb, dispatches `Up` with `xdotool`, reads live process memory,
and fails unless player coordinates change while the player mobj remains active.

Latest direct run:

```text
out=/tmp/c99inrust-doom-movement-smoke-20260520
compile_ok=62 compile_fail=0
link_status=0
display_status=0
window_status=0
window_id=0x400002
input_status=0
movement_status=0
tail_status=124
run_status=124

before 5 5 1 1 1 69206016 -236978176 0 0 0 -1
up_11 120 120 1 1 1 69194459 -208771260 -97 236756 0 2
```

Latest tmux manual QA rerun, without `tmux kill-server`, on commit
`1d91681`:

```text
tmux_session=c99inrust-doom-movement-20260521015300
out=/tmp/c99inrust-doom-movement-20260521015300-out
tmux_command_status=0
compile_ok=62 compile_fail=0
link_status=0
display_status=0
window_status=0
window_id=0x400002
input_status=0
movement_status=0
tail_status=124
run_status=124

before 5 5 1 1 1 69206016 -236978176 0 0 0 -1
up_11 119 119 1 1 1 69194566 -209032509 -107 261249 25 3
```

Sample columns are:

```text
gametic leveltime found_in_thinker_list function_is_P_MobjThinker state_is_nonnull x y momx momy forwardmove tics
```

`movement_status=0` means the script observed coordinate movement with
`found_in_thinker_list=1`, `function_is_P_MobjThinker=1`, and
`state_is_nonnull=1`.

Repeat with:

```bash
cargo build
tools/doom-movement-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-movement-smoke
```
