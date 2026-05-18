# Branch Rules

Recommended GitHub ruleset for `main`:

- Require pull request before merging.
- Require status checks:
  - `rust`
- Require branches to be up to date before merge.
- Block force pushes.
- Block deletions.
- Require linear history.

The local setup step attempts to create the public GitHub repository. Branch
rules may require GitHub plan/API availability; if the API rejects them, keep
this file as the source of truth and apply the equivalent rules in the UI.
