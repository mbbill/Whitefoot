# WFGREP-BASELINE build and phase driver. Phases run in protocol order:
#   build -> gen -> verify -> null-before -> bench -> null-after -> counters
# Profiling is driven per PROTOCOL.md after the bench summaries exist.
#
# This driver is the record of the single run of 2026-08-06 (RESULTS.md),
# retained with the frozen protocol. It is not replayable from HEAD: the
# subject it measured is the tests/programs/wfgrep.wf of that day (its
# digest is pinned in PROTOCOL.md), and that program became a recursive
# search printing PATH:LINE:TEXT lines on 2026-08-18, so `verify` no longer
# matches the pins and the timed phases would measure a different program.
# MANIFEST.txt stays the pinned corpus and output identity that
# ../wfgrep-double-walk/ still verifies its shape sources against. A replay
# writes its JSON lines to the scratch root, never to the committed raw/
# evidence of the frozen run.

ROOT := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-wfgrep-baseline
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/cargo-target
BUNDLE := $(abspath .)
RAW ?= $(WORK_ROOT)/replay.jsonl
ENV := WFB_WORK_ROOT=$(WORK_ROOT) WFB_MANIFEST=$(BUNDLE)/MANIFEST.txt WFB_RAW=$(RAW)
CLANG := /usr/bin/clang
RUSTC := rustc

.PHONY: build prepare gen manifest verify null-before bench null-after counters check

build: prepare $(BUILD)/wfgrep $(BUILD)/wfgrep.raw.ll $(BUILD)/wfgrep.opt.ll $(BUILD)/wfgrep.s $(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET)

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

# Attribution profiling: make profile CASE=many REPS=15 SECONDS=6 BIN=wfgrep
profile: build
	$(ENV) $(BUILD)/runner profile $(CASE) $(REPS) $(SECONDS) $(BIN)

check: verify
