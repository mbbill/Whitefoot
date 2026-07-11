ROOT=../../..
PY=python3
CC=/usr/bin/clang

build:
	$(PY) $(ROOT)/prototype/democ/democ.py chunk_wc.xl
	$(CC) -O3 chunk_wc.ll driver.c -lpthread -o xchunkwc_facts
	cp xchunkwc_facts xchunkwc
	$(PY) $(ROOT)/prototype/democ/democ.py chunk_wc.xl --no-facts
	$(CC) -O3 chunk_wc.ll driver.c -lpthread -o xchunkwc_nofacts
	$(PY) $(ROOT)/prototype/democ/democ.py chunk_wc.xl
	rustc -C opt-level=3 rust_chunk_wc.rs -o rust_chunk_wc
	$(CC) -O3 c_control.c driver.c -lpthread -o c_chunk_wc

check: build
	$(PY) check_algebra.py
	$(PY) verify.py

bench: build
	$(PY) benchmark.py ../wc/big.txt 5

.PHONY: build check bench
