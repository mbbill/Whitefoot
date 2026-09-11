# Serves compute-bench: the reducer. It turns $(RESULTS)/raw.tsv into the one
# table the bundle exists to print. It is awk so the bundle keeps no Python.
#
#   awk -f reduce.awk -v passes=5 -v calls=5 [-v host=.. -v cpus=.. ...] raw.tsv
#
# NOTHING HERE FAILS ON A MEASUREMENT. The exit status is nonzero only on a
# missing or malformed row: a cell with the wrong number of processes, a row
# count other than calls+1, a non-dense call index, `first` in the wrong
# place, or a header/trailer mismatch. Never on a ratio, never on spread,
# never on an ordering, never on an elapsed time.
#
# The median routine is the insertion-sort form of the research bundle's
# pure-compare.awk; everything else is new, because every median in that
# bundle's record was computed by hand outside its harness.

function sorted(a, n,   i, j, x) {
    for (i = 2; i <= n; i++) { x = a[i]; j = i - 1
        while (j > 0 && a[j] > x) { a[j + 1] = a[j]; j-- }
        a[j + 1] = x }
}
function median(a, n) {
    sorted(a, n)
    return n % 2 ? a[(n + 1) / 2] : (a[n / 2] + a[n / 2 + 1]) / 2
}
function fraction(a, n, f,   i) {           # a must already be sorted
    i = int((n - 1) * f + 0.5) + 1
    return a[i]
}
function absolute(x) { return x < 0 ? -x : x }
function field(line, key,   rest) {          # a value with no spaces in it
    if (!match(line, " " key "=")) return ""
    rest = substr(line, RSTART + RLENGTH)
    sub(/ .*/, "", rest)
    return rest
}
function span(line, key, stopkey,   rest) {  # a value that may contain spaces
    if (!match(line, " " key "=")) return ""
    rest = substr(line, RSTART + RLENGTH)
    if (match(rest, " " stopkey "=")) rest = substr(rest, 1, RSTART - 1)
    return rest
}
function bad(why) { printf "reduce: %s\n", why > "/dev/stderr"; broken = 1 }

BEGIN {
    FS = "\t"
    if (passes + 0 < 1 || calls + 0 < 1) { print "reduce: passes and calls required" > "/dev/stderr"; exit 1 }
    open_process = 0
}

/^# driver=/ {
    if (open_process) bad("a header opened before the previous trailer")
    h_kernel = field($0, "driver"); h_form = field($0, "form")
    h_grain = span($0, "grain", "width"); h_width = field($0, "width") + 0
    h_over = field($0, "oversubscribed") + 0
    h_calls = field($0, "calls") + 0; h_pass = field($0, "pass") + 0
    h_chunks = field($0, "chunks"); h_workload = span($0, "workload", "clock")
    h_note = span($0, "note", "chunks")
    rows = 0; open_process = 1
    # The A/B twin's presence is read off the stream and nowhere else: the
    # Makefile renames the twin's own `wf` to `wf-b` on its way into raw.tsv,
    # so a table that has one says so in its rows. It changes what the header
    # lines below say about the three control variables, because with a twin the
    # controls built `wf-b` and `wf` is still the plain program.
    if (h_form == "wf-b") has_twin = 1
    if (h_calls != calls + 0) bad("header calls=" h_calls " is not " calls)
    next
}

/^# batch / {
    if (!open_process) bad("a trailer with no header")
    if (field($0, "form") != h_form || field($0, "width") + 0 != h_width ||
        field($0, "calls") + 0 != h_calls) bad("header/trailer mismatch for " h_form)
    if (rows != h_calls + 1) bad("process " h_kernel "/" h_form "/w" h_width "/p" h_pass \
                                 " has " rows " rows, not " h_calls + 1)
    cell = h_kernel SUBSEP h_width SUBSEP h_form
    if (!(cell in seen_cell)) {
        seen_cell[cell] = 1
        cell_kernel[cell] = h_kernel; cell_width[cell] = h_width; cell_form[cell] = h_form
        cell_grain[cell] = h_grain;   cell_chunks[cell] = h_chunks
        cell_note[cell] = h_note
        block = h_kernel SUBSEP h_width
        if (!(block in seen_block)) {
            seen_block[block] = 1; block_over[block] = h_over
            block_kernel[block] = h_kernel; block_width[block] = h_width
            if (!(h_kernel in seen_kernel)) { seen_kernel[h_kernel] = ++kernels; kernel_at[kernels] = h_kernel }
            block_order[++blocks] = block
        }
        cell_block[cell] = block
        cell_order[++cellcount] = cell
    }
    if ((cell SUBSEP h_pass) in seen_pass) bad("cell " h_form " repeated pass " h_pass)
    seen_pass[cell SUBSEP h_pass] = 1
    delete warm; delete steal; delete burn
    n = 0
    for (i = 1; i <= h_calls; i++) { n++; warm[n] = wall[i]; steal[n] = grants[i]; burn[n] = cpu[i] }
    process_ns[cell, h_pass] = median(warm, n)
    process_cpu[cell, h_pass] = median(burn, n)
    process_steals[cell, h_pass] = median(steal, n)
    if (h_workload != "") workload[h_kernel] = h_workload
    open_process = 0
    next
}

/^#/ { next }

{
    if (!open_process) { bad("a data row outside any process"); next }
    if (NF != 10) { bad("a data row with " NF " fields"); next }
    call = $5 + 0
    if (call != rows) bad("non-dense call index " call " in " h_form)
    if ((call == 0) != ($6 == "first")) bad("phase " $6 " at call " call " in " h_form)
    if ($1 != h_kernel || $2 != h_form || $3 + 0 != h_width || $4 + 0 != h_pass)
        bad("a data row that does not match its header")
    if (call >= 1) { wall[call] = $7 + 0; cpu[call] = $8 + 0; grants[call] = $10 + 0 }
    rows++
}

END {
    if (open_process) bad("a header with no trailer")
    if (cellcount == 0) bad("no process rows at all")

    for (c = 1; c <= cellcount; c++) {
        cell = cell_order[c]
        delete s; n = 0
        for (p = 0; p < passes + 0; p++) {
            if (!((cell SUBSEP p) in seen_pass)) { bad("cell " cell_form[cell] " w" cell_width[cell] \
                " of " cell_kernel[cell] " is missing pass " p); continue }
            s[++n] = process_ns[cell, p]
        }
        if (n == 0) continue
        med[cell] = median(s, n)
        delete cs; nc = 0
        for (p = 0; p < passes + 0; p++) if ((cell SUBSEP p) in seen_pass) cs[++nc] = process_cpu[cell, p]
        cpumed[cell] = nc ? median(cs, nc) : 0
        sorted(s, n)
        p10[cell] = fraction(s, n, 0.10); p90[cell] = fraction(s, n, 0.90)
        delete d
        for (i = 1; i <= n; i++) d[i] = absolute(s[i] - med[cell])
        mad[cell] = median(d, n)
        delete t; n = 0
        for (p = 0; p < passes + 0; p++) if ((cell SUBSEP p) in seen_pass) t[++n] = process_steals[cell, p]
        steals[cell] = median(t, n)
    }

    if (broken) exit 1

    printf "compute-bench  host=%s  cpus=%s  mask=%s  date=%s\n",
        host ? host : "unknown", cpus ? cpus : "unknown", mask ? mask : "unqualified",
        date ? date : "unknown"
    printf "run=%s  compiler=%s  clang=%s  rustc=%s\n",
        run ? run : "local", compiler ? compiler : "unknown",
        clang ? clang : "unknown", rustc ? rustc : "unknown"
    if (refflags) printf "reference flags=%s\n      (identical for every reference implementation of every kernel)\n", refflags
    if (wfflags) printf "WF flags=%s\n      (module and runtime, as whitefootc links them: no -march, no loop\n      alignment -- see README)\n", wfflags
    # Printed only when the bundle's A/B control was set, so a table with this
    # line is not a plain `--par` table and can never be recorded as one. Its
    # absence is the ordinary case and says the `wf` row is plain `--par`;
    # manifest.txt carries WF_PAR_CONTROL_FLAGS either way. With a twin in the
    # stream the flags reached `wf-b` and not `wf`, and the line says which.
    if (parcontrol) {
        if (has_twin) printf "WF --par control flags=%s\n      (appended to the --par emission of the `wf-b` TWIN only: `wf` above is\n      still the plain --par program. A table with a wf-b row is an A/B\n      instrument and must not be recorded as a plain table -- see README)\n", parcontrol
        else printf "WF --par control flags=%s\n      (an A/B control appended to the --par emission: this table is NOT the\n      plain --par program and must not be recorded as one -- see README)\n", parcontrol
    }
    # The same line for the other side of the link, on the same terms: present
    # only when the runtime was built at something other than what this tree
    # ships, absent in the ordinary case, and recorded in manifest.txt either
    # way as WF_RUNTIME_CONTROL_FLAGS.
    if (runtimecontrol) {
        if (has_twin) printf "WF runtime control flags=%s\n      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:\n      `wf` above is still the runtime this tree ships. A table with a wf-b row\n      is an A/B instrument and must not be recorded as a plain table -- see\n      README)\n", runtimecontrol
        else printf "WF runtime control flags=%s\n      (an A/B control appended to the compile of the Whitefoot runtime: this\n      table is NOT the runtime this tree ships and must not be recorded as one\n      -- see README)\n", runtimecontrol
    }
    # The third control: the same disclosure for the flags the twin's Whitefoot
    # side was COMPILED at, as against the constants it was compiled with.
    # Present only when the twin's module object and runtime units were built at
    # something other than the flags whitefootc itself passes clang, absent in
    # the ordinary case, and recorded in manifest.txt either way as
    # WF_MODULE_CONTROL_FLAGS.
    if (modulecontrol) {
        if (has_twin) printf "WF module control flags=%s\n      (appended to the compile of the `wf-b` TWIN's emitted module object and\n      its Whitefoot runtime only: `wf` above is still built at the WF flags\n      above. A table with a wf-b row is an A/B instrument and must not be\n      recorded as a plain table -- see README)\n", modulecontrol
        else printf "WF module control flags=%s\n      (an A/B control appended to the compile of the Whitefoot module and\n      runtime: this table is NOT built at the flags whitefootc passes clang and\n      must not be recorded as one -- see README)\n", modulecontrol
    }
    # WF_AB=1 asks for the twin with no control set at all, which is the
    # instrument's own null check: two images built from one tree that differ
    # in nothing, so every `A/B` line below should read near 1.000 with mixed
    # `lower` counts. Neither line above would print, so this one does.
    if (has_twin && !parcontrol && !runtimecontrol && !modulecontrol) printf "WF A/B twin: `wf-b` is built from the same sources as `wf` with NO control\n      flag set (WF_AB=1), so the A/B lines below read this host's own\n      within-pass spread over identical behaviour -- see README\n"
    if (pins) printf "pins: %s\n", pins
    for (i = 1; i <= kernels; i++) printf "sizes: %-12s %s\n", kernel_at[i], workload[kernel_at[i]]
    printf "passes=%d calls=%d\n\n", passes + 0, calls + 0
    # No grain column. Each block prints its own grain strings in full as a
    # legend under its verdict line; see the end of `report` below.
    printf "%-11s %2s %-16s %10s %5s %-22s %10s %-17s %6s %5s %7s %s\n",
        "kernel", "w", "form", "median_us", "mad%", "p10..p90_us", "cpu_us", "ratio", "cpu_r", "lower", "steals", "note"

    for (i = 1; i <= kernels; i++) {
        k = kernel_at[i]
        for (stage = 0; stage <= 1; stage++)
            for (b = 1; b <= blocks; b++) {
                block = block_order[b]
                if (block_kernel[block] != k) continue
                if (stage == 0 && block_width[block] == 1) continue
                if (stage == 1 && block_width[block] != 1) continue
                report(block)
            }
    }
    exit 0
}

# The ratio columns of one Whitefoot row: the wall ratio with its range, the
# paired CPU ratio, the count of paired passes that were lower, and the steal
# median. `wf` and the A/B twin `wf-b` are printed by the same function against
# the same per-pass reference, so the two rows are read the same way and a
# reader never has to ask whether the twin's ratio means something else.
function ratio_columns(r, n, cr, ncr, low, st,   text, cputext) {
    sorted(r, n)
    text = sprintf("%.3f [%.2f-%.2f]", median(r, n), r[1], r[n])
    cputext = ncr > 0 ? sprintf("%.3f", median(cr, ncr)) : ""
    printf "%-17s %6s %5s %7d ", text, cputext, sprintf("%d/%d", low, n), st
}

function report(block,   c, cell, n, i, p, order, best, bestname, bestcell, wfcell, ratios, nr, low, madpct, spread,
                         cpuratios, ncr, fastest, fastmed, note, count, modal, modalname, verdict,
                         wfbcell, bratios, nrb, lowb, bcpuratios, ncrb, abwall, abcpu, nab, ncab, lowab, abcputext) {
    n = 0; wfcell = ""; wfbcell = ""
    for (c = 1; c <= cellcount; c++) {
        cell = cell_order[c]
        if (cell_block[cell] != block) continue
        if (!(cell in med)) continue
        order[++n] = cell
        if (cell_form[cell] == "wf") wfcell = cell
        if (cell_form[cell] == "wf-b") wfbcell = cell
    }
    if (n == 0) return
    # ascending by median; the table is read top down
    for (i = 2; i <= n; i++) {
        cell = order[i]; p = i - 1
        while (p > 0 && med[order[p]] > med[cell]) { order[p + 1] = order[p]; p-- }
        order[p + 1] = cell
    }
    # The verdict line's FASTEST is the fastest of the plain row and the
    # references, and it skips the twin: the question it answers is whether the
    # program this tree produces is the fastest thing in its row, and a twin
    # built from a control flag is not a thing this tree produces. The A/B line
    # under it is where the twin's verdict is read.
    fastest = ""
    for (i = 1; i <= n; i++) if (cell_form[order[i]] != "wf-b") { fastest = order[i]; break }
    if (fastest == "") fastest = order[1]
    fastmed = med[fastest]

    nr = 0; low = 0; ncr = 0; nrb = 0; lowb = 0; ncrb = 0
    if (block_width[block] > 1) {
        for (p = 0; p < passes + 0; p++) {
            best = ""; bestname = ""; bestcell = ""
            for (c = 1; c <= cellcount; c++) {
                cell = cell_order[c]
                if (cell_block[cell] != block) continue
                # Neither Whitefoot row is a reference for the other: the twin
                # is excluded here exactly as `wf` is, so `wf`'s ratio is the
                # same number it would have been without a twin in the run.
                if (cell_form[cell] == "wf" || cell_form[cell] == "wf-b") continue
                if (!((cell SUBSEP p) in seen_pass)) continue
                if (best == "" || process_ns[cell, p] < best) {
                    best = process_ns[cell, p]; bestname = cell_form[cell]; bestcell = cell
                }
            }
            if (best == "") continue
            if (wfcell != "" && ((wfcell SUBSEP p) in seen_pass)) {
                ratios[++nr] = process_ns[wfcell, p] / best
                # The CPU ratio is paired exactly as the wall ratio is: the same
                # pass and the same reference -- the one that was fastest by WALL in
                # that pass -- so the two ratios are about the same pairs and can be
                # read side by side. Pairing CPU against whichever reference burned
                # least CPU would answer a different question and would not line up
                # with the verdict line.
                if (process_cpu[bestcell, p] > 0) cpuratios[++ncr] = process_cpu[wfcell, p] / process_cpu[bestcell, p]
                count[bestname]++
                if (process_ns[wfcell, p] < best) low++
            }
            if (wfbcell != "" && ((wfbcell SUBSEP p) in seen_pass)) {
                bratios[++nrb] = process_ns[wfbcell, p] / best
                if (process_cpu[bestcell, p] > 0) bcpuratios[++ncrb] = process_cpu[wfbcell, p] / process_cpu[bestcell, p]
                if (process_ns[wfbcell, p] < best) lowb++
            }
        }
    }
    modal = 0; modalname = "none"
    for (bestname in count) if (count[bestname] > modal) { modal = count[bestname]; modalname = bestname }

    for (i = 1; i <= n; i++) {
        cell = order[i]
        note = cell_note[cell]
        if (cell_form[cell] == "wf" || cell_form[cell] == "wf-b") {
            if (cell_chunks[cell] != "" && cell_chunks[cell] != "na")
                note = note (note ? " " : "") cell_chunks[cell] " chunks"
            if (block_width[block] > 1 && steals[cell] == 0)
                note = note (note ? " " : "") "no-lanes"
        }
        madpct = med[cell] > 0 ? 100.0 * mad[cell] / med[cell] : 0
        spread = sprintf("%.1f..%.1f", p10[cell] / 1000.0, p90[cell] / 1000.0)
        printf "%-11s %2d %-16s %10.1f %5.1f %-22s %10.1f ", block_kernel[block], block_width[block], cell_form[cell], med[cell] / 1000.0, madpct, spread, cpumed[cell] / 1000.0
        if (cell == wfcell && nr > 0) ratio_columns(ratios, nr, cpuratios, ncr, low, steals[cell])
        else if (cell == wfbcell && nrb > 0) ratio_columns(bratios, nrb, bcpuratios, ncrb, lowb, steals[cell])
        else printf "%-17s %6s %5s %7s ", "", "", "", ""
        printf "%s\n", note
    }
    if (block_width[block] > 1) {
        verdict = block_over[block] ? "n/a (oversubscribed)" : (cell_form[fastest] == "wf" ? "yes" : "no")
        printf "%-11s %2d BEST REFERENCE = %-12s FASTEST = %-12s WF fastest: %s\n",
            block_kernel[block], block_width[block], modalname, cell_form[fastest], verdict
    }
    # The twin verdict, and the reason the twin exists: the two Whitefoot
    # images ran as separate processes inside every pass, in the same rotated
    # and reversed order as every other form, so they can be paired PASS BY
    # PASS. The wall figure is the median of those paired ratios with its range,
    # `lower` is how many pairs had the twin lower, and `cpu` is the same
    # pairing over process CPU. This is the number an A/B run is read off:
    # dividing the twin's median by the plain image's would put this host's
    # run-to-run spread back into the comparison, which is what a separate
    # control run already cannot see past. It is printed at every width the
    # block carries, width one included, because "the twin changed nothing at
    # width one" is a thing an A/B run has to be able to say.
    if (wfcell != "" && wfbcell != "") {
        nab = 0; ncab = 0; lowab = 0
        for (p = 0; p < passes + 0; p++) {
            if (!((wfcell SUBSEP p) in seen_pass) || !((wfbcell SUBSEP p) in seen_pass)) continue
            if (process_ns[wfcell, p] <= 0) continue
            abwall[++nab] = process_ns[wfbcell, p] / process_ns[wfcell, p]
            if (process_ns[wfbcell, p] < process_ns[wfcell, p]) lowab++
            if (process_cpu[wfcell, p] > 0) abcpu[++ncab] = process_cpu[wfbcell, p] / process_cpu[wfcell, p]
        }
        if (nab > 0) {
            sorted(abwall, nab)
            # The CPU text is built before the printf and not inside its
            # argument list: awk reads a bare `>` there as an output
            # redirection, so an inline `ncab > 0 ? ... : ...` writes the table
            # into a file named after the false branch instead of printing it.
            abcputext = ncab > 0 ? sprintf("%.3f", median(abcpu, ncab)) : "n/a"
            printf "%-11s %2d A/B  wf-b/wf  wall %.3f [%.2f-%.2f]  lower %d/%d  cpu %s\n",
                block_kernel[block], block_width[block], median(abwall, nab), abwall[1], abwall[nab],
                lowab, nab, abcputext
        }
    }
    # The legend: one line per form of this block, in the row order above it,
    # every grain string in full and NEVER truncated. These strings were a
    # column once. A fixed 32-column field cut every reference's policy
    # sentence in half, and it cut the `static` row's disclosure -- that it is
    # a regular-work reference and not a dynamic-scheduling ceiling for skew --
    # out of the table entirely, which is the one thing readers of that row
    # kept getting wrong. Sizing the column to the longest string the run
    # printed kept the disclosure and made every data row about 200 characters
    # wide instead, past the width of any terminal that has to read the
    # numbers. Printing the same strings once per block keeps both.
    for (i = 1; i <= n; i++) printf "  %s: %s\n", cell_form[order[i]], cell_grain[order[i]]
    printf "\n"
}
