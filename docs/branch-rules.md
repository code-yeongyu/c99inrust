# Branch Rules

Recommended GitHub ruleset for `main` when branch protection is re-enabled:

- Require pull request before merging.
- Require status checks:
  - `rust`
- Require branches to be up to date before merge.
- Block force pushes.
- Block deletions.
- Require linear history.

Current repository state:

- Repository: `https://github.com/code-yeongyu/c99inrust`
- Default branch: `main`
- Active rulesets: none (`gh api repos/code-yeongyu/c99inrust/rulesets` returned `[]`)
- Classic branch protection: none (`GET /branches/main/protection` returned `Branch not protected`)
- Current workflow: direct commits to `main`, push after every commit.

Historical applied repository ruleset:

- Repository: `https://github.com/code-yeongyu/c99inrust`
- Ruleset: `main protection`
- Ruleset ID: `16534619`
- Applied: `2026-05-18T19:46:06+09:00`
- Deleted before the current direct-main workflow.
