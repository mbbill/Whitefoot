ROOT := $(abspath ../../..)
CARGO ?= cargo
EXE := $(if $(filter Windows_NT,$(OS)),.exe,)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc$(EXE)

.PHONY: check measure compiler x1-observe slab-deque-check slab-deque-measure map-native-check

x1-observe:
	$(MAKE) -C x1 observe WHITEFOOTC="$(WHITEFOOTC)"

slab-deque-check: compiler
	$(MAKE) -C slab-library check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C deque-library check WHITEFOOTC="$(WHITEFOOTC)"

slab-deque-measure: slab-deque-check
	$(MAKE) -C slab-library measure WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C deque-library measure WHITEFOOTC="$(WHITEFOOTC)"

map-native-check:
	$(MAKE) -C map-library native-check

compiler:
	$(CARGO) build --manifest-path $(ROOT)/compiler/Cargo.toml --profile gate --bin whitefootc --locked --offline

check: compiler
	$(MAKE) -C authority check
	$(MAKE) -C foundation check
	$(MAKE) -C lifecycle check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C dense check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C costs check
	$(MAKE) -C families check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C vector-library check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C slab-library check WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C deque-library check WHITEFOOTC="$(WHITEFOOTC)"

measure: check
	$(MAKE) -C foundation measure
	$(MAKE) -C dense measure WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C costs measure
	$(MAKE) -C families measure WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C vector-library measure WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C slab-library measure WHITEFOOTC="$(WHITEFOOTC)"
	$(MAKE) -C deque-library measure WHITEFOOTC="$(WHITEFOOTC)"
