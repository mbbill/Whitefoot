# WFGREP-DOUBLE-WALK: the frozen run's record and the shapes' replay check.
#
# The single run of 2026-08-06 (RESULTS.md) paired three candidate source
# shapes against B0, the then-current tests/programs/wfgrep.wf, through the
# phases build -> gen -> verify -> code-shape -> null -> bench -> counters
# -> confirm. B0 cannot be rebuilt from HEAD: that program became a
# recursive search printing PATH:LINE:TEXT lines on 2026-08-18, and its
# frozen bytes predate specification v0.40. The comparative phases therefore
# live only in the freeze commit RESULTS.md names. This driver keeps what
# HEAD can still assert: the three shape sources, kept on the active
# specification, compile through the ordinary compiler and reproduce the
# inherited WFGREP-BASELINE manifest byte for byte.
#
#   make -C research/experiments/wfgrep-double-walk check
#
# The corpus and binaries live in the scratch root. Nothing here writes to
# the committed raw/ evidence of the frozen run.

ROOT := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-wfgrep-double-walk
BUILD := $(WORK_ROOT)/build
TMP := $(WORK_ROOT)/tmp
BUNDLE := $(abspath .)
ENV := WFD_WORK_ROOT=$(WORK_ROOT) WFD_MANIFEST=$(BUNDLE)/../wfgrep-baseline/MANIFEST.txt
RUSTC := rustc
WFC := $(ROOT)/compiler/target/gate/whitefootc

# The corpus is regenerated only when this file is missing; `verify` then
# checks every corpus digest against the pinned manifest, so a stale or
# partial corpus fails loudly and `make gen` rebuilds it.
CORPUS_STAMP := $(WORK_ROOT)/corpus/floor/empty.txt

SHAPES := s1 s2 s3
BINARIES := $(foreach name,$(SHAPES),$(BUILD)/$(name))

.PHONY: build prepare compiler gen verify check

build: prepare compiler $(BINARIES) $(BUILD)/runner

prepare:
	mkdir -p $(BUILD) $(TMP)

# The gate profile is the compiler's own optimized build; the program it
# emits does not depend on that profile, and the dev profile takes minutes
# per shape.
compiler:
	cd $(ROOT)/compiler && env CARGO_TARGET_DIR=$(ROOT)/compiler/target \
		cargo build --profile gate --bin whitefootc --locked --offline

$(BUILD)/runner: runner.rs | prepare
	$(RUSTC) --edition=2024 -C opt-level=2 -C panic=abort $< -o $@

$(BUILD)/s1: shapes/s1-hoisted-first-byte.wf compiler | prepare
	env TMPDIR=$(TMP) $(WFC) -o $@ $<

$(BUILD)/s2: shapes/s2-fused-scan-match.wf compiler | prepare
	env TMPDIR=$(TMP) $(WFC) -o $@ $<

$(BUILD)/s3: shapes/s3-swar-newline-scan.wf compiler | prepare
	env TMPDIR=$(TMP) $(WFC) -o $@ $<

$(CORPUS_STAMP): | $(BUILD)/runner
	$(ENV) $(BUILD)/runner gen

gen: $(BUILD)/runner
	$(ENV) $(BUILD)/runner gen

verify: build $(CORPUS_STAMP)
	$(ENV) $(BUILD)/runner verify

check: verify
