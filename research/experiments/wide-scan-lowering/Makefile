# WIDE-SCAN-LOWERING build and phase driver. Phases run in protocol order:
#   build -> gen -> verify -> code-shape -> null-before -> bench-grep ->
#   bench-wide -> null-after -> counters -> confirm
#
# Both subjects are the same tests/programs/wfgrep.wf bytes; `base` is
# built by the base-revision compiler (BASE_ROOT, pinned at the task's
# base revision) and `wide` by the candidate compiler (this repository).
#
# This driver is the record of the single run of 2026-08-06 (RESULTS.md),
# retained with the frozen protocol. It is not replayable from HEAD: both
# subjects were the tests/programs/wfgrep.wf of that day, which became a
# recursive search printing PATH:LINE:TEXT lines on 2026-08-18, and `base`
# needs the base-revision compiler checked out at BASE_ROOT. A replay writes
# its JSON lines to the scratch root, never to the committed raw/ evidence
# of the frozen run.

ROOT := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
BASE_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/wf-0026-base
WORK_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-wide-scan-lowering
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
CARGO_TARGET := $(WORK_ROOT)/cargo-target
BASE_CARGO_TARGET := $(WORK_ROOT)/cargo-target-base
BUNDLE := $(abspath .)
RAW ?= $(WORK_ROOT)/replay.jsonl
ENV := WFD_WORK_ROOT=$(WORK_ROOT) WFD_MANIFEST=$(BUNDLE)/../wfgrep-baseline/MANIFEST.txt WFD_RAW=$(RAW)
CLANG := /usr/bin/clang
RUSTC := rustc

BINARIES := $(BUILD)/base $(BUILD)/wide
ARTIFACTS := $(foreach name,base wide,$(BUILD)/$(name).raw.ll $(BUILD)/$(name).opt.ll $(BUILD)/$(name).s)

.PHONY: build prepare gen verify code-shape null-before null-after counters check \
        bench-grep bench-wide confirm

build: prepare $(BINARIES) $(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP) $(CARGO_TARGET) $(BASE_CARGO_TARGET)

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

root_of = $(if $(filter base,$(1)),$(BASE_ROOT),$(ROOT))
target_of = $(if $(filter base,$(1)),$(BASE_CARGO_TARGET),$(CARGO_TARGET))

define BUILD_SUBJECT
$(BUILD)/$(1): $(call root_of,$(1))/tests/programs/wfgrep.wf | prepare
	cd $(call root_of,$(1))/compiler && env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(call target_of,$(1)) \
		cargo run --bin whitefootc --locked --offline -- -o $$@ $$<

$(BUILD)/$(1).raw.ll: $(call root_of,$(1))/tests/programs/wfgrep.wf | prepare
	cd $(call root_of,$(1))/compiler && env TMPDIR=$(TMP) CARGO_TARGET_DIR=$(call target_of,$(1)) \
		cargo run --bin whitefootc --locked --offline -- --emit-llvm -o $$@ $$<

$(BUILD)/$(1).opt.ll: $(BUILD)/$(1).raw.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S -emit-llvm $$< -o $$@

$(BUILD)/$(1).s: $(BUILD)/$(1).raw.ll
	$(CLANG) -O2 -Wno-override-module -x ir -S $$< -o $$@
endef

$(eval $(call BUILD_SUBJECT,base))
$(eval $(call BUILD_SUBJECT,wide))

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

bench-wide: build
	$(ENV) $(BUILD)/runner bench-shape wide

null-after: build
	$(ENV) $(BUILD)/runner null after

counters: build
	$(ENV) $(BUILD)/runner counters

confirm: build
	$(ENV) $(BUILD)/runner confirm wide

check: verify
