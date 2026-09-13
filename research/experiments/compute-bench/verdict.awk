# Serves compute-bench: the one place a number in `table.txt` decides a
# pass/fail, and the only such place in this bundle.
#
#   awk -f verdict.awk [-v widths="1 2 4"] [-v wall=0.97] [-v cpu=0.90] \
#       [-v lower=4] [-v pairs=5] <table.txt>
#
# It reads the `A/B  wf-b/wf` lines reduce.awk prints under each block and
# nothing else. Those lines exist only when a twin was built, and the twin the
# compute-regression workflow builds is THE MERGE-BASE'S RUNTIME AND EMISSION
# (`WF_B_SCHED_DIR`, `WF_B_FLOOR`, `WF_B_WFC` in the Makefile), timed against
# this tree's `wf` in the same passes. So a `wall` figure below one means the
# tree the branch started from was faster than the branch, which is the one
# question this file answers.
#
# THE RULE, IN TWO PARTS. A block is ADVERSE when its paired wall median is
# below `wall` AND the baseline was the faster arm in at least `lower` of the
# `pairs` pairs. Both halves are required: a median can sit under the band on
# one adverse pair, and a count of adverse pairs says nothing about their size.
# It is read only at the widths `widths` names -- the ones a four-CPU hosted
# runner can resolve -- and never on a block the reducer marked oversubscribed.
#
# A KERNEL then FAILS when at least TWO of its read blocks are adverse. ONE
# ADVERSE BLOCK ALONE IS A `suspect`: printed on its row and again under the
# table so a reviewer sees it, and it fails nothing.
#
# The second part is there because a single width on this host class does not
# separate a slower program from a differently placed one. Two images linked
# from BYTE-IDENTICAL sources land their functions at different offsets, and
# width one is where that shows: pull request #50's first attempt read
# `records W=1 wall 0.938` with 5/5 lower on a branch that changes no unit the
# compute path compiles on Linux, and run 34671025894 read `records` at 0.912
# at W=1 with the instrumented code it added not present in either arm. Both
# were one block. A real regression in the scheduler runtime or the emission
# does not confine itself to one width, while code placement does exactly
# that -- the offsets that move at W=1 are not the offsets that move at W=4.
# So the `suspect` line is the report, and two of them in one kernel is the
# verdict. $(WF_ALIGN) in the Makefile closes the part of this a compiler flag
# can reach; this rule is what covers the part it cannot.
#
# CPU IS A REPORT, NEVER A FAILURE. A paired CPU ratio below `cpu` under the
# same `lower` count is printed as a line under the table and changes no exit
# status. It cannot fail a job because the reducer prints one `lower` count,
# the WALL one, and a CPU signal read against a wall-pair count is evidence
# worth looking at rather than evidence worth failing on.
#
# EXIT STATUS. 0 when no kernel fails -- suspects included, which is the whole
# point of the word -- 1 when one or more kernels fail, and 2 when the table
# cannot answer the question at all: no A/B line (the twin was never built,
# which would otherwise pass vacuously), no A/B line at a recorded width, or
# fewer pairs behind a line than the rule names. A refusal is not a pass and is
# not a regression; it is a broken run.

BEGIN {
    if (widths == "") widths = "1 2 4"
    if (wall + 0 <= 0) wall = 0.97
    if (cpu + 0 <= 0) cpu = 0.90
    if (lower + 0 <= 0) lower = 4
    if (pairs + 0 <= 0) pairs = 5

    nw = split(widths, wlist, /[ ,]+/)
    for (i = 1; i <= nw; i++)
        if (wlist[i] != "") recorded[wlist[i] + 0] = 1

    seen_ab = 0; read_rows = 0; cpu_reports = 0
    skipped_width = 0; skipped_over = 0; underpowered = 0
}

# The block verdict line, read only for the reducer's own oversubscription
# mark. A width above the host's CPU count is emitted and timed, and the
# reducer refuses to call a winner there because oversubscription rewards
# schedulers that yield; a regression read off such a block would be reading
# the same thing. It precedes its block's A/B line, so recording it here is
# enough.
/ BEST REFERENCE = / {
    if ($0 ~ /oversubscribed/) oversubscribed[$1, $2 + 0] = 1
    next
}

# mandelbrot   4 A/B  wf-b/wf  wall 0.981 [0.96-1.01]  lower 4/5  cpu 1.002
$3 == "A/B" && $4 == "wf-b/wf" && $5 == "wall" {
    seen_ab++
    kernel = $1
    width = $2 + 0
    wallratio = $6 + 0
    split($9, lowfield, "/")
    lowcount = lowfield[1] + 0
    paircount = lowfield[2] + 0
    cpuratio = $11

    if (!(width in recorded)) { skipped_width++; next }
    if ((kernel, width) in oversubscribed) { skipped_over++; next }

    read_rows++
    if (paircount < pairs + 0) {
        underpowered++
        thin[underpowered] = sprintf("%s W=%d: %d pairs, the rule names %d",
                                     kernel, width, paircount, pairs + 0)
    }

    # Whether this block is adverse is decided here; whether that makes it a
    # failure or a suspect cannot be, because it depends on the kernel's other
    # blocks and a later line can still be one of them. So the row is carried
    # to END with its adverse mark and given its word there.
    adverse = (wallratio < wall + 0 && lowcount >= lower + 0)
    if (adverse) {
        rowadverse[read_rows] = 1
        adverse_blocks[kernel]++
        adverse_where[kernel, adverse_blocks[kernel]] = \
            sprintf("%s W=%d: wall %.3f with %d/%d pairs lower",
                    kernel, width, wallratio, lowcount, paircount)
    } else {
        rowadverse[read_rows] = 0
    }
    rowkernel[read_rows] = kernel

    cpumark = ""
    if (cpuratio != "n/a" && cpuratio + 0 < cpu + 0 && lowcount >= lower + 0) {
        cpu_reports++
        cpumark = " *"
        report[cpu_reports] = sprintf("%s W=%d: cpu %.3f with %d/%d wall pairs lower",
                                      kernel, width, cpuratio + 0, lowcount, paircount)
    }

    row[read_rows] = sprintf("%-12s %2d  %7.3f    %d/%-3d  %7s%-2s  ",
                             kernel, width, wallratio, lowcount, paircount,
                             cpuratio, cpumark)
}

END {
    printf "compute-regression verdict: the merge-base twin `wf-b` against this tree's `wf`\n"
    printf "rule: a block is adverse when wall < %.3f and the baseline was lower in >= %d\n",
        wall + 0, lower + 0
    printf "      of %d pairs, at W in {%s}, oversubscribed blocks excluded. A kernel FAILs\n",
        pairs + 0, widths
    printf "      on two adverse blocks; one alone is a `suspect` and fails nothing, because\n"
    printf "      one width cannot separate a slower program from a differently placed one.\n"
    printf "      A cpu ratio < %.3f under the same count is reported (*) and fails nothing.\n\n",
        cpu + 0

    if (seen_ab == 0) {
        printf "REFUSED: the table carries no `A/B  wf-b/wf` line, so no twin was built and\n"
        printf "         there is nothing to compare. Build the twin by naming a baseline\n"
        printf "         (WF_B_SCHED_DIR / WF_B_FLOOR / WF_B_WFC) and re-run `compare`.\n"
        exit 2
    }
    if (read_rows == 0) {
        printf "REFUSED: %d A/B line(s) present, none of them at a recorded width in {%s}\n",
            seen_ab, widths
        printf "         (%d skipped as an unrecorded width, %d as oversubscribed).\n",
            skipped_width, skipped_over
        exit 2
    }

    # The word each adverse row gets, now that every block of every kernel has
    # been read: `FAIL` where its kernel has two or more, `suspect` where it is
    # the only one. Non-adverse rows read `pass` whatever their kernel does,
    # because a block that is not under the band is not evidence against the
    # branch at that width.
    failed_blocks = 0; failed_kernels = 0; suspects = 0
    for (i = 1; i <= read_rows; i++) {
        if (!rowadverse[i]) { verdict[i] = "pass"; continue }
        if (adverse_blocks[rowkernel[i]] >= 2) {
            verdict[i] = "FAIL"
            failed_blocks++
        } else {
            verdict[i] = "suspect"
            suspects++
            suspect_line[suspects] = adverse_where[rowkernel[i], 1]
        }
    }
    for (k in adverse_blocks)
        if (adverse_blocks[k] >= 2) failed_kernels++

    printf "%-12s %2s  %7s    %-5s  %7s    %s\n", "kernel", "W", "wall", "lower", "cpu", "verdict"
    for (i = 1; i <= read_rows; i++) print row[i] verdict[i]
    printf "\n"

    for (i = 1; i <= cpu_reports; i++) printf "cpu report: %s\n", report[i]
    if (cpu_reports > 0) printf "\n"

    # Printed whatever the exit status is. A suspect on a failing run is still
    # a block a reviewer has to look at, and a suspect on a passing run is the
    # only trace of it in the job.
    for (i = 1; i <= suspects; i++)
        printf "suspect: %s -- the only adverse block in its kernel, so it fails nothing.\n",
            suspect_line[i]
    if (suspects > 0) printf "\n"

    printf "read %d block(s); skipped %d at an unrecorded width and %d oversubscribed.\n",
        read_rows, skipped_width, skipped_over

    if (underpowered > 0) {
        for (i = 1; i <= underpowered; i++) printf "REFUSED: %s\n", thin[i]
        printf "REFUSED: a verdict from fewer pairs than the rule names is not the rule.\n"
        exit 2
    }

    if (failed_kernels > 0) {
        printf "VERDICT: FAIL -- %d kernel(s) slower than the merge-base at "\
               "%d block(s), %d suspect(s), %d cpu report(s).\n",
            failed_kernels, failed_blocks, suspects, cpu_reports
        exit 1
    }
    printf "VERDICT: PASS -- no kernel slower than the merge-base at two blocks, "\
           "%d suspect(s), %d cpu report(s).\n",
        suspects, cpu_reports
    exit 0
}
