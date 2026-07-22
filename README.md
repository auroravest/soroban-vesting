# Stellar Token Vesting — Soroban Smart Contract

On-chain token vesting with cliff-based and linear unlock schedules. Built for Stellar Soroban.

## Features

- **Cliff vesting** — tokens unlock after a cliff period
- **Linear vesting** — gradual daily unlock after cliff
- **On-chain custody** — tokens held in contract until vested
- **Multi-beneficiary** — single contract serves multiple vesting schedules
- **Revocable** — admin can revoke unvested tokens
- **Events** — full event emission for off-chain indexing

## Architecture

```
Creator
  │
  ├─ create_vesting(beneficiary, amount, cliff, duration)
  │   └─ Tokens transferred to contract
  │
  ├─ revoke_vesting(vesting_id)
  │   └─ Unvested tokens returned to creator
  │
  └─ Beneficiary
       └─ claim(vesting_id)
            └─ Vested tokens transferred to beneficiary
```

## Quick Start

```bash
make build
make test
make wasm
```

The Makefile requires GNU Make and Rust with Cargo. Install the WASM target
before building the contract:

```bash
rustup target add wasm32-unknown-unknown
```

Coverage and dependency audits use optional Cargo extensions:

```bash
cargo install cargo-llvm-cov cargo-audit
```

| Target | Command | Purpose |
|--------|---------|---------|
| `build` | `make build` | Build every workspace package in debug mode |
| `test` | `make test` | Run all workspace tests |
| `fmt` | `make fmt` | Check Rust formatting without changing files |
| `lint` | `make lint` | Run Clippy for all targets and fail on warnings |
| `clean` | `make clean` | Remove Cargo build artifacts |
| `wasm` | `make wasm` | Build the `vesting` contract as optimized WASM |
| `coverage` | `make coverage` | Generate workspace coverage with `cargo-llvm-cov` |
| `audit` | `make audit` | Check dependencies with `cargo-audit` |

Override `CARGO`, `CONTRACT_PACKAGE`, or `WASM_TARGET` when a toolchain or
contract uses a different executable, package, or compilation target.

## Contract API

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin, token)` | Admin | One-time contract setup |
| `create_vesting(beneficiary, amount, cliff, duration)` | Creator | Create a vesting schedule |
| `claim(vesting_id)` | Beneficiary | Claim vested tokens |
| `revoke(vesting_id)` | Creator | Revoke unvested tokens |
| `get_vesting(vesting_id)` | Public | View vesting details |
| `get_claimable(vesting_id)` | Public | Calculate claimable amount |

## Security

- All state-changing functions require authorization
- Token transfers happen before storage updates
- Integer overflow protection via checked arithmetic
- Emergency pause support planned

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Issues labeled `GrantFox OSS` are part of the Stellar FWC26 campaign and are eligible for rewards.

## License

MIT © 2026
