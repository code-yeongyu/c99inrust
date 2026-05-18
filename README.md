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
| Parser | int function bodies with local `int` declarations, assignments, `if`/`else`, `while`, `for`, blocks, returns, plus Doom-shaped surface declarations |
| IR | scoped local-slot lowering, label lowering, and short-circuit logical lowering for supported `int` statements and expressions |
| Codegen | native macOS ARM64 assembly, plus modeled x86_64 Darwin/Linux assembly |
| Doom | official source audit command and QA plan |

Full C99, full Doom playability, all-world architecture coverage, and
Clang-beating optimization are explicit future milestones. The compiler will
not claim those until the checks prove them.

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
```

The current compile slice accepts:

```c
int main(void) { int total = 0; for (int i = 0; i < 5; i = i + 1) { total = total + i; } return total; }
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
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src tests
```

Manual QA must run the CLI in tmux without `tmux kill-server`.

## License

MIT.
