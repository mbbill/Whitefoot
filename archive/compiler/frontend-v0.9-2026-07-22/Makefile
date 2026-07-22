# Compiler gate — plain cargo. No-unsafe is guaranteed by the workspace lint
# `unsafe_code = "forbid"`; the toolchain is pinned by rust-toolchain.toml and
# deps by Cargo.lock. Nothing here re-implements in Python what cargo already does.

PYTHON := python3 -B
CARGO := cargo

check: format lint test docs contracts
	@echo "== RUST FRONTEND GREEN; SEMANTIC IMPLEMENTATION ABSENT =="

format:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	$(CARGO) test --workspace --all-targets --all-features --locked --offline

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --all-features --no-deps --locked --offline

# Interim: verify that the compiler's generated terminal/grammar/canonical tables
# still match the spec. This re-derives the grammar in Python and is the wrong
# design; it folds into the native grammar verifier (see docs/roadmap.md) that
# reuses the compiler's own lexer and parser. Kept until that tool exists.
contracts:
	$(PYTHON) tools/test_terminal_contract.py
	$(PYTHON) tools/check_terminal_contract.py
	$(PYTHON) tools/test_syntax_contract.py
	$(PYTHON) tools/check_syntax_contract.py
	$(PYTHON) tools/test_canonical_contract.py
	$(PYTHON) tools/check_canonical_contract.py

.PHONY: check format lint test docs contracts
