#!/bin/sh
# Delivered compute core versus repaired historical controls. Existing FIR
# attribution panel; ordinary command and other workloads have separate jobs.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT}"
CC=${CC:-/usr/bin/clang}
unset WF_SPLIT_WORK WF_STACKS
WF_SCHED_REPORT=0
export WF_SCHED_REPORT
# Observe the already measured images externally; never relink them for this
# diagnostic or substitute sampled runs for the ordinary timing matrix.
if test "${1:-screen}" = profile; then
    test "$(uname -s)" = Linux
    out=$(cd "$OUT/formal-screen" && pwd)
    test -s "$out/manifest.sha256"
    profile=$out/profile
    mkdir -p "$profile"
    if test -e "$profile/inputs.txt" || test -e "$profile/commands.txt"; then
        echo 'profile already contains an attempt; use a fresh formal screen' >&2
        exit 2
    fi
    profile_manifest() {
        find "$profile" -type f ! -name manifest.sha256 -exec sha256sum {} + > "$profile/manifest.sha256"
    }
    trap profile_manifest 0
    : > "$profile/images.sha256"
    for image in old recovered candidate replica; do
        awk -v path="$out/$image" '
            substr($0,67)==path {print; found++}
            END {if(found!=1)exit 1}' "$out/manifest.sha256" >> "$profile/images.sha256"
    done
    sha256sum -c "$profile/images.sha256" > "$profile/images-check.txt"
    perf=${PERF:-perf}
    case "$(uname -m)" in
        x86_64) n=4096; tile=64; calls=4096;;
        aarch64) n=65536; tile=1024; calls=512;;
        *) printf '%s\n' 'unsupported profiling host' > "$profile/availability.txt"; exit 0;;
    esac
    {
        printf 'kernel=wf k=16 n=%s tile=%s calls=%s seed=92821 workers=1,4 passes=5\n' "$n" "$tile" "$calls"
        printf '%s\n' 'unchanged images; every CPU-sampled process has a preceding plain process' \
            'cpu-clock samples attribute user/kernel CPU, not off-CPU wait duration or hardware stalls' \
            'sample CPU and guest topology describe observed placement, not exact migrations or host-core contention' \
            'sampled timings are descriptive; no observer cost is subtracted from the original screen'
        uname -a
        for path in /proc/sys/kernel/perf_event_paranoid /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; fi
        done
        for path in /sys/devices/system/cpu/cpu[0-9]*/topology/thread_siblings_list \
                    /sys/devices/system/cpu/cpu[0-9]*/topology/core_id \
                    /sys/devices/system/cpu/cpu[0-9]*/topology/physical_package_id; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; fi
        done
        sha256sum "$out/old" "$out/recovered" "$out/candidate" "$out/replica"
    } > "$profile/inputs.txt"
    if ! "$perf" version > "$profile/perf-version.txt" 2>&1; then
        printf '%s\n' 'perf unavailable' > "$profile/availability.txt"
        exit 0
    fi
    if test "$(getconf _NPROCESSORS_ONLN)" -lt 4; then
        printf '%s\n' 'four native CPUs unavailable' > "$profile/availability.txt"
        exit 0
    fi
    if ! "$perf" record -e cpu-clock -F 997 --sample-cpu -o "$profile/probe.data" -- true \
        > "$profile/probe.log" 2>&1; then
        printf '%s\n' 'cpu-clock sampling unavailable; see probe.log' > "$profile/availability.txt"
        exit 0
    fi
    printf '%s\n' 'cpu-clock sampling available' > "$profile/availability.txt"
    : > "$profile/commands.txt"
    for width in 1 4; do
        pass=0
        while test "$pass" -lt 5; do
            order='old recovered candidate replica'
            if test "$((pass % 2))" = 1; then order='replica candidate recovered old'; fi
            for image in $order; do
                stem=$profile/$image-w$width-p$pass
                printf 'WF_WORKERS=%s WF_SCHED_REPORT=0 %s wf 16 %s %s %s 92821 %s\n' \
                    "$width" "$out/$image" "$n" "$tile" "$calls" "$pass" >> "$profile/commands.txt"
                WF_WORKERS="$width" "$out/$image" wf 16 "$n" "$tile" "$calls" 92821 "$pass" > "$stem.plain.tsv"
                WF_WORKERS="$width" "$perf" record -e cpu-clock -F 997 --sample-cpu -o "$stem.cpu.data" \
                    -- "$out/$image" wf 16 "$n" "$tile" "$calls" 92821 "$pass" \
                    > "$stem.sampled.tsv" 2> "$stem.cpu.log"
                for log in "$stem.plain.tsv" "$stem.sampled.tsv"; do
                    awk -F '\t' -v width="$width" -v n="$n" -v tile="$tile" -v calls="$calls" -v pass="$pass" '
                        /^# actual_lanes=/ {if($0!="# actual_lanes="width)bad=1; lanes++}
                        /^# FIR bench PASS:/ {if(index($0,"calls="(calls+1)" ")==0)bad=1; passed++}
                        $1!~/^#/ && $1!="runtime" {
                            if(NF!=12 || $2!="wf" || $3!=width || $4!=16 || $5!=n ||
                                $6!=tile || $7!=92821 || $8!=pass || $9!=rows ||
                                $10!=(rows==0?"first":"warm") || $11!~/^[0-9]+$/ ||
                                $12!~/^[0-9]+$/ || $12<$11)bad=1
                            rows++
                        }
                        END {exit bad || lanes!=1 || passed!=1 || rows!=calls+1}' "$log"
                done
                "$perf" report --stdio --no-children -i "$stem.cpu.data" > "$stem.cpu.txt"
                "$perf" script -F +cpu --ns -i "$stem.cpu.data" > "$stem.cpu-events.txt"
            done
            pass=$((pass + 1))
        done
    done
    exit 0
fi
test "${1:-screen}" = screen
: "${WFC:?set WFC to the current compiler}"
root=$(git rev-parse --show-toplevel)
old=9051576f6a4d723b4eb072850f49859853decae7
recovered=d858008f560b25da896af2a17f8b1d07ac49fd6e
previous=4fabd26448eddc9316499f5c58e66c556d9ca7cb
host=$(uname -s)
exe=; floor=wf_floor.c; leaf=prim_host.c; platform_flags=-pthread; libraries=-lm
modes='old recovered previous candidate replica'
references='old recovered previous replica'
case "$host" in
    Darwin|Linux) ;;
    MINGW*|MSYS*)
        exe=.exe; floor=wf_floor_windows.c; leaf=prim_windows.c
        platform_flags=; libraries='-lpsapi -lws2_32'
        modes='old previous candidate replica'; references='old previous replica'
        ;;
    *) echo "unsupported native host: $host" >&2; exit 2;;
esac
mkdir -p "$OUT"
mkdir "$OUT/formal-screen"
out=$(cd "$OUT/formal-screen" && pwd)
mkdir "$out/source" "$out/raw"
cp fir.wf fir_direct.wf fir_host.ll fir_bench.c fir_native.c fir_native.h fir_check.c \
    formal-screen.sh runtime.h runtime_events.h pure-compare-repairs.patch "$out/source/"
cp -R "$root/compiler/src/backend/sched" "$out/source/"
cp "$root/compiler/src/backend/$floor" "$out/source/"
cp "$WFC" "$out/whitefootc$exe"
flags="-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic $platform_flags -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto"
unaligned_flags=$flags
if test "$(uname -m)" = x86_64; then
    flags="$flags -falign-loops=32"
    modes="unaligned $modes"
    references="unaligned $references"
fi
{
    git rev-parse HEAD
    printf 'historical=%s\nrecovered=%s\nprevious=%s\nflags=%s\nmodes=%s\n' "$old" "$recovered" "$previous" "$flags" "$modes"
    uname -a
    "$CC" --version
    rustc -vV
    if test "$host" = Darwin; then sysctl hw.model hw.ncpu hw.memsize
    elif test "$host" = Linux; then lscpu; lscpu --caches
    else powershell.exe -NoProfile -Command 'Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | Format-List'; fi
} > "$out/host.txt"
cat > "$out/flags.txt" <<'TEXT'
Same scalar O3 WF object and native oracle in each scheduler comparison; ordinary
CLI also compiled at its normal optimization level and checked at each width.
On x86-64, all scheduler references share the maintained compiler's 32-byte loop
alignment. The separate unaligned image uses the current candidate sources but
recompiles WF, runtime C, host and oracle without that option. It tests the
whole host-setting change, not only the WF-object change in the prior diagnostic.
All images link the current floor/configuration/platform thread leaves.
POSIX old/recovered cores call pthread directly; candidate calls wf_prim
wrappers. Linking the same leaves does not imply the same executed call path.
The old/recovered cores preserve their own compute startup and algorithms.
Historical POSIX and recovered ring accesses receive the recorded correctness
repairs; Windows historical thief reads use its existing SC load primitive.
POSIX controls have their original global steal counter; production counts per
lane. Historical Windows has its default counters off, read-only started-worker
observation added; no unqualified research port is used. Its historical worker
configuration admits only 2..64, so the paired Windows panel uses widths 2/4;
ordinary-command CI separately covers width 1. Replica is byte-identical to
production. Raw logs retain every core/cycle duration, batch wall and CPU,
peak memory and context switches (unavailable on Windows). Five process pairs
per cell; first64 is an overlapping view, not independent extra evidence.
The prior shared-runtime layout/idle variants are retired with that scheduler;
their sources and prior artifacts remain in git and the unified checkpoint.
TEXT
# Preserve the exact pre-repair sources as well as the compiled copies.
if test -z "$exe"; then
    git show "$old:compiler/src/backend/par_runtime.c" > "$out/old.c"
    git show "$recovered:research/experiments/compute-runtime/runtime.c" > "$out/research.c"
    cp "$out/old.c" "$out/source/old.original.c"
    cp "$out/research.c" "$out/source/research.original.c"
    (cd "$out" && patch --batch -p0 < "$root/research/experiments/compute-runtime/pure-compare-repairs.patch")
    cat > "$out/control-observer.c" <<'C'
extern unsigned wf_compute_worker_count(void);
unsigned wf__sched_pool_running(void) {
    unsigned count = wf_compute_worker_count();
    return count > 1 ? count - 1 : 0;
}
unsigned wf_bench_worker_count(void) { return wf__sched_pool_running() + 1; }
C
else
    git show "$old:compiler/src/backend/par_runtime_windows.c" > "$out/source/old.original.c"
    sed -e 's/wf__par_load64_acquire(\&victim->top)/wf__par_load64_seq(\&victim->top)/' \
        -e 's/wf__par_load64_acquire(\&victim->bottom)/wf__par_load64_seq(\&victim->bottom)/' \
        -e '/^static LONG64 wf__par_load64_acquire(/,/^}/d' \
        "$out/source/old.original.c" > "$out/old.c"
    cat >> "$out/old.c" <<'C'
/* Read-only observer; historical production counters were disabled. */
unsigned wf__sched_pool_running(void) {
    return (unsigned)wf__par_load32_acquire(&wf__par_ready_workers);
}
unsigned long wf__par_grants(void) { return 0; }
C
    cat > "$out/control-observer.c" <<'C'
extern unsigned wf__sched_pool_running(void);
unsigned wf_bench_worker_count(void) { return wf__sched_pool_running() + 1; }
C
    cp "$root/compiler/src/backend/windows_runtime.c" "$root/compiler/src/backend/windows_runtime.h" "$out/source/"
    cp -R "$root/compiler/src/backend/completion" "$out/source/"
fi
# Freeze the maintained core and inline primitives as the previous control.
# Rejected slot/counter/spin-hint adjustments stay in git and measured artifacts.
git show "$previous:compiler/src/backend/sched/core.c" > "$out/previous.c"
mkdir "$out/previous-sched"
for header in core.h entry.h prim.h; do
    git show "$previous:compiler/src/backend/sched/$header" > "$out/previous-sched/$header"
done
printf '\nPrevious maintained core/headers=%s; same WF/host objects, flags and platform source implementations. The x86-64 candidate adds a power-of-two remainder fast path with the same victim sequence; previous and ARM retain the general remainder. Both use the internal candidate label; filenames and means.tsv distinguish their process samples.\n' "$previous" >> "$out/flags.txt"
cat > "$out/candidate-observer.c" <<'C'
extern unsigned wf__sched_pool_running(void);
unsigned wf_bench_worker_count(void) { return wf__sched_pool_running() + 1; }
C
"$WFC" --par --no-vectorize --emit-llvm fir.wf fir_direct.wf -o "$out/module.ll"
"$WFC" --par --no-vectorize fir.wf fir_direct.wf -o "$out/command$exe"
sed -e 's/@main(/@wf_research_fir_command_main(/g' \
    -e 's/@wmain(/@wf_research_fir_windows_command_main(/g' \
    -e 's/@wf__main_body(/@wf_research_fir_command_body(/g' "$out/module.ll" > "$out/host.ll"
cat fir_host.ll >> "$out/host.ll"
"$CC" $flags -Wno-override-module -c "$out/host.ll" -o "$out/wf.o"
"$CC" $flags -c fir_native.c -o "$out/native.o"
"$CC" $flags -DWF_FILTER_NATIVE fir_check.c "$out/native.o" $libraries -o "$out/oracle$exe"
"$out/oracle$exe" > "$out/oracle.txt"
"$CC" $flags -DWF_RUNTIME_CONTROL '-DFIR_RUNTIME="candidate"' -c fir_bench.c -o "$out/candidate-host.o"
if test "$flags" != "$unaligned_flags"; then
    "$CC" $unaligned_flags -Wno-override-module -c "$out/host.ll" -o "$out/unaligned-wf.o"
    "$CC" $unaligned_flags -c fir_native.c -o "$out/unaligned-native.o"
    "$CC" $unaligned_flags -DWF_RUNTIME_CONTROL '-DFIR_RUNTIME="candidate"' -c fir_bench.c -o "$out/unaligned-host.o"
    printf 'unaligned_flags=%s\n' "$unaligned_flags" >> "$out/flags.txt"
fi
for mode in $modes; do
    if test "$mode" = replica; then
        cp "$out/candidate$exe" "$out/replica$exe"
        cmp "$out/candidate$exe" "$out/replica$exe"
        continue
    fi
    set -- "$out/source/$floor" "$out/source/sched/entry.c" "$out/source/sched/$leaf"
    case "$mode" in
        old) set -- "$@" "$out/old.c" "$out/control-observer.c";;
        recovered) set -- "$@" -I"$out/source" "$out/research.c" "$out/control-observer.c";;
        previous) set -- "$@" -I"$out/previous-sched" "$out/previous.c" "$out/candidate-observer.c";;
        candidate|unaligned) set -- "$@" "$out/source/sched/core.c" "$out/candidate-observer.c";;
    esac
    if test -n "$exe"; then
        for unit in windows_runtime.c completion/runtime.c completion/wait_windows.c \
            completion/file_adapter.c completion/file_windows.c completion/bridge.c completion/windows_iocp.c; do
            set -- "$@" "$out/source/$unit"
        done
    fi
    host_input=fir_bench.c
    case "$mode" in candidate|previous) host_input=$out/candidate-host.o;; esac
    link_flags=$flags; wf_input=$out/wf.o; native_input=$out/native.o
    runtime_label=$mode
    if test "$mode" = unaligned; then
        link_flags=$unaligned_flags; wf_input=$out/unaligned-wf.o
        native_input=$out/unaligned-native.o; host_input=$out/unaligned-host.o
        runtime_label=candidate
    fi
    "$CC" $link_flags -DWF_RUNTIME_CONTROL "-DFIR_RUNTIME=\"$runtime_label\"" "$host_input" \
        "$@" "$wf_input" "$native_input" $libraries -o "$out/$mode$exe"
done
if test -n "$exe"; then cpus=$NUMBER_OF_PROCESSORS; else cpus=$(getconf _NPROCESSORS_ONLN); fi
widths=1; test -z "$exe" || widths=2
if test "$cpus" -ge 2 && test -z "$exe"; then widths="$widths 2"; fi
if test "$cpus" -ge 3; then widths="$widths 3"; fi
if test "$cpus" -ge 4; then widths="$widths 4"; fi
printf 'mode\tworkers\tn\ttile\tpass\tcore_mean_ns\tcycle_mean_ns\n' > "$out/means.tsv"
cp "$out/means.tsv" "$out/short-means.tsv"
for width in $widths; do
    WF_WORKERS="$width" "$out/command$exe" > "$out/cli-w$width.txt"
    for n in 4096 65536; do
        calls=512
        if test "$n" = 4096; then calls=4096; fi
        for tile in 16 64 1024; do
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
            identical=(ref ~ /(^|-)replica$/)
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
serial_tile=
serial_modes=
if test -n "$exe"; then
    serial_tile=1024
    serial_modes='old candidate replica'
elif test "$(uname -s)-$(uname -m)" = Linux-x86_64; then
    serial_tile=64
    serial_modes='old recovered candidate replica'
fi
if test -n "$serial_tile"; then
    # Reuse the existing sequential WF entry with the same result tree as
    # the platform's measured full-call gap. Every image must run one lane
    # despite WF_WORKERS=4. Keep the parallel acceptance matrix authoritative.
    mkdir "$out/serial-control"
    printf 'mode\tworkers\tn\ttile\tpass\tcore_mean_ns\tcycle_mean_ns\n' > "$out/serial-control/means.tsv"
    pass=0
    while test "$pass" -lt 5; do
        order=$serial_modes
        if test "$((pass % 2))" = 1; then
            order=
            for mode in $serial_modes; do order="$mode $order"; done
        fi
        for mode in $order; do
            log="$out/serial-control/$mode-p$pass.tsv"
            WF_WORKERS=4 "$out/$mode$exe" wf-seq 16 4096 "$serial_tile" 4096 92821 "$pass" > "$log"
            awk '
                /^# runtime=/ {
                    headers++
                    if($3!="kernel=wf-seq" || $4!="workers_requested=4" || $5!="world=sequential")exit 1
                }
                /^# actual_lanes=/ {split($2,a,"="); seen++; if(a[2]!=1)exit 1}
                END {if(headers!=1 || seen!=1)exit 1}' "$log"
            awk -F '\t' -v mode="$mode" -v p="$pass" -v tile="$serial_tile" '
                $10=="warm" {core+=$11;cycle+=$12;calls++}
                END {
                    if(calls!=4096)exit 1
                    printf "%s\t1\t4096\t%s\t%s\t%.3f\t%.3f\n",mode,tile,p,core/calls,cycle/calls
                }' "$log" >> "$out/serial-control/means.tsv"
        done
        pass=$((pass + 1))
    done
fi
# Observe the existing counters after the measured Linux batch. Both cores
# already update them in the original images; report=2 only registers the
# exit-time read. Keep these observations separate from the timing matrix.
if test "$(uname -s)-$(uname -m)" = Linux-x86_64; then
    mkdir "$out/scheduler-observation"
    printf 'mode\tpass\tcalls\tsteals\n' > "$out/scheduler-observation/counts.tsv"
    pass=0
    while test "$pass" -lt 5; do
        order='old recovered previous candidate replica'
        if test "$((pass % 2))" = 1; then order='replica candidate previous recovered old'; fi
        for mode in $order; do
            log="$out/scheduler-observation/$mode-p$pass"
            WF_WORKERS=4 WF_SCHED_REPORT=2 "$out/$mode" wf 16 4096 64 4096 92821 "$pass" > "$log.tsv" 2> "$log.stderr"
            awk '
                /^# actual_lanes=/ {if($2!="actual_lanes=4")exit 1; lanes++}
                /^# FIR bench PASS:/ {if($5!="calls=4097")exit 1; passed++}
                END {if(lanes!=1 || passed!=1)exit 1}' "$log.tsv"
            awk -v mode="$mode" -v pass="$pass" '
                /^compute:/ {
                    if(NF!=5 || $2!="threads=4" || $3!="workers_started=3" ||
                        $4!~/^steals=[0-9]+$/ || $5!="slots_per_lane=64")exit 1
                    split($4,a,"="); steals=a[2]; reports++
                }
                END {
                    if(reports!=1 || steals>4097*63)exit 1
                    printf "%s\t%s\t4097\t%s\n",mode,pass,steals
                }' "$log.stderr" >> "$out/scheduler-observation/counts.tsv"
        done
        pass=$((pass + 1))
    done
fi
# Binaries, copied sources, actual flags and every raw sample are reproducible
# evidence even when the performance band fails.
if test -n "$exe"; then
    find "$out" -type f ! -name manifest.sha256 -exec sha256sum {} + > "$out/manifest.sha256"
else
    find "$out" -type f ! -name manifest.sha256 -exec shasum -a 256 {} + > "$out/manifest.sha256"
fi
test -s "$out/manifest.sha256"
exit "$result"
