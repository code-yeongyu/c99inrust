# Doom Compile Scan Recheck

Date: 2026-05-21 04:35 KST
Commit: `deea66e test(c99): deepen requested clang oracle edge cases`

This rechecked the official public Doom compile surface after adding the deeper
Clang-oracle C99 edge tests. It did not touch DISPLAY, VNC, Xvfb, or manual play
state.

Command:

```bash
tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-compile-scan-deea66e.txt
```

Result:

```text
official-doom-root=/tmp/c99inrust-doom-src
linuxdoom=/tmp/c99inrust-doom-src/linuxdoom-1.10
compiler=target/debug/c99inrust
ok=62
fail=0
```

All 62 `linuxdoom-1.10` C translation units emitted assembly with:

```text
compile -S -D NORMALUNIX -D LINUX -I /tmp/c99inrust-doom-src/linuxdoom-1.10
```

This proves the current compiler commit still reaches assembly generation for
the full official Doom C source set. It is compile evidence only; link/run/input
and movement evidence remain the stronger smoke gates recorded separately.
