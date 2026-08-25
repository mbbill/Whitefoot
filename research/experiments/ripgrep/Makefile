PYTHON ?= python3
WORK_ROOT ?=
RUN_ID ?=
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
SCRATCH_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-rg-base/work

.PHONY: test gate select baseline require-work-root require-run-id

test:
	PYTHONDONTWRITEBYTECODE=1 TMPDIR="$(SCRATCH_ROOT)" \
		$(PYTHON) -m unittest -v test_runner.py

require-work-root:
	@test -n "$(WORK_ROOT)" || (echo "WORK_ROOT is required" >&2; exit 2)

require-run-id:
	@test -n "$(RUN_ID)" || (echo "RUN_ID is required" >&2; exit 2)

gate: require-work-root test
	PYTHONDONTWRITEBYTECODE=1 TMPDIR="$(WORK_ROOT)/work" \
		$(PYTHON) runner.py verify --work-root "$(WORK_ROOT)"

select: require-work-root require-run-id test
	PYTHONDONTWRITEBYTECODE=1 TMPDIR="$(WORK_ROOT)/work" \
		$(PYTHON) runner.py select \
		--work-root "$(WORK_ROOT)" --run-id "$(RUN_ID)"

baseline: require-work-root require-run-id test
	PYTHONDONTWRITEBYTECODE=1 TMPDIR="$(WORK_ROOT)/work" \
		$(PYTHON) runner.py baseline \
		--work-root "$(WORK_ROOT)" --run-id "$(RUN_ID)"
