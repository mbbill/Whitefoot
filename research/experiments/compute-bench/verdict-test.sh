#!/bin/sh
# Serves compute-bench: the test of verdict.awk, which is the one rule in this
# bundle that decides a pass/fail. It builds table fragments by hand -- the
# reducer's exact `A/B  wf-b/wf` and `BEST REFERENCE` spellings -- feeds each
# to the rule, and asserts the exit status and the verdict line.
#
# It measures nothing, links nothing and needs no compiler, which is why the
# repository's `make check` can run it (root `Makefile`, `research-tests`)
# while the regression gate it protects stays outside the gate entirely.
#
# Removal condition: it goes when verdict.awk goes.
set -e

here=$(dirname "$0")
work=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-verdict-test.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

cases=0
failures=0

# Run the rule over $work/table.txt and require an exact exit status and a
# fragment of its output. A wrong status with the right text and a right
# status with the wrong text are both failures: the status is what the
# workflow reads and the text is what a reader acts on.
expect() {
    name=$1
    want_status=$2
    want_text=$3
    cases=$((cases + 1))
    set +e
    out=$(awk -f "$here/verdict.awk" "$work/table.txt" 2>&1)
    got_status=$?
    set -e
    if [ "$got_status" -ne "$want_status" ]; then
        echo "verdict-test FAIL [$name]: exit $got_status, expected $want_status"
        echo "$out" | sed 's/^/    /'
        failures=$((failures + 1))
        return
    fi
    case $out in
        *"$want_text"*) ;;
        *)
            echo "verdict-test FAIL [$name]: output does not contain: $want_text"
            echo "$out" | sed 's/^/    /'
            failures=$((failures + 1))
            return
            ;;
    esac
    echo "verdict-test ok   [$name]"
}

# A clean run: four recorded blocks, the twin never convincingly ahead, and one
# block under the wall band on too few adverse pairs to be the rule's business.
cat > "$work/table.txt" <<'TABLE'
fir           1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
fir           1 A/B  wf-b/wf  wall 1.002 [0.98-1.03]  lower 2/5  cpu 1.000
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 0.995 [0.97-1.02]  lower 3/5  cpu 0.998
fir           4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           4 A/B  wf-b/wf  wall 0.960 [0.92-1.01]  lower 2/5  cpu 0.990
mandelbrot    4 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: yes
mandelbrot    4 A/B  wf-b/wf  wall 1.010 [0.99-1.04]  lower 1/5  cpu 1.004
TABLE
expect "pass" 0 "VERDICT: PASS"

# The regression the gate exists for: the baseline faster by more than the band
# in four of five pairs, at two of the kernel's three widths.
cat > "$work/table.txt" <<'TABLE'
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 0.940 [0.90-0.99]  lower 4/5  cpu 0.995
fir           4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           4 A/B  wf-b/wf  wall 0.948 [0.91-0.98]  lower 5/5  cpu 0.991
TABLE
expect "two-block-fail" 1 "VERDICT: FAIL"

# One adverse block and nothing else is the false positive this rule was
# rewritten for: pull request #50's first attempt read records W=1 at 0.938
# with five of five lower on a branch that changes no unit the compute path
# compiles, and run 34671025894 read the same kernel at 0.912 at W=1 with the
# code it added present in neither arm. Both are one block, both are code
# placement, and a single block is reported rather than acted on.
cat > "$work/table.txt" <<'TABLE'
records       1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
records       1 A/B  wf-b/wf  wall 0.938 [0.92-0.95]  lower 5/5  cpu 0.996
records       2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
records       2 A/B  wf-b/wf  wall 0.998 [0.97-1.02]  lower 2/5  cpu 1.001
records       4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
records       4 A/B  wf-b/wf  wall 1.004 [0.99-1.03]  lower 1/5  cpu 1.000
TABLE
expect "one-block-suspect" 0 "suspect: records W=1: wall 0.938 with 5/5 pairs lower"

# The two-block count is per kernel and is not pooled across the table. One
# adverse block in each of two kernels is two suspects, not a failure: a
# regression in the shared runtime shows at more than one width of the kernel
# it slows, which is the thing being asked, and two unrelated single blocks are
# what a placement-sensitive host produces on its own.
cat > "$work/table.txt" <<'TABLE'
fir           1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
fir           1 A/B  wf-b/wf  wall 0.940 [0.90-0.99]  lower 4/5  cpu 0.995
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 1.000 [0.99-1.02]  lower 2/5  cpu 1.000
records       1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
records       1 A/B  wf-b/wf  wall 0.951 [0.93-0.97]  lower 4/5  cpu 0.998
records       2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
records       2 A/B  wf-b/wf  wall 1.002 [0.99-1.03]  lower 1/5  cpu 1.001
TABLE
expect "two-kernels-one-block-each" 0 "2 suspect(s)"

# Both halves of the rule are required. A median under the band with two
# adverse pairs is one arm of a noisy host, not a regression. The kernel's
# second block is here so the two-block rule has the two readable blocks it
# refuses without; it is the W=2 line that is under test.
cat > "$work/table.txt" <<'TABLE'
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 0.900 [0.60-1.30]  lower 2/5  cpu 1.000
fir           4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           4 A/B  wf-b/wf  wall 1.000 [0.99-1.01]  lower 2/5  cpu 1.000
TABLE
expect "wall-band-without-adverse-pairs" 0 "VERDICT: PASS"

# Four adverse pairs inside the band are not a regression either.
cat > "$work/table.txt" <<'TABLE'
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 0.985 [0.97-1.00]  lower 4/5  cpu 1.000
fir           4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           4 A/B  wf-b/wf  wall 1.000 [0.99-1.01]  lower 2/5  cpu 1.000
TABLE
expect "adverse-pairs-inside-band" 0 "VERDICT: PASS"

# No twin was built. The table is a perfectly good plain table and cannot
# answer this question, so the rule refuses rather than passing vacuously --
# the failure mode a gate reading absent evidence would otherwise have.
cat > "$work/table.txt" <<'TABLE'
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
TABLE
expect "no-twin" 2 "REFUSED: the table carries no"

# An oversubscribed width is excluded even when it is in the recorded set: a
# two-CPU host emits W=4 oversubscribed, and the reducer's own mark is what
# says so.
cat > "$work/table.txt" <<'TABLE'
fir           1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
fir           1 A/B  wf-b/wf  wall 1.000 [0.99-1.01]  lower 2/5  cpu 1.000
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 1.001 [0.99-1.02]  lower 2/5  cpu 1.000
fir           4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
fir           4 A/B  wf-b/wf  wall 0.800 [0.70-0.90]  lower 5/5  cpu 0.700
TABLE
expect "oversubscribed-excluded" 0 "VERDICT: PASS"

# W=8 and above are not resolvable on a four-CPU runner and are not in the
# recorded set, so a failure there is read, printed as skipped, and ignored.
cat > "$work/table.txt" <<'TABLE'
fir           1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
fir           1 A/B  wf-b/wf  wall 1.000 [0.99-1.01]  lower 2/5  cpu 1.000
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 1.004 [0.99-1.02]  lower 1/5  cpu 1.002
fir           8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           8 A/B  wf-b/wf  wall 0.850 [0.80-0.90]  lower 5/5  cpu 0.800
TABLE
expect "unrecorded-width-ignored" 0 "skipped 1 at an unrecorded width"

# Only unrecorded widths carry a twin: nothing the rule can read, so refuse
# rather than report a pass over an empty set.
cat > "$work/table.txt" <<'TABLE'
fir           8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           8 A/B  wf-b/wf  wall 0.850 [0.80-0.90]  lower 5/5  cpu 0.800
TABLE
expect "only-unrecorded-widths" 2 "none of them at a recorded width"

# Fewer pairs than the rule names. `lower >= 4 of 5` cannot be evaluated over
# two passes, and a rule that quietly passed everything on an under-powered
# run would be a gate that a shortened run switches off.
cat > "$work/table.txt" <<'TABLE'
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 0.999 [0.99-1.01]  lower 1/2  cpu 1.000
TABLE
expect "under-powered" 2 "2 pairs, the rule names 5"

# The CPU signal: below its band with the adverse count, reported under the
# table and failing nothing.
cat > "$work/table.txt" <<'TABLE'
fir           2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           2 A/B  wf-b/wf  wall 0.990 [0.98-1.00]  lower 4/5  cpu 0.850
fir           4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir           4 A/B  wf-b/wf  wall 1.000 [0.99-1.01]  lower 2/5  cpu 1.000
TABLE
expect "cpu-reported-never-failing" 0 "cpu report: fir W=2"

# No kernel has two readable blocks, so the two-block rule cannot reach FAIL on
# a table of this shape however bad the readings are. Both blocks here are
# adverse and the exit status is a refusal, not the pass a rule that could only
# pass would report: a gate that cannot fail is a gate that is switched off.
cat > "$work/table.txt" <<'TABLE'
fir           1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
fir           1 A/B  wf-b/wf  wall 0.930 [0.91-0.95]  lower 5/5  cpu 0.990
records       1 BEST REFERENCE = n/a          FASTEST = wf           WF fastest: yes
records       1 A/B  wf-b/wf  wall 0.940 [0.92-0.96]  lower 4/5  cpu 0.995
TABLE
expect "one-readable-block-per-kernel" 2 "no kernel has two readable blocks"

# A table with no rows at all is the same refusal as a table with no twin.
: > "$work/table.txt"
expect "empty-table" 2 "REFUSED: the table carries no"

if [ "$failures" -ne 0 ]; then
    echo "verdict-test: $failures of $cases case(s) failed"
    exit 1
fi
echo "verdict-test PASS: $cases case(s)"
