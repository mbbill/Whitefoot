CARGO := cargo
CC ?= cc
NM ?= nm
WINDOWS_CC ?= zig cc
WINDOWS_OBJDUMP ?= objdump

COMPLETION_DIR := src/backend/completion
COMPLETION_TMP ?= $(HOME)/do_not_scan/whitefoot-completion-test
COMPLETION_CFLAGS := -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread \
	-DWF_COMPLETION_GETDIRENTRIES64=wf_completion_test_getdirentries64 \
	-DWF_COMPLETION_POLL=wf_completion_test_poll
# The harness names the WF_IO_NOCACHE application so it can count it. The
# named function performs the same host call the shipped policy performs, so
# only the counting is added, and only this build has it.
COMPLETION_HARNESS_CFLAGS := $(COMPLETION_CFLAGS) \
	-DWF_FILE_UNCACHED_APPLY=wf_completion_test_uncached_apply
COMPLETION_SOURCES := \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/bridge.c \
	$(COMPLETION_DIR)/linux_io_uring.c \
	$(COMPLETION_DIR)/writer_scheduler.c \
	$(COMPLETION_DIR)/native_contract.c \
	$(COMPLETION_DIR)/harness.c
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

check: format lint test docs spec completion-test
	@echo "== WHITEFOOT COMPILER GATE GREEN =="

format:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets --locked --offline -- -D warnings

# The gate profile (Cargo.toml [profile.gate]) is release codegen with debug
# assertions and overflow checks retained: the suite's owning tests analyze
# real programs, ~16x faster there than at the unoptimized default, so the
# whole suite runs in one gate with no fast/heavy split. A bare
# `cargo test <filter>` during development still uses the default profile
# and keeps incremental builds cheap.
test:
	$(CARGO) test --profile gate --all-targets --locked --offline

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --locked --offline

spec:
	$(CARGO) run --bin whitefoot-spec --locked --offline

completion-test: completion-core-read-test
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

completion-sanitize: completion-core-read-sanitize
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		-DWF_COMPLETION_GETDIRENTRIES64=wf_completion_test_getdirentries64 \
		-DWF_COMPLETION_POLL=wf_completion_test_poll \
		-fsanitize=address,undefined \
		-DWF_FILE_UNCACHED_APPLY=wf_completion_test_uncached_apply \
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
		-I$(COMPLETION_DIR) \
		$(COMPLETION_DIR)/windows_completion.c \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/windows_completion_probe.c \
		-o "$(COMPLETION_TMP)/windows-completion-probe.exe"
	ZIG_GLOBAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-global" \
	ZIG_LOCAL_CACHE_DIR="$(COMPLETION_TMP)/windows-zig-local" \
	$(WINDOWS_CC) -target x86_64-windows-gnu \
		-std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		-I$(COMPLETION_DIR) \
		$(COMPLETION_DIR)/windows_iocp.c \
		$(COMPLETION_DIR)/native_adapter_probe.c \
		-o "$(COMPLETION_TMP)/windows-adapter-probe.exe"
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
	@echo "completion Windows x86-64 cross-link: PASS (not runtime-qualified)"

.PHONY: check format lint test docs spec completion-test \
	completion-core-read-source-boundary completion-core-read-test \
	completion-core-read-stress completion-core-read-tsan completion-sanitize \
	completion-core-read-sanitize completion-windows-cross
