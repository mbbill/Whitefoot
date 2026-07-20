PYTHON = python3 -B
CARGO = cargo

check: policy format build lint test docs reproducible
	@echo "== RUST COMPILER FOUNDATION GREEN; IMPLEMENTATION INCOMPLETE =="

policy:
	$(PYTHON) tools/test_check_workspace.py
	$(PYTHON) tools/check_workspace.py

format:
	$(CARGO) fmt --all -- --check

build:
	$(CARGO) check --workspace --all-targets --all-features --locked --offline

lint:
	$(CARGO) clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	$(CARGO) test --workspace --all-targets --all-features --locked --offline

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --all-features --no-deps --locked --offline

reproducible:
	$(PYTHON) tools/check_reproducible.py

.PHONY: check policy format build lint test docs reproducible
