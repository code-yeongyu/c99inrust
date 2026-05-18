# Rust, Slop, And Performance Gate Evidence

Date: 2026-05-18
Codegen performance commit: `d93fdf8`
Strict Rust gate commit: `a9648cb`
Benchmark root: `/tmp/c99inrust-perf-rXMLo7`
Local compiler: Apple Clang `21.0.0 (clang-2100.0.123.102)`

## Rust Programmer Gate

Commands run locally:

```text
rustup run stable cargo fmt --all -- --check
rustup run stable cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo
bash /Users/yeongyu/.agents/skills/rust-programmer/scripts/check-no-excuse-rules.sh src/parser/mod.rs src/ir/mod.rs src/codegen/mod.rs src/bin/c99inrust.rs src/front_end/lexer.rs src/front_end/preprocessor.rs tests/compiler.rs tests/clang_oracle.rs
rustup run stable cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo machete
cargo deny check
cargo audit
```

Results:

```text
clippy: PASS, no warnings
no-excuse: PASS for 8 files
cargo test: PASS, 23 tests
nextest: PASS, 23 tests
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
compile-to-assembly, 200 iterations:
c99inrust real 0.54 user 0.17 sys 0.26
clang     real 6.28 user 2.43 sys 2.59

end-to-end binary build, 50 iterations:
c99inrust real 3.94 user 2.15 sys 2.03
clang     real 2.43 user 1.76 sys 1.69
```

Runtime results:

```text
single run:
c99inrust real 0.04 user 0.04 sys 0.00
clang     real 0.04 user 0.03 sys 0.00

10 runs:
c99inrust real 0.53 user 0.45 sys 0.02
clang     real 0.62 user 0.45 sys 0.02
```

Assembly-output metrics:

```text
c99_lines=47
clang_lines=68
c99_instruction_lines=37
clang_instruction_lines=41
c99_stack_refs=16
clang_stack_refs=15
linked __TEXT size: tied at 16384
```

Status:

```text
PASS: c99inrust compile-to-assembly time is faster than local Clang.
PASS: c99inrust 10-run wall-clock runtime is faster; user/sys CPU time ties.
PASS: c99inrust assembly has fewer total lines and fewer instruction lines.
BLOCKER: c99inrust end-to-end binary build is slower than local Clang.
BLOCKER: c99inrust has one more stack-reference line than Clang on this benchmark.
```

The full "better than local Clang for build time, runtime, and asm output
quality" requirement is therefore not fully satisfied yet if build time means
end-to-end binary creation or if stack-reference count is a required assembly
quality metric.
