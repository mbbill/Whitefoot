ROOT := ../../..
.DEFAULT_GOAL := check
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
OUT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-compute-runtime
CC := /usr/bin/clang
SANITIZER ?= address,undefined
ASAN_OPTIONS ?= detect_stack_use_after_return=1
export ASAN_OPTIONS
CFLAGS := -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread
FLOOR := $(ROOT)/compiler/src/backend/wf_floor.c
WFC ?= $(OUT)/compiler/gate/whitefootc
SHARED := $(ROOT)/compiler/src/backend/sched
BENCH_ARCH ?=
BENCH_CFLAGS := -std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread \
	-fno-fast-math -ffp-contract=off $(BENCH_ARCH)

# Mach-O plain nm prints T for weak function definitions too. Check its full
# binding description; ELF nm distinguishes a strong T from weak W directly.
define require-strong
	@if test "$$(uname -s)" = Darwin; then \
		nm -m "$(1)" > "$(1).bindings"; \
		awk -v name='_$(2)' '$$NF==name && /\(__TEXT,__text\) external/ && !/weak/ {ok=1} END {exit !ok}' \
			"$(1).bindings"; \
	else \
		grep -E ' T $(2)$$' "$(1).symbols"; \
	fi
endef

.PHONY: check build check-runtime check-abi check-fir fir-ir check-native native-object \
	build-bench check-bench bench-calibrate compiler force

force:

build: $(OUT)/probe

$(OUT)/probe: runtime.c runtime.h probe.c $(FLOOR) force
	mkdir -p "$(OUT)"
	$(CC) $(CFLAGS) -DWF_COMPUTE_TEST -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		runtime.c probe.c $(FLOOR) -o "$@"

check: check-runtime check-abi check-fir check-bench

check-runtime: build
	WF_WORKERS=2 "$(OUT)/probe" protocol
	WF_WORKERS=4 "$(OUT)/probe" protocol
	WF_WORKERS=2 "$(OUT)/probe" delayed-read
	WF_WORKERS=2 "$(OUT)/probe" retired-frame
	WF_WORKERS=4 "$(OUT)/probe" zero-start
	WF_WORKERS=4 "$(OUT)/probe" partial-start
	nm "$(OUT)/probe" > "$(OUT)/probe.symbols"
	@if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/probe.symbols"; then \
		echo 'compute control unexpectedly links shared scheduler or completion code' >&2; exit 1; \
	fi

compiler:
	cargo build --manifest-path $(ROOT)/compiler/Cargo.toml --profile gate --bin whitefootc \
		--locked --offline --target-dir "$(OUT)/compiler"

# Refresh the default compiler after source changes. A supplied compiler is
# an explicit caller-owned dependency and must already be built for this tree.
ifeq ($(WFC),$(OUT)/compiler/gate/whitefootc)
$(WFC): compiler
else
$(WFC):
	@test -x "$(WFC)"
endif

check-abi: $(WFC)
	mkdir -p "$(OUT)"
	"$(WFC)" --par --par-ledger --emit-llvm abi.wf -o "$(OUT)/abi.ll" > "$(OUT)/abi.ledger"
	grep -F 'declare' "$(OUT)/abi.ll" > "$(OUT)/abi.declarations"
	@if grep -E '@wf__completion_|@wf_sched_' "$(OUT)/abi.ll"; then \
		echo 'compute ABI fixture unexpectedly requires I/O or scheduler symbols' >&2; exit 1; \
	fi
	$(CC) $(CFLAGS) -Wno-override-module -DWF_COMPUTE_ABI_OBSERVER -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		runtime.c probe.c $(FLOOR) "$(OUT)/abi.ll" -o "$(OUT)/abi"
	$(CC) $(CFLAGS) -Wno-override-module -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		$(FLOOR) "$(OUT)/abi.ll" -o "$(OUT)/abi-sequential"
	WF_WORKERS=0 "$(OUT)/abi"
	WF_WORKERS=1 "$(OUT)/abi"
	WF_WORKERS=2 "$(OUT)/abi"
	WF_WORKERS=4 "$(OUT)/abi"
	"$(OUT)/abi-sequential"
	nm "$(OUT)/abi" > "$(OUT)/abi.symbols"
	$(call require-strong,$(OUT)/abi,wf__par_acquire_lane)
	@if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/abi.symbols"; then \
		echo 'compute ABI binary unexpectedly links shared scheduler or completion code' >&2; exit 1; \
	fi

fir-ir: $(WFC)
	mkdir -p "$(OUT)"
	"$(WFC)" --par --par-ledger --emit-llvm fir.wf -o "$(OUT)/fir.ll" > "$(OUT)/fir.ledger"
	grep -F 'pair(filter_tiles, filter_tiles)' "$(OUT)/fir.ledger"
	grep -F 'call void @wf__par_publish(' "$(OUT)/fir.ll"
	grep -E '^define i32 @main\(' "$(OUT)/fir.ll"
	grep -E '^define i32 @wf__main_body\(' "$(OUT)/fir.ll"
	# Preserve the emitted module and command; only its command symbols are
	# renamed in the host copy. Typed shims are appended before optimization.
	sed -e 's/@main(/@wf_research_fir_command_main(/g' \
		-e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$(OUT)/fir.ll" > "$(OUT)/fir-host.ll"
	cat fir_host.ll >> "$(OUT)/fir-host.ll"

check-fir: fir-ir
	$(CC) $(CFLAGS) -fno-fast-math -ffp-contract=off -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		fir_check.c -lm -o "$(OUT)/fir-oracle"
	$(CC) $(CFLAGS) -Wno-override-module -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		runtime.c $(FLOOR) "$(OUT)/fir.ll" -o "$(OUT)/fir-command"
	$(CC) $(CFLAGS) -Wno-override-module -fno-fast-math -ffp-contract=off \
		-DWF_FILTER_HOST -DWF_COMPUTE_CONTROL -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		runtime.c fir_check.c $(FLOOR) "$(OUT)/fir-host.ll" -lm -o "$(OUT)/fir"
	$(CC) $(CFLAGS) -Wno-override-module -fno-fast-math -ffp-contract=off \
		-DWF_FILTER_HOST -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		fir_check.c $(FLOOR) "$(OUT)/fir-host.ll" -lm -o "$(OUT)/fir-sequential"
	"$(OUT)/fir-oracle"
	WF_WORKERS=0 "$(OUT)/fir-command"
	WF_WORKERS=2 "$(OUT)/fir-command"
	@for workers in 0 1 2 4; do \
		case $$workers in 0|1) selected=sequential-clone;; *) selected=parallel;; esac; \
		WF_WORKERS=$$workers "$(OUT)/fir" > "$(OUT)/fir-workers-$$workers.log" || exit $$?; \
		cat "$(OUT)/fir-workers-$$workers.log"; \
		grep -Fxq "FIR WF qualification PASS: world=$$selected calls=6408 samples=485431 history=290955 misses=19224" \
			"$(OUT)/fir-workers-$$workers.log" || exit 1; \
	done
	"$(OUT)/fir-sequential" > "$(OUT)/fir-sequential.log"
	cat "$(OUT)/fir-sequential.log"
	grep -Fxq 'FIR WF qualification PASS: world=sequential-clone calls=6408 samples=485431 history=290955 misses=19224' \
		"$(OUT)/fir-sequential.log"
	nm "$(OUT)/fir" > "$(OUT)/fir.symbols"
	$(call require-strong,$(OUT)/fir,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/fir,wf__floor_run)
	@if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/fir.symbols"; then \
		echo 'FIR unexpectedly links shared scheduler or completion code' >&2; exit 1; \
	fi

# The timing object is also linked to the complete native checker. Qualify
# both a sanitizer build and these exact ordinary optimized kernel bytes.
native-object:
	mkdir -p "$(OUT)"
	$(CC) $(BENCH_CFLAGS) -c fir_native.c -o "$(OUT)/fir-native.o"
	$(CC) $(BENCH_CFLAGS) -S fir_native.c -o "$(OUT)/fir-native.s"
	$(CC) $(BENCH_CFLAGS) -S -emit-llvm fir_native.c -o "$(OUT)/fir-native.ll"

check-native: native-object
	$(CC) $(BENCH_CFLAGS) -DWF_FILTER_NATIVE -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer fir_native.c fir_check.c -lm -o "$(OUT)/fir-native-check"
	$(CC) $(CFLAGS) -fno-fast-math -ffp-contract=off -DWF_FILTER_NATIVE \
		fir_check.c "$(OUT)/fir-native.o" -lm -o "$(OUT)/fir-native-release-check"
	@for form in check release-check; do \
		"$(OUT)/fir-native-$$form" > "$(OUT)/fir-native-$$form.log" || exit $$?; \
		cat "$(OUT)/fir-native-$$form.log"; \
		grep -Fxq 'Native FIR qualification PASS: forms=4 calls=107408 samples=1294336 K=1..64' \
			"$(OUT)/fir-native-$$form.log" || exit 1; \
	done

build-bench: fir-ir check-native
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -c "$(OUT)/fir-host.ll" -o "$(OUT)/fir-wf.o"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -S "$(OUT)/fir-host.ll" -o "$(OUT)/fir-wf.s"
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"weak"' fir_bench.c $(FLOOR) \
		"$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-weak"
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"recovered"' -DWF_COMPUTE_CONTROL \
		fir_bench.c runtime.c $(FLOOR) "$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-recovered"
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"shared"' -DWF_SHARED_CONTROL \
		fir_bench.c $(FLOOR) $(SHARED)/core.c $(SHARED)/prim_host.c $(SHARED)/entry.c \
		"$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-shared"
	@for form in weak recovered shared; do \
		nm "$(OUT)/bench-$$form" > "$(OUT)/bench-$$form.symbols"; \
	done
	$(call require-strong,$(OUT)/bench-weak,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered,wf__floor_run)
	$(call require-strong,$(OUT)/bench-shared,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/bench-shared,wf__par_acquire_lane)
	grep -E ' T _?wf_sched_join$$' "$(OUT)/bench-shared.symbols"
	@if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/bench-recovered.symbols"; then \
		echo 'recovered timing binary unexpectedly links shared runtime' >&2; exit 1; \
	fi
	$(CC) --version > "$(OUT)/bench-toolchain.txt"
	@printf '%s\n' '$(BENCH_CFLAGS)' >> "$(OUT)/bench-toolchain.txt"
	shasum -a 256 "$(WFC)" >> "$(OUT)/bench-toolchain.txt"

check-bench: build-bench
	OUT="$(OUT)" sh fir-bench.sh check

bench-calibrate: build-bench
	OUT="$(OUT)" sh fir-bench.sh calibrate
