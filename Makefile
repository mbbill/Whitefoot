# Whitefoot research-compiler gate. It validates the capabilities that exist;
# it does not claim that the language or compiler is complete.

PY=python3 -B

check: project-state spec-guard spec facets catalog-identity lexical-model reference-model conformance grammar-evidence phase5-proposal-evidence compiler
	@echo "== V0.9 RESEARCH COMPILER FRONTEND GREEN; SEMANTICS AND BACKEND ABSENT =="

project-state:
	$(PY) tools/test_verify_project_state.py
	$(PY) tools/verify_project_state.py

spec-guard:
	$(PY) tools/spec_guard.py --check
	$(PY) tools/test_spec_guard.py

approve-spec:
	$(PY) tools/spec_guard.py --approve --reason "$(REASON)"

spec:
	$(PY) tools/test_spec_ci.py
	$(PY) tools/spec_ci.py

facets:
	$(PY) tools/test_facet_catalog.py
	$(PY) tools/facet_catalog.py check
	$(PY) tools/test_semantic_catalog.py
	$(PY) tools/semantic_catalog.py check
	$(PY) tools/test_facet_discrepancies.py
	$(PY) tools/facet_discrepancies.py check

catalog-identity:
	$(PY) tools/test_catalog_identity.py
	$(PY) tools/catalog_identity.py check

lexical-model:
	# The v0.8 model remains executable, but its observer receipt is immutable:
	# the active compiler accepts only exact-v0.9 requests.
	$(PY) tools/test_v08_lexical_model.py
	$(PY) tools/test_v09_lexical_model.py
	$(PY) tools/test_v09_lexical_observer.py

reference-model:
	cd prototype/checker && $(PY) test_checker.py -v
	cd prototype/checker && $(PY) modelcheck.py 10000

conformance:
	cd conformance && $(PY) test_runner.py
	$(PY) conformance/runner.py coverage

grammar-evidence:
	cmp -s spec/kernel-spec-v0.9.md grammar-verifier/proposal/kernel-spec-successor-candidate.md
	$(MAKE) -C grammar-verifier check

phase5-proposal-evidence:
	$(PY) optimizer-language-research/implementation/phase5-successor-proposal/generate_candidate.py --check
	$(PY) optimizer-language-research/implementation/phase5-successor-proposal/test_generate_candidate.py
	$(PY) optimizer-language-research/implementation/phase5-successor-proposal/protected_surface_census.py --check
	$(PY) optimizer-language-research/implementation/phase5-successor-proposal/test_protected_surface_census.py
	$(PY) optimizer-language-research/implementation/phase5-successor-proposal/diagnostic_evidence/run.py
	$(PY) -m unittest discover -s optimizer-language-research/implementation/phase5-successor-proposal/diagnostic_evidence -p 'test_*.py'

compiler:
	$(MAKE) -C compiler check

conformance-run:
	$(PY) conformance/runner.py run

.PHONY: check project-state spec-guard approve-spec spec facets catalog-identity lexical-model reference-model conformance grammar-evidence phase5-proposal-evidence compiler conformance-run
