#!/usr/bin/env bash
# Disposable manual-only records projected-capture diagnostic. The caller must
# run this complete build-and-measure script under .github/run-check.pl.
set -euo pipefail
if [[ ${WF_RUN_APPROVED:-} != 1 || $# != 4 ]]; then
  echo 'usage: WF_RUN_APPROVED=1 run-element-base.sh CHECKOUT ARTIFACT BUILT RESULTS' >&2
  exit 2
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || {
  echo 'element-base diagnostic requires Linux x86-64' >&2
  exit 2
}

# Refuse an unguarded invocation: all compilation, oracle execution and timing
# belong to the one repository-wide owner established by the manual workflow.
guard_owner=${WHITEFOOT_CHECK_OWNER:-}
guard_lock=${WHITEFOOT_CHECK_LOCK_DIR:-}
[[ $guard_owner =~ ^[0-9]+$ && -n $guard_lock && -f $guard_lock/pid ]] || {
  echo 'element-base diagnostic must run under .github/run-check.pl' >&2
  exit 2
}
[[ $(<"$guard_lock/pid") == "$guard_owner" ]] && kill -0 "$guard_owner" || {
  echo 'element-base diagnostic guard owner is not live' >&2
  exit 2
}

checkout=$(realpath "$1")
artifact=$(realpath "$2")
built=$(realpath -m "$3")
results=$(realpath -m "$4")
here=$(cd -- "$(dirname -- "$0")" && pwd)
compare=$checkout/tests/performance/compare.sh
source_ll=$artifact/performance-candidate/records.ll
candidate=$artifact/performance-candidate
baseline=$artifact/performance-baseline

[[ ! -e $built ]] || { echo 'built path must be fresh' >&2; exit 2; }
[[ ! -e $results ]] || { echo 'results path must be fresh' >&2; exit 2; }
mkdir -p "$built" "$results" "$results/staged"

inconclusive_now() {
  local message=$1
  printf 'INCONCLUSIVE: %s\n' "$message" | tee "$results/INCONCLUSIVE.txt" >&2
  exit 2
}

check_sha() {
  local expected=$1 file=$2
  [[ -f $file ]] || inconclusive_now "missing qualified input: $file"
  [[ $(sha256sum "$file" | awk '{print $1}') == "$expected" ]] ||
    inconclusive_now "hash mismatch: $file"
}

# Pinned compute artifact from run 35534896287. Qualify every original image
# before restoring executable permission, together with the exact generated
# module and the runner/oracle objects reused by the diagnostic link.
check_sha 6d7afa2b0722edd5a48a8c1c61b44607e71e0aa17c4d58c5208b57b9a02c77d6 \
  "$artifact/performance-results/identity.txt"
check_sha 8b60bc3da92f0914de1c90dbdeac5aaa99427136ed6093626894154a0ca7ac63 \
  "$source_ll"
check_sha 414de402069c32cca26b55ba05a0ea2c70f5c1f6bdb4b1dbbf632ea36a4b2707 \
  "$candidate/runner.o"
check_sha 3f38a448805d6c9376b19c7cab33328068cea053f6e212834259d725ce1c42ef \
  "$candidate/records_oracle.o"

check_sha b4d465213da606f2ea9e10bbab270f1f2dae0ffc9257306ff3e90d32be40c246 \
  "$baseline/mandelbrot"
check_sha 904028379508662fc988cb256896f90dbb4f24a82746c771b0e1cc6024412ce5 \
  "$baseline/records"
check_sha 3ba99b5d88818f3b6c0bd648f73f3facded50d2aa7b4cdff2a7d29efcb404410 \
  "$baseline/fir"
check_sha 5cd90a0e80699800322394da7515db7bcbfbadc78be982470ac8ea49e3d99136 \
  "$baseline/quadrature"
check_sha c63c2656c9a49d5b03c4d79a0d1f7b0abd6c01e4f2513d36abf6234be3bae67b \
  "$baseline/stencil"
check_sha d71fe5141455b44fa2e8df9e51a9a27bad0f0164abdba4b410a54a91d7f46e3b \
  "$candidate/mandelbrot"
check_sha 7d0f38f2f61e109cd2260c8ba335beaff4eae44a47a0d7ec8e1b06bc954fbf68 \
  "$candidate/records"
check_sha 234a5805ffd6e4ecab517b140dff1a3856f9a9c96d3e97aa8de9676272e650cf \
  "$candidate/fir"
check_sha 5cd90a0e80699800322394da7515db7bcbfbadc78be982470ac8ea49e3d99136 \
  "$candidate/quadrature"
check_sha 89b9e80dea97bee12007382940f440845e2087b037733c3df0997d672d8b7912 \
  "$candidate/stencil"

# The comparison implementation and its runner source are also pinned. Their
# object counterpart above is what enters the linked element-base image.
check_sha 9e1558b6a7178c6a5b94552baadb528885bd361b4901cdf54a6edd6480c92e1b \
  "$checkout/tests/performance/compare.sh"
check_sha 4407035ee63942b7c1cb798a6ba7ee5e0c9ee9f5a41445fb840543df038965d4 \
  "$checkout/tests/performance/reduce.awk"
check_sha eb369ca7890164839b2fec5543a30d17395968d54dab7bdd341031fa9b9b4358 \
  "$checkout/tests/performance/verdict.awk"
check_sha e1bcfc3b33159af702242438b121d09a93bccc3da75df656a570cc72efb1d813 \
  "$checkout/tests/performance/runner.c"

for arm in "$baseline" "$candidate"; do
  for kernel in mandelbrot records fir quadrature stencil; do
    chmod +x "$arm/$kernel"
  done
done

# Apply the complete registered transformation to the original module. The
# output hash proves both exact context and exact result; no allocator symbol
# is rewritten, and ordinary malloc/free remain the module's allocation ABI.
cp "$source_ll" "$built/records.ll"
patch --batch --fuzz=0 -p1 -d "$built" < "$here/element-base.patch"
check_sha 73980c11fd59fb5b09bca2387cdceba67168e5f17a0e1946a769cc1291028e41 \
  "$built/records.ll"
clang -O2 -Wno-override-module -x ir -c "$built/records.ll" -o "$built/records.o"
nm -u "$built/records.o" | grep -Eq '[[:space:]]U malloc$'
nm -u "$built/records.o" | grep -Eq '[[:space:]]U free$'
if nm -u "$built/records.o" | grep -q 'wf_records_result_'; then
  inconclusive_now 'element-base object unexpectedly names the custom allocator'
fi

native_objects=()
while IFS= read -r object; do native_objects+=("$object"); done \
  < <(find "$candidate/native" -type f -name '*.o' | sort)
(( ${#native_objects[@]} != 0 )) || inconclusive_now 'candidate native-object set is empty'
sha256sum "${native_objects[@]}" > "$built/native-objects.sha256"
objects=("$built/records.o" "$candidate/records_oracle.o" \
  "$candidate/runner.o" "${native_objects[@]}")
clang -O2 "${objects[@]}" -pthread -lm -o "$built/records"
sha256sum "$built/records.ll" "$built/records.o" "$built/records" \
  > "$built/element-base.sha256"

stage_arm() {
  local name=$1 records=$2 controls=$3
  local directory=$results/staged/$name
  mkdir -p "$directory"
  ln "$records" "$directory/records"
  for kernel in mandelbrot fir quadrature stencil; do
    ln "$controls/$kernel" "$directory/$kernel"
  done
}
stage_arm exact-baseline "$baseline/records" "$baseline"
stage_arm exact-candidate "$candidate/records" "$candidate"
stage_arm element-base "$built/records" "$candidate"

(( $(nproc) >= 4 )) || inconclusive_now 'at least four available CPUs are required'

run_comparison() {
  local left=$1 right=$2 destination=$3 mode=${4:-} status=0
  if [[ $mode == slow ]]; then
    bash "$compare" "$left" "$right" "$destination" slow || status=$?
  else
    bash "$compare" "$left" "$right" "$destination" || status=$?
  fi
  if [[ $status != 0 && $status != 1 ]]; then
    inconclusive_now "comparison failed before a complete verdict: $destination (exit $status)"
  fi
  [[ -s $destination/verdict.txt && -s $destination/paired.tsv ]] ||
    inconclusive_now "comparison did not produce complete result tables: $destination"
  printf '%s\n' "$status" > "$destination/comparison-exit-status.txt"
}

# Retain all five ordinary comparison tables before interpreting any control.
# Exit 1 is a measured verdict, not a script failure.
run_comparison "$results/staged/exact-candidate" "$results/staged/exact-candidate" \
  "$results/exact-candidate-null"
run_comparison "$results/staged/exact-baseline" "$results/staged/exact-candidate" \
  "$results/baseline-candidate-reproduction"
run_comparison "$results/staged/exact-candidate" "$results/staged/exact-candidate" \
  "$results/slow-control" slow
run_comparison "$results/staged/exact-baseline" "$results/staged/element-base" \
  "$results/baseline-element-base"
run_comparison "$results/staged/exact-candidate" "$results/staged/element-base" \
  "$results/candidate-element-base"

declare -a invalid=()
has_signal() {
  local verdict=$1 pattern=$2
  grep -Eq "^($pattern)[[:space:]].* (FAIL|suspect)$" "$verdict"
}

null_verdict=$results/exact-candidate-null/verdict.txt
if has_signal "$null_verdict" 'mandelbrot|records|fir|quadrature|stencil' ||
   ! grep -q '^VERDICT: PASS -- 0 kernel(s) adverse at two widths$' "$null_verdict"; then
  invalid+=('exact-candidate identical-image null was not clean')
fi

reproduction=$results/baseline-candidate-reproduction/verdict.txt
grep -q '^records[[:space:]].*W=2 .* FAIL$' "$reproduction" ||
  invalid+=('original records W2 failure did not reproduce')
grep -q '^records[[:space:]].*W=4 .* FAIL$' "$reproduction" ||
  invalid+=('original records W4 failure did not reproduce')
if has_signal "$reproduction" 'mandelbrot|fir|quadrature|stencil'; then
  invalid+=('an unchanged kernel signaled in baseline/candidate reproduction')
fi

slow=$results/slow-control/verdict.txt
if ! grep -q '^VERDICT: FAIL -- 5 kernel(s) adverse at two widths$' "$slow"; then
  invalid+=('known slowdown control did not detect all five kernels')
fi

for comparison in baseline-element-base candidate-element-base; do
  verdict=$results/$comparison/verdict.txt
  if has_signal "$verdict" 'mandelbrot|fir|quadrature|stencil'; then
    invalid+=("an unchanged kernel signaled in $comparison")
  fi
done

if (( ${#invalid[@]} != 0 )); then
  {
    printf 'INCONCLUSIVE: one or more preregistered conditions failed\n'
    printf -- '- %s\n' "${invalid[@]}"
  } | tee "$results/INCONCLUSIVE.txt" >&2
  exit 2
fi

printf '%s\n' \
  'strict controls PASS; interpret the two retained records tables only by the preregistered README criterion' \
  > "$results/VALIDATION.txt"
cat "$results/VALIDATION.txt"
