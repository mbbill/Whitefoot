# The C measurement harnesses link the same ordinary library as whitefootc.
# Keep this list aligned with compiler/src/bin/whitefootc.rs::runtime_units.
# This build-only include is owned by dense/ and families/; remove it when
# the compiler ships a reusable linked-library artifact for these harnesses.
NATIVE_ROOT := $(ROOT)/compiler/src/backend
NATIVE_C := ordinary_values.c sched/core.c sched/entry.c \
            completion/runtime.c completion/file_adapter.c completion/bridge.c
ifeq ($(OS),Windows_NT)
NATIVE_C += wf_floor_windows.c windows_runtime.c sched/prim_windows.c \
            completion/wait_windows.c completion/file_windows.c completion/windows_iocp.c
NATIVE_COMPILE_FLAGS :=
NATIVE_LINK_FLAGS := -lws2_32 -lshell32
else
NATIVE_C += wf_floor.c sched/prim_host.c completion/wait_host.c \
            completion/file_posix.c completion/linux_io_uring.c
NATIVE_COMPILE_FLAGS := -pthread
NATIVE_LINK_FLAGS := -pthread -lm
endif
NATIVE_HEADERS := $(wildcard $(NATIVE_ROOT)/*.h $(NATIVE_ROOT)/sched/*.h $(NATIVE_ROOT)/completion/*.h)
NATIVE_OBJECTS := $(addprefix $(BUILD)/native/,$(NATIVE_C:.c=.o)) $(BUILD)/native/ordinary_values_ir.o

$(BUILD)/native/%.o: $(NATIVE_ROOT)/%.c $(NATIVE_HEADERS)
	mkdir -p $(dir $@)
	$(CLANG) -std=c11 -O2 $(NATIVE_COMPILE_FLAGS) -I$(NATIVE_ROOT) -I$(NATIVE_ROOT)/completion -c $< -o $@

$(BUILD)/native/%_ir.o: $(NATIVE_ROOT)/%.ll
	mkdir -p $(dir $@)
	$(CLANG) -O2 -Wno-override-module -x ir -c $< -o $@
