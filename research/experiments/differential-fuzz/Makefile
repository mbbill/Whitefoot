# Differential fuzzing of the overlap lowerings.
#
# One generator writes accepted Whitefoot command programs that do real I/O and
# control flow; one oracle compiles each program under every overlap lowering
# and runs it across the worker and helper matrix, comparing published bytes
# and exit status against the program's own sequential run.
#
# This is an evidence campaign, not a gate. It is deliberately unreachable from
# the repository's `make check`: it builds thousands of executables, runs for
# minutes to hours, and its verdict is a report rather than a pass/fail a build
# should depend on.
#
#   make -C research/experiments/differential-fuzz smoke      a 20-program run
#   make -C research/experiments/differential-fuzz campaign    the full campaign
#   make -C research/experiments/differential-fuzz probes       the recorded probes
#   make -C research/experiments/differential-fuzz lint         fmt + clippy
#
# Everything generated -- the fixture tree, the compiled programs, the raw run
# output, and the findings -- stays under the scratch root. Only the generator,
# the oracle, the recorded probe programs, and the campaign report are tracked.

ROOT := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-differential-fuzz
CARGO_TARGET_DIR ?= $(WORK)/cargo-target
CARGO := cargo
WFC ?= $(ROOT)/compiler/target/gate/whitefootc
FUZZ := $(CARGO_TARGET_DIR)/release/wf-difffuzz

# Campaign shape. PROGRAMS counts accepted programs, not attempts; BUDGET caps
# the wall clock so a shared machine is never held hostage. NICE keeps the whole
# campaign out of the way of interactive work.
PROGRAMS ?= 2000
BUDGET ?= 5400
JOBS ?= 4
SEED ?= 1
REPS ?= 2
NICE ?= 19

export CARGO_TARGET_DIR

build:
	$(CARGO) build --release --locked --offline

$(FUZZ): build

# The compiler the campaign differentiates. Built at the same profile the
# repository gate uses, because that is the compiler whose lowerings are under
# test.
compiler:
	$(CARGO) build --manifest-path $(ROOT)/compiler/Cargo.toml \
		--profile gate --bin whitefootc --locked --offline

smoke: build compiler
	@mkdir -p "$(WORK)"
	nice -n $(NICE) $(FUZZ) campaign --whitefootc "$(WFC)" --work "$(WORK)/smoke" \
		--programs 20 --budget 600 --jobs $(JOBS) --seed $(SEED) --reps 1

campaign: build compiler
	@mkdir -p "$(WORK)"
	nice -n $(NICE) $(FUZZ) campaign --whitefootc "$(WFC)" --work "$(WORK)/campaign" \
		--programs $(PROGRAMS) --budget $(BUDGET) --jobs $(JOBS) --seed $(SEED) \
		--reps $(REPS)

# Re-runs every recorded probe program under the same oracle that found it.
# A probe that stops reproducing is the signal that its defect was fixed, and
# the probe is then either promoted to a compiler test or deleted.
probes: build compiler
	@mkdir -p "$(WORK)"
	nice -n $(NICE) $(FUZZ) probes --whitefootc "$(WFC)" --work "$(WORK)/probes" \
		--directory probes --reps $(REPS)

# One generated program on stdout, for reading what the generator emits.
sample: build
	$(FUZZ) generate --seed $(SEED)

lint:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --locked --offline -- -D warnings

clean:
	rm -rf "$(WORK)"

.PHONY: build compiler smoke campaign probes sample lint clean
