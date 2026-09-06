#!/usr/bin/env bash
# Paired scheduler-locality experiment. The same emitted programs, stack
# representation, mutex, completion engine and worker count are used by every
# policy. Owned by io-model/SCHEDULER-EXPERIMENT.md; retire after that decision.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=${ROOT:-$(cd "$HERE/../../.." && pwd)}
OUT=${OUT:-${WHITEFOOT_SCRATCH_ROOT:-$HOME/do_not_scan}/whitefoot-scheduler-experiment}
CLANG=${CLANG:-/usr/bin/clang}
BACKEND=$ROOT/compiler/src/backend
MODE=${1:-bench}
ROUNDS=${ROUNDS:-7}
WARMUP=${WARMUP:-2}
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)

check_policy() {
        local policy=$1 configuration threads stacks log
        "$CLANG" -std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic \
            -DWF_SCHED_ENUMERATE -DWF_SCHED_LANE_SLOTS=2u \
            -DWF_SCHED_MAX_THREADS=4u -DWF_SCHED_MAX_STACKS=8u \
            -DWF_SCHED_IDLE_SPIN_ROUNDS=1u -DWF_SCHED_IDLE_YIELD_ROUNDS=0u \
            "-DWF_SCHED_READY_POLICY=$policy" \
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
    # Every existing schedule and every design configuration, for all three
    # policies. Independent policies run together to keep the CI stage short.
    # Every child's exit is checked; no schedule or configuration is omitted.
    local policy pid status=0 pids=()
    for policy in 0 1 2; do
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
[[ $ROUNDS =~ ^[1-9][0-9]*$ && $WARMUP =~ ^[0-9]+$ ]] || exit 2
WORKERS=${WORKERS:-$(nproc)}
[[ $WORKERS =~ ^[1-9][0-9]*$ && $WORKERS -ge 2 && $WORKERS -le 64 ]] || exit 2
mkdir -p "$OUT/bin" "$OUT/samples" "$OUT/observed" "$OUT/tree"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/compiler/target}
(cd "$ROOT/compiler" && cargo build --profile gate --bin whitefootc --locked --offline)
WFC=$CARGO_TARGET_DIR/gate/whitefootc

{
    git -C "$ROOT" rev-parse HEAD
    uname -a
    "$CLANG" --version
    lscpu
    echo "workers=$WORKERS rounds=$ROUNDS warmup=$WARMUP"
    echo "io_uring_disabled=$(cat /proc/sys/kernel/io_uring_disabled 2>/dev/null || echo absent)"
    echo "affinity=$(taskset -pc $$)"
} > "$OUT/host.txt"

link_form() {
    local module=$1 output=$2 policy=$3 observed=$4
    local observer=()
    if [[ $observed == 1 ]]; then observer=("$BACKEND/sched/grant_observer.c"); fi
    "$CLANG" -std=c11 -O2 -pthread -I "$BACKEND" -I "$BACKEND/completion" \
        -I "$BACKEND/sched" "-DWF_SCHED_READY_POLICY=$policy" \
        "-DWF_SCHED_OBSERVE_RESUMES=$observed" \
        -x c "$BACKEND/wf_floor.c" "$BACKEND/sched/core.c" \
        "$BACKEND/sched/prim_host.c" "$BACKEND/sched/entry.c" \
        "$BACKEND/completion/runtime.c" "$BACKEND/completion/wait_host.c" \
        "$BACKEND/completion/file_adapter.c" "$BACKEND/completion/file_posix.c" \
        "$BACKEND/completion/bridge.c" "$BACKEND/completion/linux_io_uring.c" \
        "${observer[@]}" -x ir "$module" -Wno-override-module -lm -o "$output"
}

"$WFC" --par --emit-llvm -o "$OUT/echo.ll" "$HERE/programs/tcp_echo_server.wf"
"$WFC" --par --emit-llvm -o "$OUT/compute.ll" "$ROOT/tests/programs/par_layout.wf"
"$WFC" --par --emit-llvm -o "$OUT/mixed.ll" "$HERE/programs/windows_runtime_mixed.wf"
for policy in 0 1 2; do
    for program in echo compute mixed; do
        link_form "$OUT/$program.ll" "$OUT/bin/$program-$policy" "$policy" 0
    done
    link_form "$OUT/echo.ll" "$OUT/bin/echo-$policy-observed" "$policy" 1
done
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
    directory="$OUT/samples/$sample-$form-k$connections-b$bytes"
    if [[ $observed == 1 ]]; then directory="$OUT/observed/$form-k$connections"; fi
    mkdir -p "$directory"
    port=$(free_port)
    case $form in
        uring|epoll)
            binary="$OUT/bin/${form}_echo"
            arguments=(--threads "$WORKERS") ;;
        0|1|2)
            binary="$OUT/bin/echo-$form"
            if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
            environment=("WF_WORKERS=$WORKERS" WF_STACKS=1100 "WF_SCHED_REPORT=$observed") ;;
        *) return 2 ;;
    esac
    setsid /usr/bin/time -f '%U\t%S\t%M\t%w\t%c' -o "$directory/resources.tsv" \
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
    "$OUT/bin/netload" "$port" "$connections" "$trips" "$bytes" --threads "$WORKERS" \
        > "$directory/client.tsv" 2> "$directory/client.err"
    if ! wait "$server_pid"; then cat "$directory/server.err" >&2; return 1; fi
    server_pid=''
    [[ ! -s $directory/server.out && ! -s $directory/client.err ]]
    if [[ $observed == 0 ]]; then [[ ! -s $directory/server.err ]]; fi
    if [[ $pass -ge 0 ]]; then
        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
            "$pass" "$form" "$connections" "$bytes" "$trips" \
            "$(field "$directory/client.tsv" rt_per_s)" \
            "$(field "$directory/client.tsv" p50_us)" "$(field "$directory/client.tsv" p99_us)" \
            "$(cat "$directory/resources.tsv")" "$directory" >> "$OUT/network.tsv"
    fi
}

# Correctness and migration counters are outside the timed cohort. Verify the
# io_uring reference as well: absence of the native engine is a failed run.
for form in uring epoll 0 1 2; do network_case "$form" 4 200 64 -1 0; done
for form in 0 1 2; do
    for connections in 4 64; do
        network_case "$form" "$connections" 2000 64 -1 1
        # A Linux reading must actually use the native ring. Keep these
        # observations separate from the timed binaries and their diagnostics.
        awk '/^ring:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
             END { exit !(value["submissions"] > 0 && value["completions"] > 0) }' \
            "$OUT/observed/$form-k$connections/server.err"
    done
done

printf 'pass\tform\tconnections\tbytes\ttrips\trt_per_s\tp50_us\tp99_us\tuser_s\tsystem_s\tmax_rss_kib\tvoluntary_switches\tinvoluntary_switches\tsample\n' > "$OUT/network.tsv"
forms=(0 1 2 uring epoll)
reverse=(epoll uring 2 1 0)
for ((pass=-WARMUP; pass<ROUNDS; pass++)); do
    order=("${forms[@]}")
    if (( (pass + WARMUP) % 2 )); then order=("${reverse[@]}"); fi
    while read -r connections trips bytes; do
        for form in "${order[@]}"; do
            echo "network pass=$pass form=$form connections=$connections bytes=$bytes"
            network_case "$form" "$connections" "$trips" "$bytes" "$pass" 0
        done
    done <<'CASES'
1 20000 64
4 20000 64
64 2000 64
1024 200 64
64 200 65536
CASES
done

# Existing compiler-independent expected bytes from the Windows qualification.
# Warm positioned reads plus compute measure coexistence; they do not establish
# a bound on network latency while every worker runs a long computation.
for program in compute mixed; do
    : > "$OUT/$program.plan"
    for workers in 2 4 8; do
        for policy in 0 1 2; do
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

# Keep raw samples; summarize ranges as well as medians. All rate ratios use
# the global FIFO measured in the same pass, connection count and payload.
awk -F '\t' '
    NR == 1 { next }
    { key=$3 "/" $4; cohort=$1 SUBSEP key; group=$2 SUBSEP key
      count[group]++; rates[group,count[group]]=$6; lat[group,count[group]]=$7;
      tail[group,count[group]]=$8; cpu[group,count[group]]=($9+$10)*1e6/($3*$5);
      rss[group,count[group]]=$11; switches[group,count[group]]=($12+$13)/($3*$5);
      if ($2 == "0") base[cohort]=$6;
      samples[cohort,$2]=$6; groups[group]=1; cohorts[cohort]=1; }
    function summary(values,g, n,i,j,t) {
      n=count[g]; for(i=1;i<=n;i++) sorted[i]=values[g,i];
      for(i=1;i<=n;i++) for(j=i+1;j<=n;j++) if(sorted[j]<sorted[i]) {t=sorted[i];sorted[i]=sorted[j];sorted[j]=t}
      return n%2 ? sorted[(n+1)/2] : (sorted[n/2]+sorted[n/2+1])/2;
    }
    END {
      print "form case median_rt/s min_rt/s max_rt/s p50_us p99_us server_cpu_us/trip peak_rss_kib switches/trip paired_rate_ratio";
      for(g in groups) {
        split(g,parts,SUBSEP); form=parts[1]; key=parts[2]; n=0;
        for(c in cohorts) { split(c,p,SUBSEP); if(p[2]==key) ratios[g,++n]=samples[c,form]/base[c]; }
        rate=summary(rates,g); low=sorted[1]; high=sorted[count[g]];
        printf "%s %s %.1f %.1f %.1f %.1f %.1f %.3f %.0f %.4f %.3f\n", form,key,rate,low,high,
          summary(lat,g),summary(tail,g),summary(cpu,g),summary(rss,g),summary(switches,g),summary(ratios,g);
      }
    }' "$OUT/network.tsv" > "$OUT/network-summary.txt"
cat "$OUT/network-summary.txt" "$OUT/compute.txt" "$OUT/mixed.txt"
