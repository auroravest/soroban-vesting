CARGO ?= cargo
CONTRACT_PACKAGE ?= vesting
WASM_TARGET ?= wasm32-unknown-unknown

.PHONY: build test fmt lint clean wasm coverage audit

build:
	$(CARGO) build --workspace

test:
	$(CARGO) test --workspace

fmt:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

clean:
	$(CARGO) clean

wasm:
	$(CARGO) build --package $(CONTRACT_PACKAGE) --release --target $(WASM_TARGET)

coverage:
	$(CARGO) llvm-cov --workspace --all-features

audit:
	$(CARGO) audit
