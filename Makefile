# Whitefoot's canonical all-tests entry point: compiler checks and tests, the
# complete native conformance adapter, conformance structure and coverage,
# specification/archive identity, and the recorded-verdict snapshot corpus.
# The adapter prints its current tally rather than baking a count into this
# file.

PY := python3 -B
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
RESEARCH_TEST_TMP := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-research-tests-tmp
RESEARCH_CARGO_TARGET := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-research-tests-target

# The stages `check` runs, in order. Each is a target of its own, so the CI
# jobs that run the gate in parallel run exactly these targets and nothing
# beside them: what a job checks is what this list says it checks.
# `approval-history-integrity` and `spec-archive-integrity` were retired with
# the approval ledger they both read.
CHECK_STAGES := repository-invariants spec-append-only spec-prose-integrity \
	conformance compiler research-tests conformance-run snapshot-run

# Where the stage table is assembled. A gate nobody can profile is a gate that
# silently grows: `check` times each stage and ends with the breakdown, so a
# stage that doubled is visible in the run that doubled it rather than a month
# later. `make -C compiler check` prints its own breakdown the same way.
STAGE_DIR := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-gate-stages

# Keep an unexpected executable stop from leaving a large core artifact in the
# gate workspace. Such a stop fails the adapter; it is never a corpus verdict.
NO_CORE_DUMPS := ulimit -c 0;

check:
	@mkdir -p "$(STAGE_DIR)"
	@: > "$(STAGE_DIR)/summary"
	@for stage in $(CHECK_STAGES); do \
		started=$$(date +%s); \
		$(MAKE) --no-print-directory "$$stage" || exit 1; \
		printf '%-28s %6d s\n' "$$stage" "$$(( $$(date +%s) - started ))" \
			>> "$(STAGE_DIR)/summary"; \
	done
	@echo ""
	@echo "stage wall, this run:"
	@cat "$(STAGE_DIR)/summary"
	@echo "== WHITEFOOT ALL TESTS GREEN =="

# Both supported agent entry points carry exactly the same project rules.
repository-invariants:
	@test -s AGENTS.md -a -s CLAUDE.md || { echo "AGENTS.md or CLAUDE.md missing" >&2; exit 1; }
	@cmp -s AGENTS.md CLAUDE.md || { echo "AGENTS.md and CLAUDE.md differ" >&2; exit 1; }
	@mac_home="$$(printf '/%s/' Users)"; \
	linux_home="$$(printf '/%s/' home)"; \
	encoded_home="$$(printf -- '-%s-' Users)"; \
	windows_home="$$(printf '\\%s\\' Users)"; \
	matches="$$(git grep -a -l -F -e "$$mac_home" -e "$$linux_home" -e "$$encoded_home" -e "$$windows_home" -- . || { status=$$?; test "$$status" -eq 1 || exit "$$status"; })" || exit 1; \
	name_matches="$$(git ls-files | grep -F -e "$$mac_home" -e "$$linux_home" -e "$$encoded_home" -e "$$windows_home" || { status=$$?; test "$$status" -eq 1 || exit "$$status"; })" || exit 1; \
	if test -n "$$matches$$name_matches"; then \
		echo "repository invariants: tracked content or filenames contain a personal home path:" >&2; \
		test -z "$$matches" || echo "$$matches" >&2; \
		test -z "$$name_matches" || echo "$$name_matches" >&2; \
		exit 1; \
	fi

# Released version archives are never edited. Comparing with main makes this a
# property of the exact merge candidate rather than a hook or human process.
spec-append-only:
	@git rev-parse --verify --quiet refs/heads/main >/dev/null || { echo "spec append-only: local main ref is required" >&2; exit 1; }
	@changes="$$(git diff --name-status --diff-filter=MDRCT main -- 'spec/kernel-spec-v*.md')" || exit 1; \
	if test -n "$$changes"; then \
		echo "spec append-only violation: released specifications changed:" >&2; \
		echo "$$changes" >&2; \
		exit 1; \
	fi
	@echo "spec append-only: no released kernel specification was modified or removed"

spec-append-only-staged:
	@changes="$$(git diff --cached --name-status --diff-filter=MDRCT -- 'spec/kernel-spec-v*.md')" || exit 1; \
	if test -n "$$changes"; then \
		echo "spec append-only violation: released specifications changed:" >&2; \
		echo "$$changes" >&2; \
		exit 1; \
	fi
	@echo "spec append-only: no released kernel specification was modified or removed"

# The specification's own bytes are its identity, and the generated
# compiler/src/spec_identity.rs names them, machine-checked against those
# bytes. Live prose quotes neither: a quoted digest or an "active vN" sentence
# went stale at every activation and forced a six-file edit to keep in step
# (found landed: the derivation ledger still described v0.28 as the installed
# authority after the v0.29 activation; retired 2026-09-04 in favour of this
# negative check). Frozen history — archive/done/, research records, archived
# specifications, the approval record, and the derivation ledger's per-version
# amendment bindings — legitimately quotes superseded identities; only the
# ledger's "active authority" sentence is live prose, so the ledger is held to
# the phrase check alone.
spec-prose-integrity:
	@failed=0; \
	for file in README.md AGENTS.md CLAUDE.md compiler/README.md docs/*.md; do \
		if grep -nE '(^|[^0-9a-f])[0-9a-f]{64}([^0-9a-f]|$$)' "$$file"; then \
			echo "spec prose integrity: $$file quotes a specification digest; the identity lives in compiler/src/spec_identity.rs" >&2; failed=1; \
		fi; \
	done; \
	for file in README.md AGENTS.md CLAUDE.md compiler/README.md docs/*.md spec/derivation/derivation-ledger.md; do \
		if grep -nE 'Kernel specification v[0-9]+\.[0-9]+ is the active|[Aa]ctive language authority(:| is) v[0-9]+\.[0-9]+|active v[0-9]+\.[0-9]+ (guidance|authority)|the exact v[0-9]+\.[0-9]+ bytes' "$$file"; then \
			echo "spec prose integrity: $$file names a version as the active authority; say 'the active specification at spec/kernel-spec.md' instead" >&2; failed=1; \
		fi; \
	done; \
	test "$$failed" -eq 0 || exit 1; \
	echo "spec prose integrity: live prose quotes no specification digest and names no version as the active authority"

conformance:
	cd tests/conformance && $(PY) test_runner.py
	$(PY) tests/conformance/runner.py coverage

compiler:
	$(MAKE) -C compiler check

# Maintained executable tests under research/. Self-described deferred archive
# prototypes that require a removed historical compiler are evidence artifacts,
# not current tests; their directory README states that boundary explicitly.
research-tests:
	@mkdir -p "$(RESEARCH_TEST_TMP)/frequency" "$(RESEARCH_TEST_TMP)/ripgrep" "$(RESEARCH_CARGO_TARGET)"
	TMPDIR="$(RESEARCH_TEST_TMP)/frequency" $(MAKE) -C research/experiments/frequency-study check PYTHON=python3 CARGO_TARGET_DIR="$(RESEARCH_CARGO_TARGET)/frequency"
	$(MAKE) -C research/experiments/ripgrep test PYTHON=python3 SCRATCH_ROOT="$(RESEARCH_TEST_TMP)/ripgrep"
	cd research/experiments/default-floor && TMPDIR="$(RESEARCH_TEST_TMP)" $(PY) -m unittest discover -s tests -p 'test_*.py' -v
	cd research/experiments/raw-deflate-default-shape && TMPDIR="$(RESEARCH_TEST_TMP)" $(PY) test_oracle.py
	TMPDIR="$(RESEARCH_TEST_TMP)" CARGO_TARGET_DIR="$(RESEARCH_CARGO_TARGET)/utf8-baseline" cargo test --locked --offline --manifest-path research/experiments/default-floor/utf8parse/rust-baseline/Cargo.toml
	TMPDIR="$(RESEARCH_TEST_TMP)" CARGO_TARGET_DIR="$(RESEARCH_CARGO_TARGET)/utf8-harness" cargo test --locked --offline --manifest-path research/experiments/default-floor/utf8parse/harness/Cargo.toml
	TMPDIR="$(RESEARCH_TEST_TMP)" CARGO_TARGET_DIR="$(RESEARCH_CARGO_TARGET)/percent-baseline" cargo test --locked --offline --manifest-path research/experiments/default-floor/percent-decode/rust-baseline/Cargo.toml
	TMPDIR="$(RESEARCH_TEST_TMP)" CARGO_TARGET_DIR="$(RESEARCH_CARGO_TARGET)/percent-harness" cargo test --locked --offline --manifest-path research/experiments/default-floor/percent-decode/harness/Cargo.toml

# Enumerate every declared case through the native adapter. Every non-pending
# case reaches an actual compiler verdict; run cases are linked and
# executed, while the declared pending case is reported as Skip. `check`
# depends on this target.
# `NO_CORE_DUMPS` only limits harness artifacts if an executable stops
# unexpectedly; the corpus contains no abnormal-termination expectation.
#
# `--profile gate` for the reason `compiler/Cargo.toml` states: this adapter
# runs the whole compiler over five hundred cases, which is exactly the
# compute-bound front-end analysis the gate profile exists for, and it kept
# every debug assertion and overflow check. Left at the default profile it was
# both a second unoptimized build of the crate and an unoptimized run of it.
conformance-run:
	$(NO_CORE_DUMPS) cd compiler && cargo test --profile gate --test conformance --locked --offline -- --ignored --nocapture

# Recompile every program in `tests/snapshot` and compare the accept/reject
# verdict each row records. Compile only: no link, no execution. The corpus is
# a snapshot of this compiler and carries no specification authority, which is
# why it is a stage of its own rather than part of `conformance-run`; see
# `tests/snapshot/README.md`. `--profile gate` for the same reason that target
# gives: this is compute-bound front-end analysis over hundreds of programs.
snapshot-run:
	cd compiler && cargo test --profile gate --test snapshot --locked --offline -- --ignored --nocapture

# one-time: point git at the tracked hooks (pre-commit and pre-merge-commit)
install-hooks:
	git config core.hooksPath governance/hooks
	@echo "installed governance/hooks (pre-commit, pre-merge-commit)"

.PHONY: check repository-invariants spec-append-only spec-append-only-staged spec-prose-integrity conformance compiler research-tests conformance-run snapshot-run install-hooks
