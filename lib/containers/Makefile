SHELL := /bin/sh
ROOT := $(abspath ../..)
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot
BUILD := $(WHITEFOOT_SCRATCH_ROOT)/container-library-tests
VECTOR_SOURCES := vector.wf tests/vector_program.wf

.PHONY: check

check:
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
	@echo "container library: vector program passed in default, par, and no-overlap modes"
