# One revision's formal fixtures compiled by that revision's compiler and
# linked with that revision's ordinary runtime. The hosted runner may override
# FIXTURES when constructing the merge-base image with this timing instrument.
# No framework, research input or timing is selected in this file.
HERE := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
CURRENT := $(abspath $(HERE)/../..)
ROOT ?= $(CURRENT)
BUILD ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot-performance-candidate
WFC ?= $(ROOT)/compiler/target/gate/whitefootc
CLANG ?= /usr/bin/clang
FIXTURES ?= $(CURRENT)/tests/programs/compute
KERNELS := mandelbrot records fir quadrature stencil
IMAGES := $(addprefix $(BUILD)/,$(KERNELS))
.DEFAULT_GOAL := build
.DELETE_ON_ERROR:

include $(CURRENT)/compiler/runtime.mk

build: $(IMAGES)

$(BUILD)/%.ll: $(FIXTURES)/%.wf $(FIXTURES)/%_host.ll $(FIXTURES)/host-adapter.awk $(WFC) Makefile
	@mkdir -p "$(dir $@)"
	"$(WFC)" --par --emit-llvm "$<" -o "$@.raw"
	awk -f "$(FIXTURES)/host-adapter.awk" "$@.raw" "$(FIXTURES)/$*_host.ll" > "$@.bound"
	sed -e 's/@main(/@wf_fixture_main(/g' -e 's/@wf__main_body(/@wf_fixture_body(/g' "$@.bound" > "$@"
	@rm "$@.raw" "$@.bound"

$(BUILD)/%.o: $(BUILD)/%.ll $(NATIVE_CONFIG)
	$(CLANG) -O2 -Wno-override-module -x ir -c "$<" -o "$@"

$(BUILD)/%_oracle.o: $(FIXTURES)/%_oracle.c $(FIXTURES)/oracle.h $(NATIVE_CONFIG) Makefile
	@mkdir -p "$(dir $@)"
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -DWF_ORACLE_NO_MAIN -DWF_ORACLE_PERFORMANCE -c "$<" -o "$@"

$(BUILD)/runner.o: runner.c $(FIXTURES)/oracle.h $(NATIVE_CONFIG) Makefile
	$(CLANG) -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -DWF_ORACLE_NO_MAIN -DWF_ORACLE_PERFORMANCE -c "$<" -o "$@"

$(BUILD)/%: $(BUILD)/%.o $(BUILD)/%_oracle.o $(BUILD)/runner.o $(NATIVE_OBJECTS)
	$(CLANG) -O2 $^ $(NATIVE_LINK_FLAGS) -o "$@"

.SECONDARY: $(addsuffix .ll,$(IMAGES)) $(addsuffix .o,$(IMAGES)) $(addsuffix _oracle.o,$(IMAGES)) $(BUILD)/runner.o
.PHONY: build
