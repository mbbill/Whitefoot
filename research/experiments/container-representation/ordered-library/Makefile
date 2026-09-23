SHELL := /bin/sh
ROOT := $(abspath ../../../..)
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
WHITEFOOTC_LABEL ?= configured-whitefootc
CLANG ?= clang
BUILD ?= .build
SOURCES := $(ROOT)/lib/containers/ordered-map.wf ordered-library.wf
CONTROLS := word-source record-source word-direct record-direct word-avl record-avl
MODES := normal retained
CFLAGS := -std=c11 -O2 -Wall -Wextra -Werror
.DEFAULT_GOAL := check
.DELETE_ON_ERROR:
.SECONDARY:

include ../../../../compiler/runtime.mk

.PHONY: build check verify measure measure-only native-check structural-check identities clean

$(BUILD):
	mkdir -p $@

$(BUILD)/ordered-library.raw.ll: $(SOURCES) $(WHITEFOOTC) Makefile | $(BUILD)
	$(WHITEFOOTC) --emit-llvm $(SOURCES) -o $@

$(BUILD)/ordered-library-normal.ll: $(BUILD)/ordered-library.raw.ll
	sed -e 's/@malloc(/@wf_cost_allocate(/g' -e 's/@free(/@wf_cost_release(/g' -e 's/@main(/@wf_fixture_main(/g' \
	    -e 's/^declare ptr @wf_cost_allocate(i64)$$/declare noalias nonnull ptr @wf_cost_allocate(i64)/' $< | \
	  awk '/^define / && !/^define (internal|private|weak) / && /@wf_/ && !/@wf__/ && \
	       !/@wf_ordered_cost_(word|record)_trace[(]/ { sub(/^define /, "define internal ") } { print }' > $@

# Retain exactly the public map operations and supplied callbacks. The private
# node helpers, trace, digest helpers, and payload-content helpers stay ordinary.
$(BUILD)/ordered-library-retained.ll: $(BUILD)/ordered-library-normal.ll Makefile
	awk '/^define .* @wf_ordered_map_(new|len|put|lookup|edit|remove|each|range|free)([$$(]|[[:space:]])/ || \
	     /^define .* @wf_ordered_cost_(compare|word_(make|observe|visit|edit|consume)|record_(make|observe|visit|edit|consume))[$$(]/ \
	     { if (sub(/ #0 [{]$$/, " noinline #0 {") != 1) exit 1; retained++ } \
	     { print } END { if (retained == 0) exit 1; print "retained public WF operations/callbacks: " retained > "/dev/stderr" }' $< > $@

$(BUILD)/ordered-library-%.opt.ll: $(BUILD)/ordered-library-%.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

# One specialization per C translation unit gives C and WF the same opportunity
# to optimize private code without an indirect comparator or payload-size loop.
$(BUILD)/control-%.ll: ordered-costs.c Makefile | $(BUILD)
	@set -eu; stem='$*'; element=$${stem%%-*}; tail=$${stem#*-}; shape=$${tail%%-*}; mode=$${tail#*-}; \
	  flags='-DCONTROL_TEMPLATE'; prefix=$${element}_$${shape}; \
	  case "$$element" in word) ;; record) flags="$$flags -DWIDE_PAIR" ;; *) exit 1 ;; esac; \
	  case "$$shape" in source) flags="$$flags -DSOURCE_CONTROL" ;; direct) ;; avl) flags="$$flags -DAVL_CONTROL" ;; *) exit 1 ;; esac; \
	  case "$$mode" in normal) ;; retained) flags="$$flags -DRETAIN_PUBLIC" ;; audit) flags="$$flags -DSTRUCTURAL_AUDIT" ;; *) exit 1 ;; esac; \
	  $(CLANG) $(CFLAGS) $$flags -DPREFIX=$$prefix -S -emit-llvm $< -o $@

define mode_rules
$(BUILD)/ordered-costs-$(1)$(EXE): ordered-costs.c $(addprefix $(BUILD)/control-,$(addsuffix -$(1).ll,$(CONTROLS))) $(BUILD)/ordered-library-$(1).ll $(NATIVE_OBJECTS)
	$(CLANG) $(CFLAGS) -Wno-override-module $(if $(filter retained,$(1)),-DRETAIN_PUBLIC,) $$^ $(NATIVE_LINK_FLAGS) -o $$@

$(BUILD)/native-costs-$(1)$(EXE): ordered-costs.c $(addprefix $(BUILD)/control-,$(addsuffix -$(1).ll,$(CONTROLS)))
	$(CLANG) $(CFLAGS) -DNATIVE_ONLY -Wno-override-module $(if $(filter retained,$(1)),-DRETAIN_PUBLIC,) $$^ -o $$@
endef
$(foreach mode,$(MODES),$(eval $(call mode_rules,$(mode))))

$(BUILD)/structural-costs$(EXE): ordered-costs.c $(BUILD)/control-word-source-audit.ll \
       $(addprefix $(BUILD)/control-,$(addsuffix -normal.ll,$(filter-out word-source,$(CONTROLS))))
	$(CLANG) $(CFLAGS) -DNATIVE_ONLY -DSTRUCTURAL_DRIVER -Wno-override-module $^ -o $@

build: $(addprefix $(BUILD)/ordered-costs-,$(addsuffix $(EXE),$(MODES))) \
       $(addprefix $(BUILD)/ordered-library-,$(addsuffix .opt.ll,$(MODES))) $(BUILD)/structural-costs$(EXE)

verify:
	@grep -Eq '^declare noalias nonnull ptr @wf_cost_allocate\(i64\)' $(BUILD)/ordered-library-normal.ll
	$(BUILD)/ordered-costs-normal$(EXE) check
	$(BUILD)/ordered-costs-retained$(EXE) check
	$(BUILD)/structural-costs$(EXE) structure > $(BUILD)/structural-paths.csv
	@set -eu; wf=`grep -Ec 'call .*@"?wf_ordered_map_(put|lookup|edit|remove|each|range|free)[$$(]' $(BUILD)/ordered-library-retained.opt.ll || true`; \
	  test "$$wf" -gt 0; echo "Retained WF public-operation call sites: $$wf"; \
	  for source in $(BUILD)/control-*-retained.ll; do \
	    count=`grep -Ec 'call .*@(word|record)_(source|direct|avl)_(put|lookup|edit|remove|each|range|free)[(]' "$$source" || true`; \
	    test "$$count" -gt 0; echo "Retained C public-operation call sites: $$source $$count"; \
	  done

check: build
	$(MAKE) verify

native-check: $(addprefix $(BUILD)/native-costs-,$(addsuffix $(EXE),$(MODES))) $(BUILD)/structural-costs$(EXE)
	$(BUILD)/native-costs-normal$(EXE) native-check
	$(BUILD)/native-costs-retained$(EXE) native-check
	$(BUILD)/structural-costs$(EXE) structure > $(BUILD)/structural-paths.csv

structural-check: $(BUILD)/structural-costs$(EXE)
	$(BUILD)/structural-costs$(EXE) structure

measure: check
	$(MAKE) measure-only

# Sample 0 is the single warm-up; 1--5 are the prespecified five samples.
# Within every sample the four implementation positions reverse on odd samples.
measure-only:
	@set -eu; for mode in $(MODES); do \
	  $(BUILD)/ordered-costs-$$mode$(EXE) measure > $(BUILD)/measurements-$$mode.csv; \
	done; \
	  cat $(BUILD)/measurements-normal.csv > $(BUILD)/measurements.csv; \
	  sed '1d' $(BUILD)/measurements-retained.csv >> $(BUILD)/measurements.csv; \
	  echo "Raw samples including warm-up: $(BUILD)/measurements.csv"

identities:
	@set -eu; { git -C "$(ROOT)" rev-parse HEAD; git -C "$(ROOT)" status --short; uname -srm; \
	  $(CLANG) --version | sed '/^InstalledDir:/d'; \
	  shasum -a 256 Makefile ordered-costs.c ordered-library.wf; \
	  (cd "$(ROOT)" && shasum -a 256 lib/containers/ordered-map.wf); \
	  shasum -a 256 "$(WHITEFOOTC)" | awk -v label='$(WHITEFOOTC_LABEL)' '{ print $$1 "  " label }'; \
	  for file in $(BUILD)/ordered-library*.ll $(BUILD)/control-*.ll $(BUILD)/ordered-costs-*$(EXE); do \
	    shasum -a 256 "$$file" | awk -v label="build/$${file##*/}" '{ print $$1 "  " label }'; \
	  done; \
	} > $(BUILD)/identities.txt

clean:
	rm -rf $(BUILD)
