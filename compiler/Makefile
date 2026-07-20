override PYTHON := python3 -B
override CARGO := $(PYTHON) tools/cargo_policy.py

check: policy format build lint test docs reproducible
	@echo "== RUST COMPILER FOUNDATION GREEN; IMPLEMENTATION INCOMPLETE =="

policy:
	$(PYTHON) tools/test_cargo_policy.py
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
	$(CARGO) doc --workspace --all-features --no-deps --locked --offline

reproducible:
	$(PYTHON) tools/check_reproducible.py

.PHONY: check policy format build lint test docs reproducible
