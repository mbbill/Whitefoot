ROOT := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot
WORK_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-proof-use-cost
WFC ?= $(ROOT)/compiler/target/gate/whitefootc
SIZES ?= 16,32,64,128,256
REPETITIONS ?= 3

.PHONY: build compiler check bench compare

build:
	mkdir -p "$(WORK_ROOT)"
	rustfmt --edition=2024 --check runner.rs
	rustc --edition=2024 --forbid unsafe_code --deny warnings -C opt-level=2 runner.rs -o "$(WORK_ROOT)/runner"

compiler:
	cargo build --manifest-path "$(ROOT)/compiler/Cargo.toml" --profile gate --bin whitefootc --locked --offline

check: build compiler
	ulimit -c 0; "$(WORK_ROOT)/runner" "$(WFC)" "$(WORK_ROOT)/check" check

bench: build compiler
	ulimit -c 0; "$(WORK_ROOT)/runner" "$(WFC)" "$(WORK_ROOT)/bench" bench "$(SIZES)" "$(REPETITIONS)"

# BASELINE and CANDIDATE name already built compilers; this target cannot
# silently rebuild one side from the current checkout.
compare: build
	@test -n "$(BASELINE)" -a -n "$(CANDIDATE)"
	ulimit -c 0; "$(WORK_ROOT)/runner" "$(BASELINE)" "$(WORK_ROOT)/compare" compare "$(CANDIDATE)" "$(SIZES)" 5 $(REAL_SOURCES)
