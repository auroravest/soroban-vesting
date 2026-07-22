# Contributing to Soroban Vesting

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/soroban-vesting.git`
3. Create a branch: `git checkout -b feat/your-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Run linter: `cargo clippy -- -D warnings`
7. Push and open a Pull Request

## Development Setup

- Rust 1.81+
- Soroban SDK 22.x
- WASM target: `rustup target add wasm32-unknown-unknown`

### Pre-commit Checks

Install [pre-commit](https://pre-commit.com/) and enable the repository hooks:

```bash
pre-commit install
```

The hooks run `cargo fmt --all -- --check` and workspace-wide Clippy whenever a
commit includes Rust files. Run both hooks against the whole repository at any
time with:

```bash
pre-commit run --all-files
```

To skip one hook for an exceptional commit, set `SKIP=cargo-fmt` or
`SKIP=cargo-clippy` for that `git commit` command. `git commit --no-verify`
skips all hooks. Any skipped check must still pass before opening a pull
request.

## Issue Labels

| Label | Description |
|-------|-------------|
| `GrantFox OSS` | FWC26 campaign issue, eligible for rewards |
| `good first issue` | Suitable for new contributors |
| `security` | Security audit / hardening |
| `bug` | Something is broken |
| `enhancement` | New feature or improvement |
| `documentation` | Docs improvements |

## Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `checked_add` / `checked_mul` for all arithmetic
- Emit events for all state-changing operations
- Include tests for both happy path and edge cases

## PR Checklist

- [ ] Tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt --all -- --check`
- [ ] No test snapshots committed
- [ ] Issue referenced in PR description
