# Doom Translation Unit Compile Progress

Date: 2026-05-18
Public Doom checkout: `id-Software/DOOM` at `a77dfb9`

## Baseline

The public Doom checkout contains 62 C files and 62 header files under
`linuxdoom-1.10`.

```text
cargo run --quiet -- doom-audit /tmp/DOOM
official-doom-root=/tmp/DOOM
linuxdoom-c-files=62
linuxdoom-h-files=62
linuxdoom-makefile=true
status=audited language surface only; full Doom compilation is a future milestone
```

The frontend surface gate still passes across all 124 C/header files:

```text
parse_check_files=124
```

## Compile Scan Before This Slice

Before the translation-unit compile slice, all 62 C files failed before reaching
any function body because `compile -S` expected every token stream to begin with
an `int` function definition.

Representative failure:

```text
FAIL am_map.c
  error: 24:1: expected keyword Int
```

## Compile Scan After This Slice

`compile -S` now skips top-level declarations and prototypes, then compiles only
supported executable function definitions. This moves Doom compilation past
file-level declarations and into actual function signatures or bodies.

Current scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-tu.txt
fail=62
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7137:1: unsupported function definition: AM_getIslope
FAIL i_main.c
  error: 579:1: unsupported function definition: main
FAIL d_net.c
  error: 4126:13: expected expression
FAIL m_random.c
  error: 60:20: expected punctuator ;
```

## Status

This is forward progress toward the Doom compile goal, not a playable Doom
claim. The next required compiler work is Doom function signatures and bodies:

```text
void and non-int return types
parameter lists
pointers and arrays
static function linkage
function call arguments
broader expression and statement coverage
```
