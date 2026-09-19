SHELL := /bin/sh
ROOT := $(abspath ../../../..)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
CLANG ?= clang
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
BUILD := .build
SOURCES := $(ROOT)/lib/containers/vector.wf vector-library.wf
.DELETE_ON_ERROR:

include ../../../../compiler/runtime.mk

.PHONY: check measure clean

$(BUILD):
	mkdir -p $@

$(BUILD)/vector-library.raw.ll: $(SOURCES) $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --no-overlap --emit-llvm $(SOURCES) -o $@

$(BUILD)/vector-library-normal.ll: $(BUILD)/vector-library.raw.ll
	sed -e 's/@malloc(/@wf_cost_allocate(/g' \
	    -e 's/@free(/@wf_cost_release(/g' \
	    -e 's/@main(/@wf_fixture_main(/g' $< > $@

$(BUILD)/vector-library-retained.ll: $(BUILD)/vector-library-normal.ll
	awk '/^define .* @wf_(grow_vector_|vector_library_round|vector_digest_accept)/ { if (sub(/ #0 [{]$$/, " noinline #0 {") != 1) exit 1; retained++ } { print } END { if (retained != 11) exit 1 }' $< > $@

$(BUILD)/vector-library-%.opt.ll: $(BUILD)/vector-library-%.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/vector-costs-%$(EXE): vector-costs.c $(BUILD)/vector-library-%.ll $(NATIVE_OBJECTS)
	@set -eu; case '$*' in normal) flags= ;; retained) flags=-DRETAIN_HELPERS ;; *) exit 1 ;; esac; \
	  $(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module $$flags $< $(BUILD)/vector-library-$*.ll $(NATIVE_OBJECTS) $(NATIVE_LINK_FLAGS) -o $@

check: $(addprefix $(BUILD)/vector-costs-,normal$(EXE) retained$(EXE)) \
       $(addprefix $(BUILD)/vector-library-,normal.opt.ll retained.opt.ll)
	$(BUILD)/vector-costs-normal$(EXE) check
	$(BUILD)/vector-costs-retained$(EXE) check
	@set -eu; normal_calls=`grep -Ec 'call .*@"?wf_grow_vector_' $(BUILD)/vector-library-normal.opt.ll || true`; \
	 retained_calls=`grep -Ec 'call .*@"?wf_grow_vector_' $(BUILD)/vector-library-retained.opt.ll || true`; \
	 test "$$retained_calls" -gt 0; \
	 echo "vector library optimized helper calls: normal=$$normal_calls retained=$$retained_calls"

measure: check
	@output=$(BUILD)/measurements.csv; \
	 echo 'cohort,contract,path,variant,sample,calls,rounds,elapsed_ns,checksum,requests,peak_bytes' > $$output; \
	 for cohort in 0 1; do \
	   if test $$cohort = 0; then contracts='normal retained'; else contracts='retained normal'; fi; \
	   for contract in $$contracts; do \
	     $(BUILD)/vector-costs-$$contract$(EXE) measure > $(BUILD)/sample.csv; \
	     awk -v cohort=$$cohort 'NR > 1 { print cohort "," $$0 }' $(BUILD)/sample.csv >> $$output; \
	   done; \
	 done

clean:
	rm -rf $(BUILD)
