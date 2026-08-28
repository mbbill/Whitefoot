#!/bin/sh
# The Linux half of io-completion-bench. It builds the compiler and the C
# tools from one worktree, generates the tree on a local filesystem, and
# prints the same three-line table the macOS `bench` target prints, with
# io_uring baselines added.
#
# Two hosts run it and neither is privileged over the other. The experiment's
# `linux` target starts a container and leaves every path at its default; a
# real Linux machine — the project's continuous-integration runner — exports
# ROOT, OUT, and CARGO_TARGET_DIR and runs the same bytes with no container in
# the way. Only the paths differ; the protocol below is one protocol.
set -e

FILES=${FILES:-8192}
MAX_KIB=${MAX_KIB:-16}
ROUNDS=${ROUNDS:-7}
WARMUP=${WARMUP:-2}
EXPECTED=${EXPECTED:-"17098009301725298919 00000000000071024640"}

ROOT=${ROOT:-/work}
BUNDLE=$ROOT/research/experiments/io-completion-bench
OUT=${OUT:-/scratch/io-completion-bench}
CLANG=${CLANG:-/usr/bin/clang}

cd "$ROOT/compiler"
cargo build --profile gate --bin whitefootc --locked --offline 2>&1 | tail -1
WFC=${CARGO_TARGET_DIR:-$ROOT/compiler/target}/gate/whitefootc

rm -rf "$OUT"
mkdir -p "$OUT"
cd "$BUNDLE"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror gen.c -o "$OUT/gen"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread baseline.c -o "$OUT/baseline"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror runner.c -o "$OUT/runner"
"$OUT/gen" "$OUT/tree" "$FILES" "$MAX_KIB"

cd "$BUNDLE/programs"
"$WFC" -o "$OUT/wide" many_files_wide.wf
"$WFC" -o "$OUT/wide8" many_files_wide8.wf
"$WFC" -o "$OUT/narrow" many_files_narrow.wf
"$WFC" --no-overlap -o "$OUT/wide_seq" many_files_wide.wf
"$WFC" --no-overlap -o "$OUT/wide8_seq" many_files_wide8.wf
"$WFC" --no-overlap -o "$OUT/narrow_seq" many_files_narrow.wf

for line in "$OUT/baseline $OUT/tree direct $FILES" "$OUT/wide" "$OUT/wide_seq" \
            "$OUT/wide8" "$OUT/wide8_seq" "$OUT/narrow" "$OUT/narrow_seq"; do
    got=$(cd "$OUT/tree" && $line)
    if [ "$got" != "$EXPECTED" ]; then
        echo "MISMATCH: $line published [$got], expected [$EXPECTED]"
        exit 1
    fi
done
echo "all lines publish $EXPECTED"

{
printf 'N.direct\t\t%s/baseline\t%s/tree\tdirect\t%s\n' "$OUT" "$OUT" "$FILES"
for t in 1 2 4 8; do
    printf 'N.pool%s\t\t%s/baseline\t%s/tree\tpool\t%s\t%s\n' "$t" "$OUT" "$OUT" "$FILES" "$t"
done
for d in 2 4 8 16 32; do
    printf 'N.uring%s\t\t%s/baseline\t%s/tree\turing\t%s\t%s\n' "$d" "$OUT" "$OUT" "$FILES" "$d"
done
printf 'S.narrow\t\t%s/narrow_seq\n' "$OUT"
printf 'S.wide\t\t%s/wide_seq\n' "$OUT"
printf 'S.wide8\t\t%s/wide8_seq\n' "$OUT"
printf 'C.narrow.default\t\t%s/narrow\n' "$OUT"
printf 'C.wide.default\t\t%s/wide\n' "$OUT"
printf 'C.wide8.default\t\t%s/wide8\n' "$OUT"
for h in 0 1 2 4; do
    printf 'C.wide.w0.h%s\tWF_WORKERS=0,WF_IO_HELPERS=%s\t%s/wide\n' "$h" "$h" "$OUT"
done
for h in 0 1 2 4; do
    printf 'C.wide8.w0.h%s\tWF_WORKERS=0,WF_IO_HELPERS=%s\t%s/wide8\n' "$h" "$h" "$OUT"
done
for w in 1 2; do
    printf 'C.wide.w%s.hdefault\tWF_WORKERS=%s\t%s/wide\n' "$w" "$w" "$OUT"
done
} > "$OUT/plan.txt"

cd "$OUT/tree"
"$OUT/runner" "$OUT/plan.txt" "$ROUNDS" "$WARMUP" "$EXPECTED"
