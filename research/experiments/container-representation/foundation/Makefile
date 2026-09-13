RUSTC ?= rustc
CC ?= cc
CLANG ?= clang
ROOT := $(abspath ../../../..)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)
BUILD := .build
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)

.PHONY: check measure

$(BUILD):
	mkdir -p $@

$(BUILD)/model$(EXE): model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/tests$(EXE): model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

$(BUILD)/construction$(EXE): construction.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/construction-tests$(EXE): construction.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

$(BUILD)/rust-baseline$(EXE): rust-baseline.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/rust-baseline-tests$(EXE): rust-baseline.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

$(BUILD)/layout$(EXE): layout.c | $(BUILD)
	$(CC) -std=c11 -O2 -Wall -Wextra -Werror $< -o $@

$(BUILD)/large-result.ll: large-result.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) --emit-llvm -o $@ $<

$(BUILD)/large-result$(EXE): large-result.wf $(WHITEFOOTC) | $(BUILD)
	$(WHITEFOOTC) -o $@ $<

# Keep the producer boundary to inspect aggregate materialization independently
# of inlining. This native control is not a source acceptance path.
$(BUILD)/retained-result.ll: $(BUILD)/large-result.ll
	sed '/^define void @wf_make_record/s/ #0 {/ noinline #0 {/' $< > $@

$(BUILD)/optimized-result.ll: $(BUILD)/retained-result.ll
	awk '/^define void @wf_make_record.*noinline/ { producer++ } /^define void @wf_build_record/ { consumer++ } END { exit(producer != 1 || consumer != 1) }' $<
	$(CLANG) -O2 -Wno-override-module -S -emit-llvm -x ir $< -o $@

check: $(BUILD)/model$(EXE) $(BUILD)/tests$(EXE) $(BUILD)/layout$(EXE) \
       $(BUILD)/large-result.ll $(BUILD)/large-result$(EXE) \
       $(BUILD)/construction$(EXE) $(BUILD)/construction-tests$(EXE) \
       $(BUILD)/rust-baseline$(EXE) $(BUILD)/rust-baseline-tests$(EXE)
	rustfmt --edition 2024 --check model.rs construction.rs rust-baseline.rs
	$(BUILD)/tests$(EXE)
	$(BUILD)/model$(EXE)
	$(BUILD)/layout$(EXE)
	$(BUILD)/large-result$(EXE)
	$(BUILD)/construction-tests$(EXE)
	$(BUILD)/construction$(EXE)
	$(BUILD)/rust-baseline-tests$(EXE)
	$(BUILD)/rust-baseline$(EXE)

measure: check $(BUILD)/optimized-result.ll
	$(BUILD)/layout$(EXE)
