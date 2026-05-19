---
description: Rust file size discipline
globs: ["**/*.rs"]
alwaysApply: false
---

Rust source files must stay at or below 250 pure LOC per file.
Pure LOC means non-blank, non-comment Rust source lines in one `.rs` file.

If you find any `.rs` file above 250 pure LOC, treat it as an immediate design problem, not a formatting problem. Pause and reason deeply about the right module structure: challenge the current design, challenge the first proposed split, then rebuild the design around cohesive responsibilities and stable boundaries.

Before changing behavior, pin the current functionality and observable behavior with focused tests or equivalent executable checks. Refactor only after the behavior is pinned, then keep refactoring until the file-size rule is satisfied without weakening the design.
