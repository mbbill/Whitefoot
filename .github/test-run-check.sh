#!/bin/sh
# Exercise the gate wrapper's ownership and exit-status boundaries cheaply.
set -eu
runner=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)/run-check.pl
work=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-check-test.XXXXXX")
holder=
cleanup() {
    if [ -n "$holder" ]; then wait "$holder" 2>/dev/null || :; fi
    rm -rf "$work"
}
trap cleanup EXIT
unset WHITEFOOT_CHECK_OWNER
WHITEFOOT_CHECK_LOCK_DIR=$work/lock
export WHITEFOOT_CHECK_LOCK_DIR

status=0
perl "$runner" failure sh -c 'exit 17' > "$work/failure.log" 2>&1 || status=$?
test "$status" -eq 17
test ! -e "$work/lock"

perl "$runner" parent perl "$runner" nested true > "$work/nested.log" 2>&1
test ! -e "$work/lock"

perl "$runner" holder sh -c 'touch "$1"; sleep 2' sh "$work/ready" > "$work/holder.log" 2>&1 &
holder=$!
attempt=0
while [ ! -f "$work/ready" ]; do
    attempt=$((attempt + 1))
    test "$attempt" -lt 100
    sleep 0.02
done
status=0
perl "$runner" competing true > "$work/competing.log" 2>&1 || status=$?
test "$status" -eq 75
grep -q 'already owned' "$work/competing.log"
wait "$holder"
holder=
test ! -e "$work/lock"

(
    unset CARGO_BUILD_JOBS RUST_TEST_THREADS JOBS
    perl "$runner" defaults sh -c 'test "$CARGO_BUILD_JOBS:$RUST_TEST_THREADS:$JOBS" = 2:2:2'
) > "$work/defaults.log" 2>&1
test ! -e "$work/lock"

status=0
WHITEFOOT_CHECK_TIMEOUT=1 perl "$runner" timeout sh -c \
    'sleep 30 & echo $! > "$1"; wait' sh "$work/child" \
    > "$work/timeout.log" 2>&1 || status=$?
test "$status" -eq 124
test ! -e "$work/lock"
! kill -0 "$(cat "$work/child")" 2>/dev/null

perl "$runner" signal perl "$runner" nested sh -c \
    'sleep 30 & echo $! > "$1"; wait' sh "$work/signal-child" \
    > "$work/signal.log" 2>&1 &
holder=$!
attempt=0
while [ ! -f "$work/signal-child" ]; do
    attempt=$((attempt + 1))
    test "$attempt" -lt 100
    sleep 0.02
done
kill -TERM "$holder"
status=0
wait "$holder" || status=$?
holder=
test "$status" -eq 143
test ! -e "$work/lock"
! kill -0 "$(cat "$work/signal-child")" 2>/dev/null

status=0
perl "$runner" orphan sh -c 'sleep 30 & echo $! > "$1"' sh "$work/orphan" \
    > "$work/orphan.log" 2>&1 || status=$?
test "$status" -eq 1
test ! -e "$work/lock"
! kill -0 "$(cat "$work/orphan")" 2>/dev/null
echo 'check runner: status, nesting, exclusion, limits, timeout, cancellation and orphan cleanup pass'
