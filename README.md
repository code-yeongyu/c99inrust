# c99inrust

`c99inrust` is a strict Rust C99 compiler project aimed at compiling the
official public Doom source release from id Software.

Written using [`../omo`](https://github.com/code-yeongyu/oh-my-openagent).

## Status

This repository currently ships the first verified vertical slice:

| Surface | Status |
| ------- | ------ |
| Lexer | comments, identifiers, C keywords, integer/string/char literals, punctuators |
| Preprocessor | local/system includes, `-D`, `#if/#elif/#ifdef/#ifndef/#undef`, object/function-like macros, line splicing |
| Parser | int function bodies with local `int` declarations, assignments, zero-argument calls, `if`/`else`, `while`, `for`, blocks, returns, plus Doom-shaped surface declarations |
| IR | scoped local-slot lowering, label lowering, zero-argument call lowering, constant integer call folding, and short-circuit logical lowering for supported `int` statements and expressions |
| Codegen | native macOS ARM64 assembly, native executable build via host `cc`, plus modeled x86_64 Darwin/Linux assembly with zero-argument direct calls |
| Doom | official source audit command and QA plan |

Full C99, full Doom playability, all-world architecture coverage, and
general Clang-beating optimization are explicit future milestones. The current
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

Current Doom-facing evidence: `preprocess + lex + parse-check` runs across all
124 official `linuxdoom-1.10` C/header files with `NORMALUNIX` and `LINUX`
defined. `parse-check` is a surface parser gate for top-level typedefs,
declarations, prototypes, and function-definition boundaries; it is not yet full
semantic C parsing or code generation.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src tests
```

Manual QA must run the CLI in tmux without `tmux kill-server`.

## License

MIT.
