ROOT := $(abspath ../../../..)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc
CLANG ?= clang
RUSTC ?= rustc
BUILD := build
SIZES := 16 256 4096
case_size = $(word 1,$(subst x, ,$(1)))
case_lanes = $(if $(findstring x,$(1)),4,1)
case_flags = -DSIZE=$(call case_size,$(1)) -DLANES=$(call case_lanes,$(1))
CFLAGS := -O2 -std=c17 -Wall -Wextra -Werror
IRFLAGS := -O2 -Wno-override-module

.PHONY: all check measure shape clean
.PRECIOUS: $(BUILD)/dense%.wf $(BUILD)/raw%.ll $(BUILD)/linked%.ll \
           $(BUILD)/dense%.o $(BUILD)/native%.o
all: $(SIZES:%=$(BUILD)/bench%) $(SIZES:%=$(BUILD)/smoke%) \
     $(BUILD)/boundary-bench256 shape

$(BUILD):
	mkdir -p $@

$(BUILD)/driver: driver.rs | $(BUILD)
	$(RUSTC) --edition=2024 -C opt-level=2 -D warnings $< -o $@

$(BUILD)/dense%.wf: dense.wf.in $(BUILD)/driver
	$(BUILD)/driver generate $< $* $@

$(BUILD)/raw%.ll: $(BUILD)/dense%.wf $(WHITEFOOTC)
	$(WHITEFOOTC) --emit-llvm -o $@ $<

$(BUILD)/linked%.ll: $(BUILD)/raw%.ll $(BUILD)/driver
	$(BUILD)/driver adapt $< $@

$(BUILD)/dense%.o: $(BUILD)/dense%.s
	$(CLANG) -c $< -o $@

$(BUILD)/native%.o: reference.c
	$(CLANG) $(CFLAGS) $(call case_flags,$*) -fstack-usage -c $< -o $@

$(BUILD)/bench%: harness.c $(BUILD)/dense%.o $(BUILD)/native%.o
	$(CLANG) $(CFLAGS) $(call case_flags,$*) $^ -o $@

$(BUILD)/smoke%: $(BUILD)/dense%.wf $(WHITEFOOTC)
	$(WHITEFOOTC) -o $@ $<

$(BUILD)/optimized%.ll: $(BUILD)/linked%.ll
	$(CLANG) $(IRFLAGS) -x ir -S -emit-llvm $< -o $@

$(BUILD)/dense%.s: $(BUILD)/linked%.ll
	$(CLANG) $(IRFLAGS) -fstack-usage -x ir -S $< -o $@

$(BUILD)/native%.opt.ll: reference.c
	$(CLANG) $(CFLAGS) $(call case_flags,$*) -S -emit-llvm $< -o $@

$(BUILD)/native%.s: reference.c
	$(CLANG) $(CFLAGS) $(call case_flags,$*) -S $< -o $@

$(BUILD)/boundary-bench256: harness.c reference.c $(BUILD)/dense256.o
	$(CLANG) $(CFLAGS) -DSIZE=256 -DLANES=1 -DBOUNDARY_CONTROL=1 $^ -o $@

$(BUILD)/boundary256.ll: reference.c
	$(CLANG) $(CFLAGS) -DSIZE=256 -DLANES=1 -DBOUNDARY_CONTROL=1 -S -emit-llvm $< -o $@

shape: $(SIZES:%=$(BUILD)/optimized%.ll) $(SIZES:%=$(BUILD)/dense%.s) \
       $(SIZES:%=$(BUILD)/native%.opt.ll) $(SIZES:%=$(BUILD)/native%.s)

shape: $(BUILD)/boundary256.ll

check: all
	rustfmt --edition 2024 --check driver.rs
	@set -eu; status=0; rm -f $(BUILD)/inline-view.ll; \
	  $(WHITEFOOTC) --emit-llvm -o $(BUILD)/inline-view.ll inline-view.wf > $(BUILD)/inline-view.log 2>&1 || status=$$?; \
	  if test "$$status" -eq 0; then \
	    $(WHITEFOOTC) -o $(BUILD)/inline-view inline-view.wf; $(BUILD)/inline-view; \
	    echo 'inline view: supported and executed'; \
	  else \
	    test "$$status" -eq 1; test ! -f $(BUILD)/inline-view.ll; \
	    grep -F 'Semantics/Unsupported' $(BUILD)/inline-view.log >/dev/null; \
	    grep -F 'ExclusiveViewOverInlineRun' $(BUILD)/inline-view.log >/dev/null; \
	    echo 'inline view: compiler capability unsupported (ExclusiveViewOverInlineRun)'; \
	  fi
	$(BUILD)/driver generate dense.wf.in 16x4 $(BUILD)/wide-probe.wf
	$(BUILD)/driver probe $(WHITEFOOTC) $(BUILD)/wide-probe.wf $(BUILD)/wide-probe.txt
	@set -eu; for size in $(SIZES); do \
		$(BUILD)/smoke$$size; \
		$(BUILD)/bench$$size verify 7640891576956012809; \
	done
	$(BUILD)/boundary-bench256 verify 7640891576956012809

measure: check
	@set -eu; : > $(BUILD)/measurements.csv; for size in $(SIZES); do \
		$(BUILD)/bench$$size measure 7640891576956012809 >> $(BUILD)/measurements.csv; \
	done
	$(BUILD)/driver summarize $(BUILD)/measurements.csv > $(BUILD)/summary.csv
	cat $(BUILD)/summary.csv

clean:
	rm -rf $(BUILD)
