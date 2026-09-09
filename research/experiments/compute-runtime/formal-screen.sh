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
modes='before candidate replica recovered previous idle4096'
references='before recovered previous idle4096 replica'
diagnostic_modes='candidate previous idle4096'
case "$host" in
    Darwin|Linux) ;;
    MINGW*|MSYS*)
        exe=.exe
        floor=wf_floor_windows.c
        leaf=prim_windows.c
        platform_flags=
        libraries='-lpsapi -lws2_32'
        # The frozen recovered control has no qualified Windows port.
        modes='before candidate replica previous idle4096'
        references='before previous idle4096 replica'
        diagnostic_modes='before candidate previous idle4096'
        ;;
    *) echo "unsupported native screen host: $host" >&2; exit 1 ;;
esac
before=188088d41552d0d3bccf8368798dcc44702bf75c
previous=f2d9d0fab8a8b9892c6c3c8fc74f7159164cff45
root=$(git rev-parse --show-toplevel)
git -C "$root" diff --exit-code "$before" -- \
    research/experiments/compute-runtime/runtime.c \
    research/experiments/compute-runtime/runtime.h \
    research/experiments/compute-runtime/runtime_events.h
mkdir -p "$OUT"
mkdir "$OUT/formal-screen"
out=$(cd "$OUT/formal-screen" && pwd)
mkdir -p "$out/baseline-source" "$out/previous-source" "$out/source" "$out/raw" "$out/diagnostics"
git -C "$root" archive "$before" compiler/src/backend/sched "compiler/src/backend/$floor" \
    > "$out/before.tar"
tar -xf "$out/before.tar" -C "$out/baseline-source"
git -C "$root" archive "$previous" compiler/src/backend/sched compiler/src/backend/completion \
    compiler/src/backend/windows_runtime.c compiler/src/backend/windows_runtime.h "compiler/src/backend/$floor" \
    > "$out/previous.tar"
tar -xf "$out/previous.tar" -C "$out/previous-source"
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
    printf 'before=%s\nprevious=%s\n' "$before" "$previous"
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
    'idle4096: current runtime sources with only -DWF_SCHED_IDLE_SPIN_ROUNDS=4096u changed; default remains 256' \
    'diagnostics: separate longer batches; candidate/previous/idle4096 use WF_SCHED_REPORT=1, before uses 0; not pooled into wall samples' \
    'wall samples: 4096 warm calls for n4096, 512 for n65536; first64 retained as a separate short view of each process, not independent extra samples' \
    'WF: --par defaults; same optimized WF object in every attribution image' \
    'CLI: normal --par link, correctness only; end-to-end timing remains open' > "$out/flags.txt"
if test -n "$exe"; then
    printf '%s\n' 'Windows before: old scheduler/floor plus current host/completion overlay. Previous: its own frozen scheduler/floor/host/completion sources. Candidate: all current sources. Every set uses matching private headers.' \
        'Windows host-wait counts: batch deltas from each matching bridge, including verification; announcements are not guaranteed kernel sleeps, signals are not awakened-thread counts.' \
        'Windows diagnostic before: WF_SCHED_REPORT=0; historical scheduler counters are never read live.' >> "$out/flags.txt"
fi
"$WFC" --par --emit-llvm fir.wf fir_direct.wf -o "$out/module.ll"
"$WFC" --par fir.wf fir_direct.wf -o "$out/command$exe"
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
    if test "$mode" = idle4096; then policy_flags=-DWF_SCHED_IDLE_SPIN_ROUNDS=4096u; fi
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
awk -F '\t' -v references="$references" '
    BEGIN {reference_count=split(references,reference," ")}
    NR==1 {next}
    {cell=$2 FS $3 FS $4; cells[cell]=1; value[$1 FS cell FS $5]=$6}
    END {
        print "workers\tn\ttile\treference\tmedian_paired_ratio\tmin_ratio\tmax_ratio\twall_screen"
        for(cell in cells) for(r=1;r<=reference_count;r++) {
            ref=reference[r]
            for(p=0;p<5;p++) {
                a=value["candidate" FS cell FS p]; b=value[ref FS cell FS p]
                if(a<=0||b<=0)exit 2
                ratio[p]=a/b
            }
            for(i=0;i<5;i++)for(j=i+1;j<5;j++)if(ratio[j]<ratio[i]) {
                tmp=ratio[i];ratio[i]=ratio[j];ratio[j]=tmp
            }
            within=ratio[2]<=1.05 && (ref!="replica" || ratio[2]>=1/1.05)
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
                WF_SCHED_REPORT="$reports" WF_WORKERS="$width" "$out/$mode$exe" \
                    wf 16 "$n" "$tile" "$diagnostic_calls" 92821 0 > "$log"
                wait_reports=0
                if test -n "$exe"; then wait_reports=1; fi
                awk -F '\t' -v width="$width" -v expected="$diagnostic_calls" \
                    -v expected_reports="$reports" -v expected_wait="$wait_reports" \
                    -v expected_spin="$spin_rounds" '
                    {sub(/\r$/, "")}
                    /^# actual_lanes=/ {split($0,a,"="); lanes++; if(a[2]!=width)bad=1}
                    /^# sched: / {
                        reports++
                        if($0 !~ (" spin_rounds=" expected_spin " "))bad=1
                    }
                    /^# host_wait: announcements=[0-9]+ signals=[0-9]+ / {waits++}
                    $10=="warm" {calls++}
                    END {exit bad || lanes!=1 || reports!=expected_reports ||
                        waits!=expected_wait || calls!=expected}' "$log"
            done
        done
    done
done
if test -n "$exe"; then
    find "$out" -type f ! -name manifest.sha256 -exec sha256sum {} + > "$out/manifest.sha256"
else
    find "$out" -type f ! -name manifest.sha256 -exec shasum -a 256 {} + > "$out/manifest.sha256"
fi
exit "$result"
