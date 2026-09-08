ROOT := ../../..
.DEFAULT_GOAL := check
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
OUT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-compute-runtime
CC := /usr/bin/clang
CXX := /usr/bin/clang++
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
SCHEDULER_FLAGS := -O3 -g -DNDEBUG -Wall -Wextra -Werror -Wpedantic -pthread \
	$(BENCH_ARCH) -fno-vectorize -fno-slp-vectorize -fno-lto
SCHEDULER_COMMON := $(OUT)/scheduler-host.o $(OUT)/scheduler-native.o \
	$(OUT)/scheduler-oracle.o $(OUT)/scheduler-work.o $(OUT)/scheduler-floor.o
STATIC_ORACLE_FLAGS := -DWF_FILTER_NATIVE -Dmain=fir_qualified_oracle_main \
	-Dfir_native_direct=fir_static_check_direct -Dfir_native_lanes4=fir_static_check_lanes4 \
	-Dfir_native_lanes8=fir_static_check_lanes8 -Dfir_native_lanes16=fir_static_check_lanes16
WF_ORACLE_FLAGS := -DWF_FILTER_NATIVE -DWF_FILTER_WF_OBJECT -Dmain=fir_qualified_oracle_main \
	-Dfir_native_direct=fir_wf_tile1 -Dfir_native_lanes4=fir_wf_tile17 \
	-Dfir_native_lanes8=fir_wf_tile257 -Dfir_native_lanes16=fir_wf_tile65536

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

.PHONY: check build check-runtime check-abi check-fir fir-ir check-native native-object check-static \
	wf-objects check-wf-objects build-bench check-bench bench-calibrate compiler force \
	records-build check-records records-calibrate scheduler-fetch scheduler-build \
	check-scheduler check-scheduler-memory scheduler-calibrate

force:

build: $(OUT)/probe

$(OUT)/probe: runtime.c runtime.h probe.c $(FLOOR) force
	mkdir -p "$(OUT)"
	$(CC) $(CFLAGS) -DWF_COMPUTE_TEST -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		runtime.c probe.c $(FLOOR) -o "$@"

check: check-runtime check-abi check-fir check-bench check-records check-scheduler

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

# One ordinary optimized WF object and one native object are qualified before
# reuse by the record timing hosts. The typed shim exposes only the buffer's
# declared element pointer/count; release still consumes the WF buffer.
records-build: $(WFC)
	mkdir -p "$(OUT)"
	"$(WFC)" --par --par-ledger --emit-llvm records.wf -o "$(OUT)/records.ll" > "$(OUT)/records.ledger"
	grep -F 'split independent map' "$(OUT)/records.ledger"
	grep -F 'call void @wf__par_publish(' "$(OUT)/records.ll"
	sed -e 's/@main(/@wf_record_command_main(/g' \
		-e 's/@wf__main_body(/@wf_record_command_body(/g' "$(OUT)/records.ll" > "$(OUT)/records-host.ll"
	cat records_host.ll >> "$(OUT)/records-host.ll"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -c "$(OUT)/records-host.ll" -o "$(OUT)/records-wf.o"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -S "$(OUT)/records-host.ll" -o "$(OUT)/records-wf.s"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module runtime.c $(FLOOR) "$(OUT)/records.ll" -o "$(OUT)/records-command"
	$(CC) $(BENCH_CFLAGS) -c records_native.c -o "$(OUT)/records-native.o"
	$(CC) $(BENCH_CFLAGS) -S records_native.c -o "$(OUT)/records-native.s"
	$(CC) $(BENCH_CFLAGS) records.c records_oracle.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native.o" -o "$(OUT)/records-weak"
	$(CC) $(BENCH_CFLAGS) -DWF_COMPUTE_CONTROL -DRECORD_RUNTIME='"recovered"' \
		records.c records_oracle.c runtime.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native.o" -o "$(OUT)/records-recovered"
	$(CC) $(BENCH_CFLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -c records_native.c -o "$(OUT)/records-native-sanitized.o"
	$(CC) $(BENCH_CFLAGS) -DWF_COMPUTE_CONTROL -DRECORD_RUNTIME='"recovered"' -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer records.c records_oracle.c runtime.c $(FLOOR) \
		"$(OUT)/records-wf.o" "$(OUT)/records-native-sanitized.o" -o "$(OUT)/records-sanitized"
	@for mode in weak recovered sanitized; do nm "$(OUT)/records-$$mode" > "$(OUT)/records-$$mode.symbols"; done
	$(call require-strong,$(OUT)/records-weak,wf__floor_run)
	$(call require-strong,$(OUT)/records-recovered,wf__floor_run)
	$(call require-strong,$(OUT)/records-sanitized,wf__floor_run)
	$(call require-strong,$(OUT)/records-recovered,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/records-sanitized,wf__par_acquire_lane)
	@printf '%s\n' '$(BENCH_CFLAGS)' > "$(OUT)/records-flags.txt"
	$(CC) --version >> "$(OUT)/records-flags.txt"
	shasum -a 256 "$(WFC)" >> "$(OUT)/records-flags.txt"

check-records: records-build
	WF_WORKERS=0 "$(OUT)/records-command"
	WF_WORKERS=4 "$(OUT)/records-command"
	@for mode in weak recovered sanitized; do \
		workers=4; world=parallel; if test "$$mode" = weak; then workers=0; world=sequential; fi; \
		WF_WORKERS=$$workers "$(OUT)/records-$$mode" check > "$(OUT)/records-$$mode-check.log" || exit $$?; \
		cat "$(OUT)/records-$$mode-check.log"; \
		grep -Fxq 'UTF8 leaf qualification PASS: calls=4595603 all-codepoints all-truncations two-byte-pairs' "$(OUT)/records-$$mode-check.log" || exit 1; \
		grep -Fxq "UTF8 batch qualification PASS: world=$$world calls=342 results=1821070" "$(OUT)/records-$$mode-check.log" || exit 1; \
	done
	OUT="$(OUT)" sh records-bench.sh check

records-calibrate: check-records
	OUT="$(OUT)" sh records-bench.sh calibrate

# Download is explicit, as with Cargo's offline research dependencies. The
# build verifies pins and only installs the private library below OUT.
scheduler-fetch:
	OUT="$(OUT)" sh records-scheduler-deps.sh fetch

scheduler-build:
	mkdir -p "$(OUT)"
	OUT="$(OUT)" CC="$(CC)" CXX="$(CXX)" sh records-scheduler-deps.sh build
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -c records_scheduler.cpp -o "$(OUT)/scheduler-host.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c records_native.c -o "$(OUT)/scheduler-native.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -S records_native.c -o "$(OUT)/scheduler-native.s"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c records_oracle.c -o "$(OUT)/scheduler-oracle.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c records_work.c -o "$(OUT)/scheduler-work.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -S records_work.c -o "$(OUT)/scheduler-work.s"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c $(FLOOR) -o "$(OUT)/scheduler-floor.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c runtime.c -o "$(OUT)/scheduler-runtime.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c records_runtime.c -o "$(OUT)/scheduler-wf-adapter.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c records_static.c -o "$(OUT)/scheduler-static-adapter.o"
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -isystem "$(OUT)/scheduler-deps/include" \
		-c records_tbb.cpp -o "$(OUT)/scheduler-tbb-adapter.o"
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -isystem "$(OUT)/scheduler-deps/include" \
		-c records_parlay.cpp -o "$(OUT)/scheduler-parlay-adapter.o"
	$(CXX) -pthread -fno-lto $(SCHEDULER_COMMON) "$(OUT)/scheduler-wf-adapter.o" \
		"$(OUT)/scheduler-runtime.o" -o "$(OUT)/scheduler-wf"
	$(CXX) -pthread -fno-lto $(SCHEDULER_COMMON) "$(OUT)/scheduler-static-adapter.o" -o "$(OUT)/scheduler-static"
	$(CXX) -pthread -fno-lto $(SCHEDULER_COMMON) "$(OUT)/scheduler-tbb-adapter.o" \
		-L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" -ltbb -o "$(OUT)/scheduler-tbb"
	$(CXX) -pthread -fno-lto $(SCHEDULER_COMMON) "$(OUT)/scheduler-parlay-adapter.o" -o "$(OUT)/scheduler-parlay"
	@printf '%s\n' 'C: $(CC) -std=c11 $(SCHEDULER_FLAGS)' \
		'C++: $(CXX) -std=c++17 $(SCHEDULER_FLAGS)' \
		'link: $(CXX) -pthread -fno-lto; common objects reused unchanged' > "$(OUT)/scheduler-flags.txt"
	$(CC) --version >> "$(OUT)/scheduler-flags.txt"
	$(CXX) --version >> "$(OUT)/scheduler-flags.txt"

check-scheduler-memory: scheduler-build
	@for unit in records_native records_oracle records_work records_runtime records_static runtime; do \
		$(CC) -std=c11 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
			-fno-omit-frame-pointer -c $$unit.c -o "$(OUT)/scheduler-memory-$$unit.o" || exit $$?; \
	done
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -c $(FLOOR) -o "$(OUT)/scheduler-memory-floor.o"
	@for unit in records_scheduler records_tbb records_parlay; do \
		$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
			-fno-omit-frame-pointer -isystem "$(OUT)/scheduler-deps/include" \
			-c $$unit.cpp -o "$(OUT)/scheduler-memory-$$unit.o" || exit $$?; \
	done
	@for mode in wf static tbb parlay; do \
		case $$mode in \
			wf) adapter='records_runtime runtime';; \
			static) adapter=records_static;; \
			tbb) adapter=records_tbb;; \
			parlay) adapter=records_parlay;; \
		esac; \
		set --; \
		for unit in records_scheduler records_native records_oracle records_work floor $$adapter; do \
			set -- "$$@" "$(OUT)/scheduler-memory-$$unit.o"; \
		done; \
		if test $$mode = tbb; then set -- "$$@" -L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" -ltbb; fi; \
		$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
			-o "$(OUT)/scheduler-memory-$$mode" || exit $$?; \
		for width in 1 2 4; do \
			WF_WORKERS=$$width "$(OUT)/scheduler-memory-$$mode" check $$width \
				> "$(OUT)/scheduler-memory-$$mode-w$$width.log" || exit $$?; \
			cat "$(OUT)/scheduler-memory-$$mode-w$$width.log"; \
		done; \
	done

check-scheduler: scheduler-build check-scheduler-memory records-build
	# Qualify the exact scalar timing leaf with the complete independent oracle.
	$(CC) -std=c11 $(SCHEDULER_FLAGS) records.c records_oracle.c $(FLOOR) \
		"$(OUT)/records-wf.o" "$(OUT)/scheduler-native.o" -o "$(OUT)/scheduler-leaf-check"
	WF_WORKERS=0 "$(OUT)/scheduler-leaf-check" check > "$(OUT)/scheduler-leaf-check.log"
	grep -Fx 'UTF8 leaf qualification PASS: calls=4595603 all-codepoints all-truncations two-byte-pairs' "$(OUT)/scheduler-leaf-check.log"
	grep -Fx 'UTF8 batch qualification PASS: world=sequential calls=342 results=1821070' "$(OUT)/scheduler-leaf-check.log"
	@for mode in wf static tbb parlay; do \
		for width in 1 2 4; do \
			WF_WORKERS=$$width "$(OUT)/scheduler-$$mode" check $$width > "$(OUT)/scheduler-$$mode-check-w$$width.log" || exit $$?; \
			cat "$(OUT)/scheduler-$$mode-check-w$$width.log"; \
		done; \
	done
	OUT="$(OUT)" sh records-scheduler-bench.sh check

scheduler-calibrate: check-scheduler
	OUT="$(OUT)" sh records-scheduler-bench.sh calibrate

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
	"$(WFC)" --par --par-ledger --emit-llvm fir.wf fir_direct.wf -o "$(OUT)/fir.ll" > "$(OUT)/fir.ledger"
	grep -F 'pair(filter_tiles, filter_tiles)' "$(OUT)/fir.ledger"
	grep -F 'call void @wf__par_publish(' "$(OUT)/fir.ll"
	grep -E '^define i32 @main\(' "$(OUT)/fir.ll"
	grep -E '^define i32 @wf__main_body\(' "$(OUT)/fir.ll"
	# Preserve the emitted module and command; only its command symbols are
	# renamed in the host copy. Typed shims are appended before optimization.
	sed -e 's/@main(/@wf_research_fir_command_main(/g' \
		-e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$(OUT)/fir.ll" > "$(OUT)/fir-host.ll"
	cat fir_host.ll >> "$(OUT)/fir-host.ll"
	"$(WFC)" --par --par-ledger --emit-llvm fir.wf fir_lanes.wf -o "$(OUT)/fir-lanes.ll" > "$(OUT)/fir-lanes.ledger"
	grep -F 'pair(filter_tiles, filter_tiles)' "$(OUT)/fir-lanes.ledger"
	grep -F 'call void @wf__par_publish(' "$(OUT)/fir-lanes.ll"
	grep -E '^define i32 @main\(' "$(OUT)/fir-lanes.ll"
	grep -E '^define i32 @wf__main_body\(' "$(OUT)/fir-lanes.ll"
	sed -e 's/@main(/@wf_research_fir_command_main(/g' \
		-e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$(OUT)/fir-lanes.ll" > "$(OUT)/fir-lanes-host.ll"
	cat fir_host.ll >> "$(OUT)/fir-lanes-host.ll"

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

# Rename only oracle references. Both ordinary kernels and the static pool
# retain their original symbols and link unchanged into the measured host.
check-static: native-object
	$(CC) $(BENCH_CFLAGS) -c fir_static.c -o "$(OUT)/fir-static.o"
	$(CC) $(BENCH_CFLAGS) $(STATIC_ORACLE_FLAGS) -c fir_check.c -o "$(OUT)/fir-static-oracle.o"
	$(CC) $(BENCH_CFLAGS) fir_static_check.c "$(OUT)/fir-static.o" \
		"$(OUT)/fir-native.o" "$(OUT)/fir-static-oracle.o" -lm -o "$(OUT)/fir-static-release-check"
	$(CC) $(BENCH_CFLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -c fir_native.c -o "$(OUT)/fir-static-native-sanitized.o"
	$(CC) $(BENCH_CFLAGS) $(STATIC_ORACLE_FLAGS) -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer -c fir_check.c -o "$(OUT)/fir-static-oracle-sanitized.o"
	$(CC) $(BENCH_CFLAGS) -DFIR_STATIC_TEST -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer fir_static.c fir_static_check.c \
		"$(OUT)/fir-static-native-sanitized.o" "$(OUT)/fir-static-oracle-sanitized.o" \
		-lm -o "$(OUT)/fir-static-check"
	@for lanes in 1 2 4; do \
		"$(OUT)/fir-static-release-check" $$lanes > "$(OUT)/fir-static-release-$$lanes.log" || exit $$?; \
		cat "$(OUT)/fir-static-release-$$lanes.log"; \
		grep -Fxq 'Native FIR qualification PASS: forms=4 calls=107408 samples=1294336 K=1..64' \
			"$(OUT)/fir-static-release-$$lanes.log" || exit 1; \
		grep -Fxq "FIR static ordinary PASS: requested=$$lanes actual=$$lanes helpers=$$((lanes - 1)) idle=condvar spin=0" \
			"$(OUT)/fir-static-release-$$lanes.log" || exit 1; \
	done
	@for setting in '1 3' '2 3' '4 3' '4 1' '4 0'; do \
		set -- $$setting; \
		"$(OUT)/fir-static-check" $$1 $$2 > "$(OUT)/fir-static-$$1-$$2.log" || exit $$?; \
		cat "$(OUT)/fir-static-$$1-$$2.log"; \
		grep -Fxq 'Native FIR qualification PASS: forms=4 calls=107408 samples=1294336 K=1..64' \
			"$(OUT)/fir-static-$$1-$$2.log" || exit 1; \
	done

wf-objects: fir-ir
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -c "$(OUT)/fir-host.ll" -o "$(OUT)/fir-wf.o"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -S "$(OUT)/fir-host.ll" -o "$(OUT)/fir-wf.s"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -c "$(OUT)/fir-lanes-host.ll" -o "$(OUT)/fir-wf-lanes16.o"
	$(CC) $(BENCH_CFLAGS) -Wno-override-module -S "$(OUT)/fir-lanes-host.ll" -o "$(OUT)/fir-wf-lanes16.s"
	$(CC) $(BENCH_CFLAGS) -fno-vectorize -Wno-override-module -c "$(OUT)/fir-lanes-host.ll" -o "$(OUT)/fir-wf-lanes16-slp.o"
	$(CC) $(BENCH_CFLAGS) -fno-vectorize -Wno-override-module -S "$(OUT)/fir-lanes-host.ll" -o "$(OUT)/fir-wf-lanes16-slp.s"
	@printf '%s\n' 'wf: fir.wf fir_direct.wf; $(BENCH_CFLAGS)' \
		'wf-lanes16: fir.wf fir_lanes.wf; $(BENCH_CFLAGS)' \
		'wf-lanes16-slp: fir.wf fir_lanes.wf; $(BENCH_CFLAGS) -fno-vectorize' > "$(OUT)/fir-wf-flags.txt"

# C checker/runtime instrumentation does not imply that already emitted WF
# LLVM receives frontend sanitizer instrumentation. Reuse the exact timed WF
# object, and retain both the ordinary and instrumented host qualifications.
check-wf-objects: wf-objects
	$(CC) $(BENCH_CFLAGS) $(WF_ORACLE_FLAGS) -c fir_check.c -o "$(OUT)/fir-wf-oracle.o"
	$(CC) $(BENCH_CFLAGS) $(WF_ORACLE_FLAGS) -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer -c fir_check.c -o "$(OUT)/fir-wf-oracle-sanitized.o"
	@for variant in direct lanes16 lanes16-slp; do \
		suffix=-$$variant; if test "$$variant" = direct; then suffix=; fi; \
		$(CC) $(BENCH_CFLAGS) fir_wf_check.c $(FLOOR) "$(OUT)/fir-wf$$suffix.o" \
			"$(OUT)/fir-wf-oracle.o" -lm -o "$(OUT)/fir-wf-$$variant-check" || exit $$?; \
		$(CC) $(BENCH_CFLAGS) -DWF_COMPUTE_CONTROL -fsanitize=$(SANITIZER) \
			-fno-sanitize-recover=all -fno-omit-frame-pointer fir_wf_check.c runtime.c $(FLOOR) \
			"$(OUT)/fir-wf$$suffix.o" "$(OUT)/fir-wf-oracle-sanitized.o" \
			-lm -o "$(OUT)/fir-wf-$$variant-sanitized-check" || exit $$?; \
		WF_WORKERS=0 "$(OUT)/fir-wf-$$variant-check" > "$(OUT)/fir-wf-$$variant-check.log" || exit $$?; \
		WF_WORKERS=4 "$(OUT)/fir-wf-$$variant-sanitized-check" > "$(OUT)/fir-wf-$$variant-sanitized-check.log" || exit $$?; \
		cat "$(OUT)/fir-wf-$$variant-check.log" "$(OUT)/fir-wf-$$variant-sanitized-check.log"; \
		for mode in check sanitized-check; do \
			world=sequential; if test "$$mode" = sanitized-check; then world=parallel; fi; \
			grep -Fxq "FIR WF object qualification PASS: world=$$world tiles=4 calls=107408 samples=1294336 history=3752300 misses=322224 K=1..64" \
				"$(OUT)/fir-wf-$$variant-$$mode.log" || exit 1; \
		done; \
	done

build-bench: check-wf-objects check-native check-static
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"weak"' fir_bench.c $(FLOOR) \
		"$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-weak"
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"static"' -DWF_FILTER_STATIC fir_bench.c $(FLOOR) \
		"$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" "$(OUT)/fir-static.o" -lm -o "$(OUT)/bench-static"
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"recovered"' -DWF_COMPUTE_CONTROL \
		fir_bench.c runtime.c $(FLOOR) "$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-recovered"
	$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"shared"' -DWF_SHARED_CONTROL \
		fir_bench.c $(FLOOR) $(SHARED)/core.c $(SHARED)/prim_host.c $(SHARED)/entry.c \
		"$(OUT)/fir-wf.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-shared"
	@for variant in lanes16 lanes16-slp; do \
		$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"weak"' -DFIR_WF_KERNEL=\"wf-$$variant\" \
			fir_bench.c $(FLOOR) "$(OUT)/fir-wf-$$variant.o" "$(OUT)/fir-native.o" \
			-lm -o "$(OUT)/bench-weak-wf-$$variant" || exit $$?; \
		$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"recovered"' -DWF_COMPUTE_CONTROL -DFIR_WF_KERNEL=\"wf-$$variant\" \
			fir_bench.c runtime.c $(FLOOR) "$(OUT)/fir-wf-$$variant.o" "$(OUT)/fir-native.o" \
			-lm -o "$(OUT)/bench-recovered-wf-$$variant" || exit $$?; \
		$(CC) $(BENCH_CFLAGS) -DFIR_RUNTIME='"shared"' -DWF_SHARED_CONTROL -DFIR_WF_KERNEL=\"wf-$$variant\" \
			fir_bench.c $(FLOOR) $(SHARED)/core.c $(SHARED)/prim_host.c $(SHARED)/entry.c \
			"$(OUT)/fir-wf-$$variant.o" "$(OUT)/fir-native.o" -lm -o "$(OUT)/bench-shared-wf-$$variant" || exit $$?; \
	done
	@for form in weak static recovered shared weak-wf-lanes16 recovered-wf-lanes16 shared-wf-lanes16 \
		weak-wf-lanes16-slp recovered-wf-lanes16-slp shared-wf-lanes16-slp; do \
		nm "$(OUT)/bench-$$form" > "$(OUT)/bench-$$form.symbols"; \
	done
	$(call require-strong,$(OUT)/bench-weak,wf__floor_run)
	$(call require-strong,$(OUT)/bench-static,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered,wf__floor_run)
	$(call require-strong,$(OUT)/bench-shared,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/bench-shared,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/bench-weak-wf-lanes16,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered-wf-lanes16,wf__floor_run)
	$(call require-strong,$(OUT)/bench-shared-wf-lanes16,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered-wf-lanes16,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/bench-shared-wf-lanes16,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/bench-weak-wf-lanes16-slp,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered-wf-lanes16-slp,wf__floor_run)
	$(call require-strong,$(OUT)/bench-shared-wf-lanes16-slp,wf__floor_run)
	$(call require-strong,$(OUT)/bench-recovered-wf-lanes16-slp,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/bench-shared-wf-lanes16-slp,wf__par_acquire_lane)
	grep -E ' T _?wf_sched_join$$' "$(OUT)/bench-shared.symbols"
	grep -E ' T _?wf_sched_join$$' "$(OUT)/bench-shared-wf-lanes16.symbols"
	grep -E ' T _?wf_sched_join$$' "$(OUT)/bench-shared-wf-lanes16-slp.symbols"
	@for suffix in '' -wf-lanes16 -wf-lanes16-slp; do \
		if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/bench-recovered$$suffix.symbols"; then \
			echo 'recovered timing binary unexpectedly links shared runtime' >&2; exit 1; \
		fi; \
	done
	$(CC) --version > "$(OUT)/bench-toolchain.txt"
	@printf '%s\n' '$(BENCH_CFLAGS)' >> "$(OUT)/bench-toolchain.txt"
	shasum -a 256 "$(WFC)" >> "$(OUT)/bench-toolchain.txt"

check-bench: build-bench
	OUT="$(OUT)" sh fir-bench.sh check

bench-calibrate: build-bench
	OUT="$(OUT)" sh fir-bench.sh calibrate
