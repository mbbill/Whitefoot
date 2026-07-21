# Whitefoot development gate. The active Rust workspace is incomplete; this
# gate validates its current exact capability without making a release claim.

PY=python3 -B

check: project-state spec-guard spec facets catalog-identity capabilities lexical-model reference-model conformance grammar-evidence compiler
	@echo "== DEVELOPMENT GATE GREEN; GRAMMAR EVIDENCE REPRODUCED; NO RELEASE CLAIM =="

project-state:
	$(PY) tools/test_verify_project_state.py
	$(PY) tools/verify_project_state.py

spec-guard:
	$(PY) tools/spec_guard.py --check
	$(PY) tools/test_spec_guard.py

approve-spec:
	$(PY) tools/spec_guard.py --approve --reason "$(REASON)"

spec:
	$(PY) tools/spec_ci.py

facets:
	$(PY) tools/test_facet_catalog.py
	$(PY) tools/facet_catalog.py check
	$(PY) tools/test_semantic_catalog.py
	$(PY) tools/semantic_catalog.py check
	$(PY) tools/test_facet_discrepancies.py
	$(PY) tools/test_v08_terminal_ident_audit.py
	$(PY) tools/facet_discrepancies.py check

catalog-identity:
	$(PY) tools/test_catalog_identity.py
	$(PY) tools/catalog_identity.py check

capabilities:
	$(PY) tools/test_capability_overlay.py
	$(PY) tools/capability_overlay.py check

lexical-model:
	$(PY) tools/test_v08_lexical_model.py
	$(PY) tools/test_v08_lexical_observer.py

reference-model:
	cd prototype/checker && $(PY) test_checker.py -v
	cd prototype/checker && $(PY) modelcheck.py 10000

conformance:
	$(PY) conformance/runner.py coverage

grammar-evidence:
	$(MAKE) -C grammar-verifier check

compiler:
	$(MAKE) -C compiler check

conformance-run:
	$(PY) conformance/runner.py run

release-check:
	@echo "release gate unavailable: the exact-v0.8 Rust compiler is incomplete"
	@false

.PHONY: check project-state spec-guard approve-spec spec facets catalog-identity capabilities lexical-model reference-model conformance grammar-evidence compiler conformance-run release-check
