#!/bin/sh
# Bounded historical/recovered runtime comparison; called by make pure-compare.
# Retire with the owner-reviewed comparison, retaining its measured artifacts.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT}"
: "${WFC:?set WFC}"
CC=${CC:-/usr/bin/clang}
CXX=${CXX:-/usr/bin/clang++}
mode=${1:-run}
case "$mode" in build|run) ;; *) exit 2;; esac
unset WF_SPLIT_WORK WF_SCHED_REPORT
root=$(git rev-parse --show-toplevel)
mkdir -p "$OUT/pure-compare"
out=$(cd "$OUT/pure-compare" && pwd)
flags='-O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto'
old=9051576f6a4d723b4eb072850f49859853decae7
recovered=d858008f560b25da896af2a17f8b1d07ac49fd6e
if test "$mode" = build; then
    mkdir -p "$out/source"
    git show "$old:compiler/src/backend/par_runtime.c" > "$out/old.c"
    git show "$recovered:research/experiments/compute-runtime/runtime.c" > "$out/research.c"
    cp "$out/old.c" "$out/source/old.original.c"
    cp "$out/research.c" "$out/source/research.original.c"
    (cd "$out" && patch --batch -p0 < "$root/research/experiments/compute-runtime/pure-compare-repairs.patch")
    cp runtime.h runtime_events.h pure-compare-repairs.patch pure-compare.sh pure-compare.awk \
        fir.wf fir_direct.wf fir_host.ll fir_bench.c fir_check.c fir_native.c fir_native.h \
        mandelbrot.wf mandelbrot_command.wf mandelbrot_command.cpp command_runner.c "$out/source/"
    cp "$root/compiler/src/backend/wf_floor.c" "$out/source/"
    cp "$WFC" "$out/whitefootc"
    "$WFC" --par --no-vectorize --emit-llvm fir.wf fir_direct.wf -o "$out/fir.ll"
    sed -e 's/@main(/@wf_research_fir_command_main(/g' \
        -e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$out/fir.ll" > "$out/fir-host.ll"
    cat fir_host.ll >> "$out/fir-host.ll"
    "$CC" $flags -Wno-override-module -c "$out/fir-host.ll" -o "$out/fir.o"
    "$CC" -std=c11 $flags -c fir_native.c -o "$out/native.o"
    "$CC" -std=c11 $flags -DWF_FILTER_NATIVE fir_check.c "$out/native.o" -lm -o "$out/oracle"
    "$out/oracle" > "$out/oracle.txt"
    "$WFC" --par --no-vectorize --emit-llvm mandelbrot.wf mandelbrot_command.wf -o "$out/mandelbrot.ll"
    "$CC" $flags -Wno-override-module -c "$out/mandelbrot.ll" -o "$out/mandelbrot.o"
    "$CXX" -std=c++17 $flags mandelbrot_command.cpp -o "$out/native"
    "$CC" -std=c11 $flags command_runner.c -o "$out/runner"
    # Common read-only benchmark observer, outside the generated WF module.
    # Its exit-time stderr write is charged equally to each command image.
    cat > "$out/source/observer.c" <<'EOF'
#include <stdio.h>
extern unsigned wf_compute_worker_count(void);
__attribute__((destructor)) static void report_width(void) {
    fprintf(stderr, "actual_pool_lanes=%u\n", wf_compute_worker_count());
}
EOF
    "$CC" -std=c11 $flags -c "$out/source/observer.c" -o "$out/observer.o"
    for variant in old research research-off; do
        source=research; stats=1
        test "$variant" != old || source=old
        test "$variant" != research-off || stats=0
        "$CC" -std=c11 $flags -I"$out/source" -DWF_COMPUTE_STATS=$stats \
            -c "$out/$source.c" -o "$out/$variant.o"
        "$CC" -std=c11 $flags -DWF_COMPUTE_STATS=$stats -DWF_COMPUTE_CONTROL \
            "-DFIR_RUNTIME=\"$variant\"" fir_bench.c "$out/$variant.o" \
            "$out/source/wf_floor.c" "$out/fir.o" "$out/native.o" -lm -o "$out/fir-$variant"
        "$CC" -std=c11 $flags "$out/$variant.o" "$out/source/wf_floor.c" \
            "$out/mandelbrot.o" "$out/observer.o" -lm -o "$out/mandelbrot-$variant"
    done
    cp "$out/fir-research" "$out/fir-replica"
    cp "$out/mandelbrot-research" "$out/mandelbrot-replica"
    cmp "$out/fir-research" "$out/fir-replica"
    cmp "$out/mandelbrot-research" "$out/mandelbrot-replica"
    {
        git rev-parse HEAD
        printf 'historical=%s\nrecovered=%s\nflags=%s\n' "$old" "$recovered" "$flags"
        uname -a
        "$CC" --version
        if test "$(uname -s)" = Darwin; then
            sysctl hw.model hw.ncpu machdep.cpu.brand_string || printf 'sysctl metadata unavailable\n'
        else lscpu; fi
    } > "$out/metadata.txt"
    shasum -a 256 "$out/source/"* "$out/"*.c "$out/"*.o "$out/"*.ll \
        "$out/whitefootc" "$out/native" "$out/runner" > "$out/manifest.sha256"
    for variant in old research research-off replica; do
        shasum -a 256 "$out/fir-$variant" "$out/mandelbrot-$variant" >> "$out/manifest.sha256"
    done
    exit 0
fi
test -x "$out/fir-old"
cpus=$(getconf _NPROCESSORS_ONLN)
widths=${PURE_WIDTHS:-"1 2 4"}
passes=${PURE_PASSES:-5}
case "$passes" in ''|*[!0-9]*) exit 2;; esac
test "$passes" -ge 1 && test "$passes" -le 20
for w in $widths; do
    case "$w" in 1|2|4|8|16|32|64) ;; *) exit 2;; esac
    test "$w" -le "$cpus" || { echo "width exceeds host CPUs" >&2; exit 2; }
done
mkdir "$out/raw"
printf 'widths=%s\npasses=%s\n' "$widths" "$passes" > "$out/plan.txt"
printf 'suite\tcase\tworkers\tpass\tvariant\twall_ns\tuser_ns\tsystem_ns\trss_bytes\tvoluntary\tinvoluntary\tstatus\n' > "$out/process.tsv"
printf 'shape\tcount\tlimit\trepetitions\tseed\texpected\n' > "$out/oracles.tsv"
# The native oracle checks known orbits and a separately evaluated recurrence.
for shape in 0 1 2 3 4 5 6; do
    for count in 4096 65536; do
        repetitions=16
        expected=$("$out/native" oracle "$shape" "$count" 256 "$repetitions" 92821)
        printf '%s\t%s\t256\t%s\t92821\t%s\n' "$shape" "$count" "$repetitions" "$expected" >> "$out/oracles.tsv"
    done
done
pass=1
while test "$pass" -le "$passes"; do
    # Rotation plus reversal distributes process-order effects across variants.
    case $((pass % 4)) in
        1) variants='old research replica research-off';;
        2) variants='research-off replica research old';;
        3) variants='research replica research-off old';;
        0) variants='old research-off replica research';;
    esac
    for w in $widths; do
        for n in 4096 65536; do
            for tile in 16 64 1024; do
                calls=512; test "$n" != 65536 || calls=64
                cell=$n-$tile
                for variant in $variants; do
                    log="$out/raw/fir-$cell-$w-$pass-$variant.txt"
                    WF_WORKERS=$w "$out/runner" "$out/fir-$variant" wf 16 "$n" "$tile" "$calls" 92821 "$pass" > "$log"
                    grep -Eq "^# actual_lanes=$w( |$)" "$log"
                    printf 'fir\t%s\t%s\t%s\t%s\t' "$cell" "$w" "$pass" "$variant" >> "$out/process.tsv"
                    tail -n 1 "$log" >> "$out/process.tsv"
                done
            done
        done
        tail -n +2 "$out/oracles.tsv" | while read -r shape count limit repetitions seed expected; do
            cell=$shape-$count
            for variant in $variants; do
                log="$out/raw/mandelbrot-$cell-$w-$pass-$variant.txt"
                WF_WORKERS=$w "$out/runner" "$out/mandelbrot-$variant" "$shape" "$count" "$limit" "$repetitions" "$seed" "$expected" > "$log" 2> "$log.width"
                test "$(wc -l < "$log" | tr -d ' ')" = 1
                active=$w
                if test "$w" = 1 || test "$count" = 4096; then active=0; fi
                test "$(cat "$log.width")" = "actual_pool_lanes=$active"
                printf 'mandelbrot\t%s\t%s\t%s\t%s\t' "$cell" "$w" "$pass" "$variant" >> "$out/process.tsv"
                cat "$log" >> "$out/process.tsv"
            done
        done
    done
    printf 'Completed pure-runtime paired pass %s/%s\n' "$pass" "$passes"
    pass=$((pass + 1))
done
awk -v widths="$widths" -v expected_passes="$passes" -f pure-compare.awk "$out/process.tsv" > "$out/summary.tsv"
cat "$out/summary.tsv"
