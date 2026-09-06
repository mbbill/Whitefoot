WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK_ROOT ?= $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-park-on-miss-switch-cost
CC ?= cc
COMMON_C := -O2 -std=c11 -Wall -Wextra -Werror -Wno-deprecated-declarations -pthread

.PHONY: run build

run: build
	$(WORK_ROOT)/switch

build: $(WORK_ROOT)/switch

$(WORK_ROOT)/switch: switch.c
	mkdir -p $(WORK_ROOT)
	$(CC) $(COMMON_C) $< -o $@
