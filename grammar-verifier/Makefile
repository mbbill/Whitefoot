override PYTHON := python3
override PYTHON_ISOLATED := $(PYTHON) -I -S -B
override RUST_TOOLCHAIN := 1.91.1
override PINNED_CARGO := env RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) CARGO_NET_OFFLINE=true cargo

check: test review-packet installed-evidence float-evidence frontend-boundary-evidence
	@echo "== GRAMMAR EVIDENCE GREEN; NO LANGUAGE-CHANGE CLAIM =="

test: runner-test oracle-test static-test static-quality

runner-test:
	$(PYTHON_ISOLATED) -m unittest discover -s $(CURDIR)/tests -p 'test_*.py' -v

oracle-test:
	$(PYTHON_ISOLATED) $(CURDIR)/oracle/tests/run.py

static-test:
	$(PINNED_CARGO) test --locked --offline --all-targets --all-features --manifest-path $(CURDIR)/static-auditor/Cargo.toml

static-quality:
	env RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) cargo fmt --all --manifest-path $(CURDIR)/static-auditor/Cargo.toml -- --check
	$(PINNED_CARGO) clippy --locked --offline --all-targets --all-features --manifest-path $(CURDIR)/static-auditor/Cargo.toml -- -D warnings
	env RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) RUSTDOCFLAGS='-D warnings' CARGO_NET_OFFLINE=true cargo doc --locked --offline --no-deps --all-features --manifest-path $(CURDIR)/static-auditor/Cargo.toml

evidence: installed-evidence

review-packet:
	$(PYTHON_ISOLATED) run.py --review-packet-only

installed-evidence:
	$(PYTHON_ISOLATED) run.py

float-evidence:
	$(PYTHON_ISOLATED) $(CURDIR)/proposal/float_canonical_evidence.py --check

form2-evidence:
	$(PYTHON_ISOLATED) -m unittest discover -s $(CURDIR)/tests -p 'test_form2_*.py' -v

frontend-boundary-evidence:
	$(PYTHON_ISOLATED) $(CURDIR)/proposal/frontend_boundary_evidence.py --check

.PHONY: check test runner-test oracle-test static-test static-quality evidence review-packet installed-evidence float-evidence form2-evidence frontend-boundary-evidence
