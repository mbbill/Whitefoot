RUSTC ?= rustc
BUILD := .build
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)

.PHONY: check local

$(BUILD):
	mkdir -p $@

$(BUILD)/model$(EXE): model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/tests$(EXE): model.rs local.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

check: $(BUILD)/model$(EXE) $(BUILD)/tests$(EXE)
	rustfmt --edition 2024 --check model.rs local.rs
	$(BUILD)/tests$(EXE)
	$(BUILD)/model$(EXE)

local: $(BUILD)/tests$(EXE)
	rustfmt --edition 2024 --check model.rs local.rs
	$(BUILD)/tests$(EXE) local:: --nocapture --test-threads=1
