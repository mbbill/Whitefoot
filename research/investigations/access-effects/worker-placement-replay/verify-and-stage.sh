#!/usr/bin/env bash
# Correctness-only staging. This never invokes measure mode.
set -euo pipefail
if [[ $# != 3 ]]; then
    echo 'usage: verify-and-stage.sh AC34_ARTIFACT_ROOT BUILT_OUTPUT STAGED_OUTPUT' >&2
    exit 2
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || exit 2
artifact=$(realpath "$1")
built=$(realpath "$2")
staged=$(realpath -m "$3")
candidate="$artifact/performance-candidate"
mkdir -p "$staged/verify-logs" "$staged/pad0" "$staged/pad16" "$staged/pad32" "$staged/pad48"

for pad in 0 16 32 48; do
    image="$built/images/records-pad$pad"
    for width in 1 2 4; do
        env -u WF_SPLIT_WORK WF_WORKERS="$width" timeout 60s "$image" verify \
            > "$staged/verify-logs/verify-records-pad$pad-w$width.log" 2>&1
    done
    ln "$image" "$staged/pad$pad/records"
done

# Every non-records image is the same downloaded candidate inode in every arm.
# They exercise the existing five-kernel reducer without adding a second
# workload or changing any control program.
for kernel in mandelbrot fir quadrature stencil; do
    source="$candidate/$kernel"
    [[ -x $source ]] || { echo "missing control image: $source" >&2; exit 2; }
    for pad in 0 16 32 48; do
        ln "$source" "$staged/pad$pad/$kernel"
    done
done
{
    for kernel in mandelbrot records fir quadrature stencil; do
        for pad in 0 16 32 48; do
            stat -c '%d:%i %s %n' "$staged/pad$pad/$kernel"
        done
    done
    sha256sum "$staged"/pad*/{mandelbrot,records,fir,quadrature,stencil}
} > "$staged/STAGING.txt"
printf 'VERIFY PASSED; no timings were run.\n'
