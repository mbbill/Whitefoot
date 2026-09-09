#!/bin/sh
# Initial shared-runtime attribution, not the full delivery qualification.
# Called by `make formal-screen`; every timed image uses the same WF object.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT}"
: "${WFC:?set WFC to the current compiler}"
CC=${CC:-/usr/bin/clang}
# Scheduler reports use race-free candidate and previous-runtime counters;
# the older before counters are never read live. Windows images also read
# host-bridge wait counters at batch boundaries, outside timed core calls.
WF_SCHED_REPORT=0
export WF_SCHED_REPORT
host=$(uname -s)
exe=
floor=wf_floor.c
leaf=prim_host.c
platform_flags=-pthread
libraries=-lm
modes='before candidate replica recovered previous unaligned slotbase idle4096 nostats'
references='before recovered previous unaligned slotbase idle4096 nostats replica'
diagnostic_modes='candidate previous unaligned slotbase idle4096 nostats'
case "$host" in
    Darwin|Linux) ;;
    MINGW*|MSYS*)
        exe=.exe
        floor=wf_floor_windows.c
        leaf=prim_windows.c
        platform_flags=
        libraries='-lpsapi -lws2_32'
        # The frozen recovered control has no qualified Windows port.
        modes='before candidate replica previous unaligned slotbase idle4096 nostats'
        references='before previous unaligned slotbase idle4096 nostats replica'
        diagnostic_modes='before candidate previous unaligned slotbase idle4096 nostats'
        ;;
    *) echo "unsupported native screen host: $host" >&2; exit 1 ;;
esac
before=188088d41552d0d3bccf8368798dcc44702bf75c
previous=f2d9d0fab8a8b9892c6c3c8fc74f7159164cff45
unaligned=8b61e7c4387628c05fe5e98006ea524e54964521
slotbase=dc383eef2787d028d1c094902e7c2ed216dc1d0f
root=$(git rev-parse --show-toplevel)
git -C "$root" diff --exit-code "$before" -- \
    research/experiments/compute-runtime/runtime.c \
    research/experiments/compute-runtime/runtime.h \
    research/experiments/compute-runtime/runtime_events.h
mkdir -p "$OUT"
mkdir "$OUT/formal-screen"
out=$(cd "$OUT/formal-screen" && pwd)
mkdir -p "$out/baseline-source" "$out/previous-source" "$out/unaligned-source" "$out/slotbase-source" "$out/source" "$out/raw" "$out/diagnostics"
git -C "$root" archive "$before" compiler/src/backend/sched "compiler/src/backend/$floor" \
    > "$out/before.tar"
tar -xf "$out/before.tar" -C "$out/baseline-source"
git -C "$root" archive "$previous" compiler/src/backend/sched compiler/src/backend/completion \
    compiler/src/backend/windows_runtime.c compiler/src/backend/windows_runtime.h "compiler/src/backend/$floor" \
    > "$out/previous.tar"
tar -xf "$out/previous.tar" -C "$out/previous-source"
git -C "$root" archive "$unaligned" compiler/src/backend/sched compiler/src/backend/completion \
    compiler/src/backend/windows_runtime.c compiler/src/backend/windows_runtime.h "compiler/src/backend/$floor" \
    > "$out/unaligned.tar"
tar -xf "$out/unaligned.tar" -C "$out/unaligned-source"
git -C "$root" archive "$slotbase" compiler/src/backend/sched compiler/src/backend/completion \
    compiler/src/backend/windows_runtime.c compiler/src/backend/windows_runtime.h "compiler/src/backend/$floor" \
    > "$out/slotbase.tar"
tar -xf "$out/slotbase.tar" -C "$out/slotbase-source"
cp fir.wf fir_direct.wf fir_host.ll fir_bench.c fir_native.c fir_native.h \
    fir_check.c formal-screen.sh runtime.c runtime.h runtime_events.h "$out/source/"
cp -R "$root/compiler/src/backend/sched" "$out/source/"
cp "$root/compiler/src/backend/$floor" "$out/source/"
if test -n "$exe"; then
    # The generated Windows object reaches host diagnostics even for compute.
    # The oldest baseline needs the current host overlay; previous keeps its
    # frozen host/completion implementation. Each uses matching private headers.
    for destination in "$out/source" "$out/baseline-source/compiler/src/backend"; do
        mkdir -p "$destination/completion"
        cp "$root/compiler/src/backend/windows_runtime.h" "$destination/"
        cp "$root"/compiler/src/backend/completion/*.h "$destination/completion/"
        for unit in windows_runtime.c completion/runtime.c completion/wait_windows.c \
            completion/file_adapter.c completion/file_windows.c completion/bridge.c \
            completion/windows_iocp.c; do
            cp "$root/compiler/src/backend/$unit" "$destination/$unit"
        done
    done
fi
cp "$WFC" "$out/whitefootc"
{
    git rev-parse HEAD
    printf 'before=%s\nprevious=%s\nunaligned=%s\nslotbase=%s\n' "$before" "$previous" "$unaligned" "$slotbase"
    git diff --stat
    uname -a
    "$CC" --version
    rustc -vV
    if test "$host" = Darwin; then
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    elif test "$host" = Linux; then
        lscpu
        cat /proc/self/status
        for quota in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpu/cpu.cfs_quota_us; do
            if test -f "$quota"; then cat "$quota"; fi
        done
    else
        powershell.exe -NoProfile -Command 'Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | Format-List'
        powershell.exe -NoProfile -Command 'Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,TotalVisibleMemorySize | Format-List'
        MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' powercfg.exe /getactivescheme
    fi
} > "$out/host.txt"
git diff --binary > "$out/source.patch"
flags="-std=c11 -O3 -g $platform_flags -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto"
printf '%s\n' "$CC $flags; recovered additionally -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_CONTROL; shared additionally -DWF_SHARED_CONTROL" \
    "measured_modes=$modes; references=$references; libraries=$libraries" \
    'replica: byte-identical copy of candidate, independently invoked; raw runtime label remains candidate; A/A wall band is symmetric' \
    'previous: frozen f2d9d0fa maintained scheduler/floor and Windows host/completion sources with repaired wait rearming, before owned-inline completion' \
    'unaligned: frozen 8b61e7c4 maintained runtime before counter isolation and owner-local task slots; retained historical comparison.' \
    'slotbase: frozen dc383eef maintained runtime with owner-local task slots and original idle registration; restored after the delayed-registration experiment at 461a7130. Retained direct baseline; instruction layout can also differ.' \
    'idle4096: current runtime sources with only -DWF_SCHED_IDLE_SPIN_ROUNDS=4096u changed; default remains 256' \
    'nostats: current runtime sources with only -DWF_SCHED_STATS=0 changed; default remains 1. Ordinary link placement can differ, so this is not an isolated instruction-cost claim.' \
    'diagnostics: separate longer batches; candidate/previous/unaligned/slotbase/idle4096/nostats request WF_SCHED_REPORT=1, before uses 0. nostats must omit scheduler counters while retaining the external epoch observation; wake_epoch counts notification-epoch advances, not task publications or kernel wakeups; not pooled into wall samples' \
    'wall samples: 4096 warm calls for n4096, 512 for n65536; first64 retained as a separate short view of each process, not independent extra samples' \
    'WF: --par --no-vectorize; same optimized WF object in every attribution image' \
    'CLI: normal --par --no-vectorize link at -O2, correctness only; end-to-end timing remains open' > "$out/flags.txt"
if test -n "$exe"; then
    printf '%s\n' 'Windows before: old scheduler/floor plus current host/completion overlay. Previous: its own frozen scheduler/floor/host/completion sources. Candidate: all current sources. Every set uses matching private headers.' \
        'Windows host-wait counts: batch deltas from each matching bridge, including verification; announcements are not guaranteed kernel sleeps, signals are not awakened-thread counts.' \
        'Windows diagnostic before: WF_SCHED_REPORT=0; historical scheduler counters are never read live.' >> "$out/flags.txt"
fi
"$WFC" --par --no-vectorize --emit-llvm fir.wf fir_direct.wf -o "$out/module.ll"
"$WFC" --par --no-vectorize fir.wf fir_direct.wf -o "$out/command$exe"
sed -e 's/@main(/@wf_research_fir_command_main(/g' \
    -e 's/@wmain(/@wf_research_fir_windows_command_main(/g' \
    -e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$out/module.ll" > "$out/host.ll"
cat fir_host.ll >> "$out/host.ll"
# Deliberate word splitting: flags is the fixed compiler argument list above.
# shellcheck disable=SC2086
"$CC" $flags -Wno-override-module -c "$out/host.ll" -o "$out/wf.o"
# shellcheck disable=SC2086
"$CC" $flags -c fir_native.c -o "$out/native.o"
# Qualify the native oracle independently before using it in timed comparisons.
# shellcheck disable=SC2086
"$CC" $flags -DWF_FILTER_NATIVE fir_check.c "$out/native.o" $libraries -o "$out/oracle$exe"
"$out/oracle$exe" > "$out/oracle.txt"
for mode in $modes; do
    if test "$mode" = replica; then
        # Measure host/process variability with identical code and data.
        # Keep candidate/replica adjacent; the sample loop reverses their order.
        cp "$out/candidate$exe" "$out/replica$exe"
        cmp -s "$out/candidate$exe" "$out/replica$exe"
        continue
    fi
    runtime="$root/compiler/src/backend"
    policy_flags=
    if test "$mode" = previous; then runtime="$out/previous-source/compiler/src/backend"; fi
    if test "$mode" = unaligned; then runtime="$out/unaligned-source/compiler/src/backend"; fi
    if test "$mode" = slotbase; then runtime="$out/slotbase-source/compiler/src/backend"; fi
    if test "$mode" = idle4096; then policy_flags=-DWF_SCHED_IDLE_SPIN_ROUNDS=4096u; fi
    if test "$mode" = nostats; then policy_flags=-DWF_SCHED_STATS=0; fi
    if test "$mode" = before; then runtime="$out/baseline-source/compiler/src/backend"; fi
    set --
    if test -n "$exe"; then
        set -- -DWF_COMPLETION_WAIT_STATS "-I$runtime/completion"
        for unit in windows_runtime.c completion/runtime.c completion/wait_windows.c \
            completion/file_adapter.c completion/file_windows.c completion/bridge.c \
            completion/windows_iocp.c; do
            set -- "$@" "$runtime/$unit"
        done
    fi
    if test "$mode" = recovered; then
        # The existing, unchanged POSIX control is a baseline only.
        # shellcheck disable=SC2086
        "$CC" $flags -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_CONTROL "-DFIR_RUNTIME=\"$mode\"" \
            fir_bench.c runtime.c "$runtime/$floor" "$out/wf.o" "$out/native.o" \
            $libraries -o "$out/$mode$exe"
    else
        # shellcheck disable=SC2086
        "$CC" $flags $policy_flags -DWF_SHARED_CONTROL "-DFIR_RUNTIME=\"$mode\"" fir_bench.c \
            "$runtime/$floor" "$runtime/sched/core.c" \
            "$runtime/sched/$leaf" "$runtime/sched/entry.c" \
            "$@" "$out/wf.o" "$out/native.o" $libraries -o "$out/$mode$exe"
    fi
done
if test -n "$exe"; then cpus=$NUMBER_OF_PROCESSORS; else cpus=$(getconf _NPROCESSORS_ONLN); fi
widths=1
if test "$cpus" -ge 2; then widths="$widths 2"; fi
if test "$cpus" -ge 4; then widths="$widths 4"; fi
printf 'mode\tworkers\tn\ttile\tpass\tcore_mean_ns\tcycle_mean_ns\n' > "$out/means.tsv"
cp "$out/means.tsv" "$out/short-means.tsv"
for width in $widths; do
    WF_WORKERS="$width" "$out/command$exe" > "$out/cli-w$width.txt"
    for n in 4096 65536; do
        calls=512
        if test "$n" = 4096; then calls=4096; fi
        for tile in 16 64 256 1024; do
            pass=0
            while test "$pass" -lt 5; do
                order=$modes
                if test "$((pass % 2))" = 1; then
                    order=
                    for mode in $modes; do order="$mode $order"; done
                fi
                for mode in $order; do
                    log="$out/raw/$mode-w$width-n$n-t$tile-p$pass.tsv"
                    WF_WORKERS="$width" "$out/$mode$exe" wf 16 "$n" "$tile" "$calls" 92821 "$pass" > "$log"
                    awk -v width="$width" '
                        /^# actual_lanes=/ {split($2,a,"="); seen++; if(a[2]!=width)exit 1}
                        END {if(seen!=1)exit 1}' "$log"
                    awk -F '\t' -v mode="$mode" -v w="$width" -v n="$n" -v t="$tile" -v p="$pass" \
                        -v expected="$calls" -v short_file="$out/short-means.tsv" \
                        '$10=="warm" {
                            core+=$11;cycle+=$12;calls++
                            if(calls<=64){short_core+=$11;short_cycle+=$12}
                        }
                        END {
                            if(calls!=expected)exit 1
                            printf "%s\t%s\t%s\t%s\t%s\t%.3f\t%.3f\n",mode,w,n,t,p,core/calls,cycle/calls
                            printf "%s\t%s\t%s\t%s\t%s\t%.3f\t%.3f\n",mode,w,n,t,p,short_core/64,short_cycle/64 >> short_file
                        }' "$log" >> "$out/means.tsv"
                done
                pass=$((pass + 1))
            done
        done
    done
done
# Process means, not individual warm calls, are the independent samples.
# The short view overlaps the long view; it is never pooled as extra evidence.
summarize() {
awk -F '\t' -v references="${4:-$references}" -v subject="${3:-candidate}" '
    BEGIN {reference_count=split(references,reference," ")}
    NR==1 {next}
    {cell=$2 FS $3 FS $4; cells[cell]=1; value[$1 FS cell FS $5]=$6}
    END {
        print "workers\tn\ttile\treference\tmedian_paired_ratio\tmin_ratio\tmax_ratio\twall_screen"
        for(cell in cells) for(r=1;r<=reference_count;r++) {
            ref=reference[r]
            for(p=0;p<5;p++) {
                a=value[subject FS cell FS p]; b=value[ref FS cell FS p]
                if(a<=0||b<=0)exit 2
                ratio[p]=a/b
            }
            for(i=0;i<5;i++)for(j=i+1;j<5;j++)if(ratio[j]<ratio[i]) {
                tmp=ratio[i];ratio[i]=ratio[j];ratio[j]=tmp
            }
            identical=(ref=="replica" || ref=="layout-replica")
            within=ratio[2]<=1.05 && (!identical || ratio[2]>=1/1.05)
            verdict=within?"within-band":"investigate"
            if(verdict=="investigate")failed=1
            printf "%s\t%s\t%.4f\t%.4f\t%.4f\t%s\n",cell,ref,ratio[2],ratio[0],ratio[4],verdict
        }
        exit failed
    }' "$1" > "$2"
}
summarize "$out/means.tsv" "$out/summary.tsv" && result=0 || result=$?
summarize "$out/short-means.tsv" "$out/short-summary.tsv" || result=$?
cat "$out/summary.tsv"
cat "$out/short-summary.tsv"
# Preserve the completed wall verdict even if a later diagnostic fails.
# Current and previous-source counters are race-free. The older before control is excluded from
# scheduler-counter observation. Windows before additionally observes only
# its current bridge counters. Longer batches reduce CPU-accounting quantization but are
# diagnostic process samples, not extra independent samples in the wall test.
for width in $widths; do
    for n in 4096 65536; do
        diagnostic_calls=512
        if test "$n" = 4096; then diagnostic_calls=4096; fi
        for tile in 16 64 256 1024; do
            for mode in $diagnostic_modes; do
                log="$out/diagnostics/$mode-w$width-n$n-t$tile.tsv"
                reports=1
                spin_rounds=256
                if test "$mode" = idle4096; then spin_rounds=4096; fi
                # Historical scheduler counters are not race-free. The host
                # wait counters in the current overlay may safely be observed
                # for before too; previous observes its own frozen bridge.
                if test "$mode" = before; then reports=0; fi
                expected_reports=$reports
                if test "$mode" = nostats; then expected_reports=0; fi
                WF_SCHED_REPORT="$reports" WF_WORKERS="$width" "$out/$mode$exe" \
                    wf 16 "$n" "$tile" "$diagnostic_calls" 92821 0 > "$log"
                wait_reports=0
                if test -n "$exe"; then wait_reports=1; fi
                awk -F '\t' -v width="$width" -v expected="$diagnostic_calls" \
                    -v expected_reports="$expected_reports" -v expected_epochs="$reports" \
                    -v expected_wait="$wait_reports" \
                    -v expected_spin="$spin_rounds" '
                    {sub(/\r$/, "")}
                    /^# actual_lanes=/ {split($0,a,"="); lanes++; if(a[2]!=width)bad=1}
                    /^# sched: / {
                        reports++
                        if($0 !~ (" spin_rounds=" expected_spin " "))bad=1
                    }
                    /^# host_wait: announcements=[0-9]+ signals=[0-9]+ / {waits++}
                    /^# wake_epoch: advances=[0-9]+ scope=batch_including_checks$/ {epochs++}
                    $10=="warm" {calls++}
                    END {exit bad || lanes!=1 || reports!=expected_reports ||
                        waits!=expected_wait || epochs!=expected_epochs || calls!=expected}' "$log"
            done
        done
    done
done
if test "$host" = Linux; then
    # Keep the earlier join-placement question frozen at 8b61e7c4 versus f2.
    # Subsequent core changes alter other function sizes and are not a valid
    # subject for this non-join-address invariant. Ordinary screens above
    # still test the current candidate against every reference.
    # A declaration changes placement only, never the maintained runtime body.
    # Keeping join in a separate ELF section prevents its size from shifting
    # later computation functions. Verify every other text address before use.
    mkdir "$out/layout"
    cat > "$out/layout/join-section.h" <<'EOF'
struct wf_sched_core;
struct wf_sched_record;
__attribute__((section(".wf_join")))
void wf_sched_join(struct wf_sched_core *, struct wf_sched_record *, int);
EOF
    printf '%s\n' 'SECTIONS { .wf_join : { *(.wf_join) } } INSERT AFTER .fini;' > "$out/layout/join.ld"
    for mode in previous candidate; do
        runtime="$out/unaligned-source/compiler/src/backend"
        if test "$mode" = previous; then runtime="$out/previous-source/compiler/src/backend"; fi
        # shellcheck disable=SC2086
        "$CC" $flags -include "$out/layout/join-section.h" \
            -c "$runtime/sched/core.c" -o "$out/layout/$mode-core.o"
        # Both images use the same driver label to preserve string layout too.
        # shellcheck disable=SC2086
        "$CC" $flags -DWF_SHARED_CONTROL '-DFIR_RUNTIME="layout"' fir_bench.c \
            "$runtime/$floor" "$out/layout/$mode-core.o" \
            "$runtime/sched/$leaf" "$runtime/sched/entry.c" \
            "$out/wf.o" "$out/native.o" "-Wl,-T,$out/layout/join.ld" $libraries -o "$out/layout-$mode"
        nm -n --defined-only "$out/layout-$mode" > "$out/layout/$mode-symbols.txt"
        nm -n -S --defined-only "$out/layout-$mode" > "$out/layout/$mode-symbol-sizes.txt"
        readelf -W -S -l "$out/layout-$mode" > "$out/layout/$mode-sections.txt"
        objdump -d "$out/layout-$mode" > "$out/layout/$mode-disassembly.txt"
        awk '$2 ~ /^[tT]$/ && $3!="wf_sched_join" {print $1,$3}' \
            "$out/layout/$mode-symbols.txt" > "$out/layout/$mode-other-text.txt"
        test -s "$out/layout/$mode-other-text.txt"
    done
    cmp "$out/layout/previous-other-text.txt" "$out/layout/candidate-other-text.txt"
    cp "$out/layout-candidate" "$out/layout-replica"
    cmp "$out/layout-candidate" "$out/layout-replica"
    printf '%s\n' 'Linux layout cohort: frozen 8b61e7c4 (layout-candidate) versus f2 (layout-previous), not the current runtime candidate. Join placed after .fini; all non-join text symbol addresses match. Instruction/data equality is not assumed: size-bearing symbols, ELF maps and disassembly are retained. Smaller separate cohort, never pooled with ordinary samples.' >> "$out/flags.txt"
    printf 'mode\tworkers\tn\ttile\tpass\tcore_mean_ns\tcycle_mean_ns\n' > "$out/layout-means.tsv"
    cp "$out/layout-means.tsv" "$out/layout-short-means.tsv"
    for width in $widths; do
        for n in 4096 65536; do
            tile=16
            calls=512
            if test "$n" = 4096; then tile=1024; calls=4096; fi
            pass=0
            while test "$pass" -lt 5; do
                order='previous candidate replica'
                if test "$((pass % 2))" = 1; then order='replica candidate previous'; fi
                for mode in $order; do
                    mode="layout-$mode"
                    log="$out/raw/$mode-w$width-n$n-t$tile-p$pass.tsv"
                    WF_WORKERS="$width" "$out/$mode" wf 16 "$n" "$tile" "$calls" 92821 "$pass" > "$log"
                    awk -F '\t' -v mode="$mode" -v w="$width" -v n="$n" -v t="$tile" -v p="$pass" \
                        -v expected="$calls" -v short_file="$out/layout-short-means.tsv" '
                        /^# actual_lanes=/ {split($0,a,"="); lanes++; if(a[2]!=w)bad=1}
                        $10=="warm" {
                            core+=$11;cycle+=$12;calls++
                            if(calls<=64){short_core+=$11;short_cycle+=$12}
                        }
                        END {
                            if(bad || lanes!=1 || calls!=expected)exit 1
                            printf "%s\t%s\t%s\t%s\t%s\t%.3f\t%.3f\n",mode,w,n,t,p,core/calls,cycle/calls
                            printf "%s\t%s\t%s\t%s\t%s\t%.3f\t%.3f\n",mode,w,n,t,p,short_core/64,short_cycle/64 >> short_file
                        }' "$log" >> "$out/layout-means.tsv"
                done
                pass=$((pass + 1))
            done
        done
    done
    summarize "$out/layout-means.tsv" "$out/layout-summary.tsv" layout-candidate 'layout-previous layout-replica' || result=$?
    summarize "$out/layout-short-means.tsv" "$out/layout-short-summary.tsv" layout-candidate 'layout-previous layout-replica' || result=$?
    cat "$out/layout-summary.tsv"
    cat "$out/layout-short-summary.tsv"
fi
if test -n "$exe"; then
    find "$out" -type f ! -name manifest.sha256 -exec sha256sum {} + > "$out/manifest.sha256"
else
    find "$out" -type f ! -name manifest.sha256 -exec shasum -a 256 {} + > "$out/manifest.sha256"
fi
exit "$result"
