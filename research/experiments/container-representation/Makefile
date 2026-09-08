ROOT := $(abspath ../../..)
CARGO ?= cargo
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)

.PHONY: check measure compiler

compiler:
	$(CARGO) build --manifest-path $(ROOT)/compiler/Cargo.toml --profile gate --bin whitefootc --locked --offline

check: compiler
	$(MAKE) -C authority check
	$(MAKE) -C lifecycle check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C dense check WHITEFOOTC="$(WHITEFOOTC)"

measure: check
	$(MAKE) -C dense measure WHITEFOOTC="$(WHITEFOOTC)"
