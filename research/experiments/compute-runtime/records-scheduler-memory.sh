#!/bin/sh
# Validate complete functional reports and the pinned Rayon caller-worker leak.
# The explicit replay mode interprets archived Linux status supplied by a test;
# the maintained runner must use validate with the actual child exit status.
set -eu
usage() {
    echo 'usage: records-scheduler-memory.sh validate|validate-clean|replay-linux-x86_64 LOG STATUS BACKEND WIDTH CASE [EXPECTED_EXTRA_BYTES]' >&2
    exit 1
}
test "$#" -eq 6 || test "$#" -eq 7 || usage
mode=$1; log=$2; status=$3; backend=$4; width=$5; test_case=$6; extra=${7:-0}
case "$mode" in
    validate) platform="$(uname -s)/$(uname -m)";;
    validate-clean) platform=clean;;
    replay-linux-x86_64) platform=Linux/x86_64;;
    *) usage;;
esac
case "$backend" in
    wf|wf-runtime) backend=wf-runtime; shutdown=0;;
    wf-group4|wf-runtime-group4) backend=wf-runtime-group4; shutdown=0;;
    wf-group16|wf-runtime-group16) backend=wf-runtime-group16; shutdown=0;;
    static|static-spin) backend=static-spin; shutdown=1;;
    tbb|oneTBB-v2023.1.0-auto-grain1) backend=oneTBB-v2023.1.0-auto-grain1; shutdown=0;;
    parlay|parlay-native-grain1) backend=parlay-native-grain1; shutdown=1;;
    rayon-join|rayon-1.12.0-join) backend=rayon-1.12.0-join; binary=rayon-join; shutdown=0;;
    rayon-iter|rayon-1.12.0-par-iter) backend=rayon-1.12.0-par-iter; binary=rayon-iter; shutdown=0;;
    *) usage;;
esac
case "$width" in 1|2|4) ;; *) usage;; esac
case "$test_case" in
    qualify) ;;
    trace-plain|trace-identity|trace-timeline) ;;
    checked|dense|sleep-100us|sleep-1ms) test "$width" = 2 || usage;;
    *) usage;;
esac
case "$extra" in 0|257) ;; *) usage;; esac
leak=0
case "$platform/$backend" in Linux/x86_64/rayon-*) leak=1;; esac
if test "$leak" = 1; then
    test "$status" = 23 || { echo "Rayon lifecycle exit status rejected: $status" >&2; exit 1; }
else
    test "$status" = 0 && test "$extra" = 0 || {
        echo "clean sanitizer exit status rejected: $status (extra=$extra)" >&2; exit 1;
    }
fi
test -s "$log"
directory=$(CDPATH= cd "$(dirname "$0")" && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-scheduler-memory.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
# This parser consumes every diagnostic line. It writes only the functional
# prefix to scratch; the original combined stdout/stderr remains untouched.
awk -v leak="$leak" -v extra="$extra" -v binary="${binary:-}" \
    -v prefix="$scratch/prefix" -f "$directory/records-scheduler-memory.awk" "$log"
if test "${test_case#trace-}" != "$test_case"; then
    awk -F '\t' -v backend="$backend" -v width="$width" -v shape=unicode -v count=33 \
        -v limit=17 -v grain=16 -v chunks=3 -v seed=828219 -v pass=0 -v reps=2 \
        -v shutdown="$shutdown" -v level="${test_case#trace-}" -v expected_bytes=329 \
        -f "$directory/records-scheduler-trace.awk" "$scratch/prefix" > /dev/null
elif test "$test_case" = qualify; then
    awk -v backend="$backend" -v width="$width" -v shutdown="$shutdown" '
        { if (NR!=1 || $8 !~ /^capacity_waves=[1-3]$/ ||
            $0 != "record scheduler qualification PASS: backend=" backend " width=" width " actual=" width " " $8 \
                  " batches=780 outputs=270660 explicit_shutdown=" shutdown) bad=1 }
        END { if (bad || NR!=1) exit 1 }
    ' "$scratch/prefix"
else
    checks=0; gap=0
    case "$test_case" in checked) checks=1;; sleep-100us) gap=100000;; sleep-1ms) gap=1000000;; esac
    awk -F '\t' -v backend="$backend" -v width="$width" -v shape=unicode -v count=33 \
        -v limit=17 -v grain=16 -v chunks=3 -v seed=828219 -v pass=0 -v reps=2 \
        -v shutdown="$shutdown" -v expected_bytes=329 -v cadence="$test_case" \
        -v requested_gap="$gap" -v checks="$checks" \
        -f "$directory/records-scheduler-record.awk" "$scratch/prefix" > /dev/null
fi
