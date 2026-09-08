RUSTC ?= rustc
BUILD := .build

.PHONY: check measure

$(BUILD):
	mkdir -p $@

$(BUILD)/model: model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/tests: model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

check: $(BUILD)/tests $(BUILD)/model
	$(BUILD)/tests
	$(BUILD)/model

measure: $(BUILD)/model
	$(BUILD)/model
