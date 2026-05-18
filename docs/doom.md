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

## Frontend Surface Gate

The current compiler can preprocess, lex, and surface-parse all official
`linuxdoom-1.10` C/header files with the upstream Linux build defines:

```bash
doom=/path/to/DOOM/linuxdoom-1.10
for file in "$doom"/*.[ch]; do
  cargo run --quiet -- parse-check -D NORMALUNIX -D LINUX -I "$doom" "$file"
done
```

This is a frontend milestone only. `parse-check` recognizes Doom-shaped
top-level declarations and function-definition boundaries, but it does not type
check, lower, compile, link, or run Doom yet.

## Compile Progress

`compile -S` now accepts full translation units enough to skip top-level
declarations and prototypes before attempting supported function definitions.
It also accepts `int` and `void` function return types, binds supported integer
parameters into ABI-backed local slots, emits terminal returns for `void`
functions that can fall through, treats supported scalar/typedef return
specifiers such as `fixed_t`, `boolean`, `char`, and `unsigned short` as the
current integer return ABI, and emits signed integer expression slices for
`long long` casts, function call arguments, and `?:` conditionals. Pointer
returns remain unsupported.

The current Doom compile scan reaches actual supported function bodies, but all
but one of the 62 C files still fail before object generation. `m_swap.c`
currently reaches assembly generation. `m_fixed.c` now gets through `FixedMul`
and stops in the active `FixedDiv2` floating-point path. Evidence is recorded in
`docs/qa/2026-05-18-doom-translation-unit.md`.

## Playability Gate

Future acceptance requires:

1. Compile `linuxdoom-1.10` with `c99inrust`.
2. Link the executable for a Linux/X11 environment.
3. Provide a legal IWAD path through `DOOMWADDIR`.
4. Run inside tmux without `tmux kill-server`.
5. Verify a window/title loop appears.
6. Start a map and verify keyboard input moves the player.

Until all six pass, this repository must not claim playable Doom support.
