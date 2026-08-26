CARGO := cargo
CC ?= cc
NM ?= nm

COMPLETION_DIR := src/backend/completion
COMPLETION_TMP ?= $(HOME)/do_not_scan/whitefoot-completion-test
COMPLETION_CFLAGS := -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread
COMPLETION_SOURCES := \
	$(COMPLETION_DIR)/runtime.c \
	$(COMPLETION_DIR)/file_adapter.c \
	$(COMPLETION_DIR)/native_contract.c \
	$(COMPLETION_DIR)/harness.c

check: format lint test docs spec completion-test
	@echo "== WHITEFOOT COMPILER GATE GREEN =="

format:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets --locked --offline -- -D warnings

# The gate profile (Cargo.toml [profile.gate]) is release codegen with debug
# assertions and overflow checks retained: the suite's owning tests analyze
# real programs, ~16x faster there than at the unoptimized default, so the
# whole suite runs in one gate with no fast/heavy split. A bare
# `cargo test <filter>` during development still uses the default profile
# and keeps incremental builds cheap.
test:
	$(CARGO) test --profile gate --all-targets --locked --offline

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --locked --offline

spec:
	$(CARGO) run --bin whitefoot-spec --locked --offline

completion-test:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) $(COMPLETION_CFLAGS) $(COMPLETION_SOURCES) -o "$(COMPLETION_TMP)/harness"
	"$(COMPLETION_TMP)/harness" "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic \
		$(COMPLETION_DIR)/pure_compute_probe.c -o "$(COMPLETION_TMP)/pure-compute"
	@if $(NM) "$(COMPLETION_TMP)/pure-compute" | grep -q 'wf_completion_'; then \
		echo "pure-compute probe unexpectedly links completion runtime" >&2; \
		exit 1; \
	fi
	@echo "completion pure-compute link boundary: PASS"

completion-sanitize:
	@mkdir -p "$(COMPLETION_TMP)"
	$(CC) -std=c11 -O1 -g -Wall -Wextra -Werror -Wpedantic -pthread \
		-fsanitize=address,undefined $(COMPLETION_SOURCES) \
		-o "$(COMPLETION_TMP)/harness-sanitize"
	ASAN_OPTIONS=detect_leaks=0 "$(COMPLETION_TMP)/harness-sanitize" "$(COMPLETION_TMP)"

.PHONY: check format lint test docs spec completion-test completion-sanitize
