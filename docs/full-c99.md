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

The first oracle tests live in `tests/clang_oracle.rs`. They compile a C snippet
with both compilers, link both outputs through the platform toolchain, run both
executables, and compare exit codes. Current covered slices include constant
returns, arithmetic precedence, local `int` declarations/assignments, and
`if`/`else` control flow over integer comparisons.

## Undefined Behavior Policy

C undefined behavior is not portable semantics. `c99inrust` will track it under
named compatibility modes:

- strict C99: diagnostics or optimization freedom where the standard permits it
- clang-compatible: match Clang's observable behavior for selected target and
  optimization settings

No UB behavior is considered supported until a target-specific oracle test
captures it.
