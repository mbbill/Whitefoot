ROOT := $(abspath ../../../..)
WHITEFOOTC ?= $(ROOT)/compiler/target/gate/whitefootc
OUT ?= /tmp/whitefoot-x1-container-probes
SOURCES := ownership-swap reverse-drain copy-ring-spans bounded-reserve ring-range append-contract unbounded-reserve linear-ring-publish
NATIVE_SOURCES ?= ownership-swap reverse-drain copy-ring-spans bounded-reserve

.PHONY: observe native

# Explicit research only. Exit codes are observations, not expected verdicts.
observe:
	@mkdir -p "$(OUT)"
	@printf 'source\texit\n' > "$(OUT)/outcomes.tsv"
	@for name in $(SOURCES); do \
	  rm -f "$(OUT)/$$name.ll"; \
	  if "$(WHITEFOOTC)" --emit-llvm -o "$(OUT)/$$name.ll" "$$name.wf" > "$(OUT)/$$name.stdout" 2> "$(OUT)/$$name.stderr"; then code=0; else code=$$?; fi; \
	  printf '%s\t%s\n' "$$name" "$$code" >> "$(OUT)/outcomes.tsv"; \
	done
	@cat "$(OUT)/outcomes.tsv"

native:
	@mkdir -p "$(OUT)"
	@set -eu; for name in $(NATIVE_SOURCES); do \
	  "$(WHITEFOOTC)" -o "$(OUT)/$$name" "$$name.wf"; \
	  "$(OUT)/$$name"; \
	  printf '%s native exit 0\n' "$$name"; \
	done
