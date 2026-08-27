# Program-level I/O measurement for the unified-state completion model.
#
# Three lines per workload, all publishing the same bytes:
#   N  the best hand-written native shape (thread pool, io_uring)
#   S  the Whitefoot program built with --no-overlap, the sequential reference
#   C  the Whitefoot program built the way it ships, with completion overlap
#
# This is a measurement bundle, not a gate. It is deliberately not reachable
# from the repository's `make check`: it generates a large tree, runs for
# minutes, and reports timings that no build should depend on.
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
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-io-completion-bench
BUILD := $(WORK)/build
TREE := $(WORK)/tree
CARGO_TARGET := $(WORK)/cargo-target
LINUX_TARGET := $(WORK)/linux-target
CLANG := /usr/bin/clang
CFLAGS := -std=c11 -O2 -Wall -Wextra -Werror
WFC := $(ROOT)/compiler/target/gate/whitefootc

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

DOCKER_IMAGE ?= wf-io-bench:linux

.PHONY: build prepare compiler gen verify bench bench-pipe linux linux-image expected clean

build: prepare compiler $(BUILD)/gen $(BUILD)/baseline $(BUILD)/runner \
	$(BUILD)/pipe_producer $(BUILD)/pipe_harness \
	$(BUILD)/many_files_wide $(BUILD)/many_files_wide_seq \
	$(BUILD)/many_files_narrow $(BUILD)/many_files_narrow_seq \
	$(BUILD)/pipe_relay $(BUILD)/pipe_relay_seq

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

$(BUILD)/many_files_narrow_seq: programs/many_files_narrow.wf compiler | prepare
	$(WFC) --no-overlap -o $@ $<

$(BUILD)/pipe_relay_seq: programs/pipe_relay.wf compiler | prepare
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
		"$(BUILD)/many_files_wide" \
		"$(BUILD)/many_files_wide_seq"; do \
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
	printf 'S.wide\t\t$(BUILD)/many_files_wide_seq\n'; \
	printf 'C.narrow.default\t\t$(BUILD)/many_files_narrow\n'; \
	printf 'C.wide.default\t\t$(BUILD)/many_files_wide\n'; \
	for w in 0 1 2 4; do for h in 0 1 2 4; do \
		printf 'C.wide.w%s.h%s\tWF_WORKERS=%s,WF_IO_HELPERS=%s\t$(BUILD)/many_files_wide\n' \
			$$w $$h $$w $$h; \
	done; done; \
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

clean:
	rm -rf $(WORK)
