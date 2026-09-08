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
RECORD_FLAGS := $(BENCH_CFLAGS) -fno-vectorize -fno-slp-vectorize -fno-lto
SCHEDULER_FLAGS := -O3 -g -DNDEBUG -Wall -Wextra -Werror -Wpedantic -pthread \
	$(BENCH_ARCH) -fno-vectorize -fno-slp-vectorize -fno-lto
SCHEDULER_COMMON := $(OUT)/scheduler-host.o $(OUT)/scheduler-native.o \
	$(OUT)/scheduler-oracle.o $(OUT)/scheduler-work.o $(OUT)/scheduler-floor.o
SCHEDULERS := wf static tbb parlay rayon-join rayon-iter
LAYOUT_SCHEDULERS := $(SCHEDULERS) wf-group4 wf-group16
RAYON_LIB := $(OUT)/scheduler-deps/rayon/release/libwf_records_rayon.a
RAYON_EVENTS_LIB := $(OUT)/scheduler-events-deps/rayon/release/libwf_records_rayon.a
RAYON_SYSTEM_LIBS := $(if $(filter Linux,$(shell uname -s)),-ldl -lm,)
SCHEDULER_MEMORY_ENV := ASAN_OPTIONS=detect_stack_use_after_return=1:halt_on_error=1$(if $(filter Linux,$(shell uname -s)),:detect_leaks=1,) LSAN_OPTIONS=exitcode=23
SCHEDULER_MEMORY_MODE := $(if $(findstring address,$(SANITIZER)),validate,validate-clean)
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
	check-scheduler check-scheduler-memory scheduler-calibrate scheduler-layout-build \
	check-scheduler-layout scheduler-layout-calibrate scheduler-trace-build check-scheduler-trace \
	scheduler-events-build check-scheduler-events

force:

build: $(OUT)/probe $(OUT)/probe-no-stats $(OUT)/probe-events

$(OUT)/probe $(OUT)/probe-no-stats $(OUT)/probe-events: runtime.c runtime.h runtime_events.h probe.c $(FLOOR) force
	mkdir -p "$(OUT)"
	$(CC) $(CFLAGS) -DWF_COMPUTE_TEST -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		-DWF_COMPUTE_STATS=$(if $(filter $(OUT)/probe,$@),1,0) \
		$(if $(filter $(OUT)/probe-events,$@),-DWF_COMPUTE_EVENTS,) \
		runtime.c probe.c $(FLOOR) -o "$@"

check: check-runtime check-protocol-cost check-abi check-fir check-bench check-records-events check-scheduler-events check-mandelbrot check-quadrature

.PHONY: quadrature-build check-quadrature quadrature-calibrate quadrature-deps
QUADRATURE_FORMS := native wf-seq wf-auto wf-leaf-seq wf-leaf cpp-seq \
	tbb-d0 tbb-d2 tbb-d4 tbb-d8 tbb-d24 parlay-d0 parlay-d2 parlay-d4 parlay-d8 parlay-d24
quadrature-deps:
	OUT="$(OUT)" CC="$(CC)" CXX="$(CXX)" sh records-scheduler-deps.sh build

quadrature-build: $(WFC) quadrature-deps
	mkdir -p "$(OUT)/quadrature/source"
	cp quadrature.wf quadrature_host.ll quadrature.c quadrature.awk quadrature_native.cpp quadrature_native.h records-scheduler-deps.sh runtime.c runtime.h runtime_events.h $(FLOOR) Makefile "$(OUT)/quadrature/source/"
	mkdir -p "$(OUT)/quadrature/deps"
	cp -R "$(OUT)/scheduler-deps/lib" "$(OUT)/quadrature/deps/"
	cp "$(OUT)/scheduler-deps/metadata.txt" "$(OUT)/scheduler-deps/build/compile_commands.json" "$(OUT)/quadrature/deps/"
	cd "$(OUT)/scheduler-deps" && find include -type f -exec shasum -a 256 {} + > "$(OUT)/quadrature/deps/headers.sha256"
	"$(WFC)" --par --par-ledger --emit-llvm quadrature.wf -o "$(OUT)/quadrature/module.ll" > "$(OUT)/quadrature/ledger.txt"
	grep -F 'pair(adaptive, adaptive)' "$(OUT)/quadrature/ledger.txt"
	grep -F 'call void @wf__par_publish(' "$(OUT)/quadrature/module.ll"
	sed -e 's/@main(/@wf_quadrature_command_main(/g' -e 's/@wf__main_body(/@wf_quadrature_command_body(/g' \
		"$(OUT)/quadrature/module.ll" > "$(OUT)/quadrature/host.ll"
	cat quadrature_host.ll >> "$(OUT)/quadrature/host.ll"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -c "$(OUT)/quadrature/host.ll" -o "$(OUT)/quadrature/wf.o"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -S "$(OUT)/quadrature/host.ll" -o "$(OUT)/quadrature/wf.s"
	"$(WFC)" --par --par-scalar-leaf-limit 16 --par-ledger --emit-llvm quadrature.wf -o "$(OUT)/quadrature/leaf-module.ll" > "$(OUT)/quadrature/leaf-ledger.txt"
	grep -F 'scalar leaf limit 16: omitted 2 compute offers' "$(OUT)/quadrature/leaf-ledger.txt"
	@test "$$(grep -c 'call void @wf__par_publish(' "$(OUT)/quadrature/leaf-module.ll")" = 1
	sed -e 's/@main(/@wf_quadrature_leaf_command_main(/g' -e 's/@wf__main_body(/@wf_quadrature_leaf_command_body(/g' \
		"$(OUT)/quadrature/leaf-module.ll" > "$(OUT)/quadrature/leaf-host.ll"
	sed 's/@wf_research_quadrature(/@wf_research_quadrature_leaf(/g' quadrature_host.ll >> "$(OUT)/quadrature/leaf-host.ll"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -c "$(OUT)/quadrature/leaf-host.ll" -o "$(OUT)/quadrature/wf-leaf.o"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -S "$(OUT)/quadrature/leaf-host.ll" -o "$(OUT)/quadrature/wf-leaf.s"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=0 -S quadrature.c -o "$(OUT)/quadrature/native.s"
	@set -e; for image in ordinary sanitized; do \
		stats=0; flags=''; if test $$image = sanitized; then stats=1; flags='-DWF_COMPUTE_EVENTS -fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer'; fi; \
		for unit in quadrature runtime; do \
			$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=$$stats $$flags -c $$unit.c -o "$(OUT)/quadrature/$$image-$$unit.o"; \
		done; \
		$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=$$stats $$flags -c $(FLOOR) -o "$(OUT)/quadrature/$$image-floor.o"; \
		$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fno-fast-math -ffp-contract=off -DWF_COMPUTE_STATS=$$stats $$flags \
			-isystem "$(OUT)/scheduler-deps/include" -c quadrature_native.cpp -o "$(OUT)/quadrature/$$image-native.o"; \
		$(CXX) -pthread -fno-lto $$flags "$(OUT)/quadrature/$$image-quadrature.o" "$(OUT)/quadrature/$$image-runtime.o" \
			"$(OUT)/quadrature/$$image-floor.o" "$(OUT)/quadrature/$$image-native.o" \
			"$(OUT)/quadrature/wf.o" "$(OUT)/quadrature/wf-leaf.o" \
			-L"$(OUT)/quadrature/deps/lib" -Wl,-rpath,"$(OUT)/quadrature/deps/lib" -ltbb -lm -o "$(OUT)/quadrature/$$image"; \
	done
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fno-fast-math -ffp-contract=off -DWF_COMPUTE_STATS=0 \
		-isystem "$(OUT)/scheduler-deps/include" -S quadrature_native.cpp -o "$(OUT)/quadrature/native-cpp.s"
	@nm "$(OUT)/quadrature/ordinary" > "$(OUT)/quadrature/ordinary.symbols"
	$(call require-strong,$(OUT)/quadrature/ordinary,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/quadrature/ordinary,wf__floor_run)
	@printf '%s\n' '$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=0' \
		'$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fno-fast-math -ffp-contract=off -DWF_COMPUTE_STATS=0' \
		'WF original: --par; scalar-leaf control: --par --par-scalar-leaf-limit 16' \
		'sanitized: -DWF_COMPUTE_STATS=1 -DWF_COMPUTE_EVENTS -fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer; host/runtime/floor/native C++ and Parlay headers; generated WF and oneTBB library ordinary' > "$(OUT)/quadrature/flags.txt"
	$(CC) --version >> "$(OUT)/quadrature/flags.txt"
	cp "$(WFC)" "$(OUT)/quadrature/whitefootc"
	git rev-parse HEAD > "$(OUT)/quadrature/revision.txt"
	cd "$(OUT)/quadrature" && shasum -a 256 source/* module.ll host.ll *.o *.s ordinary sanitized whitefootc flags.txt ledger.txt revision.txt leaf-module.ll leaf-host.ll leaf-ledger.txt > manifest.sha256
	cd "$(OUT)/quadrature" && find deps -type f -exec shasum -a 256 {} + >> manifest.sha256

check-quadrature: quadrature-build
	@set -e; for image in ordinary sanitized; do for width in 1 4; do for variant in $(QUADRATURE_FORMS); do \
		form=$${variant%-d*}; spawn=0; set -- $$form; case $$variant in *-d*) spawn=$${variant##*-d}; set -- $$form $$spawn;; esac; \
		log="$(OUT)/quadrature/check-$$image-w$$width-$$variant.tsv"; \
		WF_WORKERS=$$width "$(OUT)/quadrature/$$image" check "$$@" > "$$log" 2>&1; \
		stats=0; if test $$image = sanitized; then stats=1; fi; \
		awk -v mode=check -v form=$$form -v spawn=$$spawn -v width=$$width -v stats=$$stats -f quadrature.awk "$$log"; \
		grep -E '^# quadrature PASS: outputs=20 stats=[01] steals=[0-9]+ migrated=[0-9]+$$' "$$log"; \
	done; done; done
	@set -e; good="$(OUT)/quadrature/check-ordinary-w1-native.tsv"; \
	for mutation in missing-row wrong-form missing-footer; do \
		bad="$(OUT)/quadrature/negative-$$mutation.tsv"; \
		case $$mutation in \
			missing-row) sed '5d' "$$good" > "$$bad" ;; \
			wrong-form) sed '1s/form=native/form=wf-seq/' "$$good" > "$$bad" ;; \
			missing-footer) sed '$$d' "$$good" > "$$bad" ;; \
		esac; \
		if awk -v mode=check -v form=native -v width=1 -v stats=0 -f quadrature.awk "$$bad" > "$$bad.log" 2>&1; then \
			echo "quadrature accepted $$mutation" >&2; exit 1; \
		fi; \
	done
	@sed '3s/forks=3/forks=0/' "$(OUT)/quadrature/check-ordinary-w4-tbb-d2.tsv" > "$(OUT)/quadrature/negative-forks.tsv"
	@if awk -v mode=check -v form=tbb -v spawn=2 -v width=4 -v stats=0 -f quadrature.awk \
		"$(OUT)/quadrature/negative-forks.tsv" > "$(OUT)/quadrature/negative-forks.tsv.log" 2>&1; then \
		echo 'quadrature accepted wrong fork count' >&2; exit 1; \
	fi

quadrature-calibrate:
	@test -n "$(RESULTS)" && test -x "$(OUT)/quadrature/ordinary"
	mkdir "$(RESULTS)"
	@set -e; uname -a > "$(RESULTS)/host.txt"; \
	if test "$$(uname -s)" = Linux; then lscpu >> "$(RESULTS)/host.txt"; taskset -pc $$$$ >> "$(RESULTS)/host.txt"; \
	else sysctl hw.model hw.ncpu hw.physicalcpu >> "$(RESULTS)/host.txt"; fi; \
	for pass in 1 2 3 4 5; do \
		forms='$(QUADRATURE_FORMS)'; if test $$((pass % 2)) = 0; then forms=$$(printf '%s\n' $$forms | awk '{a[NR]=$$0} END {for(i=NR;i>0;--i)print a[i]}'); fi; \
		for width in 1 4; do for variant in $$forms; do \
			form=$${variant%-d*}; spawn=0; set -- $$form; case $$variant in *-d*) spawn=$${variant##*-d}; set -- $$form $$spawn;; esac; \
			WF_WORKERS=$$width "$(OUT)/quadrature/ordinary" bench "$$@" > "$(RESULTS)/p$$pass-w$$width-$$variant.tsv" 2>&1; \
			awk -v mode=bench -v form=$$form -v spawn=$$spawn -v width=$$width -v stats=0 -f quadrature.awk "$(RESULTS)/p$$pass-w$$width-$$variant.tsv"; \
			grep -E '^# quadrature PASS: outputs=90 stats=0 steals=0 migrated=0$$' "$(RESULTS)/p$$pass-w$$width-$$variant.tsv"; \
		done; done; \
	done

.PHONY: protocol-cost-build check-protocol-cost protocol-cost-calibrate
protocol-cost-build: runtime.c runtime.h runtime_events.h protocol_cost.c protocol_cost.awk $(FLOOR)
	mkdir -p "$(OUT)/protocol-cost"
	mkdir -p "$(OUT)/protocol-cost/source"
	cp protocol_cost.c protocol_cost.awk runtime.c runtime.h runtime_events.h $(FLOOR) Makefile "$(OUT)/protocol-cost/source/"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=1 protocol_cost.c runtime.c $(FLOOR) -o "$(OUT)/protocol-cost/ordinary"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=1 -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer \
		protocol_cost.c runtime.c $(FLOOR) -o "$(OUT)/protocol-cost/sanitized"
	@printf '%s\n' '$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=1' \
		'sanitized additionally: -fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer' \
		> "$(OUT)/protocol-cost/flags.txt"
	$(CC) --version >> "$(OUT)/protocol-cost/flags.txt"
	git rev-parse HEAD > "$(OUT)/protocol-cost/revision.txt"
	git diff HEAD -- protocol_cost.c runtime.c runtime.h runtime_events.h Makefile $(FLOOR) > "$(OUT)/protocol-cost/source.patch"
	shasum -a 256 protocol_cost.c protocol_cost.awk runtime.c runtime.h runtime_events.h $(FLOOR) Makefile \
		"$(OUT)/protocol-cost/ordinary" "$(OUT)/protocol-cost/sanitized" > "$(OUT)/protocol-cost/manifest.sha256"

check-protocol-cost: protocol-cost-build
	@set -e; for image in ordinary sanitized; do for width in 2 4; do for depth in 1 8 32; do \
		log="$(OUT)/protocol-cost/check-$$image-w$$width-d$$depth.tsv"; \
		WF_WORKERS=$$width "$(OUT)/protocol-cost/$$image" $$width $$depth 0 > "$$log" 2>&1; \
		awk -v width=$$width -v depth=$$depth -v pass=0 -f protocol_cost.awk "$$log"; \
		grep -Fx '# protocol PASS: samples=18 outputs=4718592' "$$log"; \
	done; done; done

# Run after qualification; repeated invocations use a fresh RESULTS directory.
# Raw paired samples retain all observations; no timing threshold selects PASS.
protocol-cost-calibrate:
	@test -n "$(RESULTS)" && test -x "$(OUT)/protocol-cost/ordinary"
	mkdir "$(RESULTS)"
	@set -e; uname -a > "$(RESULTS)/host.txt"; \
	if test "$$(uname -s)" = Linux; then \
		lscpu >> "$(RESULTS)/host.txt"; cat /proc/self/status >> "$(RESULTS)/host.txt"; \
		if test -r /sys/fs/cgroup/cpu.max; then cat /sys/fs/cgroup/cpu.max >> "$(RESULTS)/host.txt"; fi; \
	else sysctl hw.model hw.physicalcpu hw.logicalcpu >> "$(RESULTS)/host.txt"; fi; \
	for pass in 0 1 2 3 4; do \
		widths='2 4'; depths='1 8 32'; \
		if test $$((pass % 2)) = 1; then widths='4 2'; depths='32 8 1'; fi; \
		for width in $$widths; do for depth in $$depths; do \
			log="$(RESULTS)/w$$width-d$$depth-p$$pass.tsv"; \
			WF_WORKERS=$$width "$(OUT)/protocol-cost/ordinary" $$width $$depth $$pass > "$$log" 2>&1; \
			awk -v width=$$width -v depth=$$depth -v pass=$$pass -f protocol_cost.awk "$$log"; \
			grep -Fx '# protocol PASS: samples=18 outputs=4718592' "$$log"; \
		done; done; \
	done

check-runtime: build
	@set -e; for probe in probe probe-no-stats probe-events; do \
		WF_WORKERS=2 "$(OUT)/$$probe" protocol; \
		WF_WORKERS=4 "$(OUT)/$$probe" protocol; \
		WF_WORKERS=2 "$(OUT)/$$probe" delayed-read; \
		WF_WORKERS=2 "$(OUT)/$$probe" retired-frame; \
		WF_WORKERS=4 "$(OUT)/$$probe" zero-start; \
		WF_WORKERS=4 "$(OUT)/$$probe" partial-start; \
		nm "$(OUT)/$$probe" > "$(OUT)/$$probe.symbols"; \
		if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/$$probe.symbols"; then \
			echo 'compute control unexpectedly links shared scheduler or completion code' >&2; exit 1; \
		fi; \
	done
	@if grep -E 'wf_compute_steal_count|wf__par_grants|wf_compute_event|wf_events' "$(OUT)/probe-no-stats.symbols"; then \
		echo 'statistics-disabled runtime retains a counter or observer' >&2; exit 1; \
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
	$(CC) $(RECORD_FLAGS) -Wno-override-module -c "$(OUT)/records-host.ll" -o "$(OUT)/records-wf.o"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -S "$(OUT)/records-host.ll" -o "$(OUT)/records-wf.s"
	$(CC) $(RECORD_FLAGS) -Wno-override-module runtime.c $(FLOOR) "$(OUT)/records.ll" -o "$(OUT)/records-command"
	$(CC) $(RECORD_FLAGS) -c records_native.c -o "$(OUT)/records-native.o"
	$(CC) $(RECORD_FLAGS) -S records_native.c -o "$(OUT)/records-native.s"
	$(CC) $(RECORD_FLAGS) records.c records_oracle.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native.o" -o "$(OUT)/records-weak"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_CONTROL -DRECORD_RUNTIME='"recovered"' \
		records.c records_oracle.c runtime.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native.o" -o "$(OUT)/records-recovered"
	$(CC) $(RECORD_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -c records_native.c -o "$(OUT)/records-native-sanitized.o"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_CONTROL -DRECORD_RUNTIME='"recovered"' -fsanitize=$(SANITIZER) \
		-fno-sanitize-recover=all -fno-omit-frame-pointer records.c records_oracle.c runtime.c $(FLOOR) \
		"$(OUT)/records-wf.o" "$(OUT)/records-native-sanitized.o" -o "$(OUT)/records-sanitized"
	@for mode in weak recovered sanitized; do nm "$(OUT)/records-$$mode" > "$(OUT)/records-$$mode.symbols"; done
	$(call require-strong,$(OUT)/records-weak,wf__floor_run)
	$(call require-strong,$(OUT)/records-recovered,wf__floor_run)
	$(call require-strong,$(OUT)/records-sanitized,wf__floor_run)
	$(call require-strong,$(OUT)/records-recovered,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/records-sanitized,wf__par_acquire_lane)
	@printf '%s\n' '$(RECORD_FLAGS)' > "$(OUT)/records-flags.txt"
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

.PHONY: records-budget-build check-records-budget
records-budget-build: records-build
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_CONTROL -DWF_COMPUTE_BUDGET_CONTROL -DRECORD_RUNTIME='"budget"' \
		records.c records_oracle.c runtime.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native.o" -o "$(OUT)/records-budget"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_CONTROL -DWF_COMPUTE_BUDGET_CONTROL -DRECORD_RUNTIME='"budget"' \
		-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer \
		records.c records_oracle.c runtime.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native-sanitized.o" \
		-o "$(OUT)/records-budget-sanitized"
	@printf '%s\n' 'budget=cost,capacity,team; cadence=call,batch; same executable and emitted WF object; controls selected before floor entry' \
		'capacity bypasses estimated affordability only; span, lane limit, queue backpressure and fallback remain' \
		'team additionally caps requested terminal chunks at lanes; neither policy guarantees one job per worker' \
		'full outputs checked every call; full inputs checked every call or at batch end; both modes check inputs at batch end' \
		'statistics enabled; emitted WF object is not frontend-sanitizer-instrumented' > "$(OUT)/records-budget-flags.txt"

define qualify-record-controls
	@set -e; for image in $(1); do for policy in cost capacity team; do for cadence in call batch; do \
		log="$(OUT)/records-$$image-$$policy-$$cadence-check.log"; \
		WF_WORKERS=4 WF_BUDGET_CONTROL="$$policy" WF_RECORD_CHECK_CADENCE="$$cadence" "$(OUT)/records-$$image" check > "$$log"; \
		grep -Fxq "# budget_policy=$$policy" "$$log"; \
		grep -Fxq "# input_check_cadence=$$cadence" "$$log"; \
		grep -Fxq 'UTF8 leaf qualification PASS: calls=4595603 all-codepoints all-truncations two-byte-pairs' "$$log"; \
		grep -Fxq 'UTF8 batch qualification PASS: world=parallel calls=342 results=1821070' "$$log"; \
		log="$(OUT)/records-$$image-$$policy-$$cadence-boundary.log"; \
		WF_WORKERS=4 WF_BUDGET_CONTROL="$$policy" WF_RECORD_CHECK_CADENCE="$$cadence" \
			"$(OUT)/records-$$image" wf 33 64 error-first 256 123 0 > "$$log"; \
		grep -Fxq '# UTF8 records PASS: calls=257 results=8481' "$$log"; \
		test "$$(grep -c '^# call_gap ' "$$log")" -eq 256; \
	done; done; done
endef

check-records-budget: check-records records-budget-build
	$(call qualify-record-controls,budget budget-sanitized)
	OUT="$(OUT)" sh records-bench.sh budget-check

.PHONY: records-events-build check-records-events
records-events-build: records-budget-build
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_CONTROL -DWF_COMPUTE_BUDGET_CONTROL -DWF_COMPUTE_EVENTS -DRECORD_RUNTIME='"events"' \
		records.c records_oracle.c runtime.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native.o" -o "$(OUT)/records-events"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_CONTROL -DWF_COMPUTE_BUDGET_CONTROL -DWF_COMPUTE_EVENTS -DRECORD_RUNTIME='"events"' \
		-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer \
		records.c records_oracle.c runtime.c $(FLOOR) "$(OUT)/records-wf.o" "$(OUT)/records-native-sanitized.o" \
		-o "$(OUT)/records-events-sanitized"
	@printf '%s\n' 'events=wf-2; banks=4; counters=24; before/after core snapshots; diagnostic image only' \
		'stable jobs and successful steals settle at join; search, signal and idle events remain asynchronous' \
		'snapshots and instrumentation change timing; event timings are not production performance' > "$(OUT)/records-events-flags.txt"

check-records-events: check-records-budget records-events-build
	$(call qualify-record-controls,events events-sanitized)
	OUT="$(OUT)" sh records-bench.sh events-check

.PHONY: mandelbrot-build check-mandelbrot
mandelbrot-build: scheduler-layout-build $(WFC)
	"$(WFC)" --par --par-ledger --emit-llvm mandelbrot.wf -o "$(OUT)/mandelbrot.ll" > "$(OUT)/mandelbrot.ledger"
	grep -F 'split independent map' "$(OUT)/mandelbrot.ledger"
	sed -e 's/@main(/@wf_mandelbrot_command_main(/g' -e 's/@wf__main_body(/@wf_mandelbrot_command_body(/g' \
		"$(OUT)/mandelbrot.ll" > "$(OUT)/mandelbrot-host.ll"
	cat mandelbrot_host.ll >> "$(OUT)/mandelbrot-host.ll"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -c "$(OUT)/mandelbrot-host.ll" -o "$(OUT)/mandelbrot-wf.o"
	$(CC) $(RECORD_FLAGS) -Wno-override-module -S "$(OUT)/mandelbrot-host.ll" -o "$(OUT)/mandelbrot-wf.s"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_BUDGET_CONTROL \
		-c runtime.c -o "$(OUT)/mandelbrot-runtime.o"
	$(CC) $(RECORD_FLAGS) -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_BUDGET_CONTROL \
		-c mandelbrot.c -o "$(OUT)/mandelbrot-host.o"
	$(CC) $(RECORD_FLAGS) -S mandelbrot.c -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_BUDGET_CONTROL -o "$(OUT)/mandelbrot-host.s"
	@for mode in ordinary sanitized; do \
		host="$(OUT)/mandelbrot-host.o"; runtime="$(OUT)/mandelbrot-runtime.o"; floor="$(OUT)/scheduler-floor.o"; flags=; \
		image="$(OUT)/mandelbrot"; \
		if test $$mode = sanitized; then \
			flags='-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer'; \
			for source in mandelbrot runtime; do \
				$(CC) $(RECORD_FLAGS) $$flags -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_BUDGET_CONTROL \
					-c $$source.c -o "$(OUT)/mandelbrot-sanitized-$$source.o" || exit $$?; \
			done; \
			$(CC) $(RECORD_FLAGS) $$flags -c $(FLOOR) -o "$(OUT)/mandelbrot-sanitized-floor.o" || exit $$?; \
			host="$(OUT)/mandelbrot-sanitized-mandelbrot.o"; runtime="$(OUT)/mandelbrot-sanitized-runtime.o"; \
			floor="$(OUT)/mandelbrot-sanitized-floor.o"; image="$(OUT)/mandelbrot-sanitized"; \
		fi; \
		set -- "$$host" "$$runtime" "$$floor" "$(OUT)/mandelbrot-wf.o" "$(OUT)/scheduler-layout-selector.o"; \
		for adapter in $(LAYOUT_SCHEDULERS); do set -- "$$@" "$(OUT)/scheduler-layout-$$adapter.o"; done; \
		$(CXX) -pthread -fno-lto $$flags "$$@" -L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" \
			-ltbb "$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$$image" || exit $$?; \
	done
	@nm "$(OUT)/mandelbrot" > "$(OUT)/mandelbrot.symbols"
	$(call require-strong,$(OUT)/mandelbrot,wf__par_acquire_lane)
	$(call require-strong,$(OUT)/mandelbrot,wf__floor_run)
	@printf '%s\n' '$(RECORD_FLAGS)' 'sanitizer=$(SANITIZER)' 'stats=0 events=0; generated map and all native schedulers in one image' \
		'sanitizers=host,native-kernel,runtime,floor; WF object and scheduler adapters ordinary' > "$(OUT)/mandelbrot-flags.txt"
	$(CC) --version >> "$(OUT)/mandelbrot-flags.txt"
	shasum -a 256 "$(WFC)" >> "$(OUT)/mandelbrot-flags.txt"

check-mandelbrot: mandelbrot-build
	@for width in 1 2 4; do \
		log="$(OUT)/mandelbrot-parlay-auto-exactly-once-w$$width.log"; status=0; \
		WF_WORKERS=$$width "$(OUT)/scheduler-layout" parlay-auto check $$width > "$$log" 2>&1 || status=$$?; \
		sh records-scheduler-memory.sh validate-clean "$$log" $$status parlay-auto $$width qualify || { cat "$$log" >&2; exit 1; }; \
	done
	OUT="$(OUT)" sh mandelbrot-bench.sh check
	OUT="$(OUT)" sh mandelbrot-bench.sh grain-check
	OUT="$(OUT)" sh mandelbrot-bench.sh split-check
	OUT="$(OUT)" sh mandelbrot-bench.sh allocation-check

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
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -DWF_COMPUTE_STATS=0 -c runtime.c -o "$(OUT)/scheduler-runtime.o"
	@nm "$(OUT)/scheduler-runtime.o" > "$(OUT)/scheduler-runtime.symbols"
	@if grep -E 'wf_compute_steal_count|wf__par_grants|wf_compute_event|wf_events' "$(OUT)/scheduler-runtime.symbols"; then exit 1; fi
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
	@for strategy in join iter; do \
		variant=0; if test $$strategy = iter; then variant=1; fi; \
		$(CC) -std=c11 $(SCHEDULER_FLAGS) -DRAYON_STRATEGY=$$variant -I"$(OUT)/scheduler-deps" \
			-c records_rayon.c -o "$(OUT)/scheduler-rayon-$$strategy-adapter.o" || exit $$?; \
		$(CXX) -pthread -fno-lto $(SCHEDULER_COMMON) "$(OUT)/scheduler-rayon-$$strategy-adapter.o" \
			"$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-rayon-$$strategy" || exit $$?; \
	done
	@printf '%s\n' 'C: $(CC) -std=c11 $(SCHEDULER_FLAGS)' \
		'C++: $(CXX) -std=c++17 $(SCHEDULER_FLAGS)' \
		'WF runtime: -DWF_COMPUTE_STATS=0; counter and observer absent' \
		'link: $(CXX) -pthread -fno-lto; common objects reused unchanged' > "$(OUT)/scheduler-flags.txt"
	$(CC) --version >> "$(OUT)/scheduler-flags.txt"
	$(CXX) --version >> "$(OUT)/scheduler-flags.txt"

check-scheduler-memory: scheduler-build
	@for unit in records_native records_oracle records_work records_runtime records_static runtime; do \
		stats=; if test $$unit = runtime; then stats=-DWF_COMPUTE_STATS=0; fi; \
		$(CC) -std=c11 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
			-fno-omit-frame-pointer $$stats -c $$unit.c -o "$(OUT)/scheduler-memory-$$unit.o" || exit $$?; \
	done
	@nm "$(OUT)/scheduler-memory-runtime.o" > "$(OUT)/scheduler-memory-runtime.symbols"
	@if grep -E 'wf_compute_steal_count|wf__par_grants|wf_compute_event|wf_events' "$(OUT)/scheduler-memory-runtime.symbols"; then exit 1; fi
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -c $(FLOOR) -o "$(OUT)/scheduler-memory-floor.o"
	@for unit in records_scheduler records_tbb records_parlay; do \
		$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
			-fno-omit-frame-pointer -isystem "$(OUT)/scheduler-deps/include" \
			-c $$unit.cpp -o "$(OUT)/scheduler-memory-$$unit.o" || exit $$?; \
	done
	@for strategy in join iter; do \
		variant=0; if test $$strategy = iter; then variant=1; fi; \
		$(CC) -std=c11 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
			-fno-omit-frame-pointer -DRAYON_STRATEGY=$$variant -I"$(OUT)/scheduler-deps" \
			-c records_rayon.c -o "$(OUT)/scheduler-memory-rayon-$$strategy.o" || exit $$?; \
	done
	@failures=0; for mode in $(SCHEDULERS); do \
		case $$mode in \
			wf) adapter='records_runtime runtime'; backend=wf-runtime;; \
			static) adapter=records_static; backend=static-spin;; \
			tbb) adapter=records_tbb; backend=oneTBB-v2023.1.0-auto-grain1;; \
			parlay) adapter=records_parlay; backend=parlay-native-grain1;; \
			rayon-join) adapter=$$mode; backend=rayon-1.12.0-join;; \
			rayon-iter) adapter=$$mode; backend=rayon-1.12.0-par-iter;; \
		esac; \
		set --; \
		for unit in records_scheduler records_native records_oracle records_work floor $$adapter; do \
			set -- "$$@" "$(OUT)/scheduler-memory-$$unit.o"; \
		done; \
		if test $$mode = tbb; then set -- "$$@" -L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" -ltbb; fi; \
		case $$mode in rayon-*) set -- "$$@" "$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS);; esac; \
		$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
			-o "$(OUT)/scheduler-memory-$$mode" || exit $$?; \
		for width in 1 2 4; do \
			log="$(OUT)/scheduler-memory-$$mode-w$$width.log"; status=0; \
			env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=$$width "$(OUT)/scheduler-memory-$$mode" check $$width \
				> "$$log" 2>&1 || status=$$?; \
			if sh records-scheduler-memory.sh $(SCHEDULER_MEMORY_MODE) "$$log" $$status "$$backend" $$width qualify; then \
				printf 'scheduler memory accepted: backend=%s width=%s case=qualify child_status=%s\n' "$$backend" $$width $$status; \
			else \
				failures=$$((failures+1)); \
				cat "$$log" >&2; \
			fi; \
		done; \
		for cadence in checked dense sleep-100us sleep-1ms; do \
			log="$(OUT)/scheduler-memory-$$mode-$$cadence.log"; status=0; \
			env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=2 "$(OUT)/scheduler-memory-$$mode" 2 33 17 unicode 16 2 828219 0 $$cadence \
				> "$$log" 2>&1 || status=$$?; \
			if sh records-scheduler-memory.sh $(SCHEDULER_MEMORY_MODE) "$$log" $$status "$$backend" 2 $$cadence; then \
				printf 'scheduler memory accepted: backend=%s width=2 case=%s child_status=%s\n' "$$backend" $$cadence $$status; \
			else \
				failures=$$((failures+1)); \
				cat "$$log" >&2; \
			fi; \
		done; \
	done; test $$failures -eq 0
	# Qualify the new grouped paths with instrumented computation and runtime,
	# in addition to the shared host/selector checks and original adapter suite.
	@for group in 4 16; do \
		$(CC) -std=c11 $(SCHEDULER_FLAGS) -DRECORD_WF_GROUP=$$group \
			-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer \
			-c records_runtime.c -o "$(OUT)/scheduler-memory-wf-group$$group.o" || exit $$?; \
		set -- "$(OUT)/scheduler-memory-records_scheduler.o" "$(OUT)/scheduler-memory-wf-group$$group.o"; \
		for unit in records_native records_oracle records_work floor runtime; do \
			set -- "$$@" "$(OUT)/scheduler-memory-$$unit.o"; \
		done; \
		$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
			-o "$(OUT)/scheduler-memory-wf-group$$group" || exit $$?; \
		for width in 1 2 4; do \
			log="$(OUT)/scheduler-memory-wf-group$$group-w$$width.log"; status=0; \
			env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=$$width "$(OUT)/scheduler-memory-wf-group$$group" \
				check $$width > "$$log" 2>&1 || status=$$?; \
			sh records-scheduler-memory.sh $(SCHEDULER_MEMORY_MODE) "$$log" $$status wf-group$$group $$width qualify \
				|| { cat "$$log" >&2; exit 1; }; \
			cat "$$log"; \
		done; \
	done
	@if test "$$(uname -s)-$$(uname -m)" = Linux-x86_64 && test "$(SCHEDULER_MEMORY_MODE)" = validate; then \
		mkdir -p "$(OUT)/scheduler-memory-extra" || exit $$?; \
		$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
			-fno-omit-frame-pointer -DRECORD_SCHEDULER_LEAK_PROBE -c records_scheduler.cpp \
			-o "$(OUT)/scheduler-memory-leak-host.o" || exit $$?; \
		set -- "$(OUT)/scheduler-memory-leak-host.o"; \
		for unit in records_native records_oracle records_work floor rayon-join; do \
			set -- "$$@" "$(OUT)/scheduler-memory-$$unit.o"; \
		done; \
		$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
			"$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-memory-extra/scheduler-memory-rayon-join" || exit $$?; \
		log="$(OUT)/scheduler-memory-extra-leak.log"; status=0; \
		env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=2 "$(OUT)/scheduler-memory-extra/scheduler-memory-rayon-join" check 2 > "$$log" 2>&1 || status=$$?; \
		sh records-scheduler-memory.sh validate "$$log" $$status rayon-1.12.0-join 2 qualify 257 || { cat "$$log" >&2; exit 1; }; \
		if sh records-scheduler-memory.sh validate "$$log" $$status rayon-1.12.0-join 2 qualify \
			> "$(OUT)/scheduler-memory-extra-rejection.log" 2>&1; then \
			echo 'scheduler memory checker accepted injected extra leak' >&2; exit 1; \
		fi; \
		grep -Fxq 'scheduler sanitizer report rejected: unknown allocation or summary' \
			"$(OUT)/scheduler-memory-extra-rejection.log" || { cat "$(OUT)/scheduler-memory-extra-rejection.log" >&2; exit 1; }; \
		echo 'scheduler memory negative PASS: additional 257-byte C leak rejected after full qualification'; \
	else echo 'scheduler memory extra-leak qualification: Linux x86_64 ASan only'; fi

check-scheduler: scheduler-build check-scheduler-memory records-build
	cargo fmt --manifest-path records-rayon/Cargo.toml -- --check
	# Qualify the exact scalar timing leaf with the complete independent oracle.
	$(CC) -std=c11 $(SCHEDULER_FLAGS) records.c records_oracle.c $(FLOOR) \
		"$(OUT)/records-wf.o" "$(OUT)/scheduler-native.o" -o "$(OUT)/scheduler-leaf-check"
	WF_WORKERS=0 "$(OUT)/scheduler-leaf-check" check > "$(OUT)/scheduler-leaf-check.log"
	grep -Fx 'UTF8 leaf qualification PASS: calls=4595603 all-codepoints all-truncations two-byte-pairs' "$(OUT)/scheduler-leaf-check.log"
	grep -Fx 'UTF8 batch qualification PASS: world=sequential calls=342 results=1821070' "$(OUT)/scheduler-leaf-check.log"
	@for mode in $(SCHEDULERS); do \
		for width in 1 2 4; do \
			WF_WORKERS=$$width "$(OUT)/scheduler-$$mode" check $$width > "$(OUT)/scheduler-$$mode-check-w$$width.log" || exit $$?; \
			cat "$(OUT)/scheduler-$$mode-check-w$$width.log"; \
		done; \
	done
	OUT="$(OUT)" sh records-scheduler-bench.sh check

scheduler-calibrate: check-scheduler
	OUT="$(OUT)" sh records-scheduler-bench.sh calibrate

# Keep one common machine-code image while changing only the selected adapter.
# Macro renames affect each adapter's three public entry points; all common C
# objects are exactly those already qualified by the standalone scheduler panel.
scheduler-layout-build: scheduler-build
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -DRECORD_SCHEDULER_SELECT \
		-c records_scheduler.cpp -o "$(OUT)/scheduler-layout-host.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -c records_selector.c -o "$(OUT)/scheduler-layout-selector.o"
	@for mode in $(LAYOUT_SCHEDULERS); do \
		prefix=layout_$$(printf '%s' "$$mode" | tr - _); \
		set -- -Drecords_scheduler_run=$${prefix}_run -Drecords_scheduler_name=$${prefix}_name \
			-Drecords_scheduler_stop=$${prefix}_stop; \
		case $$mode in \
			wf-group4|wf-group16) group=$${mode#wf-group}; \
				$(CC) -std=c11 $(SCHEDULER_FLAGS) "$$@" -DRECORD_WF_GROUP=$$group \
					-c records_runtime.c -o "$(OUT)/scheduler-layout-$$mode.o" || exit $$?;; \
			wf|static) source=records_static.c; if test $$mode = wf; then source=records_runtime.c; fi; \
				$(CC) -std=c11 $(SCHEDULER_FLAGS) "$$@" -c $$source -o "$(OUT)/scheduler-layout-$$mode.o" || exit $$?;; \
			tbb|parlay) $(CXX) -std=c++17 $(SCHEDULER_FLAGS) "$$@" -isystem "$(OUT)/scheduler-deps/include" \
				-c records_$$mode.cpp -o "$(OUT)/scheduler-layout-$$mode.o" || exit $$?;; \
			rayon-join|rayon-iter) variant=0; if test $$mode = rayon-iter; then variant=1; fi; \
				$(CC) -std=c11 $(SCHEDULER_FLAGS) "$$@" -DRAYON_STRATEGY=$$variant -I"$(OUT)/scheduler-deps" \
				-c records_rayon.c -o "$(OUT)/scheduler-layout-$$mode.o" || exit $$?;; \
		esac; \
	done
	@set -- "$(OUT)/scheduler-layout-host.o" "$(OUT)/scheduler-layout-selector.o"; \
	for unit in native oracle work floor runtime; do set -- "$$@" "$(OUT)/scheduler-$$unit.o"; done; \
	for mode in $(LAYOUT_SCHEDULERS); do set -- "$$@" "$(OUT)/scheduler-layout-$$mode.o"; done; \
	$(CXX) -pthread -fno-lto "$$@" -L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" \
		-ltbb "$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-layout"
	@nm -n "$(OUT)/scheduler-layout" > "$(OUT)/scheduler-layout.symbols"
	$(call require-strong,$(OUT)/scheduler-layout,wf__floor_run)
	$(call require-strong,$(OUT)/scheduler-layout,records_state)
	$(call require-strong,$(OUT)/scheduler-layout,records_scheduler_chunk)
	@for symbol in records_state records_scheduler_chunk; do \
		test "$$(awk -v name="$$symbol" '$$2=="T" && ($$3==name || $$3=="_"name) {n++} END {print n+0}' \
			"$(OUT)/scheduler-layout.symbols")" -eq 1 || exit 1; \
	done
	@printf '%s\n' 'image=scheduler-layout selection=before-floor-entry dispatch=common-indirect' \
		'loaded=WF-runtime,static,oneTBB,Parlay,Rayon-join,Rayon-iter' \
		'WF_callback_grouping=original,4,16; compiler_cost_policy=unchanged' \
		'common_objects=scheduler-native.o,scheduler-work.o,scheduler-oracle.o,scheduler-floor.o,scheduler-runtime.o' \
		'sanitizer_scope=selector-and-host; ordinary-qualified-adapters-and-common-objects' > "$(OUT)/scheduler-layout-flags.txt"

check-scheduler-layout: check-scheduler scheduler-layout-build
	@for mode in $(LAYOUT_SCHEDULERS); do \
		for width in 1 2 4; do \
			log="$(OUT)/scheduler-layout-$$mode-check-w$$width.log"; status=0; \
			WF_WORKERS=$$width "$(OUT)/scheduler-layout" $$mode check $$width > "$$log" 2>&1 || status=$$?; \
			sh records-scheduler-memory.sh validate-clean "$$log" $$status $$mode $$width qualify || { cat "$$log" >&2; exit 1; }; \
			cat "$$log"; \
		done; \
	done
	# This additional check instruments the new selector and shared host only.
	# Existing standalone sanitizer qualification above still covers all adapters.
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -DRECORD_SCHEDULER_SELECT -c records_scheduler.cpp -o "$(OUT)/scheduler-layout-memory-host.o"
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -fsanitize=$(SANITIZER) -fno-sanitize-recover=all \
		-fno-omit-frame-pointer -c records_selector.c -o "$(OUT)/scheduler-layout-memory-selector.o"
	@set -- "$(OUT)/scheduler-layout-memory-host.o" "$(OUT)/scheduler-layout-memory-selector.o"; \
	for unit in native oracle work floor runtime; do set -- "$$@" "$(OUT)/scheduler-$$unit.o"; done; \
	for mode in $(LAYOUT_SCHEDULERS); do set -- "$$@" "$(OUT)/scheduler-layout-$$mode.o"; done; \
	$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
		-L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" \
		-ltbb "$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-layout-memory-image"
	@mkdir -p "$(OUT)/scheduler-layout-memory"; failures=0; \
	for mode in $(LAYOUT_SCHEDULERS); do \
		image="$(OUT)/scheduler-layout-memory/scheduler-memory-$$mode"; \
		ln -f "$(OUT)/scheduler-layout-memory-image" "$$image" || exit $$?; \
		cmp "$(OUT)/scheduler-layout-memory-image" "$$image" || exit $$?; \
		for width in 1 2 4; do \
			log="$(OUT)/scheduler-layout-memory-$$mode-w$$width.log"; status=0; \
			env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=$$width "$$image" $$mode check $$width > "$$log" 2>&1 || status=$$?; \
			if sh records-scheduler-memory.sh $(SCHEDULER_MEMORY_MODE) "$$log" $$status $$mode $$width qualify; then \
				printf 'scheduler layout memory accepted: backend=%s width=%s child_status=%s\n' $$mode $$width $$status; \
			else failures=$$((failures+1)); cat "$$log" >&2; fi; \
		done; \
	done; test $$failures -eq 0
	@for argument in missing invalid; do \
		set --; if test $$argument = invalid; then set -- unknown check 1; fi; \
		status=0; "$(OUT)/scheduler-layout" "$$@" > "$(OUT)/scheduler-layout-$$argument.log" 2>&1 || status=$$?; \
		test $$status -eq 1 || exit 1; \
		message='record scheduler: shared-image backend argument'; \
		if test $$argument = invalid; then message='record scheduler: invalid or repeated shared-image selection'; fi; \
		grep -Fxq "$$message" "$(OUT)/scheduler-layout-$$argument.log" || exit 1; \
	done
	OUT="$(OUT)" sh records-scheduler-bench.sh layout-check

scheduler-layout-calibrate: check-scheduler-layout
	OUT="$(OUT)" sh records-scheduler-bench.sh layout

# A separate observer image retains the ordinary shared leaf and adapters.
# Its plain/identity/timeline levels compare observation perturbation; the
# original timing image contains no trace code or event storage.
scheduler-trace-build: scheduler-layout-build
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -DRECORD_SCHEDULER_SELECT -DRECORD_SCHEDULER_TRACE \
		-c records_scheduler.cpp -o "$(OUT)/scheduler-trace-host.o"
	@set -- "$(OUT)/scheduler-trace-host.o" "$(OUT)/scheduler-layout-selector.o"; \
	for unit in native oracle work floor runtime; do set -- "$$@" "$(OUT)/scheduler-$$unit.o"; done; \
	for mode in $(LAYOUT_SCHEDULERS); do set -- "$$@" "$(OUT)/scheduler-layout-$$mode.o"; done; \
	$(CXX) -pthread -fno-lto "$$@" -L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" \
		-ltbb "$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-trace"
	@nm -n "$(OUT)/scheduler-trace" > "$(OUT)/scheduler-trace.symbols"
	$(call require-strong,$(OUT)/scheduler-trace,records_state)
	$(call require-strong,$(OUT)/scheduler-trace,records_scheduler_chunk)
	@if grep -E 'wf_compute_steal_count|wf__par_grants|wf_compute_event|wf_events' "$(OUT)/scheduler-trace.symbols"; then exit 1; fi
	@printf '%s\n' 'image=scheduler-trace levels=plain,identity,timeline' \
		'events=common-callback; runtime-jobs-and-OS-switch-events=unobserved' \
		'identity=per-cell-relaxed-claim,cached-native-TID; timeline=identity-plus-two-clocks' \
		'interval=common-callback-wall; includes-preemption; not-CPU-occupancy' \
		'$(SCHEDULER_FLAGS)' > "$(OUT)/scheduler-trace-flags.txt"

check-scheduler-trace: check-scheduler-layout scheduler-trace-build
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -DRECORD_SCHEDULER_SELECT -DRECORD_SCHEDULER_TRACE \
		-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer \
		-c records_scheduler.cpp -o "$(OUT)/scheduler-trace-memory-host.o"
	@set -- "$(OUT)/scheduler-trace-memory-host.o" "$(OUT)/scheduler-layout-memory-selector.o"; \
	for unit in records_native records_oracle records_work floor runtime; do \
		set -- "$$@" "$(OUT)/scheduler-memory-$$unit.o"; \
	done; \
	for mode in $(LAYOUT_SCHEDULERS); do set -- "$$@" "$(OUT)/scheduler-layout-$$mode.o"; done; \
	$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
		-L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" \
		-ltbb "$(RAYON_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-trace-memory-image"
	@mkdir -p "$(OUT)/scheduler-trace-memory"; \
	for mode in $(LAYOUT_SCHEDULERS); do \
		image="$(OUT)/scheduler-trace-memory/scheduler-memory-$$mode"; \
		ln -f "$(OUT)/scheduler-trace-memory-image" "$$image" || exit $$?; \
		for width in 1 2 4; do for level in plain identity timeline; do \
			log="$(OUT)/scheduler-trace-memory-$$mode-w$$width-$$level.log"; status=0; \
			env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=$$width "$$image" $$mode trace $$width 33 17 unicode 16 2 828219 0 $$level \
				> "$$log" 2>&1 || status=$$?; \
			sh records-scheduler-memory.sh $(SCHEDULER_MEMORY_MODE) "$$log" $$status $$mode $$width trace-$$level \
				|| { cat "$$log" >&2; exit 1; }; \
			printf 'scheduler trace memory accepted: backend=%s width=%s level=%s child_status=%s\n' $$mode $$width $$level $$status; \
		done; done; \
	done
	OUT="$(OUT)" sh records-scheduler-trace.sh check

# Runtime counters remain confined to this diagnostic image. Its common scalar
# computation and all non-Rayon adapters reuse the ordinary timing objects.
scheduler-events-build: scheduler-trace-build
	OUT="$(OUT)" sh records-rayon-events.sh build
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_EVENTS \
		-c runtime.c -o "$(OUT)/scheduler-events-runtime.o"
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -DRECORD_SCHEDULER_SELECT -DRECORD_SCHEDULER_TRACE \
		-DRECORD_SCHEDULER_EVENTS -I"$(OUT)/scheduler-events-deps" \
		-c records_scheduler.cpp -o "$(OUT)/scheduler-events-host.o"
	@set -e; for strategy in join iter; do \
		variant=0; if test $$strategy = iter; then variant=1; fi; \
		$(CC) -std=c11 $(SCHEDULER_FLAGS) -DRAYON_STRATEGY=$$variant -I"$(OUT)/scheduler-events-deps" \
			-Drecords_scheduler_run=layout_rayon_$${strategy}_run \
			-Drecords_scheduler_name=layout_rayon_$${strategy}_name \
			-Drecords_scheduler_stop=layout_rayon_$${strategy}_stop \
			-c records_rayon.c -o "$(OUT)/scheduler-events-rayon-$$strategy.o"; \
	done
	@set -- "$(OUT)/scheduler-events-host.o" "$(OUT)/scheduler-events-runtime.o" "$(OUT)/scheduler-layout-selector.o"; \
	for unit in native oracle work floor; do set -- "$$@" "$(OUT)/scheduler-$$unit.o"; done; \
	for mode in wf wf-group4 wf-group16 static tbb parlay; do set -- "$$@" "$(OUT)/scheduler-layout-$$mode.o"; done; \
	$(CXX) -pthread -fno-lto "$$@" "$(OUT)/scheduler-events-rayon-join.o" "$(OUT)/scheduler-events-rayon-iter.o" \
		-L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" -ltbb \
		"$(RAYON_EVENTS_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-events"
	@nm -n "$(OUT)/scheduler-events" > "$(OUT)/scheduler-events.symbols"
	@printf '%s\n' 'image=scheduler-events; trace levels unchanged; internal events always enabled' \
		'WF=wf-2,24 counters; Rayon=rayon-join-1,13 counters; schemas are not interchangeable' \
		'per-call cumulative snapshots outside call clocks, inside batch; per-bank deltas follow call rows' \
		'ordinary scheduler-trace is the counter-free control; separate image layout contributes perturbation' \
		'$(SCHEDULER_FLAGS)' > "$(OUT)/scheduler-events-flags.txt"

check-scheduler-events: check-runtime check-scheduler-trace scheduler-events-build
	$(CC) -std=c11 $(SCHEDULER_FLAGS) -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_EVENTS \
		-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer \
		-c runtime.c -o "$(OUT)/scheduler-events-memory-runtime.o"
	$(CXX) -std=c++17 $(SCHEDULER_FLAGS) -DRECORD_SCHEDULER_SELECT -DRECORD_SCHEDULER_TRACE \
		-DRECORD_SCHEDULER_EVENTS -I"$(OUT)/scheduler-events-deps" \
		-fsanitize=$(SANITIZER) -fno-sanitize-recover=all -fno-omit-frame-pointer \
		-c records_scheduler.cpp -o "$(OUT)/scheduler-events-memory-host.o"
	@set -- "$(OUT)/scheduler-events-memory-host.o" "$(OUT)/scheduler-events-memory-runtime.o" \
		"$(OUT)/scheduler-layout-memory-selector.o"; \
	for unit in records_native records_oracle records_work floor; do set -- "$$@" "$(OUT)/scheduler-memory-$$unit.o"; done; \
	for mode in wf wf-group4 wf-group16 static tbb parlay; do set -- "$$@" "$(OUT)/scheduler-layout-$$mode.o"; done; \
	$(CXX) -pthread -fno-lto -fsanitize=$(SANITIZER) -fno-sanitize-recover=all "$$@" \
		"$(OUT)/scheduler-events-rayon-join.o" "$(OUT)/scheduler-events-rayon-iter.o" \
		-L"$(OUT)/scheduler-deps/lib" -Wl,-rpath,"$(OUT)/scheduler-deps/lib" -ltbb \
		"$(RAYON_EVENTS_LIB)" $(RAYON_SYSTEM_LIBS) -o "$(OUT)/scheduler-events-memory-image"
	@mkdir -p "$(OUT)/scheduler-events-memory"; \
	for mode in wf rayon-join; do \
		image="$(OUT)/scheduler-events-memory/scheduler-memory-$$mode"; \
		ln -f "$(OUT)/scheduler-events-memory-image" "$$image" || exit $$?; \
		for width in 1 2 4; do for level in plain identity timeline; do \
			log="$(OUT)/scheduler-events-memory-$$mode-w$$width-$$level.log"; status=0; \
			env $(SCHEDULER_MEMORY_ENV) WF_WORKERS=$$width "$$image" $$mode trace $$width 33 17 unicode 16 2 828219 0 $$level \
				> "$$log" 2>&1 || status=$$?; \
			sh records-scheduler-memory.sh $(SCHEDULER_MEMORY_MODE) "$$log" $$status $$mode $$width events-$$level \
				|| { cat "$$log" >&2; exit 1; }; \
			printf 'scheduler events memory accepted: backend=%s width=%s level=%s child_status=%s\n' $$mode $$width $$level $$status; \
		done; done; \
	done
	OUT="$(OUT)" sh records-scheduler-trace.sh events-check

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
