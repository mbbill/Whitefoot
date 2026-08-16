CARGO := cargo

check: format lint test docs spec
	@echo "== WHITEFOOT COMPILER GATE GREEN (heavy owning tests excluded; run 'make -C compiler heavy' before integrating or activating) =="

format:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets --locked --offline -- -D warnings

test:
	$(CARGO) test --all-targets --locked --offline

# The 30 `#[ignore = "heavy owning test…"]` tests: end-to-end program
# executions (wfgrep, raw DEFLATE, wide scan, UTF-8 pipeline), backend cost
# shapes, and the two heaviest semantic rewalks. Measured 2026-08-16 they
# carried ~44 of the gate's ~50 minutes. `check` green therefore does NOT
# include them; they MUST pass before a branch integrates to main and before
# any activation, via this target. This split changes when they run, never
# whether they exist — deleting or weakening one remains a protected change.
# Enumerated per target rather than --all-targets: the conformance adapter is
# also #[ignore] but belongs to its own `conformance-run` reporting flow and
# must not ride along here.
heavy:
	$(CARGO) test --lib --locked --offline -- --ignored
	$(CARGO) test --test programs --locked --offline -- --ignored

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --locked --offline

spec:
	$(CARGO) run --bin whitefoot-spec --locked --offline

.PHONY: check format lint test docs spec heavy
