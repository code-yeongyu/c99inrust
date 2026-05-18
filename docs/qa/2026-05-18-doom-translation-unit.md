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

## Translation-Unit Slice Status

This was forward progress toward the Doom compile goal, not a playable Doom
claim. At this point, the next required compiler work was Doom function
signatures and bodies:

```text
void and non-int return types
parameter lists
pointers and arrays
static function linkage
function call arguments
broader expression and statement coverage
```

## Compile Scan After Signature Slice

`compile -S` now accepts supported Doom function signatures with `int` and
`void` returns, skips parameter-list tokens, accepts value-less `return;` in
`void` functions, rejects value-less `return;` in `int` functions, rejects
valued returns in `void` functions, and emits a terminal `return;` for `void`
functions whose body can fall through.

Regression coverage added:

```text
compiler_emits_void_functions_with_value_less_return
compiler_adds_terminal_return_to_void_functions_that_can_fall_through
compiler_rejects_value_less_return_from_int_functions
compiler_rejects_value_return_from_void_functions
compiler_accepts_parameter_list_signatures_when_body_does_not_use_parameters
void_function_slice_matches_host_c_compiler_exit_code
parameter_list_signature_slice_matches_host_c_compiler_exit_code
```

Frontend surface gate after this slice:

```text
scan=/tmp/c99inrust-doom-parse-check-after-signature.txt
parse_check_files=124
fail=0
```

Current compile scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-signature.txt
ok=0
fail=62
```

Manual CLI QA was run inside tmux session `c99sigqa` with
`target/debug/c99inrust`:

```text
tmux_session=c99sigqa
void_explicit_exit=42
params_exit=42
void_fallthrough_exit=42
void_fallthrough_terminal_ret=PASS
error: int function must return a value
error: void function cannot return a value
manual_qa=PASS
```

Local Rust/slop gate for the signature slice:

```text
fmt: PASS
strict clippy: PASS, no warnings
rust-programmer no-excuse: PASS for 5 changed files
LSP diagnostics: PASS, 0 diagnostics
cargo test --all-targets --all-features: PASS, 36 tests
cargo nextest run --all-targets --all-features: PASS, 36 tests
cargo machete: PASS
cargo deny check: PASS
cargo audit: PASS
unsafe/miri: N/A; unsafe code is forbidden in Cargo.toml and src/lib.rs
remove-ai-slops: PASS for this slice; no debug leftovers, warning suppressions,
dead code, or needless behavior-changing cleanup found
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL i_main.c
  error: 587:16: expected punctuator =
FAIL i_net.c
  error: 3889:17: expected punctuator )
FAIL m_fixed.c
  error: 429:1: unsupported function definition: FixedMul
FAIL r_sky.c
  error: assignment to undeclared local: skytexturemid
```

The remaining compile blockers have moved beyond the initial `void main(...)`
and parameter-list signature failures into C type/declarator and body coverage:

```text
typedef-backed return types such as fixed_t and boolean
local declarations with non-int Doom typedefs and pointer declarators
multiple declarators in one statement
function call arguments
pointer/member/subscript expressions
global variable storage and lookup
```

This is still not a playable Doom claim. Full success requires compiling all
translation units, linking the Doom executable, and manually running a playable
public Doom target.
