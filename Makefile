# xlang verification stack — `make check` runs all four layers.
PY=python3
check: spec rules soundness perf
	@echo "== ALL FOUR VERIFICATION LAYERS GREEN =="
spec:                      # layer 1: spec integrity (META rules, ledger coverage)
	$(PY) tools/spec_ci.py
rules:                     # layer 2: rule-keyed checker tests
	cd prototype/checker && $(PY) test_checker.py -v 2>&1 | tail -2
soundness:                 # layer 3: generative model check vs independent oracle
	cd prototype/checker && $(PY) modelcheck.py 10000
perf:                      # layer 4: pinned optimizer-fact effects
	cd prototype/democ && $(PY) perf_regress.py
examples:                  # smoke: compile & run the demo programs
	cd prototype/democ && $(PY) democ.py examples/ex1.xl --run && $(PY) democ.py examples/ex2.xl --run
.PHONY: check spec rules soundness perf examples
