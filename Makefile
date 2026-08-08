# Whitefoot gate — only what a research compiler needs: the compiler builds and
# passes its tests, the conformance corpus has valid structure and declared rule
# coverage, and the numbered spec stays append-only. Everything else is guarded
# by AGENTS.md/CLAUDE.md. A green gate states only what it exercises: `check`
# exercises corpus structure and declared coverage, not every case's verdict.
# Verdicts are `conformance-run`, which is reported and not yet green.

PY := python3 -B

check: repository-invariants spec-append-only spec-archive-integrity conformance compiler
	@echo "== WHITEFOOT GATE GREEN (active compiler + independent evidence) =="

# repository invariants: identical agent instructions and the canonical outline marker
repository-invariants:
	@cmp -s AGENTS.md CLAUDE.md || { echo "AGENTS.md and CLAUDE.md differ" >&2; exit 1; }
	@grep -q '^Status: CANONICAL DIRECTION OUTLINE' docs/roadmap.md || { echo "docs/roadmap.md is not marked canonical" >&2; exit 1; }

# the one spec protection: released kernel specs are never edited (new version only)
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
# `spec-append-only` above only sees what one commit touches, and `pre-commit`
# is bypassable by `--no-verify`, by merge commits, and by a clone whose
# `core.hooksPath` points elsewhere, so this is the guard that actually holds.
# `shasum` is used deliberately: it is the tool the digests were recorded with,
# and it shares no code with the compiler's own SHA-256.
spec-archive-integrity:
	@set -- $$(awk '/^(ACTIVE|ARCHIVE)-SPEC: /{print $$2, $$3}' governance/APPROVALS.md); \
	recorded=0; \
	while test $$# -ge 2; do \
		version="$$1"; digest="$$2"; shift 2; \
		file="spec/kernel-spec-$$version.md"; \
		if test ! -f "$$file"; then \
			echo "spec archive integrity: $$version is recorded but $$file is missing" >&2; \
			exit 1; \
		fi; \
		actual="$$(shasum -a 256 "$$file" | cut -d' ' -f1)"; \
		if test "$$actual" != "$$digest"; then \
			echo "spec archive integrity: $$file hashes to $$actual, recorded as $$digest" >&2; \
			exit 1; \
		fi; \
		recorded=$$((recorded + 1)); \
	done; \
	versions="$$(awk '/^(ACTIVE|ARCHIVE)-SPEC: /{printf " %s", $$2}' governance/APPROVALS.md)"; \
	for file in spec/kernel-spec-v*.md; do \
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

# drive every case through the native adapter: compile, arrange, run, compare.
# not in `check`: 123 pre-existing runnable cases do not reach their declared
# verdict through this compiler, and resolving that needs decisions outside the
# task that built the adapter (docs/ongoing/0014-first-slice-conformance-execution.md).
conformance-run:
	cd compiler && cargo test --test conformance --locked --offline -- --ignored --nocapture

# one-time: point git at the tracked hooks (pre-commit and pre-merge-commit)
install-hooks:
	git config core.hooksPath governance/hooks
	@echo "installed governance/hooks (pre-commit, pre-merge-commit)"

.PHONY: check repository-invariants spec-append-only spec-append-only-staged spec-archive-integrity conformance compiler conformance-run install-hooks
