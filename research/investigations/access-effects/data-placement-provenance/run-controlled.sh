#!/usr/bin/env bash
set -euo pipefail
if [[ ${WF_RUN_APPROVED:-} != 1 || $# != 4 ]]; then
  echo 'usage: WF_RUN_APPROVED=1 run-controlled.sh CHECKOUT ARTIFACT BUILT RESULTS' >&2
  exit 2
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || exit 2
case "$(lscpu | sed -n 's/^Model name:[[:space:]]*//p')" in
  'AMD EPYC 7763 64-Core Processor'|'AMD EPYC 9V45 96-Core Processor') ;;
  *) echo 'runner model is outside the preregistered set' >&2; exit 2 ;;
esac
checkout=$(realpath "$1")
artifact=$(realpath "$2")
built=$(realpath "$3")
results=$(realpath -m "$4")
compare=$checkout/tests/performance/compare.sh
test "$(sha256sum "$checkout/tests/performance/compare.sh" | awk '{print $1}')" = \
  9e1558b6a7178c6a5b94552baadb528885bd361b4901cdf54a6edd6480c92e1b
test "$(sha256sum "$checkout/tests/performance/reduce.awk" | awk '{print $1}')" = \
  4407035ee63942b7c1cb798a6ba7ee5e0c9ee9f5a41445fb840543df038965d4
test "$(sha256sum "$checkout/tests/performance/verdict.awk" | awk '{print $1}')" = \
  eb369ca7890164839b2fec5543a30d17395968d54dab7bdd341031fa9b9b4358
test "$(sha256sum "$checkout/tests/performance/runner.c" | awk '{print $1}')" = \
  e1bcfc3b33159af702242438b121d09a93bccc3da75df656a570cc72efb1d813
[[ ! -e $results ]] || { echo 'results path must be fresh' >&2; exit 2; }
mkdir -p "$results" "$results/staged"

stage_arm() {
  local name=$1
  local records=$2
  local controls=${3:-$artifact/performance-candidate}
  local directory=$results/staged/$name
  mkdir -p "$directory"
  ln "$records" "$directory/records"
  for kernel in mandelbrot fir quadrature stencil; do
    ln "$controls/$kernel" "$directory/$kernel"
  done
}
for residue in 16 24 48 56; do
  stage_arm r$residue "$built/records-r$residue"
  ln "$built/records-controlled" "$results/staged/r$residue/records-controlled"
done
stage_arm exact-baseline "$artifact/performance-baseline/records" \
  "$artifact/performance-baseline"
stage_arm exact-candidate "$artifact/performance-candidate/records"

# Full existing correctness oracle before measurement, including the controlled
# residue assertion summarized by the process-shutdown hook.
for residue in 16 24 48 56; do
  for width in 1 2 4; do
    log=$results/verify-r$residue-w$width.log
    WF_WORKERS=$width timeout 60s "$results/staged/r$residue/records" verify >"$log" 2>&1
    grep -q 'records oracle PASS:' "$log"
    test "$(grep -c "^WF_RESULT_PAYLOAD residue=$residue " "$log")" -eq 1
    grep -q "^WF_RESULT_PAYLOAD residue=$residue allocations=.* releases=" "$log"
  done
done

run_allow_verdict() {
  local left=$1 right=$2 destination=$3 mode=${4:-} status=0
  if [[ $mode == slow ]]; then
    bash "$compare" "$left" "$right" "$destination" slow || status=$?
  else
    bash "$compare" "$left" "$right" "$destination" || status=$?
  fi
  [[ $status == 0 || $status == 1 ]] || exit "$status"
  printf '%s\n' "$status" > "$destination/comparison-exit-status.txt"
}

assert_no_control_signal() {
  local verdict=$1
  if grep -Eq '^(mandelbrot|fir|quadrature|stencil)[[:space:]].* (FAIL|suspect)$' "$verdict"; then
    echo "unchanged control kernel has a FAIL or suspect: $verdict" >&2
    exit 2
  fi
}

assert_clean_null() {
  local verdict=$1
  if grep -Eq '^(mandelbrot|records|fir|quadrature|stencil)[[:space:]].* (FAIL|suspect)$' "$verdict"; then
    echo "null has a FAIL or suspect: $verdict" >&2
    exit 2
  fi
  grep -q '^VERDICT: PASS -- 0 kernel(s) adverse at two widths$' "$verdict"
}

# Same-host gate: an exact-candidate null must pass, then the exact unmodified
# pair must reproduce the original two-width records failure before controlled
# observations have any interpretation.
bash "$compare" "$results/staged/exact-candidate" "$results/staged/exact-candidate" \
  "$results/exact-null"
assert_clean_null "$results/exact-null/verdict.txt"
run_allow_verdict "$results/staged/exact-baseline" "$results/staged/exact-candidate" \
  "$results/reproduction"
grep -q '^records[[:space:]].*W=2 .* FAIL$' "$results/reproduction/verdict.txt"
grep -q '^records[[:space:]].*W=4 .* FAIL$' "$results/reproduction/verdict.txt"
test "$(grep -Ec '^(mandelbrot|fir|quadrature|stencil)[[:space:]].* FAIL$' \
  "$results/reproduction/verdict.txt")" -eq 0
assert_no_control_signal "$results/reproduction/verdict.txt"
grep -E '^records[[:space:]].*W=(2|4) ' "$results/reproduction/verdict.txt" \
  > "$results/reproduction/REPRODUCED-W2-W4.txt"
test "$(wc -l < "$results/reproduction/REPRODUCED-W2-W4.txt")" -eq 2

# One identical-image null and one existing sensitivity control qualify the
# controlled ELF before either predeclared residue contrast is read.
bash "$compare" "$results/staged/r16" "$results/staged/r16" "$results/null"
assert_clean_null "$results/null/verdict.txt"
run_allow_verdict "$results/staged/r16" "$results/staged/r16" "$results/slow" slow
grep -q '^VERDICT: FAIL -- 5 kernel(s) adverse at two widths$' "$results/slow/verdict.txt"
run_allow_verdict "$results/staged/r16" "$results/staged/r24" "$results/r16-r24"
run_allow_verdict "$results/staged/r48" "$results/staged/r56" "$results/r48-r56"
assert_no_control_signal "$results/r16-r24/verdict.txt"
assert_no_control_signal "$results/r48-r56/verdict.txt"

# Each controlled process emits one destructor summary after all clocks stop;
# every allocation already checked its selected residue and counts must balance.
for pair in null slow r16-r24 r48-r56; do
  case $pair in
    null|slow) allowed='16' ;;
    r16-r24) allowed='16|24' ;;
    r48-r56) allowed='48|56' ;;
  esac
  while IFS= read -r log; do
    expected=6
    if [[ $pair == slow && $log == *-candidate-* ]]; then expected=12; fi
    test "$(grep -Ec "^WF_RESULT_PAYLOAD residue=($allowed) allocations=$expected releases=$expected module_data=0x[0-9a-f]+$" "$log")" -eq 1
    awk '/^WF_RESULT_PAYLOAD / {
      split($3, a, "="); split($4, r, "="); if (a[2] != r[2]) exit 1
    }' "$log"
  done < <(find "$results/$pair/logs" -type f -name 'records-*.log' | sort)
done
printf 'controlled comparisons complete; interpret only by README preregistration\n'
