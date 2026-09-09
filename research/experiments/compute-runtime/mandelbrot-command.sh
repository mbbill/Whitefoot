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
case "$mode" in build|check|screen|summarize) ;; *) exit 2;; esac
# This panel owns the policy matrix; inherited tuning must not alter a cell.
unset WF_SPLIT_WORK
mkdir -p "$OUT/mandelbrot-command"
out=$(cd "$OUT/mandelbrot-command" && pwd)
exe=
thread_flags=-pthread
runner_flags=
case "$(uname -s)" in
    MINGW*|MSYS*) exe=.exe; thread_flags=; runner_flags='-municode -lpsapi';;
esac
scalar_flags='-O2 -g -Wall -Wextra -Werror -Wpedantic -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto'
if test "$mode" = build; then
    "$WFC" --par --no-vectorize mandelbrot.wf mandelbrot_command.wf -o "$out/par$exe"
    "$WFC" --no-overlap --no-vectorize mandelbrot.wf mandelbrot_command.wf -o "$out/seq$exe"
    cp "$out/par$exe" "$out/replica$exe"
    cmp "$out/par$exe" "$out/replica$exe"
    # Word splitting is intentional for these fixed compiler argument lists.
    "$CXX" -std=c++17 $scalar_flags $thread_flags mandelbrot_command.cpp -o "$out/native$exe"
    "$CC" -std=c11 $scalar_flags command_runner.c $runner_flags -o "$out/runner$exe"
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
            'native serial: kernel/command reference; native static: persistent equal contiguous partitions, caller participates, pause/yield idle policy, startup and shutdown charged' \
            'static partitions are a strong regular-work reference, not a dynamic scheduling ceiling for skewed work' \
            'oracle mode checks the native point kernel against an independent volatile binary64 recurrence and known orbits; timed commands check an ordered 64-bit digest' \
            'command runner: process wall, child CPU and peak memory; POSIX context switches, unavailable on Windows'
    } > "$out/flags.txt"
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
                    if test "$workers" = 1; then order="seq serial $order"; fi
                    if test "$((pass%2))" = 1; then
                        reversed=
                        for form in $order; do reversed="$form $reversed"; done
                        order=$reversed
                    fi
                    for form in $order; do
                        case "$form" in
                            serial|static) set -- "$out/native$exe" "$form";;
                            work60000|work240000|nosplit) set -- "$out/par$exe";;
                            *) set -- "$out/$form$exe";;
                        esac
                        set -- "$@" "$shape" "$count" "$limit" "$repetitions" 92821 "$expected"
                        log="$out/screen/raw/$form-w$workers-s$shape-n$count-p$pass.tsv"
                        (
                            unset WF_SPLIT_WORK
                            case "$form" in
                                work60000) export WF_SPLIT_WORK=60000;;
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
