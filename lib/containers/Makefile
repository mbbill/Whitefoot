SHELL := /bin/sh
ROOT := $(abspath ../..)
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
CLANG ?= clang
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot
BUILD := $(WHITEFOOT_SCRATCH_ROOT)/container-library-tests
VECTOR_SOURCES := vector.wf tests/vector_program.wf
.DELETE_ON_ERROR:

include ../../research/experiments/container-representation/native.mk

.PHONY: check

check: $(addprefix $(BUILD)/vector-allocation-observer-,default$(EXE) par$(EXE) no-overlap$(EXE))
	@mkdir -p "$(BUILD)"
	@set -eu; \
	for mode in default par no-overlap; do \
		case "$$mode" in \
			default) flags= ;; \
			par) flags=--par ;; \
			no-overlap) flags=--no-overlap ;; \
		esac; \
		program="$(BUILD)/vector-$$mode$(EXE)"; \
		"$(WHITEFOOTC)" $$flags $(VECTOR_SOURCES) -o "$$program"; \
		"$$program"; \
	done
	@$(BUILD)/vector-allocation-observer-default$(EXE)
	@$(BUILD)/vector-allocation-observer-par$(EXE)
	@$(BUILD)/vector-allocation-observer-no-overlap$(EXE)
	@echo "container library: vector program passed in default, par, and no-overlap modes"

$(BUILD)/vector-observed-%.raw.ll: $(VECTOR_SOURCES) $(WHITEFOOTC)
	@mkdir -p "$(BUILD)"
	@set -eu; case '$*' in default) flags= ;; par) flags=--par ;; no-overlap) flags=--no-overlap ;; *) exit 1 ;; esac; \
	  "$(WHITEFOOTC)" $$flags --emit-llvm $(VECTOR_SOURCES) -o "$@"

$(BUILD)/vector-observed-%.ll: $(BUILD)/vector-observed-%.raw.ll
	sed -e 's/@malloc(/@wf_observe_allocate(/g' \
	    -e 's/@free(/@wf_observe_release(/g' \
	    -e 's/@main(/@wf_fixture_main(/g' "$<" > "$@"

$(BUILD)/vector-allocation-observer-%$(EXE): tests/vector_allocation_observer.c $(BUILD)/vector-observed-%.ll $(NATIVE_OBJECTS)
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wno-override-module "$<" "$(BUILD)/vector-observed-$*.ll" $(NATIVE_OBJECTS) $(NATIVE_LINK_FLAGS) -o "$@"
