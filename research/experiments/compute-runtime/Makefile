ROOT := ../../..
.DEFAULT_GOAL := check
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
OUT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-compute-runtime
CC := /usr/bin/clang
SANITIZER ?= address,undefined
CFLAGS := -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread
FLOOR := $(ROOT)/compiler/src/backend/wf_floor.c
WFC ?= $(OUT)/compiler/gate/whitefootc

.PHONY: check build check-runtime check-abi compiler force

force:

build: $(OUT)/probe

$(OUT)/probe: runtime.c runtime.h probe.c $(FLOOR) force
	mkdir -p "$(OUT)"
	$(CC) $(CFLAGS) -DWF_COMPUTE_TEST -fsanitize=$(SANITIZER) -fno-omit-frame-pointer \
		runtime.c probe.c $(FLOOR) -o "$@"

check: check-runtime check-abi

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
	grep -E ' T _?wf__par_acquire_lane$$' "$(OUT)/abi.symbols"
	@if grep -E ' [Tt] _?(wf_sched_|wf_completion_|wf__completion_)' "$(OUT)/abi.symbols"; then \
		echo 'compute ABI binary unexpectedly links shared scheduler or completion code' >&2; exit 1; \
	fi
