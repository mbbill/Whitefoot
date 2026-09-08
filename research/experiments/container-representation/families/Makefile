SHELL := /bin/sh
ROOT := $(abspath ../../../..)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
RUSTC ?= rustc
CLANG ?= clang
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
BUILD := .build
.DELETE_ON_ERROR:

.PHONY: check measure reproduce-runtime-gap reproduce-region-gap clean

$(BUILD):
	mkdir -p $@

$(BUILD)/abi$(EXE): abi.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/priority.raw.ll: priority.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --no-overlap --emit-llvm $< -o $@

$(BUILD)/priority.ll: $(BUILD)/priority.raw.ll $(BUILD)/abi$(EXE)
	$(BUILD)/abi$(EXE) $< $@ wf_priority_trace

$(BUILD)/priority-costs$(EXE): priority-costs.c $(BUILD)/priority.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $^ -o $@

$(BUILD)/priority-boundary-costs$(EXE): priority-costs.c $(BUILD)/priority.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module -DRETAIN_HELPERS $^ -o $@

$(BUILD)/growth-abi$(EXE): growth-abi.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/growth.raw.ll: growth.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --no-overlap --emit-llvm $< -o $@

$(BUILD)/growth.instrumented.ll: $(BUILD)/growth.raw.ll $(BUILD)/growth-abi$(EXE)
	$(BUILD)/growth-abi$(EXE) $< $@

$(BUILD)/growth.opt.ll: $(BUILD)/growth.instrumented.ll $(BUILD)/growth-abi$(EXE)
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@
	$(BUILD)/growth-abi$(EXE) --verify $@

$(BUILD)/growth-costs$(EXE): growth-costs.c $(BUILD)/growth.opt.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $^ -o $@

check: $(BUILD)/priority-costs$(EXE) $(BUILD)/priority-boundary-costs$(EXE) $(BUILD)/growth-costs$(EXE)
	rustfmt --edition 2024 --check abi.rs growth-abi.rs
	@set -eu; \
	  for source in hashmap ordered priority packed-page growth boxed-migration; do \
	    for mode in default par no-overlap; do \
	      case "$$mode" in default) flags= ;; par) flags=--par ;; no-overlap) flags=--no-overlap ;; esac; \
	      "$(WHITEFOOTC)" $$flags "$$source.wf" -o "$(BUILD)/$$source-$$mode$(EXE)"; \
	      "$(BUILD)/$$source-$$mode$(EXE)"; \
	    done; \
	  done
	@set -eu; \
	  for mode in default par no-overlap; do \
	    case "$$mode" in default) flags= ;; par) flags=--par ;; no-overlap) flags=--no-overlap ;; esac; \
	    "$(WHITEFOOTC)" $$flags ordered-runtime-gap.wf -o "$(BUILD)/ordered-runtime-gap-$$mode$(EXE)"; \
	  done
	@set -eu; \
	  for case_name in rejected-wrapper rejected-shared-option-view; do \
	    case "$$case_name" in rejected-wrapper) rule='[FN-9]'; detail=InvalidPostconditionSelector ;; rejected-shared-option-view) rule='[OWN-6]'; detail=InvalidChildReborrow ;; esac; \
	    status=0; rm -f "$(BUILD)/$$case_name$(EXE)"; \
	    "$(WHITEFOOTC)" "$$case_name.wf" -o "$(BUILD)/$$case_name$(EXE)" > "$(BUILD)/$$case_name.log" 2>&1 || status=$$?; \
	    if test "$$status" -ne 1 || test -e "$(BUILD)/$$case_name$(EXE)" || ! grep -F "$$rule" "$(BUILD)/$$case_name.log" > /dev/null || ! grep -F "$$detail" "$(BUILD)/$$case_name.log" > /dev/null; then \
	      echo "Unexpected outcome: $$case_name (exit $$status)"; cat "$(BUILD)/$$case_name.log"; exit 1; \
	    fi; \
	  done
	$(BUILD)/priority-costs$(EXE) check
	$(BUILD)/priority-boundary-costs$(EXE) check
	$(BUILD)/growth-costs$(EXE) check

measure: check
	$(BUILD)/priority-costs$(EXE) measure > $(BUILD)/priority-measurements.csv
	$(BUILD)/priority-boundary-costs$(EXE) measure > $(BUILD)/priority-boundary-measurements.csv
	$(BUILD)/growth-costs$(EXE) measure > $(BUILD)/growth-measurements.csv

reproduce-runtime-gap: check
	$(BUILD)/ordered-runtime-gap-default$(EXE)

reproduce-region-gap: | $(BUILD)
	$(WHITEFOOTC) boxed-helper-gap.wf -o $(BUILD)/boxed-helper-gap$(EXE)

clean:
	rm -rf $(BUILD)
