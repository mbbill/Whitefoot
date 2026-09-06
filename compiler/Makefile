CARGO := cargo
CC ?= cc
NM ?= nm
WINDOWS_CC ?= x86_64-w64-mingw32-gcc
WINE ?= /usr/lib/wine/wine64
WINEPREFIX ?= $(WHITEFOOT_SCRATCH_ROOT)/wineprefix
WINDOWS_OBJDUMP ?= objdump
WINDOWS_NM ?= x86_64-w64-mingw32-nm
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
STAGE_DIR := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-gate-stages

# Every gate target that executes a compiled program runs it with the core
# file limit at zero.
#
# A few runtime-boundary and deliberately damaged-lowering tests end a child
# process while checking resource reporting and test-oracle sensitivity. On a
# host that writes or analyses a core for each one, the gate pays for evidence
# nothing reads. Measured in batch 0093, that is where the Linux gate's time
# went: the same library suite took 1016 seconds on the four-core
# `ubuntu-24.04` runner and 55 on the three-core `macos-14` runner beside it.
#
# This limit is the portable half of declining that, and it is deliberately not
# the whole of it. A Linux `kernel.core_pattern` that begins with a pipe —
# which `ubuntu-24.04` has, naming `systemd-coredump` — makes the kernel ignore
# RLIMIT_CORE entirely, because the limit bounds the size of a file and a pipe
# writes none. Measured: forty aborting runs left forty stored core dumps at a
# soft limit of 0 and forty more at a soft limit of 1, and none once the
# pattern named a file. A host in that state has to change the pattern, which
# needs root; `.github/workflows/gate.yml` does exactly that in one step and
# says so there. What this line covers is every host whose pattern names a
# file, where a limit of zero declines the dump the ordinary way.
#
# Nothing a case asserts depends on a core file existing: the signal that kills
# the process, its exit status and its record channel are all unchanged.
NO_CORE_DUMPS := ulimit -c 0;

# The library suite, split by cost class rather than by subject. The modules
# named here are the ones whose cases link a program and then run it tens or
# hundreds of times to sample a schedule; everything else analyses or emits and
# is over in milliseconds. The split exists so the two halves can be scheduled
# side by side — by a CI job, or by a maintainer who wants the fast half back
# quickly — and `test` below still runs both, with `test-partition` proving
# that between them they run the whole suite.
SAMPLING_MODULES := \
	backend::tests::exhaustion:: \
	backend::tests::loop_split:: \
	backend::tests::parallel::
SAMPLING_SKIPS := $(patsubst %,--skip %,$(SAMPLING_MODULES))

# The integration targets cargo discovers in `tests/`. `test-corpus` names them
# one by one so a job runs exactly the ones it means to; `test-partition`
# checks this list against the directory, so a new `tests/*.rs` cannot fall out
# of the gate by being no one's business.
INTEGRATION_TARGETS := canonical_corpus conformance programs snapshot

# The stages `check` runs, in order, and the same targets the CI jobs run.
COMPILER_STAGES := format lint test-partition test-unit test-sampling test-corpus \
	docs spec completion-test

COMPLETION_DIR := src/backend/completion
# The scheduler core is linked into every completion build: the completion
# record begins with a `wf_sched_record` and every publication goes through
# `wf_sched_complete` (design §7).
SCHED_DIR := src/backend/sched
# `entry.c` is the platform layer over the core: the one core instance every
# unit that publishes or joins a record names, the startup policy, and the
# emitted module's `wf__par_*` ABI. It is part of the core's own group, so a
# link that carries `core.c` carries it (design §7's platform layer).
SCHED_SOURCES := $(SCHED_DIR)/core.c $(SCHED_DIR)/prim_host.c $(SCHED_DIR)/entry.c
COMPLETION_TMP ?= $(HOME)/do_not_scan/whitefoot-completion-test
COMPLETION_DEFINES := \
	-DWF_COMPLETION_GETDIRENTRIES64=wf_completion_test_getdirentries64 \
	-DWF_COMPLETION_GETDENTS64=wf_completion_test_getdents64 \
	-DWF_COMPLETION_POLL=wf_completion_test_poll
COMPLETION_BASE_CFLAGS := -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic \
	-pthread
COMPLETION_CFLAGS := $(COMPLETION_BASE_CFLAGS) $(COMPLETION_DEFINES)
# The harness names the WF_IO_NOCACHE application so it can count it. The
# named function performs the same host call the shipped policy performs, so
# only the counting is added, and only this build has it.
#
# It also names the clock the helper policy measures host calls with, so that
# what the growth rule decides can be tested by scripting the measurement
# instead of by sleeping and hoping the machine agreed. Absent a script the
# named clock answers with the host's own monotonic clock, exactly as the
# shipped build does.
#
# It further names the adapter open so a test can stand at the moment the
# host refuses one, and the ledger's schedule point so a test can stand
# between the two reads a give-up decision makes. The named functions
# perform the same host calls the shipped code performs and the point does
# nothing at all where it is unnamed, so only the observation is added.
#
# The defines are named once, here, and used by every build that links the
# harness: a sanitizer build that named a different set would run a different
# runtime from the one the ordinary build runs.
COMPLETION_HARNESS_DEFINES := \
	-DWF_FILE_UNCACHED_APPLY=wf_completion_test_uncached_apply \
	-DWF_FILE_MONOTONIC_NS=wf_completion_test_monotonic_ns \
	-DWF_FILE_OPENAT=wf_completion_test_openat
COMPLETION_HARNESS_CFLAGS := $(COMPLETION_CFLAGS) $(COMPLETION_HARNESS_DEFINES)
COMPLETION_SOURCES := \
	$(SCHED_SOURCES) \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/wait_host.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/file_posix.c \
	$(COMPLETION_DIR)/bridge.c \
	$(COMPLETION_DIR)/linux_io_uring.c \
	$(COMPLETION_DIR)/native_contract.c \
	$(COMPLETION_DIR)/harness.c
# The same runtime the harness links, with the probe in place of the harness.
# It is deliberately built without COMPLETION_DEFINES: the route this probe is
# about is the shipped one, so it makes the shipped host calls and reads the
# host's own clock.
COMPLETION_DEFAULT_ROUTE_SOURCES := \
	$(filter-out %/harness.c,$(COMPLETION_SOURCES)) \
	$(COMPLETION_DIR)/bridge_default_probe.c
# The same runtime again, with the probe that puts a refused open on every
# route at once. Built without COMPLETION_DEFINES for the same reason as the
# default-route probe: the schedule it is about is the shipped one.
COMPLETION_CORE_READ_SOURCES := \
	$(SCHED_SOURCES) \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/wait_host.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/file_posix.c \
	$(COMPLETION_DIR)/core_read_probe.c
COMPLETION_CORE_READ_ALLOWED_SOURCES := \
	$(SCHED_SOURCES) \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/wait_host.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/file_posix.c \
	$(COMPLETION_DIR)/core_read_probe.c
COMPLETION_CORE_READ_UNEXPECTED_SOURCES := $(filter-out \
	$(COMPLETION_CORE_READ_ALLOWED_SOURCES),$(COMPLETION_CORE_READ_SOURCES))
COMPLETION_CORE_READ_FORBIDDEN_SOURCES := $(filter \
	%/bridge.c,$(COMPLETION_CORE_READ_SOURCES))
COMPLETION_CORE_READ_DEFINES := \
	-DWF_COMPLETION_PREAD=wf_completion_test_pread

check:
	@mkdir -p "$(STAGE_DIR)"
	@: > "$(STAGE_DIR)/compiler-summary"
	@for stage in $(COMPILER_STAGES); do \
		started=$$(date +%s); \
		$(MAKE) --no-print-directory "$$stage" || exit 1; \
		printf '  compiler/%-18s %6d s\n' "$$stage" \
			"$$(( $$(date +%s) - started ))" >> "$(STAGE_DIR)/compiler-summary"; \
	done
	@echo ""
	@echo "compiler stage wall, this run:"
	@cat "$(STAGE_DIR)/compiler-summary"
	@echo "== WHITEFOOT COMPILER GATE GREEN =="

# Everything that reads the tree without running a compiled program. One CI job
# runs exactly this while the test jobs run beside it.
static: format lint docs spec completion-test
	@echo "== WHITEFOOT COMPILER STATIC GATE GREEN =="

format:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets --locked --offline -- -D warnings

# The gate profile (Cargo.toml [profile.gate]) is release codegen with debug
# assertions and overflow checks retained: the suite's owning tests analyze
# real programs, ~16x faster there than at the unoptimized default. A bare
# `cargo test <filter>` during development still uses the default profile
# and keeps incremental builds cheap.
#
# `test` is the whole suite, exactly as `--all-targets` was: the four targets
# below partition it and `test-partition` is what proves they do.
test: test-partition test-unit test-sampling test-corpus

# The fast half of the library suite: everything the sampling modules above
# are not.
test-unit:
	$(NO_CORE_DUMPS) $(CARGO) test --profile gate --lib --locked --offline -- $(SAMPLING_SKIPS)

# The sampling half: the cases that link a program and run it many times.
test-sampling:
	$(NO_CORE_DUMPS) $(CARGO) test --profile gate --lib --locked --offline -- $(SAMPLING_MODULES)

# Everything outside the library: the integration targets, which compile and
# link real programs, and the binaries' own unit tests. The binaries are here
# rather than beside the library half because linking their four test harnesses
# costs more than running their cases does, and this is the job with the slack
# for it.
test-corpus:
	$(NO_CORE_DUMPS) $(CARGO) test --profile gate --bins $(patsubst %,--test %,$(INTEGRATION_TARGETS)) --locked --offline

# The split above is a scheduling decision, and a scheduling decision that
# quietly drops cases is the worst kind of green. This proves it does not:
# the two library halves partition the suite exactly — every case is in one of
# them and none is in both — and the integration targets `test-corpus` names
# are exactly the ones cargo discovers in `tests/`. The arithmetic alone cannot
# notice a sampling module that was renamed or deleted, because `--skip M` and
# the filter `M` are exact complements whatever M names; so every name in
# SAMPLING_MODULES must also match at least one case on its own. Listing costs
# a `--no-run` build and no test time.
test-partition:
	@whole=$$($(CARGO) test --profile gate --lib --locked --offline -- --list \
		| grep -c ': test$$'); \
	fast=$$($(CARGO) test --profile gate --lib --locked --offline -- --list $(SAMPLING_SKIPS) \
		| grep -c ': test$$'); \
	sampled=$$($(CARGO) test --profile gate --lib --locked --offline -- --list $(SAMPLING_MODULES) \
		| grep -c ': test$$'); \
	if [ "$$whole" -ne "$$(( fast + sampled ))" ]; then \
		echo "library split is not a partition: $$whole cases, $$fast fast + $$sampled sampled" >&2; \
		exit 1; \
	fi; \
	if [ "$$sampled" -eq 0 ]; then \
		echo "library split names no sampling case: SAMPLING_MODULES matches nothing" >&2; \
		exit 1; \
	fi; \
	for module in $(SAMPLING_MODULES); do \
		count=$$($(CARGO) test --profile gate --lib --locked --offline -- --list $$module \
			| grep -c ': test$$'); \
		if [ "$$count" -eq 0 ]; then \
			echo "library split names a sampling module with no case: $$module" >&2; \
			exit 1; \
		fi; \
	done; \
	echo "library split: $$whole cases = $$fast fast + $$sampled sampled"
	@declared="$$(printf '%s\n' $(INTEGRATION_TARGETS) | sort | tr '\n' ' ')"; \
	present="$$(ls tests/*.rs | sed -e 's|tests/||' -e 's|\.rs$$||' | sort | tr '\n' ' ')"; \
	if [ "$$declared" != "$$present" ]; then \
		echo "integration targets declared [$$declared] but tests/ holds [$$present]" >&2; \
		exit 1; \
	fi; \
	echo "integration targets: $$present"

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --locked --offline

spec:
	$(CARGO) run --bin whitefoot-spec --locked --offline

completion-test: completion-core-read-test completion-default-route-test sched-smoke \
	sched-enumerate
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) $(COMPLETION_HARNESS_CFLAGS) $(COMPLETION_SOURCES) -o "$(COMPLETION_TMP)/harness"
	@if $(NM) "$(COMPLETION_TMP)/harness" | \
		grep -Eq 'wf_completion_claim_many|wf__completion_file_batch_claim|wf__completion_file_.*_submit_reserved|wf__completion_output_batch'; then \
		echo "completion runtime exports removed group/root machinery" >&2; \
		exit 1; \
	fi
	WF_IO_HELPERS=0 "$(COMPLETION_TMP)/harness" "$(COMPLETION_TMP)"
	WF_IO_HELPERS=1 "$(COMPLETION_TMP)/harness" "$(COMPLETION_TMP)"
	WF_IO_HELPERS=4 "$(COMPLETION_TMP)/harness" "$(COMPLETION_TMP)"
	@# WF_IO_NOCACHE is bench-only target policy: the three runs above leave it
	@# absent and assert the policy makes no host call at all, and this one
	@# writes it and asserts one application per descriptor an open hands back.
	WF_IO_NOCACHE=1 WF_IO_HELPERS=1 "$(COMPLETION_TMP)/harness" "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		$(COMPLETION_DIR)/pure_compute_probe.c -o "$(COMPLETION_TMP)/pure-compute"
	@if $(NM) "$(COMPLETION_TMP)/pure-compute" | grep -q 'wf_completion_'; then \
		echo "pure-compute probe unexpectedly links completion runtime" >&2; \
		exit 1; \
	fi
	@echo "completion pure-compute link boundary: PASS"
	@if [ "$$(uname -s)" = Linux ]; then \
		$(CC) $(COMPLETION_CFLAGS) \
			$(SCHED_SOURCES) \
			$(COMPLETION_DIR)/runtime.c \
			$(COMPLETION_DIR)/wait_host.c \
			$(COMPLETION_DIR)/linux_io_uring.c \
			$(COMPLETION_DIR)/native_adapter_probe.c \
			-o "$(COMPLETION_TMP)/linux-native-probe"; \
		"$(COMPLETION_TMP)/linux-native-probe" \
			"$(COMPLETION_TMP)/linux-native-probe.data"; \
		status=$$?; \
		if [ $$status -ne 0 ] && [ $$status -ne 77 ]; then exit $$status; fi; \
		if [ $$status -eq 0 ]; then \
			WF_REQUIRE_LINUX_IO_URING=1 WF_IO_HELPERS=1 \
				"$(COMPLETION_TMP)/harness" "$(COMPLETION_TMP)"; \
		fi; \
	else \
		$(CC) -std=c11 -Wall -Wextra -Werror -Wpedantic -pthread \
			-fsyntax-only $(COMPLETION_DIR)/linux_io_uring.c; \
	fi

# The bridge on the helper policy a delivered program actually gets.
#
# Every harness run names WF_IO_HELPERS -- the three arms above name 0, 1 and 4
# and harness main pins 1 for any run that named nothing -- so the demand-driven
# pool and the declined positioned read, both of which choose between two routes
# for one writer-visible operation, were reachable from no test in the tree.
# `unset` rather than a bare invocation because make inherits this shell's
# environment, and the probe refuses an inherited value rather than quietly
# testing the arm the harness already covers.
completion-default-route-test:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) $(COMPLETION_BASE_CFLAGS) $(COMPLETION_DEFAULT_ROUTE_SOURCES) \
		-o "$(COMPLETION_TMP)/bridge-default-probe"
	unset WF_IO_HELPERS; \
		"$(COMPLETION_TMP)/bridge-default-probe" "$(COMPLETION_TMP)"
	@# The adapter branch of the probe's route assertion -- the demand-driven
	@# policy declined a read, or grew a helper -- cannot fire on a Linux host
	@# whose io_uring takes every positioned read, so on Linux the arm above
	@# covers only the native ring and the negative control was carried by the
	@# Darwin CI host alone. WF_IO_NO_NATIVE_RING puts this host on the route a
	@# kernel without io_uring already takes, and the probe refuses to pass if
	@# the ring answered anyway.
	@if [ "$$(uname -s)" = Linux ]; then \
		unset WF_IO_HELPERS; WF_IO_NO_NATIVE_RING=1 \
			"$(COMPLETION_TMP)/bridge-default-probe" "$(COMPLETION_TMP)"; \
	fi

# The scheduler core (src/backend/sched) on real threads, the real switch and
# the real park: not the §11 gate, which is the enumerator, but the run that
# has to pass before an enumeration is worth building.
sched-smoke:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) $(COMPLETION_BASE_CFLAGS) $(SCHED_DIR)/core.c $(SCHED_DIR)/prim_host.c \
		$(SCHED_DIR)/smoke.c -o "$(COMPLETION_TMP)/sched-smoke"
	"$(COMPLETION_TMP)/sched-smoke"

# The §11 gate: every interleaving of the core's primitive steps at the four
# configurations §11 derives from §5's floor, (T=1,S=2), (T=1,S=3), (T=2,S=3)
# and (T=2,S=4), with every invariant checked after every step rather than
# once at the end. Two lane slots because I4's second half wants a power of two.
#
# The idle window's bounded spin is pinned here for the same reason the lane
# slot count is: under `WF_SCHED_ENUMERATE` a spin round is not a delay but a
# repetition of the window's looks, and every look is a primitive step the
# search branches on. One spin round reaches both of the spin's arms -- a look
# that hits during it, and the fall through it to the park -- and costs the
# search 9.4 times the states at (2,4): 434 643 223 before the spin and
# 4 086 519 269 with it, 47 seconds against 157. A second round costs 24 times
# the states at (2,3) and is not worth what it adds.
#
# The yield rounds are pinned to zero, and that is a statement about the model
# rather than about the spin. The enumerator makes a yield block until another
# process has written shared state, which is the only way a spin can observe
# anything in a sequential model; so a yield in front of the park forces every
# device completion ahead of the park, and the one thread then never sleeps on
# the primitive at all. S10a asserts that it does, and fails with "not covered:
# the one thread never slept on the primitive" at one yield round with no spin
# rounds at all; at (2,4) two spin rounds and one yield round fail seventeen
# schedules with "thread 1 spins with nothing left to change what it spins on",
# which is the same model fact from the other side -- a yield the search cannot
# enable is a stopped thread. A yield is a delay, like the pause a spin round
# takes; the looks around it are what section 11 judges, and one spin round
# runs them.
sched-enumerate:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) $(COMPLETION_BASE_CFLAGS) -DWF_SCHED_ENUMERATE -DWF_SCHED_LANE_SLOTS=2u \
		-DWF_SCHED_MAX_THREADS=4u -DWF_SCHED_MAX_STACKS=8u \
		-DWF_SCHED_IDLE_SPIN_ROUNDS=1u -DWF_SCHED_IDLE_YIELD_ROUNDS=0u \
		$(SCHED_DIR)/core.c $(SCHED_DIR)/enumerate.c $(SCHED_DIR)/schedules.c \
		-o "$(COMPLETION_TMP)/sched-enumerate"
	@# The four sweeps are independent and the two-thread ones are long, so
	@# they run together and the target fails if any of them does.
	@status=0; pids=""; \
	for configuration in "1 2" "1 3" "2 3" "2 4"; do \
		set -- $$configuration; \
		"$(COMPLETION_TMP)/sched-enumerate" --threads $$1 --stacks $$2 \
			> "$(COMPLETION_TMP)/sched-enumerate-t$$1s$$2.log" 2>&1 & \
		pids="$$pids $$!"; \
	done; \
	for pid in $$pids; do wait "$$pid" || status=1; done; \
	for configuration in "1 2" "1 3" "2 3" "2 4"; do \
		set -- $$configuration; \
		grep -v '^enumerate: \.\.\.' "$(COMPLETION_TMP)/sched-enumerate-t$$1s$$2.log"; \
	done; \
	exit $$status

completion-default-route-sanitize:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		-fsanitize=address,undefined \
		$(COMPLETION_DEFAULT_ROUTE_SOURCES) \
		-o "$(COMPLETION_TMP)/bridge-default-probe-sanitize"
	unset WF_IO_HELPERS; ASAN_OPTIONS=detect_leaks=0 \
		"$(COMPLETION_TMP)/bridge-default-probe-sanitize" "$(COMPLETION_TMP)"

# Thread sanitizer over the bridge rather than over the isolated core: the
# helper pool, the queue wake and the join spin are all on this route, and the
# core probe by construction links none of them.
completion-default-route-tsan:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		-fsanitize=thread \
		$(COMPLETION_DEFAULT_ROUTE_SOURCES) \
		-o "$(COMPLETION_TMP)/bridge-default-probe-tsan"
	unset WF_IO_HELPERS; TSAN_OPTIONS=halt_on_error=1 \
		"$(COMPLETION_TMP)/bridge-default-probe-tsan" "$(COMPLETION_TMP)"

completion-core-read-source-boundary:
	@if [ -n "$(strip $(COMPLETION_CORE_READ_UNEXPECTED_SOURCES) \
		$(COMPLETION_CORE_READ_FORBIDDEN_SOURCES))" ]; then \
		echo "completion core/read source list crosses the isolated boundary: \
			$(COMPLETION_CORE_READ_UNEXPECTED_SOURCES) \
			$(COMPLETION_CORE_READ_FORBIDDEN_SOURCES)" >&2; \
		exit 1; \
	fi

completion-core-read-test: completion-core-read-source-boundary
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		$(COMPLETION_CORE_READ_DEFINES) \
		$(COMPLETION_CORE_READ_SOURCES) \
		-o "$(COMPLETION_TMP)/core-read-probe"
	"$(COMPLETION_TMP)/core-read-probe" "$(COMPLETION_TMP)"
	@if $(NM) "$(COMPLETION_TMP)/core-read-probe" | \
		grep -Eq 'wf__completion_file_pread_submit'; then \
		echo "isolated completion core/read probe links the generated-code bridge" >&2; \
		exit 1; \
	fi
	@echo "completion core/read bridge isolation: PASS"

completion-sanitize: completion-core-read-sanitize \
		completion-default-route-sanitize
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		$(COMPLETION_DEFINES) $(COMPLETION_HARNESS_DEFINES) \
		-fsanitize=address,undefined \
		$(COMPLETION_SOURCES) \
		-o "$(COMPLETION_TMP)/harness-sanitize"
	ASAN_OPTIONS=detect_leaks=0 "$(COMPLETION_TMP)/harness-sanitize" "$(COMPLETION_TMP)"

completion-core-read-sanitize: completion-core-read-source-boundary
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		$(COMPLETION_CORE_READ_DEFINES) -fsanitize=address,undefined \
		$(COMPLETION_CORE_READ_SOURCES) \
		-o "$(COMPLETION_TMP)/core-read-probe-sanitize"
	ASAN_OPTIONS=detect_leaks=0 \
		"$(COMPLETION_TMP)/core-read-probe-sanitize" "$(COMPLETION_TMP)"

# These are qualification-development targets rather than canonical gates.
# Stress repeats the sleep/publication races; TSan is optional because not
# every supported build host ships a thread-sanitizer runtime.
completion-core-read-stress: completion-core-read-test
	@iteration=0; \
	while [ $$iteration -lt 200 ]; do \
		"$(COMPLETION_TMP)/core-read-probe" "$(COMPLETION_TMP)" >/dev/null \
			|| exit 1; \
		iteration=$$((iteration + 1)); \
	done
	@echo "completion core/read stress: PASS (200 runs)"

# The bridge and the native adapters, under the thread sanitizer.
#
# `completion-core-read-tsan` below covers the isolated core and the bounded
# POSIX adapter and deliberately links neither the bridge nor the ring. Those
# two carry their own shared state — the deferred doorbell's staging, the
# retire-and-retry hand-back, the helper policy's readiness flags — and nothing
# was checking it. This runs the whole harness instead, at the helper counts
# `completion-test` runs it at, which is where a helper and a joining scheduler
# actually contend.
completion-tsan:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		$(COMPLETION_DEFINES) $(COMPLETION_HARNESS_DEFINES) \
		-fsanitize=thread \
		$(COMPLETION_SOURCES) \
		-o "$(COMPLETION_TMP)/harness-tsan"
	TSAN_OPTIONS=halt_on_error=1 WF_IO_HELPERS=0 \
		"$(COMPLETION_TMP)/harness-tsan" "$(COMPLETION_TMP)"
	TSAN_OPTIONS=halt_on_error=1 WF_IO_HELPERS=1 \
		"$(COMPLETION_TMP)/harness-tsan" "$(COMPLETION_TMP)"
	TSAN_OPTIONS=halt_on_error=1 WF_IO_HELPERS=4 \
		"$(COMPLETION_TMP)/harness-tsan" "$(COMPLETION_TMP)"

completion-core-read-tsan: completion-core-read-source-boundary
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		$(COMPLETION_CORE_READ_DEFINES) -fsanitize=thread \
		$(COMPLETION_CORE_READ_SOURCES) \
		-o "$(COMPLETION_TMP)/core-read-probe-tsan"
	TSAN_OPTIONS=halt_on_error=1 \
		"$(COMPLETION_TMP)/core-read-probe-tsan" "$(COMPLETION_TMP)"

# The Windows link set, in the same three groups the driver stages: the
# scheduler core with its Windows leaf, the completion runtime with its Windows
# wait, host leaf and ring, and the Windows host runtime beside them. It is the
# same list as the POSIX one with the leaves swapped, which is the whole point
# of step (iv).
WINDOWS_LINK_SOURCES := \
	$(SCHED_DIR)/core.c \
	$(SCHED_DIR)/prim_windows.c \
	$(SCHED_DIR)/entry.c \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/wait_windows.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/file_windows.c \
	$(COMPLETION_DIR)/bridge.c \
	$(COMPLETION_DIR)/windows_iocp.c \
	src/backend/windows_runtime.c
WINDOWS_CFLAGS := -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic \
	-I$(COMPLETION_DIR) -Isrc/backend
# Winsock is the one facility this target does not get from the import
# libraries a link takes by default, so a link that carries the TCP routes
# names it (`src/bin/whitefootc.rs`, TARGET_LINK_LIBRARIES, says the same for
# the driver's own links).
WINDOWS_LINK_LIBRARIES := -lws2_32

# These two are deliberately explicit qualification-development targets rather
# than part of `check`, and they are two rather than one because they prove
# different things and neither is the thing the gate needs.
#
# A Windows cross build is not execution on Windows: it proves that these units
# compile and link into a PE importing the real IOCP, fiber and condition
# facilities, and nothing about what they do when they run. And wine is not
# Windows: it runs the PE well enough to exercise the port, the fibers and the
# adapter route, which makes it a useful early signal and never a substitute
# for `.github/workflows/io-hosts.yml`'s `completion-windows` job on a real
# host. The floor's classification of a stack overflow, in particular, is a
# kernel behaviour wine only approximates.
completion-windows-cross:
	@mkdir -p "$(COMPLETION_TMP)"
	$(WINDOWS_CC) $(WINDOWS_CFLAGS) \
		$(WINDOWS_LINK_SOURCES) \
		$(COMPLETION_DIR)/bridge_default_probe.c \
		-o "$(COMPLETION_TMP)/windows-default-route-probe.exe" \
		$(WINDOWS_LINK_LIBRARIES)
	$(WINDOWS_CC) $(WINDOWS_CFLAGS) \
		$(SCHED_DIR)/core.c \
		$(SCHED_DIR)/prim_windows.c \
		$(COMPLETION_DIR)/runtime.c \
		$(COMPLETION_DIR)/wait_windows.c \
		$(COMPLETION_DIR)/native_contract.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		src/backend/windows_runtime.c \
		$(COMPLETION_DIR)/native_adapter_probe.c \
		-o "$(COMPLETION_TMP)/windows-adapter-probe.exe" \
		$(WINDOWS_LINK_LIBRARIES)
	$(WINDOWS_CC) $(WINDOWS_CFLAGS) \
		-c src/backend/wf_floor_windows.c \
		-o "$(COMPLETION_TMP)/wf-floor-windows.o"
	$(WINDOWS_CC) $(WINDOWS_CFLAGS) \
		-c src/backend/windows_runtime.c \
		-o "$(COMPLETION_TMP)/windows-runtime.o"
	$(WINDOWS_CC) $(WINDOWS_CFLAGS) \
		src/backend/windows_namespace_probe.c \
		-o "$(COMPLETION_TMP)/windows-namespace-probe.exe" \
		$(WINDOWS_LINK_LIBRARIES)
	@imports="$$($(WINDOWS_OBJDUMP) -p \
		"$(COMPLETION_TMP)/windows-default-route-probe.exe")"; \
	for symbol in CreateIoCompletionPort GetQueuedCompletionStatus \
		PostQueuedCompletionStatus ReadFile WriteFile \
		CreateFiberEx SwitchToFiber ConvertThreadToFiberEx \
		AcquireSRWLockExclusive SleepConditionVariableSRW \
		VirtualAlloc _beginthreadex; do \
		if ! printf '%s\n' "$$imports" | grep -q "  $$symbol$$"; then \
			echo "Windows default-route probe does not import $$symbol" >&2; \
			exit 1; \
		fi; \
	done
	@# The Windows host runtime does not depend on the completion runtime or
	@# the scheduler core; the dependency runs the other way. A program that
	@# submits nothing and hands out nothing stages the floor group alone, so
	@# a single call from `windows_runtime.c` into either of the other groups
	@# is an unresolved external in every such link -- and that link is on the
	@# real host and nowhere else, which is how one crossed the gate. This is
	@# the same fact stated where it can be checked in seconds: the symbols
	@# the floor group leaves undefined, minus what it answers itself, may not
	@# include anything the other two groups define.
	@#
	@# "Answers itself" includes its weak defaults. `wf__sched_entry_stack` is
	@# the core's own seam (design section 5): the floor declares it weakly in
	@# its own translation unit, a link that carries the core overrides it, and
	@# a link that does not takes the default beside the reference. A COFF weak
	@# external is reported undefined (`w`) as well as defined, so the `w` and
	@# `V` classes join the defined set or the seam would read as a crossing.
	@$(WINDOWS_NM) -u \
		"$(COMPLETION_TMP)/wf-floor-windows.o" \
		"$(COMPLETION_TMP)/windows-runtime.o" \
		| awk '{print $$2}' | sort -u > "$(COMPLETION_TMP)/floor-group.undef"
	@$(WINDOWS_NM) \
		"$(COMPLETION_TMP)/wf-floor-windows.o" \
		"$(COMPLETION_TMP)/windows-runtime.o" \
		| awk 'NF >= 2 && $$(NF - 1) ~ /^[TDBRtwWVv]$$/ {print $$NF}' | sort -u \
		> "$(COMPLETION_TMP)/floor-group.def"
	@rm -rf "$(COMPLETION_TMP)/group-objects"
	@mkdir -p "$(COMPLETION_TMP)/group-objects"
	@for source in $(SCHED_DIR)/core.c $(SCHED_DIR)/prim_windows.c \
		$(SCHED_DIR)/entry.c $(COMPLETION_DIR)/runtime.c \
		$(COMPLETION_DIR)/wait_windows.c $(COMPLETION_DIR)/file_adapter.c \
		$(COMPLETION_DIR)/file_windows.c $(COMPLETION_DIR)/bridge.c \
		$(COMPLETION_DIR)/windows_iocp.c; do \
		$(WINDOWS_CC) $(WINDOWS_CFLAGS) -c "$$source" \
			-o "$(COMPLETION_TMP)/group-objects/$$(basename "$$source" .c).o" \
			|| exit 1; \
	done
	@$(WINDOWS_NM) --defined-only "$(COMPLETION_TMP)/group-objects"/*.o \
		| awk '$$2 ~ /^[TDBRt]$$/ {print $$3}' | sort -u \
		> "$(COMPLETION_TMP)/other-groups.def"
	@crossed="$$(comm -23 \
		"$(COMPLETION_TMP)/floor-group.undef" \
		"$(COMPLETION_TMP)/floor-group.def" \
		| comm -12 - "$(COMPLETION_TMP)/other-groups.def")"; \
	if [ -n "$$crossed" ]; then \
		echo "the Windows host runtime reaches the completion runtime or the scheduler core:" >&2; \
		printf '%s\n' "$$crossed" >&2; \
		exit 1; \
	fi
	@echo "completion Windows x86-64 cross-link: PASS (not runtime-qualified)"

# Runs what the cross build produced, under wine, when a wine is installed.
# A missing wine is not a failure of this target: it is a development aid, not
# evidence, and the evidence is the native job.
completion-windows-wine: completion-windows-cross
	@if [ ! -x "$(WINE)" ]; then \
		echo "completion Windows wine run: SKIP ($(WINE) is not installed)"; \
		exit 0; \
	fi; \
	mkdir -p "$(WINEPREFIX)"; \
	set -e; \
	echo "== native adapter probe =="; \
	WINEDEBUG=-all WINEPREFIX="$(WINEPREFIX)" "$(WINE)" \
		"$(COMPLETION_TMP)/windows-adapter-probe.exe" \
		"$(COMPLETION_TMP)/windows-adapter-probe.data"; \
	echo "== default route, native ring =="; \
	WINEDEBUG=-all WINEPREFIX="$(WINEPREFIX)" "$(WINE)" \
		"$(COMPLETION_TMP)/windows-default-route-probe.exe" \
		"$(COMPLETION_TMP)" 2>&1 | tee "$(COMPLETION_TMP)/windows-ring.txt"; \
	grep -q 'route=native-ring' "$(COMPLETION_TMP)/windows-ring.txt"; \
	echo "== default route, bounded adapter =="; \
	WF_IO_NO_NATIVE_RING=1 WINEDEBUG=-all WINEPREFIX="$(WINEPREFIX)" "$(WINE)" \
		"$(COMPLETION_TMP)/windows-default-route-probe.exe" \
		"$(COMPLETION_TMP)" 2>&1 | tee "$(COMPLETION_TMP)/windows-adapter.txt"; \
	grep -q 'route=posix-adapter' "$(COMPLETION_TMP)/windows-adapter.txt"; \
	echo "== the TCP round trip, on both routes =="; \
	if ! grep -q 'tcp-port=[1-9]' "$(COMPLETION_TMP)/windows-ring.txt" \
		|| ! grep -q 'tcp-port=[1-9]' "$(COMPLETION_TMP)/windows-adapter.txt"; then \
		echo "wine ran no loopback round trip on one of the two routes" >&2; \
		exit 1; \
	fi; \
	if grep -q 'tcp-ring=[1-9]' "$(COMPLETION_TMP)/windows-ring.txt"; then \
		echo "wine's Winsock carried the connect, the receive and the send on the completion port"; \
	else \
		echo "wine's Winsock ran the loopback round trip but carried none of it on the completion port; a real Windows host is what judges that (.github/workflows/io-hosts.yml, completion-windows)"; \
	fi; \
	if ! grep -q 'tcp-ring=0' "$(COMPLETION_TMP)/windows-adapter.txt"; then \
		echo "the refused-ring run still reached the completion port" >&2; \
		exit 1; \
	fi; \
	echo "the accept is the shared file adapter's on every Windows host, not only under wine: an AcceptEx address pair is 88 bytes and the record has none to give (completion/windows_iocp.c, wf_windows_iocp_carries), so wine's own AcceptEx is not in question here"; \
	echo "completion Windows wine run: PASS (wine is not Windows)"


# The measurements of design section 12 and of the plan's added choices, taken
# with one method and recorded in
# `research/experiments/park-on-miss-measurements/README.md`.
#
# This is a measurement and not a gate. It is deliberately outside `check`: it
# builds the compiler, generates a tree, runs for many minutes, and reports
# timings that no build may depend on. It is here rather than in the
# experiment's own Makefile because everything it measures is built out of this
# directory: every program it times is linked against this file's runtime, and
# the lane-slot sweep is a define of `sched/core.h`. Slice 4b removed the
# compile-time variants it swept, and with them the two sections that ran this
# file's `format`, `lint` and `completion-test` once per variant, so what
# remains measures the shipped form.
#
#   make -C compiler park-on-miss-measurements
#   make -C compiler park-on-miss-measurements SECTIONS="park compute"
#
# It is removed when design section 12's remaining items are answered or
# retired.
PARK_ON_MISS_MEASUREMENTS := \
	../research/experiments/park-on-miss-measurements/run.sh
SECTIONS ?=

park-on-miss-measurements:
	sh $(PARK_ON_MISS_MEASUREMENTS) $(SECTIONS)
.PHONY: check static format lint test test-unit test-sampling test-corpus \
	test-partition docs spec completion-test \
	completion-core-read-source-boundary completion-core-read-test \
	completion-core-read-stress completion-core-read-tsan completion-tsan \
	completion-sanitize \
	completion-core-read-sanitize completion-default-route-test \
	completion-default-route-sanitize completion-default-route-tsan sched-smoke \
	sched-enumerate completion-windows-cross completion-windows-wine \
	park-on-miss-measurements
