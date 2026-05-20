## Summary

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo`
- [ ] `cargo test --all-targets --all-features`
- [ ] `cargo nextest run --all-targets --all-features`
- [ ] `cargo machete`
- [ ] `cargo deny check`
- [ ] `cargo audit`
- [ ] `tools/check-rust-no-excuses.sh src tests`
- [ ] Doom or compiler CLI smoke-tested in tmux, if behavior changed
