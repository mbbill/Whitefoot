ROOT := $(abspath ../../..)
EXPERIMENT := $(abspath .)
WORK_ROOT ?= /Users/bytedance/do_not_scan/whitefoot-literal-line-floor
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/compiler-target
CEILING_TARGET := $(WORK_ROOT)/ceiling-target
INPUT_DIR := $(WORK_ROOT)/input
INPUT := $(INPUT_DIR)/ru-prefix-67108864.bin
SOURCE_INPUT ?= /Users/bytedance/do_not_scan/whitefoot-rg-base/corpora/ru.txt
NEEDLE_HEX := d0a8d0b5d180d0bbd0bed0ba20d0a5d0bed0bbd0bcd181
CLANG := /usr/bin/clang
RUSTC := rustc
CARGO := cargo
COMMON_C := -O2 -std=c17 -Wall -Wextra -Werror
COMMON_RUST := --edition=2024 -C opt-level=2 -C panic=abort -C codegen-units=1 -C lto=no

.PHONY: check bench build prepare prepare-input

check: build prepare-input
	WF_LITERAL_LINE_WORK_ROOT=$(WORK_ROOT) $(BUILD)/runner verify

bench: check
	WF_LITERAL_LINE_WORK_ROOT=$(WORK_ROOT) $(BUILD)/runner bench --freeze-commit $(FREEZE_COMMIT)

build: prepare \
	$(BUILD)/whitefoot $(BUILD)/c $(BUILD)/naive-rust $(BUILD)/memmem-rust \
	$(BUILD)/wf.raw.ll $(BUILD)/wf.opt.ll $(BUILD)/wf.s \
	$(BUILD)/c.ll $(BUILD)/c.s $(BUILD)/naive-rust.ll $(BUILD)/naive-rust.s \
	$(BUILD)/memmem-rust.ll $(BUILD)/memmem-rust.s $(BUILD)/memmem-rust.disasm $(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET) $(CEILING_TARGET) $(INPUT_DIR)

prepare-input: $(BUILD)/runner | prepare
	$(BUILD)/runner prepare-input $(SOURCE_INPUT) $(INPUT)

$(BUILD)/expose: expose.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/wf.raw.ll: literal_line.wf | prepare
	env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		$(CARGO) run --manifest-path $(ROOT)/compiler/Cargo.toml \
		--bin whitefootc --locked --offline -- --emit-llvm -o $@ $<

$(BUILD)/wf.exposed.ll: $(BUILD)/wf.raw.ll $(BUILD)/expose
	$(BUILD)/expose $< $@

$(BUILD)/wf.o: $(BUILD)/wf.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -c $< -o $@

$(BUILD)/harness-wf.o: harness.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -c $< -o $@

$(BUILD)/whitefoot: $(BUILD)/harness-wf.o $(BUILD)/wf.o
	$(CLANG) -O2 $^ -o $@

$(BUILD)/c-kernel.o: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -c $< -o $@

$(BUILD)/harness-c.o: harness.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -c $< -o $@

$(BUILD)/c: $(BUILD)/harness-c.o $(BUILD)/c-kernel.o
	$(CLANG) -O2 $^ -o $@

$(BUILD)/libwf_literal_line_control.rlib: rust_kernel.rs | prepare
	$(RUSTC) $(COMMON_RUST) --crate-name wf_literal_line_control --crate-type rlib $< -o $@

$(BUILD)/naive-rust: rust_harness.rs harness_common.rs $(BUILD)/libwf_literal_line_control.rlib
	$(RUSTC) $(COMMON_RUST) $< \
		--extern wf_literal_line_control=$(BUILD)/libwf_literal_line_control.rlib -o $@

$(BUILD)/memmem-rust: ceiling/Cargo.toml ceiling/Cargo.lock ceiling/src/lib.rs ceiling/src/main.rs harness_common.rs | prepare
	env TMPDIR=$(TMP) $(CARGO) build --manifest-path ceiling/Cargo.toml \
		--target-dir $(CEILING_TARGET) --release --locked --offline
	cp $(CEILING_TARGET)/release/wf-literal-line-ceiling $@

$(BUILD)/wf.opt.ll: $(BUILD)/wf.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/wf.s: $(BUILD)/wf.exposed.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S $< -o $@

$(BUILD)/c.ll: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -S -emit-llvm $< -o $@

$(BUILD)/c.s: control.c kernel.h | prepare
	$(CLANG) $(COMMON_C) -S $< -o $@

$(BUILD)/naive-rust.ll $(BUILD)/naive-rust.s: rust_kernel.rs | prepare
	$(RUSTC) $(COMMON_RUST) --crate-name wf_literal_line_control --crate-type lib \
		--emit=llvm-ir=$(BUILD)/naive-rust.ll,asm=$(BUILD)/naive-rust.s $<

$(BUILD)/memmem-rust.ll $(BUILD)/memmem-rust.s: ceiling/Cargo.toml ceiling/Cargo.lock ceiling/src/lib.rs | prepare
	env TMPDIR=$(TMP) $(CARGO) rustc --manifest-path ceiling/Cargo.toml \
		--target-dir $(CEILING_TARGET) --lib --release --locked --offline -- \
		--emit=llvm-ir=$(BUILD)/memmem-rust.ll,asm=$(BUILD)/memmem-rust.s

$(BUILD)/memmem-rust.disasm: $(BUILD)/memmem-rust
	/usr/bin/otool -tvV $< > $@
