SHELL := /bin/sh
ROOT := $(abspath ../../../..)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
RUSTC ?= rustc
CLANG ?= clang
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
BUILD := .build
.DELETE_ON_ERROR:

.PHONY: check check-interface measure measure-interface reproduce-runtime-gap reproduce-region-gap clean

$(BUILD)/owning-growth-%.raw.ll: owning-growth.wf $(WHITEFOOTC) | $(BUILD)
	@set -eu; case '$*' in default) flags= ;; par) flags=--par ;; no-overlap) flags=--no-overlap ;; *) exit 1 ;; esac; \
	  $(WHITEFOOTC) $$flags --emit-llvm $< -o $@

$(BUILD)/owning-growth-%.ll: $(BUILD)/owning-growth-%.raw.ll $(BUILD)/abi$(EXE)
	$(BUILD)/abi$(EXE) --observe-allocations $< $@.tmp
	awk '/^define internal i64 @wf_map_contract\(ptr %v0, i64 %v1, i64 %v2, i64 %v3\)/ { sub(/internal /, ""); exported++ } { print } END { if (exported != 1) exit 1 }' $@.tmp > $@
	rm $@.tmp

$(BUILD)/owning-growth-observer-%$(EXE): owning-growth-observer.c $(BUILD)/owning-growth-%.ll ../costs/sparse-owned.c
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $< $(BUILD)/owning-growth-$*.ll -o $@

.PHONY: check-owning-growth
check-owning-growth: $(addprefix $(BUILD)/owning-growth-observer-,default$(EXE) par$(EXE) no-overlap$(EXE))
	$(BUILD)/owning-growth-observer-default$(EXE)
	$(BUILD)/owning-growth-observer-par$(EXE)
	$(BUILD)/owning-growth-observer-no-overlap$(EXE)

$(BUILD)/owning-growth-abi$(EXE): owning-growth-abi.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/owning-growth-abi-tests$(EXE): owning-growth-abi.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings --test $< -o $@

$(BUILD)/owning-growth-cost.raw.ll: owning-growth.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --no-overlap --emit-llvm $< -o $@

$(addprefix $(BUILD)/owning-growth-cost-,normal.ll retained.ll inlined.ll): $(BUILD)/owning-growth-cost-%.ll: $(BUILD)/owning-growth-cost.raw.ll $(BUILD)/owning-growth-abi$(EXE)
	$(BUILD)/owning-growth-abi$(EXE) $< $@ $*

$(BUILD)/owning-growth-cost-%.opt.ll: $(BUILD)/owning-growth-cost-%.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/owning-growth-costs-%$(EXE): owning-growth-costs.c $(BUILD)/owning-growth-cost-%.ll ../costs/sparse-owned.c
	@set -eu; case '$*' in normal) flags= ;; retained) flags=-DRETAIN_HELPERS ;; inlined) flags=-DINLINE_HELPERS ;; *) exit 1 ;; esac; \
	  $(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $$flags $< $(BUILD)/owning-growth-cost-$*.ll -o $@

.PHONY: check-owning-costs measure-owning-costs
check-owning-costs: $(BUILD)/owning-growth-abi-tests$(EXE) \
                    $(addprefix $(BUILD)/owning-growth-costs-,normal$(EXE) retained$(EXE) inlined$(EXE)) \
                    $(addprefix $(BUILD)/owning-growth-cost-,normal.opt.ll retained.opt.ll inlined.opt.ll)
	rustfmt --edition 2024 --check owning-growth-abi.rs
	$(BUILD)/owning-growth-abi-tests$(EXE)
	$(BUILD)/owning-growth-abi$(EXE) --inspect $(addprefix $(BUILD)/owning-growth-cost-,normal.opt.ll retained.opt.ll inlined.opt.ll)
	$(BUILD)/owning-growth-costs-normal$(EXE) check
	$(BUILD)/owning-growth-costs-retained$(EXE) check
	$(BUILD)/owning-growth-costs-inlined$(EXE) check

measure-owning-costs: check-owning-costs
	@set -eu; output=$(BUILD)/owning-growth-measurements.csv; \
	  echo 'cohort,contract,variant,operation,capacity,sample,repetitions,operations,elapsed_ns,checksum,digest,requests,peak_bytes' > "$$output"; \
	  for cohort in 0 1; do \
	    if test "$$cohort" = 0; then contracts='normal retained inlined'; else contracts='inlined retained normal'; fi; \
	    for contract in $$contracts; do \
	      $(BUILD)/owning-growth-costs-$$contract$(EXE) measure > $(BUILD)/owning-growth-sample.csv; \
	      awk -v cohort="$$cohort" 'NR > 1 { print cohort "," $$0 }' $(BUILD)/owning-growth-sample.csv >> "$$output"; \
	    done; \
	  done

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

$(BUILD)/priority-borrowed.raw.ll: priority-borrowed.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --no-overlap --emit-llvm $< -o $@

$(BUILD)/priority-borrowed.ll: $(BUILD)/priority-borrowed.raw.ll $(BUILD)/abi$(EXE)
	$(BUILD)/abi$(EXE) $< $@ wf_priority_trace

$(BUILD)/priority-borrowed-costs$(EXE): priority-costs.c $(BUILD)/priority-borrowed.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $^ -o $@

$(BUILD)/priority-borrowed-boundary-costs$(EXE): priority-costs.c $(BUILD)/priority-borrowed.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module -DRETAIN_HELPERS $^ -o $@

$(BUILD)/priority-borrowed-retained.ll: $(BUILD)/priority-borrowed.ll
	awk '/^define internal (void|i64) @wf_(push|pop)\(/ { if (sub(/ #0 [{]$$/, " noinline #0 {") != 1) exit 1; retained++ } { print } END { if (retained != 2) exit 1 }' $< > $@

$(BUILD)/priority-borrowed-retained-costs$(EXE): priority-costs.c $(BUILD)/priority-borrowed-retained.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module -DRETAIN_HELPERS $^ -o $@

$(BUILD)/priority-placement$(EXE): priority-placement.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/priority-placement.stamp: $(BUILD)/priority.ll $(BUILD)/priority-placement$(EXE)
	$(BUILD)/priority-placement$(EXE) $< $(BUILD)/priority-retained.ll $(BUILD)/priority-placed.ll
	touch $@

$(BUILD)/priority-retained.ll $(BUILD)/priority-placed.ll: $(BUILD)/priority-placement.stamp
	@test -f $@

$(BUILD)/priority-retained.opt.ll: $(BUILD)/priority-retained.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/priority-placed.opt.ll: $(BUILD)/priority-placed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/priority-retained-costs$(EXE): priority-costs.c $(BUILD)/priority-retained.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module -DRETAIN_HELPERS $^ -o $@

$(BUILD)/priority-placed-costs$(EXE): priority-costs.c $(BUILD)/priority-placed.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module -DRETAIN_HELPERS $^ -o $@

check-interface: $(BUILD)/priority-retained-costs$(EXE) $(BUILD)/priority-placed-costs$(EXE) \
                 $(BUILD)/priority-borrowed-retained-costs$(EXE) \
                 $(BUILD)/priority-retained.opt.ll $(BUILD)/priority-placed.opt.ll
	$(BUILD)/priority-placement$(EXE) --inspect $(BUILD)/priority-retained.opt.ll $(BUILD)/priority-placed.opt.ll
	$(BUILD)/priority-retained-costs$(EXE) check
	$(BUILD)/priority-placed-costs$(EXE) check
	$(BUILD)/priority-borrowed-retained-costs$(EXE) check

# Fresh observations stay in .build; check never overwrites retained CSVs.
measure-interface: check-interface $(BUILD)/priority-costs$(EXE)
	@set -eu; \
	  output=$(BUILD)/priority-interface-measurements.csv; \
	  echo 'cohort,contract,variant,rounds,sample,calls,elapsed_ns,checksum' > "$$output"; \
	  for cohort in 0 1; do \
	    if test "$$cohort" = 0; then variants='original retained placed borrowed'; else variants='borrowed placed retained original'; fi; \
	    for variant in $$variants; do \
	      case "$$variant" in original) binary=priority-costs ;; retained) binary=priority-retained-costs ;; placed) binary=priority-placed-costs ;; borrowed) binary=priority-borrowed-retained-costs ;; esac; \
	      $(BUILD)/$$binary$(EXE) measure > $(BUILD)/priority-interface-sample.csv; \
	      awk -v cohort="$$cohort" -v contract="$$variant" 'NR > 1 { print cohort "," contract "," $$0 }' $(BUILD)/priority-interface-sample.csv >> "$$output"; \
	    done; \
	  done

$(BUILD)/owning-map.raw.ll: owning-map.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --no-overlap --emit-llvm $< -o $@

$(BUILD)/owning-map.observed.ll: $(BUILD)/owning-map.raw.ll $(BUILD)/abi$(EXE)
	$(BUILD)/abi$(EXE) --observe-allocations $< $@

$(BUILD)/owning-map-observer$(EXE): owning-map-observer.c $(BUILD)/owning-map.observed.ll
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $^ -o $@

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

check: check-interface check-owning-growth check-owning-costs $(BUILD)/priority-costs$(EXE) $(BUILD)/priority-boundary-costs$(EXE) \
       $(BUILD)/priority-borrowed-costs$(EXE) $(BUILD)/priority-borrowed-boundary-costs$(EXE) \
       $(BUILD)/priority-borrowed-retained-costs$(EXE) \
       $(BUILD)/owning-map-observer$(EXE) $(BUILD)/growth-costs$(EXE)
	rustfmt --edition 2024 --check abi.rs growth-abi.rs priority-placement.rs
	@set -eu; \
	  for source in hashmap owning-map owning-growth ordered priority priority-borrowed packed-page growth boxed-migration ordered-runtime-gap boxed-helper-gap shared-option-view; do \
	    for mode in default par no-overlap; do \
	      case "$$mode" in default) flags= ;; par) flags=--par ;; no-overlap) flags=--no-overlap ;; esac; \
	      "$(WHITEFOOTC)" $$flags "$$source.wf" -o "$(BUILD)/$$source-$$mode$(EXE)"; \
	      "$(BUILD)/$$source-$$mode$(EXE)"; \
	    done; \
	  done
	@set -eu; \
	  for case_name in rejected-wrapper; do \
	    rule='[FN-9]'; detail=InvalidPostconditionSelector; \
	    status=0; rm -f "$(BUILD)/$$case_name$(EXE)"; \
	    "$(WHITEFOOTC)" "$$case_name.wf" -o "$(BUILD)/$$case_name$(EXE)" > "$(BUILD)/$$case_name.log" 2>&1 || status=$$?; \
	    if test "$$status" -ne 1 || test -e "$(BUILD)/$$case_name$(EXE)" || ! grep -F "$$rule" "$(BUILD)/$$case_name.log" > /dev/null || ! grep -F "$$detail" "$(BUILD)/$$case_name.log" > /dev/null; then \
	      echo "Unexpected outcome: $$case_name (exit $$status)"; cat "$(BUILD)/$$case_name.log"; exit 1; \
	    fi; \
	  done
	$(BUILD)/priority-costs$(EXE) check
	$(BUILD)/priority-boundary-costs$(EXE) check
	$(BUILD)/priority-borrowed-costs$(EXE) check
	$(BUILD)/priority-borrowed-boundary-costs$(EXE) check
	$(BUILD)/priority-borrowed-retained-costs$(EXE) check
	$(BUILD)/owning-map-observer$(EXE)
	$(BUILD)/growth-costs$(EXE) check

measure: check
	$(BUILD)/priority-costs$(EXE) measure > $(BUILD)/priority-measurements.csv
	$(BUILD)/priority-boundary-costs$(EXE) measure > $(BUILD)/priority-boundary-measurements.csv
	$(BUILD)/priority-borrowed-costs$(EXE) measure > $(BUILD)/priority-borrowed-measurements.csv
	$(BUILD)/priority-borrowed-boundary-costs$(EXE) measure > $(BUILD)/priority-borrowed-boundary-measurements.csv
	$(BUILD)/priority-borrowed-retained-costs$(EXE) measure > $(BUILD)/priority-borrowed-retained-measurements.csv
	$(BUILD)/growth-costs$(EXE) measure > $(BUILD)/growth-measurements.csv

reproduce-runtime-gap: check
	$(BUILD)/ordered-runtime-gap-default$(EXE)

reproduce-region-gap: check
	$(BUILD)/boxed-helper-gap-default$(EXE)

clean:
	rm -rf $(BUILD)
