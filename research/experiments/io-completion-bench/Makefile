# Program-level I/O measurement for the unified-state completion model.
#
# Three lines per workload, all publishing the same bytes:
#   N  hand-written native candidates (thread pool, io_uring)
#   S  the Whitefoot program built with --no-overlap, the sequential reference
#   C  the Whitefoot program built the way it ships, with completion overlap
#
# Timing targets are not a gate: they generate large trees, run for minutes,
# and report timings that no build should depend on. Explicit scheduler and
# Rayon correctness targets are reached by the repository's `make check`.
#
#   make -C research/experiments/io-completion-bench verify   correctness only
#   make -C research/experiments/io-completion-bench bench     the macOS table
#   make -C research/experiments/io-completion-bench bench-pipe
#   make -C research/experiments/io-completion-bench linux     the Linux table
#
# Generated trees, binaries, and raw output stay in the scratch root. Only the
# summarized table is committed, in
# research/investigations/io-model/RESULTS.md.

ROOT := $(abspath ../../..)
.DEFAULT_GOAL := build

# Scheduler experiment: exhaustive correctness is a canonical check stage;
# timings run separately on a Linux CI host and never select acceptance.
.PHONY: scheduler-check scheduler-bench scheduler-mixed scheduler-fairness scheduler-inline scheduler-sustain scheduler-checkpoint scheduler-footprint scheduler-paced scheduler-chunks scheduler-canonical scheduler-stackful scheduler-stackful-paced scheduler-nodelay scheduler-owner scheduler-owner-paced scheduler-memory scheduler-dispatch scheduler-dispatch-paced scheduler-wake scheduler-wake-paced scheduler-service scheduler-service-paced scheduler-storage scheduler-pages stackful-check
scheduler-check:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" bash scheduler-bench.sh check

.PHONY: scheduler-enumeration-check scheduler-stream-check
scheduler-enumeration-check:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" bash scheduler-bench.sh check-enumeration

scheduler-stream-check:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" bash scheduler-bench.sh check-streams

scheduler-bench:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" bash scheduler-bench.sh bench

scheduler-mixed:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=mixed bash scheduler-bench.sh bench

scheduler-fairness:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=fairness bash scheduler-bench.sh bench

scheduler-inline:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=inline bash scheduler-bench.sh bench

scheduler-sustain:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=sustain bash scheduler-bench.sh bench

scheduler-checkpoint:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=checkpoint bash scheduler-bench.sh bench

scheduler-footprint:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=footprint bash scheduler-bench.sh bench

scheduler-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=paced bash scheduler-bench.sh bench

scheduler-chunks:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=chunks bash scheduler-bench.sh bench

scheduler-canonical:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=canonical bash scheduler-bench.sh bench

scheduler-stackful:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=stackful bash scheduler-bench.sh bench

scheduler-stackful-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=stackful-paced bash scheduler-bench.sh bench

scheduler-nodelay:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=nodelay bash scheduler-bench.sh bench

scheduler-owner:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=owner bash scheduler-bench.sh bench

scheduler-owner-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=owner-paced bash scheduler-bench.sh bench

scheduler-memory:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=memory bash scheduler-bench.sh bench

scheduler-dispatch:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=dispatch bash scheduler-bench.sh bench

scheduler-dispatch-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=dispatch-paced bash scheduler-bench.sh bench

scheduler-wake:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=wake bash scheduler-bench.sh bench

scheduler-wake-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=wake-paced bash scheduler-bench.sh bench

scheduler-service:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=service bash scheduler-bench.sh bench

scheduler-service-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=service-paced bash scheduler-bench.sh bench

scheduler-storage:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=storage bash scheduler-bench.sh bench

scheduler-pages:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=pages bash scheduler-bench.sh bench

.PHONY: scheduler-native-baselines
scheduler-native-baselines:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" NATIVE_BASELINES=2 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-go-screen
scheduler-go-screen:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" GO="$(GO)" CORO_CXX="$(CORO_CXX)" GO_SCREEN=1 CONTINUATION_SCREEN=3 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-continuation-index
scheduler-continuation-index:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" CONTINUATION_SCREEN=4 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-client-headroom
scheduler-client-headroom:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" CLIENT_HEADROOM=1 NATIVE_BASELINES=2 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-uring-diagnostic
scheduler-uring-diagnostic:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=3 WARMUP=1 CORO_CXX="$(CORO_CXX)" URING_DIAGNOSTIC=1 NATIVE_BASELINES=2 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-client-diagnostic
scheduler-client-diagnostic:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=3 WARMUP=1 CORO_CXX="$(CORO_CXX)" CLIENT_DIAGNOSTIC=1 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-client-readiness
scheduler-client-readiness:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=5 WARMUP=1 CORO_CXX="$(CORO_CXX)" CLIENT_READINESS=1 CONTINUATION_SCREEN=4 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-client-capacity
scheduler-client-capacity:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=5 WARMUP=1 CORO_CXX="$(CORO_CXX)" CLIENT_CAPACITY=1 CONTINUATION_SCREEN=4 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-native-frontier
scheduler-native-frontier:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=5 WARMUP=1 CORO_CXX="$(CORO_CXX)" NATIVE_FRONTIER=1 CONTINUATION_SCREEN=4 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-storage-control
scheduler-storage-control:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=5 WARMUP=1 CORO_CXX="$(CORO_CXX)" STORAGE_CONTROL=1 CONTINUATION_SCREEN=4 NATIVE_BASELINES=1 EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-allocation
scheduler-allocation:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=allocation bash scheduler-bench.sh bench

.PHONY: scheduler-allocator
scheduler-allocator:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" EXPERIMENT=allocator bash scheduler-bench.sh bench

.PHONY: scheduler-combined
scheduler-combined:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" EXPERIMENT=allocator bash scheduler-bench.sh combine

.PHONY: scheduler-coroutine scheduler-coroutine-paced
scheduler-coroutine:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" EXPERIMENT=coroutine bash scheduler-bench.sh bench

scheduler-coroutine-paced:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" EXPERIMENT=coroutine-paced bash scheduler-bench.sh bench

.PHONY: scheduler-profile
scheduler-profile:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS=3 WARMUP=0 CORO_CXX="$(CORO_CXX)" EXPERIMENT=coroutine-paced bash scheduler-bench.sh profile

.PHONY: scheduler-client-service
scheduler-client-service:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" EXPERIMENT=coroutine-paced bash scheduler-bench.sh client

.PHONY: scheduler-placement
scheduler-placement:
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" CORO_CXX="$(CORO_CXX)" EXPERIMENT=coroutine-paced bash scheduler-bench.sh placement

WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-io-completion-bench
BUILD := $(WORK)/build
TREE := $(WORK)/tree
BIGTREE := $(WORK)/bigtree
CARGO_TARGET := $(WORK)/cargo-target
LINUX_TARGET := $(WORK)/linux-target
CLANG := /usr/bin/clang
CFLAGS := -std=c11 -O2 -Wall -Wextra -Werror
CORO_CXX ?= $(shell command -v clang++-20 || command -v clang++)
CORO_CLANG ?= $(shell command -v clang-20 || command -v clang)
CORO_FLAGS := -std=c++20 -O2 -Wall -Wextra -Werror -Wpedantic -fno-exceptions -pthread
COMPLETION_CORO_SANITIZERS ?= address,undefined
COMPLETION_CORO_ARGS ?=
CONTINUATION_RUNTIME_UNITS := sched/core.c sched/prim_host.c sched/entry.c \
	completion/runtime.c completion/wait_host.c completion/file_adapter.c \
	completion/file_posix.c completion/bridge.c completion/linux_io_uring.c
WFC := $(ROOT)/compiler/target/gate/whitefootc

# Independent CPU reference: no Rayon dependency enters the compiler crate.
RAYON_CARGO_TARGET ?= $(WORK)/rayon-target
.PHONY: rayon-check rayon-bench rayon-resource-bench rayon-ring-bench rayon-startup-bench rayon-idle-bench rayon-build-bench rayon-grain-bench mixed-rayon-check mixed-rayon-smoke
rayon-check:
	cargo fmt --manifest-path rayon-baseline/Cargo.toml -- --check
	CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" cargo test --profile gate --locked --offline --manifest-path rayon-baseline/Cargo.toml
	CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" cargo clippy --profile gate --all-targets --locked --offline --manifest-path rayon-baseline/Cargo.toml -- -D warnings

rayon-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" bash rayon-bench.sh

rayon-resource-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" RESOURCE_CONTROLS=1 bash rayon-bench.sh

rayon-ring-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" RESOURCE_CONTROLS=2 bash rayon-bench.sh

rayon-startup-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" RESOURCE_CONTROLS=3 bash rayon-bench.sh

rayon-idle-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" RESOURCE_CONTROLS=4 bash rayon-bench.sh

rayon-build-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" RESOURCE_CONTROLS=5 bash rayon-bench.sh

rayon-grain-bench: compiler rayon-check
	ROOT="$(ROOT)" WFC="$(WFC)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" ROUNDS="$(ROUNDS)" WARMUP="$(WARMUP)" RESOURCE_CONTROLS=6 bash rayon-bench.sh

# Optional mixed feature does not enter the CPU-only binary. Its correctness
# includes real bounded-offload saturation and the existing external oracle.
mixed-rayon-check: prepare
	cargo fmt --manifest-path rayon-baseline/Cargo.toml -- --check
	CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" cargo test --profile gate --features mixed --bin mixed-rayon --locked --offline --manifest-path rayon-baseline/Cargo.toml
	CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" cargo clippy --profile gate --all-targets --all-features --locked --offline --manifest-path rayon-baseline/Cargo.toml -- -D warnings
	CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" cargo build --profile gate --features mixed --bin mixed-rayon --locked --offline --manifest-path rayon-baseline/Cargo.toml
	$(CLANG) $(CFLAGS) -Wpedantic -pthread stream_check.c -o "$(BUILD)/stream_check"
	@set -e; for threads in 2 4; do for mode in compute truncated; do \
	  "$(BUILD)/stream_check" "$(RAYON_CARGO_TARGET)/gate/mixed-rayon" "$$mode" "$$threads"; \
	done; done

mixed-rayon-smoke: mixed-rayon-check
	ROOT="$(ROOT)" WHITEFOOT_SCRATCH_ROOT="$(WHITEFOOT_SCRATCH_ROOT)" CARGO_TARGET_DIR="$(RAYON_CARGO_TARGET)" CLANG="$(CLANG)" bash mixed-rayon-smoke.sh

# The workload's shape. FILES x up to MAX_KIB, read through one positioned
# read each. The checksum below is a property of these two numbers alone, so
# changing either means recomputing it with `make expected`.
FILES ?= 8192
MAX_KIB ?= 16
EXPECTED ?= 17098009301725298919 00000000000071024640

# Timing protocol: WARMUP unrecorded runs, then ROUNDS recorded ones.
ROUNDS ?= 7
WARMUP ?= 2

# The pipe workload. Chunks are larger than the host pipe buffer so a write
# genuinely waits for its consumer, and each consumer sleeps DELAY_US per read
# so the consumer, not the producer's CPU, sets the pace.
PIPE_ROUNDS ?= 64
PIPE_DELAY_US ?= 1000
PIPE_BYTES ?= 16777216


# The read-heavy workload's shape: READ_FILES files of READ_KIB KiB each,
# opened once and then read at READS_* positioned offsets. Its tree is
# separate from the many-files tree above because the two answer different
# questions and neither should perturb the other's page-cache state.
READ_FILES ?= 8
READ_KIB ?= 65536
# Both windows do the same number of reads, so the only thing that differs
# between the two tables is the size of one transfer. An uncached read costs
# about the same at either window on this host -- it is a device round trip,
# not a bandwidth question -- so equal read counts also keep the two tables
# roughly equal in wall time.
READS_64K ?= 32768
READS_4K ?= 32768
EXPECTED_READ_64K ?= 09425560703784339892 00000000002147483648
EXPECTED_READ_4K ?= 18028327385673861873 00000000000134217728
READ_NAME_ARGS := f00000.dat f00001.dat f00002.dat f00003.dat \
	f00004.dat f00005.dat f00006.dat f00007.dat
READ_NAME_PLAN_ARGS := f00000.dat\tf00001.dat\tf00002.dat\tf00003.dat\tf00004.dat\tf00005.dat\tf00006.dat\tf00007.dat

# The read table is measured to medians of fifteen on macOS, because the host
# is shared with an endpoint-security stack whose interference is bursty, and
# to medians of nine on Linux.
READ_ROUNDS ?= 15
READ_LINUX_ROUNDS ?= 9

# The residency check that gives the words "uncached" and "warm" their
# meaning. Immediately before and immediately after each table, READ_PROBES
# positioned reads are timed in every file through a descriptor that does not
# populate the cache, and the table is refused unless all but
# READ_TOLERANCE_PERCENT of those reads fell on the side of the threshold the
# label claims. On this host a cached 64 KiB read costs 6 to 20 us and one the
# device answers costs about 134 us, so 40 us separates the two populations
# with room on both sides. read_baseline.c explains why the rule bounds a
# share rather than testing every file.
READ_PROBES ?= 16
READ_THRESHOLD_US ?= 40
READ_TOLERANCE_PERCENT ?= 10

# How long a freshly generated tree is left alone before it may be measured.
# Writing half a gigabyte makes the host's malware scanner walk the new files
# once, which holds a few of them resident for a minute or two; `read-settle`
# waits that out rather than measuring through it. See the comment above
# `read-settle` for the observation.
READ_SETTLE_ATTEMPTS ?= 24
READ_SETTLE_SECONDS ?= 15
DOCKER_IMAGE ?= wf-io-bench:linux

.PHONY: build prepare compiler gen verify bench bench-pipe linux linux-image expected clean chain \
	read-build read-gen read-uncache read-settle read-warm read-verify expected-read bench-read \
	linux-read net-tools net-verify linux-net

build: prepare compiler $(BUILD)/gen $(BUILD)/baseline $(BUILD)/runner \
	$(BUILD)/pipe_producer $(BUILD)/pipe_harness \
	$(BUILD)/many_files_wide $(BUILD)/many_files_wide_seq \
	$(BUILD)/many_files_wide8 $(BUILD)/many_files_wide8_seq \
	$(BUILD)/many_files_narrow $(BUILD)/many_files_narrow_seq \
	$(BUILD)/many_files_loop $(BUILD)/many_files_loop_seq \
	$(BUILD)/pipe_relay $(BUILD)/pipe_relay_seq

read-build: prepare compiler $(BUILD)/gen $(BUILD)/read_baseline $(BUILD)/runner \
	$(BUILD)/read_heavy_narrow $(BUILD)/read_heavy_narrow_seq \
	$(BUILD)/read_heavy_wide8 $(BUILD)/read_heavy_wide8_seq \
	$(BUILD)/read_heavy_narrow_4k $(BUILD)/read_heavy_narrow_4k_seq \
	$(BUILD)/read_heavy_wide8_4k $(BUILD)/read_heavy_wide8_4k_seq

prepare:
	@mkdir -p $(BUILD) $(CARGO_TARGET)

compiler:
	cd $(ROOT)/compiler && env CARGO_TARGET_DIR=$(ROOT)/compiler/target \
		cargo build --profile gate --bin whitefootc --locked --offline

$(BUILD)/gen: gen.c workload.h | prepare
	$(CLANG) $(CFLAGS) $< -o $@

$(BUILD)/baseline: baseline.c workload.h uring_baseline.h | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

$(BUILD)/runner: runner.c | prepare
	$(CLANG) $(CFLAGS) $< -o $@

$(BUILD)/pipe_producer: pipe_producer.c | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

$(BUILD)/pipe_harness: pipe_harness.c | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

# The C line and the S line are one source compiled two ways. Nothing else
# differs between them, which is what makes the pair a statement about the
# lowering rather than about two programs.
$(BUILD)/%: programs/%.wf compiler | prepare
	$(WFC) -o $@ $<

$(BUILD)/many_files_wide_seq: programs/many_files_wide.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/many_files_wide8_seq: programs/many_files_wide8.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/many_files_narrow_seq: programs/many_files_narrow.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/many_files_loop_seq: programs/many_files_loop.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/pipe_relay_seq: programs/pipe_relay.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<


$(BUILD)/read_baseline: read_baseline.c workload.h uring_baseline.h | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

$(BUILD)/read_heavy_narrow_seq: programs/read_heavy_narrow.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/read_heavy_wide8_seq: programs/read_heavy_wide8.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/read_heavy_narrow_4k_seq: programs/read_heavy_narrow_4k.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/read_heavy_wide8_4k_seq: programs/read_heavy_wide8_4k.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

gen: $(BUILD)/gen
	@rm -rf $(TREE)
	$(BUILD)/gen $(TREE) $(FILES) $(MAX_KIB)
	@echo "generated $$(ls $(TREE) | wc -l) files in $(TREE)"

# Every line must publish the same bytes before any of them reports a time.
verify: build gen
	@cd $(TREE) && for line in \
		"$(BUILD)/baseline $(TREE) direct $(FILES)" \
		"$(BUILD)/many_files_narrow" \
		"$(BUILD)/many_files_narrow_seq" \
		"$(BUILD)/many_files_loop" \
		"$(BUILD)/many_files_loop_seq" \
		"$(BUILD)/many_files_wide" \
		"$(BUILD)/many_files_wide_seq" \
		"$(BUILD)/many_files_wide8" \
		"$(BUILD)/many_files_wide8_seq"; do \
		got=$$($$line); \
		if [ "$$got" != "$(EXPECTED)" ]; then \
			echo "MISMATCH: $$line published [$$got], expected [$(EXPECTED)]"; \
			exit 1; \
		fi; \
	done
	@echo "all lines publish $(EXPECTED)"

# Prints the checksum the current tree actually folds, for when FILES or
# MAX_KIB move.
expected: $(BUILD)/baseline gen
	@$(BUILD)/baseline $(TREE) direct $(FILES)

bench: verify
	@{ \
	printf 'N.direct\t\t$(BUILD)/baseline\t$(TREE)\tdirect\t$(FILES)\n'; \
	for t in 1 2 4 6 8 10; do \
		printf 'N.pool%s\t\t$(BUILD)/baseline\t$(TREE)\tpool\t$(FILES)\t%s\n' $$t $$t; \
	done; \
	printf 'S.narrow\t\t$(BUILD)/many_files_narrow_seq\n'; \
	printf 'S.loop\t\t$(BUILD)/many_files_loop_seq\n'; \
	printf 'S.wide\t\t$(BUILD)/many_files_wide_seq\n'; \
	printf 'S.wide8\t\t$(BUILD)/many_files_wide8_seq\n'; \
	printf 'C.narrow.default\t\t$(BUILD)/many_files_narrow\n'; \
	printf 'C.loop.default\t\t$(BUILD)/many_files_loop\n'; \
	printf 'C.wide.default\t\t$(BUILD)/many_files_wide\n'; \
	printf 'C.wide8.default\t\t$(BUILD)/many_files_wide8\n'; \
	for w in 0 1 2 4; do for h in 0 1 2 4; do \
		printf 'C.wide.w%s.h%s\tWF_WORKERS=%s,WF_IO_HELPERS=%s\t$(BUILD)/many_files_wide\n' \
			$$w $$h $$w $$h; \
	done; done; \
	for h in 0 1 2 4 6; do \
		printf 'C.wide8.h%s\tWF_IO_HELPERS=%s\t$(BUILD)/many_files_wide8\n' $$h $$h; \
	done; \
	} > $(WORK)/plan.txt
	cd $(TREE) && $(BUILD)/runner $(WORK)/plan.txt $(ROUNDS) $(WARMUP) "$(EXPECTED)"

bench-pipe: build
	@echo "pipe relay: $(PIPE_ROUNDS) rounds, consumer sleeps $(PIPE_DELAY_US)us per read"
	@for line in \
		"N.seq:$(BUILD)/pipe_producer seq $(PIPE_ROUNDS)" \
		"N.threads:$(BUILD)/pipe_producer threads $(PIPE_ROUNDS)" \
		"S.relay:$(BUILD)/pipe_relay_seq" \
		"C.relay:$(BUILD)/pipe_relay"; do \
		label=$${line%%:*}; command=$${line#*:}; \
		for warm in $$(seq 1 $(WARMUP)); do \
			$(BUILD)/pipe_harness $(PIPE_DELAY_US) $(PIPE_BYTES) $$command >/dev/null || exit 1; \
		done; \
		printf '%-12s ' $$label; \
		for round in $$(seq 1 $(ROUNDS)); do \
			$(BUILD)/pipe_harness $(PIPE_DELAY_US) $(PIPE_BYTES) $$command | cut -d' ' -f1; \
		done | sort -n | awk '{a[NR]=$$1} END {printf "median %.2f ms (min %.2f, max %.2f, n=%d)\n", (NR%2 ? a[(NR+1)/2] : (a[NR/2]+a[NR/2+1])/2), a[1], a[NR], NR}'; \
	done

linux-image: linux.Dockerfile
	docker build -f linux.Dockerfile -t $(DOCKER_IMAGE) .

# io_uring is not in Docker's default seccomp allowlist, so both the native
# baseline and the Whitefoot Linux adapter fall back without this flag. The
# whole pipeline runs inside one container because the generated tree must sit
# on a container-local filesystem: measuring a bind mount would measure the
# host's file sharing rather than the kernel's I/O path.
linux: linux-image
	@mkdir -p $(LINUX_TARGET)
	docker run --rm --security-opt seccomp=unconfined \
		-v $(ROOT):/work \
		-v $(LINUX_TARGET):/target \
		-e CARGO_TARGET_DIR=/target \
		-e FILES=$(FILES) -e MAX_KIB=$(MAX_KIB) \
		-e ROUNDS=$(ROUNDS) -e WARMUP=$(WARMUP) \
		-e EXPECTED="$(EXPECTED)" \
		$(DOCKER_IMAGE) \
		sh /work/research/experiments/io-completion-bench/linux-bench.sh


# --- the read-heavy workload --------------------------------------------
#
# The many-files table above is dominated by openat, which on the macOS host
# costs 116 us because of an endpoint-security stack: it measures the host's
# open path far more than it measures the completion framework. This table
# opens eight large files once, outside any loop, and then does tens of
# thousands of positioned reads into them, so what the time is made of is
# reads.
#
# The opens are inside the timed region — the runner times whole processes —
# but there are exactly eight of them in every line, N, S, and C alike, and at
# 116 us each that is 0.93 ms against a table whose fastest line is over a
# hundred. A constant that every line pays identically cannot move a ratio.

read-gen: $(BUILD)/gen
	@rm -rf $(BIGTREE)
	$(BUILD)/gen $(BIGTREE) $(READ_FILES) $(READ_KIB) fixed
	@echo "generated $(READ_FILES) x $(READ_KIB) KiB in $(BIGTREE)"

# Every line must publish the same bytes before any of them reports a time,
# and must publish them with the cache policy off and on, because a policy
# that changed a byte would not be a policy.
read-verify: read-build read-gen
	@cd $(BIGTREE) && for nocache in 0 1; do \
		for line in \
			"$(BUILD)/read_baseline $(BIGTREE) direct $(READS_64K) 65536:$(EXPECTED_READ_64K)" \
			"$(BUILD)/read_heavy_narrow:$(EXPECTED_READ_64K)" \
			"$(BUILD)/read_heavy_narrow_seq:$(EXPECTED_READ_64K)" \
			"$(BUILD)/read_heavy_wide8:$(EXPECTED_READ_64K)" \
			"$(BUILD)/read_heavy_wide8_seq:$(EXPECTED_READ_64K)" \
			"$(BUILD)/read_baseline $(BIGTREE) direct $(READS_4K) 4096:$(EXPECTED_READ_4K)" \
			"$(BUILD)/read_heavy_narrow_4k:$(EXPECTED_READ_4K)" \
			"$(BUILD)/read_heavy_narrow_4k_seq:$(EXPECTED_READ_4K)" \
			"$(BUILD)/read_heavy_wide8_4k $(READ_NAME_ARGS):$(EXPECTED_READ_4K)" \
			"$(BUILD)/read_heavy_wide8_4k_seq $(READ_NAME_ARGS):$(EXPECTED_READ_4K)"; do \
			command=$${line%%:*}; want=$${line#*:}; \
			if [ "$$nocache" = 1 ]; then got=$$(WF_IO_NOCACHE=1 $$command); \
			else got=$$($$command); fi; \
			if [ "$$got" != "$$want" ]; then \
				echo "MISMATCH (WF_IO_NOCACHE=$$nocache): $$command published [$$got], expected [$$want]"; \
				exit 1; \
			fi; \
		done; \
	done
	@echo "every read-heavy line publishes the same bytes with the cache policy off and on"

# Prints the checksums the current tree actually folds, for when the shape
# constants above move.
expected-read: $(BUILD)/read_baseline read-gen
	@$(BUILD)/read_baseline $(BIGTREE) direct $(READS_64K) 65536
	@$(BUILD)/read_baseline $(BIGTREE) direct $(READS_4K) 4096

$(WORK)/read-plan-64k.txt: Makefile | prepare
	@{ \
	printf 'N.direct\t\t$(BUILD)/read_baseline\t$(BIGTREE)\tdirect\t$(READS_64K)\t65536\n'; \
	for t in 2 4 6 8; do \
		printf 'N.pool%s\t\t$(BUILD)/read_baseline\t$(BIGTREE)\tpool\t$(READS_64K)\t65536\t%s\n' $$t $$t; \
	done; \
	printf 'S.narrow\t\t$(BUILD)/read_heavy_narrow_seq\n'; \
	printf 'S.wide8\t\t$(BUILD)/read_heavy_wide8_seq\n'; \
	printf 'C.narrow.default\t\t$(BUILD)/read_heavy_narrow\n'; \
	printf 'C.wide8.default\t\t$(BUILD)/read_heavy_wide8\n'; \
	for h in 0 1 4; do \
		printf 'C.narrow.h%s\tWF_IO_HELPERS=%s\t$(BUILD)/read_heavy_narrow\n' $$h $$h; \
	done; \
	for h in 0 1 2 4 8; do \
		printf 'C.wide8.h%s\tWF_IO_HELPERS=%s\t$(BUILD)/read_heavy_wide8\n' $$h $$h; \
	done; \
	} > $@

$(WORK)/read-plan-4k.txt: Makefile | prepare
	@{ \
	printf 'N.direct\t\t$(BUILD)/read_baseline\t$(BIGTREE)\tdirect\t$(READS_4K)\t4096\n'; \
	for t in 2 4 6 8; do \
		printf 'N.pool%s\t\t$(BUILD)/read_baseline\t$(BIGTREE)\tpool\t$(READS_4K)\t4096\t%s\n' $$t $$t; \
	done; \
	printf 'S.narrow\t\t$(BUILD)/read_heavy_narrow_4k_seq\n'; \
	printf 'S.wide8\t\t$(BUILD)/read_heavy_wide8_4k_seq\t$(READ_NAME_PLAN_ARGS)\n'; \
	printf 'C.narrow.default\t\t$(BUILD)/read_heavy_narrow_4k\n'; \
	printf 'C.wide8.default\t\t$(BUILD)/read_heavy_wide8_4k\t$(READ_NAME_PLAN_ARGS)\n'; \
	for h in 0 1 4; do \
		printf 'C.narrow.h%s\tWF_IO_HELPERS=%s\t$(BUILD)/read_heavy_narrow_4k\n' $$h $$h; \
	done; \
	for h in 0 1 2 4 8; do \
		printf 'C.wide8.h%s\tWF_IO_HELPERS=%s\t$(BUILD)/read_heavy_wide8_4k\t$(READ_NAME_PLAN_ARGS)\n' $$h $$h; \
	done; \
	} > $@

# Ordering, and the check that makes the word "uncached" mean something.
#
# F_NOCACHE stops a read populating the page cache; it does not evict a page
# that is already resident. A table therefore cannot make itself uncached by
# asking for the policy: it has to be handed a tree that is not resident.
# `read-uncache` produces one — the generator writes through a non-populating
# descriptor and flushes each file, so nothing it wrote is left in the cache.
#
# That is checked rather than assumed. `read_baseline probe-uncached` times
# READ_PROBES positioned reads in every file immediately before the table and
# again immediately after it, and make stops unless all but
# READ_TOLERANCE_PERCENT of them cost more than READ_THRESHOLD_US; the warm
# tables are checked the same way in the other direction. The pair of probes
# is what rules out both mistakes: the tree being resident when the table
# starts, and something making it resident while the table runs.
#
# Batch 0092's first attempt had no such check. It took its "uncached" numbers
# straight after generating the tree, when every page was still resident from
# the writes, and published a page-cache table under the uncached label;
# That cost is recorded in this directory's README.
#
# The uncached tables run first, because they are the ones the design question
# turns on and the ones a shared host can spoil; the warm tables follow, after
# `read-warm` reads the whole tree back in. A full sequential pass, not a rerun
# of the workload: 32,768 pseudo-random reads would leave about two per cent of
# the blocks untouched.
read-uncache: $(BUILD)/gen
	@rm -rf $(BIGTREE)
	@$(BUILD)/gen $(BIGTREE) $(READ_FILES) $(READ_KIB) fixed
	@sync

# The wait that a generation needs before it may be measured.
#
# Writing the tree does not only put bytes on the device: it hands the host a
# fresh half-gigabyte of files, and something on this host reads them. A probe
# run every fifteen seconds on an otherwise idle machine watches residency
# walk from f00000 towards f00007 over a minute or two and then disappear as
# the pages age out -- three files resident at one probe, a different three at
# the next, none a minute later. No benchmark line does it: the same walk
# happens across idle gaps with nothing running, and every line reads through
# the non-populating policy. The host's malware scanner is the obvious
# candidate; `XprotectService` holds about ten per cent of a core throughout.
#
# So the tree is left to settle instead of being measured through the scan.
# `read-settle` runs the same probe the table is gated on, quietly, until it
# passes -- which is a wait for an outside reader to finish, not a softer
# threshold. The gate probes below are unchanged and still fail the table.
read-settle: $(BUILD)/read_baseline
	@attempt=0; \
	while :; do \
		if $(BUILD)/read_baseline $(BIGTREE) probe-uncached $(READ_PROBES) 65536 \
			$(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT) > /dev/null 2>&1; then \
			echo "read-settle: the tree is not resident after $$attempt wait(s)"; \
			exit 0; \
		fi; \
		attempt=$$((attempt + 1)); \
		if [ $$attempt -ge $(READ_SETTLE_ATTEMPTS) ]; then \
			echo "read-settle: still resident after $(READ_SETTLE_ATTEMPTS) waits" >&2; \
			$(BUILD)/read_baseline $(BIGTREE) probe-uncached $(READ_PROBES) 65536 \
				$(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT); \
			exit 1; \
		fi; \
		sleep $(READ_SETTLE_SECONDS); \
	done

read-warm: $(BUILD)/read_baseline
	@$(BUILD)/read_baseline $(BIGTREE) warm 1 65536

bench-read: read-verify $(WORK)/read-plan-64k.txt $(WORK)/read-plan-4k.txt
	@echo "== read-heavy 64 KiB, uncached (WF_IO_NOCACHE=1) =="
	$(MAKE) read-uncache
	$(MAKE) read-settle
	$(BUILD)/read_baseline $(BIGTREE) probe-uncached $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	cd $(BIGTREE) && env WF_IO_NOCACHE=1 $(BUILD)/runner $(WORK)/read-plan-64k.txt \
		$(READ_ROUNDS) $(WARMUP) "$(EXPECTED_READ_64K)"
	$(BUILD)/read_baseline $(BIGTREE) probe-uncached $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	@echo "== read-heavy 4 KiB, uncached (WF_IO_NOCACHE=1) =="
	$(MAKE) read-uncache
	$(MAKE) read-settle
	$(BUILD)/read_baseline $(BIGTREE) probe-uncached $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	cd $(BIGTREE) && env WF_IO_NOCACHE=1 $(BUILD)/runner $(WORK)/read-plan-4k.txt \
		$(READ_ROUNDS) $(WARMUP) "$(EXPECTED_READ_4K)"
	$(BUILD)/read_baseline $(BIGTREE) probe-uncached $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	@echo "== read-heavy 64 KiB, page cache warm =="
	$(MAKE) read-warm
	$(BUILD)/read_baseline $(BIGTREE) probe-warm $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	cd $(BIGTREE) && $(BUILD)/runner $(WORK)/read-plan-64k.txt \
		$(READ_ROUNDS) $(WARMUP) "$(EXPECTED_READ_64K)"
	$(BUILD)/read_baseline $(BIGTREE) probe-warm $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	@echo "== read-heavy 4 KiB, page cache warm =="
	$(BUILD)/read_baseline $(BIGTREE) probe-warm $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)
	cd $(BIGTREE) && $(BUILD)/runner $(WORK)/read-plan-4k.txt \
		$(READ_ROUNDS) $(WARMUP) "$(EXPECTED_READ_4K)"
	$(BUILD)/read_baseline $(BIGTREE) probe-warm $(READ_PROBES) 65536 $(READ_THRESHOLD_US) $(READ_TOLERANCE_PERCENT)

# The Linux read tables, in the same container the many-files `linux` target
# uses and for the same reason: the tree must sit on a container-local
# filesystem, and io_uring needs the unconfined seccomp profile.
#
# `read-bench.sh` is one protocol for every host that can run it -- this
# container, a hosted Linux runner, a hosted macOS runner -- and differs from
# `bench-read` above in one deliberate way: it labels a table by what the
# residency probe measured instead of refusing to print one the probe did not
# confirm. Its header says why. `bench-read` keeps the refusal, because on
# this machine the two populations the threshold separates are known.
linux-read: linux-image
	@mkdir -p $(LINUX_TARGET)
	docker run --rm --security-opt seccomp=unconfined \
		-v $(ROOT):/work \
		-v $(LINUX_TARGET):/target \
		-e CARGO_TARGET_DIR=/target \
		-e READ_FILES=$(READ_FILES) -e READ_KIB=$(READ_KIB) \
		-e READS_64K=$(READS_64K) -e READS_4K=$(READS_4K) \
		-e ROUNDS=$(READ_LINUX_ROUNDS) -e WARMUP=$(WARMUP) \
		-e EXPECTED_READ_64K="$(EXPECTED_READ_64K)" \
		-e EXPECTED_READ_4K="$(EXPECTED_READ_4K)" \
		$(DOCKER_IMAGE) \
		sh /work/research/experiments/io-completion-bench/read-bench.sh

# The four-stage chain of design section 12's fourth item, in its four shapes.
# Not one of the three lines: it is a C program on raw io_uring, it links the
# scheduler core for the stack-switch shape, and what it reports is the
# dependent stage's in-flight depth beside the wall time.
CHAIN_JOBS ?= 1000
CHAIN_THREADS ?= 8
CHAIN_SLOTS ?= 32
CHAIN_STACKS ?= 72
SCHED_DIR := $(ROOT)/compiler/src/backend/sched

$(BUILD)/chain: chain.c uring_baseline.h workload.h $(SCHED_DIR)/core.c $(SCHED_DIR)/prim_host.c | prepare
	$(CLANG) $(CFLAGS) -Wpedantic -pthread -I. -I$(SCHED_DIR) \
		chain.c $(SCHED_DIR)/core.c $(SCHED_DIR)/prim_host.c -o $@

chain: $(BUILD)/chain $(BUILD)/gen
	@rm -rf $(WORK)/chaintree
	$(BUILD)/gen $(WORK)/chaintree $(CHAIN_JOBS) $(MAX_KIB)
	@for shape in nested compensate switch pipeline; do \
		$(BUILD)/chain $(WORK)/chaintree $$shape $(CHAIN_JOBS) \
			$(CHAIN_THREADS) $(CHAIN_SLOTS) $(CHAIN_STACKS) || exit 1; \
	done

# --- the TCP echo workload ------------------------------------------------
#
# Three servers on one contract -- the io_uring reference, the epoll reference,
# and the Whitefoot program -- measured by one load generator over the same
# loopback. `linux-net-bench.sh` owns the protocol; these targets only build
# the pieces and hand it this repository's paths, so the container, this host
# and the runner all execute the same bytes.
#
#   make net-tools    build the three C tools
#   make net-verify   every server against netload at 4 connections
#   make linux-net    the protocol, here
NET_OUT := $(WORK)/net
NET_ROUNDS ?= 5
NET_WARMUP ?= 1
WF_ECHO_SOURCE := $(wildcard programs/tcp_echo_server.wf)

net-tools: $(BUILD)/uring_echo $(BUILD)/epoll_echo $(BUILD)/netload

$(BUILD)/uring_echo: uring_echo.c | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

$(BUILD)/epoll_echo: epoll_echo.c compute_protocol.h | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

$(BUILD)/netload: netload.c compute_protocol.h | prepare
	$(CLANG) $(CFLAGS) -pthread $< -o $@

# This client policy is independent of the server representation. Budget one
# forces the edge-triggered userspace continuation; both enter the timed panel.
.PHONY: client-service-check
client-service-check: $(BUILD)/epoll_echo
	@set -e; \
	for budget in 1 8; do \
	  $(CLANG) $(CFLAGS) -Wpedantic -pthread "-DWF_NETLOAD_SERVICE_ROUNDS=$$budget" \
	    netload.c -o "$(BUILD)/netload-service$$budget"; \
	  env ROOT="$(ROOT)" OUT="$(BUILD)" NET_LINES=epoll NETLOAD="$(BUILD)/netload-service$$budget" \
	    sh linux-net-bench.sh verify-client; \
	done

.PHONY: netload-observe-check client-observer-check
netload-observe-check: prepare
	$(CLANG) $(CFLAGS) -Wpedantic -fsanitize=address,undefined netload_observe_check.c -o "$(BUILD)/netload-observe-check"
	"$(BUILD)/netload-observe-check"

client-observer-check: netload-observe-check $(BUILD)/epoll_echo
	@set -e; \
	for budget in 0 1 8; do \
	  $(CLANG) $(CFLAGS) -Wpedantic -pthread -DWF_NETLOAD_OBSERVE=1 "-DWF_NETLOAD_SERVICE_ROUNDS=$$budget" \
	    netload.c -o "$(BUILD)/netload-observe$$budget"; \
	  env ROOT="$(ROOT)" OUT="$(BUILD)" NET_LINES=epoll NETLOAD="$(BUILD)/netload-observe$$budget" \
	    sh linux-net-bench.sh verify-observer; \
	done

# The scripted checks execute the real exchange body, with strict syscall
# ordering that rejects a lost read edge. Socket checks execute each ordinary
# client separately; observer-only cases prove partial I/O actually occurred.
.PHONY: client-readiness-check
client-readiness-check: prepare
	$(CLANG) $(CFLAGS) -pthread -DWF_BENCH_COMPUTE epoll_echo.c -o "$(BUILD)/epoll_compute"
	@set -e; \
	for budget in 0 1 8; do \
	  $(CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=address,undefined \
	    -DWF_NETLOAD_READINESS=1 "-DWF_NETLOAD_SERVICE_ROUNDS=$$budget" \
	    netload_check.c -o "$(BUILD)/netload-check$$budget"; \
	  "$(BUILD)/netload-check$$budget"; \
	  for readiness in 0 1; do \
	    $(CLANG) $(CFLAGS) -Wpedantic -pthread "-DWF_NETLOAD_READINESS=$$readiness" \
	      "-DWF_NETLOAD_SERVICE_ROUNDS=$$budget" netload.c -o "$(BUILD)/netload-ready$$readiness-service$$budget"; \
	    "$(BUILD)/netload-check$$budget" "$(BUILD)/netload-ready$$readiness-service$$budget"; \
	    ROOT="$(ROOT)" OUT="$(BUILD)" NET_LINES=compute NETLOAD="$(BUILD)/netload-ready$$readiness-service$$budget" \
	      timeout --signal=TERM --kill-after=5s 30s sh linux-net-bench.sh verify-readiness-paced; \
	  done; \
	done; \
	for readiness in 0 1; do \
	  $(CLANG) $(CFLAGS) -Wpedantic -pthread "-DWF_NETLOAD_READINESS=$$readiness" \
	    -DWF_NETLOAD_OBSERVE=1 netload.c -o "$(BUILD)/netload-ready$$readiness-observed"; \
	  "$(BUILD)/netload-check0" "$(BUILD)/netload-ready$$readiness-observed" observed; \
	done; \
	mkdir -p "$(BUILD)/netload-lost-edge"; \
	cp netload_check.c "$(BUILD)/netload-lost-edge/"; \
	sed '/link->read_events |= events\[at\].events/d' netload.c > "$(BUILD)/netload-lost-edge/netload.c"; \
	$(CLANG) $(CFLAGS) -Wpedantic -pthread -I . -DWF_NETLOAD_READINESS=1 \
	  "$(BUILD)/netload-lost-edge/netload_check.c" -o "$(BUILD)/netload-lost-edge/check"; \
	status=0; "$(BUILD)/netload-lost-edge/check" > "$(BUILD)/netload-lost-edge/out" 2> "$(BUILD)/netload-lost-edge/err" || status=$$?; \
	test "$$status" = 2; \
	echo 'netload trace: missing read-edge mutation rejected PASS'

# Completion lifetime uses the actual C runtime, without changing its ABI.
# Only the bridge publisher is interposed by the test registration coordinator.
# Both ordinary and elided nested frames must drain borrowed storage before
# destruction. Linux also qualifies its real helper fallback explicitly.
.PHONY: completion-coroutine-check
completion-coroutine-check: prepare
	@set -e; \
	backend="$(ROOT)/compiler/src/backend"; \
	build="$(BUILD)/completion-coroutine-$(COMPLETION_CORO_SANITIZERS)"; \
	mkdir -p "$$build"; set --; \
	for source in $(CONTINUATION_RUNTIME_UNITS); do \
	  define=''; if [ "$$source" = completion/bridge.c ]; then define=-Dwf_sched_complete=wf_probe_publish; fi; \
	  object="$$build/$$(basename "$$source").o"; \
	  $(CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=$(COMPLETION_CORO_SANITIZERS) \
	    -I "$$backend" $$define -c "$$backend/$$source" -o "$$object"; \
	  set -- "$$@" "$$object"; \
	done; \
	$(CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=$(COMPLETION_CORO_SANITIZERS) \
	  -I "$$backend" -x c -c coroutine_completion.cpp -o "$$build/protocol.o"; \
	for form in heap elide; do \
	  define=''; if [ "$$form" = elide ]; then define=-DPROBE_ELIDE; fi; \
	  $(CORO_CXX) $(CORO_FLAGS) -g -fsanitize=$(COMPLETION_CORO_SANITIZERS) \
	    -I "$$backend" $$define coroutine_completion.cpp "$$build/protocol.o" "$$@" -o "$$build/$$form"; \
	  for buckets in 1 2 1024; do \
	    WF_CONTINUATION_PENDING_BUCKETS=$$buckets "$$build/$$form" $(COMPLETION_CORO_ARGS); \
	    if [ "$$(uname -s)" = Linux ]; then WF_CONTINUATION_PENDING_BUCKETS=$$buckets WF_IO_NO_NATIVE_RING=1 "$$build/$$form" --require-helper; fi; \
	  done; \
	done

# The same coordinator now drives actual WF-generated continuations. Keep the
# independent C++ lifetime check as a prerequisite. The observer handshake
# gates input until a root is suspended; timings are not part of this target.
.PHONY: compiler-continuation-check
compiler-continuation-check: completion-coroutine-check compiler
	@set -e; \
	backend="$(ROOT)/compiler/src/backend"; \
	build="$(BUILD)/compiler-continuation"; \
	common="$(BUILD)/completion-coroutine-$(COMPLETION_CORO_SANITIZERS)"; \
	mkdir -p "$$build/tree"; set --; \
	for source in $(CONTINUATION_RUNTIME_UNITS); do set -- "$$@" "$$common/$$(basename "$$source").o"; done; \
	$(CORO_CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=$(COMPLETION_CORO_SANITIZERS) \
	  -I "$$backend" -DPROBE_WF_GENERATED -DPROBE_LOOKUP_OBSERVE=1 -x c -c coroutine_completion.cpp -o "$$build/host.o"; \
	$(CORO_CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=$(COMPLETION_CORO_SANITIZERS) \
	  -I "$$backend" -c "$$backend/wf_floor.c" -o "$$build/floor.o"; \
	$(CORO_CLANG) $(CFLAGS) -Wpedantic -pthread continuation_stream.c -o "$$build/stream-check"; \
	$(CORO_CLANG) $(CFLAGS) -Wpedantic -pthread stream_check.c -o "$$build/protocol-check"; \
	case "$(COMPLETION_CORO_SANITIZERS)" in \
	  *address*) ir_attribute=sanitize_address ;; \
	  thread) ir_attribute=sanitize_thread ;; \
	  *) echo 'compiler-continuation-check needs address or thread instrumentation' >&2; exit 2 ;; \
	esac; \
	for program in boundary stdin recursive tcp client refused fanout staged compute; do \
	  staging=''; \
	  case "$$program" in \
	    boundary) source="$(ROOT)/tests/programs/completion_read_boundary.wf" ;; \
	    stdin) source="$(ROOT)/tests/programs/stdin_echo.wf" ;; \
	    recursive) source=programs/continuation_stream.wf ;; \
	    tcp) source=programs/tcp_echo_server.wf ;; \
	    client) source="$(ROOT)/tests/programs/tcp_client.wf" ;; \
	    refused) source="$(ROOT)/tests/programs/tcp_refused.wf" ;; \
	    fanout) source="$(ROOT)/tests/programs/tcp_fanout.wf"; staging=--par ;; \
	    staged) source=programs/tcp_echo_server.wf; staging=--par ;; \
	    compute) source=programs/tcp_compute_server.wf; staging=--par ;; \
	  esac; \
	  "$(WFC)" --emit-llvm --continuations $$staging "$$source" -o "$$build/$$program.ll"; \
	  awk -v attribute="$$ir_attribute" '/^define / {sub(/ \{$$/, " " attribute " {")} {print}' \
	    "$$build/$$program.ll" > "$$build/$$program-instrumented.ll"; \
	  $(CORO_CLANG) -Wno-override-module -O2 -g -pthread -fsanitize=$(COMPLETION_CORO_SANITIZERS) \
	    "$$build/$$program-instrumented.ll" "$$build/host.o" "$$build/floor.o" "$$@" -o "$$build/$$program"; \
	done; \
	for buckets in 1 1024; do \
	export WF_CONTINUATION_PENDING_BUCKETS=$$buckets; \
	for policy in 0:1 1:1 1:32; do \
	export WF_CONTINUATION_OWNER_PROGRESS=$${policy%:*} WF_CONTINUATION_PROGRESS_BATCH=$${policy#*:}; \
	echo "compiled continuation progress owner=$${policy%:*} batch=$${policy#*:} pending_buckets=$$buckets"; \
	printf A > "$$build/tree/a"; printf B > "$$build/tree/b"; printf AB > "$$build/expected"; \
	(cd "$$build/tree" && WF_IO_HELPERS=4 ../boundary a b) > "$$build/actual"; \
	cmp "$$build/expected" "$$build/actual"; \
	status=0; (cd "$$build/tree" && ../boundary absent b) > "$$build/absent" || status=$$?; \
	test "$$status" = 5 && test ! -s "$$build/absent"; \
	: > "$$build/tree/a"; \
	status=0; (cd "$$build/tree" && ../boundary a b) > "$$build/empty" || status=$$?; \
	test "$$status" = 8 && test ! -s "$$build/empty"; \
	status=0; "$$build/boundary" > "$$build/no-args" || status=$$?; \
	test "$$status" = 1 && test ! -s "$$build/no-args"; \
	echo 'compiled continuation file outcomes: PASS success/open-failure/EOF/no-I/O-return'; \
	for program in stdin recursive tcp client refused occupied fanout staged; do \
	  transport=''; \
	  case "$$program" in \
	    tcp) transport=--tcp ;; \
	    client) transport=--client ;; \
	    refused) transport=--refused ;; \
	    occupied) transport=--occupied; program=tcp ;; \
	    fanout) transport=--fanout ;; \
	    staged) transport=--staged ;; \
	  esac; \
	  "$$build/stream-check" $$transport "$$build/$$program" $(COMPLETION_CORO_ARGS); \
	  if [ "$$(uname -s)" = Linux ]; then \
	    WF_IO_NO_NATIVE_RING=1 "$$build/stream-check" $$transport "$$build/$$program" --require-helper; \
	  fi; \
	done; \
	for force_helper in 0 1; do \
	  if [ "$$force_helper" = 1 ] && [ "$$(uname -s)" != Linux ]; then continue; fi; \
	  route=0; no_ring=$$force_helper; \
	  if [ "$(COMPLETION_CORO_ARGS)" = --require-ring ]; then route=1; fi; \
	  if [ "$(COMPLETION_CORO_ARGS)" = --require-helper ]; then route=2; no_ring=1; fi; \
	  if [ "$$force_helper" = 1 ]; then route=2; fi; \
	  for mode in compute truncated oversized; do \
	    tasks=4; error_arg=''; \
	    if [ "$$mode" = truncated ]; then tasks=1; error_arg=9; fi; \
	    if [ "$$mode" = oversized ]; then tasks=1; error_arg=10; fi; \
	    log="$$build/compute-$$mode-$${policy%:*}-$${policy#*:}-buckets$$buckets-helper$$force_helper.log"; \
	    if ! env -u WF_CONTINUATION_REPORT_BRIDGE WF_WORKERS=1 WF_IO_NO_NATIVE_RING=$$no_ring \
	      WF_CONTINUATION_WINDOW=16 WF_CONTINUATION_OBSERVE=1 WF_CONTINUATION_REPORT_SOCKET_ROUTES=1 \
	      WF_CONTINUATION_REPORT_TASKS=1 "$$build/protocol-check" "$$build/compute" "$$mode" 1 $$error_arg \
	      > "$$log" 2>&1; then cat "$$log"; exit 1; fi; \
	    cat "$$log"; \
	    awk -v owner=$${policy%:*} -v batch=$${policy#*:} -v buckets=$$buckets -v tasks="$$tasks" -v route="$$route" -v mode="$$mode" \
	      '/^WF continuation host: suspended$$/ {next} \
	       /^WF continuation host:/ {reports++; for(i=4;i<=NF;i++) {split($$i,a,"=");v[a[1]]=a[2]+0}; next} \
	       /^stream_check: PASS/ {passes++; if($$3!="mode=" mode || $$4!="workers=1") bad=1; next} \
	       {bad=1} END {exit !(reports==3 && passes==1 && !bad && v["registered"]>0 && \
	         v["registered"]==v["dequeued"] && ("owner_progress" in v) && v["owner_progress"]==owner && \
	         ("progress_batch" in v) && v["progress_batch"]==batch && v["pending_buckets"]==buckets && v["tasks"]==tasks && \
	         v["lookup_calls"]>0 && v["lookup_steps"]>=v["lookup_max_steps"] && \
	         v["completed"]==tasks && v["retired"]==tasks && v["peak"]==tasks && \
	         v["accept_ring"]+v["accept_helper"]==tasks && (mode!="compute" || v["receive_ring"]+v["receive_helper"]>0) && \
	         (route!=1 || (v["uring"]>0 && v["accept_ring"]==tasks && (mode!="compute" || v["receive_ring"]>0) && v["receive_helper"]==0)) && \
	         (route!=2 || (v["uring"]==0 && v["helper"]>0 && v["accept_helper"]==tasks && v["receive_ring"]==0)))}' "$$log"; \
	  done; \
	done; \
	done; \
	done

# The first generated-WF timing form keeps the qualified coordinator and one
# resumer. Root-scheduler metadata and TCP_NODELAY match the small stackful
# control. Host counters remain enabled in both timed and observed runs.
.PHONY: compiler-continuation-bench
compiler-continuation-bench: prepare compiler
	@set -e; \
	backend="$(ROOT)/compiler/src/backend"; \
	build="$(BUILD)/compiler-continuation-bench"; mkdir -p "$$build"; set --; \
	flags='-DWF_TCP_NODELAY=1 -DWF_SCHED_COMPACT_STACKS=1 -DWF_SCHED_INIT_USED_LANES=1'; \
	for source in $(CONTINUATION_RUNTIME_UNITS) wf_floor.c; do \
	  define=''; if [ "$$source" = completion/bridge.c ]; then define=-Dwf_sched_complete=wf_probe_publish; fi; \
	  object="$$build/$$(basename "$$source").o"; \
	  $(CORO_CLANG) $(CFLAGS) -g -Wpedantic -pthread -I "$$backend" $$flags $$define \
	    -c "$$backend/$$source" -o "$$object"; \
	  set -- "$$@" "$$object"; \
	done; \
	"$(WFC)" --emit-llvm --continuations --par programs/tcp_echo_server.wf -o "$$build/echo.ll"; \
	$(CORO_CLANG) -O2 -S -emit-llvm -Wno-override-module "$$build/echo.ll" -o "$$build/echo-optimized.ll"; \
	$(CORO_CLANG) $(CFLAGS) -Wpedantic -pthread continuation_stream.c -o "$$build/stream-check"; \
	for observed in 0 1; do \
	  suffix=''; if [ "$$observed" = 1 ]; then suffix=-lookup-observed; fi; \
	  $(CORO_CLANG) $(CFLAGS) -g -Wpedantic -pthread -I "$$backend" $$flags \
	    -DPROBE_WF_GENERATED -DPROBE_LOOKUP_OBSERVE=$$observed -x c -c coroutine_completion.cpp -o "$$build/host$$suffix.o"; \
	  $(CORO_CLANG) -O2 -g -Wno-override-module -pthread "$$build/echo.ll" \
	    "$$build/host$$suffix.o" "$$@" -o "$$build/wf-coro$$suffix"; \
	for buckets in 1 1024; do \
	for policy in 0:1 1:1 1:32; do \
	  WF_CONTINUATION_OWNER_PROGRESS=$${policy%:*} WF_CONTINUATION_PROGRESS_BATCH=$${policy#*:} WF_CONTINUATION_PENDING_BUCKETS=$$buckets \
	    "$$build/stream-check" --staged "$$build/wf-coro$$suffix" $(COMPLETION_CORO_ARGS); \
	done; \
	done; \
	done

# Linux-only native completion reference correctness. The full streaming
# oracle is shared with the epoll controls; no request-size bound substitutes
# for handling TCP back pressure, short transfers and EOF.
.PHONY: uring-check
URING_POOL_CHECK ?= 0
uring-check: prepare
	@set -e; \
	test "$(URING_POOL_CHECK)" = 0 || test "$(URING_POOL_CHECK)" = 1; \
	for inline_send in 0 1; do \
	 for observed in 0 1; do \
	  observe_flag=''; if test "$$observed" = 1; then observe_flag=-DWF_BENCH_URING_OBSERVE; fi; \
	  binary="$(BUILD)/uring-trace-inline$$inline_send-observe$$observed"; \
	  $(CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=address,undefined \
	    "-DWF_BENCH_URING_INLINE_SEND=$$inline_send" $$observe_flag uring_echo_check.c -o "$$binary"; \
	  "$$binary"; \
	 done; \
	done; \
	$(CLANG) $(CFLAGS) -Wpedantic -pthread stream_check.c -o "$(BUILD)/stream_check"; \
	for bytes in 8192 65536; do \
	 for inline_send in 0 1; do \
	  counts=0; if test "$(URING_POOL_CHECK)" = 1 && test "$$bytes" = 65536; then counts='0 2 32 64 128'; fi; \
	  for count in $$counts; do \
	  binary="$(BUILD)/check-uring-$$bytes-inline$$inline_send"; \
	  if test "$$count" != 0; then binary="$$binary-pool$$count"; fi; \
	  $(CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=address,undefined \
	    -DWF_BENCH_SNDBUF=4096 -DWF_BENCH_TCP_VERIFY -DWF_BENCH_URING_OBSERVE \
	    "-DWF_BENCH_URING_BUFFER_BYTES=$$bytes" "-DWF_BENCH_URING_INLINE_SEND=$$inline_send" \
	    "-DWF_BENCH_URING_BUFFER_COUNT=$$count" uring_echo.c -o "$$binary"; \
	  for workers in 1 4; do \
	    log="$$binary-w$$workers.log"; \
	    if ! "$(BUILD)/stream_check" "$$binary" echo "$$workers" > "$$log" 2>&1; then cat "$$log"; exit 1; fi; \
	    cat "$$log"; \
	    awk -v bytes="$$bytes" -v count="$$count" -v workers="$$workers" -v inline_send="$$inline_send" '/^uring:/ { \
	      for(key in v) delete v[key]; \
	      for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]=a[2]+0}; seen++; \
	      n=split("worker buffer_bytes buffers provided_bytes ring_entries receives sends receive_bytes send_bytes exhausted deepest_queue inline_send inline_attempts inline_bytes inline_succeeded inline_short inline_eagain inline_requested_bytes inline_requested_vectors ring_requests ring_requested_bytes ring_requested_vectors buffer_acquires buffer_returns buffer_terminal loaned_buffers peak_loaned_buffers receive_arms starved_parks starved_rearms starved_closed peak_starved late_exhaustion_retries",required," "); \
	      for(i=1;i<=n;i++) if(!(required[i] in v) || v[required[i]]<0) bad=1; \
	      if(v["worker"]<0 || v["worker"]>=workers || ids[v["worker"]]++) bad=1; \
	      if(v["buffer_bytes"]!=bytes || v["provided_bytes"]!=(count ? count*bytes : 2097152) || \
	         v["buffers"]*bytes!=v["provided_bytes"] || v["inline_send"]!=inline_send) bad=1; \
	      if(v["buffer_acquires"]!=v["receives"]+v["buffer_terminal"] || v["buffer_returns"]!=v["buffer_acquires"] || \
	         v["loaned_buffers"]!=0 || v["peak_loaned_buffers"]>v["buffers"] || \
	         v["deepest_queue"]>v["peak_loaned_buffers"] || \
	         (v["receives"]>0 && v["peak_loaned_buffers"]==0) || \
	         v["starved_parks"]!=v["starved_rearms"]+v["starved_closed"] || \
	         v["exhausted"]!=v["starved_parks"]+v["late_exhaustion_retries"] || \
	         v["receive_arms"]<v["starved_rearms"]+v["late_exhaustion_retries"]) bad=1; \
	      if(v["ring_requests"]!=v["sends"] || v["ring_requested_bytes"]<v["send_bytes"]-v["inline_bytes"] || \
	         v["ring_requested_vectors"]<v["ring_requests"] || v["inline_requested_bytes"]<v["inline_bytes"] || \
	         v["inline_requested_vectors"]<v["inline_attempts"] || v["inline_short"]>v["inline_succeeded"] || \
	         v["inline_succeeded"]+v["inline_eagain"]>v["inline_attempts"]) bad=1; \
	      received+=v["receive_bytes"]; sent+=v["send_bytes"]; ring_sends+=v["sends"]; \
	      inline_bytes+=v["inline_bytes"]; attempts+=v["inline_attempts"]; exhausted+=v["exhausted"]; rearms+=v["starved_rearms"]+v["late_exhaustion_retries"] \
	    } END {exit !(seen==workers && !bad && received==8388608 && sent==received && ring_sends>0 && \
	      (count!=2 || (exhausted>0 && rearms>0)) && \
	      (inline_send ? inline_bytes>0 && attempts>0 : inline_bytes==0 && attempts==0))}' "$$log"; \
	  done; \
	  done; \
	 done; \
	done



# External sequential-API reference, qualified independently before timing.
# Toolchain upgrades are new comparison revisions, never an implicit download.
GO ?= go
GO_VERSION := go1.27.1
# Non-recipe continuations become spaces. Inside a single-quoted recipe jq
# program GNU Make preserves the backslash/newline, which jq cannot parse.
GO_QUALIFY_FILTER = length == (if $$mode=="reset" then 1 else 2 end) and \
    all(.[]; .kind=="go-net" and .version=="$(GO_VERSION)" and .gomaxprocs==$$width and .buffer_bytes==65536 and .buffer_owner==$$owner and \
      .metrics["/sched/gomaxprocs:threads"]==$$width and .metrics["/sched/threads/total:threads"]>=1) and \
    .[0].stage=="listening" and (if $$mode=="reset" then true else \
      .[1].stage=="complete" and (.[1].sockets|length)==(if $$mode=="resident" then 64 else 4 end) and \
      all(.[1].sockets[]; .nodelay!=0 and .send_buffer>=4096) end)
# Pinned Linux/amd64 allocation-control evidence, not a timing threshold.
GO_STORAGE_ASM_FILTER = \
    /^TEXT main.startEchoStack.func1\(/ {fn="stack"; next} \
    /^TEXT main.startEchoHeap.func1\(/ {fn="heap"; next} \
    /^TEXT main.echoBuffer\(/ {fn="shared"; next} \
    /^TEXT main.startEchoHeap\(/ {fn="allocator"; next} /^TEXT/ {fn=""} \
    fn=="stack" && /SUBQ \$$0x1[[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]], SP/ {large++} \
    fn=="stack" && /REP; STOSQ/ {zero++} \
    (fn=="heap" || fn=="shared") && /SUBQ.*SP/ {frames++; if(/SUBQ \$$0x[[:xdigit:]][[:xdigit:]], SP/) small++} \
    (fn=="stack" || fn=="heap") && /CALL main.echoBuffer\(/ {shared++} \
    fn=="allocator" && /CALL runtime.makeslice\(/ {alloc++} \
    END {exit !(large==1 && zero==1 && frames==2 && small==2 && shared==2 && alloc==1)}
.PHONY: go-check
go-check: prepare
	@set -eu; \
	export GOTOOLCHAIN=local GOCACHE="$(BUILD)/go-cache"; \
	actual=$$("$(GO)" env GOVERSION); \
	if test "$$actual" != "$(GO_VERSION)"; then echo "go-check: requires $(GO_VERSION), found $$actual; set GO to the pinned binary" >&2; exit 1; fi; \
	go_root=$$("$(GO)" env GOROOT); \
	test -z "$$("$$go_root/bin/gofmt" -l go_echo.go)"; \
	"$(GO)" vet go_echo.go; \
	$(CLANG) $(CFLAGS) -Wpedantic -pthread stream_check.c -o "$(BUILD)/go-stream-check"; \
	mkdir -p "$(BUILD)/go-qualification"; \
	"$(GO)" version > "$(BUILD)/go-qualification/toolchain.txt"; \
	"$(GO)" env GOOS GOARCH GOAMD64 CGO_ENABLED GOTOOLCHAIN GOEXPERIMENT >> "$(BUILD)/go-qualification/toolchain.txt"; \
	for profile in release race; do \
	  binary="$(BUILD)/go-qualification/go-echo-$$profile"; \
	  if test "$$profile" = race; then \
	    CGO_ENABLED=1 CC="$(CLANG)" "$(GO)" build -race -trimpath -gcflags=-m=2 -o "$$binary" go_echo.go 2> "$$binary.escape.txt"; \
	  else CGO_ENABLED=0 "$(GO)" build -trimpath -gcflags=-m=2 -o "$$binary" go_echo.go 2> "$$binary.escape.txt"; fi; \
	  "$(GO)" version -m "$$binary" > "$$binary.build.txt"; \
	  "$(GO)" tool objdump -s 'main\.(echo|startEcho)' "$$binary" > "$$binary.objdump.txt"; \
	done; \
	stack_line=$$(awk '/func echoStack\(/ {inside=1} inside && /buffer := make\(/ {print NR; exit}' go_echo.go); \
	heap_line=$$(awk '/func startEchoHeap\(/ {inside=1} inside && /buffer := make\(/ {print NR; exit}' go_echo.go); \
	awk -v location="go_echo.go:$$stack_line:" 'index($$0,location) && /make\(\[\]byte, 65536\) does not escape$$/ {seen=1} END {exit !seen}' "$(BUILD)/go-qualification/go-echo-release.escape.txt"; \
	awk -v location="go_echo.go:$$heap_line:" 'index($$0,location) && /make\(\[\]byte, 65536\) escapes to heap$$/ {seen=1} END {exit !seen}' "$(BUILD)/go-qualification/go-echo-release.escape.txt"; \
	if test "$$("$(GO)" env GOOS)/$$("$(GO)" env GOARCH)" = linux/amd64; then \
	  awk '$(GO_STORAGE_ASM_FILTER)' "$(BUILD)/go-qualification/go-echo-release.objdump.txt"; \
	fi; \
	for phase in lifecycle resident; do \
	 for profile in release race; do \
	  binary="$(BUILD)/go-qualification/go-echo-$$profile"; \
	  for owner in handler acceptor; do \
	   export WF_BENCH_GO_BUFFER_OWNER="$$owner"; \
	   for width in 1 4; do \
	    sample="$$binary-$$owner-p$$width"; \
	    modes=resident; \
	    if test "$$phase" = lifecycle; then \
	      modes='echo reset'; log="$$sample-quiet.log"; \
	      if ! env -u WF_BENCH_GO_OBSERVE -u WF_BENCH_GO_SNDBUF "$(BUILD)/go-stream-check" "$$binary" echo "$$width" > "$$log" 2>&1; then cat "$$log"; exit 1; fi; \
	      test "$$(wc -l < "$$log" | tr -d ' ')" = 1; \
	    fi; \
	    for mode in $$modes; do \
	      log="$$sample-$$mode.log"; \
	      prefix="-"; if test "$$(uname -s)" = Linux; then prefix="$$sample-live"; fi; \
	      if test "$$mode" = resident; then \
	        WF_BENCH_GO_OBSERVE=1 WF_BENCH_GO_SNDBUF=4096 WF_BENCH_PROCESS_DETAILS=1 \
	          "$(BUILD)/go-stream-check" "$$binary" resident "$$width" 64 65536 "$$prefix" > "$$log" 2>&1; \
	      else WF_BENCH_GO_OBSERVE=1 WF_BENCH_GO_SNDBUF=4096 \
	        "$(BUILD)/go-stream-check" "$$binary" "$$mode" "$$width" > "$$log" 2>&1; fi; \
	      cat "$$log"; \
	      awk -v mode="$$mode" '/^\{/ {next} /^stream_check: PASS/ {passes++; next} /^go_echo:/ && mode=="reset" {errors++; next} {bad=1} END {exit !(passes==1 && !bad && (mode=="reset" ? errors==1 : errors==0))}' "$$log"; \
	      awk '/^\{/' "$$log" | jq -s -e --argjson width "$$width" --arg mode "$$mode" --arg owner "$$owner" \
	        '$(GO_QUALIFY_FILTER)' > /dev/null; \
	      if test "$$mode" = resident && test "$$prefix" != "-"; then \
	        test -s "$$prefix.smaps"; test -s "$$prefix.status"; \
	        awk -v expected="$${WF_BENCH_SERVER_CPUS:-}" '/^Cpus_allowed_list:/ {seen++; if(expected!="" && $$2!=expected) bad=1} END {exit !(seen>0 && !bad)}' "$$prefix".task-*-status; \
	        awk '/^tfd:/ {seen++} END {exit !(seen>0)}' "$$prefix".fdinfo-*; \
	      fi; \
	    done; \
	   done; \
	  done; \
	 done; \
	done


# Linux-only reference correctness, also reached by scheduler-check on Linux.
# The send-buffer override and observers never enter timed binaries.
.PHONY: coroutine-check
coroutine-check: prepare
	@set -e; \
	$(CLANG) $(CFLAGS) -Wpedantic -pthread stream_check.c -o "$(BUILD)/stream_check"; \
	for mode in echo compute quantum; do \
	  flags=''; storage_policies=0; cases='compute truncated'; \
	  if [ "$$mode" = echo ]; then storage_policies='0 3'; cases=echo; else flags=-DWF_BENCH_COMPUTE; fi; \
	  if [ "$$mode" = quantum ]; then flags="$$flags -DWF_BENCH_QUANTUM"; fi; \
	  for form in manual stackful heap elide; do \
	    extra=''; \
	    if [ "$$form" != manual ]; then extra='-DWF_BENCH_STACKFUL -DWF_BENCH_OBSERVE'; fi; \
	    if [ "$$form" = heap ] || [ "$$form" = elide ]; then extra="$$extra -DWF_BENCH_COROUTINE"; fi; \
	    if [ "$$form" = elide ]; then extra="$$extra -DWF_BENCH_CORO_ELIDE"; fi; \
	    for storage in $$storage_policies; do \
	      binary="$(BUILD)/coro-$$mode-$$form-s$$storage"; \
	      $(CORO_CXX) $(CORO_FLAGS) -DWF_BENCH_SNDBUF=4096 -DWF_BENCH_STORAGE_OBSERVE \
	        "-DWF_BENCH_RECEIVE_STORAGE=$$storage" $$flags $$extra -x c++ epoll_echo.c -o "$$binary"; \
	      for workers in 1 4; do \
	       for case_name in $$cases; do \
	        log="$$binary-w$$workers-$$case_name.log"; \
	        if ! "$(BUILD)/stream_check" "$$binary" "$$case_name" "$$workers" > "$$log" 2>&1; then cat "$$log"; exit 1; fi; \
	        cat "$$log"; \
	        awk -v storage="$$storage" '/^storage:/ {for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]=a[2]+0}; seen=1} \
	          END {exit !(seen && v["policy"]==storage && v["accepted"]>0 && v["closed"]==v["accepted"])}' "$$log"; \
	        if [ "$$form" = heap ] || [ "$$form" = elide ]; then \
	          expected=4; if [ "$$case_name" = truncated ]; then expected=1; fi; \
	          awk -v form="$$form" -v expected="$$expected" '/^coroutine:/ {for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]=a[2]+0}; seen=1} \
	            END {exit !(seen && v["allocations"]>0 && v["allocations"]==v["frees"] && (form!="elide" || v["allocations"]==expected))}' "$$log"; \
	        fi; \
	        if [ "$$form" != manual ] && [ "$$case_name" != truncated ]; then \
	          awk -v mode="$$mode" '/^stackful:/ {for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]+=a[2]+0}} \
	            END {exit !(v["resumes"]>0 && v["waits"]>0 && (mode!="echo" || v["send_waits"]>0) && (mode!="quantum" || v["yields"]>0))}' "$$log"; \
	        fi; \
	       done; \
	      done; \
	    done; \
	  done; \
	done; \
	for form in heap elide; do \
	  extra=''; if [ "$$form" = elide ]; then extra=-DWF_BENCH_CORO_ELIDE; fi; \
	  binary="$(BUILD)/coro-lifetime-$$form"; \
	  $(CORO_CXX) $(CORO_FLAGS) -g -fsanitize=address,undefined -DWF_BENCH_STACKFUL \
	    -DWF_BENCH_COROUTINE -DWF_BENCH_COMPUTE -DWF_BENCH_OBSERVE $$extra coroutine_lifetime.cpp -o "$$binary"; \
	  "$$binary"; \
	done

# The ARM storage control adds server-only syscall evidence. LeakSanitizer
# runs without ptrace first; traced repetitions retain ASan/UBSan but disable
# leak detection, which cannot operate under strace. Neither enters timing.
EPOLL_STORAGE_CHECK ?= 0
stackful-check: prepare
	@set -e; \
	$(CLANG) $(CFLAGS) -Wpedantic -pthread stream_check.c -o "$(BUILD)/stream_check"; \
	for mode in echo compute quantum; do \
	  flags=''; \
	  if [ "$$mode" != echo ]; then flags='-DWF_BENCH_COMPUTE'; fi; \
	  if [ "$$mode" = quantum ]; then flags="$$flags -DWF_BENCH_QUANTUM"; fi; \
	  storage_policies=0; \
	  if [ "$$mode" = echo ]; then storage_policies='0 1 2 3'; fi; \
	  for form in manual stackful; do \
	    extra=''; \
	    if [ "$$form" = stackful ]; then extra='-DWF_BENCH_STACKFUL -DWF_BENCH_OBSERVE'; fi; \
	   for storage in $$storage_policies; do \
	    placements=0; if [ "$$mode" = echo ] && [ "$$storage" = 3 ]; then placements='0 1'; fi; \
	    for placement in $$placements; do \
	    placement_flags=''; suffix=''; \
	    if [ "$$placement" = 1 ]; then placement_flags=-DWF_BENCH_MAIN_WORKER; suffix=-main; fi; \
	    binary="$(BUILD)/check-$$mode-$$form-s$$storage$$suffix"; \
	    $(CLANG) $(CFLAGS) -Wpedantic -pthread -DWF_BENCH_SNDBUF=4096 \
	      -DWF_BENCH_STORAGE_OBSERVE "-DWF_BENCH_RECEIVE_STORAGE=$$storage" $$flags $$extra $$placement_flags epoll_echo.c -o "$$binary"; \
	    cases='compute truncated'; \
	    if [ "$$mode" = echo ]; then cases=echo; fi; \
	    for workers in 1 4; do \
	      for case_name in $$cases; do \
	        log="$$binary-w$$workers-$$case_name.log"; \
	        if ! "$(BUILD)/stream_check" "$$binary" "$$case_name" "$$workers" > "$$log" 2>&1; then cat "$$log"; exit 1; fi; \
	        cat "$$log"; \
	        awk -v storage="$$storage" -v placement="$$placement" '/^storage:/ {for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]=a[2]+0}; seen=1} \
	          END {exit !(seen && v["policy"]==storage && ("main_worker" in v) && v["main_worker"]==placement && v["accepted"]>0 && v["closed"]==v["accepted"])}' "$$log"; \
	        if [ "$$form" = stackful ] && [ "$$case_name" != truncated ]; then \
	          awk -v mode="$$mode" '/^stackful:/ {for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]+=a[2]+0}} \
	            END {exit !(v["resumes"]>0 && v["waits"]>0 && (mode!="echo" || v["send_waits"]>0) && (mode!="quantum" || v["yields"]>0))}' "$$log"; \
	        fi; \
	      done; \
	    done; \
	    done; \
	   done; \
	  done; \
	done
	@if test "$(EPOLL_STORAGE_CHECK)" = 1; then \
	  set -e; command -v strace; strace --version; \
	  for storage in 0 3; do \
	    binary="$(BUILD)/storage-sanitized-s$$storage"; \
	    $(CLANG) $(CFLAGS) -g -Wpedantic -pthread -fsanitize=address,undefined -fno-sanitize-recover=all \
	      -DWF_BENCH_SNDBUF=4096 -DWF_BENCH_STORAGE_OBSERVE \
	      "-DWF_BENCH_RECEIVE_STORAGE=$$storage" epoll_echo.c -o "$$binary"; \
	    for workers in 1 4; do \
	      prefix="$$binary-w$$workers"; \
	      ASAN_OPTIONS=detect_leaks=1 "$(BUILD)/stream_check" "$$binary" echo "$$workers" > "$$prefix-sanitizer.log" 2>&1 || { cat "$$prefix-sanitizer.log"; exit 1; }; \
	      cat "$$prefix-sanitizer.log"; \
	      printf '#!/usr/bin/env bash\nset -euo pipefail\nexec strace --kill-on-exit -ff -o "%s" -e trace=execve,clone,clone3,sendto,recvfrom,close -e raw=sendto,recvfrom "%s" "$$@"\n' \
	        "$$prefix-trace" "$$binary" > "$$prefix-launch"; \
	      chmod +x "$$prefix-launch"; \
	      ASAN_OPTIONS=detect_leaks=0 "$(BUILD)/stream_check" "$$prefix-launch" echo "$$workers" > "$$prefix-trace.log" 2>&1 || { cat "$$prefix-trace.log"; exit 1; }; \
	      cat "$$prefix-trace.log"; \
	      for kind in sanitizer trace; do \
	        awk -v storage="$$storage" '/^storage:/ {for(i=2;i<=NF;i++) {split($$i,a,"=");v[a[1]]=a[2]+0}; seen++} \
	          END {exit !(seen==1 && ("policy" in v) && v["policy"]==storage && ("main_worker" in v) && v["main_worker"]==0 && \
	            v["transfer_bytes"]==65536 && v["accepted"]==4 && v["closed"]==4)}' "$$prefix-$$kind.log"; \
	      done; \
	      awk -v binary="$$binary" -v workers="$$workers" 'function number(s, i,n,d) { \
	        if(s ~ /^0x[0-9a-f]+$$/) {n=0; for(i=3;i<=length(s);i++) {d=index("0123456789abcdef",substr(s,i,1))-1; n=16*n+d}; return n} \
	        if(s ~ /^[0-9]+$$/) return s+0; bad=1; return -1 \
	      } function transfer(line, call,a,r,requested,moved,sending) { \
	        call=line; split(call,a,", "); requested=number(a[3]); \
	        if(call !~ /\) += /) {bad=1; return}; sub(/^.*\) += /,"",call); split(call,r," "); \
	        sending=(line ~ /^sendto/); \
	        calls[FILENAME]++; \
	        if(r[1]=="-1") {if(r[2]=="EAGAIN" && sending) again++; else if(r[2]!="EAGAIN" && r[2]!="EINTR") bad=1; return} \
	        moved=number(r[1]); if(moved>requested) bad=1; \
	        if(sending) sent+=moved; else received+=moved; \
	        if(moved>0 && moved<requested) {if(sending) short_send++; else short_recv++} \
	      } /^execve\(/ { \
	        if(index($$0,"execve(\"" binary "\",")==1 && / = 0$$/) {execs++; main_file=FILENAME} else bad=1 \
	      } /^clone3?\(|^<\.\.\. clone3? resumed>/ { \
	        if(/\) += [1-9][0-9]*$$/) children[$$NF]=1 \
	      } /^sendto\(|^recvfrom\(/ { \
	        if(FILENAME in pending) bad=1; \
	        if(/<unfinished \.\.\.>$$/) {pending[FILENAME]=$$0; sub(/<unfinished \.\.\.>$$/,"",pending[FILENAME])} \
	        else transfer($$0); next \
	      } /^<\.\.\. (sendto|recvfrom) resumed>/ { \
	        if(!(FILENAME in pending)) {bad=1; next}; \
	        resumed=$$0; sub(/^<\.\.\. /,"",resumed); sub(/ resumed>.*$$/,"",resumed); \
	        if(index(pending[FILENAME],resumed "(")!=1) bad=1; \
	        suffix=$$0; sub(/^<\.\.\. [a-z]+ resumed>/,"",suffix); \
	        transfer(pending[FILENAME] suffix); delete pending[FILENAME] \
	      } END {for(file in pending) bad=1; \
	        for(tid in children) child_count++; \
	        for(file in calls) {tid=file; sub(/^.*\./,"",tid); if(file==main_file || !(tid in children)) bad=1; \
	          printf "storage-trace-thread: file=%s tid=%s network_outcomes=%d\n",file,tid,calls[file]}; \
	        printf "storage-trace-process: executable=%s main_file=%s worker_threads=%d\n",binary,main_file,child_count; \
	        printf "storage-trace: receive_bytes=%.0f send_bytes=%.0f short_recv=%d short_send=%d send_eagain=%d\n",received,sent,short_recv,short_send,again; \
	        exit !(!bad && execs==1 && child_count==workers && received==8388608 && sent==received && short_recv>0 && short_send>0 && again>0)}' \
	        "$$prefix-trace".[0-9]* > "$$prefix-counts.log" || { cat "$$prefix-counts.log"; exit 1; }; \
	      cat "$$prefix-counts.log"; \
	    done; \
	  done; \
	else test "$(EPOLL_STORAGE_CHECK)" = 0; fi

$(BUILD)/wf_echo: programs/tcp_echo_server.wf compiler | prepare
	$(WFC) --par -o $@ $<

# The Whitefoot line joins this check when its program is in the bundle; the
# two references are always in it.
net-verify: net-tools $(if $(WF_ECHO_SOURCE),$(BUILD)/wf_echo)
	@env ROOT=$(ROOT) OUT=$(BUILD) sh linux-net-bench.sh verify

linux-net:
	@mkdir -p $(NET_OUT)
	env ROOT=$(ROOT) OUT=$(NET_OUT) CLANG=$(CLANG) \
		CARGO_TARGET_DIR=$(ROOT)/compiler/target \
		ROUNDS=$(NET_ROUNDS) WARMUP=$(NET_WARMUP) \
		sh linux-net-bench.sh

clean:
	rm -rf $(WORK)
