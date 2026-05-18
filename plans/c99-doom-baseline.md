# c99inrust Doom Baseline Plan

## Scope

This repository starts empty. The in-turn deliverable is a strict Rust compiler
foundation with a runnable vertical slice:

- lex and preprocess C-shaped input
- parse `int` functions that return constant integer expressions
- emit host-assembling assembly for the native macOS ARM64 environment and
  model x86_64 SysV/Linux and x86_64 Darwin output
- document the official Doom source target and blockers honestly
- provide CI, docs, git history, and a verifier gate

This is not yet a full C99 implementation, not yet Doom-playable, and not yet
faster than Clang. Those are product milestones, not honest one-turn claims.

## Sources

- Official Doom source: https://github.com/id-Software/DOOM
- Historic Doom source archive: https://www.gamers.org/pub/idgames/idstuff/source/
- C99 draft N1256: https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1256.pdf
- Microsoft x64 ABI: https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention

## Parallel Task Graph

| Task | Depends on | Acceptance |
| ---- | ---------- | ---------- |
| Repo scaffold | none | `cargo metadata` succeeds |
| Red tests | scaffold | targeted tests fail for missing implementation |
| Frontend | red tests | lexer/preprocessor tests pass |
| Parser/IR | frontend | constant-return C tests pass |
| Native codegen | parser/IR | assembly links and executable returns expected code |
| Doom docs/audit | frontend | audit command scans official source checkout |
| CI/docs | scaffold | workflow and README describe exact supported slice |
| tmux QA | codegen/CLI | captured tmux pane shows compile/link/run |
| verifier | all | `gpt-5.2 xhigh` approves or emits fixes |
| commits | verified | Conventional Commits, clean status |

## Dependency Matrix

| Component | Requires | Provides |
| --------- | -------- | -------- |
| `front_end` | source text | tokens and preprocessed text |
| `parser` | tokens | syntax tree |
| `ir` | syntax tree | constant evaluation |
| `codegen` | evaluated functions | target assembly |
| CLI | all modules | user-facing compiler commands |
| Doom audit | filesystem | feature gap report |

## QA Scenarios

Happy path:
- compile `int main(void) { return 42; }`
- assemble with host `cc`
- run binary and observe exit code `42`

Edge cases:
- comments around tokens
- object-like macros
- include search path
- arithmetic precedence
- unsupported C emits a clear diagnostic

Adjacent surface:
- `lex` output remains deterministic
- `preprocess` preserves non-directive source order
- `doom-audit` never vendors Doom source into this repo

## Commit Strategy

1. `chore(repo): initialize strict Rust compiler workspace`
2. `feat(frontend): add C lexer and preprocessor slice`
3. `feat(compiler): emit native assembly for constant returns`
4. `docs(doom): document official source QA path`
5. `ci(repo): add Rust validation workflow`

Each commit must build and pass the relevant tests at its point in history.
