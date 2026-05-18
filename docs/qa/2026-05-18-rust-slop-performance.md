# Rust, Slop, And Performance Gate Evidence

Date: 2026-05-18
Codegen performance baseline: `366311b`
Strict Rust gate baseline: `366311b`
Benchmark root: `/tmp/c99inrust-perf-AyDmJQ`
Local compiler: Apple Clang `21.0.0 (clang-2100.0.123.102)`

## Rust Programmer Gate

Commands run locally:

```text
rustup run stable cargo fmt --all -- --check
rustup run stable cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src/ir/mod.rs src/codegen/mod.rs src/bin/c99inrust.rs tests/compiler.rs tests/clang_oracle.rs
rustup run stable cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
```

Results:

```text
clippy: PASS, no warnings
no-excuse: PASS for 5 files
cargo test: PASS, 27 tests
nextest: PASS, 27 tests
cargo machete: PASS, no unused dependencies
cargo deny: PASS, advisories/bans/licenses/sources ok
cargo audit: PASS, 1 crate scanned
unsafe/miri: N/A for this slice; crate has unsafe_code = "forbid" and #![forbid(unsafe_code)]
```

## Remove AI Slops Gate

Scope: branch diff against `main`.

Behavior lock before cleanup:

```text
rustup run stable cargo test --all-targets --all-features
```

Cleanup findings and actions:

```text
dead/debug leftovers: none found
warning-policy drift: fixed README and CI clippy commands to use -D warnings
performance equivalence: replaced AArch64 booleanized comparison branches with direct conditional branches
performance equivalence: lowered AArch64 local +/- small integer updates without temporary stack spills
performance equivalence: added a native `build` command that pipes generated assembly into `cc`
performance equivalence: used preserved AArch64 register `x19` for direct-call RHS temporaries
performance equivalence: folded calls to same-translation-unit integer constant functions
excessive bool state: replaced conditional preprocessor branch booleans with BranchState
false-positive no-excuse trigger: renamed ConditionParser::expect to expect_token
strict lint slop: fixed pedantic/nursery/cargo clippy findings without #[allow(...)]
```

Quality gates after cleanup:

```text
fmt: PASS
strict clippy: PASS
LSP diagnostics over src: PASS, 0 diagnostics
no-excuse: PASS
cargo test: PASS
nextest: PASS
deny/audit/machete: PASS
```

## Clang Performance Gate

Benchmark source:

```c
int tick(void) { return 1; }
int main(void) {
    int total = 0;
    for (int i = 0; i < 50000000; i = i + 1) {
        total = total + tick();
    }
    if (total == 50000000) { return 0; }
    return 1;
}
```

Build-time results:

```text
compile-to-assembly, 300 iterations:
c99inrust real 1.24 user 0.35 sys 0.52
clang     real 8.45 user 3.50 sys 3.48

one-command binary build, 100 iterations:
c99inrust build real 5.18 user 3.59 sys 3.62
clang           real 6.75 user 4.28 sys 3.77
```

Runtime results:

```text
100 runs:
c99inrust real 3.06 user 2.58 sys 0.20
clang     real 5.63 user 4.89 sys 0.23
```

Assembly-output metrics:

```text
c99_lines=41
clang_lines=68
c99_instruction_lines=30
clang_instruction_lines=41
c99_stack_refs=10
clang_stack_refs=15
c99_status=0
clang_status=0
```

Status:

```text
PASS: c99inrust compile-to-assembly time is faster than local Clang.
PASS: c99inrust one-command binary build is faster than local Clang.
PASS: c99inrust 100-run wall-clock and user/sys runtime are faster than local Clang.
PASS: c99inrust assembly has fewer total lines, instruction lines, and stack references.
```

This file proves the local benchmark performance gate only. It does not prove
the broader thread goal because full public Doom compile/link/run/play evidence
is still missing.

## Signature Slice Recheck

Date: 2026-05-18
Scope: `src/parser/mod.rs`, `src/ir/mod.rs`, `src/codegen/mod.rs`,
`tests/compiler.rs`, `tests/clang_oracle.rs`

Commands run locally after the Doom signature slice:

```text
rustup run stable cargo fmt --all -- --check
rustup run stable cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src/parser/mod.rs src/ir/mod.rs src/codegen/mod.rs tests/compiler.rs tests/clang_oracle.rs
LSP diagnostics on the five changed Rust files
rustup run stable cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
```

Results:

```text
fmt: PASS
strict clippy: PASS, no warnings
no-excuse: PASS for 5 files
LSP diagnostics: PASS, 0 diagnostics
cargo test: PASS, 36 tests
nextest: PASS, 36 tests
cargo machete: PASS, no unused dependencies
cargo deny: PASS, advisories/bans/licenses/sources ok
cargo audit: PASS, 1 crate scanned
unsafe/miri: N/A; crate has unsafe_code = "forbid" and #![forbid(unsafe_code)]
remove-ai-slops scan: PASS for this slice; no debug leftovers, dead code, warning suppressions, or needless behavior-changing cleanup found
```

## Parameter Binding Slice Recheck

Date: 2026-05-18
Scope: `src/parser/mod.rs`, `src/ir/mod.rs`, `src/codegen/mod.rs`,
`tests/compiler.rs`, `tests/clang_oracle.rs`

Commands run locally after the parameter binding slice:

```text
rustup run stable cargo fmt --all -- --check
rustup run stable cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src/parser/mod.rs src/ir/mod.rs src/codegen/mod.rs tests/compiler.rs tests/clang_oracle.rs
LSP diagnostics on the five changed Rust files
rustup run stable cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
```

Results:

```text
fmt: PASS
strict clippy: PASS, no warnings
no-excuse: PASS for 5 files
LSP diagnostics: PASS, 0 diagnostics
cargo test: PASS, 45 tests
nextest: PASS, 45 tests
cargo machete: PASS, no unused dependencies
cargo deny: PASS, advisories/bans/licenses/sources ok
cargo audit: PASS, 1 crate scanned
unsafe/miri: N/A; crate has unsafe_code = "forbid" and #![forbid(unsafe_code)]
remove-ai-slops scan: PASS for this slice; no debug leftovers, dead code, warning suppressions, or needless behavior-changing cleanup found
```

## Expression Slice Recheck

Date: 2026-05-18
Scope: `src/parser/mod.rs`, `src/ir/mod.rs`, `src/codegen/mod.rs`,
`tests/compiler.rs`, `tests/clang_oracle.rs`

Commands run locally after the signed long long cast, function call argument,
and conditional expression slice:

```text
rustup run stable cargo fmt --all -- --check
rustup run stable cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src/parser/mod.rs src/ir/mod.rs src/codegen/mod.rs tests/compiler.rs tests/clang_oracle.rs
LSP diagnostics on the five changed Rust files
rustup run stable cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
```

Results:

```text
fmt: PASS
strict clippy: PASS, no warnings
no-excuse: PASS for 5 files
LSP diagnostics: PASS, 0 diagnostics
cargo test: PASS, 51 tests
nextest: PASS, 51 tests
cargo machete: PASS, no unused dependencies
cargo deny: PASS, advisories/bans/licenses/sources ok
cargo audit: PASS, 1 crate scanned
unsafe/miri: N/A; crate has unsafe_code = "forbid" and #![forbid(unsafe_code)]
remove-ai-slops scan: PASS for this slice; no debug leftovers, dead code, warning suppressions, unsafe blocks, or needless behavior-changing cleanup found
```

## Stricter Performance Follow-up

Date: 2026-05-18
Scope: current expression-slice workspace after ABI register assertion hardening.

The final thread goal requires c99inrust to beat local Clang for build time,
runtime, and assembly output quality. The benchmark below is still a local
slice, not a full Doom proof.

Current local results against Apple Clang `21.0.0 (clang-2100.0.123.102)`:

```text
compile-to-assembly, 300 iterations:
c99inrust real 1.37 user 0.38 sys 0.54
clang     real 12.59 user 4.78 sys 5.03

one-command binary build, 3 warmed 100-iteration rounds:
c99inrust real 6.29, 6.63, 6.37; median 6.37
clang     real 7.93, 6.04, 5.89; median 6.04

runtime, 100 runs:
c99inrust real 4.99 user 3.99 sys 0.24
clang     real 7.60 user 5.67 sys 0.27

assembly-output metrics:
c99_lines=47
clang_lines=68
c99_instruction_lines=36
clang_instruction_lines=54
c99_stack_refs=14
clang_stack_refs=15
```

Status:

```text
PASS: c99inrust compile-to-assembly time is faster than local Clang.
FAIL: c99inrust one-command binary build median is not faster than local Clang.
PASS: c99inrust 100-run runtime is faster than local Clang.
PASS: c99inrust assembly has fewer lines, instruction lines, and stack refs.
```

This keeps the stricter performance gate open. Completion still requires a
current proof that the chosen build-time gate beats local Clang, plus the full
public Doom compile/link/run/play evidence.

## Scalar Return Slice Recheck

Date: 2026-05-18
Scope: `src/parser/mod.rs`, `tests/compiler.rs`, `tests/clang_oracle.rs`

Commands run locally after the scalar return signature slice:

```text
rustup run stable cargo fmt --all -- --check
rustup run stable cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src/parser/mod.rs tests/compiler.rs tests/clang_oracle.rs
LSP diagnostics on the three changed Rust files
rustup run stable cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
```

Results:

```text
fmt: PASS
strict clippy: PASS, no warnings
no-excuse: PASS for 3 files
LSP diagnostics: PASS, 0 diagnostics
cargo test: PASS, 42 tests
nextest: PASS, 42 tests
cargo machete: PASS, no unused dependencies
cargo deny: PASS, advisories/bans/licenses/sources ok
cargo audit: PASS, 1 crate scanned
unsafe/miri: N/A; crate has unsafe_code = "forbid" and #![forbid(unsafe_code)]
remove-ai-slops scan: PASS for this slice; no debug leftovers, dead code, warning suppressions, or needless behavior-changing cleanup found
```
