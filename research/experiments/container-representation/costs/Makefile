CC ?= cc
RUSTC ?= rustc
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
BUILD := .build

.PHONY: check measure

$(BUILD):
	mkdir -p $@

$(BUILD)/map-layout$(EXE): map-layout.c | $(BUILD)
	$(CC) -std=c11 -O2 -Wall -Wextra -Werror $< -o $@

$(BUILD)/rust-map$(EXE): rust-map.rs | $(BUILD)
	$(RUSTC) --edition=2024 -C opt-level=2 -D warnings $< -o $@

check: $(BUILD)/map-layout$(EXE) $(BUILD)/rust-map$(EXE)
	rustfmt --edition 2024 --check rust-map.rs
	$(BUILD)/map-layout$(EXE) check
	$(BUILD)/rust-map$(EXE) check

measure: check
	$(BUILD)/map-layout$(EXE) measure > $(BUILD)/samples.csv
	$(BUILD)/rust-map$(EXE) measure > $(BUILD)/rust-samples.csv
	cat $(BUILD)/samples.csv
	cat $(BUILD)/rust-samples.csv
