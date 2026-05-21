# c99inrust

`c99inrust` is a strict Rust C99 compiler project aimed at compiling the
official public Doom source release from id Software.

Written using [`../omo`](https://github.com/code-yeongyu/oh-my-openagent).

## Status

This repository currently ships a verified C99-compiler vertical slice aimed at
the official public Doom source tree:

| Surface | Status |
| ------- | ------ |
| Lexer | comments, identifiers, C keywords, integer/string/char literals, punctuators |
| Preprocessor | local/system includes, `-D`, `#if/#elif/#ifdef/#ifndef/#undef`, object/function-like macros, line splicing |
| Parser | Doom-shaped C declarations, typedefs, structs/unions/enums, pointers, arrays, control flow, calls, expressions, and supported C99 function bodies |
| IR | scoped locals/globals, pointer referents, struct fields, arrays, calls, sizeof, control flow, and Doom-specific libc/X11 surface lowering |
| Codegen | native macOS ARM64 assembly, native executable build via host `cc`, plus x86_64 Linux assembly used for official Doom linking |
| Doom | all 62 official `linuxdoom-1.10` C translation units compile to assembly, link into a Linux/X11 executable, open a viewable X11 window under Xvfb, accept scripted keyboard input, move the player under live state sampling, and survive until the QA timeout |

Full C99, human-verified Doom playability, all-world architecture coverage,
and general Clang-beating optimization are still active milestones. The current
benchmark slice beats local Apple Clang for compile-to-assembly time,
one-command binary build time, runtime, and basic assembly-shape counters; see
`docs/qa/2026-05-18-rust-slop-performance.md`.

The full-spec track is enforced through a Clang-oracle test harness: supported
C99 snippets must compile with `c99inrust`, compile with the host C compiler in
`-std=c99` mode, and produce the same observable exit code before the supported
surface can grow.

## Install

```bash
cargo build --release
```

No Rust dependencies are used.

## Use

```bash
c99inrust lex examples/answer.c
c99inrust preprocess -I include examples/answer.c
c99inrust compile -S examples/answer.c -o answer.s
cc answer.s -o answer
./answer
c99inrust build examples/answer.c -o answer
./answer
```

The current compile slice accepts:

```c
int answer(void) { return 40; }
int main(void) { int total = 0; for (int i = 0; i < 2; i = i + 1) { total = total + answer(); } return total - 38; }
```

## Official Doom Target

The public source target is [`id-Software/DOOM`](https://github.com/id-Software/DOOM).
The release is Linux-oriented, requires real Doom data such as `doom1.wad`, and
uses X11/Xext-era platform code.

Audit a checkout:

```bash
git clone https://github.com/id-Software/DOOM /tmp/DOOM
cargo run -- doom-audit /tmp/DOOM
```

`doom-audit` checks that the input checkout has the expected 62-unit
`linuxdoom-1.10` source shape and prints recorded QA evidence with
`recorded-*` prefixes. It does not run the compile/link/movement smoke itself;
use the smoke scripts below for live verification.

Current Doom-facing compiler evidence: `preprocess + lex + parse-check` runs
across all 124 official `linuxdoom-1.10` C/header files with `NORMALUNIX` and
`LINUX` defined, `compile -S` emits x86_64 Linux assembly for all 62 official
Doom C translation units, and those assembly files link into a Linux/X11 ELF
with system `gcc`. The latest CI no-IWAD compile/link gate on commit
`f587a7c` in GitHub Actions run `26200199678` produced
`compile_ok=62 compile_fail=0`, `link_status=0`, and an x86_64 Linux/X11 ELF
with BuildID `419dfd402923c6c45dbd7a06a5f66e4ff505bcea`. The latest
automated Xvfb movement smoke on commit `2eb3f12` produced
`compile_ok=62 compile_fail=0`, `link_status=0`, `display_status=0`,
`window_status=0`, `input_status=0`, `movement_status=0`, and `run_status=124`
after dispatching `Up` to a viewable `320x200` Doom window. Live memory samples
showed the player coordinates change from `69206016,-236978176` to
`69194790,-209582386` while the player mobj stayed linked to
`P_MobjThinker`. This proves visible-window startup, scripted keyboard delivery,
keyboard-driven player movement, and survival to the QA timeout; it is still not
a human playthrough claim.

Repeat the current Doom smoke on a machine with Docker, the public Doom
checkout, and a legal IWAD:

```bash
cargo build
tools/doom-link-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-link-scan
tools/doom-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-smoke
tools/doom-input-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-input-smoke
tools/doom-movement-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-movement-smoke
DOOM_MANUAL_RUN=0 tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-manual-play
```

The link scan is also a CI gate and does not require an IWAD: it proves that all
62 official C translation units compile to x86_64 Linux assembly and link into a
Linux/X11 ELF.

The latest manual harness build-only run on commit `20120f6` compiled all 62
official C files, linked the Doom binary, wrote `manual-transcript.txt`, and
stopped deliberately with `manual_run=skipped` because `DOOM_MANUAL_RUN=0` was
set. A completed human-visible X11 playthrough transcript is still the
remaining manual evidence gap; validate one with
`tools/doom-validate-manual-transcript.sh` before claiming manual playability,
or set `DOOM_MANUAL_VALIDATE=1` on the manual harness run.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
tools/check-rust-no-excuses.sh src tests
```

Manual QA must run the CLI in tmux without `tmux kill-server`.

## License

MIT.
