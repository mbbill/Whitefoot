CARGO := cargo
CC ?= cc
NM ?= nm
WINDOWS_CC ?= zig cc
WINDOWS_OBJDUMP ?= objdump
WINDOWS_IOCP_PROBE_DEFINES := \
	-DWF_WINDOWS_IOCP_READ_CALL=wf_windows_iocp_probe_read \
	-DWF_WINDOWS_IOCP_WRITE_CALL=wf_windows_iocp_probe_write \
	-DWF_WINDOWS_IOCP_RESULT_CALL=wf_windows_iocp_probe_result \
	-DWF_WINDOWS_IOCP_NOTIFICATION_CALL=wf_windows_iocp_probe_notification
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
	backend::tests::parallel:: \
	backend::tests::stackless::
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
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/bridge.c \
	$(COMPLETION_DIR)/linux_io_uring.c \
	$(COMPLETION_DIR)/writer_scheduler.c \
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
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/core_read_probe.c
COMPLETION_CORE_READ_ALLOWED_SOURCES := \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/core_read_probe.c
COMPLETION_CORE_READ_UNEXPECTED_SOURCES := $(filter-out \
	$(COMPLETION_CORE_READ_ALLOWED_SOURCES),$(COMPLETION_CORE_READ_SOURCES))
COMPLETION_CORE_READ_FORBIDDEN_SOURCES := $(filter \
	%/bridge.c %/writer_scheduler.c,$(COMPLETION_CORE_READ_SOURCES))
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

completion-test: completion-core-read-test completion-default-route-test sched-smoke
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
			$(COMPLETION_DIR)/runtime.c \
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
SCHED_DIR := src/backend/sched
sched-smoke:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) $(COMPLETION_BASE_CFLAGS) $(SCHED_DIR)/core.c $(SCHED_DIR)/prim_host.c \
		$(SCHED_DIR)/smoke.c -o "$(COMPLETION_TMP)/sched-smoke"
	"$(COMPLETION_TMP)/sched-smoke"

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
		grep -Eq 'wf__writer_run_root|wf__completion_file_pread_submit'; then \
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

# This is deliberately an explicit qualification-development target rather
# than part of `check`: it needs a Windows cross toolchain, and a successful
# cross-link is not a substitute for execution on Windows.  It keeps the
# target-only core and the opaque adapter ABI independently linkable, then
# proves that the full PE imports real IOCP and overlapped file facilities.
completion-windows-cross:
	@mkdir -p "$(COMPLETION_TMP)/windows-zig-global" \
		"$(COMPLETION_TMP)/windows-zig-local"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
		$(WINDOWS_CC) -target x86_64-windows-gnu \
			-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
			$(WINDOWS_IOCP_PROBE_DEFINES) \
			-I$(COMPLETION_DIR) -Isrc/backend \
		$(COMPLETION_DIR)/windows_completion.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/windows_completion_probe.c \
		-o "$(COMPLETION_TMP)/windows-completion-probe.exe"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		-I$(COMPLETION_DIR) -Isrc/backend \
		$(COMPLETION_DIR)/native_contract.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/native_adapter_probe.c \
		-o "$(COMPLETION_TMP)/windows-adapter-probe.exe"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		-I$(COMPLETION_DIR) -Isrc/backend \
		$(COMPLETION_DIR)/windows_completion.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/windows_blocking.c \
		$(COMPLETION_DIR)/windows_blocking_probe.c \
		-o "$(COMPLETION_TMP)/windows-blocking-probe.exe"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		-I$(COMPLETION_DIR) -Isrc/backend \
		src/backend/windows_runtime.c \
		$(COMPLETION_DIR)/windows_completion.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/windows_blocking.c \
		$(COMPLETION_DIR)/windows_bridge.c \
		$(COMPLETION_DIR)/writer_scheduler_windows.c \
		-shared -o "$(COMPLETION_TMP)/windows-runtime.dll"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		-I$(COMPLETION_DIR) -Isrc/backend \
		src/backend/windows_runtime.c \
		$(COMPLETION_DIR)/windows_completion.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/windows_blocking.c \
		$(COMPLETION_DIR)/windows_bridge.c \
		$(COMPLETION_DIR)/writer_scheduler_windows.c \
		$(COMPLETION_DIR)/windows_bridge_capacity_probe.c \
		-o "$(COMPLETION_TMP)/windows-bridge-capacity-probe.exe"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		-c src/backend/wf_floor_windows.c \
		-o "$(COMPLETION_TMP)/wf-floor-windows.o"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		src/backend/windows_namespace_probe.c \
		-o "$(COMPLETION_TMP)/windows-namespace-probe.exe"
	@imports="$$($(WINDOWS_OBJDUMP) -p \
		"$(COMPLETION_TMP)/windows-completion-probe.exe")"; \
	for symbol in CreateIoCompletionPort GetQueuedCompletionStatus \
		PostQueuedCompletionStatus ReadFile WriteFile \
		AcquireSRWLockExclusive SleepConditionVariableSRW; do \
		if ! printf '%s\n' "$$imports" | grep -q "  $$symbol$$"; then \
			echo "Windows completion probe does not import $$symbol" >&2; \
			exit 1; \
		fi; \
	done
	@imports="$$($(WINDOWS_OBJDUMP) -p \
		"$(COMPLETION_TMP)/windows-blocking-probe.exe")"; \
	for symbol in _beginthreadex CreateIoCompletionPort \
		GetQueuedCompletionStatus PostQueuedCompletionStatus; do \
		if ! printf '%s\n' "$$imports" | grep -q "  $$symbol$$"; then \
			echo "Windows blocking probe does not import $$symbol" >&2; \
			exit 1; \
		fi; \
	done
	@echo "completion Windows x86-64 cross-link: PASS (not runtime-qualified)"

.PHONY: check static format lint test test-unit test-sampling test-corpus \
	test-partition docs spec completion-test \
	completion-core-read-source-boundary completion-core-read-test \
	completion-core-read-stress completion-core-read-tsan completion-tsan \
	completion-sanitize \
	completion-core-read-sanitize completion-default-route-test \
	completion-default-route-sanitize completion-default-route-tsan sched-smoke \
	completion-windows-cross
