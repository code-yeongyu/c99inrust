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
manual_run=skipped
reason=DOOM_MANUAL_RUN=0
```

Latest tmux build-only QA, without `tmux kill-server`:

```text
tmux_session=c99inrust-doom-manual-1779273652
out=/tmp/c99inrust-doom-manual-1779273652-out
cargo_status=0
manual_status=0
compile_ok=62 compile_fail=0
link_status=0
binary=/tmp/c99inrust-doom-manual-1779273652-out/linuxdoom-c99inrust
manual_run=skipped
reason=DOOM_MANUAL_RUN=0
```

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
tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad
```

Interactive acceptance checks:

1. The Doom window appears on the host display.
2. A new game starts on E1M1.
3. Arrow keys move and turn the player.
4. Strafe, fire, and use keys respond.
5. The process exits cleanly from Doom's menu or by closing the tmux command.

## Current Evidence Status

The automated movement smoke already proves visible-window startup, scripted
input delivery, coordinate-changing movement, and survival to timeout. This
manual harness closes the tooling gap for a human-visible play session, but a
completed human playthrough transcript should still be recorded before claiming
human-verified playability.
