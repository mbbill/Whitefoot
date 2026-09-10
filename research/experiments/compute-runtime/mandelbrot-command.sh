#!/bin/sh
# Ordinary CLI qualification and whole-command measurement preparation.
# Called by the existing compute experiment Makefile; retire with this workload.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT}"
: "${WFC:?set WFC}"
CC=${CC:-/usr/bin/clang}
CXX=${CXX:-/usr/bin/clang++}
mode=${1:-check}
case "$mode" in build|check|screen|summarize|diagnose|profile) ;; *) exit 2;; esac
# This panel owns the policy matrix; inherited tuning must not alter a cell.
unset WF_SPLIT_WORK PLACEMENT_NICE PLACEMENT_REPORT
WF_SCHED_REPORT=0
export WF_SCHED_REPORT
mkdir -p "$OUT/mandelbrot-command"
out=$(cd "$OUT/mandelbrot-command" && pwd)
exe=
thread_flags=-pthread
runner_flags=
case "$(uname -s)" in
    MINGW*|MSYS*) exe=.exe; thread_flags=; runner_flags='-municode -lpsapi';;
esac
scalar_flags='-O2 -g -Wall -Wextra -Werror -Wpedantic -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto'
if test "$(uname -m)" = x86_64; then scalar_flags="$scalar_flags -falign-loops=32"; fi
placement_available() {
    # Online CPUs can exceed a container's actual permitted affinity mask.
    taskset -pc $$ | awk '
        {n=split($NF, ranges, ","); total=0
         for(i=1;i<=n;i++) {
             if(ranges[i]!~/^[0-9]+(-[0-9]+)?$/) {bad=1; exit}
             m=split(ranges[i], ends, "-")
             total += m==1 ? 1 : ends[2]-ends[1]+1
         }}
        END {exit bad || total<4}'
}
verify_placement() {
    awk -F '\t' '
        NR==1 {next}
        {if(NF!=12 || $1!~/^(32|256)$/ || $2!~/^[0-4]$/ ||
            $3!~/^(work60000|work120000|static)$/ || $4!~/^[01]$/ || $5!~/^[01]$/) bad=1
         key=$1 SUBSEP $2 SUBSEP $3 SUBSEP $4 SUBSEP $5
         if(seen[key]++) bad=1
         for(i=6;i<=12;i++) if($i!~/^[0-9]+$/) bad=1
         if($6<=0 || $9<=0 || $12!=0) bad=1}
        END {exit bad || NR!=121}' "$1"
}
verify_report() {
    awk -v threads="$2" -v started="$3" '
        BEGIN {split("threads workers_started steals slots_per_lane", keys, " ")}
        {sub(/\r$/, "")}
        NF!=5 || $1!="compute:" || $2!=("threads=" threads) {bad=1}
        started!="any" && $3!=("workers_started=" started) {bad=1}
        {for(i=2;i<=NF;i++) if($i!~("^" keys[i-1] "=[0-9]+$")) bad=1}
        END {exit bad || NR!=1}
    ' "$1"
}
if test "$mode" = build; then
    "$WFC" --par --no-vectorize mandelbrot.wf mandelbrot_command.wf -o "$out/par$exe"
    "$WFC" --no-overlap --no-vectorize mandelbrot.wf mandelbrot_command.wf -o "$out/seq$exe"
    cp "$out/par$exe" "$out/replica$exe"
    cmp "$out/par$exe" "$out/replica$exe"
    # Word splitting is intentional for these fixed compiler argument lists.
    "$CXX" -std=c++17 $scalar_flags $thread_flags mandelbrot_command.cpp -o "$out/native$exe"
    "$CC" -std=c11 $scalar_flags command_runner.c $runner_flags -o "$out/runner$exe"
    if test "$(uname -s)" = Linux; then
        "$CC" -std=c11 $scalar_flags -fPIC -shared -pthread thread-placement.c \
            -ldl -o "$out/thread-placement.so"
    fi
    if test -z "$exe"; then
        "$CXX" -std=c++17 $scalar_flags $thread_flags -fsanitize=address,undefined \
            -fno-sanitize-recover=all mandelbrot_command.cpp -o "$out/native-asan"
        "$CXX" -std=c++17 $scalar_flags $thread_flags -fsanitize=thread \
            mandelbrot_command.cpp -o "$out/native-tsan"
    fi
    {
        git rev-parse HEAD
        "$CXX" --version
        printf '%s\n' "native=$scalar_flags $thread_flags" \
            'WF: ordinary --par or --no-overlap with --no-vectorize at default -O2; no private ABI or runtime override' \
            'policy controls use the identical par image with WF_SPLIT_WORK=60000,240000,0; baseline and replica use the unset default' \
            'shape4/count4096/workers4 additionally uses work120000: exactly four leaves at measured weight219, isolating decomposition before runtime changes' \
            'native serial: kernel/command reference; native static: persistent equal contiguous partitions, caller participates, pause/yield idle policy, startup and shutdown charged' \
            'static partitions are a strong regular-work reference, not a dynamic scheduling ceiling for skewed work' \
            'oracle mode checks the native point kernel against an independent volatile binary64 recurrence and known orbits; timed commands check an ordered 64-bit digest' \
            'command runner: process wall, child CPU and peak memory; POSIX context switches, unavailable on Windows'
    } > "$out/flags.txt"
    exit 0
fi
if test "$mode" = profile; then
    # External observation of the unchanged ordinary CLI image. These runs
    # are attribution, never replacements for the unobserved screen above.
    test "$(uname -s)" = Linux
    mkdir -p "$out/profile"
    profile=$out/profile
    perf=${PERF:-perf}
    {
        printf '%s\n' 'shape=4 count=4096 limit=256 seed=92821 workers=4' \
            'cpu samples: 256 repetitions; scheduling trace: original 32 repetitions' \
            'plain runner envelopes accompany each observer; observer perturbation is not subtracted' \
            'sched trace is system-wide on this isolated CI runner; use workload PIDs when interpreting other tasks'
        uname -a
        for path in /proc/sys/kernel/perf_event_paranoid /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; fi
        done
        sha256sum "$out/par" "$out/native" "$out/runner"
    } > "$profile/inputs.txt"
    if ! "$perf" version > "$profile/perf-version.txt" 2>&1; then
        printf '%s\n' 'perf unavailable' > "$profile/availability.txt"
        exit 0
    fi
    perf=$(command -v "$perf")
    sudo=$(command -v sudo || true)
    # perf creates root-only data files when recording scheduling events.
    # Return those experiment files to the runner even after a later failure,
    # so artifact upload can preserve the measurements and probe diagnostics.
    cleanup_profile() {
        if test -n "$sudo"; then
            for data in "$profile"/*.data; do
                if test -e "$data" && ! test -r "$data"; then
                    "$sudo" -n chown "$(id -u):$(id -g)" "$data"
                fi
            done
        fi
    }
    trap cleanup_profile 0
    if test "$(getconf _NPROCESSORS_ONLN)" -lt 4; then
        printf '%s\n' 'four native CPUs unavailable' > "$profile/availability.txt"
        exit 0
    fi
    cpu_available=0; sched_available=0
    if "$perf" record -e cpu-clock -F 997 -o "$profile/cpu-probe.data" -- true \
        > "$profile/cpu-probe.log" 2>&1; then cpu_available=1; fi
    if test -n "$sudo" && "$sudo" -n "$perf" record -a -e sched:sched_switch -e sched:sched_wakeup \
        -o "$profile/sched-probe.data" -- true > "$profile/sched-probe.log" 2>&1; then sched_available=1; fi
    printf 'cpu_samples=%s\nscheduling_trace=%s\n' "$cpu_available" "$sched_available" > "$profile/availability.txt"
    for repetitions in 32 256; do
        expected=$("$out/native" oracle 4 4096 256 "$repetitions" 92821)
        for form in work60000 work120000 static; do
            work=1200000
            case "$form" in
                work60000) work=60000; set -- "$out/par";;
                work120000) work=120000; set -- "$out/par";;
                static) set -- "$out/native" static;;
            esac
            set -- "$@" 4 4096 256 "$repetitions" 92821 "$expected"
            stem=$profile/$form-r$repetitions
            WF_WORKERS=4 WF_SPLIT_WORK=$work "$out/runner" "$@" > "$stem.plain.tsv"
            if test "$repetitions" = 256 && test "$cpu_available" = 1; then
                WF_WORKERS=4 WF_SPLIT_WORK=$work "$out/runner" "$perf" record \
                    -e cpu-clock -F 997 -o "$stem.cpu.data" -- "$@" \
                    > "$stem.sampled.tsv" 2> "$stem.cpu.log"
                "$perf" report --stdio --no-children -i "$stem.cpu.data" > "$stem.cpu.txt"
                "$perf" script -i "$stem.cpu.data" > "$stem.cpu-events.txt"
            fi
            if test "$repetitions" = 32 && test "$sched_available" = 1; then
                "$out/runner" "$sudo" -n "$perf" record -a \
                    -e sched:sched_switch -e sched:sched_wakeup -o "$stem.sched.data" \
                    -- env WF_WORKERS=4 WF_SPLIT_WORK=$work WF_SCHED_REPORT=0 "$@" \
                    > "$stem.traced.tsv" 2> "$stem.sched.log"
                "$sudo" -n "$perf" script -i "$stem.sched.data" > "$stem.sched.txt"
            fi
        done
    done
    # Also verify the stored rows: no profile may conceal a failed command
    # or replace the expected single runner observation with extra output.
    awk -F '\t' -v rows="$((6+3*cpu_available+3*sched_available))" \
        'FNR!=1 || NF!=7 || $1<=0 || $7!=0 {bad=1} END {exit bad || NR!=rows}' "$profile"/*.tsv
    # Test CPU placement without replacing or relinking the actual programs.
    # The original unwrapped observations above remain separate. On/off and
    # A/A runs below use the same wrapper within each program; the native
    # main participates directly, whereas WF creates a command thread.
    placement_condition() {
        bound=$condition; nice=keep
        if test "$panel" = priority; then
            bound=1; nice=0
            if test "$condition" = 1; then nice=-10; fi
        fi
    }
    placement_command() {
        if test "$panel" = priority; then "$sudo" -n "$@"; else "$@"; fi
    }
    placement_panel() {
        placement=$profile/$panel
        mkdir -p "$placement"
        if ! placement_available; then
            printf '%s\n' 'four allowed CPUs unavailable; placement control skipped' > "$placement/availability.txt"
            return
        fi
        taskset -pc $$ > "$placement/availability.txt"
        env_command=$(command -v env)
        cp thread-placement.c "$placement/"
        sha256sum "$out/thread-placement.so" >> "$profile/inputs.txt"
        if test "$panel" = priority; then
            if test -z "$sudo" || ! "$sudo" -n true > "$placement/permission.stdout" 2> "$placement/permission.stderr"; then
                printf '%s\n' 'sudo unavailable; priority control skipped' >> "$placement/availability.txt"
                return
            fi
            probe_expected=$("$out/native" oracle 4 257 128 3 92821)
            if "$sudo" -n "$env_command" WF_WORKERS=4 \
                LD_PRELOAD="$out/thread-placement.so" PLACEMENT_BIND=1 PLACEMENT_MAIN=caller \
                PLACEMENT_REPORT=1 PLACEMENT_NICE=-10 "$out/native" static 4 257 128 3 92821 "$probe_expected" \
                > "$placement/probe.stdout" 2> "$placement/probe.stderr"; then
                test ! -s "$placement/probe.stdout"
            else
                printf 'thread placement: cannot establish requested priority\n' > "$placement/permission.expected"
                if cmp -s "$placement/probe.stderr" "$placement/permission.expected"; then
                    printf '%s\n' 'requested nice=-10 unavailable; priority control skipped' >> "$placement/availability.txt"
                    return
                fi
                cat "$placement/probe.stderr" >&2
                return 1
            fi
        fi
        printf '%s\n' 'condition: placement 0/1 means unbound/bound with inherited priority; priority 0/1 means bound nice=0/-10' \
            'priority panel: both conditions execute the measurement runner as root via sudo before its timer; placement panel runs as the invoking user' \
            'all scheduling traces run as root; perf itself retains its inherited priority' > "$placement/conditions.txt"
        printf 'repetitions\tpass\tform\tcondition\treplica\twall_ns\tuser_ns\tsystem_ns\trss_bytes\tvoluntary\tinvoluntary\tstatus\n' > "$placement/processes.tsv"
        for repetitions in 32 256; do
            expected=$("$out/native" oracle 4 4096 256 "$repetitions" 92821)
            for pass in 0 1 2 3 4; do
                forms='work60000 work120000 static'; conditions='0 1'
                if test "$((pass%2))" = 1; then forms='static work120000 work60000'; conditions='1 0'; fi
                for form in $forms; do
                    work=1200000; role=launcher
                    case "$form" in
                        work60000) work=60000; set -- "$out/par";;
                        work120000) work=120000; set -- "$out/par";;
                        static) role=caller; set -- "$out/native" static;;
                    esac
                    set -- "$@" 4 4096 256 "$repetitions" 92821 "$expected"
                    for condition in $conditions; do
                        placement_condition
                        for replica in 0 1; do
                            stem=$placement/$form-r$repetitions-p$pass-c$condition-a$replica
                            placement_command "$out/runner" "$env_command" WF_WORKERS=4 WF_SPLIT_WORK=$work \
                                WF_SCHED_REPORT=0 LD_PRELOAD="$out/thread-placement.so" \
                                PLACEMENT_BIND=$bound PLACEMENT_MAIN=$role PLACEMENT_NICE=$nice PLACEMENT_REPORT=0 \
                                "$@" > "$stem.tsv" 2> "$stem.stderr"
                            test ! -s "$stem.stderr"
                            printf '%s\t%s\t%s\t%s\t%s\t' "$repetitions" "$pass" "$form" "$condition" "$replica" >> "$placement/processes.tsv"
                            cat "$stem.tsv" >> "$placement/processes.tsv"
                        done
                    done
                done
            done
        done
        # Observe after all timed pairs so perf cannot perturb their ordering.
        expected=$("$out/native" oracle 4 4096 256 32 92821)
        forms='work60000 work120000 static'; conditions='0 1'
        if test "$panel" = priority; then forms='static work120000 work60000'; conditions='1 0'; fi
        for form in $forms; do
            work=1200000; role=launcher
            case "$form" in
                work60000) work=60000; set -- "$out/par";;
                work120000) work=120000; set -- "$out/par";;
                static) role=caller; set -- "$out/native" static;;
            esac
            set -- "$@" 4 4096 256 32 92821 "$expected"
            for condition in $conditions; do
                placement_condition
                stem=$placement/$form-c$condition
                placement_command "$env_command" WF_WORKERS=4 WF_SPLIT_WORK=$work WF_SCHED_REPORT=0 \
                    LD_PRELOAD="$out/thread-placement.so" PLACEMENT_BIND=$bound \
                    PLACEMENT_MAIN=$role PLACEMENT_NICE=$nice PLACEMENT_REPORT=1 "$@" \
                    > "$stem.stdout" 2> "$stem.assignment.txt"
                test ! -s "$stem.stdout"
                awk '/^placement: / {n++} END {exit n!=4}' "$stem.assignment.txt"
                if test "$sched_available" = 1; then
                    "$sudo" -n "$perf" record -a -e sched:sched_switch \
                        -e sched:sched_wakeup -e sched:sched_wakeup_new \
                        -o "$profile/$panel-$form-c$condition.data" -- "$env_command" \
                        WF_WORKERS=4 WF_SPLIT_WORK=$work WF_SCHED_REPORT=0 \
                        LD_PRELOAD="$out/thread-placement.so" PLACEMENT_BIND=$bound \
                        PLACEMENT_MAIN=$role PLACEMENT_NICE=$nice PLACEMENT_REPORT=1 "$@" \
                        > "$stem.traced.stdout" 2> "$stem.traced.log"
                    "$sudo" -n "$perf" script -i "$profile/$panel-$form-c$condition.data" \
                        > "$stem.sched.txt"
                fi
            done
        done
        awk -F '\t' '{for(i=1;i<=NF;i++) if($i!~/^[0-9]+$/) bad=1}
            FNR!=1 || NF!=7 || $1<=0 || $4<=0 || $7!=0 {bad=1} END {exit bad || NR!=120}' \
            "$placement"/*-a[01].tsv
        verify_placement "$placement/processes.tsv"
    }
    for panel in placement priority; do placement_panel; done
    exit 0
fi
if test "$mode" = diagnose; then
    mkdir "$out/diagnostics"
    if test -n "$exe"; then cpus=$NUMBER_OF_PROCESSORS; else cpus=$(getconf _NPROCESSORS_ONLN); fi
    widths=1
    if test "$cpus" -ge 2; then widths="$widths 2"; fi
    if test "$cpus" -ge 4; then widths="$widths 4"; fi
    printf 'shape\tcount\tworkers\tsplit_work\tlimit\trepetitions\tseed\texpected\n' > "$out/diagnostics/inputs.tsv"
    for shape in 0 1 2 3 4 5 6; do
        for count in 4096 65536; do
            repetitions=32
            if test "$count" = 65536; then repetitions=2; fi
            expected=$("$out/native$exe" oracle "$shape" "$count" 256 "$repetitions" 92821)
            for workers in $widths; do
                work_values='0 60000 240000 1200000'
                if test "$shape/$count/$workers" = 4/4096/4; then
                    work_values="$work_values 120000"
                fi
                for work in $work_values; do
                    log="$out/diagnostics/s$shape-n$count-w$workers-work$work"
                    WF_WORKERS=$workers WF_SPLIT_WORK=$work WF_SCHED_REPORT=2 \
                        "$out/par$exe" "$shape" "$count" 256 "$repetitions" 92821 "$expected" \
                        > "$log.stdout" 2> "$log.stderr"
                    test ! -s "$log.stdout"
                    verify_report "$log.stderr" "$workers" any
                    printf '%s\t%s\t%s\t%s\t256\t%s\t92821\t%s\n' \
                        "$shape" "$count" "$workers" "$work" "$repetitions" "$expected" >> "$out/diagnostics/inputs.tsv"
                done
            done
        done
    done
    printf 'Ordinary-command scheduler diagnostics complete: %s\n' "$out/diagnostics"
    exit 0
fi
if test "$mode" = summarize; then
    awk -v expected_workers="$(cat "$out/screen/planned-workers.txt")" \
        -f mandelbrot-command.awk "$out/screen/processes.tsv" > "$out/screen/summary.tsv"
    cat "$out/screen/summary.tsv"
    exit 0
fi
if test "$mode" = screen; then
    mkdir "$out/screen"
    mkdir "$out/screen/raw" "$out/screen/source"
    cp mandelbrot.wf mandelbrot_command.wf mandelbrot_command.cpp command_runner.c \
        mandelbrot-command.sh mandelbrot-command.awk "$out/screen/source/"
    cp "$out/flags.txt" "$out/screen/flags.txt"
    {
        uname -a
        if test -n "$exe"; then
            powershell.exe -NoProfile -Command 'Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | Format-List'
        elif test "$(uname -s)" = Darwin; then
            sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
        else
            lscpu
        fi
    } > "$out/screen/host.txt"
    if test -n "$exe"; then cpus=$NUMBER_OF_PROCESSORS; else cpus=$(getconf _NPROCESSORS_ONLN); fi
    widths=1
    if test "$cpus" -ge 2; then widths="$widths 2"; fi
    if test "$cpus" -ge 4; then widths="$widths 4"; fi
    planned_workers=1
    for workers in $widths; do planned_workers=$workers; done
    printf '%s\n' "$planned_workers" > "$out/screen/planned-workers.txt"
    printf 'shape\tcount\tlimit\trepetitions\tpass\tform\trequested_workers\twall_ns\tuser_ns\tsystem_ns\trss_bytes\tvoluntary\tinvoluntary\tstatus\n' > "$out/screen/processes.tsv"
    printf 'shape\tcount\tlimit\trepetitions\tseed\texpected\n' > "$out/screen/oracles.tsv"
    for shape in 0 1 2 3 4 5 6; do
        for count in 4096 65536; do
            repetitions=32
            if test "$count" = 65536; then repetitions=2; fi
            limit=256
            expected=$("$out/native$exe" oracle "$shape" "$count" "$limit" "$repetitions" 92821)
            printf '%s\t%s\t%s\t%s\t92821\t%s\n' "$shape" "$count" "$limit" "$repetitions" "$expected" >> "$out/screen/oracles.tsv"
            for workers in $widths; do
                pass=0
                while test "$pass" -lt 5; do
                    order='par replica work60000 work240000 nosplit static'
                    if test "$shape/$count/$workers" = 4/4096/4; then
                        order="$order work120000"
                    fi
                    if test "$workers" = 1; then order="seq serial $order"; fi
                    if test "$((pass%2))" = 1; then
                        reversed=
                        for form in $order; do reversed="$form $reversed"; done
                        order=$reversed
                    fi
                    for form in $order; do
                        case "$form" in
                            serial|static) set -- "$out/native$exe" "$form";;
                            work60000|work120000|work240000|nosplit) set -- "$out/par$exe";;
                            *) set -- "$out/$form$exe";;
                        esac
                        set -- "$@" "$shape" "$count" "$limit" "$repetitions" 92821 "$expected"
                        log="$out/screen/raw/$form-w$workers-s$shape-n$count-p$pass.tsv"
                        (
                            unset WF_SPLIT_WORK
                            case "$form" in
                                work60000) export WF_SPLIT_WORK=60000;;
                                work120000) export WF_SPLIT_WORK=120000;;
                                work240000) export WF_SPLIT_WORK=240000;;
                                nosplit) export WF_SPLIT_WORK=0;;
                            esac
                            WF_WORKERS=$workers "$out/runner$exe" "$@"
                        ) > "$log"
                        awk -F '\t' '{sub(/\r$/, "")} NF!=7 || $1<=0 || $7!=0 {bad=1} END {exit bad || NR!=1}' "$log"
                        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t' "$shape" "$count" "$limit" "$repetitions" "$pass" "$form" "$workers" >> "$out/screen/processes.tsv"
                        tr -d '\r' < "$log" >> "$out/screen/processes.tsv"
                    done
                    pass=$((pass+1))
                done
            done
        done
    done
    if test -n "$exe"; then
        sha256sum "$WFC" "$out"/*.exe "$out/screen/source/"* > "$out/screen/manifest.sha256"
    else
        shasum -a 256 "$WFC" "$out/par" "$out/seq" "$out/replica" "$out/native" "$out/runner" "$out/screen/source/"* > "$out/screen/manifest.sha256"
    fi
    printf 'Mandelbrot ordinary-command screen complete: %s\n' "$out/screen/processes.tsv"
    exec sh mandelbrot-command.sh summarize
fi
mkdir -p "$out/check"
expect_status() {
    expected_status=$1
    shift
    actual_status=0
    "$@" > "$out/check/stdout.txt" 2> "$out/check/stderr.txt" || actual_status=$?
    test "$actual_status" = "$expected_status"
    test ! -s "$out/check/stdout.txt"
    test ! -s "$out/check/stderr.txt"
}
for image in par seq native; do expect_status 0 "$out/$image$exe"; done
printf 'shape\tcount\tlimit\trepetitions\tseed\texpected\n' > "$out/check/oracles.tsv"
verify_case() {
    shape=$1 count=$2 limit=$3 repetitions=$4 seed=$5
    expected=$("$out/native$exe" oracle "$shape" "$count" "$limit" "$repetitions" "$seed")
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$shape" "$count" "$limit" "$repetitions" "$seed" "$expected" >> "$out/check/oracles.tsv"
    expect_status 0 "$out/seq$exe" "$shape" "$count" "$limit" "$repetitions" "$seed" "$expected"
    expect_status 0 "$out/native$exe" serial "$shape" "$count" "$limit" "$repetitions" "$seed" "$expected"
    for workers in 1 2 4; do
        for split_work in 0 60000 240000 1200000; do
            WF_WORKERS=$workers WF_SPLIT_WORK=$split_work expect_status 0 "$out/par$exe" "$shape" "$count" "$limit" "$repetitions" "$seed" "$expected"
        done
        WF_WORKERS=$workers expect_status 0 "$out/native$exe" static "$shape" "$count" "$limit" "$repetitions" "$seed" "$expected"
    done
}
for shape in 0 1 2 3 4 5 6; do
    for count in 0 7 257; do
        for seed in 0 18446744073709551615; do verify_case "$shape" "$count" 128 3 "$seed"; done
    done
done
for limit in 0 1 65536; do verify_case 5 1 "$limit" 2 92821; done
verify_case 0 257 128 0 92821
verify_case 1 4096 512 2 92821
verify_case 6 4096 1024 2 92821
for image in par seq; do
    expect_status 1 "$out/$image$exe" 0 0 0 0 0 0
    for invalid in '' x +1 -1 18446744073709551616 000000000000000000000; do
        expect_status 2 "$out/$image$exe" 0 "$invalid" 128 1 0 0
        expect_status 2 "$out/native$exe" static 0 "$invalid" 128 1 0 0
    done
    expect_status 2 "$out/$image$exe" 7 0 0 0 0 0
    expect_status 2 "$out/$image$exe" 0 1048577 0 0 0 0
    expect_status 2 "$out/$image$exe" 0 0 65537 0 0 0
    expect_status 2 "$out/$image$exe" 0 0 0 4097 0 0
done
for invalid in -1 no 1000000001 18446744073709551616; do
    actual_status=0
    WF_SPLIT_WORK=$invalid "$out/par$exe" > "$out/check/stdout.txt" 2> "$out/check/stderr.txt" || actual_status=$?
    test "$actual_status" = 1
    test ! -s "$out/check/stdout.txt"
    tr -d '\r' < "$out/check/stderr.txt" > "$out/check/diagnostic.txt"
    printf 'whitefoot scheduler: WF_SPLIT_WORK must be an integer from 0 through 1000000000\n' > "$out/check/expected-diagnostic.txt"
    cmp "$out/check/diagnostic.txt" "$out/check/expected-diagnostic.txt"
done
# The normal executable must report configured/started workers without an
# observer library. A positive tiny work threshold ensures this small loop
# actually reaches the worker-start path, regardless of who steals its tasks.
expected=$("$out/native$exe" oracle 4 257 128 3 92821)
for workers in 1 4; do
    log="$out/check/report-w$workers"
    WF_WORKERS=$workers WF_SPLIT_WORK=1 WF_SCHED_REPORT=2 \
        "$out/par$exe" 4 257 128 3 92821 "$expected" > "$log.stdout" 2> "$log.stderr"
    test ! -s "$log.stdout"
    verify_report "$log.stderr" "$workers" "$((workers-1))"
done
if test "$(uname -s)" = Linux && placement_available; then
    printf '%s\n' 'four allowed CPUs available' > "$out/check/placement-availability.txt"
    # The measurement shim must preserve real program results and reject a
    # process without the promised four participants, rather than fake width.
    for bound in 0 1; do
        WF_WORKERS=4 WF_SPLIT_WORK=1 LD_PRELOAD="$out/thread-placement.so" \
            PLACEMENT_BIND=$bound PLACEMENT_MAIN=launcher \
            "$out/par" 4 257 128 3 92821 "$expected"
        WF_WORKERS=4 LD_PRELOAD="$out/thread-placement.so" \
            PLACEMENT_BIND=$bound PLACEMENT_MAIN=caller \
            "$out/native" static 4 257 128 3 92821 "$expected"
        actual_status=0
        LD_PRELOAD="$out/thread-placement.so" PLACEMENT_BIND=$bound PLACEMENT_MAIN=caller \
            /usr/bin/true > "$out/check/placement.stdout" 2> "$out/check/placement.stderr" || actual_status=$?
        test "$actual_status" = 2
        printf 'thread placement: expected exactly four participants\n' > "$out/check/placement.expected"
        cmp "$out/check/placement.expected" "$out/check/placement.stderr"
    done
elif test "$(uname -s)" = Linux; then
    printf '%s\n' 'four allowed CPUs unavailable; placement control skipped' > "$out/check/placement-availability.txt"
fi
if test -z "$exe"; then
    for sanitizer in asan tsan; do
        for shape in 0 1 6; do
            expected=$("$out/native$exe" oracle "$shape" 257 128 3 92821)
            WF_WORKERS=4 expect_status 0 "$out/native-$sanitizer" static "$shape" 257 128 3 92821 "$expected"
        done
    done
fi
# Exercise the process runner with a command whose exit code is known.
"$out/runner$exe" "$out/par$exe" > "$out/check/runner.tsv"
awk -F '\t' '{sub(/\r$/, "")} NF!=7 || $1<=0 || $2<0 || $3<0 || $4<=0 || $7!=0 {bad=1} END {exit bad || NR!=1}' "$out/check/runner.tsv"
if "$out/runner$exe" "$out/par$exe" 0 0 0 0 0 0 > "$out/check/runner-wrong.tsv"; then exit 1; fi
awk -F '\t' '{sub(/\r$/, "")} NF!=7 || $7!=1 {bad=1} END {exit bad || NR!=1}' "$out/check/runner-wrong.tsv"
printf 'Mandelbrot ordinary-command qualification PASS: 48 input cases, scalar WF/serial/static, workers 1/2/4, split work 0/60000/240000/1200000; %s\n' "$out/check"
