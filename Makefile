# Whitefoot gate — only what a research compiler needs: the compiler builds and
# passes its tests, the conformance corpus has valid structure and declared rule
# coverage, and the numbered spec stays append-only. Everything else is guarded
# by AGENTS.md/CLAUDE.md. A green gate states only what it exercises; the full
# corpus has no compiler adapter yet.

PY := python3 -B

check: repository-invariants spec-append-only conformance compiler
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

conformance:
	cd tests/conformance && $(PY) test_runner.py
	$(PY) tests/conformance/runner.py coverage

compiler:
	$(MAKE) -C compiler check

conformance-run:
	$(PY) tests/conformance/runner.py run

# one-time: point git at the tracked hooks (spec append-only pre-commit)
install-hooks:
	git config core.hooksPath governance/hooks
	@echo "installed governance/hooks (spec append-only pre-commit)"

.PHONY: check repository-invariants spec-append-only spec-append-only-staged conformance compiler conformance-run install-hooks
