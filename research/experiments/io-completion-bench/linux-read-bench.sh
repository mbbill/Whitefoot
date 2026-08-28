#!/bin/sh
# The Linux half of the read-heavy io-completion-bench workload, run inside
# the container the experiment's `linux-read` target starts. It builds the
# compiler and the C tools from the bind-mounted worktree, generates the large
# files on a container-local filesystem, and prints the same four tables the
# macOS `bench-read` target prints, with io_uring baselines added.
set -e

READ_FILES=${READ_FILES:-8}
READ_KIB=${READ_KIB:-65536}
READS_64K=${READS_64K:-32768}
READS_4K=${READS_4K:-32768}
ROUNDS=${ROUNDS:-9}
WARMUP=${WARMUP:-2}
EXPECTED_READ_64K=${EXPECTED_READ_64K:-"09425560703784339892 00000000002147483648"}
EXPECTED_READ_4K=${EXPECTED_READ_4K:-"18028327385673861873 00000000000134217728"}
PROBES=${PROBES:-16}
THRESHOLD_US=${THRESHOLD_US:-40}
TOLERANCE_PERCENT=${TOLERANCE_PERCENT:-10}

BUNDLE=/work/research/experiments/io-completion-bench
OUT=/scratch/io-read-bench

cd /work/compiler
cargo build --profile gate --bin whitefootc --locked --offline 2>&1 | tail -1
WFC=${CARGO_TARGET_DIR:-/target}/gate/whitefootc

rm -rf "$OUT"
mkdir -p "$OUT"
cd "$BUNDLE"
/usr/bin/clang -std=c11 -O2 -Wall -Wextra -Werror gen.c -o "$OUT/gen"
/usr/bin/clang -std=c11 -O2 -Wall -Wextra -Werror -pthread read_baseline.c -o "$OUT/read_baseline"
/usr/bin/clang -std=c11 -O2 -Wall -Wextra -Werror runner.c -o "$OUT/runner"
"$OUT/gen" "$OUT/tree" "$READ_FILES" "$READ_KIB" fixed

cd "$BUNDLE/programs"
for shape in narrow wide8 narrow_4k wide8_4k; do
    "$WFC" -o "$OUT/$shape" "read_heavy_$shape.wf"
    "$WFC" --no-overlap -o "$OUT/${shape}_seq" "read_heavy_$shape.wf"
done

# Every line publishes the same bytes with the cache policy off and on before
# any of them reports a time.
for nocache in 0 1; do
    for line in "$OUT/read_baseline $OUT/tree direct $READS_64K 65536:$EXPECTED_READ_64K" \
                "$OUT/narrow:$EXPECTED_READ_64K" "$OUT/narrow_seq:$EXPECTED_READ_64K" \
                "$OUT/wide8:$EXPECTED_READ_64K" "$OUT/wide8_seq:$EXPECTED_READ_64K" \
                "$OUT/read_baseline $OUT/tree uring $READS_64K 65536 8:$EXPECTED_READ_64K" \
                "$OUT/read_baseline $OUT/tree direct $READS_4K 4096:$EXPECTED_READ_4K" \
                "$OUT/narrow_4k:$EXPECTED_READ_4K" "$OUT/narrow_4k_seq:$EXPECTED_READ_4K" \
                "$OUT/wide8_4k:$EXPECTED_READ_4K" "$OUT/wide8_4k_seq:$EXPECTED_READ_4K"; do
        command=${line%%:*}
        want=${line#*:}
        if [ "$nocache" = 1 ]; then
            got=$(cd "$OUT/tree" && WF_IO_NOCACHE=1 $command)
        else
            got=$(cd "$OUT/tree" && $command)
        fi
        if [ "$got" != "$want" ]; then
            echo "MISMATCH (WF_IO_NOCACHE=$nocache): $command published [$got], expected [$want]"
            exit 1
        fi
    done
done
echo "every read-heavy line publishes the same bytes with the cache policy off and on"

emit_plan() {
    reads=$1
    window=$2
    suffix=$3
    printf 'N.direct\t\t%s/read_baseline\t%s/tree\tdirect\t%s\t%s\n' "$OUT" "$OUT" "$reads" "$window"
    for t in 1 2 4; do
        printf 'N.pool%s\t\t%s/read_baseline\t%s/tree\tpool\t%s\t%s\t%s\n' \
            "$t" "$OUT" "$OUT" "$reads" "$window" "$t"
    done
    for d in 4 8 32; do
        printf 'N.uring%s\t\t%s/read_baseline\t%s/tree\turing\t%s\t%s\t%s\n' \
            "$d" "$OUT" "$OUT" "$reads" "$window" "$d"
    done
    printf 'S.narrow\t\t%s/narrow%s_seq\n' "$OUT" "$suffix"
    printf 'S.wide8\t\t%s/wide8%s_seq\n' "$OUT" "$suffix"
    printf 'C.narrow.default\t\t%s/narrow%s\n' "$OUT" "$suffix"
    printf 'C.wide8.default\t\t%s/wide8%s\n' "$OUT" "$suffix"
    for h in 0 1 4; do
        printf 'C.narrow.h%s\tWF_IO_HELPERS=%s\t%s/narrow%s\n' "$h" "$h" "$OUT" "$suffix"
    done
    for h in 0 1 2 4 8; do
        printf 'C.wide8.h%s\tWF_IO_HELPERS=%s\t%s/wide8%s\n' "$h" "$h" "$OUT" "$suffix"
    done
}

emit_plan "$READS_64K" 65536 "" > "$OUT/plan-64k.txt"
emit_plan "$READS_4K" 4096 _4k > "$OUT/plan-4k.txt"

# Regenerates the tree so that none of it is resident. The generator flushes
# each file and then drops its pages, which is what lets the probe below pass;
# a cache policy on a reading descriptor cannot evict what a write or a warm
# table has already made resident.
uncache_tree() {
    rm -rf "$OUT/tree"
    "$OUT/gen" "$OUT/tree" "$READ_FILES" "$READ_KIB" fixed
    sync
}

# The check that gives each label its meaning, run immediately before and
# immediately after every table, exactly as the macOS bench-read target runs
# it. See the comment on that target for why both ends are needed.
probe() {
    "$OUT/read_baseline" "$OUT/tree" "$1" "$PROBES" 65536 "$THRESHOLD_US" "$TOLERANCE_PERCENT"
}

# Reads the whole tree back in, so a warm table has the state it claims. A
# full sequential pass rather than a rerun of the workload: 32,768
# pseudo-random reads would leave about two per cent of the blocks untouched.
warm_tree() {
    "$OUT/read_baseline" "$OUT/tree" warm 1 65536
}

cd "$OUT/tree"
# The uncached tables run first: they are the ones the design question turns on
# and the ones a shared host can spoil.
echo "== read-heavy 64 KiB, uncached (WF_IO_NOCACHE=1) =="
uncache_tree
probe probe-uncached
WF_IO_NOCACHE=1 "$OUT/runner" "$OUT/plan-64k.txt" "$ROUNDS" "$WARMUP" "$EXPECTED_READ_64K"
probe probe-uncached
echo "== read-heavy 4 KiB, uncached (WF_IO_NOCACHE=1) =="
uncache_tree
probe probe-uncached
WF_IO_NOCACHE=1 "$OUT/runner" "$OUT/plan-4k.txt" "$ROUNDS" "$WARMUP" "$EXPECTED_READ_4K"
probe probe-uncached
echo "== read-heavy 64 KiB, page cache warm =="
warm_tree
probe probe-warm
"$OUT/runner" "$OUT/plan-64k.txt" "$ROUNDS" "$WARMUP" "$EXPECTED_READ_64K"
probe probe-warm
echo "== read-heavy 4 KiB, page cache warm =="
probe probe-warm
"$OUT/runner" "$OUT/plan-4k.txt" "$ROUNDS" "$WARMUP" "$EXPECTED_READ_4K"
probe probe-warm
