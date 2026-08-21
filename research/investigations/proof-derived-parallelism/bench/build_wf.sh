#!/bin/bash
# Generates and compiles every Whitefoot configuration, twice each: the default
# (sequential) compilation and the --par compilation. Records the ledger and the
# wall-clock compile time of each invocation.
set -uo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
WFC=${WFC:-$ROOT/../../../../compiler/target/release/whitefootc}
cd "$ROOT"
mkdir -p wf bin logs

: > logs/compile_times.txt
: > logs/ledgers.txt

now() { perl -MTime::HiRes=time -e 'printf "%.6f", time'; }

while read -r shape depth words reps; do
  case "$shape" in ''|\#*) continue ;; esac
  tag="${shape}_d${depth}_w${words}"
  src="wf/${tag}.wf"
  ./gen_wf.sh "$shape" "$depth" "$words" "$reps" "$src"

  {
    echo "=== $tag ($src, reps=$reps) ==="
    "$WFC" --par-ledger "$src"
    echo "ledger exit: $?"
  } >> logs/ledgers.txt 2>&1

  t0=$(now); "$WFC" -o "bin/${tag}_seq" "$src"; e1=$?; t1=$(now)
  t2=$(now); "$WFC" --par -o "bin/${tag}_par" "$src"; e2=$?; t3=$(now)
  perl -e "printf \"%-18s seq %.3fs (exit %d)   par %.3fs (exit %d)\n\", '$tag', $t1-$t0, $e1, $t3-$t2, $e2" \
    | tee -a logs/compile_times.txt
done < configs.txt
