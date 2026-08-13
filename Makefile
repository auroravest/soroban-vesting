# Makefile for Stellar Token Vesting (Soroban)

RUST := rustup run 1.88.0 cargo
TARGET ?= wasm32-unknown-unknown

.PHONY: build test fmt lint wasm clean

## Build the contract (host, debug)
build:
	$(RUST) build

## Run all tests
test:
	$(RUST) test

## Format all source files
fmt:
	$(RUST) fmt --all

## Check formatting (CI)
fmt-check:
	$(RUST) fmt --all -- --check

## Lint with clippy (strict, -D warnings)
lint:
	$(RUST) clippy --all-targets -- -D warnings

## Build release WASM for deployment
wasm:
	$(RUST) build --release --target $(TARGET)

## Clean build artifacts
clean:
	$(RUST) clean
