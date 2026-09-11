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
    delete warm; delete steal
    n = 0
    for (i = 1; i <= h_calls; i++) { n++; warm[n] = wall[i]; steal[n] = grants[i] }
    process_ns[cell, h_pass] = median(warm, n)
    process_steals[cell, h_pass] = median(steal, n)
    if (h_workload != "") workload[h_kernel] = h_workload
    open_process = 0
    next
}

/^#/ { next }

{
    if (!open_process) { bad("a data row outside any process"); next }
    if (NF != 9) { bad("a data row with " NF " fields"); next }
    call = $5 + 0
    if (call != rows) bad("non-dense call index " call " in " h_form)
    if ((call == 0) != ($6 == "first")) bad("phase " $6 " at call " call " in " h_form)
    if ($1 != h_kernel || $2 != h_form || $3 + 0 != h_width || $4 + 0 != h_pass)
        bad("a data row that does not match its header")
    if (call >= 1) { wall[call] = $7 + 0; grants[call] = $9 + 0 }
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
    if (pins) printf "pins: %s\n", pins
    for (i = 1; i <= kernels; i++) printf "sizes: %-12s %s\n", kernel_at[i], workload[kernel_at[i]]
    printf "passes=%d calls=%d\n\n", passes + 0, calls + 0
    # The grain column is sized to the longest grain string this run actually
    # printed, and is never truncated. A fixed 32-column field silently cut
    # every reference's policy sentence in half, and it cut the `static` row's
    # disclosure -- that it is a regular-work reference and not a
    # dynamic-scheduling ceiling for skew -- out of the table entirely, which
    # is the one thing readers of that row kept getting wrong. A wide line is
    # cheaper than a missing disclosure.
    grainw = 5
    for (c = 1; c <= cellcount; c++) {
        cell = cell_order[c]
        if (!(cell in med)) continue
        if (length(cell_grain[cell]) > grainw) grainw = length(cell_grain[cell])
    }
    grainfmt = "%-" grainw "s"

    printf "%-11s %2s %-16s " grainfmt " %10s %5s %-22s %-16s %5s %7s %s\n",
        "kernel", "w", "form", "grain", "median_us", "mad%", "p10..p90_us", "ratio", "lower", "steals", "note"

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

function report(block,   c, cell, n, i, p, order, best, bestname, wfcell, ratios, nr, low, madpct, spread, ratiotext,
                         fastest, fastmed, note, count, modal, modalname, verdict) {
    n = 0; wfcell = ""
    for (c = 1; c <= cellcount; c++) {
        cell = cell_order[c]
        if (cell_block[cell] != block) continue
        if (!(cell in med)) continue
        order[++n] = cell
        if (cell_form[cell] == "wf") wfcell = cell
    }
    if (n == 0) return
    # ascending by median; the table is read top down
    for (i = 2; i <= n; i++) {
        cell = order[i]; p = i - 1
        while (p > 0 && med[order[p]] > med[cell]) { order[p + 1] = order[p]; p-- }
        order[p + 1] = cell
    }
    fastest = order[1]; fastmed = med[fastest]

    nr = 0; low = 0
    if (block_width[block] > 1 && wfcell != "") {
        for (p = 0; p < passes + 0; p++) {
            best = ""; bestname = ""
            for (c = 1; c <= cellcount; c++) {
                cell = cell_order[c]
                if (cell_block[cell] != block || cell_form[cell] == "wf") continue
                if (!((cell SUBSEP p) in seen_pass)) continue
                if (best == "" || process_ns[cell, p] < best) {
                    best = process_ns[cell, p]; bestname = cell_form[cell]
                }
            }
            if (best == "" || !((wfcell SUBSEP p) in seen_pass)) continue
            ratios[++nr] = process_ns[wfcell, p] / best
            count[bestname]++
            if (process_ns[wfcell, p] < best) low++
        }
    }
    modal = 0; modalname = "none"
    for (bestname in count) if (count[bestname] > modal) { modal = count[bestname]; modalname = bestname }

    for (i = 1; i <= n; i++) {
        cell = order[i]
        note = cell_note[cell]
        if (cell_form[cell] == "wf") {
            if (cell_chunks[cell] != "" && cell_chunks[cell] != "na")
                note = note (note ? " " : "") cell_chunks[cell] " chunks"
            if (block_width[block] > 1 && steals[cell] == 0)
                note = note (note ? " " : "") "no-lanes"
        }
        madpct = med[cell] > 0 ? 100.0 * mad[cell] / med[cell] : 0
        spread = sprintf("%.1f..%.1f", p10[cell] / 1000.0, p90[cell] / 1000.0)
        printf "%-11s %2d %-16s " grainfmt " %10.1f %5.1f %-22s ", block_kernel[block], block_width[block], cell_form[cell], cell_grain[cell], med[cell] / 1000.0, madpct, spread
        if (cell == wfcell && nr > 0) {
            sorted(ratios, nr)
            ratiotext = sprintf("%.3f [%.2f-%.2f]", median(ratios, nr), ratios[1], ratios[nr])
            printf "%-16s %5s %7d ", ratiotext, sprintf("%d/%d", low, nr), steals[cell]
        } else printf "%-16s %5s %7s ", "", "", ""
        printf "%s\n", note
    }
    if (block_width[block] > 1) {
        verdict = block_over[block] ? "n/a (oversubscribed)" : (cell_form[fastest] == "wf" ? "yes" : "no")
        printf "%-11s %2d BEST REFERENCE = %-12s FASTEST = %-12s WF fastest: %s\n",
            block_kernel[block], block_width[block], modalname, cell_form[fastest], verdict
    }
    printf "\n"
}
