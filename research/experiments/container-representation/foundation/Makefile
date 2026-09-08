RUSTC ?= rustc
CC ?= cc
BUILD := .build
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)

.PHONY: check measure

$(BUILD):
	mkdir -p $@

$(BUILD)/model$(EXE): model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/tests$(EXE): model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

$(BUILD)/layout$(EXE): layout.c | $(BUILD)
	$(CC) -std=c11 -O2 -Wall -Wextra -Werror $< -o $@

check: $(BUILD)/model$(EXE) $(BUILD)/tests$(EXE) $(BUILD)/layout$(EXE)
	rustfmt --edition 2024 --check model.rs
	$(BUILD)/tests$(EXE)
	$(BUILD)/model$(EXE)
	$(BUILD)/layout$(EXE)

measure: check
	$(BUILD)/layout$(EXE)
