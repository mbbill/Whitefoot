#!/bin/sh
# Common callback observations, separate from the ordinary timing panel.
# Owned by this mechanism experiment; retire with its shared callback boundary.
set -eu
cd "$(dirname "$0")"
requested=${1:?check, calibrate, events-check or events-calibrate required}
events=0
case "$requested" in
    check|calibrate) mode=$requested;;
    events-check) mode=check; events=1;;
    events-calibrate) mode=calibrate; events=1;;
    *) exit 1;;
esac
: "${OUT:?absolute build output required}"
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
widths=4; reps=8
if test "$mode" = check; then rounds=1; widths='1 2 4'; reps=2; fi
results=${RESULTS:-$OUT/scheduler-trace-$requested-$$}
test ! -e "$results"
test -x "$OUT/scheduler-trace"
test -s "$OUT/scheduler-trace-flags.txt"
image_hash=$(shasum -a 256 "$OUT/scheduler-trace")
if test "$events" -eq 1; then
    test -x "$OUT/scheduler-events"
    events_hash=$(shasum -a 256 "$OUT/scheduler-events")
fi
mkdir -p "$results"
printf '%s\n' "$results" > "$OUT/last-scheduler-trace-$requested-path.txt"
{
    printf 'mode=%s rounds=%s reps=%s units=common-callbacks\n' "$mode" "$rounds" "$reps"
    printf '%s\n' 'plain=observer-image-control; not-original-timing-image' \
        'identity=per-cell-claim-and-cached-native-TID; timeline=identity-and-two-clocks' \
        'callback-time=wall-including-preemption; runtime-task-counts=image-dependent' \
        'offered-bytes=input-range-size; actual-bytes-examined=unobserved' \
        'first=lazy-start; batch=dispatch-and-observation; checks-capacity-reporting=after-batch'
    git rev-parse HEAD
    git status --short
    uname -a
    if test "$(uname -s)" = Linux; then
        lscpu
        cat /proc/self/status
        for path in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective /proc/sys/kernel/perf_event_paranoid; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; else printf '%s: unqualified\n' "$path"; fi
        done
    else
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    fi
    cat "$OUT/scheduler-flags.txt" "$OUT/scheduler-layout-flags.txt" "$OUT/scheduler-trace-flags.txt" "$OUT/scheduler-deps/metadata.txt"
    printf '%s\n' "$image_hash"
    if test "$events" -eq 1; then
        printf '%s\n' 'runtime_events=separate-image; matched-trace-control-interleaved' \
            'runtime_snapshots=per-call-clock-and-resource-envelope; validation-and-snapshots-inside-batch' \
            'joined-job-conservation=checked; background-search-wait-signal-deltas=not-instantaneous-flows' \
            "$events_hash"
        cat "$OUT/scheduler-events-flags.txt" "$OUT/scheduler-events-deps/metadata.txt"
        shasum -a 256 runtime_events.h records-rayon-events.sh records-rayon-events.patch
        find "$OUT/scheduler-events-deps" -type f -exec shasum -a 256 {} +
    fi
    shasum -a 256 records_scheduler.cpp records_scheduler.h records_work.c records_native.c records_native.h \
        records_oracle.c records_runtime.c records_static.c records_tbb.cpp records_parlay.cpp records_rayon.c \
        records_selector.c runtime.c runtime.h Makefile records-scheduler-trace.sh records-scheduler-trace.awk \
        records-scheduler-memory.sh records-scheduler-memory.awk records-scheduler-deps.sh \
        records-rayon/Cargo.toml records-rayon/Cargo.lock records-rayon/adapter.rs ../../../compiler/src/backend/wf_floor.c
    shasum -a 256 "$OUT"/scheduler-*.o "$OUT"/scheduler-*.s "$OUT"/scheduler-*.symbols \
        "$OUT/scheduler-layout" "$OUT/scheduler-trace-flags.txt"
    find "$OUT/scheduler-deps" -type f -exec shasum -a 256 {} +
} > "$results/manifest.txt"
git diff --binary HEAD > "$results/source.patch"
while read -r selector backend shutdown; do
    variants=trace
    if test "$events" -eq 1; then
        case "$selector" in wf|rayon-join) variants='trace events';; *) continue;; esac
    fi
    for width in $widths; do for level in plain identity timeline; do
        for variant in $variants; do
            printf '%s %s %s %s %s %s\n' "$selector" "$backend" "$width" "$shutdown" "$level" "$variant"
        done
    done; done
done > "$results/configurations.txt" <<'CONFIG'
wf wf-runtime 0
static static-spin 1
tbb oneTBB-v2023.1.0-auto-grain1 0
parlay parlay-native-grain1 1
rayon-join rayon-1.12.0-join 0
rayon-iter rayon-1.12.0-par-iter 0
wf-group4 wf-runtime-group4 0
wf-group16 wf-runtime-group16 0
CONFIG
if test "$mode" = check; then
    cat > "$results/cells.txt" <<'CELLS'
ascii 0 0 1 820300
error-first 1 1 16 828219
unicode 33 17 16 828219
skew 257 64 16 836138
CELLS
else
    cat > "$results/cells.txt" <<'CELLS'
error-first 256 65536 16 828219
unicode 256 65536 16 828219
CELLS
    if test "$events" -eq 1; then printf '%s\n' 'error-first 4097 64 16 828219' >> "$results/cells.txt"; fi
fi
printf 'backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\treps\tlevel\tcalls\ttraced_chunks\tbatch_start_ns\tbatch_end_ns' > "$results/summary.tsv"
if test "$events" -eq 1; then printf '\tcounter_image' >> "$results/summary.tsv"; fi
printf '\n' >> "$results/summary.tsv"
cell=0; processes=0
while read -r shape count limit grain seed; do
    cell=$((cell+1)); chunks=$(((count+grain-1)/grain)); pass=0; expected_bytes=''
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((cell+pass))" -v reverse="$((pass%2))" '
            { a[NR]=$0 } END { for(j=0;j<NR;j++) { i=(j+shift)%NR; if(reverse) i=NR-1-i; print a[i+1] } }
        ' "$results/configurations.txt" > "$results/order.txt"
        while read -r selector backend width shutdown level variant; do
            file="$results/$shape-n$count-$selector-w$width-$level-p$pass.tsv"
            schema=''
            if test "$events" -eq 1; then file="$results/$shape-n$count-$selector-w$width-$level-$variant-p$pass.tsv"; fi
            if test "$variant" = events; then
                case "$selector" in wf) schema=wf-1;; rayon-join) schema=rayon-join-1;; *) exit 1;; esac
            fi
            WF_WORKERS="$width" "$OUT/scheduler-$variant" "$selector" trace "$width" "$count" "$limit" "$shape" \
                "$grain" "$reps" "$seed" "$pass" "$level" > "$file" 2>&1 || { cat "$file" >&2; exit 1; }
            awk -F '\t' -v backend="$backend" -v width="$width" -v shape="$shape" -v count="$count" \
                -v limit="$limit" -v grain="$grain" -v chunks="$chunks" -v seed="$seed" -v pass="$pass" \
                -v reps="$reps" -v level="$level" -v shutdown="$shutdown" -v runtime_schema="$schema" \
                -f records-scheduler-trace.awk "$file" > "$results/row.tsv"
            bytes=$(cut -f5 "$results/row.tsv")
            if test -z "$expected_bytes"; then expected_bytes=$bytes; else test "$bytes" = "$expected_bytes"; fi
            if test "$events" -eq 1; then
                awk -v image="$variant" '{print $0 "\t" image}' "$results/row.tsv" >> "$results/summary.tsv"
            else cat "$results/row.tsv" >> "$results/summary.tsv"; fi
            processes=$((processes+1))
        done < "$results/order.txt"
        pass=$((pass+1))
    done
done < "$results/cells.txt"
configurations=$(wc -l < "$results/configurations.txt")
if test "$events" -eq 1; then
    if test "$mode" = check; then test "$configurations" -eq 36; else test "$configurations" -eq 12; fi
else
    if test "$mode" = check; then test "$configurations" -eq 72; else test "$configurations" -eq 24; fi
fi
test "$processes" -eq "$((cell*configurations*rounds))"
test "$(wc -l < "$results/summary.tsv")" -eq "$((processes+1))"
test "$(shasum -a 256 "$OUT/scheduler-trace")" = "$image_hash"
if test "$events" -eq 1; then test "$(shasum -a 256 "$OUT/scheduler-events")" = "$events_hash"; fi
if test "$mode" = check && test "$events" -eq 0; then
    original="$results/unicode-n33-wf-w4-timeline-p0.tsv"
    for fault in missing duplicate trailing; do
        awk -v fault="$fault" '
            /^chunk\t/ && !changed { changed=1; if(fault=="missing")next; if(fault=="duplicate")print }
            { print } END { if(fault=="trailing")print "unexpected trailing diagnostic" }
        ' "$original" > "$results/negative-$fault.tsv"
        if awk -F '\t' -v backend=wf-runtime -v width=4 -v shape=unicode -v count=33 \
            -v limit=17 -v grain=16 -v chunks=3 -v seed=828219 -v pass=0 -v reps=2 \
            -v level=timeline -v shutdown=0 -f records-scheduler-trace.awk "$results/negative-$fault.tsv" \
            > "$results/negative-$fault-summary.tsv" 2> "$results/negative-$fault.log"; then
            echo "trace validator accepted $fault report" >&2; exit 1
        fi
        test ! -s "$results/negative-$fault-summary.tsv"
        grep -Eq '^scheduler trace report rejected at line [0-9]+$' "$results/negative-$fault.log"
    done
fi
if test "$mode" = check && test "$events" -eq 1; then
    original="$results/unicode-n33-wf-w4-timeline-events-p0.tsv"
    for fault in missing duplicate completion schema trailing; do
        awk -F '\t' -v OFS='\t' -v fault="$fault" '
            /^# runtime_events / && fault=="schema" {next}
            /^runtime\t/ && !changed {
                changed=1; if(fault=="missing")next; if(fault=="duplicate")print
                if(fault=="completion")$11=$11+1
            }
            {print} END {if(fault=="trailing")print "unexpected trailing diagnostic"}
        ' "$original" > "$results/negative-$fault.tsv"
        if awk -F '\t' -v backend=wf-runtime -v width=4 -v shape=unicode -v count=33 \
            -v limit=17 -v grain=16 -v chunks=3 -v seed=828219 -v pass=0 -v reps=2 \
            -v level=timeline -v shutdown=0 -v runtime_schema=wf-1 \
            -f records-scheduler-trace.awk "$results/negative-$fault.tsv" \
            > "$results/negative-$fault-summary.tsv" 2> "$results/negative-$fault.log"; then
            echo "event validator accepted $fault report" >&2; exit 1
        fi
        test ! -s "$results/negative-$fault-summary.tsv"
        grep -Eq '^scheduler trace report rejected at line [0-9]+$' "$results/negative-$fault.log"
    done
fi
printf 'record scheduler trace %s PASS: processes=%s calls=%s results=%s\n' "$requested" "$processes" "$((processes*(reps+1)))" "$results"
