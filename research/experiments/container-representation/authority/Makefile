RUSTC ?= rustc
BUILD := .build

.PHONY: check measure

$(BUILD):
	mkdir -p $@

$(BUILD)/model: model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/tests: model.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

$(BUILD)/membership: membership.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 $< -o $@

$(BUILD)/membership-tests: membership.rs | $(BUILD)
	$(RUSTC) --edition=2024 -D warnings -C opt-level=2 --test $< -o $@

check: $(BUILD)/tests $(BUILD)/model $(BUILD)/membership-tests $(BUILD)/membership
	rustfmt --edition 2024 --check membership.rs
	$(BUILD)/tests
	$(BUILD)/model
	$(BUILD)/membership-tests
	$(BUILD)/membership

measure: $(BUILD)/model
	$(BUILD)/model
