# Strict callback trace contract, used by the compute-runtime trace collector.
# Retire with that diagnostic. Parameters: backend,width,shape,count,limit,
# grain,chunks,seed,pass,reps,level,shutdown; optional expected_bytes and
# runtime_schema (wf-1 or rayon-join-1 for the separate event image). No output
# precedes full validation.
# Successful output is one headerless TSV row:
# backend width shape records bytes max_length grain chunks seed pass reps level
# calls traced_chunks batch_start_ns batch_end_ns
# Count/configuration parameters are limited to 2^31-1 for exact AWK indexing;
# timestamps, thread IDs, byte/resource totals retain exact decimal arithmetic.
function fail() {
    bad = 1
    print "scheduler trace report rejected at line " NR > "/dev/stderr"
    exit 1
}
function uint(s) { return s ~ /^[0-9]+$/ }
function canon(s) { sub(/^0+/, "", s); return length(s) ? s : "0" }
# Timestamps, OS thread IDs and cumulative resources may exceed exact AWK
# integer precision. Compare and add their decimal strings without rounding.
function cmp(a, b) {
    a = canon(a); b = canon(b)
    if (length(a) != length(b)) return length(a) < length(b) ? -1 : 1
    return ("x" a) == ("x" b) ? 0 : (("x" a) < ("x" b) ? -1 : 1)
}
function add(a, b, i, j, carry, digit, result) {
    i = length(a); j = length(b); carry = 0; result = ""
    while (i || j || carry) {
        digit = carry
        if (i) digit += substr(a, i--, 1) + 0
        if (j) digit += substr(b, j--, 1) + 0
        result = (digit % 10) result
        carry = int(digit / 10)
    }
    return canon(result)
}
function mul(a, b, i, j, result, partial) {
    result = "0"
    for (i = 1; i <= length(b); ++i) {
        partial = "0"
        for (j = 0; j < substr(b, i, 1) + 0; ++j) partial = add(partial, a)
        result = add(result "0", partial)
    }
    return canon(result)
}
function value(position, key, pair) {
    if (split(p[position], pair, "=") != 2 || pair[1] != key || !length(pair[2])) fail()
    return pair[2]
}
function number(position, key, result) {
    result = value(position, key)
    if (!uint(result)) fail()
    return canon(result)
}
function expected(position, key, wanted) {
    if (("x" value(position, key)) != ("x" wanted)) fail()
}
function complete_call() {
    if (call_seen && (chunk_seen != (level == "plain" ? 0 : chunks) ||
        (level != "plain" && cmp(offered, bytes)))) fail()
    if (!call_seen || !length(runtime_schema)) return
    if (runtime_seen != runtime_banks || cmp(runtime_sums[0], jobs)) fail()
    if (runtime_schema == "wf-1") {
        if (cmp(add(runtime_sums[1], runtime_sums[5]), jobs) ||
            cmp(runtime_sums[6], jobs) || cmp(runtime_sums[7], jobs) ||
            cmp(runtime_sums[9], jobs) || cmp(runtime_sums[8], runtime_sums[1]) > 0 ||
            cmp(runtime_sums[19], "0")) fail()
    } else {
        if (cmp(add(runtime_sums[1], runtime_sums[2]), jobs) ||
            cmp(runtime_sums[3], jobs) || cmp(add(runtime_sums[4], runtime_sums[5]), jobs)) fail()
        for (event = 6; event < runtime_events; ++event) if (cmp(runtime_sums[event], "0")) fail()
    }
}
BEGIN {
    if (!length(backend) || !length(shape) || !uint(width) || width == 0 ||
        !uint(count) || !uint(limit) || !uint(grain) || grain == 0 ||
        !uint(chunks) || !uint(seed) || !uint(pass) || !uint(reps) ||
        (level != "plain" && level != "identity" && level != "timeline") ||
        (shutdown != "0" && shutdown != "1")) fail()
    if (width > 2147483647 || count > 2147483647 || limit > 2147483647 ||
        grain > 2147483647 || chunks > 2147483647 || pass > 2147483647 || reps > 2147483647) fail()
    if (chunks != int(count / grain) + (count % grain != 0)) fail()
    call_seen = 0; traced = "0"
    user_sum = system_sum = voluntary_sum = involuntary_sum = peak_rss = "0"
    if (length(runtime_schema)) {
        if (width != 1 && width != 2 && width != 4) fail()
        if (runtime_schema == "wf-1" && backend == "wf-runtime") {
            runtime_banks = width; runtime_events = 20
        } else if (runtime_schema == "rayon-join-1" && backend == "rayon-1.12.0-join") {
            runtime_banks = 5; runtime_events = 13
        } else fail()
        jobs = sprintf("%.0f", chunks && (runtime_schema != "wf-1" || width != 1) ? chunks - 1 : 0)
    }
}
NR == 1 {
    if (split($0, p, " ") != 16 || p[1] != "#" || p[2] != "trace") fail()
    expected(3, "backend", backend); expected(4, "width", width)
    expected(5, "shape", shape); expected(6, "records", count)
    bytes = number(7, "bytes")
    if (length(expected_bytes) && (!uint(expected_bytes) || cmp(bytes, expected_bytes))) fail()
    expected(8, "max_length", limit); expected(9, "grain", grain)
    expected(10, "chunks", chunks); expected(11, "seed", seed)
    expected(12, "pass", pass); expected(13, "reps", reps)
    expected(14, "level", level)
    if (number(15, "caller_tid") == "0") fail()
    number(16, "clock_pair_min_ns")
    if (cmp(bytes, mul(count, limit)) > 0) fail()
    meta = 1
    next
}
/^# runtime_events / {
    if (!length(runtime_schema) || NR != 2 || meta != 1 || runtime_meta ||
        split($0, p, " ") != 5 || p[1] != "#" || p[2] != "runtime_events") fail()
    expected(3, "schema", runtime_schema); expected(4, "banks", runtime_banks)
    expected(5, "events", runtime_events)
    runtime_meta = 1
    next
}
/^call\t/ {
    if (meta != 1 || batch || capacity || passed || call_seen > reps ||
        split($0, p, "\t") != 10 || p[1] != "call" ||
        !uint(p[2]) || p[2] != call_seen || p[3] != (call_seen ? "warm" : "first")) fail()
    if (length(runtime_schema) && runtime_meta != 1) fail()
    complete_call()
    for (i = 4; i <= 10; ++i) if (!uint(p[i])) fail()
    if (cmp(p[4], p[5]) > 0 || (call_seen && cmp(p[4], call_end) < 0)) fail()
    if (!call_seen) first_start = p[4]
    call_start = p[4]; call_end = p[5]
    user_sum = add(user_sum, p[6]); system_sum = add(system_sum, p[7])
    if (cmp(p[8], peak_rss) > 0) peak_rss = p[8]
    voluntary_sum = add(voluntary_sum, p[9]); involuntary_sum = add(involuntary_sum, p[10])
    ++call_seen; chunk_seen = 0; offered = "0"; thread_count = 0
    for (key in tids) delete tids[key]
    runtime_seen = 0
    for (key in runtime_sums) delete runtime_sums[key]
    next
}
/^runtime\t/ {
    if (!runtime_meta || !call_seen || batch || capacity || passed || chunk_seen ||
        runtime_seen >= runtime_banks || split($0, p, "\t") != 3 + runtime_events ||
        !uint(p[2]) || p[2] != call_seen - 1 || !uint(p[3]) || p[3] != runtime_seen) fail()
    for (event = 0; event < runtime_events; ++event) {
        v = p[4 + event]
        if (!uint(v) || cmp(v, "18446744073709551615") > 0 ||
            (runtime_schema == "rayon-join-1" && runtime_seen >= width && cmp(v, "0"))) fail()
        runtime_sums[event] = add(runtime_sums[event], v)
        if (cmp(runtime_sums[event], "18446744073709551615") > 0) fail()
    }
    ++runtime_seen
    next
}
/^chunk\t/ {
    if (!call_seen || batch || capacity || passed || level == "plain" ||
        split($0, p, "\t") != 9 || p[1] != "chunk") fail()
    if (length(runtime_schema) && runtime_seen != runtime_banks) fail()
    for (i = 2; i <= 9; ++i) if (!uint(p[i])) fail()
    if (p[2] != call_seen - 1 || p[3] != chunk_seen || chunk_seen >= chunks || canon(p[4]) == "0") fail()
    first = chunk_seen * grain; records = count - first
    if (records > grain) records = grain
    if (p[7] != first || p[8] != records || cmp(p[9], mul(sprintf("%.0f", records), limit)) > 0) fail()
    if (level == "identity") {
        if (canon(p[5]) != "0" || canon(p[6]) != "0") fail()
    } else if (cmp(p[5], p[6]) > 0 || cmp(p[5], call_start) < 0 || cmp(p[6], call_end) > 0) fail()
    key = "tid:" canon(p[4])
    if (!(key in tids)) { tids[key] = 1; if (++thread_count > width) fail() }
    offered = add(offered, p[9]); ++chunk_seen; traced = add(traced, "1")
    next
}
/^# batch / {
    complete_call()
    if (meta != 1 || call_seen != reps + 1 || batch || capacity || passed ||
        split($0, p, " ") != 9 || p[1] != "#" || p[2] != "batch") fail()
    batch_start = number(3, "start_ns"); batch_end = number(4, "end_ns")
    if (cmp(batch_start, first_start) > 0 || cmp(batch_end, call_end) < 0 ||
        cmp(batch_start, batch_end) > 0 || cmp(number(5, "user_us"), user_sum) < 0 ||
        cmp(number(6, "system_us"), system_sum) < 0 || cmp(number(7, "maxrss_bytes"), peak_rss) < 0 ||
        cmp(number(8, "voluntary_switches"), voluntary_sum) < 0 ||
        cmp(number(9, "involuntary_switches"), involuntary_sum) < 0) fail()
    batch = 1
    next
}
/^# post_timing_capacity=/ {
    if (batch != 1 || capacity || passed || split($0, p, " ") != 6 || p[1] != "#") fail()
    expected(2, "post_timing_capacity", width); expected(3, "includes_caller", "1")
    if (value(4, "capacity_waves") !~ /^[1-3]$/) fail()
    expected(5, "explicit_shutdown", shutdown); number(6, "stop_ns")
    capacity = 1
    next
}
/^record scheduler trace PASS:/ {
    if (capacity != 1 || passed || split($0, p, " ") != 7 ||
        p[1] != "record" || p[2] != "scheduler" || p[3] != "trace" || p[4] != "PASS:") fail()
    if (cmp(number(5, "calls"), sprintf("%.0f", call_seen)) ||
        cmp(number(6, "outputs"), mul(sprintf("%.0f", call_seen), count)) ||
        cmp(number(7, "traced_chunks"), traced)) fail()
    passed = 1
    next
}
{ fail() }
END {
    if (bad || meta != 1 || batch != 1 || capacity != 1 || passed != 1) exit 1
    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%.0f\t%s\t%s\t%s\n", \
        backend, width, shape, count, bytes, limit, grain, chunks, seed, pass, reps, level, \
        call_seen, traced, batch_start, batch_end
}
