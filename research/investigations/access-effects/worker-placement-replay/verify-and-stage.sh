#!/usr/bin/env bash
# Correctness-only staging. This never invokes measure mode.
set -euo pipefail
if [[ $# != 2 ]]; then
    echo 'usage: verify-and-stage.sh BUILT_OUTPUT LOG_OUTPUT' >&2
    exit 2
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || exit 2
built=$(realpath "$1")
logs=$(realpath -m "$2")
mkdir -p "$logs"

for pad in 0 16 32 48; do
    image="$built/images/records-pad$pad"
    for width in 1 2 4; do
        env -u WF_SPLIT_WORK WF_WORKERS="$width" timeout 60s "$image" verify \
            > "$logs/verify-records-pad$pad-w$width.log" 2>&1
    done
done
printf 'VERIFY PASSED; no timings were run.\n'
