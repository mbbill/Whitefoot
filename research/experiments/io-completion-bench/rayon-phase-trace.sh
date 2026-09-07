#!/usr/bin/env bash
# Coarse process boundaries for rayon-bench.sh's startup control panel.
# The io-model investigation owns this diagnostic; remove it with that panel.
set -euo pipefail

OUT=${OUT:?set OUT to the qualified startup panel directory}
TRACE="$OUT/phase-trace"
EXPECTED='420a993efa7437a1 41fa962893d45299'
PERF=${PROFILE_PERF:-perf}
CPU_YIELD_TRACE=${CPU_YIELD_TRACE:-0}
CPU_YIELD_CALLER=${CPU_YIELD_CALLER:-0}
[[ $CPU_YIELD_TRACE == 0 || $CPU_YIELD_TRACE == 1 ]] || {
    echo 'rayon-phase-trace: CPU_YIELD_TRACE must be 0 or 1' >&2
    exit 2
}
[[ $CPU_YIELD_CALLER == 0 || $CPU_YIELD_CALLER == 1 ]] || {
    echo 'rayon-phase-trace: CPU_YIELD_CALLER must be 0 or 1' >&2
    exit 2
}
[[ $CPU_YIELD_CALLER == 0 || $CPU_YIELD_TRACE == 1 ]] || {
    echo 'rayon-phase-trace: CPU_YIELD_CALLER requires CPU_YIELD_TRACE=1' >&2
    exit 2
}
mkdir -p "$TRACE"
printf 'status=preflight\n' > "$TRACE/status.txt"

incomplete() {
    printf 'status=incomplete\nreason=%s\n' "$*" > "$TRACE/status.txt"
    echo "rayon-phase-trace: incomplete: $*" >&2
    # Ordinary checksum-validated timings remain usable; this is explicitly
    # not a successful attribution trace, and CI retains the failure detail.
    exit 0
}
[[ $(uname -s) == Linux ]] || incomplete 'Linux tracepoints are unavailable on this host'
if [[ $CPU_YIELD_CALLER == 1 ]]; then
    [[ $(uname -m) == x86_64 ]] || incomplete 'yield caller capture requires the x86-64 entry stack convention'
    command -v objdump > /dev/null || incomplete 'objdump is unavailable for caller attribution'
fi
command -v "$PERF" > /dev/null || incomplete 'perf executable is unavailable'
sudo -n true || incomplete 'noninteractive privileged recording is unavailable'
trace_root=''
for candidate in /sys/kernel/tracing /sys/kernel/debug/tracing; do
    if sudo -n test -r "$candidate/available_events"; then trace_root=$candidate; break; fi
done
[[ -n $trace_root ]] || incomplete 'tracefs is not mounted or readable'
{
    uname -a
    "$PERF" --version
    lscpu
    lscpu -e=CPU,CORE,SOCKET,NODE,ONLINE
    awk '/Cpus_allowed_list:/ {print}' /proc/self/status
    printf 'tracefs=%s\nworkload_user=%s\n' "$trace_root" "$(id -un)"
    printf 'cohort=one batch; four forms; two opposite orders; no trace timing enters resource.tsv\n'
    printf 'probe_kind=entry only; no return probes, recursive layout probes or compute-join probes\n'
    printf 'write_filter=per-event --exclude-perf; global scheduler events retained\n'
    printf 'yield_syscalls=%s; paired enter/exit records, no call stacks\n' "$CPU_YIELD_TRACE"
    printf 'yield_caller=%s; entry stack word and probe IP, no return probe\n' "$CPU_YIELD_CALLER"
    if [[ $CPU_YIELD_CALLER == 1 ]]; then objdump --version; fi
    printf 'attribution=observed intervals only; syscall tracing may change scheduling\n'
    cat /proc/sys/kernel/perf_event_paranoid /proc/sys/kernel/kptr_restrict
} > "$TRACE/host.txt"
sudo -n cat "$trace_root/available_events" > "$TRACE/available-events.txt"
events=(sched:sched_process_exec sched:sched_process_fork sched:sched_process_exit
    sched:sched_switch sched:sched_waking syscalls:sys_enter_write syscalls:sys_exit_write
    syscalls:sys_enter_wait4 syscalls:sys_exit_wait4 syscalls:sys_enter_exit_group)
if [[ $CPU_YIELD_TRACE == 1 ]]; then
    events+=(syscalls:sys_enter_sched_yield syscalls:sys_exit_sched_yield)
fi
event_args=()
for event in "${events[@]}"; do
    grep -Fxq "$event" "$TRACE/available-events.txt" || incomplete "missing required event $event"
    sudo -n cat "$trace_root/events/${event%%:*}/${event#*:}/format" > "$TRACE/${event/:/-}.format"
    event_args+=(-e "$event")
    case "$event" in
        syscalls:sys_enter_write|syscalls:sys_exit_write) event_args+=(--exclude-perf) ;;
    esac
done

# perf probe resolves each ELF function to its file offset; do not treat a
# symbol's virtual address as its uprobe file offset. Use a private group and
# remove only that group's events, including after partial registration.
group="wfphase_$$"
cleanup() {
    sudo -n "$PERF" probe --del "$group:*" >> "$TRACE/probe-cleanup.log" 2>&1 || true
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
symbols=(wf__floor_run wf_sched_init wf_sched_run wf__main_body
    wf__completion_file_write_submit wf__completion_file_join wf_sched_post_status)
names=(floor init run body submit join post)
for tag in ordinary lanes seq; do
    case "$tag" in
        ordinary) binary="$OUT/wf-par" ;;
        lanes) binary="$OUT/wf-manual-1" ;;
        seq) binary="$OUT/wf-seq" ;;
    esac
    nm -n "$binary" > "$TRACE/$tag.symbols"
    readelf -Wl "$binary" > "$TRACE/$tag.segments"
    shasum -a 256 "$binary" >> "$TRACE/host.txt"
    for ((index=0; index<${#symbols[@]}; index++)); do
        symbol=${symbols[index]}
        awk -v symbol="$symbol" '$3==symbol {seen++} END {exit seen!=1}' "$TRACE/$tag.symbols" || \
            incomplete "missing or ambiguous entry symbol $tag/$symbol"
        event="${tag}_${names[index]}"
        if ! sudo -n "$PERF" probe -x "$binary" --add "$group:$event=$symbol" \
            >> "$TRACE/probe-registration.log" 2>&1; then
            incomplete "entry probe registration failed for $tag/$symbol"
        fi
        event_args+=(-e "$group:$event")
        sudo -n cat "$trace_root/events/$group/$event/format" > "$TRACE/$event.format"
    done
    if [[ $CPU_YIELD_CALLER == 1 ]]; then
        # At the exact x86-64 function entry, stack0 is the caller's return
        # address. The event's implicit probe IP supplies the same-image ASLR
        # base: caller - probe_ip + ELF(wf_prim_yield). Retain the full
        # disassembly so the audit can require an actual preceding call and
        # distinguish multiple waits in the same function. This adds one
        # uprobe per WF yield and can perturb scheduling; it is not a timing.
        awk '$3=="wf_prim_yield" {seen++} END {exit seen!=1}' "$TRACE/$tag.symbols" || \
            incomplete "missing or ambiguous yield symbol for $tag"
        objdump -f "$binary" > "$TRACE/$tag.elf"
        grep -Fq 'file format elf64-x86-64' "$TRACE/$tag.elf" || \
            incomplete "yield caller capture requires x86-64 ELF for $tag"
        objdump -d "$binary" > "$TRACE/$tag.disassembly"
        event="${tag}_yield_caller"
        if ! sudo -n "$PERF" probe -x "$binary" \
            --add "$group:$event=wf_prim_yield+0 caller=\$stack0:x64" \
            >> "$TRACE/probe-registration.log" 2>&1; then
            incomplete "yield caller probe registration failed for $tag"
        fi
        event_args+=(-e "$group:$event")
        sudo -n cat "$trace_root/events/$group/$event/format" > "$TRACE/$event.format"
    fi
done
sudo -n cat "$trace_root/uprobe_events" | awk -v group="$group/" 'index($1,group)>0' > "$TRACE/registered-probes.txt"
printf 'probe_group=%s\n' "$group" >> "$TRACE/host.txt"

# Save both PIDs without adding a writer to the child's checksum pipe. The
# runner and each child exec in place after these untimed launch wrappers.
cat > "$TRACE/launch" <<'LAUNCH'
#!/bin/sh
record=$1
redirect=$2
shift 2
printf '%s\n' "$$" > "$record.pid"
if [ "$redirect" = capture ]; then
    exec "$@" > "$record.out" 2> "$record.err"
fi
exec "$@"
LAUNCH
chmod +x "$TRACE/launch"
printf 'pass\tform\trunner_pid\tchild_pid\trecorder_pid\n' > "$TRACE/captures.tsv"
for pass in 0 1; do
    forms=(ordinary lanes seq rayon)
    if [[ $pass == 1 ]]; then forms=(rayon seq lanes ordinary); fi
    for tag in "${forms[@]}"; do
        record="$TRACE/p$pass-$tag"
        environment='WF_WORKERS=4,WF_STACKS=12,WF_SCHED_REPORT=0'
        case "$tag" in
            ordinary) command=("$OUT/wf-par") ;;
            lanes) command=("$OUT/wf-manual-1") ;;
            seq) command=("$OUT/wf-seq") ;;
            rayon) command=("$OUT/rust-layout" rayon 4 4 1); environment='' ;;
        esac
        printf '%s\t%s\t%s\t%s\tinherit' "$tag" "$environment" "$TRACE/launch" "$record.child" > "$record.plan"
        printf '\t%s' "${command[@]}" >> "$record.plan"
        printf '\n' >> "$record.plan"
        record_command=(sudo -n "$TRACE/launch" "$record.recorder" inherit
            "$PERF" record -a --clockid mono -m 16M "${event_args[@]}" -o "$record.data" --
            sudo -n -u "$(id -un)" -- env -u WF_IO_NO_NATIVE_RING "WF_BENCH_RAW=$record.tsv"
            "$TRACE/launch" "$record.runner" capture "$OUT/runner" "$record.plan" 1 0 "$EXPECTED")
        printf '%q ' "${record_command[@]}" > "$record.command"
        printf '\n' >> "$record.command"
        record_status=0
        "${record_command[@]}" > "$record.recorder.out" 2> "$record.recorder.err" || record_status=$?
        # A failed recorder can still leave useful partial data owned by root.
        # Make that file uploadable before taking the incomplete-result path.
        if [[ -f $record.data ]]; then sudo -n chown "$(id -u):$(id -g)" "$record.data"; fi
        if [[ -f $record.recorder.pid ]]; then sudo -n chown "$(id -u):$(id -g)" "$record.recorder.pid"; fi
        if [[ $record_status != 0 ]]; then
            incomplete "recorder or checksum runner failed for p$pass-$tag; inspect retained stderr"
        fi
        [[ -s $record.child.pid && -s $record.runner.pid ]] || incomplete "missing PID in p$pass-$tag"
        # The existing runner prints one progress line to stderr. Any extra
        # runtime output remains a diagnostic rather than being discarded.
        printf 'runner: pass 1 of 1 (plan order)\n' | cmp - "$record.runner.err" || \
            incomplete "unexpected runner/workload stderr in p$pass-$tag"
        if ! "$PERF" script --ns --show-lost-events -i "$record.data" > "$record.samples" 2> "$record.script.err"; then
            incomplete "perf script failed for p$pass-$tag"
        fi
        if grep -Ei 'PERF_RECORD_LOST|LOST [0-9]+|lost [1-9][0-9]* (events|chunks)|throttl' \
            "$record.samples" "$record.recorder.err" "$record.script.err" > "$record.loss.txt"; then
            incomplete "loss/throttling marker in p$pass-$tag"
        fi
        [[ -s $record.samples && ! -s $record.script.err ]] || incomplete "empty or errored decode in p$pass-$tag"
        if [[ $CPU_YIELD_CALLER == 1 ]]; then
            # Raw records expose LOST_SAMPLES and throttling even when the
            # ordinary event rendering does not print a corresponding line.
            if ! "$PERF" script -D -i "$record.data" > "$record.raw" 2> "$record.raw.err"; then
                incomplete "raw record decode failed for p$pass-$tag"
            fi
            [[ -s $record.raw && ! -s $record.raw.err ]] || incomplete "empty or errored raw decode in p$pass-$tag"
            if grep -E 'PERF_RECORD_(LOST|THROTTLE|UNTHROTTLE)' "$record.raw" >> "$record.loss.txt"; then
                incomplete "raw loss/throttling record in p$pass-$tag"
            fi
        fi
        child=$(cat "$record.child.pid")
        runner=$(cat "$record.runner.pid")
        recorder=$(cat "$record.recorder.pid")
        [[ $child =~ ^[0-9]+$ && $runner =~ ^[0-9]+$ && $recorder =~ ^[0-9]+$ ]] || incomplete "invalid PID in p$pass-$tag"
        awk -v pid="$recorder" '
            $0 ~ ("[[:space:]]" pid "[[:space:]]+\\[") &&
              ($0 ~ /syscalls:sys_enter_write:/ || $0 ~ /syscalls:sys_exit_write:/) {seen++}
            END {exit seen!=0}' "$record.samples" || incomplete "recorder write events survived filtering in p$pass-$tag"
        # Presence is a capture qualification, not yet a matched-retcode or
        # latency analysis. The retained tracepoint schemas and raw records
        # allow the later audit to reconstruct exec/fork membership and reap.
        for pair in "$child syscalls:sys_enter_exit_group:" "$runner syscalls:sys_exit_wait4:"; do
            read -r pid event <<< "$pair"
            awk -v pid="$pid" -v event="$event" '
                $0 ~ ("[[:space:]]" pid "[[:space:]]+\\[") && index($0,event)>0 {seen++}
                END {exit seen==0}' "$record.samples" || incomplete "missing $event for PID $pid in p$pass-$tag"
        done
        if [[ $tag != rayon ]]; then
            for name in "${names[@]}"; do
                grep -Fq "$group:${tag}_${name}:" "$record.samples" || incomplete "missing $tag/$name milestone in pass $pass"
            done
        fi
        printf '%s\t%s\t%s\t%s\t%s\n' "$pass" "$tag" "$runner" "$child" "$recorder" >> "$TRACE/captures.tsv"
        echo "rayon-phase-trace: p$pass-$tag captured; PID/thread/reap attribution still requires audit"
    done
done
printf 'status=captured\nrecords=8\nattribution=requires independent PID/thread/milestone/reap audit\n' > "$TRACE/status.txt"
