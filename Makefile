# Whitefoot's canonical all-tests entry point: compiler checks and tests, the
# complete native conformance adapter, conformance structure and coverage, and
# specification/archive identity. The adapter prints its current tally rather
# than baking a count into this file.

PY := python3 -B
RESEARCH_TEST_TMP := /Users/bytedance/do_not_scan/whitefoot-research-tests-tmp
RESEARCH_CARGO_TARGET := /Users/bytedance/do_not_scan/whitefoot-research-tests-target

check: repository-invariants approval-history-integrity spec-append-only spec-archive-integrity spec-digest-sync conformance compiler research-tests conformance-run
	@echo "== WHITEFOOT ALL TESTS GREEN =="

# Both supported agent entry points carry exactly the same project rules.
repository-invariants:
	@test -s AGENTS.md -a -s CLAUDE.md || { echo "AGENTS.md or CLAUDE.md missing" >&2; exit 1; }
	@cmp -s AGENTS.md CLAUDE.md || { echo "AGENTS.md and CLAUDE.md differ" >&2; exit 1; }

# The prose before the first dated entry may explain the current four rules,
# but every historical approval entry already on main is immutable. New rule-4
# records append after that prefix. Comparing with main makes this a property of
# the exact merge candidate rather than a hook or human-process dependency.
approval-history-integrity:
	@git rev-parse --verify --quiet refs/heads/main >/dev/null || { echo "approval history integrity: local main ref is required" >&2; exit 1; }
	@audit_dir="$$(mktemp -d /Users/bytedance/do_not_scan/whitefoot-approval-history.XXXXXX)" || exit 1; \
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
# Optional branch-candidate mode: a stable file whose status line declares
# `Status: CANDIDATE vM supersedes vN <sha256-of-vN>` is a work-branch
# candidate. `make spec-candidate-integrity` passes exactly when vN is the one recorded version without an
# archive, the declared supersedes digest is vN's recorded digest, the title
# token equals the declared vM, and vM is vN's successor. Its own digest is
# deliberately unchecked because it is not an activated identity. Canonical
# `make check` calls `spec-archive-integrity`, which rejects CANDIDATE status:
# a merge-ready revision must archive the outgoing active bytes and install an
# ACTIVE identity in the chain. This is artifact integrity, not another
# approval or workflow stage.
spec-archive-integrity spec-candidate-integrity:
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
		candidate="$$(awk '/^Status: /{ if ($$2 == "CANDIDATE" && $$4 == "supersedes" && NF >= 6) print $$3, $$5, $$6; exit }' "$$stable_file")"; \
		if test -n "$$candidate"; then \
			if test "$@" != "spec-candidate-integrity"; then \
				echo "spec archive integrity: CANDIDATE status is valid branch work but is not a merge-ready ACTIVE identity" >&2; \
				exit 1; \
			fi; \
			set -- $$candidate; cand_version="$$1"; prev_version="$$2"; prev_digest="$$3"; \
			if test "$$prev_version" != "$$missing_version"; then \
				echo "spec archive integrity: $$stable_file declares a candidate superseding $$prev_version, but $$missing_version is the unarchived recorded version" >&2; \
				exit 1; \
			fi; \
			if test "$$prev_digest" != "$$missing_digest"; then \
				echo "spec archive integrity: $$stable_file candidate supersedes digest $$prev_digest, but the record names $$missing_digest for $$missing_version" >&2; \
				exit 1; \
			fi; \
			if test "$$stable_version" != "$$cand_version"; then \
				echo "spec archive integrity: $$stable_file is titled $$stable_version but declares candidate $$cand_version" >&2; \
				exit 1; \
			fi; \
			minor="$${missing_version##*.}"; expected="$${missing_version%%.*}.$$((minor + 1))"; \
			if test "$$cand_version" != "$$expected"; then \
				echo "spec archive integrity: candidate $$cand_version does not succeed $$missing_version (expected $$expected)" >&2; \
				exit 1; \
			fi; \
			echo "spec archive integrity: $$stable_file is a declared candidate $$cand_version superseding the recorded $$missing_version"; \
		else \
			if test "$$stable_version" != "$$missing_version"; then \
				echo "spec archive integrity: $$stable_file names $$stable_version, but $$missing_version is the unarchived recorded version" >&2; \
				exit 1; \
			fi; \
			actual="$$(shasum -a 256 "$$stable_file" | cut -d' ' -f1)"; \
			if test "$$actual" != "$$missing_digest"; then \
				echo "spec archive integrity: $$stable_file hashes to $$actual, recorded as $$missing_digest" >&2; \
				exit 1; \
			fi; \
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

# The activation chain tail in governance/APPROVALS.md is the sole authority
# for the active specification's identity, but a handful of live prose sites
# quote that identity for readers. This target checks those quotes, so an
# activation cannot leave prose naming the superseded version (found landed:
# the derivation ledger still described v0.28 as the installed authority after
# the v0.29 activation). Checking only — it never rewrites prose. Green means
# exactly that the anchored claims below name the chain tail; it does not mean
# a listed file has no other stale sentence, and it says nothing about files
# outside the list. Frozen history (docs/done/, research records, archived
# specs) legitimately quotes superseded identities and is deliberately not
# listed.
spec-digest-sync:
	@tail="$$(awk '/^ACTIVE-SPEC: /{version=$$2; digest=$$3} END{if (version == "") exit 1; print version, digest}' governance/APPROVALS.md)" || { echo "spec digest sync: governance/APPROVALS.md has no activation chain" >&2; exit 1; }; \
	set -- $$tail; version="$$1"; digest="$$2"; failed=0; \
	for file in README.md compiler/README.md docs/roadmap.md docs/current-plan.md spec/derivation/derivation-ledger.md; do \
		grep -qF "$$digest" "$$file" || { echo "spec digest sync: $$file does not quote the active digest ($$version $$digest)" >&2; failed=1; }; \
	done; \
	grep -qF "Kernel specification $$version" README.md || { echo "spec digest sync: README.md does not name $$version as the kernel specification" >&2; failed=1; }; \
	grep -qF "the exact $$version bytes" compiler/README.md || { echo "spec digest sync: compiler/README.md does not target the exact $$version bytes" >&2; failed=1; }; \
	grep -qF "The active language authority is $$version" docs/roadmap.md || { echo "spec digest sync: docs/roadmap.md does not name $$version as the active authority" >&2; failed=1; }; \
	grep -qF "Active language authority: $$version" docs/current-plan.md || { echo "spec digest sync: docs/current-plan.md does not name $$version as the active authority" >&2; failed=1; }; \
	grep -qF "active $$version guidance" docs/patterns.md || { echo "spec digest sync: docs/patterns.md does not carry active $$version guidance" >&2; failed=1; }; \
	grep -qF "the active $$version authority" spec/derivation/derivation-ledger.md || { echo "spec digest sync: spec/derivation/derivation-ledger.md does not name $$version as the active authority" >&2; failed=1; }; \
	test "$$failed" -eq 0 || exit 1; \
	echo "spec digest sync: live prose quotes the chain tail ($$version $$digest)"

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
# case reaches an actual compiler verdict; run/trap cases are linked and
# executed, while the declared pending case is reported as Skip. `check`
# depends on this target.
conformance-run:
	cd compiler && cargo test --test conformance --locked --offline -- --ignored --nocapture

# one-time: point git at the tracked hooks (pre-commit and pre-merge-commit)
install-hooks:
	git config core.hooksPath governance/hooks
	@echo "installed governance/hooks (pre-commit, pre-merge-commit)"

.PHONY: check repository-invariants approval-history-integrity spec-append-only spec-append-only-staged spec-archive-integrity spec-candidate-integrity spec-digest-sync conformance compiler research-tests conformance-run install-hooks
