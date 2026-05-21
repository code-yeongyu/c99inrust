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

## CI Recheck After Local Struct Array Support

Date: 2026-05-21 10:36 KST
Commit: `f587a7c fix(c99): support local struct arrays`
GitHub Actions run: `26200199678`

The full CI workflow rebuilt the compiler, cloned the official public Doom
source, and reran the compile scan after adding local struct array oracle
coverage and compiler support.

Downloaded artifact: `doom-compile-link-proof-f587a7c0a41d4e61a8ba9b3ff8a6127464f26898`

Result:

```text
official-doom-root=/home/runner/work/_temp/DOOM
linuxdoom=/home/runner/work/_temp/DOOM/linuxdoom-1.10
compiler=target/debug/c99inrust
ok=62
fail=0
conclusion=success
```
