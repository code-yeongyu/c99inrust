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
