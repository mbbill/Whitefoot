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
CHECK_STAGES := repository-invariants approval-history-integrity spec-append-only \
	spec-archive-integrity spec-prose-integrity conformance compiler research-tests \
	conformance-run snapshot-run

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

# The prose before the first dated entry may explain the current four rules,
# but every historical approval entry already on main is immutable. New rule-4
# records append after that prefix. Comparing with main makes this a property of
# the exact merge candidate rather than a hook or human-process dependency.
approval-history-integrity:
	@git rev-parse --verify --quiet refs/heads/main >/dev/null || { echo "approval history integrity: local main ref is required" >&2; exit 1; }
	@mkdir -p "$(WHITEFOOT_SCRATCH_ROOT)"
	@audit_dir="$$(mktemp -d "$(WHITEFOOT_SCRATCH_ROOT)/whitefoot-approval-history.XXXXXX")" || exit 1; \
	trap 'rm -rf "$$audit_dir"' EXIT HUP INT TERM; \
	git show main:governance/APPROVALS.md > "$$audit_dir/main-source" || exit 1; \
	awk 'started || /^## / { started=1; print }' "$$audit_dir/main-source" > "$$audit_dir/main" || exit 1; \
	awk 'started || /^## / { started=1; print }' governance/APPROVALS.md > "$$audit_dir/current" || exit 1; \
	main_bytes="$$(wc -c < "$$audit_dir/main" | tr -d ' ')"; \
	current_bytes="$$(wc -c < "$$audit_dir/current" | tr -d ' ')"; \
	if test "$$current_bytes" -lt "$$main_bytes"; then \
		echo "approval history integrity: the historical record was shortened" >&2; \
		exit 1; \
	fi; \
	dd if="$$audit_dir/current" of="$$audit_dir/prefix" bs=1 count="$$main_bytes" 2>/dev/null; \
	cmp -s "$$audit_dir/main" "$$audit_dir/prefix" || { \
		echo "approval history integrity: an existing main record was changed" >&2; \
		exit 1; \
	}; \
	if test "$$current_bytes" -gt "$$main_bytes"; then \
		dd if="$$audit_dir/current" of="$$audit_dir/suffix" bs=1 skip="$$main_bytes" 2>/dev/null; \
		first="$$(awk 'NF { print; exit }' "$$audit_dir/suffix")"; \
		case "$$first" in \
			'## '*) ;; \
			*) echo "approval history integrity: appended content must begin a new dated record" >&2; exit 1;; \
		esac; \
	fi; \
	echo "approval history integrity: existing main records are an exact prefix"

# Released version archives are never edited; the stable active file is checked
# against the recorded chain by `spec-archive-integrity` below.
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

# landed state, not staged diff: every recorded specification identity still
# names bytes that hash to it, and every released specification has a record.
# Before the stable-file switchover every recorded version is archived at its
# versioned path. Afterwards exactly one recorded version lives at the stable
# path, and the stable file's own version token selects that identity.
# `spec-append-only` above only sees what one commit touches, and `pre-commit`
# is bypassable by `--no-verify`, by merge commits, and by a clone whose
# `core.hooksPath` points elsewhere, so this is the guard that actually holds.
# `shasum` is used deliberately: it is the tool the digests were recorded with,
# and it shares no code with the compiler's own SHA-256.
#
# The stable file always declares `Status: ACTIVE vN` and hashes to the chain
# tail: an amendment lands in one change with the amended file, the archive of
# the outgoing bytes, and the appended chain line, and this target is what
# says that change is complete. There is no separate candidate state (retired
# 2026-09-04: it produced zero-content activation merges and a red main after
# every candidate merge). This is artifact integrity, not another approval or
# workflow stage.
spec-archive-integrity:
	@records="$$(awk 'function fail(message) { print "spec archive integrity: identity record line " NR ": " message > "/dev/stderr"; exit 1 } function is_version(value) { return value ~ /^v[0-9]+\.[0-9]+$$/ } function is_digest(value) { return length(value) == 64 && value ~ /^[0-9a-f]+$$/ } /^ACTIVE-SPEC:/ { if (NF != 4) fail("ACTIVE-SPEC record must have four fields"); if (!is_version($$2)) fail("invalid version " $$2); if (!is_digest($$3)) fail("invalid digest for " $$2); if ($$4 != "-" && !is_digest($$4)) fail("invalid previous digest for " $$2); if (seen[$$2]++) fail($$2 " has more than one recorded identity"); print "ACTIVE", $$2, $$3, $$4; next } /^ARCHIVE-SPEC:/ { if (NF != 3) fail("ARCHIVE-SPEC record must have three fields"); if (!is_version($$2)) fail("invalid version " $$2); if (!is_digest($$3)) fail("invalid digest for " $$2); if (seen[$$2]++) fail($$2 " has more than one recorded identity"); print "ARCHIVE", $$2, $$3, "-" }' governance/APPROVALS.md)" || exit 1; \
	set -- $$records; \
	if test $$# -eq 0; then \
		echo "spec archive integrity: identity record contains no specification identities" >&2; \
		exit 1; \
	fi; \
	recorded=0; missing=0; missing_kind=""; missing_version=""; missing_digest=""; versions=""; \
	active_count=0; last_active_version=""; last_active_digest=""; \
	stable_file="spec/kernel-spec.md"; \
	while test $$# -ge 4; do \
		kind="$$1"; version="$$2"; digest="$$3"; previous="$$4"; shift 4; \
		case " $$versions " in \
			*" $$version "*) echo "spec archive integrity: $$version has more than one recorded identity" >&2; exit 1;; \
		esac; \
		versions="$$versions $$version"; \
		if test "$$kind" = "ACTIVE"; then \
			if test "$$active_count" -eq 0; then \
				test "$$previous" = "-" || { echo "spec archive integrity: first ACTIVE record $$version must have '-' predecessor" >&2; exit 1; }; \
			elif test "$$previous" != "$$last_active_digest"; then \
				echo "spec archive integrity: ACTIVE record $$version supersedes $$previous, but the preceding ACTIVE digest is $$last_active_digest" >&2; \
				exit 1; \
			fi; \
			active_count=$$((active_count + 1)); last_active_version="$$version"; last_active_digest="$$digest"; \
		elif test "$$kind" != "ARCHIVE"; then \
			echo "spec archive integrity: unknown identity record kind $$kind" >&2; exit 1; \
		fi; \
		file="spec/kernel-spec-$$version.md"; \
		if test -e "$$file" || test -L "$$file"; then \
			if test ! -f "$$file" || test -L "$$file"; then \
				echo "spec archive integrity: $$file is not a regular archive file" >&2; \
				exit 1; \
			fi; \
			actual="$$(shasum -a 256 "$$file" | cut -d' ' -f1)"; \
			if test "$$actual" != "$$digest"; then \
				echo "spec archive integrity: $$file hashes to $$actual, recorded as $$digest" >&2; \
				exit 1; \
			fi; \
		else \
			missing=$$((missing + 1)); \
			missing_kind="$$kind"; missing_version="$$version"; missing_digest="$$digest"; \
		fi; \
		recorded=$$((recorded + 1)); \
	done; \
	test $$# -eq 0 || { echo "spec archive integrity: malformed identity-record token stream" >&2; exit 1; }; \
	test "$$active_count" -gt 0 || { echo "spec archive integrity: identity record contains no ACTIVE chain" >&2; exit 1; }; \
	if test -e "$$stable_file" || test -L "$$stable_file"; then \
		if test ! -f "$$stable_file" || test -L "$$stable_file"; then \
			echo "spec archive integrity: $$stable_file is not a regular active specification file" >&2; \
			exit 1; \
		fi; \
		if test "$$missing" -ne 1; then \
			echo "spec archive integrity: stable-file layout requires exactly one recorded version without an archive, found $$missing" >&2; \
			exit 1; \
		fi; \
		if test "$$missing_kind" != "ACTIVE" || test "$$missing_version" != "$$last_active_version" || test "$$missing_digest" != "$$last_active_digest"; then \
			echo "spec archive integrity: the unarchived identity must be the ACTIVE chain tail $$last_active_version" >&2; \
			exit 1; \
		fi; \
		stable_version="$$(awk 'NR == 1 && $$0 ~ /^# Kernel Specification v[0-9]+\.[0-9]+$$/ { print $$4 }' "$$stable_file")"; \
		if test -z "$$stable_version"; then \
			echo "spec archive integrity: $$stable_file has no exact first-line version token" >&2; \
			exit 1; \
		fi; \
		status="$$(awk '/^Status: /{ print $$2, $$3; exit }' "$$stable_file")"; \
		if test "$$status" != "ACTIVE $$stable_version"; then \
			echo "spec archive integrity: $$stable_file must declare 'Status: ACTIVE $$stable_version' (found '$$status')" >&2; \
			exit 1; \
		fi; \
		if test "$$stable_version" != "$$missing_version"; then \
			echo "spec archive integrity: $$stable_file names $$stable_version, but $$missing_version is the unarchived recorded version" >&2; \
			exit 1; \
		fi; \
		actual="$$(shasum -a 256 "$$stable_file" | cut -d' ' -f1)"; \
		if test "$$actual" != "$$missing_digest"; then \
			echo "spec archive integrity: $$stable_file hashes to $$actual, recorded as $$missing_digest" >&2; \
			exit 1; \
		fi; \
	elif test "$$missing" -ne 0; then \
		echo "spec archive integrity: $$missing_version is recorded but spec/kernel-spec-$$missing_version.md is missing and $$stable_file is absent" >&2; \
		exit 1; \
	fi; \
	for file in spec/kernel-spec-v*.md; do \
		if test ! -f "$$file" || test -L "$$file"; then \
			echo "spec archive integrity: $$file is not a regular archive file" >&2; \
			exit 1; \
		fi; \
		version="$${file#spec/kernel-spec-}"; version="$${version%.md}"; \
		case " $$versions " in \
			*" $$version "*) ;; \
			*) echo "spec archive integrity: $$file has no recorded identity" >&2; exit 1;; \
		esac; \
	done; \
	echo "spec archive integrity: $$recorded recorded specifications hash as recorded"

# The activation chain tail in governance/APPROVALS.md and the generated
# compiler/src/spec_identity.rs are the two places that name the active
# specification's version and digest, and both are machine-checked against the
# bytes. Live prose quotes neither: a quoted digest or an "active vN" sentence
# went stale at every activation and forced a six-file edit to keep in step
# (found landed: the derivation ledger still described v0.28 as the installed
# authority after the v0.29 activation; retired 2026-09-04 in favour of this
# negative check). Frozen history — docs/done/, research records, archived
# specifications, the approval record, and the derivation ledger's per-version
# amendment bindings — legitimately quotes superseded identities; only the
# ledger's "active authority" sentence is live prose, so the ledger is held to
# the phrase check alone.
spec-prose-integrity:
	@failed=0; \
	for file in README.md AGENTS.md CLAUDE.md compiler/README.md docs/*.md; do \
		if grep -nE '(^|[^0-9a-f])[0-9a-f]{64}([^0-9a-f]|$$)' "$$file"; then \
			echo "spec prose integrity: $$file quotes a specification digest; the identity lives in governance/APPROVALS.md and compiler/src/spec_identity.rs" >&2; failed=1; \
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

.PHONY: check repository-invariants approval-history-integrity spec-append-only spec-append-only-staged spec-archive-integrity spec-prose-integrity conformance compiler research-tests conformance-run snapshot-run install-hooks
