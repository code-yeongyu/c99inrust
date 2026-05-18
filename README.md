# c99inrust

`c99inrust` is a strict Rust C99 compiler project aimed at compiling the
official public Doom source release from id Software.

Written using [`../omo`](https://github.com/code-yeongyu/oh-my-openagent).

## Status

This repository currently ships the first verified vertical slice:

| Surface | Status |
| ------- | ------ |
| Lexer | comments, identifiers, C keywords, integer/string/char literals, punctuators |
| Preprocessor | quoted includes, object-like macros, `#ifdef`, `#ifndef`, `#else`, `#endif` |
| Parser | `int name(void) { return <constant-expression>; }` |
| IR | checked constant evaluation |
| Codegen | native macOS ARM64 assembly, plus modeled x86_64 Darwin/Linux assembly |
| Doom | official source audit command and QA plan |

Full C99, full Doom playability, all-world architecture coverage, and
Clang-beating optimization are explicit future milestones. The compiler will
not claim those until the checks prove them.

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
int main(void) { return 40 + 2; }
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

The next compiler milestone is a Doom-shaped frontend: typedefs, structs,
enums, pointers, arrays, declarations, function calls, variadic declarations,
conditional preprocessing, and the Linux SysV ABI.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src tests
```

Manual QA must run the CLI in tmux without `tmux kill-server`.

## License

MIT.
