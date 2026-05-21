# Doom Manual Play Harness

Public Doom checkout: `id-Software/DOOM` at `/tmp/c99inrust-doom-src`.

This QA artifact adds a manual play harness for the same official Doom target
covered by the automated smoke scripts. It compiles all 62 official
`linuxdoom-1.10` C translation units with `c99inrust`, links the Linux/X11 ELF
inside Docker, and then runs that ELF against a host X11 display for direct
keyboard control.

## Harness

Use build-only mode on headless machines:

```bash
cargo build
DOOM_MANUAL_RUN=0 tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-manual-play
```

Expected build-only status:

```text
compile_ok=62 compile_fail=0
link_status=0
transcript=/tmp/c99inrust-doom-manual-play/manual-transcript.txt
manual_run=skipped
reason=DOOM_MANUAL_RUN=0
```

Latest manual harness build-only QA, on commit `20120f6`:

```text
out=/tmp/c99inrust-doom-manual-20120f6-transcript
compile_ok=62 compile_fail=0
link_status=0
binary=/tmp/c99inrust-doom-manual-20120f6-transcript/linuxdoom-c99inrust
manual_run=skipped
reason=DOOM_MANUAL_RUN=0
transcript=/tmp/c99inrust-doom-manual-20120f6-transcript/manual-transcript.txt
```

This proves the manual harness still builds and links the official Doom binary
on current `main` and now writes a transcript scaffold. An older tmux manual
harness QA on commit `d7e708e` also verified the no-`tmux kill-server` workflow
and stopped with `manual_run=blocked` when no host display was available. The
remaining evidence gap is a completed human-visible X11 playthrough transcript,
not C compilation or Linux/X11 linking.

Use interactive mode when an X11 display is available:

```bash
cargo build
tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-manual-play -- -warp 1 1 -nosound
```

For Linux/X11 Docker hosts, allow the container to reach the display before
running the script:

```bash
xhost +local:root
tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad
```

For macOS with XQuartz, enable network clients in XQuartz, allow local
connections, then pass the Docker display explicitly:

```bash
xhost + 127.0.0.1
DOOM_DOCKER_DISPLAY=host.docker.internal:0 tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad
```

## tmux Manual QA

Manual QA should run inside tmux and exit the session naturally. Do not use
`tmux kill-server`.

```bash
tmux new -s c99inrust-doom-manual
cargo build
DOOM_MANUAL_OPERATOR="$USER" DOOM_MANUAL_VALIDATE=1 \
  tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad
```

Interactive acceptance checks:

1. The Doom window appears on the host display.
2. A new game starts on E1M1.
3. Arrow keys move and turn the player.
4. Strafe, fire, and use keys respond.
5. The process exits cleanly from Doom's menu or by closing the tmux command.

Transcript fields to record when a host X11 display is available:

```text
operator=
date=
commit=
tmux_session=
out=
doom_source=
iwad=
display=
compile_ok=62 compile_fail=0
link_status=0
manual_run=finished
window_visible=
map_started=
arrow_keys_move=
strafe_fire_use_respond=
exit_method=
final_status=
notes=
```

The harness writes these fields to `manual-transcript.txt` in the output
directory. Build-only and blocked runs prefill the compile/link fields. For a
human-visible play session, the harness prompts for the gameplay fields after
the Doom process exits when stdin is a TTY. Set `DOOM_MANUAL_PROMPT=0` to skip
the prompts, or prefill fields with `DOOM_MANUAL_WINDOW_VISIBLE`,
`DOOM_MANUAL_MAP_STARTED`, `DOOM_MANUAL_ARROW_KEYS_MOVE`,
`DOOM_MANUAL_STRAFE_FIRE_USE_RESPOND`, `DOOM_MANUAL_EXIT_METHOD`, and
`DOOM_MANUAL_NOTES`.

Validate a completed transcript before claiming human-visible manual
playability:

```bash
tools/doom-validate-manual-transcript.sh /tmp/c99inrust-doom-manual-play/manual-transcript.txt
```

The validator fails skipped or blocked runs. It only accepts a transcript with
`manual_run=finished`, `final_status=0`, `compile_ok=62 compile_fail=0`,
`link_status=0`, a real commit, a tmux session, a display, and truthy
`window_visible`, `map_started`, `arrow_keys_move`, and
`strafe_fire_use_respond` fields.

## Current Evidence Status

The automated movement smoke already proves visible-window startup, scripted
input delivery, coordinate-changing movement, and survival to timeout. This
manual harness closes the tooling gap for a human-visible play session, but a
completed human playthrough transcript should still be recorded before claiming
human-verified playability.
