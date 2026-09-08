#!/bin/sh
set -eu
cd "$(dirname "$0")"
mode=${1:?mode required}
case "$mode" in check|calibrate|layout|layout-check) ;; *) echo 'unknown scheduler mode' >&2; exit 1;; esac
layout=0
case "$mode" in layout|layout-check) layout=1;; esac
: "${OUT:?build output required}"
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
case "$mode" in check|layout-check) rounds=1;; esac
results=${RESULTS:-$OUT/scheduler-$mode-$$}
test ! -e "$results"
test -s "$OUT/scheduler-flags.txt"
test -s "$OUT/scheduler-deps/metadata.txt"
grep -Fxq 'WF runtime: -DWF_COMPUTE_STATS=0; counter and observer absent' "$OUT/scheduler-flags.txt"
for unit in host native oracle work floor runtime wf-adapter static-adapter tbb-adapter parlay-adapter rayon-join-adapter rayon-iter-adapter; do
    test -s "$OUT/scheduler-$unit.o"
done
for unit in native work; do test -s "$OUT/scheduler-$unit.s"; done
for scheduler in wf static tbb parlay rayon-join rayon-iter; do test -x "$OUT/scheduler-$scheduler"; done
if test "$layout" = 1; then
    test -x "$OUT/scheduler-layout"
    grep -Fxq 'image=scheduler-layout selection=before-floor-entry dispatch=common-indirect' "$OUT/scheduler-layout-flags.txt"
    layout_hash=$(shasum -a 256 "$OUT/scheduler-layout")
fi
# Both common C computation and C++ dispatch are ordinary scalar release builds.
awk '
    /^C: / || /^C\+\+: / {
        if ($0 !~ /(^| )-fno-vectorize( |$)/ || $0 !~ /(^| )-fno-slp-vectorize( |$)/ ||
            $0 !~ /(^| )-fno-lto( |$)/ || $0 !~ /(^| )-DNDEBUG( |$)/) bad=1;
        if ($1=="C:") c++; else cpp++;
    }
    END { if (bad || c!=1 || cpp!=1) exit 1 }
' "$OUT/scheduler-flags.txt"
mkdir -p "$results"
printf '%s\n' "$results" > "$OUT/last-scheduler-$mode-path.txt"
{
    printf 'mode=%s rounds=%s kernel=scalar-records-state\n' "$mode" "$rounds"
    printf '%s\n' 'common_leaf=scheduler-native.o common_callback=scheduler-work.o'
    if test "$layout" = 1; then
        printf '%s\n' 'measured_executable=scheduler-layout all24configs=same-image' \
            'comparison=within-shared-image; standalone order/layout/library-load effects remain separate'
        cat "$OUT/scheduler-layout-flags.txt"
    else
        printf '%s\n' 'measured_executables=scheduler-wf,scheduler-static,scheduler-tbb,scheduler-parlay,scheduler-rayon-join,scheduler-rayon-iter'
    fi
    printf '%s\n' \
        'interval=joined-dispatch first=lazy-start warm=separate post-timing-capacity=excluded' \
        'outputs=distinct-preallocated-per-call batch=all-calls-and-gaps final-verification=excluded'
    git rev-parse HEAD
    git status --short
    uname -a
    cat "$OUT/scheduler-flags.txt"
    cat "$OUT/scheduler-deps/metadata.txt"
    if test "$(uname -s)" = Linux; then
        lscpu
        cat /proc/self/status
        for path in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; else printf '%s: unqualified\n' "$path"; fi
        done
    else
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    fi
    shasum -a 256 records_scheduler.cpp records_scheduler.h records_work.c records_native.c records_native.h \
        records_oracle.c records_runtime.c records_static.c records_tbb.cpp records_parlay.cpp records_selector.c \
        runtime.c runtime.h records-scheduler-bench.sh records-scheduler-deps.sh \
        records-scheduler-record.awk records-scheduler-memory.sh records-scheduler-memory.awk Makefile \
        records.c records.wf records_host.ll ../../../compiler/src/backend/wf_floor.c \
        records_rayon.c records-rayon/Cargo.toml records-rayon/Cargo.lock records-rayon/adapter.rs
    shasum -a 256 "$OUT"/scheduler-*.o "$OUT"/scheduler-*.s "$OUT/scheduler-flags.txt" \
        "$OUT/scheduler-wf" "$OUT/scheduler-static" "$OUT/scheduler-tbb" "$OUT/scheduler-parlay" \
        "$OUT/scheduler-rayon-join" "$OUT/scheduler-rayon-iter"
    if test "$layout" = 1; then
        printf '%s\n' "$layout_hash"
        shasum -a 256 "$OUT/scheduler-layout-flags.txt" "$OUT/scheduler-layout.symbols"
        cat "$OUT/scheduler-layout.symbols"
    fi
    find "$OUT/scheduler-deps" -type f -exec shasum -a 256 {} +
} > "$results/manifest.txt"
git diff --binary HEAD > "$results/source.patch"
while read -r scheduler backend shutdown; do
    for width in 1 2 4; do printf '%s %s %s %s\n' "$scheduler" "$backend" "$width" "$shutdown"; done
done > "$results/configurations.txt" <<'CONFIG'
wf wf-runtime 0
static static-spin 1
tbb oneTBB-v2023.1.0-auto-grain1 0
parlay parlay-native-grain1 1
rayon-join rayon-1.12.0-join 0
rayon-iter rayon-1.12.0-par-iter 0
CONFIG
if test "$layout" = 1; then
    for group in 4 16; do
        for width in 1 2 4; do
            printf 'wf-group%s wf-runtime-group%s %s 0\n' "$group" "$group" "$width"
        done
    done >> "$results/configurations.txt"
fi
configurations=$(wc -l < "$results/configurations.txt")
if test "$layout" = 1; then test "$configurations" -eq 24; else test "$configurations" -eq 18; fi
while read -r scheduler backend width shutdown; do
    qualifier="$OUT/scheduler-$scheduler-check-w$width.log"
    if test "$layout" = 1; then qualifier="$OUT/scheduler-layout-$scheduler-check-w$width.log"; fi
    awk -v backend="$backend" -v width="$width" -v shutdown="$shutdown" '
        { if (NR!=1 || $8 !~ /^capacity_waves=[1-3]$/ ||
            $0 != "record scheduler qualification PASS: backend=" backend " width=" width " actual=" width " " $8 \
                  " batches=780 outputs=270660 explicit_shutdown=" shutdown) bad=1 }
        END { if (bad || NR!=1) exit 1 }
    ' "$qualifier"
    cp "$qualifier" "$results/"
done < "$results/configurations.txt"
awk '
    NR==1 { if ($0!="UTF8 leaf qualification PASS: calls=4595603 all-codepoints all-truncations two-byte-pairs") bad=1 }
    NR==2 { if ($0!="UTF8 batch qualification PASS: world=sequential calls=342 results=1821070") bad=1 }
    END { if (bad || NR!=2) exit 1 }
' "$OUT/scheduler-leaf-check.log"
cp "$OUT/scheduler-leaf-check.log" "$results/"

# Each input retains the same seed across grains, cadences, widths and passes.
# Fields: shape, count, max length, records per callback, seed, cadence.
if test "$layout" = 1; then
    # Same inputs/seeds and grain16 as the corresponding standalone cells.
    # Four dense cells bound this layout diagnostic independently of the full panel.
    cat > "$results/cells.txt" <<'CELLS'
unicode 256 65536 16 844057 dense
unicode 4097 128 16 851976 dense
error-first 256 65536 16 867814 dense
error-first 4097 128 16 875733 dense
CELLS
elif test "$mode" = check; then
    while read -r shape count limit grain seed; do
        for cadence in checked dense sleep-100us sleep-1ms; do
            printf '%s %s %s %s %s %s\n' "$shape" "$count" "$limit" "$grain" "$seed" "$cadence"
        done
    done > "$results/cells.txt" <<'CELLS'
ascii 0 0 1 820300
unicode 33 17 16 828219
skew 257 64 64 836138
CELLS
else
    input=0
    for shape in ascii unicode error-first error-last skew; do
        for size in '256 65536' '4097 128' '65536 16'; do
            input=$((input+1))
            seed=$((812381+input*7919))
            for grain in 16 64; do
                cadences='checked dense'
                case "$shape $size $grain" in
                    'error-first 256 65536 16'|'error-first 4097 128 16'|'unicode 4097 128 16')
                        cadences="$cadences sleep-100us sleep-1ms";;
                esac
                for cadence in $cadences; do
                    printf '%s %s %s %s %s\n' "$shape" "$size" "$grain" "$seed" "$cadence"
                done
            done
        done
    done > "$results/cells.txt"
fi
printf 'shape\trecords\tmax_length\tseed\tbytes\n' > "$results/input-bytes.tsv"
printf 'backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\twarm_calls\tcall_mean_ns\tcall_min_ns\tcall_max_ns\tuser_mean_us\tsystem_mean_us\tcpu_mean_us\tvoluntary_switches_sum\tinvoluntary_switches_sum\tprocess_maxrss_bytes\tcadence\trequested_gap_ns\tgap_mean_ns\tgap_min_ns\tgap_max_ns\tbatch_ns\tbatch_user_us\tbatch_system_us\tbatch_cpu_us\tbatch_maxrss_bytes\tbatch_voluntary_switches\tbatch_involuntary_switches\tbatch_includes_checks\n' > "$results/summary.tsv"
cell=0
processes=0
previous_input=''
input_bytes=''
while read -r shape count limit grain seed cadence; do
    cell=$((cell+1))
    key="$shape-$count-$limit-$seed"
    if test "$key" != "$previous_input"; then previous_input=$key; input_bytes=''; fi
    chunks=$(((count+grain-1)/grain))
    reps=$((8000000/(count*limit+1)))
    if test "$reps" -lt 4; then reps=4; fi
    if test "$reps" -gt 32; then reps=32; fi
    case "$mode" in check|layout-check) reps=2;; esac
    checks=0
    case "$cadence" in
        checked) requested_gap=0; checks=1;;
        dense) requested_gap=0;;
        sleep-100us) requested_gap=100000;;
        sleep-1ms) requested_gap=1000000;;
        *) exit 1;;
    esac
    pass=0
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((pass+cell))" -v reverse="$((pass%2))" '
            { a[NR]=$0 } END { for(j=0;j<NR;j++) { i=(j+shift)%NR; if(reverse) i=NR-1-i; print a[i+1] } }
        ' "$results/configurations.txt" > "$results/order.txt"
        while read -r scheduler backend width shutdown; do
            file="$results/$shape-n$count-l$limit-g$grain-$cadence-$scheduler-w$width-p$pass.tsv"
            if test "$layout" = 1; then set -- "$OUT/scheduler-layout" "$scheduler"; else set -- "$OUT/scheduler-$scheduler"; fi
            if WF_WORKERS="$width" "$@" "$width" "$count" "$limit" "$shape" \
                "$grain" "$reps" "$seed" "$pass" "$cadence" > "$file" 2>&1; then :; else
                status=$?
                printf 'scheduler child rejected: status=%s file=%s\n' "$status" "$file" >&2
                cat "$file" >&2
                exit "$status"
            fi
            if awk -F '\t' -v backend="$backend" -v width="$width" -v shape="$shape" -v count="$count" \
                -v limit="$limit" -v grain="$grain" -v chunks="$chunks" -v seed="$seed" -v pass="$pass" \
                -v reps="$reps" -v shutdown="$shutdown" -v expected_bytes="$input_bytes" \
                -v cadence="$cadence" -v requested_gap="$requested_gap" -v checks="$checks" \
                -f records-scheduler-record.awk "$file" >> "$results/summary.tsv"; then :; else
                status=$?
                printf 'scheduler row contract rejected: status=%s file=%s\n' "$status" "$file" >&2
                cat "$file" >&2
                exit "$status"
            fi
            if test -z "$input_bytes"; then
                input_bytes=$(awk -F '\t' 'NR==3 { print $5 }' "$file")
                printf '%s\t%s\t%s\t%s\t%s\n' "$shape" "$count" "$limit" "$seed" "$input_bytes" >> "$results/input-bytes.tsv"
            fi
            processes=$((processes+1))
        done < "$results/order.txt"
        pass=$((pass+1))
    done
done < "$results/cells.txt"
expected=$((cell*configurations*rounds))
test "$processes" -eq "$expected"
test "$(wc -l < "$results/summary.tsv")" -eq "$((expected+1))"
if test "$layout" = 1; then test "$(shasum -a 256 "$OUT/scheduler-layout")" = "$layout_hash"; fi
printf 'record scheduler %s PASS: processes=%s results=%s\n' "$mode" "$processes" "$results"
