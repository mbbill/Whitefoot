#!/bin/zsh
# One command for the whole timing pass: measure, byte-compare, rebuild tables.
# usage: ./rerun.zsh [ROUNDS]        (default 9)
#
# The Whitefoot and Rust binaries are NOT rebuilt here; ./build_wf.sh and
# `cargo build --release --manifest-path rust/Cargo.toml` do that, and nothing
# about the programs changes between passes. Only logs/results.tsv, out/, and
# logs/t_*.md are replaced, so a clean pass replaces the timing tables and
# leaves the ledger, compile-time, and bit-identity evidence intact.
set -u
ROOT=${0:A:h}
cd "$ROOT"
ROUNDS=${1:-9}

stamp=$(date +%Y%m%d-%H%M%S)
if [[ -f logs/results.tsv ]]; then
  cp logs/results.tsv "logs/results-$stamp.tsv"
  print "previous pass archived as logs/results-$stamp.tsv"
fi

print "system state before the pass:"
sysctl -n vm.loadavg
ps -Ao pcpu,comm -r | head -5

rm -f out/*.txt
./run_bench.zsh "$ROUNDS"
./compare_outputs.zsh | tee logs/byte_comparison.txt
./make_tables.zsh

print ""
print "system state after the pass:"
sysctl -n vm.loadavg
cat logs/t_spread.md
