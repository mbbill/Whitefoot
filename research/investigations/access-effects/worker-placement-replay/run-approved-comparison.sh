#!/usr/bin/env bash
# The one registered timing experiment. Construction and correctness have
# already passed; this script first qualifies an identical pad48 null and then
# runs only the preselected pad48-versus-pad16 comparison.
set -euo pipefail
if [[ ${WF_RUN_APPROVED:-} != 1 || $# != 4 ]]; then
    echo 'usage: WF_RUN_APPROVED=1 run-approved-comparison.sh CHECKOUT STAGED PRIMARY_RESULTS NULL_RESULTS' >&2
    exit 2
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || exit 2
checkout=$(realpath "$1")
staged=$(realpath "$2")
primary_results=$(realpath -m "$3")
null_results=$(realpath -m "$4")
compare="$checkout/tests/performance/compare.sh"
[[ -f $compare ]] || exit 2
[[ ! -e $primary_results && ! -e $null_results ]] || {
    echo 'timing result paths must be fresh' >&2
    exit 2
}

# The null must pass. With `set -e`, any inconclusive null stops here and the
# preselected primary is never executed.
mkdir -p "$staged/null-a" "$staged/null-b"
for kernel in mandelbrot records fir quadrature stencil; do
    source="$staged/pad48/$kernel"
    ln "$source" "$staged/null-a/$kernel"
    ln "$source" "$staged/null-b/$kernel"
done
bash "$compare" "$staged/null-a" "$staged/null-b" "$null_results"

# This is the only timed placement pair. A failing regression verdict is the
# expected form of positive evidence, so preserve its status and full output
# rather than turning the research job into an infrastructure failure.
status=0
bash "$compare" "$staged/pad48" "$staged/pad16" "$primary_results" || status=$?
printf '%s\n' "$status" > "$primary_results/comparison-exit-status.txt"
if ((status == 1)); then
    grep -Eq '^VERDICT: FAIL -- [1-9][0-9]* kernel\(s\) adverse at two widths$' \
        "$primary_results/verdict.txt" || {
        echo 'primary exited 1 without a complete adverse verdict' >&2
        exit 2
    }
elif ((status != 0)); then
    echo "primary comparison failed before a verdict: status=$status" >&2
    exit "$status"
fi
printf 'primary comparison exit status=%s\n' "$status"
