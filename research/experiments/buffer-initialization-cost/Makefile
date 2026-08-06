ROOT := $(abspath ../../..)
WORK_ROOT ?= /Users/bytedance/do_not_scan/whitefoot-buffer-init-cost
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/cargo-target
CLANG := /usr/bin/clang
RUSTC := rustc
CARGO := cargo
COMMON_C := -O2 -std=c17 -Wall -Wextra -Werror

.PHONY: check bench build prepare

check: build
	WF_INIT_WORK_ROOT=$(WORK_ROOT) $(BUILD)/runner verify

bench: build
	WF_INIT_WORK_ROOT=$(WORK_ROOT) $(BUILD)/runner bench

build: prepare $(BUILD)/whitefoot-drain $(BUILD)/control $(BUILD)/runner \
	$(BUILD)/drain.opt.ll

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET)

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/drain.ll: drain.wf | prepare
	env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		$(CARGO) run --manifest-path $(ROOT)/compiler/Cargo.toml \
		--bin whitefootc --locked --offline -- --emit-llvm -o $@ $<

# The shape the row is about is a property of the module the host optimizer
# leaves, so the runner's oracle reads this file rather than trusting the
# emitted one.
$(BUILD)/drain.opt.ll: $(BUILD)/drain.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/whitefoot-drain: $(BUILD)/drain.ll
	$(CLANG) -O2 -Wno-override-module -x ir $< -o $@

$(BUILD)/control: control.c | prepare
	$(CLANG) $(COMMON_C) $< -o $@
