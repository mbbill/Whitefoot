SHELL := /bin/sh
ROOT := $(abspath ../../../..)
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
CLANG ?= clang
BUILD ?= .build
SOURCES := dense-map.wf sparse-map.wf map-library.wf
OBSERVER := $(ROOT)/tests/programs/containers/container-allocation-observer.c
.DELETE_ON_ERROR:
.DEFAULT_GOAL := check

include $(ROOT)/compiler/runtime.mk

.PHONY: native-check build-candidates check-candidates build-costs check-costs check measure clean

$(BUILD):
	mkdir -p $@

$(BUILD)/map-native-%$(EXE): map-costs.c Makefile | $(BUILD)
	@set -eu; case '$*' in normal) flags= ;; retained) flags=-DRETAIN_HELPERS ;; *) exit 1 ;; esac; \
	  $(CLANG) -std=c11 -O2 -Wall -Wextra -Werror $$flags $< -o $@

native-check: $(addprefix $(BUILD)/map-native-,normal$(EXE) retained$(EXE))
	$(BUILD)/map-native-normal$(EXE) check
	$(BUILD)/map-native-retained$(EXE) check

# Three CLI configurations exercise two distinct lowerings: default and
# --no-overlap both select sequential lowering; --par selects parallel.
FIXTURES := dense dense-owned sparse sparse-owned
MODES := default no-overlap par
FIXTURE_dense := dense-map.wf dense-check.wf
FIXTURE_dense-owned := dense-map.wf dense-owned-check.wf
FIXTURE_sparse := sparse-map.wf sparse-check.wf
FIXTURE_sparse-owned := sparse-map.wf sparse-owned-check.wf
EXPECTED_dense := 24
EXPECTED_dense-owned := 14
EXPECTED_sparse := 29
EXPECTED_sparse-owned := 14
FLAGS_default :=
FLAGS_no-overlap := --no-overlap
FLAGS_par := --par

define fixture_rules
$(BUILD)/$(1)-$(2).raw.ll: $$(FIXTURE_$(1)) $(WHITEFOOTC) Makefile | $(BUILD)
	$$(WHITEFOOTC) $$(FLAGS_$(2)) --emit-llvm $$(FIXTURE_$(1)) -o $$@

$(BUILD)/$(1)-$(2)$(EXE): $(BUILD)/$(1)-$(2).raw.ll $(NATIVE_OBJECTS)
	$$(CLANG) -O2 -Wno-override-module $$< $$(NATIVE_OBJECTS) $$(NATIVE_LINK_FLAGS) -o $$@

$(BUILD)/$(1)-$(2).observed.ll: $(BUILD)/$(1)-$(2).raw.ll Makefile
	sed -e 's/@malloc(/@wf_observe_allocate(/g' -e 's/@free(/@wf_observe_release(/g' -e 's/@main(/@wf_fixture_main(/g' $$< > $$@

$(BUILD)/$(1)-$(2)-observed$(EXE): $(BUILD)/$(1)-$(2).observed.ll $(OBSERVER) $(NATIVE_OBJECTS)
	$$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module -I$$(NATIVE_ROOT) $$< $$(OBSERVER) $$(NATIVE_OBJECTS) $$(NATIVE_LINK_FLAGS) -o $$@
endef
$(foreach fixture,$(FIXTURES),$(foreach mode,$(MODES),$(eval $(call fixture_rules,$(fixture),$(mode)))))

FIXTURE_BINARIES := $(foreach fixture,$(FIXTURES),$(foreach mode,$(MODES),$(BUILD)/$(fixture)-$(mode)$(EXE) $(BUILD)/$(fixture)-$(mode)-observed$(EXE)))
build-candidates: $(FIXTURE_BINARIES)

check-candidates: build-candidates
	@set -eu; \
	  for item in $(foreach fixture,$(FIXTURES),$(foreach mode,$(MODES),$(fixture)-$(mode):$(EXPECTED_$(fixture)))); do \
	    stem=$${item%:*}; count=$${item##*:}; \
	    $(BUILD)/$$stem$(EXE) > $(BUILD)/$$stem.stdout 2> $(BUILD)/$$stem.stderr; \
	    test ! -s $(BUILD)/$$stem.stdout; test ! -s $(BUILD)/$$stem.stderr; \
	    $(BUILD)/$$stem-observed$(EXE) > $(BUILD)/$$stem.observed.stdout 2> $(BUILD)/$$stem.observed.stderr; \
	    test ! -s $(BUILD)/$$stem.observed.stderr; \
	    test "$$(cat $(BUILD)/$$stem.observed.stdout)" = "container allocation observer: $$count allocations, each released exactly once"; \
	    echo "$$stem: ordinary execution and $$count exact allocation identities passed"; \
	  done
	@set -eu; observer=$(BUILD)/dense-par-observed$(EXE); \
	  $$observer concurrent > $(BUILD)/observer.stdout 2> $(BUILD)/observer.stderr; \
	  test ! -s $(BUILD)/observer.stderr; \
	  test "$$(cat $(BUILD)/observer.stdout)" = 'container allocation observer: 32 allocations, each released exactly once'; \
	  for control in double-release foreign-release missing-release; do \
	    status=0; $$observer $$control > $(BUILD)/observer.stdout 2> $(BUILD)/observer.stderr || status=$$?; \
	    test "$$status" = 1; test ! -s $(BUILD)/observer.stdout; \
	    case $$control in \
	      double-release) message='allocation released twice' ;; \
	      foreign-release) message='release did not return an allocated address' ;; \
	      missing-release) message='every allocation is released exactly once' ;; \
	    esac; \
	    test "$$(cat $(BUILD)/observer.stderr)" = "container allocation observer: $$message"; \
	  done; \
	  echo 'Existing quarantining observer: concurrent and three negative controls passed'

$(BUILD)/map-library.raw.ll: $(SOURCES) $(WHITEFOOTC) Makefile | $(BUILD)
	$(WHITEFOOTC) --emit-llvm $(SOURCES) -o $@

$(BUILD)/map-library-normal.ll: $(BUILD)/map-library.raw.ll Makefile
	sed -e 's/@malloc(/@wf_cost_allocate(/g' -e 's/@free(/@wf_cost_release(/g' -e 's/@main(/@wf_fixture_main(/g' \
	    -e 's/^declare ptr @wf_cost_allocate(i64)$$/declare noalias nonnull ptr @wf_cost_allocate(i64)/' $< | \
	  awk '/^define .* @wf_map_cost_(dense|sparse)_(word|record)_trace\(/ { if (sub(/ #0 [{]$$/, " noinline #0 {") != 1) exit 1; traces++ } { print } END { if (traces != 4) exit 1 }' > $@

$(BUILD)/map-library-retained.ll: $(BUILD)/map-library-normal.ll Makefile
	awk '/^define .* @wf_(dense|sparse)_map_(new|put|remove|lookup|reserve|rehash|free)[$$(]/ || \
	     /^define .* @wf_map_cost_(hash|equal|word_observe|record_observe|word_consume|record_consume)\(/ \
	     { if (sub(/ #0 [{]$$/, " noinline #0 {") != 1) exit 1; retained++ } \
	     { print } END { if (retained == 0) exit 1; print "Retained public map/callback definitions: " retained > "/dev/stderr" }' $< > $@

$(BUILD)/map-library-%.opt.ll: $(BUILD)/map-library-%.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/control-%.opt.ll: map-costs.c Makefile | $(BUILD)
	@set -eu; case '$*' in normal) flags= ;; retained) flags=-DRETAIN_HELPERS ;; *) exit 1 ;; esac; \
	  $(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -DWITH_WF $$flags -S -emit-llvm $< -o $@

$(BUILD)/map-costs-%$(EXE): map-costs.c $(BUILD)/map-library-%.ll $(NATIVE_OBJECTS)
	@set -eu; case '$*' in normal) flags= ;; retained) flags=-DRETAIN_HELPERS ;; *) exit 1 ;; esac; \
	  $(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -DWITH_WF -Wno-override-module $$flags $< $(BUILD)/map-library-$*.ll $(NATIVE_OBJECTS) $(NATIVE_LINK_FLAGS) -o $@

build-costs: $(addprefix $(BUILD)/map-costs-,normal$(EXE) retained$(EXE)) \
             $(addprefix $(BUILD)/map-library-,normal.opt.ll retained.opt.ll) \
             $(addprefix $(BUILD)/control-,normal.opt.ll retained.opt.ll)

check-costs: build-costs
	@grep -Eq '^declare noalias nonnull ptr @wf_cost_allocate\(i64\)' $(BUILD)/map-library-normal.ll
	$(BUILD)/map-costs-normal$(EXE) check
	$(BUILD)/map-costs-retained$(EXE) check
	@set -eu; for operation in new put remove lookup reserve rehash free; do \
	  grep -Eq "call .*@\"?wf_(dense|sparse)_map_$${operation}"'[$$(]' $(BUILD)/map-library-retained.opt.ll; \
	  grep -Eq "call .*@(word|record)_[a-z]+_$${operation}[.(]" $(BUILD)/control-retained.opt.ll; \
	done; \
	for callback in hash equal word_observe record_observe word_consume record_consume; do \
	  grep -Eq "call .*@wf_map_cost_$${callback}[(]" $(BUILD)/map-library-retained.opt.ll; \
	done; \
	for callback in key_hash key_equal word_observe record_observe word_consume record_consume; do \
	  grep -Eq "call .*@$${callback}[.(]" $(BUILD)/control-retained.opt.ll; \
	done; \
	echo 'Public map operations and supplied callbacks survive in both retained modules'

check: native-check check-candidates check-costs

measure: check
	@set -eu; output=$(BUILD)/measurements.csv; first=yes; \
	  for cohort in 0 1; do \
	    if test "$$cohort" = 0; then modes='normal retained'; else modes='retained normal'; fi; \
	    for mode in $$modes; do \
	      $(BUILD)/map-costs-$$mode$(EXE) measure $$cohort > $(BUILD)/cohort-$$cohort-$$mode.csv; \
	      if test "$$first" = yes; then head -n 1 $(BUILD)/cohort-$$cohort-$$mode.csv > "$$output"; first=no; fi; \
	      awk 'NR > 1' $(BUILD)/cohort-$$cohort-$$mode.csv >> "$$output"; \
	    done; \
	  done; \
	  echo "Raw samples: $$output"

clean:
	rm -rf $(BUILD)
