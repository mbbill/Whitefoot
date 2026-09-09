#!/bin/sh
# Initial POSIX shared-runtime attribution, not the full delivery qualification.
# Called by `make formal-screen`; every timed image uses the same WF object.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT}"
: "${WFC:?set WFC to the current compiler}"
CC=${CC:-/usr/bin/clang}
before=188088d41552d0d3bccf8368798dcc44702bf75c
root=$(git rev-parse --show-toplevel)
git -C "$root" diff --exit-code "$before" -- \
    research/experiments/compute-runtime/runtime.c \
    research/experiments/compute-runtime/runtime.h \
    research/experiments/compute-runtime/runtime_events.h
mkdir -p "$OUT"
mkdir "$OUT/formal-screen"
out=$(cd "$OUT/formal-screen" && pwd)
mkdir -p "$out/baseline-source" "$out/source" "$out/raw"
git -C "$root" archive "$before" compiler/src/backend/sched compiler/src/backend/wf_floor.c \
    > "$out/before.tar"
tar -xf "$out/before.tar" -C "$out/baseline-source"
cp fir.wf fir_direct.wf fir_host.ll fir_bench.c fir_native.c fir_native.h \
    fir_check.c formal-screen.sh runtime.c runtime.h runtime_events.h "$out/source/"
cp -R "$root/compiler/src/backend/sched" "$out/source/"
cp "$root/compiler/src/backend/wf_floor.c" "$out/source/"
cp "$WFC" "$out/whitefootc"
{
    git rev-parse HEAD
    printf 'before=%s\n' "$before"
    git diff --stat
    uname -a
    "$CC" --version
    rustc -vV
    if test "$(uname -s)" = Darwin; then
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    else
        lscpu
        cat /proc/self/status
        for quota in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpu/cpu.cfs_quota_us; do
            if test -f "$quota"; then cat "$quota"; fi
        done
    fi
} > "$out/host.txt"
git diff --binary > "$out/source.patch"
flags='-std=c11 -O3 -g -pthread -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto'
printf '%s\n' "$CC $flags; recovered additionally -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_CONTROL; shared additionally -DWF_SHARED_CONTROL" \
    'WF: --par defaults; same optimized WF object in every attribution image' \
    'CLI: normal --par link, correctness only; end-to-end timing remains open' > "$out/flags.txt"
"$WFC" --par --emit-llvm fir.wf fir_direct.wf -o "$out/module.ll"
"$WFC" --par fir.wf fir_direct.wf -o "$out/command"
sed -e 's/@main(/@wf_research_fir_command_main(/g' \
    -e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$out/module.ll" > "$out/host.ll"
cat fir_host.ll >> "$out/host.ll"
# Deliberate word splitting: flags is the fixed compiler argument list above.
# shellcheck disable=SC2086
"$CC" $flags -Wno-override-module -c "$out/host.ll" -o "$out/wf.o"
# shellcheck disable=SC2086
"$CC" $flags -c fir_native.c -o "$out/native.o"
# Qualify the native oracle independently before using it in timed comparisons.
# shellcheck disable=SC2086
"$CC" $flags -DWF_FILTER_NATIVE fir_check.c "$out/native.o" -lm -o "$out/oracle"
"$out/oracle" > "$out/oracle.txt"
for mode in before candidate recovered; do
    runtime="$root/compiler/src/backend"
    if test "$mode" = before; then runtime="$out/baseline-source/compiler/src/backend"; fi
    if test "$mode" = recovered; then
        # The existing, unchanged POSIX control is a baseline only.
        # shellcheck disable=SC2086
        "$CC" $flags -DWF_COMPUTE_STATS=0 -DWF_COMPUTE_CONTROL "-DFIR_RUNTIME=\"$mode\"" \
            fir_bench.c runtime.c "$runtime/wf_floor.c" "$out/wf.o" "$out/native.o" \
            -lm -o "$out/$mode"
    else
        # shellcheck disable=SC2086
        "$CC" $flags -DWF_SHARED_CONTROL "-DFIR_RUNTIME=\"$mode\"" fir_bench.c \
            "$runtime/wf_floor.c" "$runtime/sched/core.c" \
            "$runtime/sched/prim_host.c" "$runtime/sched/entry.c" \
            "$out/wf.o" "$out/native.o" -lm -o "$out/$mode"
    fi
done
cpus=$(getconf _NPROCESSORS_ONLN)
widths=1
if test "$cpus" -ge 2; then widths="$widths 2"; fi
if test "$cpus" -ge 4; then widths="$widths 4"; fi
printf 'mode\tworkers\tn\ttile\tpass\tcore_mean_ns\tcycle_mean_ns\n' > "$out/means.tsv"
for width in $widths; do
    WF_WORKERS="$width" "$out/command" > "$out/cli-w$width.txt"
    for n in 4096 65536; do
        for tile in 16 64 256 1024; do
            pass=0
            while test "$pass" -lt 5; do
                order='before candidate recovered'
                if test "$((pass % 2))" = 1; then order='recovered candidate before'; fi
                for mode in $order; do
                    log="$out/raw/$mode-w$width-n$n-t$tile-p$pass.tsv"
                    WF_WORKERS="$width" "$out/$mode" wf 16 "$n" "$tile" 64 92821 "$pass" > "$log"
                    awk -v width="$width" '
                        /^# actual_lanes=/ {split($2,a,"="); seen++; if(a[2]!=width)exit 1}
                        END {if(seen!=1)exit 1}' "$log"
                    awk -F '\t' -v mode="$mode" -v w="$width" -v n="$n" -v t="$tile" -v p="$pass" \
                        '$10=="warm" {core+=$11;cycle+=$12;calls++}
                        END {if(calls!=64)exit 1; printf "%s\t%s\t%s\t%s\t%s\t%.3f\t%.3f\n",mode,w,n,t,p,core/calls,cycle/calls}' \
                        "$log" >> "$out/means.tsv"
                done
                pass=$((pass + 1))
            done
        done
    done
done
# Process means, not individual warm calls, are the independent samples.
awk -F '\t' '
    NR==1 {next}
    {cell=$2 FS $3 FS $4; cells[cell]=1; value[$1 FS cell FS $5]=$6}
    END {
        print "workers\tn\ttile\treference\tmedian_paired_ratio\tmin_ratio\tmax_ratio\twall_screen"
        for(cell in cells) for(r=0;r<2;r++) {
            ref=r==0?"before":"recovered"
            for(p=0;p<5;p++) {
                a=value["candidate" FS cell FS p]; b=value[ref FS cell FS p]
                if(a<=0||b<=0)exit 2
                ratio[p]=a/b
            }
            for(i=0;i<5;i++)for(j=i+1;j<5;j++)if(ratio[j]<ratio[i]) {
                tmp=ratio[i];ratio[i]=ratio[j];ratio[j]=tmp
            }
            verdict=ratio[2]<=1.05?"within-band":"investigate"
            if(verdict=="investigate")failed=1
            printf "%s\t%s\t%.4f\t%.4f\t%.4f\t%s\n",cell,ref,ratio[2],ratio[0],ratio[4],verdict
        }
        exit failed
    }' "$out/means.tsv" > "$out/summary.tsv" && result=0 || result=$?
cat "$out/summary.tsv"
find "$out" -type f ! -name manifest.sha256 -exec shasum -a 256 {} + > "$out/manifest.sha256"
exit "$result"
