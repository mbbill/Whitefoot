#!/bin/sh
# The read-heavy io-completion-bench workload, as one protocol on every host
# that can run it: the container the experiment's `linux-read` target starts,
# a GitHub-hosted Linux runner, and a GitHub-hosted macOS runner. It builds
# the compiler and the C tools from one worktree, generates the large files on
# a local filesystem, and prints the same four tables the macOS `bench-read`
# target prints -- 64 KiB and 4 KiB, each uncached and warm -- with the
# io_uring baselines added where the host has io_uring.
#
# Only paths and the host's own capabilities differ between callers. ROOT,
# OUT, CLANG and CARGO_TARGET_DIR name the paths; `uname -s` decides the
# io_uring lines. Nothing else branches.
#
# One deliberate difference from `make bench-read`. That target refuses to
# print a table whose cache-state label the residency probe did not confirm,
# which is the right rule on a machine whose two populations -- 6 to 20 us
# from the cache, about 134 us from the device -- are known and far apart. A
# hosted runner's storage is not known in advance: its "device" may be a
# host-cached network disk answering faster than the threshold, in which case
# the honest result is a table labelled by what was measured, not no table at
# all. So here the probe always runs, always prints its per-file medians and
# its verdict, and the verdict is printed beside the table instead of stopping
# it. A table this script prints therefore carries the probe's word for its
# label; a table it prints under a refused probe says so on the same line.
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

ROOT=${ROOT:-/work}
BUNDLE=$ROOT/research/experiments/io-completion-bench
OUT=${OUT:-/scratch/io-read-bench}
CLANG=${CLANG:-/usr/bin/clang}
HOST=$(uname -s)

cd "$ROOT/compiler"
cargo build --profile gate --bin whitefootc --locked --offline 2>&1 | tail -1
WFC=${CARGO_TARGET_DIR:-$ROOT/compiler/target}/gate/whitefootc

rm -rf "$OUT"
mkdir -p "$OUT"
cd "$BUNDLE"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror gen.c -o "$OUT/gen"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread read_baseline.c -o "$OUT/read_baseline"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror runner.c -o "$OUT/runner"
"$OUT/gen" "$OUT/tree" "$READ_FILES" "$READ_KIB" fixed

cd "$BUNDLE/programs"
for shape in narrow wide8 narrow_4k wide8_4k; do
    "$WFC" -o "$OUT/$shape" "read_heavy_$shape.wf"
    "$WFC" --no-overlap -o "$OUT/${shape}_seq" "read_heavy_$shape.wf"
done

# Every line publishes the same bytes with the cache policy off and on before
# any of them reports a time.
verify_lines="$OUT/read_baseline $OUT/tree direct $READS_64K 65536:$EXPECTED_READ_64K
$OUT/narrow:$EXPECTED_READ_64K
$OUT/narrow_seq:$EXPECTED_READ_64K
$OUT/wide8:$EXPECTED_READ_64K
$OUT/wide8_seq:$EXPECTED_READ_64K
$OUT/read_baseline $OUT/tree direct $READS_4K 4096:$EXPECTED_READ_4K
$OUT/narrow_4k:$EXPECTED_READ_4K
$OUT/narrow_4k_seq:$EXPECTED_READ_4K
$OUT/wide8_4k:$EXPECTED_READ_4K
$OUT/wide8_4k_seq:$EXPECTED_READ_4K"
if [ "$HOST" = Linux ]; then
    verify_lines="$verify_lines
$OUT/read_baseline $OUT/tree uring $READS_64K 65536 8:$EXPECTED_READ_64K"
fi
for nocache in 0 1; do
    echo "$verify_lines" | while IFS= read -r line; do
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
    done || exit 1
done
echo "every read-heavy line publishes the same bytes with the cache policy off and on"

emit_plan() {
    reads=$1
    window=$2
    suffix=$3
    printf 'N.direct\t\t%s/read_baseline\t%s/tree\tdirect\t%s\t%s\n' "$OUT" "$OUT" "$reads" "$window"
    for t in 1 2 4 8; do
        printf 'N.pool%s\t\t%s/read_baseline\t%s/tree\tpool\t%s\t%s\t%s\n' \
            "$t" "$OUT" "$OUT" "$reads" "$window" "$t"
    done
    if [ "$HOST" = Linux ]; then
        for d in 4 8 32; do
            printf 'N.uring%s\t\t%s/read_baseline\t%s/tree\turing\t%s\t%s\t%s\n' \
                "$d" "$OUT" "$OUT" "$reads" "$window" "$d"
        done
    fi
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
# each file and then, on Linux, drops its pages; a cache policy on a reading
# descriptor cannot evict what a write or a warm table has already made
# resident, so an uncached table has to be handed a tree that is not there.
uncache_tree() {
    rm -rf "$OUT/tree"
    "$OUT/gen" "$OUT/tree" "$READ_FILES" "$READ_KIB" fixed
    sync
}

# Reads the whole tree back in, so a warm table has the state it claims. A
# full sequential pass rather than a rerun of the workload: 32,768
# pseudo-random reads would leave about two per cent of the blocks untouched.
warm_tree() {
    "$OUT/read_baseline" "$OUT/tree" warm 1 65536
}

# The residency check, run immediately before and immediately after every
# table. It prints every file's median either way; `verdict` carries its word
# to the table's own label rather than stopping the run. See the header.
verdict=unknown
probe() {
    if "$OUT/read_baseline" "$OUT/tree" "$1" "$PROBES" 65536 "$THRESHOLD_US" \
        "$TOLERANCE_PERCENT"; then
        verdict=confirmed
    else
        verdict=refused
    fi
}

# What the two probes around one table are each worth, and it differs by host.
#
# Before the table, both hosts are making the same claim and the probe checks
# it: the tree is not resident, so the reads the table is about to time will
# reach the device.
#
# After the table, only Darwin is. F_NOCACHE is a mode of the descriptor, so
# on Darwin a table that asked for uncached reads cannot have populated the
# cache while it ran, and a refusal there means something outside the
# benchmark made the tree resident -- exactly the mistake the second probe
# exists to catch. Linux has no such mode. Its only lever is
# POSIX_FADV_DONTNEED at open, which evicts what is resident at that moment
# and nothing later, so every Linux line starts from a cold tree and then
# warms it as it reads. The after-probe there measures how far that went; it
# is reported for what it is and never read as a verdict on the label.
table() {
    label=$1
    plan=$2
    expected=$3
    nocache=$4
    before=$verdict
    if [ "$nocache" = 1 ]; then
        WF_IO_NOCACHE=1 "$OUT/runner" "$plan" "$ROUNDS" "$WARMUP" "$expected"
    else
        "$OUT/runner" "$plan" "$ROUNDS" "$WARMUP" "$expected"
    fi
    probe "$5"
    echo "table: $label -- probe before the table: $before; probe after it: $verdict"
    if [ "$before" != confirmed ]; then
        echo "table: $label -- the label above is NOT confirmed: the probe refused it before the table ran, so read the per-file medians printed above and treat the label as a claim about what was asked for, not about what was measured"
    fi
}

echo "host: $(uname -srvm)"
cd "$OUT/tree"
# The uncached tables run first: they are the ones the design question turns
# on and the ones a shared host can spoil.
echo "== read-heavy 64 KiB, uncached (WF_IO_NOCACHE=1) =="
uncache_tree
probe probe-uncached
table "64 KiB uncached" "$OUT/plan-64k.txt" "$EXPECTED_READ_64K" 1 probe-uncached
echo "== read-heavy 4 KiB, uncached (WF_IO_NOCACHE=1) =="
uncache_tree
probe probe-uncached
table "4 KiB uncached" "$OUT/plan-4k.txt" "$EXPECTED_READ_4K" 1 probe-uncached
echo "== read-heavy 64 KiB, page cache warm =="
warm_tree
probe probe-warm
table "64 KiB warm" "$OUT/plan-64k.txt" "$EXPECTED_READ_64K" 0 probe-warm
echo "== read-heavy 4 KiB, page cache warm =="
warm_tree
probe probe-warm
table "4 KiB warm" "$OUT/plan-4k.txt" "$EXPECTED_READ_4K" 0 probe-warm
