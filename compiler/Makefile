CARGO := cargo

check: format lint test docs spec
	@echo "== WHITEFOOT COMPILER GATE GREEN =="

format:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets --locked --offline -- -D warnings

# The gate profile (Cargo.toml [profile.gate]) is release codegen with debug
# assertions and overflow checks retained: the suite's owning tests analyze
# real programs, ~16x faster there than at the unoptimized default, so the
# whole suite runs in one gate with no fast/heavy split. A bare
# `cargo test <filter>` during development still uses the default profile
# and keeps incremental builds cheap.
test:
	$(CARGO) test --profile gate --all-targets --locked --offline

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --locked --offline

spec:
	$(CARGO) run --bin whitefoot-spec --locked --offline

.PHONY: check format lint test docs spec
