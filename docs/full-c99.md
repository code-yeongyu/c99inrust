# Full C99 And Clang Compatibility Track

The goal is full hosted C99 behavior with an explicit Clang-compatible mode for
the code that real projects depend on, including common undefined-behavior
patterns that Clang happens to lower predictably on a target.

This repository does not claim that goal yet.

## Evidence Gate

Each new language feature needs:

1. A focused parser/semantic/codegen test.
2. A differential oracle test against the host C compiler with `-std=c99`.
3. A target note when behavior is implementation-defined or undefined by C.
4. Doom-source evidence when the feature is required by `linuxdoom-1.10`.

The oracle harness lives in `tests/clang_oracle.rs` and split modules under
`tests/clang_oracle/`. Each case compiles a C snippet with both compilers, links
both outputs through the platform toolchain, runs both executables, and compares
observable stdout plus exit code. The suite currently contains 373 oracle tests.

Current covered slices include fundamentals, control flow, calls, multi-file
extern linkage, Doom-shaped globals, pointer and array operations, undefined or
implementation-defined C behavior captured in Clang-compatible mode, and deeper
C99 edges such as sequence points, aliasing/type punning, K&R definitions,
function-pointer arrays, struct/union layouts, VLA shapes, `_Bool`, enums,
Duff-style fallthrough, `goto` out of VLA scope, volatile access, macro
stringification/token pasting/variadics, predefined macros, trigraphs/digraphs,
wide-character literals, signed bitwise operations, initializer nesting,
`sizeof`, char signedness, `long long`, decimal floats, and hex float constants.
Local struct initializers are also covered, including brace-sensitive nested
initialization, partial zero-fill, copy initialization, mixed scalar/pointer
fields, and scalar array fields.
The latest deep edge slices add exact-result oracle checks for nested aggregate
layout, function-pointer tables, old-style definitions, anonymous aggregates,
GNU `typeof`/attributes, multi-file extern objects, static inline calls,
`restrict`, constant-expression array sizes, macro expansion, variadic macros,
predefined macros, nested initializers, ternary side effects, `sizeof`
evaluation suppression, incompatible pointer casts, char signedness, `long long`
arithmetic, decimal float forms, and hex float forms.
Additional Doom-adjacent slices cover local short scalar narrowing, local static
state, local include plus multi-file typedef/macro linkage, extern function
pointer installation, pointer arrays over structs, partial aggregate zero-fill,
`defined`/`#elif`, parameter-array `sizeof` decay, struct copy of nested arrays,
comma-expression initializer ordering, local char matrix row decay and nested
byte reads, and static local multi-declaration initializers.
New Doom-runtime table slices cover global and multi-file string pointer tables,
Doom-style fixed-point multiply/shift behavior, signed-byte `ticcmd_t` fields,
event ring-buffer struct copies through pointers, recursive mobj links, state
function-pointer tables, and drawseg pointer cursor arithmetic.
The next-wave requested edge suites add repeated passes over all 20 requested
categories with stdout-visible cases for nested padding/alignment, function
pointer arrays, K&R definitions, anonymous aggregate members, GNU extensions,
multi-file extern linkage, static inline calls, `restrict`, constant
expressions, macro recursion, variadic macros, predefined macros, nested braces,
ternary side effects, `sizeof`, incompatible casts, char signedness, `long long`,
decimal float literals, and hex float constants. The latest mega-D wave repeats
those categories with additional observable stdout plus exit-code checks and
keeps each Rust test module below the 250 pure-LOC project rule.

## Undefined Behavior Policy

C undefined behavior is not portable semantics. `c99inrust` will track it under
named compatibility modes:

- strict C99: diagnostics or optimization freedom where the standard permits it
- clang-compatible: match Clang's observable behavior for selected target and
  optimization settings

No UB behavior is considered supported until a target-specific oracle test
captures it.
