# Whitefoot gate — only what a research compiler needs: the compiler builds and
# passes its tests, the conformance corpus has valid structure and declared rule
# coverage, and the active specification plus immutable archives match the
# recorded chain. Everything else is guarded by AGENTS.md/CLAUDE.md. A green
# gate states only what it exercises: `check` exercises corpus structure and
# declared coverage, not every case's verdict. Verdicts are `conformance-run`,
# which is reported independently and is not yet green.

PY := python3 -B

check: repository-invariants spec-append-only spec-archive-integrity conformance compiler
	@echo "== WHITEFOOT GATE GREEN (active compiler + independent evidence) =="

# repository invariants: identical agent instructions and the canonical outline marker
repository-invariants:
	@cmp -s AGENTS.md CLAUDE.md || { echo "AGENTS.md and CLAUDE.md differ" >&2; exit 1; }
	@grep -q '^Status: CANONICAL DIRECTION OUTLINE' docs/roadmap.md || { echo "docs/roadmap.md is not marked canonical" >&2; exit 1; }

# Released version archives are never edited; the stable active file is checked
# against the recorded chain by `spec-archive-integrity` below.
spec-append-only:
	@changes="$$(git diff --name-status --diff-filter=MDRCT HEAD -- 'spec/kernel-spec-v*.md')"; \
	if test -n "$$changes"; then \
		echo "spec append-only violation: released specifications changed:" >&2; \
		echo "$$changes" >&2; \
		exit 1; \
	fi
	@echo "spec append-only: no released kernel specification was modified or removed"

spec-append-only-staged:
	@changes="$$(git diff --cached --name-status --diff-filter=MDRCT -- 'spec/kernel-spec-v*.md')"; \
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
spec-archive-integrity:
	@records="$$(awk 'function fail(message) { print "spec archive integrity: approval record line " NR ": " message > "/dev/stderr"; exit 1 } function is_version(value) { return value ~ /^v[0-9]+\.[0-9]+$$/ } function is_digest(value) { return length(value) == 64 && value ~ /^[0-9a-f]+$$/ } /^ACTIVE-SPEC:/ { if (NF != 4) fail("ACTIVE-SPEC record must have four fields"); if (!is_version($$2)) fail("invalid version " $$2); if (!is_digest($$3)) fail("invalid digest for " $$2); if ($$4 != "-" && !is_digest($$4)) fail("invalid previous digest for " $$2); if (seen[$$2]++) fail($$2 " has more than one recorded identity"); print $$2, $$3; next } /^ARCHIVE-SPEC:/ { if (NF != 3) fail("ARCHIVE-SPEC record must have three fields"); if (!is_version($$2)) fail("invalid version " $$2); if (!is_digest($$3)) fail("invalid digest for " $$2); if (seen[$$2]++) fail($$2 " has more than one recorded identity"); print $$2, $$3 }' governance/APPROVALS.md)" || exit 1; \
	set -- $$records; \
	if test $$# -eq 0; then \
		echo "spec archive integrity: approval record contains no specification identities" >&2; \
		exit 1; \
	fi; \
	recorded=0; missing=0; missing_version=""; missing_digest=""; versions=""; \
	stable_file="spec/kernel-spec.md"; \
	while test $$# -ge 2; do \
		version="$$1"; digest="$$2"; shift 2; \
		case " $$versions " in \
			*" $$version "*) echo "spec archive integrity: $$version has more than one recorded identity" >&2; exit 1;; \
		esac; \
		versions="$$versions $$version"; \
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
			missing_version="$$version"; missing_digest="$$digest"; \
		fi; \
		recorded=$$((recorded + 1)); \
	done; \
	if test -e "$$stable_file" || test -L "$$stable_file"; then \
		if test ! -f "$$stable_file" || test -L "$$stable_file"; then \
			echo "spec archive integrity: $$stable_file is not a regular active specification file" >&2; \
			exit 1; \
		fi; \
		if test "$$missing" -ne 1; then \
			echo "spec archive integrity: stable-file layout requires exactly one recorded version without an archive, found $$missing" >&2; \
			exit 1; \
		fi; \
		stable_version="$$(awk 'NR == 1 && $$0 ~ /^# Kernel Specification v[0-9]+\.[0-9]+$$/ { print $$4 }' "$$stable_file")"; \
		if test -z "$$stable_version"; then \
			echo "spec archive integrity: $$stable_file has no exact first-line version token" >&2; \
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

conformance:
	cd tests/conformance && $(PY) test_runner.py
	$(PY) tests/conformance/runner.py coverage

compiler:
	$(MAKE) -C compiler check

# Drive every case through the native adapter: compile, arrange, run, compare.
# This remains separate from `check`: the current result is
# Pass=423, Fail=1, Skip=13, and the one runnable attribution divergence is
# tracked independently rather than hidden inside the repository gate.
conformance-run:
	cd compiler && cargo test --test conformance --locked --offline -- --ignored --nocapture

# one-time: point git at the tracked hooks (pre-commit and pre-merge-commit)
install-hooks:
	git config core.hooksPath governance/hooks
	@echo "installed governance/hooks (pre-commit, pre-merge-commit)"

.PHONY: check repository-invariants spec-append-only spec-append-only-staged spec-archive-integrity conformance compiler conformance-run install-hooks
