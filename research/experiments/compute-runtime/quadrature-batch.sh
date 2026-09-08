#!/bin/sh
# Joined repeated-input CPU/PMU attribution. Called by check-quadrature and
# quadrature-batch-calibrate; retire with that experiment, not a timing gate.
set -eu
cd "$(dirname "$0")"
mode=${1:?check or calibrate required}
case "$mode" in check|calibrate) ;; *) exit 1;; esac
: "${OUT:?absolute build output required}"
case "$OUT" in /*) ;; *) exit 1;; esac
results=${RESULTS:-$OUT/quadrature/batch-check-$$}
test ! -e "$results"
mkdir "$results"
unset WF_PERF_CONTROL_FD WF_PERF_ACK_FD
export LC_ALL=C
validator=$PWD/quadrature-batch.awk
image=$OUT/quadrature/ordinary
test -x "$image"
cp quadrature-batch.sh quadrature-batch.awk quadrature-memory.sh records-scheduler-memory.awk "$results/"
memory_profile=$(cat "$OUT/quadrature/memory-profile.txt")
case "$memory_profile" in address|clean) ;; *) exit 1;; esac
shasum -a 256 "$image" "$OUT/quadrature/sanitized" "$OUT/quadrature/memory-profile.txt" \
    quadrature-batch.sh quadrature-batch.awk quadrature-memory.sh records-scheduler-memory.awk > "$results/manifest.sha256"
validate() {
    awk -v input="$input" -v form="$form" -v spawn="$spawn" -v width="$width" \
        -v repeats="$repeats" -v stats="$stats" -v control="$control" -f "$validator" "$log"
}
invoke() {
    if test "$spawn" = 0 && test "$form" != tbb && test "$form" != parlay-left && test "$form" != wf-value; then
        set -- "$form"
    else set -- "$form" "$spawn"; fi
    if test "$mode" = check; then
        status=0
        LSAN_OPTIONS=exitcode=23 WF_WORKERS=$width WF_QUADRATURE_INPUT=$input WF_QUADRATURE_REPEATS=$repeats \
            "$image" batch "$@" > "$log" 2> "$log.stderr" || status=$?
        sh quadrature-memory.sh "$memory_profile" "$log.stderr" "$status" "$form" "$stats"
    else
        WF_WORKERS=$width WF_QUADRATURE_INPUT=$input WF_QUADRATURE_REPEATS=$repeats \
            "$image" batch "$@" > "$log" 2>&1
    fi
}
decode() {
    form=${variant%-d*};spawn=0
    case "$variant" in *-d*) spawn=${variant##*-d};; esac
}
validate_perf() {
    awk -F ';' -v events="$events" '
        function bad(why) { print "quadrature perf report: " why > "/dev/stderr";failed=1;exit 1 }
        function number(x) { return x ~ /^[0-9]+(\.[0-9]+)?$/ }
        BEGIN {total=split(events,names,",");for(i=1;i<=total;++i)wanted[names[i]]=1}
        $3 in wanted {
            if(seen[$3]++)bad("duplicate event")
            if(NF<5 || !number($1) || !number($4) || $4<=0 || !number($5) || $5<=0 || $5>100)
                bad("unmeasured event")
            ++count
        }
        END {if(!failed && count!=total)bad("missing event");if(failed)exit 1}
    ' "$perf_log"
}
if test "$mode" = check; then
    repeats=3;control=0
    for kind in ordinary sanitized; do
        image=$OUT/quadrature/$kind;stats=0;test "$kind" = ordinary || stats=1
        for width in 1 4; do
            for variant in native wf-leaf-seq wf-leaf wf-refusal wf-frontier-seq-d4 wf-frontier-d4 wf-frontier-seq-d8 wf-frontier-d8 cpp-seq rust-seq rayon-d4 rayon-d8 rayon-left-d4 rayon-left-d8 wf-value-d8 parlay-left-d8 tbb-d8; do
                decode
                for input in center-peak right-peak empty depth-zero; do
                    log=$results/$kind-w$width-$variant-$input.tsv
                    invoke;validate
                done
            done
        done
    done
    image=$OUT/quadrature/ordinary;stats=0;width=1;variant=native;decode
    for input in smooth center-peak left-peak right-peak outside-peak loose depth-zero depth-cap empty reverse; do
        log=$results/input-$input.tsv;invoke;validate
    done
    # Real pipe I/O checks both perf acknowledgement encodings. No PMU is
    # required for this protocol test; actual perf availability is probed below.
    mkfifo "$results/control.fifo" "$results/ack.fifo"
    exec 3<>"$results/control.fifo" 4<>"$results/ack.fifo"
    for encoding in line nul; do
        (
            for expected in enable disable; do
                IFS= read -r command <&3;test "$command" = "$expected"
                if test "$encoding" = line; then printf 'ack\n' >&4;else printf 'ack\n\000' >&4;fi
            done
        ) & responder=$!
        input=smooth;control=1;log=$results/protocol-$encoding.tsv
        (WF_PERF_CONTROL_FD=3 WF_PERF_ACK_FD=4 invoke)
        wait "$responder";validate
    done
    exec 3>&- 4>&-
    rm "$results/control.fifo" "$results/ack.fifo"
    control=0;good=$results/input-smooth.tsv
    awk 'NR==2 {$5=4} {print}' "$good" > "$results/wrong-repeats.tsv"
    log=$results/wrong-repeats.tsv
    if validate > "$results/wrong-repeats.log" 2>&1; then exit 1;fi
    grep -Fx 'quadrature batch report: batch identity' "$results/wrong-repeats.log"
    log=$results/unpaired.log
    if (WF_PERF_CONTROL_FD=3 invoke); then exit 1;fi
    grep -Fx 'quadrature: paired perf descriptors' "$log.stderr"
    events=cycles,instructions;perf_log=$results/perf-good.csv
    printf '100;;cycles;1000000;100.00;\n200;;instructions;1000000;100.00;\n' > "$perf_log"
    validate_perf
    for mutation in missing uncounted runtime; do
        perf_log=$results/perf-$mutation.csv
        case "$mutation" in
            missing) sed '2d' "$results/perf-good.csv" > "$perf_log";reason='missing event';;
            uncounted) sed '1s/^100;/<not counted>;/' "$results/perf-good.csv" > "$perf_log";reason='unmeasured event';;
            runtime) sed '1s/1000000/0/' "$results/perf-good.csv" > "$perf_log";reason='unmeasured event';;
        esac
        if validate_perf > "$perf_log.log" 2>&1;then exit 1;fi
        grep -Fx "quadrature perf report: $reason" "$perf_log.log"
    done
    printf '%s\n' 'quadrature batch qualification PASS: processes=284 outputs=3124 protocol=2 negative=5'
    exit 0
fi
rounds=${ROUNDS:-5};repeats=${REPEATS:-4096}
case "$rounds:$repeats" in *[!0-9:]*|:*|*:) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
test "$repeats" -ge 1 && test "$repeats" -le 65536
{
    printf 'mode=sustained repeats=%s warmup=8 rounds=%s\n' "$repeats" "$rounds"
    printf '%s\n' 'interval=joined repeated calls and bitwise checks; no per-call clock, output, or oracle' \
        'CPU=getrusage encloses process/caller CPU clocks; process encloses caller; caller encloses wall clocks' \
        'perf additionally includes enable-ack/disable-command boundary; API differences are observations, not acceptance limits' \
        'perf=inherited process counters, not system-wide; idle/spinning workers included' \
        'perf event runtime/percentage retained; multiplexed events are not exact simultaneous costs'
    uname -a
    if test "$(uname -s)" = Linux; then
        lscpu;cat /proc/self/status
        for path in /proc/sys/kernel/perf_event_paranoid /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path";cat "$path";else printf '%s: unavailable\n' "$path";fi
        done
    else sysctl hw.model hw.ncpu hw.physicalcpu;fi
} > "$results/host.txt"
perf=${PERF:-perf};events='';observers=plain
if test "$(uname -s)" = Linux && "$perf" version > "$results/perf-version.txt" 2>&1; then
    perf_path=$(command -v "$perf")
    cp "$perf_path" "$results/perf-command"
    shasum -a 256 "$results/perf-command" >> "$results/manifest.sha256"
    printf 'perf_command=%s\n' "$perf_path" >> "$results/host.txt"
    "$perf" version --build-options > "$results/perf-build-options.txt" 2>&1
    # Preserve actual event attributes for diagnosing counter disagreements;
    # a failed probe is evidence of unavailability, not a measurement of zero.
    if "$perf" stat -vv -e task-clock -- sleep 0.01 > "$results/perf-task-clock-attributes.txt" 2>&1;then
        printf '%s\n' 'probe_exit=0' >> "$results/perf-task-clock-attributes.txt"
    else printf '%s\n' 'probe_exit=nonzero' >> "$results/perf-task-clock-attributes.txt";fi
    for event in task-clock context-switches cpu-migrations page-faults cycles instructions branches branch-misses cache-references cache-misses; do
        if "$perf" stat -x ';' -o "$results/probe-$event.csv" -e "$event" -- sleep 0.01 > "$results/probe-$event.log" 2>&1 &&
            awk -F ';' -v event="$event" '$3==event && $1 ~ /^[0-9]+(\.[0-9]+)?$/ {ok=1} END {exit !ok}' "$results/probe-$event.csv"; then
            printf '%s\tavailable\n' "$event" >> "$results/availability.tsv"
            events=${events:+$events,}$event
        else printf '%s\tunavailable\n' "$event" >> "$results/availability.tsv";fi
    done
    if test -n "$events"; then observers='plain perf';fi
else printf '%s\n' 'perf unavailable on this host' > "$results/availability.tsv";fi
printf '%s\n' "$events" > "$results/events.txt"
printf 'pass\tobserver\tinput\tform\tworkers\tspawn_depth\trepeats\twarmup\tperf_control\tstats\tnodes_per_call\twall_ns\tuser_us\tsystem_us\tvoluntary\tinvoluntary\tminor_faults\tmajor_faults\twf_lanes\tprocess_cpu_ns\tcaller_cpu_ns\n' > "$results/summary.tsv"
for width in 1 4; do
    for input in center-peak left-peak right-peak depth-cap; do
        for variant in native wf-leaf-seq wf-leaf wf-refusal wf-frontier-seq-d4 wf-frontier-d4 wf-frontier-seq-d8 wf-frontier-d8 cpp-seq rust-seq rayon-d4 rayon-d8 rayon-left-d4 rayon-left-d8 wf-value-d8 parlay-left-d4 parlay-left-d8 tbb-d8; do
            for observer in $observers; do printf '%s %s %s %s\n' "$width" "$input" "$variant" "$observer";done
        done
    done
done > "$results/cells.txt"
pass=1;stats=0
while test "$pass" -le "$rounds"; do
    if test "$((pass % 2))" = 0; then awk '{a[NR]=$0} END {for(i=NR;i>0;--i)print a[i]}' "$results/cells.txt" > "$results/order.txt"
    else cp "$results/cells.txt" "$results/order.txt";fi
    while read -r width input variant observer; do
        decode;control=0;stem=$results/p$pass-w$width-$input-$variant-$observer;log=$stem.tsv
        if test "$observer" = plain; then invoke
        else
            control=1;mkfifo "$stem.control" "$stem.ack"
            (
                exec 3<>"$stem.control" 4<>"$stem.ack"
                set -- "$form";case "$variant" in *-d*) set -- "$form" "$spawn";;esac
                WF_WORKERS=$width WF_QUADRATURE_INPUT=$input WF_QUADRATURE_REPEATS=$repeats \
                    WF_PERF_CONTROL_FD=3 WF_PERF_ACK_FD=4 \
                    timeout 60 "$perf" stat --delay=-1 --control=fd:3,4 -x ';' \
                    -o "$stem.perf.csv" -e "$events" -- "$image" batch "$@" > "$log" 2> "$stem.perf.log"
            )
            rm "$stem.control" "$stem.ack"
            perf_log=$stem.perf.csv;validate_perf
        fi
        validate
        awk -v pass="$pass" -v observer="$observer" 'NR==2 {print pass "\t" observer "\t" $0}' "$log" >> "$results/summary.tsv"
    done < "$results/order.txt"
    pass=$((pass+1))
done
printf 'quadrature batch calibration PASS: processes=%s repeats=%s observers=%s\n' \
    "$(awk 'END {print NR-1}' "$results/summary.tsv")" "$repeats" "$observers"
