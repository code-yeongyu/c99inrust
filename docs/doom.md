# Official Doom QA Target

The target source is the public id Software release:

- GitHub: https://github.com/id-Software/DOOM
- Historic archive: https://www.gamers.org/pub/idgames/idstuff/source/

The upstream README states the source release is Linux-only and still needs real
Doom game data. The Linux target links X11/Xext and a small set of platform
libraries.

## Audit

```bash
git clone https://github.com/id-Software/DOOM /tmp/DOOM
cargo run -- doom-audit /tmp/DOOM
```

Expected today:

- counts C and header files under `linuxdoom-1.10`
- confirms the Makefile exists
- reports that full Doom compilation is still a future milestone

## Playability Gate

Future acceptance requires:

1. Compile `linuxdoom-1.10` with `c99inrust`.
2. Link the executable for a Linux/X11 environment.
3. Provide a legal IWAD path through `DOOMWADDIR`.
4. Run inside tmux without `tmux kill-server`.
5. Verify a window/title loop appears.
6. Start a map and verify keyboard input moves the player.

Until all six pass, this repository must not claim playable Doom support.
