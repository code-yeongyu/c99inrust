# Architecture

`c99inrust` follows the C translation pipeline instead of parsing raw source as
if preprocessing did not exist.

## Pipeline

1. Source text
2. Preprocessor
3. Lexer
4. Parser
5. IR lowering
6. Target assembly emission
7. External assembler/linker for the current milestone

## Current Target Model

The implemented backend emits assembly for constant-return functions:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`

The native target is detected at compile time. On this workstation it is
`aarch64-apple-darwin`.

## Doom Milestone Gap

The frontend now handles the official `linuxdoom-1.10` source tree through
preprocessing, lexing, and surface parse-check under the Linux build defines used
by the upstream Makefile:

- `-D NORMALUNIX`
- `-D LINUX`

That proves include traversal, line splicing, comment removal before macro
expansion, conditional directives, Doom-shaped macros, typedef/declaration
surface recognition, and function-body boundary scanning across all 124 C/header
files. Compiling official Doom still requires more than surface parsing:

- typedef names
- struct, union, and enum layout
- pointers and arrays
- function calls and external symbols
- SysV ABI argument and return handling
- object emission or a robust assembly/link path
- Linux/X11 platform dependencies and a legal IWAD for playability

Those gaps are tracked as product milestones, not hidden behind README claims.
