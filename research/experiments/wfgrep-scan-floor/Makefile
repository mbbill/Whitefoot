ROOT := $(abspath ../../..)
WORK_ROOT ?= /Users/bytedance/do_not_scan/whitefoot-wf-scan-floor
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/cargo-target
CLANG := /usr/bin/clang
RUSTC := rustc
CARGO := cargo
COMMON_C := -O2 -std=c17 -Wall -Wextra -Werror
COMMON_RUST := --edition=2024 -C opt-level=2 -C panic=abort -C codegen-units=1

.PHONY: check bench build prepare

check: build
	WF_SCAN_WORK_ROOT=$(WORK_ROOT) $(BUILD)/runner verify

bench: check
	WF_SCAN_WORK_ROOT=$(WORK_ROOT) $(BUILD)/runner bench

build: prepare \
	$(BUILD)/whitefoot-full $(BUILD)/whitefoot-early \
	$(BUILD)/c-full $(BUILD)/c-early \
	$(BUILD)/rust-full $(BUILD)/rust-early \
	$(BUILD)/wf-full.opt.ll $(BUILD)/wf-early.opt.ll \
	$(BUILD)/wf-full.s $(BUILD)/wf-early.s \
	$(BUILD)/c-full.s $(BUILD)/c-early.s $(BUILD)/rust-kernel.s \
	$(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET)

$(BUILD)/expose: expose.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/full.raw.ll: full_scan.wf | prepare
	env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		$(CARGO) run --manifest-path $(ROOT)/compiler/Cargo.toml \
		--bin whitefootc --locked --offline -- --emit-llvm -o $@ $<

$(BUILD)/early.raw.ll: early_scan.wf | prepare
	env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		$(CARGO) run --manifest-path $(ROOT)/compiler/Cargo.toml \
		--bin whitefootc --locked --offline -- --emit-llvm -o $@ $<

$(BUILD)/full.exposed.ll: $(BUILD)/full.raw.ll $(BUILD)/expose
	$(BUILD)/expose $< $@

$(BUILD)/early.exposed.ll: $(BUILD)/early.raw.ll $(BUILD)/expose
	$(BUILD)/expose $< $@

$(BUILD)/wf-full.o: $(BUILD)/full.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -c $< -o $@

$(BUILD)/wf-early.o: $(BUILD)/early.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -c $< -o $@

$(BUILD)/harness-full.o: harness.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -DWF_FULL_SCAN -c $< -o $@

$(BUILD)/harness-early.o: harness.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -DWF_EARLY_SCAN -c $< -o $@

$(BUILD)/whitefoot-full: $(BUILD)/harness-full.o $(BUILD)/wf-full.o
	$(CLANG) -O2 $^ -o $@

$(BUILD)/whitefoot-early: $(BUILD)/harness-early.o $(BUILD)/wf-early.o
	$(CLANG) -O2 $^ -o $@

$(BUILD)/c-full.o: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -DWF_FULL_SCAN -c $< -o $@

$(BUILD)/c-early.o: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -DWF_EARLY_SCAN -c $< -o $@

$(BUILD)/c-full: $(BUILD)/harness-full.o $(BUILD)/c-full.o
	$(CLANG) -O2 $^ -o $@

$(BUILD)/c-early: $(BUILD)/harness-early.o $(BUILD)/c-early.o
	$(CLANG) -O2 $^ -o $@

$(BUILD)/libwf_scan_control.rlib: rust_kernel.rs | prepare
	$(RUSTC) $(COMMON_RUST) --crate-name wf_scan_control --crate-type rlib $< -o $@

$(BUILD)/rust-full: rust_harness.rs $(BUILD)/libwf_scan_control.rlib
	$(RUSTC) $(COMMON_RUST) --cfg full_scan $< \
		--extern wf_scan_control=$(BUILD)/libwf_scan_control.rlib -o $@

$(BUILD)/rust-early: rust_harness.rs $(BUILD)/libwf_scan_control.rlib
	$(RUSTC) $(COMMON_RUST) --cfg early_scan $< \
		--extern wf_scan_control=$(BUILD)/libwf_scan_control.rlib -o $@

$(BUILD)/wf-full.opt.ll: $(BUILD)/full.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/wf-early.opt.ll: $(BUILD)/early.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/wf-full.s: $(BUILD)/full.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S $< -o $@

$(BUILD)/wf-early.s: $(BUILD)/early.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S $< -o $@

$(BUILD)/c-full.s: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -DWF_FULL_SCAN -S $< -o $@

$(BUILD)/c-early.s: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -DWF_EARLY_SCAN -S $< -o $@

$(BUILD)/rust-kernel.s: rust_kernel.rs | prepare
	$(RUSTC) $(COMMON_RUST) --crate-name wf_scan_control --crate-type lib \
		--emit=asm $< -o $@
