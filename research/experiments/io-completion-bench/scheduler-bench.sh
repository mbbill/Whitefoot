#!/usr/bin/env bash
# Paired compiler/runtime experiments and CPU placement. Candidate differences
# are explicit; each cohort keeps the completion engine and source workload.
# Owned by io-model/SCHEDULER-EXPERIMENT.md; replace after the next decision.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=${ROOT:-$(cd "$HERE/../../.." && pwd)}
OUT=${OUT:-${WHITEFOOT_SCRATCH_ROOT:-$HOME/do_not_scan}/whitefoot-scheduler-experiment}
CLANG=${CLANG:-/usr/bin/clang}
BACKEND=$ROOT/compiler/src/backend
MODE=${1:-bench}
ROUNDS=${ROUNDS:-7}
WARMUP=${WARMUP:-2}
EXPERIMENT=${EXPERIMENT:-idle}
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)

check_policy() {
        local policy=$1 configuration threads stacks log spin=1 progress=0
        if [[ $policy == sleep ]]; then spin=0; fi
        if [[ $policy == progress ]]; then progress=1; fi
        "$CLANG" -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic \
            -DWF_SCHED_ENUMERATE -DWF_SCHED_LANE_SLOTS=2u \
            -DWF_SCHED_MAX_THREADS=4u -DWF_SCHED_MAX_STACKS=8u \
            "-DWF_SCHED_IDLE_SPIN_ROUNDS=$spin" -DWF_SCHED_IDLE_YIELD_ROUNDS=0u \
            "-DWF_SCHED_IDLE_PROGRESS_INTERVAL=$progress" -DWF_SCHED_OBSERVE=1 \
            "$BACKEND/sched/core.c" "$BACKEND/sched/enumerate.c" \
            "$BACKEND/sched/schedules.c" -o "$OUT/enumerate-$policy"
        for configuration in '1 2' '1 3' '2 3' '2 4'; do
            read -r threads stacks <<< "$configuration"
            log="$OUT/enumerate-$policy-t${threads}s${stacks}.log"
            echo "enumerating policy=$policy threads=$threads stacks=$stacks"
            if ! "$OUT/enumerate-$policy" --threads "$threads" --stacks "$stacks" > "$log" 2>&1; then
                cat "$log"
                return 1
            fi
            tail -1 "$log"
        done
}

check() {
    # Every existing schedule and every design configuration, for immediate
    # sleep, the original spin, and a spin that progresses I/O. Small windows
    # expose their protocol transitions without treating delays as state.
    # Every child's exit is checked; no schedule or configuration is omitted.
    local policy pid status=0 pids=()
    for policy in sleep spin progress; do
        check_policy "$policy" &
        pids+=("$!")
    done
    for pid in "${pids[@]}"; do wait "$pid" || status=1; done
    return "$status"
}

if [[ $MODE == check ]]; then
    check
    exit 0
fi
if [[ $MODE != bench || $(uname -s) != Linux ]]; then
    echo 'scheduler-bench: use check on POSIX, or bench on Linux with io_uring' >&2
    exit 2
fi
[[ $EXPERIMENT == idle || $EXPERIMENT == mixed || $EXPERIMENT == fairness || $EXPERIMENT == inline || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == footprint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == priority ]] || exit 2
network_compute=0
if [[ $EXPERIMENT == mixed || $EXPERIMENT == fairness || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == priority ]]; then network_compute=1; fi
[[ $ROUNDS =~ ^[1-9][0-9]*$ && $WARMUP =~ ^[0-9]+$ ]] || exit 2
[[ $(nproc) -ge 4 ]] || { echo 'scheduler-bench: this CPU-placement experiment needs four logical CPUs' >&2; exit 2; }
mkdir -p "$OUT/bin" "$OUT/samples" "$OUT/observed" "$OUT/tree"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/compiler/target}
(cd "$ROOT/compiler" && cargo build --profile gate --bin whitefootc --locked --offline)
WFC=$CARGO_TARGET_DIR/gate/whitefootc

if [[ $EXPERIMENT == priority ]]; then
    for burst in 1 8; do
        if ! make -C "$ROOT/compiler" completion-test CC="$CLANG" \
            COMPLETION_TMP="$OUT/priority-b$burst-check" \
            COMPLETION_BASE_CFLAGS="-std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread -DWF_SCHED_COMPLETION_READY_BURST=$burst" \
            > "$OUT/priority-b$burst-check.log" 2>&1; then
            cat "$OUT/priority-b$burst-check.log"
            exit 1
        fi
    done
fi

if [[ $EXPERIMENT == inline || $EXPERIMENT == footprint ]]; then
    # The experimental path keeps every existing harness assertion, including
    # completion counts, and also runs the core's maintained enumeration.
    check_define='-DWF_COMPLETION_LOCAL_INLINE=1'
    if [[ $EXPERIMENT == footprint ]]; then check_define='-DWF_SCHED_INIT_USED_LANES=1'; fi
    if ! make -C "$ROOT/compiler" completion-test CC="$CLANG" \
        COMPLETION_TMP="$OUT/$EXPERIMENT-check" \
        COMPLETION_BASE_CFLAGS="-std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread $check_define" \
        > "$OUT/$EXPERIMENT-check.log" 2>&1; then
        cat "$OUT/$EXPERIMENT-check.log"
        exit 1
    fi
fi

{
    git -C "$ROOT" rev-parse HEAD
    uname -a
    "$CLANG" --version
    lscpu
    echo "experiment=$EXPERIMENT rounds=$ROUNDS warmup=$WARMUP"
    echo "io_uring_disabled=$(cat /proc/sys/kernel/io_uring_disabled 2>/dev/null || echo absent)"
    echo "affinity=$(taskset -pc $$)"
    lscpu -e=CPU,CORE,SOCKET,NODE,ONLINE
} > "$OUT/host.txt"

# Group only CPUs available to this process by physical core. The split2
# cohort uses disjoint logical CPUs; on a two-core SMT host those still share
# physical cores. Split1 uses one CPU on each of two different physical cores.
allowed=$(awk '/Cpus_allowed_list:/ { print $2 }' /proc/self/status)
mapfile -t physical_groups < <(lscpu -b -p=CPU,CORE,SOCKET | awk -F, -v allowed="$allowed" '
    BEGIN { n=split(allowed,a,","); for(i=1;i<=n;i++) { m=split(a[i],b,"-");
      for(j=b[1]+0;j<=(m==1 ? b[1]+0 : b[2]+0);j++) available[j]=1; } }
    !/^#/ && available[$1] { key=$3 "/" $2; if(!(key in cpus)) order[++count]=key;
      cpus[key]=cpus[key] (cpus[key]=="" ? "" : ",") $1; }
    END { for(i=1;i<=count;i++) print cpus[order[i]]; }')
[[ ${#physical_groups[@]} -ge 2 ]] || { echo 'scheduler-bench: two physical cores required' >&2; exit 2; }
server_one=${physical_groups[0]%%,*}
client_one=${physical_groups[1]%%,*}
server_two="$server_one,$client_one"
client_two=$(printf '%s\n' "${physical_groups[@]}" | tr ',' '\n' | awk -v a="$server_one" -v b="$client_one" '
    $1!=a && $1!=b && n<2 { result=result (n++ ? "," : "") $1 }
    END { if(n==2) print result; else exit 1 }')
printf 'shared4\t4\t4\t%s\t%s\nshared2\t2\t2\t%s\t%s\nsplit2\t2\t2\t%s\t%s\nsplit1\t1\t1\t%s\t%s\n' \
    "$allowed" "$allowed" "$allowed" "$allowed" "$server_two" "$client_two" "$server_one" "$client_one" > "$OUT/cohorts.tsv"

forms=(base sleep short spin poll1 poll16)
if [[ $network_compute == 1 ]]; then
    forms=(base sleep poll1)
    awk '$1=="shared4" || $1=="split2"' "$OUT/cohorts.tsv" > "$OUT/cohorts-selected.tsv"
    mv "$OUT/cohorts-selected.tsv" "$OUT/cohorts.tsv"
fi
if [[ $EXPERIMENT == fairness || $EXPERIMENT == sustain ]]; then forms=(base); fi
if [[ $EXPERIMENT == inline ]]; then forms=(base local); fi
if [[ $EXPERIMENT == checkpoint || $EXPERIMENT == paced ]]; then forms=(base cq1024 cq16384 cq65536); fi
if [[ $EXPERIMENT == chunks ]]; then forms=(base cq16384 ch1024 ch16384 ch65536); fi
if [[ $EXPERIMENT == priority ]]; then forms=(ch1024 ch1024b1 ch1024b8 ch16384 ch16384b1 ch16384b8); fi
if [[ $EXPERIMENT == footprint ]]; then forms=(base lanes); fi
form_flags() {
    local_inline=0
    init_used=0
    ready_burst=0
    case $1 in
        base) spin=256; yields=16; progress=0 ;;
        lanes) spin=256; yields=16; progress=0; init_used=1 ;;
        cq1024|cq16384|cq65536|ch1024|ch16384|ch65536) spin=256; yields=16; progress=0 ;;
        ch1024b1|ch1024b8|ch16384b1|ch16384b8) spin=256; yields=16; progress=0; ready_burst=${1##*b} ;;
        local) spin=256; yields=16; progress=0; local_inline=1 ;;
        sleep) spin=0; yields=0; progress=0 ;;
        short) spin=16; yields=0; progress=0 ;;
        spin) spin=256; yields=0; progress=0 ;;
        poll1) spin=16; yields=0; progress=1 ;;
        poll16) spin=256; yields=0; progress=16 ;;
        *) return 2 ;;
    esac
}
link_form() {
    local module=$1 output=$2 policy=$3 observed=$4
    local observer=() spin yields progress local_inline init_used ready_burst
    form_flags "$policy"
    if [[ $observed == 1 ]]; then observer=("$BACKEND/sched/grant_observer.c"); fi
    "$CLANG" -std=c11 -O2 -pthread -I "$BACKEND" -I "$BACKEND/completion" \
        -I "$BACKEND/sched" "-DWF_SCHED_IDLE_SPIN_ROUNDS=$spin" \
        "-DWF_SCHED_IDLE_YIELD_ROUNDS=$yields" "-DWF_SCHED_IDLE_PROGRESS_INTERVAL=$progress" \
        "-DWF_SCHED_OBSERVE=$observed" "-DWF_COMPLETION_LOCAL_INLINE=$local_inline" \
        "-DWF_SCHED_INIT_USED_LANES=$init_used" "-DWF_SCHED_COMPLETION_READY_BURST=$ready_burst" \
        -x c "$BACKEND/wf_floor.c" "$BACKEND/sched/core.c" \
        "$BACKEND/sched/prim_host.c" "$BACKEND/sched/entry.c" \
        "$BACKEND/completion/runtime.c" "$BACKEND/completion/wait_host.c" \
        "$BACKEND/completion/file_adapter.c" "$BACKEND/completion/file_posix.c" \
        "$BACKEND/completion/bridge.c" "$BACKEND/completion/linux_io_uring.c" \
        "${observer[@]}" -x ir "$module" -Wno-override-module -lm -o "$output"
}

programs=(echo compute mixed)
echo_source="$HERE/programs/tcp_echo_server.wf"
if [[ $network_compute == 1 ]]; then
    echo_source="$HERE/programs/tcp_compute_server.wf"
    programs=(echo)
elif [[ $EXPERIMENT == inline || $EXPERIMENT == footprint ]]; then
    programs=(echo)
else
    "$WFC" --par --emit-llvm -o "$OUT/compute.ll" "$ROOT/tests/programs/par_layout.wf"
    "$WFC" --par --emit-llvm -o "$OUT/mixed.ll" "$HERE/programs/windows_runtime_mixed.wf"
fi
if [[ $EXPERIMENT == checkpoint || $EXPERIMENT == chunks || $EXPERIMENT == priority ]]; then
    programs=(echo compute mixed)
    "$WFC" --par --emit-llvm -o "$OUT/compute.ll" "$ROOT/tests/programs/par_layout.wf"
    "$WFC" --par --emit-llvm -o "$OUT/mixed.ll" "$HERE/programs/windows_runtime_mixed.wf"
fi
"$WFC" --par --emit-llvm -o "$OUT/echo.ll" "$echo_source"
for policy in "${forms[@]}"; do
    for program in "${programs[@]}"; do
        module="$OUT/$program.ll"
        if [[ $policy == cq* || $policy == ch* ]]; then
            case $program in
                echo) source="$echo_source" ;;
                compute) source="$ROOT/tests/programs/par_layout.wf" ;;
                mixed) source="$HERE/programs/windows_runtime_mixed.wf" ;;
            esac
            module="$OUT/$program-$policy.ll"
            checkpoint_option=--sched-quantum
            if [[ $policy == ch* ]]; then checkpoint_option=--sched-chunks; fi
            checkpoint_interval=${policy:2}
            checkpoint_interval=${checkpoint_interval%b*}
            "$WFC" --par "$checkpoint_option" "$checkpoint_interval" --emit-llvm -o "$module" "$source"
            if [[ $policy == ch*b* ]]; then
                # Only the C queue policy may differ in this paired cohort.
                cmp "$OUT/$program-${policy%b*}.ll" "$module"
            fi
            if [[ $policy == ch* && $program == echo ]]; then
                # This protocol's dependent recurrence must actually use the
                # new lowering; positive runtime calls alone also fit fallback.
                awk '/^define internal i64 @wf_churn\(/ {body=1;seen=1}
                     body && /call void @wf__checkpoint_tick/ {counter=1}
                     body && /call void @wf__sched_checkpoint/ {chunk=1}
                     body && /^}/ {body=0}
                     END {exit !(seen && chunk && !counter)}' "$module"
            fi
        fi
        link_form "$module" "$OUT/bin/$program-$policy" "$policy" 0
    done
    module="$OUT/echo.ll"
    if [[ $policy == cq* || $policy == ch* ]]; then module="$OUT/echo-$policy.ll"; fi
    link_form "$module" "$OUT/bin/echo-$policy-observed" "$policy" 1
    if [[ $EXPERIMENT == chunks && ( $policy == cq16384 || $policy == ch16384 ) ]]; then
        link_form "$OUT/compute-$policy.ll" "$OUT/bin/compute-$policy-observed" "$policy" 1
        WF_WORKERS=4 WF_STACKS=1100 WF_SCHED_REPORT=1 "$OUT/bin/compute-$policy-observed" batch batch batch \
            > "$OUT/compute-$policy-observed.out" 2> "$OUT/compute-$policy-observed.err"
        printf '420a993efa7437a1 41fa962893d45299\n' | cmp - "$OUT/compute-$policy-observed.out"
        # Every hot leaf loop here is shorter than 16384. This control must
        # distinguish code-generation cost from actual scheduler switching.
        awk '/^sched:/ { seen=1; for(i=2;i<=NF;i++) {split($i,a,"=");value[a[1]]=a[2]+0} }
             END { exit !(seen && ("checkpoints" in value) && ("checkpoint_switches" in value) && value["checkpoints"]==0 && value["checkpoint_switches"]==0) }' \
            "$OUT/compute-$policy-observed.err"
    fi
done
if [[ $network_compute == 1 ]]; then
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -DWF_BENCH_COMPUTE \
        "$HERE/epoll_echo.c" -o "$OUT/bin/epoll_compute"
fi
if [[ $EXPERIMENT == fairness || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == priority ]]; then
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -DWF_BENCH_COMPUTE -DWF_BENCH_QUANTUM \
        "$HERE/epoll_echo.c" -o "$OUT/bin/epoll_quantum"
fi
for tool in netload uring_echo epoll_echo runner gen; do
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread "$HERE/$tool.c" -o "$OUT/bin/$tool"
done
"$OUT/bin/gen" "$OUT/tree" 2 65536 fixed

server_pid=''
cleanup() {
    if [[ -n $server_pid ]]; then
        kill -TERM -- "-$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

port_present() {
    local hex
    hex=$(printf '%04X' "$1")
    awk -v port="$hex" -v listening="$2" '
        split($2,a,":") == 2 && a[2] == port && (!listening || $4 == "0A") { found=1 }
        END { exit !found }' /proc/net/tcp /proc/net/tcp6
}
free_port() {
    local candidate
    while :; do
        candidate=$((10000 + RANDOM % 10000))
        if ! port_present "$candidate" 0; then echo "$candidate"; return; fi
    done
}
field() {
    tr '\t' '\n' < "$1" | sed -n "s/^$2=//p"
}
sample=0
network_case() {
    local form=$1 connections=$2 trips=$3 bytes=$4 pass=$5 observed=$6
    local binary environment=() arguments=() directory port
    sample=$((sample + 1))
    directory="$OUT/samples/$sample-$cohort-$form-k$connections-b$bytes-r$compute_rounds-a$admitted-d$duration_ms-l${light_per_second:-0}"
    if [[ $observed == 1 ]]; then directory="$OUT/observed/$cohort-$form-k$connections-a$admitted"; fi
    mkdir -p "$directory"
    port=$(free_port)
    case $form in
        uring|epoll)
            binary="$OUT/bin/${form}_echo"
            if [[ $network_compute == 1 ]]; then binary="$OUT/bin/epoll_compute"; fi
            arguments=(--threads "$server_workers") ;;
        q1024|q16384|q65536)
            binary="$OUT/bin/epoll_quantum"
            arguments=(--threads "$server_workers" --quantum "${form#q}") ;;
        base|local|lanes|sleep|short|spin|poll1|poll16|cq1024|cq16384|cq65536|ch1024|ch16384|ch65536|ch1024b1|ch1024b8|ch16384b1|ch16384b8)
            binary="$OUT/bin/echo-$form"
            if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
            environment=("WF_WORKERS=$server_workers" WF_STACKS=1100 "WF_SCHED_REPORT=$observed") ;;
        *) return 2 ;;
    esac
    echo "sample=$sample pass=$pass cohort=$cohort form=$form connections=$connections compute=$compute_rounds observed=$observed admitted=$admitted"
    setsid timeout --signal=TERM --kill-after=5s 120s \
        /usr/bin/time -f '%U\t%S\t%M\t%w\t%c' -o "$directory/resources.tsv" taskset -c "$server_cpus" \
        env "${environment[@]}" "$binary" "$port" "$connections" "${arguments[@]}" \
        > "$directory/server.out" 2> "$directory/server.err" &
    server_pid=$!
    while ! port_present "$port" 1; do
        if ! kill -0 "$server_pid" 2>/dev/null; then
            cat "$directory/server.err" >&2
            echo "scheduler-bench: $form exited before listening" >&2
            return 1
        fi
        sleep 0.01
    done
    local client_arguments=()
    if [[ $network_compute == 1 ]]; then client_arguments=(--compute "$compute_rounds" --heavy-every 4); fi
    if [[ $admitted == 1 ]]; then client_arguments+=(--admit); fi
    if [[ $duration_ms != 0 ]]; then client_arguments+=(--duration-ms "$duration_ms"); fi
    if [[ ${light_per_second:-0} != 0 ]]; then client_arguments+=(--light-per-second "$light_per_second"); fi
    timeout --signal=TERM --kill-after=5s 120s \
        /usr/bin/time -f '%U\t%S\t%M\t%w\t%c' -o "$directory/client-resources.tsv" taskset -c "$client_cpus" \
        "$OUT/bin/netload" "$port" "$connections" "$trips" "$bytes" --threads "$client_workers" "${client_arguments[@]}" \
        > "$directory/client.tsv" 2> "$directory/client.err"
    if ! wait "$server_pid"; then cat "$directory/server.err" >&2; return 1; fi
    server_pid=''
    [[ ! -s $directory/server.out && ! -s $directory/client.err ]]
    if [[ $observed == 0 ]]; then [[ ! -s $directory/server.err ]]; fi
    if [[ $pass -ge 0 ]]; then
        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
            "$pass" "$form" "$connections" "$bytes" "$trips" \
            "$(field "$directory/client.tsv" rt_per_s)" \
            "$(field "$directory/client.tsv" p50_us)" "$(field "$directory/client.tsv" p99_us)" \
            "$(cat "$directory/resources.tsv")" "$directory" "$cohort" \
            "$(cat "$directory/client-resources.tsv")" "$compute_rounds" \
            "$(field "$directory/client.tsv" light_p99_us)" "$(field "$directory/client.tsv" heavy_p99_us)" \
            "$(field "$directory/client.tsv" light_span_us)" "$(field "$directory/client.tsv" heavy_span_us)" \
            "$(field "$directory/client.tsv" client_exchange_user_us)" "$(field "$directory/client.tsv" client_exchange_system_us)" "$admitted" \
            "$(field "$directory/client.tsv" roundtrips)" "$duration_ms" "$(field "$directory/client.tsv" exchange_us)" \
            "$(field "$directory/client.tsv" drain_us)" "$(field "$directory/client.tsv" light_count)" \
            "$(field "$directory/client.tsv" heavy_count)" \
            "$(field "$directory/client.tsv" light_min_count)" "$(field "$directory/client.tsv" light_worst_peer_p99_us)" \
            "$(field "$directory/client.tsv" heavy_min_count)" "$(field "$directory/client.tsv" heavy_worst_peer_p99_us)" \
            "${light_per_second:-0}" "$(field "$directory/client.tsv" light_planned)" \
            "$(field "$directory/client.tsv" light_dispatch_p99_us)" "$(field "$directory/client.tsv" light_service_p99_us)" \
            "$(field "$directory/client.tsv" light_completed_by_deadline)" "$(field "$directory/client.tsv" heavy_completed_by_deadline)" >> "$OUT/network.tsv"
    fi
}

# Correctness and migration counters are outside the timed cohort. Verify the
# io_uring reference as well: absence of the native engine is a failed run.
references=(uring epoll)
compute_rounds=0
admitted=0
duration_ms=0
light_per_second=0
admissions=(0)
if [[ $EXPERIMENT == fairness ]]; then admissions=(0 1); fi
if [[ $network_compute == 1 ]]; then references=(epoll); compute_rounds=262144; fi
if [[ $EXPERIMENT == fairness || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks ]]; then references=(epoll q1024 q16384 q65536); fi
if [[ $EXPERIMENT == priority ]]; then references=(q1024 q16384); fi
while IFS=$'\t' read -r cohort server_workers client_workers server_cpus client_cpus; do
    for admitted in "${admissions[@]}"; do
        for form in "${references[@]}" "${forms[@]}"; do network_case "$form" 4 20 64 -1 0; done
    done
    admitted=0
    for form in "${forms[@]}"; do
      for connections in 4 64; do
        observed_trips=2000
        if [[ $network_compute == 1 ]]; then observed_trips=64; fi
        network_case "$form" "$connections" "$observed_trips" 64 -1 1
        # A Linux reading must actually use the native ring. Keep these
        # observations separate from the timed binaries and their diagnostics.
        awk '/^sched:/ { scheduler=1 }
             /^ring:/ { ring=1; for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
             END { exit !(scheduler && ring && value["submissions"] > 0 && value["completions"] > 0) }' \
            "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        if [[ ( $form == cq* || $form == ch* ) && $connections == 64 ]]; then
            awk '/^sched:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(value["checkpoints"] > 0 && value["checkpoint_switches"] > 0) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
        if [[ $EXPERIMENT == priority && $connections == 64 ]]; then
            burst=0
            if [[ $form == ch*b* ]]; then burst=${form##*b}; fi
            # Observe the route separately from timings. Both sources of
            # readiness must be exercised; selection counts remain evidence,
            # not an assumption that every run reaches a particular ordering.
            awk -v burst="$burst" '/^sched:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(("completion_ready_burst" in value) && value["completion_ready_burst"]==burst && value["ready_completions"]>0 && value["ready_yields"]>0) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
      done
    done
done < "$OUT/cohorts.tsv"

printf 'pass\tform\tconnections\tbytes\ttrips\trt_per_s\tp50_us\tp99_us\tuser_s\tsystem_s\tmax_rss_kib\tvoluntary_switches\tinvoluntary_switches\tsample\tcohort\tclient_user_s\tclient_system_s\tclient_max_rss_kib\tclient_voluntary_switches\tclient_involuntary_switches\tcompute_rounds\tlight_p99_us\theavy_p99_us\tlight_span_us\theavy_span_us\tclient_exchange_user_us\tclient_exchange_system_us\tadmitted\ttotal_roundtrips\tduration_ms\texchange_us\tdrain_us\tlight_count\theavy_count\tlight_min_count\tlight_worst_peer_p99_us\theavy_min_count\theavy_worst_peer_p99_us\tlight_per_second\tlight_planned\tlight_dispatch_p99_us\tlight_service_p99_us\tlight_completed_by_deadline\theavy_completed_by_deadline\n' > "$OUT/network.tsv"
forward=("${forms[@]}" "${references[@]}")
reverse=()
for ((at=${#forward[@]}-1;at>=0;at--)); do reverse+=("${forward[at]}"); done
if [[ $network_compute == 1 ]]; then
    cat > "$OUT/cases.tsv" <<'CASES'
4 2000 64 0
64 2000 64 0
4 256 64 16384
64 64 64 16384
4 256 64 262144
64 64 64 262144
4 128 64 2097152
64 32 64 2097152
CASES
else
    cat > "$OUT/cases.tsv" <<'CASES'
1 10000 64 0
4 10000 64 0
64 2000 64 0
CASES
fi
if [[ $EXPERIMENT == inline || $EXPERIMENT == footprint ]]; then
    printf '1024 200 64 0\n64 500 65536 0\n' >> "$OUT/cases.tsv"
fi
if [[ $EXPERIMENT == fairness ]]; then
    # Zero-compute control plus two sustained compute costs; retain both peer counts.
    awk '$4 != 16384' "$OUT/cases.tsv" > "$OUT/cases-selected.tsv"
    mv "$OUT/cases-selected.tsv" "$OUT/cases.tsv"
fi
if [[ $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == priority ]]; then
    # Both request classes stay active for a common interval. The count is a
    # storage ceiling, not a target; an early ceiling hit fails the sample.
    cat > "$OUT/cases.tsv" <<'CASES'
4 100000 64 0
64 100000 64 0
4 100000 64 262144
64 100000 64 262144
4 100000 64 2097152
64 100000 64 2097152
CASES
    admissions=(1)
    duration_ms=1000
fi
if [[ $EXPERIMENT == chunks ]]; then
    awk '$1==64' "$OUT/cases.tsv" > "$OUT/cases-selected.tsv"
    mv "$OUT/cases-selected.tsv" "$OUT/cases.tsv"
fi
if [[ $EXPERIMENT == paced || $EXPERIMENT == priority ]]; then
    # Fix light arrivals independently of service speed. Keep every scheduled
    # request, including client backlog, while heavy peers remain saturated.
    cat > "$OUT/cases.tsv" <<'CASES'
64 100000 64 0 100
64 100000 64 262144 20
64 100000 64 262144 100
64 100000 64 262144 500
64 100000 64 2097152 20
64 100000 64 2097152 100
64 100000 64 2097152 500
CASES
fi
if [[ $EXPERIMENT == priority ]]; then
    # Retain all long-compute arrival rates and one shorter-compute control.
    awk '$4!=262144 || $5==100' "$OUT/cases.tsv" > "$OUT/cases-selected.tsv"
    mv "$OUT/cases-selected.tsv" "$OUT/cases.tsv"
fi
for ((pass=-WARMUP; pass<ROUNDS; pass++)); do
    order=("${forward[@]}")
    if (( (pass + WARMUP) % 2 )); then order=("${reverse[@]}"); fi
  while IFS=$'\t' read -r cohort server_workers client_workers server_cpus client_cpus; do
    while read -r connections trips bytes compute_rounds light_per_second; do
      for admitted in "${admissions[@]}"; do
        for form in "${order[@]}"; do
            echo "network pass=$pass cohort=$cohort form=$form connections=$connections bytes=$bytes compute=$compute_rounds"
            network_case "$form" "$connections" "$trips" "$bytes" "$pass" 0
        done
      done
    done < "$OUT/cases.tsv"
  done < "$OUT/cohorts.tsv"
done

# Existing compiler-independent expected bytes from the Windows qualification.
# Warm positioned reads plus compute measure coexistence; they do not establish
# a bound on network latency while every worker runs a long computation.
cpu_programs=(compute mixed)
if [[ $EXPERIMENT != idle && $EXPERIMENT != checkpoint && $EXPERIMENT != chunks && $EXPERIMENT != priority ]]; then cpu_programs=(); fi
for program in "${cpu_programs[@]}"; do
    : > "$OUT/$program.plan"
    for workers in 2 4 8; do
        for policy in "${forms[@]}"; do
            printf '%s.w%s.p%s\tWF_WORKERS=%s,WF_STACKS=1100\t%s' \
                "$program" "$workers" "$policy" "$workers" "$OUT/bin/$program-$policy" >> "$OUT/$program.plan"
            if [[ $program == mixed ]]; then
                printf '\tf00000.dat\tf00001.dat' >> "$OUT/$program.plan"
            else
                printf '\tbatch\tbatch\tbatch' >> "$OUT/$program.plan"
            fi
            printf '\n' >> "$OUT/$program.plan"
        done
    done
    expected='420a993efa7437a1 41fa962893d45299'
    if [[ $program == mixed ]]; then expected='17574306422404092952'; fi
    (cd "$OUT/tree" && "$OUT/bin/runner" "$OUT/$program.plan" "$ROUNDS" "$WARMUP" "$expected") \
        > "$OUT/$program.txt" 2> "$OUT/$program.err"
done

# Keep raw samples; summarize ranges as well as medians. Priority ratios use
# the single FIFO at the matching chunk interval; other experiments use base.
# Every denominator is measured in the same pass and complete workload cohort.
awk -F '\t' -v priority="$([[ $EXPERIMENT == priority ]] && echo 1 || echo 0)" '
    NR == 1 { next }
    { key=$15 "/" $3 "/" $4 "/" ($21+0) "/" ($28+0) "/" ($30+0) "/" ($39+0); cohort=$1 SUBSEP key; group=$2 SUBSEP key
      count[group]++; rates[group,count[group]]=$6; lat[group,count[group]]=$7;
      tail[group,count[group]]=$8; cpu[group,count[group]]=($9+$10)*1e6/$29;
      rss[group,count[group]]=$11; switches[group,count[group]]=($12+$13)/$29;
      client_cpu[group,count[group]]=($16+$17)*1e6/$29;
      light_tail[group,count[group]]=$22+0; heavy_tail[group,count[group]]=$23+0;
      exchange_cpu[group,count[group]]=($26+$27)/$29;
      light_rate[group,count[group]]=($33+0)*1e6/$31;
      heavy_rate[group,count[group]]=($34+0)*1e6/$31;
      light_min[group,count[group]]=$35+0; light_worst[group,count[group]]=$36+0;
      heavy_min[group,count[group]]=$37+0; heavy_worst[group,count[group]]=$38+0;
      dispatch[group,count[group]]=$41+0; service[group,count[group]]=$42+0;
      light_ontime[group,count[group]]=$30 ? ($43+0)*1000/$30 : 0;
      heavy_ontime[group,count[group]]=$30 ? ($44+0)*1000/$30 : 0;
      backlog[group,count[group]]=$39 ? $40-$43 : 0;
      samples[cohort,$2]=$6; groups[group]=1; cohorts[cohort]=1; }
    function reference(form) {
      if (!priority) return "base";
      sub(/b[18]$/, "", form); sub(/^q/, "ch", form); return form;
    }
    function summary(values,g, n,i,j,t) {
      n=count[g]; for(i=1;i<=n;i++) sorted[i]=values[g,i];
      for(i=1;i<=n;i++) for(j=i+1;j<=n;j++) if(sorted[j]<sorted[i]) {t=sorted[i];sorted[i]=sorted[j];sorted[j]=t}
      return n%2 ? sorted[(n+1)/2] : (sorted[n/2]+sorted[n/2+1])/2;
    }
    END {
      print "form case median_rt/s min_rt/s max_rt/s p50_us p99_us server_cpu_us/trip peak_rss_kib switches/trip paired_rate_ratio client_lifetime_cpu_us/trip light_p99_us heavy_p99_us client_exchange_cpu_us/trip light_rt/s heavy_rt/s light_min_count light_worst_peer_p99_us heavy_min_count heavy_worst_peer_p99_us light_dispatch_p99_us light_service_p99_us light_before_deadline/s heavy_before_deadline/s light_pending_at_deadline";
      for(g in groups) {
        split(g,parts,SUBSEP); form=parts[1]; key=parts[2]; n=0;
        for(c in cohorts) { split(c,p,SUBSEP); if(p[2]==key) {
          denominator=samples[c,reference(form)];
          if(denominator<=0) { print "scheduler-bench: missing paired reference" > "/dev/stderr"; exit 1 }
          ratios[g,++n]=samples[c,form]/denominator;
        } }
        rate=summary(rates,g); low=sorted[1]; high=sorted[count[g]];
        printf "%s %s %.1f %.1f %.1f %.1f %.1f %.3f %.0f %.4f %.3f %.3f %.1f %.1f %.3f %.1f %.1f %.0f %.0f %.0f %.0f %.0f %.0f %.1f %.1f %.0f\n", form,key,rate,low,high,
          summary(lat,g),summary(tail,g),summary(cpu,g),summary(rss,g),summary(switches,g),summary(ratios,g),summary(client_cpu,g),
          summary(light_tail,g),summary(heavy_tail,g),summary(exchange_cpu,g),summary(light_rate,g),summary(heavy_rate,g),
          summary(light_min,g),summary(light_worst,g),summary(heavy_min,g),summary(heavy_worst,g),
          summary(dispatch,g),summary(service,g),summary(light_ontime,g),summary(heavy_ontime,g),summary(backlog,g);
      }
    }' "$OUT/network.tsv" > "$OUT/network-summary.txt"
cat "$OUT/network-summary.txt"
for program in "${cpu_programs[@]}"; do cat "$OUT/$program.txt"; done
