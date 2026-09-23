SHELL := /bin/sh
ROOT := $(abspath ../../../..)
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
CLANG ?= clang
BUILD ?= .build
.DELETE_ON_ERROR:

.PHONY: native-check clean

$(BUILD):
	mkdir -p $@

$(BUILD)/map-native-%$(EXE): map-costs.c Makefile | $(BUILD)
	@set -eu; case '$*' in normal) flags= ;; retained) flags=-DRETAIN_HELPERS ;; *) exit 1 ;; esac; \
	  $(CLANG) -std=c11 -O2 -Wall -Wextra -Werror $$flags $< -o $@

native-check: $(addprefix $(BUILD)/map-native-,normal$(EXE) retained$(EXE))
	$(BUILD)/map-native-normal$(EXE) check
	$(BUILD)/map-native-retained$(EXE) check

clean:
	rm -rf $(BUILD)
