SHELL := /bin/sh
ROOT := $(abspath ../..)
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
CLANG ?= clang
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot
BUILD := $(WHITEFOOT_SCRATCH_ROOT)/container-library-tests
VECTOR_SOURCES := vector.wf tests/vector_program.wf
MODES := default par
RAW_MODULES := $(foreach mode,$(MODES),$(BUILD)/vector-$(mode).ll)
OBSERVED_MODULES := $(foreach mode,$(MODES),$(BUILD)/vector-observed-$(mode).ll)
PROGRAMS := $(foreach mode,$(MODES),$(BUILD)/vector-$(mode)$(EXE))
OBSERVERS := $(foreach mode,$(MODES),$(BUILD)/vector-allocation-observer-$(mode)$(EXE))
.SECONDARY: $(RAW_MODULES) $(OBSERVED_MODULES)
.DELETE_ON_ERROR:

include $(ROOT)/compiler/runtime.mk

.PHONY: check

check: $(PROGRAMS) $(OBSERVERS)
	@set -eu; for mode in $(MODES); do \
		"$(BUILD)/vector-$$mode$(EXE)"; \
		"$(BUILD)/vector-allocation-observer-$$mode$(EXE)"; \
	done
	@echo "container library: vector values and allocation ledgers passed in sequential and parallel lowering"

# The default and --no-overlap are the same lowering. CLI option parsing has
# its own unit coverage; this caller needs one sequential and one parallel
# module, each linked normally and with the allocation-refusal observer.
$(RAW_MODULES): $(BUILD)/vector-%.ll: $(VECTOR_SOURCES) $(WHITEFOOTC) Makefile
	@mkdir -p "$(BUILD)"
	@set -eu; case '$*' in default) flags= ;; par) flags=--par ;; *) exit 1 ;; esac; \
	  "$(WHITEFOOTC)" $$flags --emit-llvm $(VECTOR_SOURCES) -o "$@"

$(OBSERVED_MODULES): $(BUILD)/vector-observed-%.ll: $(BUILD)/vector-%.ll Makefile
	sed -e 's/@malloc(/@wf_observe_allocate(/g' \
	    -e 's/@free(/@wf_observe_release(/g' \
	    -e 's/@main(/@wf_fixture_main(/g' "$<" > "$@"

$(PROGRAMS): $(BUILD)/vector-%$(EXE): $(BUILD)/vector-%.ll $(NATIVE_OBJECTS) Makefile
	$(CLANG) -O2 -Wno-override-module "$<" $(NATIVE_OBJECTS) $(NATIVE_LINK_FLAGS) -o "$@"

$(OBSERVERS): $(BUILD)/vector-allocation-observer-%$(EXE): tests/vector_allocation_observer.c $(BUILD)/vector-observed-%.ll $(NATIVE_OBJECTS) Makefile
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module "$<" "$(BUILD)/vector-observed-$*.ll" $(NATIVE_OBJECTS) $(NATIVE_LINK_FLAGS) -o "$@"
