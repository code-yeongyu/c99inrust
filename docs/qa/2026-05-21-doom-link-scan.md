# Doom Link Scan CI Gate

Date: 2026-05-21 05:38 KST
Base commit: `68e9539 Expand deep C99 oracle coverage`

This adds a no-IWAD link gate for the official public Doom source. The gate is
weaker than the IWAD-backed Xvfb run and movement smokes, but it is strong
enough for CI: it recompiles all 62 official `linuxdoom-1.10` C translation
units with `c99inrust`, then links the generated x86_64 Linux assembly into a
Linux/X11 ELF inside an amd64 Ubuntu container.

Command:

```bash
tools/doom-link-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-link-scan-68e9539-plus
```

Result:

```text
official-doom-root=/tmp/c99inrust-doom-src
linuxdoom=/tmp/c99inrust-doom-src/linuxdoom-1.10
compiler=target/debug/c99inrust
compile_ok=62 compile_fail=0
link_status=0
/out/linuxdoom-c99inrust: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=775549125ddc9ca3b0d3d4e2549cb358b391a7be, for GNU/Linux 3.2.0, with debug_info, not stripped
binary=/tmp/c99inrust-doom-link-scan-68e9539-plus/linuxdoom-c99inrust
```

CI now runs this after the existing compile-count scan and verifies:

```text
compile_ok=62 compile_fail=0
link_status=0
```

This does not require `doom1.wad`, Xvfb, or host `DISPLAY`. Run/input/movement
remain covered by the separate IWAD-backed smoke scripts.

## CI Recheck After Ultra C99 Oracle Cases

Date: 2026-05-21 06:11 KST
Commit: `2000360 fix(c99): match ultra oracle cases`
GitHub Actions run: `26190196636`

The `doom compile/link scan` job completed successfully after the latest deep
C99 oracle expansion and compiler fixes. The job rebuilt the compiler, cloned
the official public Doom source, compiled all 62 official translation units, and
linked the Linux/X11 executable.

Result:

```text
compile_ok=62 compile_fail=0
link_status=0
conclusion=success
```

## Local Matrix And Static Initializer Recheck

Date: 2026-05-21 08:31 KST
Commit: working tree after `7578468 fix(c99): support next-wave oracle edges`

This rechecked the no-IWAD compile/link gate after adding clang-oracle coverage
for local `char` matrix row decay, nested byte reads, `sizeof` on matrix rows,
and static local multi-declaration initializers. This did not run DISPLAY, VNC,
Xvfb, manual play, or any IWAD-backed smoke.

Command:

```bash
tools/doom-link-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-link-scan-7578468-edge-more
```

Result:

```text
official-doom-root=/tmp/c99inrust-doom-src
linuxdoom=/tmp/c99inrust-doom-src/linuxdoom-1.10
compiler=target/debug/c99inrust
compile_ok=62 compile_fail=0
link_status=0
/out/linuxdoom-c99inrust: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=0559184d31ed27429c4f94a0451120a9eff614fa, for GNU/Linux 3.2.0, with debug_info, not stripped
binary=/tmp/c99inrust-doom-link-scan-7578468-edge-more/linuxdoom-c99inrust
```

## Local Mega Oracle Recheck

Date: 2026-05-21 08:56 KST
Commit: `d9836c4 test(c99): add requested mega oracle wave`

This rechecked the no-IWAD compile/link gate after adding the next 20 requested
deep C99 oracle tests and the compiler fixes they exposed. This did not run
DISPLAY, VNC, Xvfb, manual play, or any IWAD-backed smoke.

Command:

```bash
tools/doom-link-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-link-scan-d9836c4-mega-c
```

Result:

```text
official-doom-root=/tmp/c99inrust-doom-src
linuxdoom=/tmp/c99inrust-doom-src/linuxdoom-1.10
compiler=target/debug/c99inrust
compile_ok=62 compile_fail=0
link_status=0
/out/linuxdoom-c99inrust: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=fb94a33afbca55422922016083b19176afb27bbf, for GNU/Linux 3.2.0, with debug_info, not stripped
binary=/tmp/c99inrust-doom-link-scan-d9836c4-mega-c/linuxdoom-c99inrust
```

## CI Recheck After Mega-D Oracle Wave

Date: 2026-05-21 09:24 KST
Commit: `5a2f0bb test(c99): add mega oracle edge wave`
GitHub Actions run: `26197824812`

The full CI workflow completed successfully after adding another 20
Clang-oracle edge cases. The `doom compile/link scan` job rebuilt the compiler,
cloned the official public Doom source, compiled all 62 official translation
units, and linked the Linux/X11 executable without requiring an IWAD or display.

Result:

```text
compile_ok=62 compile_fail=0
link_status=0
conclusion=success
```

## CI Recheck After Local Struct Array Support

Date: 2026-05-21 10:36 KST
Commit: `f587a7c fix(c99): support local struct arrays`
GitHub Actions run: `26200199678`

The full CI workflow completed successfully after adding Clang-oracle coverage
and compiler support for local arrays of struct objects decaying to typed
struct pointers. The `doom compile/link scan` job rebuilt the compiler, cloned
the official public Doom source, compiled all 62 official translation units,
linked the Linux/X11 executable, and uploaded the compile/link proof artifact.

Downloaded artifact: `doom-compile-link-proof-f587a7c0a41d4e61a8ba9b3ff8a6127464f26898`

Compile scan result:

```text
official-doom-root=/home/runner/work/_temp/DOOM
linuxdoom=/home/runner/work/_temp/DOOM/linuxdoom-1.10
compiler=target/debug/c99inrust
ok=62
fail=0
```

Link scan result:

```text
official-doom-root=/home/runner/work/_temp/DOOM
linuxdoom=/home/runner/work/_temp/DOOM/linuxdoom-1.10
compiler=target/debug/c99inrust
compile_ok=62 compile_fail=0
link_status=0
/out/linuxdoom-c99inrust: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=419dfd402923c6c45dbd7a06a5f66e4ff505bcea, for GNU/Linux 3.2.0, with debug_info, not stripped
binary=/home/runner/work/_temp/doom-link-scan/linuxdoom-c99inrust
conclusion=success
```
