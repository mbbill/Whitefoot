# whitefoot verification stack — `make check` runs every layer.
PY=python3 -B
check: project-state spec-guard spec rules soundness perf parity conformance bootstrap
	@echo "== ALL VERIFICATION LAYERS GREEN =="
project-state:             # repository structure: one plan, coherent design package
	$(PY) tools/test_verify_project_state.py
	$(PY) tools/verify_project_state.py
spec-guard:                # owner-gated surfaces: kernel spec, conformance verdicts, oracle digests, reference tests
	$(PY) tools/spec_guard.py --check
	$(PY) tools/test_spec_guard.py
approve-spec:              # regenerate the guard baseline + log an owner approval; approved changes ONLY
	$(PY) tools/spec_guard.py --approve --reason "$(REASON)"
spec:                      # layer 1: spec integrity (META rules, ledger coverage)
	$(PY) tools/spec_ci.py
rules:                     # layer 2: rule-keyed checker + stage-0 codegen correctness
	cd prototype/checker && $(PY) test_checker.py -v 2>&1 | tail -2
	cd prototype/democ && $(PY) test_codegen.py
	cd prototype/democ && $(PY) test_entry_allocas.py
soundness:                 # layer 3: generative model check vs independent oracle
	cd prototype/checker && $(PY) modelcheck.py 10000
perf:                      # layer 4: pinned optimizer-fact effects
	cd prototype/democ && $(PY) perf_regress.py
	$(PY) experiments/port-study/base64/verify.py
parity:                    # layer 5: whitefoot/facts-off/C/Rust codegen properties + visible debt
	$(PY) tools/test_checked_automation.py
	$(PY) tools/codegen_parity.py --corpus --promotion
corpus:                    # focused proof/codegen corpus; positive + adversarial gates
	$(PY) tools/test_checked_automation.py
	$(PY) tools/codegen_parity.py --corpus --tag bounds
conformance:               # layer 6: spec-anchored rule-keyed conformance suite (source -> verdict)
	$(PY) conformance/runner.py all
bootstrap:                 # layer 7: permanent wfc components compiled by disposable stage 0
	$(MAKE) -C compiler check
examples:                  # smoke: compile & run the demo programs
	cd prototype/democ && $(PY) democ.py examples/ex1.wf --run && $(PY) democ.py examples/ex2.wf --run
.PHONY: check project-state spec-guard approve-spec spec rules soundness perf parity corpus conformance bootstrap examples
