# WFGREP-BASELINE build and phase driver. Phases run in protocol order:
#   build -> gen -> verify -> null-before -> bench -> null-after -> counters
# Profiling is driven per PROTOCOL.md after the bench summaries exist.

ROOT := $(abspath ../../..)
WORK_ROOT ?= /Users/bytedance/do_not_scan/whitefoot-wfgrep-baseline
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/cargo-target
BUNDLE := $(abspath .)
RAW ?= $(BUNDLE)/raw/wfgrep-baseline-1.jsonl
ENV := WFB_WORK_ROOT=$(WORK_ROOT) WFB_MANIFEST=$(BUNDLE)/MANIFEST.txt WFB_RAW=$(RAW)
CLANG := /usr/bin/clang
RUSTC := rustc

.PHONY: build prepare gen manifest verify null-before bench null-after counters check

build: prepare $(BUILD)/wfgrep $(BUILD)/wfgrep.raw.ll $(BUILD)/wfgrep.opt.ll $(BUILD)/wfgrep.s $(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET) $(BUNDLE)/raw

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/wfgrep: $(ROOT)/tests/programs/wfgrep.wf | prepare
	cd $(ROOT)/compiler && env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		cargo run --bin whitefootc --locked --offline -- -o $@ $<

$(BUILD)/wfgrep.raw.ll: $(ROOT)/tests/programs/wfgrep.wf | prepare
	cd $(ROOT)/compiler && env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		cargo run --bin whitefootc --locked --offline -- --emit-llvm -o $@ $<

$(BUILD)/wfgrep.opt.ll: $(BUILD)/wfgrep.raw.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $< -o $@

$(BUILD)/wfgrep.s: $(BUILD)/wfgrep.raw.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S $< -o $@

gen: build
	$(ENV) $(BUILD)/runner gen

manifest: build
	$(ENV) $(BUILD)/runner gen --print-manifest

verify: build
	$(ENV) $(BUILD)/runner verify

null-before: build
	$(ENV) $(BUILD)/runner null before

bench: build
	$(ENV) $(BUILD)/runner bench

null-after: build
	$(ENV) $(BUILD)/runner null after

counters: build
	$(ENV) $(BUILD)/runner counters

check: verify
