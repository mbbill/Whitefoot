# WFGREP-DOUBLE-WALK build and phase driver. Phases run in protocol order:
#   build -> gen -> verify -> code-shape -> null-before -> bench-grep ->
#   bench-s1 -> bench-s2 -> bench-s3 -> null-after -> counters
#   [-> confirm SHAPE=sN, conditional per PROTOCOL.md]

ROOT := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-wfgrep-double-walk
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/cargo-target
BUNDLE := $(abspath .)
RAW ?= $(BUNDLE)/raw/wfgrep-double-walk-1.jsonl
ENV := WFD_WORK_ROOT=$(WORK_ROOT) WFD_MANIFEST=$(BUNDLE)/../wfgrep-baseline/MANIFEST.txt WFD_RAW=$(RAW)
CLANG := /usr/bin/clang
RUSTC := rustc

SHAPE_SOURCES := s1-hoisted-first-byte s2-fused-scan-match s3-swar-newline-scan
BINARIES := $(BUILD)/base $(BUILD)/s1 $(BUILD)/s2 $(BUILD)/s3
ARTIFACTS := $(foreach name,base s1 s2 s3,$(BUILD)/$(name).raw.ll $(BUILD)/$(name).opt.ll $(BUILD)/$(name).s)

.PHONY: build prepare gen verify code-shape null-before null-after counters check \
        bench-grep bench-s1 bench-s2 bench-s3 confirm

build: prepare $(BINARIES) $(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET) $(BUNDLE)/raw

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

source_of = $(if $(filter base,$(1)),$(ROOT)/tests/programs/wfgrep.wf,$(BUNDLE)/shapes/$(filter $(1)-%,$(SHAPE_SOURCES)).wf)

define BUILD_SUBJECT
$(BUILD)/$(1): $(call source_of,$(1)) | prepare
	cd $(ROOT)/compiler && env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		cargo run --bin whitefootc --locked --offline -- -o $$@ $$<

$(BUILD)/$(1).raw.ll: $(call source_of,$(1)) | prepare
	cd $(ROOT)/compiler && env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(CARGO_TARGET) \
		cargo run --bin whitefootc --locked --offline -- --emit-llvm -o $$@ $$<

$(BUILD)/$(1).opt.ll: $(BUILD)/$(1).raw.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $$< -o $$@

$(BUILD)/$(1).s: $(BUILD)/$(1).raw.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S $$< -o $$@
endef

$(eval $(call BUILD_SUBJECT,base))
$(eval $(call BUILD_SUBJECT,s1))
$(eval $(call BUILD_SUBJECT,s2))
$(eval $(call BUILD_SUBJECT,s3))

gen: build
	$(ENV) $(BUILD)/runner gen

verify: build
	$(ENV) $(BUILD)/runner verify

# The retained pre-timing artifacts; inspection findings are recorded in
# RESULTS.md against these exact files.
code-shape: build $(ARTIFACTS)
	shasum -a 256 $(ARTIFACTS)

null-before: build
	$(ENV) $(BUILD)/runner null before

bench-grep: build
	$(ENV) $(BUILD)/runner bench-grep

bench-s1: build
	$(ENV) $(BUILD)/runner bench-shape s1

bench-s2: build
	$(ENV) $(BUILD)/runner bench-shape s2

bench-s3: build
	$(ENV) $(BUILD)/runner bench-shape s3

null-after: build
	$(ENV) $(BUILD)/runner null after

counters: build
	$(ENV) $(BUILD)/runner counters

# Conditional confirmation phase: make confirm SHAPE=s2
confirm: build
	$(ENV) $(BUILD)/runner confirm $(SHAPE)

check: verify
