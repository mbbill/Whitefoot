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
client_experiment=0
if [[ $MODE == client || $MODE == placement ]]; then client_experiment=1; fi
profile_active=0
PROFILE_PERF=${PROFILE_PERF:-perf}
ROUNDS=${ROUNDS:-7}
WARMUP=${WARMUP:-2}
EXPERIMENT=${EXPERIMENT:-idle}
NATIVE_BASELINES=${NATIVE_BASELINES:-0}
[[ $NATIVE_BASELINES == 0 || $NATIVE_BASELINES == 1 ]] || exit 2
if [[ $NATIVE_BASELINES == 1 && ( $MODE != combine || $EXPERIMENT != allocator ) ]]; then
    echo 'scheduler-bench: native baseline panel uses combine with EXPERIMENT=allocator' >&2
    exit 2
fi
allocation_experiment=0
if [[ $EXPERIMENT == allocation || $EXPERIMENT == allocator ]]; then allocation_experiment=1; fi
page_experiment=0
if [[ $EXPERIMENT == pages || $allocation_experiment == 1 ]]; then page_experiment=1; fi
storage_experiment=0
if [[ $EXPERIMENT == storage || $page_experiment == 1 || $EXPERIMENT == coroutine ]]; then storage_experiment=1; fi
coroutine_experiment=0
if [[ $EXPERIMENT == coroutine || $EXPERIMENT == coroutine-paced || $MODE == combine ]]; then coroutine_experiment=1; fi
CORO_CXX=${CORO_CXX:-$(command -v clang++-20 || command -v clang++)}
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
    if [[ $(uname -s) == Linux ]]; then
        make -C "$HERE" compiler-continuation-check CLANG="$CLANG" CORO_CXX="$CORO_CXX" \
            WHITEFOOT_SCRATCH_ROOT="$OUT/completion-coroutine-check"
        make -C "$HERE" stackful-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/stream-check"
        make -C "$HERE" uring-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/uring-check"
        make -C "$HERE" coroutine-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/coroutine-check"
        make -C "$HERE" client-service-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/client-check"
    fi
    exit 0
fi
if [[ ( $MODE != bench && $MODE != profile && $MODE != combine && $client_experiment != 1 ) || $(uname -s) != Linux ]]; then
    echo 'scheduler-bench: use check on POSIX, or bench/profile/client/placement/combine on Linux with io_uring' >&2
    exit 2
fi
if [[ $MODE == combine && $EXPERIMENT != allocator ]]; then
    echo 'scheduler-bench: combine uses the qualified allocator echo workload' >&2
    exit 2
fi
if [[ ( $MODE == profile || $client_experiment == 1 ) && $EXPERIMENT != coroutine-paced ]]; then
    echo 'scheduler-bench: profile/client/placement uses the qualified coroutine-paced workload' >&2
    exit 2
fi
[[ $EXPERIMENT == idle || $EXPERIMENT == mixed || $EXPERIMENT == fairness || $EXPERIMENT == inline || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == footprint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == canonical || $EXPERIMENT == stackful || $EXPERIMENT == stackful-paced || $EXPERIMENT == nodelay || $EXPERIMENT == owner || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced || $EXPERIMENT == memory || $EXPERIMENT == dispatch || $EXPERIMENT == wake || $EXPERIMENT == service || $storage_experiment == 1 || $coroutine_experiment == 1 ]] || exit 2
network_compute=0
if [[ $EXPERIMENT == mixed || $EXPERIMENT == fairness || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == canonical ]]; then network_compute=1; fi
if [[ $EXPERIMENT == stackful-paced || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then network_compute=1; fi
[[ $ROUNDS =~ ^[1-9][0-9]*$ && $WARMUP =~ ^[0-9]+$ ]] || exit 2
[[ $(nproc) -ge 4 ]] || { echo 'scheduler-bench: this CPU-placement experiment needs four logical CPUs' >&2; exit 2; }
mkdir -p "$OUT/bin" "$OUT/samples" "$OUT/observed" "$OUT/tree"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/compiler/target}
(cd "$ROOT/compiler" && cargo build --profile gate --bin whitefootc --locked --offline)
WFC=$CARGO_TARGET_DIR/gate/whitefootc

if [[ $EXPERIMENT == canonical ]]; then
    # Compare the prior lowering on this host, not a prior host's timing.
    # Only compiler inputs are extracted; every binary links the current C
    # runtime below. Remove this baseline build when the lowering is settled.
    checkpoint_baseline=6380a17a800163c1ebd8c63ec1235432c444846b
    git -C "$ROOT" cat-file -e "$checkpoint_baseline^{commit}"
    mkdir -p "$OUT/compiler-before" "$OUT/codegen"
    git -C "$ROOT" archive "$checkpoint_baseline" compiler spec | tar -x -C "$OUT/compiler-before"
    (cd "$OUT/compiler-before/compiler" && CARGO_TARGET_DIR="$OUT/compiler-before/target" \
        cargo build --profile gate --bin whitefootc --locked --offline)
    OLD_WFC=$OUT/compiler-before/target/gate/whitefootc
fi

if [[ $EXPERIMENT == inline || $EXPERIMENT == footprint || $EXPERIMENT == nodelay ]]; then
    # The experimental path keeps every existing harness assertion, including
    # completion counts, and also runs the core's maintained enumeration.
    check_define='-DWF_COMPLETION_LOCAL_INLINE=1'
    if [[ $EXPERIMENT == footprint ]]; then check_define='-DWF_SCHED_INIT_USED_LANES=1'; fi
    if [[ $EXPERIMENT == nodelay ]]; then check_define='-DWF_TCP_NODELAY=1'; fi
    if ! make -C "$ROOT/compiler" completion-test CC="$CLANG" \
        COMPLETION_TMP="$OUT/$EXPERIMENT-check" \
        COMPLETION_BASE_CFLAGS="-std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread $check_define" \
        > "$OUT/$EXPERIMENT-check.log" 2>&1; then
        cat "$OUT/$EXPERIMENT-check.log"
        exit 1
    fi
fi

if [[ $EXPERIMENT == memory || $storage_experiment == 1 ]]; then
    memory_policies=(lanes compact small)
    if [[ $storage_experiment == 1 ]]; then memory_policies=(small); fi
    for policy in "${memory_policies[@]}"; do
        case $policy in
            lanes) candidate_flags='-DWF_SCHED_INIT_USED_LANES=1' ;;
            compact) candidate_flags='-DWF_SCHED_COMPACT_STACKS=1' ;;
            small) candidate_flags='-DWF_SCHED_COMPACT_STACKS=1 -DWF_SCHED_INIT_USED_LANES=1' ;;
        esac
        if ! make -C "$ROOT/compiler" completion-test CC="$CLANG" \
            COMPLETION_TMP="$OUT/$policy-check" \
            COMPLETION_BASE_CFLAGS="-std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread -DWF_TCP_NODELAY=1 $candidate_flags" \
            > "$OUT/$policy-check.log" 2>&1; then
            cat "$OUT/$policy-check.log"
            exit 1
        fi
    done
fi

if [[ $EXPERIMENT == owner || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch || $EXPERIMENT == wake || $EXPERIMENT == service || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced || $MODE == combine ]]; then
    candidates=(pinned rings owner)
    if [[ $EXPERIMENT == dispatch || $EXPERIMENT == dispatch-paced ]]; then candidates=(rings owner balanced); fi
    if [[ $EXPERIMENT == wake || $EXPERIMENT == wake-paced ]]; then candidates=(rings balanced quiet); fi
    if [[ $EXPERIMENT == service || $EXPERIMENT == service-paced ]]; then candidates=(balanced service1 service16 servicepoll16); fi
    if [[ $EXPERIMENT == coroutine-paced ]]; then candidates=(balanced); fi
    if [[ $MODE == combine ]]; then candidates=(balanced balanced-small quiet-small); fi
    for policy in "${candidates[@]}"; do
        case $policy in
            pinned) candidate_flags='-DWF_SCHED_READY_SHARDS=2 -DWF_SCHED_READY_PINNED=1' ;;
            rings) candidate_flags='-DWF_IO_OWNER_RINGS=1' ;;
            owner) candidate_flags='-DWF_SCHED_READY_SHARDS=2 -DWF_SCHED_READY_PINNED=1 -DWF_IO_OWNER_RINGS=1' ;;
            balanced|balanced-small) candidate_flags='-DWF_SCHED_IO_ROUND_ROBIN=1 -DWF_SCHED_READY_SHARDS=2 -DWF_SCHED_READY_PINNED=1 -DWF_IO_OWNER_RINGS=1' ;;
            servicepoll16) candidate_flags="-DWF_SCHED_IO_QUANTUM=16 -DWF_SCHED_IO_RESET_TURN=0 -DWF_SCHED_IO_ROUND_ROBIN=1 -DWF_SCHED_READY_SHARDS=2 -DWF_SCHED_READY_PINNED=1 -DWF_IO_OWNER_RINGS=1" ;;
            service1|service16) candidate_flags="-DWF_SCHED_IO_QUANTUM=${policy#service} -DWF_SCHED_IO_ROUND_ROBIN=1 -DWF_SCHED_READY_SHARDS=2 -DWF_SCHED_READY_PINNED=1 -DWF_IO_OWNER_RINGS=1" ;;
            quiet|quiet-small) candidate_flags='-DWF_SCHED_LOCAL_WAKE=1 -DWF_SCHED_IO_ROUND_ROBIN=1 -DWF_SCHED_READY_SHARDS=2 -DWF_SCHED_READY_PINNED=1 -DWF_IO_OWNER_RINGS=1' ;;
        esac
        if [[ $policy == *-small ]]; then candidate_flags+=' -DWF_SCHED_COMPACT_STACKS=1 -DWF_SCHED_INIT_USED_LANES=1'; fi
        if ! make -C "$ROOT/compiler" completion-test CC="$CLANG" \
            COMPLETION_TMP="$OUT/$policy-check" \
            COMPLETION_BASE_CFLAGS="-std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread -DWF_TCP_NODELAY=1 $candidate_flags" \
            > "$OUT/$policy-check.log" 2>&1; then
            cat "$OUT/$policy-check.log"
            exit 1
        fi
        if [[ $policy == rings || $policy == owner || $policy == balanced* || $policy == quiet* || $policy == service* ]]; then
            grep -q '^native-adapter-probe two-ring-epoch=pass$' "$OUT/$policy-check.log"
            grep -q '^completion owner-bridge four-thread-read: PASS$' "$OUT/$policy-check.log"
        fi
    done
fi

if [[ $EXPERIMENT == stackful || $EXPERIMENT == stackful-paced || $storage_experiment == 1 || $coroutine_experiment == 1 ]]; then
    if ! make -C "$HERE" stackful-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/stream-check" \
        > "$OUT/stackful-check.log" 2>&1; then
        cat "$OUT/stackful-check.log"
        exit 1
    fi
fi
if [[ $coroutine_experiment == 1 ]]; then
    if ! make -C "$HERE" coroutine-check CLANG="$CLANG" CORO_CXX="$CORO_CXX" \
        WHITEFOOT_SCRATCH_ROOT="$OUT/coroutine-check" > "$OUT/coroutine-check.log" 2>&1; then
        cat "$OUT/coroutine-check.log"
        exit 1
    fi
fi
if [[ $NATIVE_BASELINES == 1 ]]; then
    make -C "$HERE" uring-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/uring-check" \
        > "$OUT/uring-check.log" 2>&1 || { cat "$OUT/uring-check.log"; exit 1; }
fi

{
    git -C "$ROOT" rev-parse HEAD
    uname -a
    "$CLANG" --version
    if [[ $coroutine_experiment == 1 ]]; then "$CORO_CXX" --version; fi
    lscpu
    echo "experiment=$EXPERIMENT mode=$MODE rounds=$ROUNDS warmup=$WARMUP"
    echo "native_baselines=$NATIVE_BASELINES"
    if [[ $NATIVE_BASELINES == 1 ]]; then
        echo 'uring_buffer_policy=8192/65536 bytes; equal provided bytes per worker; counts 256..2048 / 32..256; SQPOLL excluded'
    fi
    if [[ $MODE == profile ]]; then
        "$PROFILE_PERF" --version
        echo 'profile=cpu-clock frequency=999; whole server lifetime, inherited threads, no callchain; not an unprofiled timing panel'
        for setting in perf_event_paranoid kptr_restrict; do
            printf '%s=' "$setting"; cat "/proc/sys/kernel/$setting"
        done
    fi
    if [[ $storage_experiment == 1 ]]; then
        echo "page_bytes=$(getconf PAGESIZE)"
        getconf GNU_LIBC_VERSION
    fi
    if [[ $page_experiment == 1 || $coroutine_experiment == 1 ]]; then
        if [[ $EXPERIMENT == allocator ]]; then
            echo 'page_policy=per-process PR_SET_THP_DISABLE=1; allocator_policy=glibc.malloc.top_pad=131072/0; timed server environments only'
            if [[ $MODE == combine ]]; then echo 'combined_policy=top_pad=0 only; accepted-handler buffers; allocator defaults remain unchanged'; fi
        elif [[ $page_experiment == 1 ]]; then
            echo 'page_policy=per-process PR_SET_THP_DISABLE=0/1; no global policy changes'
        fi
        for setting in /sys/kernel/mm/transparent_hugepage/enabled \
            /sys/kernel/mm/transparent_hugepage/defrag \
            /sys/kernel/mm/transparent_hugepage/hpage_pmd_size \
            /sys/kernel/mm/transparent_hugepage/hugepages-*/enabled; do
            if [[ -r $setting ]]; then printf '%s: ' "$setting"; cat "$setting"; fi
        done
    fi
    if [[ $EXPERIMENT == canonical ]]; then echo "checkpoint_baseline=$checkpoint_baseline"; fi
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

if [[ $storage_experiment == 1 || $coroutine_experiment == 1 ]]; then
    # Attribute storage on one/two server workers before broader topology work.
    awk '$1=="split1" || $1=="split2"' "$OUT/cohorts.tsv" > "$OUT/cohorts-selected.tsv"
    mv "$OUT/cohorts-selected.tsv" "$OUT/cohorts.tsv"
fi
if [[ $MODE == placement ]]; then
    # Keep two workers and two logical CPUs per role, but stop the client and
    # server from sharing physical cores. Require the topology being tested.
    [[ ${physical_groups[0]} == *,* && ${physical_groups[1]} == *,* ]] || {
        echo 'scheduler-bench: placement requires two physical cores with SMT siblings' >&2
        exit 2
    }
    IFS=, read -r first second rest <<< "${physical_groups[0]}"
    server_separate="$first,$second"
    IFS=, read -r first second rest <<< "${physical_groups[1]}"
    client_separate="$first,$second"
    printf 'separate2\t2\t2\t%s\t%s\n' "$server_separate" "$client_separate" >> "$OUT/cohorts.tsv"
fi
if [[ $page_experiment == 1 ]]; then
    # Both policies inherit the same host setting; disable=0 permits THP,
    # rather than forcing a huge-page allocation. Keep policy in each row.
    if [[ $EXPERIMENT == allocator ]]; then
        awk -v mode="$MODE" 'BEGIN {OFS="\t"} {name=$1;
            if(mode!="combine") {$1=name "-no-thp"; print}; $1=name "-top0-no-thp"; print}' \
            "$OUT/cohorts.tsv" > "$OUT/cohorts-selected.tsv"
    else
        awk 'BEGIN {OFS="\t"} {print; $1=$1 "-no-thp"; print}' "$OUT/cohorts.tsv" > "$OUT/cohorts-selected.tsv"
    fi
    mv "$OUT/cohorts-selected.tsv" "$OUT/cohorts.tsv"
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -pthread \
        "$HERE/stream_check.c" -o "$OUT/bin/stream_check"
fi
if [[ $client_experiment == 1 ]]; then
    # A forced one-round control stays informative if eight never exhausts.
    # All client policies use the same worker counts and CPU placement.
    awk -v mode="$MODE" 'BEGIN {OFS="\t"} {name=$1; print;
        if(mode=="client") {$1=name "-client8"; print}; $1=name "-client1"; print}' \
        "$OUT/cohorts.tsv" > "$OUT/cohorts-selected.tsv"
    mv "$OUT/cohorts-selected.tsv" "$OUT/cohorts.tsv"
fi
forms=(base sleep short spin poll1 poll16)
if [[ $network_compute == 1 ]]; then
    forms=(base sleep poll1)
    if [[ $EXPERIMENT != stackful-paced && $EXPERIMENT != owner-paced && $EXPERIMENT != dispatch-paced && $EXPERIMENT != wake-paced && $EXPERIMENT != service-paced && $EXPERIMENT != coroutine-paced ]]; then
        awk '$1=="shared4" || $1=="split2"' "$OUT/cohorts.tsv" > "$OUT/cohorts-selected.tsv"
        mv "$OUT/cohorts-selected.tsv" "$OUT/cohorts.tsv"
    fi
fi
if [[ $EXPERIMENT == fairness || $EXPERIMENT == sustain ]]; then forms=(base); fi
if [[ $EXPERIMENT == inline ]]; then forms=(base local); fi
if [[ $EXPERIMENT == checkpoint || $EXPERIMENT == paced ]]; then forms=(base cq1024 cq16384 cq65536); fi
if [[ $EXPERIMENT == chunks ]]; then forms=(base cq16384 ch1024 ch16384 ch65536); fi
if [[ $EXPERIMENT == canonical ]]; then forms=(base old1024 old16384 ch1024 ch16384); fi
if [[ $EXPERIMENT == footprint ]]; then forms=(base lanes); fi
if [[ $EXPERIMENT == stackful ]]; then forms=(base); fi
if [[ $EXPERIMENT == stackful-paced ]]; then forms=(base ch16384); fi
if [[ $EXPERIMENT == nodelay ]]; then forms=(base nodelay); fi
if [[ $EXPERIMENT == owner ]]; then forms=(base pinned rings owner); fi
if [[ $EXPERIMENT == owner-paced ]]; then forms=(base ch16384 owner chowner16384); fi
if [[ $EXPERIMENT == memory ]]; then forms=(base lanes compact small); fi
if [[ $storage_experiment == 1 ]]; then forms=(base small); fi
if [[ $EXPERIMENT == allocation ]]; then forms=(base small callee callee-small); fi
if [[ $EXPERIMENT == allocator ]]; then forms=(base small callee-small); fi
if [[ $EXPERIMENT == dispatch ]]; then forms=(base rings owner balanced); fi
if [[ $EXPERIMENT == dispatch-paced ]]; then forms=(base ch16384 chowner16384 chbalanced16384); fi
if [[ $EXPERIMENT == wake ]]; then forms=(base rings balanced quiet); fi
if [[ $EXPERIMENT == wake-paced ]]; then forms=(base ch16384 chbalanced16384 chquiet16384); fi
if [[ $EXPERIMENT == service ]]; then forms=(base balanced service1 service16 servicepoll16); fi
if [[ $EXPERIMENT == service-paced ]]; then forms=(base chbalanced16384 chservice1 chservice16 chservicepoll16); fi
if [[ $EXPERIMENT == coroutine-paced ]]; then forms=(base ch16384 chbalanced16384); fi
if [[ $MODE == combine ]]; then forms=(callee-small balanced balanced-small quiet-small); fi
if [[ $NATIVE_BASELINES == 1 ]]; then forms=(callee-small balanced-small); fi
form_flags() {
    local_inline=0
    init_used=0
    tcp_nodelay=0
    ready_shards=0
    ready_pinned=0
    owner_rings=0
    compact_stacks=0
    io_dispatch=0
    local_wake=0
    io_quantum=0
    io_reset_turn=1
    if [[ $EXPERIMENT == owner || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced || $EXPERIMENT == dispatch || $EXPERIMENT == wake || $EXPERIMENT == service || $EXPERIMENT == memory || $storage_experiment == 1 ]]; then tcp_nodelay=1; fi
    case $1 in
        base|callee) spin=256; yields=16; progress=0 ;;
        pinned) spin=256; yields=16; progress=0; ready_shards=2; ready_pinned=1 ;;
        rings) spin=256; yields=16; progress=0; owner_rings=1 ;;
        owner|chowner16384) spin=256; yields=16; progress=0; ready_shards=2; ready_pinned=1; owner_rings=1 ;;
        balanced|balanced-small|chbalanced16384) spin=256; yields=16; progress=0; ready_shards=2; ready_pinned=1; owner_rings=1; io_dispatch=1 ;;
        servicepoll16|chservicepoll16) spin=256; yields=16; progress=0; ready_shards=2; ready_pinned=1; owner_rings=1; io_dispatch=1; io_quantum=16; io_reset_turn=0 ;;
        service1|service16|chservice1|chservice16) spin=256; yields=16; progress=0; ready_shards=2; ready_pinned=1; owner_rings=1; io_dispatch=1; io_quantum=${1##*service} ;;
        quiet|quiet-small|chquiet16384) spin=256; yields=16; progress=0; ready_shards=2; ready_pinned=1; owner_rings=1; io_dispatch=1; local_wake=1 ;;
        nodelay) spin=256; yields=16; progress=0; tcp_nodelay=1 ;;
        lanes) spin=256; yields=16; progress=0; init_used=1 ;;
        compact) spin=256; yields=16; progress=0; compact_stacks=1 ;;
        small|callee-small) spin=256; yields=16; progress=0; compact_stacks=1; init_used=1 ;;
        cq1024|cq16384|cq65536|ch1024|ch16384|ch65536|old1024|old16384) spin=256; yields=16; progress=0 ;;
        local) spin=256; yields=16; progress=0; local_inline=1 ;;
        sleep) spin=0; yields=0; progress=0 ;;
        short) spin=16; yields=0; progress=0 ;;
        spin) spin=256; yields=0; progress=0 ;;
        poll1) spin=16; yields=0; progress=1 ;;
        poll16) spin=256; yields=0; progress=16 ;;
        *) return 2 ;;
    esac
    if [[ $1 == *-small ]]; then compact_stacks=1; init_used=1; fi
}
link_form() {
    local module=$1 output=$2 policy=$3 observed=$4
    local observer=() spin yields progress local_inline init_used tcp_nodelay ready_shards ready_pinned owner_rings compact_stacks io_dispatch local_wake io_quantum io_reset_turn
    form_flags "$policy"
    if [[ $observed == 1 ]]; then observer=("$BACKEND/sched/grant_observer.c"); fi
    "$CLANG" -std=c11 -O2 -pthread -I "$BACKEND" -I "$BACKEND/completion" \
        -I "$BACKEND/sched" "-DWF_SCHED_IDLE_SPIN_ROUNDS=$spin" \
        "-DWF_SCHED_IDLE_YIELD_ROUNDS=$yields" "-DWF_SCHED_IDLE_PROGRESS_INTERVAL=$progress" \
        "-DWF_SCHED_OBSERVE=$observed" "-DWF_COMPLETION_LOCAL_INLINE=$local_inline" \
        "-DWF_SCHED_INIT_USED_LANES=$init_used" "-DWF_TCP_NODELAY=$tcp_nodelay" \
        "-DWF_SCHED_READY_SHARDS=$ready_shards" "-DWF_SCHED_READY_PINNED=$ready_pinned" \
        "-DWF_IO_OWNER_RINGS=$owner_rings" "-DWF_SCHED_LOCAL_WAKE=$local_wake" "-DWF_SCHED_IO_QUANTUM=$io_quantum" "-DWF_SCHED_IO_RESET_TURN=$io_reset_turn" \
        "-DWF_SCHED_COMPACT_STACKS=$compact_stacks" "-DWF_SCHED_IO_ROUND_ROBIN=$io_dispatch" \
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
elif [[ $EXPERIMENT == inline || $EXPERIMENT == footprint || $EXPERIMENT == stackful || $EXPERIMENT == nodelay || $allocation_experiment == 1 ]]; then
    programs=(echo)
else
    "$WFC" --par --emit-llvm -o "$OUT/compute.ll" "$ROOT/tests/programs/par_layout.wf"
    "$WFC" --par --emit-llvm -o "$OUT/mixed.ll" "$HERE/programs/windows_runtime_mixed.wf"
fi
if [[ $EXPERIMENT == checkpoint || $EXPERIMENT == chunks || $EXPERIMENT == canonical ]]; then
    programs=(echo compute mixed)
    "$WFC" --par --emit-llvm -o "$OUT/compute.ll" "$ROOT/tests/programs/par_layout.wf"
    "$WFC" --par --emit-llvm -o "$OUT/mixed.ll" "$HERE/programs/windows_runtime_mixed.wf"
fi
"$WFC" --par --emit-llvm -o "$OUT/echo.ll" "$echo_source"
if [[ $MODE == combine ]]; then
    mkdir -p "$OUT/codegen"
    cp "$echo_source" "$OUT/codegen/echo-callee.wf"
    cp "$OUT/echo.ll" "$OUT/codegen/echo-callee.ll"
fi
if [[ $allocation_experiment == 1 && $MODE != combine ]]; then
    # Isolate where the sequential source owns its receive buffer. Compile
    # the retained caller-owned source with this compiler and this runtime.
    allocation_baseline=2de6c00039243aee98554eabba5143f011991461
    mkdir -p "$OUT/codegen"
    git -C "$ROOT" show "$allocation_baseline:research/experiments/io-completion-bench/programs/tcp_echo_server.wf" > "$OUT/codegen/echo-caller.wf"
    cp "$echo_source" "$OUT/codegen/echo-callee.wf"
    "$WFC" --par --emit-llvm -o "$OUT/echo-caller.ll" "$OUT/codegen/echo-caller.wf"
    for placement in caller callee; do
        module="$OUT/echo.ll"
        if [[ $placement == caller ]]; then module="$OUT/echo-caller.ll"; fi
        grep -q 'call void @wf__par_publish_staged' "$module"
        cp "$module" "$OUT/codegen/echo-$placement.ll"
        "$CLANG" -O2 -S -emit-llvm -x ir "$module" -Wno-override-module \
            -o "$OUT/codegen/echo-$placement-opt.ll"
    done
    echo "allocation_baseline=$allocation_baseline base/small=caller callee/callee-small=accepted-handler" >> "$OUT/host.txt"
fi
for policy in "${forms[@]}"; do
    for program in "${programs[@]}"; do
        module="$OUT/$program.ll"
        if [[ $allocation_experiment == 1 && $program == echo && ( $policy == base || $policy == small ) ]]; then module="$OUT/echo-caller.ll"; fi
        if [[ $policy == cq* || $policy == ch* || $policy == old* ]]; then
            case $program in
                echo) source="$echo_source" ;;
                compute) source="$ROOT/tests/programs/par_layout.wf" ;;
                mixed) source="$HERE/programs/windows_runtime_mixed.wf" ;;
            esac
            module="$OUT/$program-$policy.ll"
            checkpoint_option=--sched-quantum
            if [[ $policy == ch* ]]; then checkpoint_option=--sched-chunks; fi
            checkpoint_interval=${policy:2}
            if [[ $policy == chowner16384 || $policy == chbalanced16384 || $policy == chquiet16384 || $policy == chservice* ]]; then checkpoint_interval=16384; fi
            checkpoint_compiler=$WFC
            if [[ $policy == old* ]]; then
                checkpoint_compiler=$OLD_WFC
                checkpoint_option=--sched-chunks
                checkpoint_interval=${policy#old}
            fi
            "$checkpoint_compiler" --par "$checkpoint_option" "$checkpoint_interval" --emit-llvm -o "$module" "$source"
            if [[ ( $policy == ch* || $policy == old* ) && $program == echo ]]; then
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
        if [[ $EXPERIMENT == canonical && $program == compute && ( $policy == base || $policy == old16384 || $policy == ch16384 ) ]]; then
            "$CLANG" -O2 -S -x ir "$module" -Wno-override-module \
                -o "$OUT/codegen/compute-$policy.s"
        fi
    done
    module="$OUT/echo.ll"
    if [[ $allocation_experiment == 1 && ( $policy == base || $policy == small ) ]]; then module="$OUT/echo-caller.ll"; fi
    if [[ $policy == cq* || $policy == ch* || $policy == old* ]]; then module="$OUT/echo-$policy.ll"; fi
    link_form "$module" "$OUT/bin/echo-$policy-observed" "$policy" 1
    if [[ ( $EXPERIMENT == chunks && ( $policy == cq16384 || $policy == ch16384 ) ) || ( $EXPERIMENT == canonical && ( $policy == old16384 || $policy == ch16384 ) ) ]]; then
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
if [[ $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then
    for policy in "${forms[@]}"; do
        reference_module="$OUT/echo-ch16384.ll"
        if [[ $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then reference_module="$OUT/echo-chbalanced16384.ll"; fi
        if [[ $policy == ch* ]]; then cmp "$reference_module" "$OUT/echo-$policy.ll"; fi
    done
fi
if [[ $EXPERIMENT == canonical ]]; then
    # The uninstrumented compiler result is unchanged, and the candidate must
    # actually differ from the former chunk topology.
    "$OLD_WFC" --par --emit-llvm -o "$OUT/compute-before.ll" "$ROOT/tests/programs/par_layout.wf"
    cmp "$OUT/compute-before.ll" "$OUT/compute.ll"
    if cmp -s "$OUT/compute-old16384.ll" "$OUT/compute-ch16384.ll"; then
        echo 'scheduler-bench: canonical lowering did not change the module' >&2
        exit 1
    fi
fi
if [[ $network_compute == 1 ]]; then
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -DWF_BENCH_COMPUTE \
        "$HERE/epoll_echo.c" -o "$OUT/bin/epoll_compute"
fi
if [[ $EXPERIMENT == fairness || $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == canonical || $EXPERIMENT == stackful-paced || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -DWF_BENCH_COMPUTE -DWF_BENCH_QUANTUM \
        "$HERE/epoll_echo.c" -o "$OUT/bin/epoll_quantum"
fi
for tool in netload uring_echo epoll_echo runner gen; do
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread "$HERE/$tool.c" -o "$OUT/bin/$tool"
done
if [[ $NATIVE_BASELINES == 1 ]]; then
    for bytes in 8192 65536; do
        output="$OUT/bin/uring_echo"; if [[ $bytes == 65536 ]]; then output="$output-64k"; fi
        for observed in 0 1; do
            observe_flags=()
            if [[ $observed == 1 ]]; then observe_flags=(-DWF_BENCH_URING_OBSERVE -DWF_BENCH_TCP_VERIFY); fi
            binary="$output"; if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
            "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -pthread \
                "-DWF_BENCH_URING_BUFFER_BYTES=$bytes" "${observe_flags[@]}" "$HERE/uring_echo.c" -o "$binary"
        done
    done
fi
if [[ $EXPERIMENT == allocator ]]; then
    # Read back the requested value using this binary's actual ELF loader.
    loader=$(LC_ALL=C readelf -l "$OUT/bin/epoll_echo" | sed -n 's/.*Requesting program interpreter: \(.*\)]/\1/p')
    [[ -x $loader ]]
    for top_pad in 131072 0; do
        env "GLIBC_TUNABLES=glibc.malloc.top_pad=$top_pad" "$loader" --list-tunables \
            > "$OUT/allocator-top$top_pad.txt"
        actual=$(awk '$1=="glibc.malloc.top_pad:" {print $2}' "$OUT/allocator-top$top_pad.txt")
        [[ -n $actual && $((actual)) -eq $top_pad ]]
    done
fi
if [[ $client_experiment == 1 ]]; then
    mkdir -p "$OUT/codegen"
    git -C "$ROOT" show d72d0d253838c1e0134cb7f3f97ea681af105b7f:research/experiments/io-completion-bench/netload.c \
        > "$OUT/codegen/netload-before.c"
    for revision in before after; do
        client_source="$OUT/codegen/netload-before.c"
        if [[ $revision == after ]]; then client_source="$HERE/netload.c"; fi
        "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -I "$HERE" -S -emit-llvm \
            "$client_source" -o "$OUT/codegen/netload-$revision.ll"
        sed '/^; ModuleID =/d; /^source_filename =/d' "$OUT/codegen/netload-$revision.ll" \
            > "$OUT/codegen/netload-$revision.normalized.ll"
    done
    cmp "$OUT/codegen/netload-before.normalized.ll" "$OUT/codegen/netload-after.normalized.ll"
    for budget in 1 8; do
        "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -pthread \
            "-DWF_NETLOAD_SERVICE_ROUNDS=$budget" "$HERE/netload.c" -o "$OUT/bin/netload-service$budget"
    done
    make -C "$HERE" client-service-check CLANG="$CLANG" WHITEFOOT_SCRATCH_ROOT="$OUT/client-check" \
        > "$OUT/client-service-check.log" 2>&1 || { cat "$OUT/client-service-check.log"; exit 1; }
fi
if [[ $EXPERIMENT == nodelay ]]; then
    for engine in epoll uring; do
        for nagle in 0 1; do
            option_flags=()
            output="$OUT/bin/${engine}_echo"
            if [[ $nagle == 1 ]]; then option_flags=(-DWF_BENCH_NAGLE); output="$output-nagle"; fi
            if [[ $nagle == 1 ]]; then
                "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread "${option_flags[@]}" \
                    "$HERE/${engine}_echo.c" -o "$output"
            fi
            # Read back the actual accepted socket option outside timing.
            "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread "${option_flags[@]}" \
                -DWF_BENCH_TCP_VERIFY "$HERE/${engine}_echo.c" -o "$output-verified"
        done
    done
fi
if [[ $EXPERIMENT == stackful || $EXPERIMENT == stackful-paced || $storage_experiment == 1 || $coroutine_experiment == 1 ]]; then
    # The reference engine must remain the measured prior implementation.
    # Check all three manual forms on this toolchain, not only the M1 probe.
    mkdir -p "$OUT/codegen"
    git -C "$ROOT" show 2f9468788790ca466a53e88d3b4f14634fe9c4ad:research/experiments/io-completion-bench/epoll_echo.c \
        > "$OUT/codegen/epoll-before.c"
    for reference_mode in echo compute quantum; do
        reference_flags=()
        if [[ $reference_mode != echo ]]; then reference_flags=(-DWF_BENCH_COMPUTE); fi
        if [[ $reference_mode == quantum ]]; then reference_flags+=(-DWF_BENCH_QUANTUM); fi
        for revision in before after; do
            reference_source="$OUT/codegen/epoll-before.c"
            if [[ $revision == after ]]; then reference_source="$HERE/epoll_echo.c"; fi
            "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -S -emit-llvm -I "$HERE" \
                "${reference_flags[@]}" "$reference_source" -o "$OUT/codegen/$reference_mode-$revision.ll"
            sed '/^; ModuleID =/d; /^source_filename =/d' "$OUT/codegen/$reference_mode-$revision.ll" \
                > "$OUT/codegen/$reference_mode-$revision.normalized.ll"
        done
        cmp "$OUT/codegen/$reference_mode-before.normalized.ll" "$OUT/codegen/$reference_mode-after.normalized.ll"
    done
    stackful_flags=()
    if [[ $network_compute == 1 ]]; then stackful_flags=(-DWF_BENCH_COMPUTE); fi
    "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -DWF_BENCH_STACKFUL \
        "${stackful_flags[@]}" "$HERE/epoll_echo.c" -o "$OUT/bin/epoll_stackful"
    if [[ $network_compute == 1 ]]; then
        "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -pthread -DWF_BENCH_STACKFUL \
            -DWF_BENCH_COMPUTE -DWF_BENCH_QUANTUM "$HERE/epoll_echo.c" -o "$OUT/bin/epoll_stackful_quantum"
    fi
fi
if [[ $storage_experiment == 1 ]]; then
    storage_forms=(epoll epoll-arena epoll-malloc epoll-calloc fiber fiber-arena fiber-malloc fiber-calloc)
    if [[ $EXPERIMENT == allocator ]]; then storage_forms=(epoll epoll-calloc epoll-calloc-main fiber-calloc fiber-calloc-main); fi
    for form in "${storage_forms[@]}"; do
        storage=0
        storage_flags=()
        case $form in
            *-arena) storage=1 ;;
            *-malloc) storage=2 ;;
            *-calloc|*-calloc-main) storage=3 ;;
        esac
        if [[ $form == fiber* ]]; then storage_flags=(-DWF_BENCH_STACKFUL); fi
        if [[ $form == *-main ]]; then storage_flags+=(-DWF_BENCH_MAIN_WORKER); fi
        for observed in 0 1; do
            output="$OUT/bin/storage-$form"
            observe_flags=()
            if [[ $observed == 1 ]]; then output="$output-observed"; observe_flags=(-DWF_BENCH_STORAGE_OBSERVE); fi
            "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -pthread \
                "-DWF_BENCH_RECEIVE_STORAGE=$storage" "${storage_flags[@]}" "${observe_flags[@]}" \
                "$HERE/epoll_echo.c" -o "$output"
        done
        "$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -pthread -S -emit-llvm \
            "-DWF_BENCH_RECEIVE_STORAGE=$storage" "${storage_flags[@]}" "$HERE/epoll_echo.c" \
            -o "$OUT/codegen/storage-$form.ll"
    done
    "$CLANG" -O2 -S -emit-llvm -x ir "$OUT/echo.ll" -Wno-override-module \
        -o "$OUT/codegen/storage-wf.ll"
fi
if [[ $coroutine_experiment == 1 ]]; then
    # Compile the same engine as C++ for manual and stackful controls too.
    # Every normal form has a separate untimed allocation/continuation observer.
    cpp_storage=(0 3)
    cpp_protocol=()
    if [[ $network_compute == 1 ]]; then cpp_storage=(0); cpp_protocol=(-DWF_BENCH_COMPUTE -DWF_BENCH_QUANTUM); fi
    for representation in manual stackful heap elide; do
        cpp_flags=()
        if [[ $representation != manual ]]; then cpp_flags+=(-DWF_BENCH_STACKFUL); fi
        if [[ $representation == heap || $representation == elide ]]; then cpp_flags+=(-DWF_BENCH_COROUTINE); fi
        if [[ $representation == elide ]]; then cpp_flags+=(-DWF_BENCH_CORO_ELIDE); fi
        for storage in "${cpp_storage[@]}"; do
            form="cpp-$representation"
            if [[ $storage == 3 ]]; then form="$form-calloc"; fi
            for observed in 0 1; do
                output="$OUT/bin/$form"
                observe_flags=()
                if [[ $observed == 1 ]]; then output="$output-observed"; observe_flags=(-DWF_BENCH_OBSERVE -DWF_BENCH_STORAGE_OBSERVE); fi
                "$CORO_CXX" -std=c++20 -O2 -Wall -Wextra -Werror -Wpedantic -fno-exceptions -pthread \
                    "${cpp_flags[@]}" "${cpp_protocol[@]}" "${observe_flags[@]}" "-DWF_BENCH_RECEIVE_STORAGE=$storage" \
                    -x c++ "$HERE/epoll_echo.c" -o "$output"
            done
            "$CORO_CXX" -std=c++20 -O2 -Wall -Wextra -Werror -Wpedantic -fno-exceptions -pthread \
                "${cpp_flags[@]}" "${cpp_protocol[@]}" "-DWF_BENCH_RECEIVE_STORAGE=$storage" \
                -x c++ -S -emit-llvm "$HERE/epoll_echo.c" -o "$OUT/codegen/$form.ll"
        done
    done
fi
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
allocator_setting() {
    local top_pad=131072
    if [[ $cohort == *-top0-* ]]; then top_pad=0; fi
    printf 'GLIBC_TUNABLES=glibc.malloc.top_pad=%s' "$top_pad"
}
sample=0
network_case() {
    local form=$1 connections=$2 trips=$3 bytes=$4 pass=$5 observed=$6
    local binary environment=() arguments=() launcher=() directory port server_stderr
    local client_binary="$OUT/bin/netload"
    if [[ $cohort == *-client8 ]]; then client_binary="$OUT/bin/netload-service8"; fi
    if [[ $cohort == *-client1 ]]; then client_binary="$OUT/bin/netload-service1"; fi
    sample=$((sample + 1))
    directory="$OUT/samples/$sample-$cohort-$form-k$connections-b$bytes-r$compute_rounds-a$admitted-d$duration_ms-l${light_per_second:-0}"
    if [[ $observed == 1 ]]; then directory="$OUT/observed/$cohort-$form-k$connections-a$admitted"; fi
    mkdir -p "$directory"
    port=$(free_port)
    case $form in
        cpp-*)
            binary="$OUT/bin/$form"
            if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
            arguments=(--threads "$server_workers") ;;
        uring-64k)
            binary="$OUT/bin/uring_echo-64k"
            if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
            arguments=(--threads "$server_workers") ;;
        uring|epoll|uring-nagle|epoll-nagle)
            binary="$OUT/bin/${form%-nagle}_echo"
            if [[ $form == *-nagle ]]; then binary="$binary-nagle"; fi
            if [[ $EXPERIMENT == nodelay && $observed == 1 ]]; then binary="$binary-verified"; fi
            if [[ $NATIVE_BASELINES == 1 && $form == uring && $observed == 1 ]]; then binary="$binary-observed"; fi
            if [[ $network_compute == 1 ]]; then binary="$OUT/bin/epoll_compute"; fi
            arguments=(--threads "$server_workers") ;;
        q1024|q16384|q65536)
            binary="$OUT/bin/epoll_quantum"
            arguments=(--threads "$server_workers" --quantum "${form#q}") ;;
        epoll-arena|epoll-malloc|epoll-calloc|epoll-calloc-main|fiber-arena|fiber-malloc|fiber-calloc|fiber-calloc-main)
            binary="$OUT/bin/storage-$form"
            arguments=(--threads "$server_workers") ;;
        fiber)
            binary="$OUT/bin/epoll_stackful"
            arguments=(--threads "$server_workers") ;;
        f16384)
            binary="$OUT/bin/epoll_stackful_quantum"
            arguments=(--threads "$server_workers" --quantum 16384) ;;
        servicepoll16|service16|service1|chservicepoll16|chservice16|chservice1|base|callee|callee-small|nodelay|pinned|rings|owner|chowner16384|balanced|balanced-small|chbalanced16384|quiet|quiet-small|chquiet16384|local|lanes|compact|small|sleep|short|spin|poll1|poll16|cq1024|cq16384|cq65536|ch1024|ch16384|ch65536|old1024|old16384)
            binary="$OUT/bin/echo-$form"
            if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
            environment=("WF_WORKERS=$server_workers" WF_STACKS=1100 "WF_SCHED_REPORT=$observed") ;;
        *) return 2 ;;
    esac
    if [[ $storage_experiment == 1 && ( $form == epoll* || $form == fiber* ) ]]; then
        binary="$OUT/bin/storage-$form"
        if [[ $observed == 1 ]]; then binary="$binary-observed"; fi
    fi
    if [[ $page_experiment == 1 ]]; then
        local disabled=0
        if [[ $cohort == *-no-thp ]]; then disabled=1; fi
        environment+=("WF_BENCH_THP_DISABLE=$disabled")
        launcher=("$OUT/bin/stream_check" launch)
    fi
    if [[ $EXPERIMENT == allocator ]]; then environment+=("$(allocator_setting)"); fi
    server_stderr="$directory/server.err"
    if [[ $profile_active == 1 ]]; then
        # Keep recorder diagnostics separate from the server's strict stderr
        # check. The wrapper execs the same normal binary without recompiling.
        launcher=("$PROFILE_PERF" record -e cpu-clock -F 999 -o "$directory/perf.data" --
            "$OUT/bin/profile-server" "$server_stderr")
        server_stderr="$directory/perf-record.err"
    fi
    echo "sample=$sample pass=$pass cohort=$cohort form=$form connections=$connections compute=$compute_rounds observed=$observed admitted=$admitted"
    setsid timeout --signal=TERM --kill-after=5s 120s \
        /usr/bin/time -f '%U\t%S\t%M\t%w\t%c' -o "$directory/resources.tsv" taskset -c "$server_cpus" \
        env "${environment[@]}" "${launcher[@]}" "$binary" "$port" "$connections" "${arguments[@]}" \
        > "$directory/server.out" 2> "$server_stderr" &
    server_pid=$!
    while ! port_present "$port" 1; do
        if ! kill -0 "$server_pid" 2>/dev/null; then
            cat "$directory/server.err" "$server_stderr" >&2
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
        "$client_binary" "$port" "$connections" "$trips" "$bytes" --threads "$client_workers" "${client_arguments[@]}" \
        > "$directory/client.tsv" 2> "$directory/client.err"
    if ! wait "$server_pid"; then cat "$directory/server.err" "$server_stderr" >&2; return 1; fi
    server_pid=''
    [[ ! -s $directory/server.out && ! -s $directory/client.err ]]
    if [[ $observed == 0 ]]; then [[ ! -s $directory/server.err ]]; fi
    if [[ $cohort == *-client8 ]]; then [[ $(field "$directory/client.tsv" client_service_rounds) == 8 ]]; fi
    if [[ $cohort == *-client1 ]]; then
        [[ $(field "$directory/client.tsv" client_service_rounds) == 1 ]]
        [[ $(field "$directory/client.tsv" client_service_yields) -gt 0 ]]
    fi
    if [[ $profile_active == 1 ]]; then
        "$PROFILE_PERF" report --stdio --header --show-nr-samples --no-children \
            --sort comm,dso,symbol -i "$directory/perf.data" > "$directory/perf-report.txt" 2> "$directory/perf-report.err"
        "$PROFILE_PERF" script -i "$directory/perf.data" \
            -F comm,pid,tid,time,event,ip,sym,dso,period > "$directory/perf-samples.txt" 2> "$directory/perf-script.err"
        [[ -s $directory/perf-samples.txt ]]
    fi
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
if [[ $EXPERIMENT == canonical ]]; then references=(q1024 q16384); fi
if [[ $EXPERIMENT == stackful ]]; then references=(uring epoll fiber); fi
if [[ $storage_experiment == 1 ]]; then references=(uring epoll epoll-arena epoll-malloc epoll-calloc fiber fiber-arena fiber-malloc fiber-calloc); fi
if [[ $EXPERIMENT == pages ]]; then references=(epoll epoll-arena epoll-malloc epoll-calloc fiber-calloc); fi
if [[ $EXPERIMENT == allocation ]]; then references=(uring epoll epoll-calloc fiber-calloc); fi
if [[ $EXPERIMENT == allocator ]]; then references=(epoll epoll-calloc epoll-calloc-main fiber-calloc fiber-calloc-main); fi
if [[ $EXPERIMENT == stackful-paced ]]; then references=(epoll fiber q16384 f16384); fi
if [[ $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then references=(epoll q16384); fi
if [[ $EXPERIMENT == nodelay ]]; then references=(uring uring-nagle epoll epoll-nagle); fi
if [[ $EXPERIMENT == coroutine ]]; then
    references=(uring epoll epoll-calloc cpp-manual cpp-manual-calloc cpp-stackful cpp-stackful-calloc cpp-heap cpp-heap-calloc cpp-elide cpp-elide-calloc)
fi
if [[ $EXPERIMENT == coroutine-paced ]]; then references=(q16384 cpp-manual cpp-stackful cpp-heap cpp-elide); fi
if [[ $MODE == combine ]]; then references=(epoll epoll-calloc-main fiber-calloc-main cpp-elide cpp-elide-calloc); fi
if [[ $NATIVE_BASELINES == 1 ]]; then references=(uring uring-64k "${references[@]}"); fi
while IFS=$'\t' read -r cohort server_workers client_workers server_cpus client_cpus; do
    allocator_environment=()
    if [[ $EXPERIMENT == allocator ]]; then allocator_environment=("$(allocator_setting)"); fi
    if [[ $client_experiment == 1 && ( $cohort == split1 || $cohort == split2 || $cohort == separate2 ) ]]; then
        # Force the userspace continuation path even when every kernel poll
        # could otherwise supply another edge. Verify computation and pacing.
        cohort="$cohort-client1"
        network_case q16384 4 20 64 -1 0
        admitted=1 duration_ms=100 light_per_second=100
        network_case q16384 4 100000 64 -1 0
        admitted=0 duration_ms=0 light_per_second=0
        cohort=${cohort%-client1}
    fi
    if [[ $allocation_experiment == 1 ]]; then
        disabled=0
        if [[ $cohort == *-no-thp ]]; then disabled=1; fi
        for form in "${forms[@]}"; do
            # Exercise short sends, fragmented streams and EOF before timing
            # either ownership placement. Observers do not enter timed runs.
            log="$OUT/observed/$cohort-$form-stream.log"
            if ! taskset -c "$client_cpus" env "WF_WORKERS=$server_workers" WF_STACKS=1100 WF_SCHED_REPORT=1 \
                "WF_BENCH_THP_DISABLE=$disabled" "WF_BENCH_SERVER_CPUS=$server_cpus" "${allocator_environment[@]}" \
                "$OUT/bin/stream_check" "$OUT/bin/echo-$form-observed" echo "$server_workers" > "$log" 2>&1; then
                cat "$log" >&2
                exit 1
            fi
            cat "$log" >> "$OUT/allocation-check.log"
        done
    fi
    for admitted in "${admissions[@]}"; do
        for form in "${references[@]}" "${forms[@]}"; do
            preflight_observed=0
            if [[ $form == cpp-* ]]; then preflight_observed=1; fi
            if [[ $NATIVE_BASELINES == 1 && $form == uring* ]]; then preflight_observed=1; fi
            if [[ $EXPERIMENT == nodelay || ( $storage_experiment == 1 && ( $form == epoll* || $form == fiber* ) ) ]]; then preflight_observed=1; fi
            network_case "$form" 4 20 64 -1 "$preflight_observed"
            if [[ $NATIVE_BASELINES == 1 && $form == uring* ]]; then
                expected_bytes=8192; if [[ $form == uring-64k ]]; then expected_bytes=65536; fi
                awk -v bytes="$expected_bytes" -v workers="$server_workers" '/^uring:/ {
                    for(i=2;i<=NF;i++) {split($i,a,"=");v[a[1]]=a[2]+0}; seen++;
                    if(v["buffer_bytes"]!=bytes || v["provided_bytes"]!=2097152 || v["buffers"]*bytes!=v["provided_bytes"]) bad=1;
                    received+=v["receive_bytes"]; sent+=v["send_bytes"]
                } END {exit !(seen==workers && !bad && received==5120 && sent==received)}' \
                    "$OUT/observed/$cohort-$form-k4-a0/server.err"
            fi
            if [[ $form == cpp-* ]]; then
                expected_storage=0
                if [[ $form == *-calloc ]]; then expected_storage=3; fi
                awk -v expected="$expected_storage" '/^storage:/ {for(i=2;i<=NF;i++) {split($i,a,"=");v[a[1]]=a[2]+0}; seen=1}
                    END {exit !(seen && v["policy"]==expected && v["accepted"]==4 && v["closed"]==4)}' \
                    "$OUT/observed/$cohort-$form-k4-a0/server.err"
                if [[ $form == cpp-heap* || $form == cpp-elide* ]]; then
                    awk -v form="$form" '/^coroutine:/ {for(i=2;i<=NF;i++) {split($i,a,"=");v[a[1]]=a[2]+0}; seen=1}
                        END {exit !(seen && v["allocations"]>0 && v["allocations"]==v["frees"] && (form!~/elide/ || v["allocations"]==4))}' \
                        "$OUT/observed/$cohort-$form-k4-a0/server.err"
                fi
            fi
            if [[ $storage_experiment == 1 && ( $form == epoll* || $form == fiber* ) ]]; then
                expected_storage=0
                expected_main=0
                if [[ $form == *-main ]]; then expected_main=1; fi
                case $form in
                    *-arena) expected_storage=1 ;;
                    *-malloc) expected_storage=2 ;;
                    *-calloc|*-calloc-main) expected_storage=3 ;;
                esac
                awk -v expected="$expected_storage" -v main="$expected_main" '/^storage:/ {for(i=2;i<=NF;i++) {split($i,a,"=");v[a[1]]=a[2]+0}; seen=1}
                    END {exit !(seen && v["policy"]==expected && ("main_worker" in v) && v["main_worker"]==main && v["transfer_bytes"]==65536 && v["accepted"]==4 && v["closed"]==4)}' \
                    "$OUT/observed/$cohort-$form-k4-a0/server.err"
            fi
        done
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
        if [[ $EXPERIMENT == memory || $storage_experiment == 1 ]]; then
            compact=0
            used=0
            if [[ $form == compact || $form == small || $form == *-small ]]; then compact=1; fi
            if [[ $form == lanes || $form == small || $form == *-small ]]; then used=1; fi
            awk -v compact="$compact" -v used="$used" '
                 /^sched:|^ring:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(value["tcp_nodelay"]==1 && ("compact_stacks" in value) &&
                     ("init_used_lanes" in value) && value["compact_stacks"]==compact && value["init_used_lanes"]==used) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
        if [[ $EXPERIMENT == nodelay ]]; then
            expected=0
            if [[ $form == nodelay ]]; then expected=1; fi
            awk -v expected="$expected" '/^ring:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(("tcp_nodelay" in value) && value["tcp_nodelay"]==expected) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
        if [[ $EXPERIMENT == owner || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch || $EXPERIMENT == wake || $EXPERIMENT == service || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced || $MODE == combine ]]; then
            local_wake=0
            if [[ $form == quiet* || $form == chquiet16384 ]]; then local_wake=1; fi
            pinned=0
            rings=0
            if [[ $form == pinned || $form == owner || $form == chowner16384 || $form == balanced* || $form == chbalanced16384 || $form == quiet* || $form == chquiet16384 || $form == service* || $form == chservice* ]]; then pinned=1; fi
            if [[ $form == rings || $form == owner || $form == chowner16384 || $form == balanced* || $form == chbalanced16384 || $form == quiet* || $form == chquiet16384 || $form == service* || $form == chservice* ]]; then rings=1; fi
            # Initial work stealing is opportunistic. A valid pinned run can
            # keep all connections on one thread, which is a load-balancing
            # defect to measure, not evidence that its bridge failed. The
            # separate four-thread probe requires four actual submitting rings.
            awk -v pinned="$pinned" -v rings="$rings" -v local_wake="$local_wake" '
                 /^sched:|^ring:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(value["tcp_nodelay"]==1 && ("ready_pinned" in value) &&
                     value["ready_pinned"]==pinned && ("local_wake" in value) && value["local_wake"]==local_wake &&
                     (!pinned || (value["ready_shards"]==2 && value["resumes"]>0 && value["resume_migrations"]==0)) &&
                     (!rings || value["owner_rings"] >= 1)) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
            if [[ $rings == 1 ]]; then
                awk -v cohort="$cohort" -v form="$form" -v peers="$connections" '
                     /^sched:|^ring:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                     END { printf "owner distribution cohort=%s form=%s peers=%d rings=%d steals=%d\n", cohort, form, peers, value["owner_rings"], value["steals"] }' \
                    "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
            fi
        fi
        if [[ $form == balanced* || $form == chbalanced16384 || $form == quiet* || $form == chquiet16384 || $form == service* || $form == chservice* ]]; then
            # This source has one producer and one staged task per connection.
            # Check actual starts, not steals: deliberate placement bypasses deques.
            awk -v workers="$server_workers" -v peers="$connections" '
                 /^sched:|^ring:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(value["io_dispatch"]==1 && value["dispatch_workers"]==workers &&
                     value["io_started"]==peers && value["io_workers"]==(peers<workers?peers:workers) &&
                     value["io_min"]==int(peers/workers) && value["io_max"]==int((peers+workers-1)/workers) &&
                     value["owner_rings"]>=workers) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
        if [[ $EXPERIMENT == service || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then
            expected_quantum=0
            if [[ $form == service* || $form == chservice* ]]; then expected_quantum=${form##*service}; fi
            expected_reset=1
            if [[ $form == *servicepoll16 ]]; then expected_quantum=16; expected_reset=0; fi
            awk -v quantum="$expected_quantum" -v reset="$expected_reset" '
                 /^sched:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(("io_quantum" in value) && value["io_quantum"]==quantum && ("io_reset_turn" in value) && value["io_reset_turn"]==reset &&
                     ("io_checkpoints" in value) && (quantum || value["io_checkpoints"]==0) &&
                     ((quantum!=1 && reset!=0) || value["io_checkpoints"]>0)) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
        if [[ ( $form == cq* || $form == ch* || $form == old* ) && $connections == 64 ]]; then
            awk '/^sched:/ { for(i=2;i<=NF;i++) { split($i,a,"="); value[a[1]]=a[2]+0 } }
                 END { exit !(value["checkpoints"] > 0 && value["checkpoint_switches"] > 0) }' \
                "$OUT/observed/$cohort-$form-k$connections-a$admitted/server.err"
        fi
      done
    done
done < "$OUT/cohorts.tsv"

if [[ $page_experiment == 1 ]]; then
    mkdir -p "$OUT/resident"
    printf 'repetition\tcohort\tform\tconnections\tbytes\tthp_disabled\trss_kib\tanonymous_kib\tanon_huge_kib\tprivate_dirty_kib\tswap_kib\n' > "$OUT/resident.tsv"
    # Same normal binaries as timing, with all peers held open after a checked
    # exchange. Include every storage/representation control for attribution;
    # the timed panel below keeps only the controls needed for the page test.
    resident_forms=(base small uring epoll epoll-arena epoll-malloc epoll-calloc fiber fiber-arena fiber-malloc fiber-calloc)
    if [[ $allocation_experiment == 1 ]]; then resident_forms=("${forms[@]}" "${references[@]}"); fi
    for repetition in 0 1 2; do
      while IFS=$'\t' read -r cohort server_workers client_workers server_cpus client_cpus; do
        disabled=0
        if [[ $cohort == *-no-thp ]]; then disabled=1; fi
        allocator_environment=()
        if [[ $EXPERIMENT == allocator ]]; then allocator_environment=("$(allocator_setting)"); fi
        for form in "${resident_forms[@]}"; do
          binary="$OUT/bin/storage-$form"
          if [[ $form == base || $form == small || $form == callee || $form == *-small || $form == balanced ]]; then binary="$OUT/bin/echo-$form"; fi
          if [[ $form == uring ]]; then binary="$OUT/bin/uring_echo"; fi
          if [[ $form == uring-64k ]]; then binary="$OUT/bin/uring_echo-64k"; fi
          if [[ $form == cpp-* ]]; then binary="$OUT/bin/$form"; fi
          for resident_case in '64 64' '1024 64' '64 65536'; do
            read -r connections bytes <<< "$resident_case"
            record="$OUT/resident/r$repetition-$cohort-$form-k$connections-b$bytes"
            if ! taskset -c "$client_cpus" env "WF_WORKERS=$server_workers" WF_STACKS=1100 WF_SCHED_REPORT=0 \
                "WF_BENCH_THP_DISABLE=$disabled" "WF_BENCH_SERVER_CPUS=$server_cpus" "${allocator_environment[@]}" \
                "$OUT/bin/stream_check" "$binary" resident "$server_workers" "$connections" "$bytes" "$record" \
                > "$record.log" 2> "$record.err"; then
                cat "$record.log" "$record.err" >&2
                exit 1
            fi
            [[ ! -s $record.err && -s $record.smaps && -s $record.status ]]
            awk -v disabled="$disabled" '$1=="THP_enabled:" {seen=1; enabled=$2}
                END {exit !(seen && enabled==1-disabled)}' "$record.status"
            printf '%s\t%s\t%s\t%s\t%s\t%s\t' "$repetition" "$cohort" "$form" "$connections" "$bytes" "$disabled" >> "$OUT/resident.tsv"
            awk '$1=="Rss:" {rss+=$2; seen++} $1=="Anonymous:" {anon+=$2}
                $1=="AnonHugePages:" {huge+=$2; huge_seen++} $1=="Private_Dirty:" {dirty+=$2}
                $1=="Swap:" {swap+=$2}
                END {if(!seen || !huge_seen || rss<=0) exit 1;
                    printf "%d\t%d\t%d\t%d\t%d\n", rss,anon,huge,dirty,swap}' "$record.smaps" >> "$OUT/resident.tsv"
            cat "$record.log"
          done
        done
      done < "$OUT/cohorts.tsv"
    done
fi

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
if [[ $EXPERIMENT == inline || $EXPERIMENT == footprint || $EXPERIMENT == stackful || $EXPERIMENT == nodelay || $EXPERIMENT == owner || $EXPERIMENT == dispatch || $EXPERIMENT == wake || $EXPERIMENT == service || $EXPERIMENT == memory || $storage_experiment == 1 ]]; then
    printf '1024 200 64 0\n64 500 65536 0\n' >> "$OUT/cases.tsv"
fi
if [[ $page_experiment == 1 && $MODE != combine ]]; then
    awk '$1>=64' "$OUT/cases.tsv" > "$OUT/cases-selected.tsv"
    mv "$OUT/cases-selected.tsv" "$OUT/cases.tsv"
fi
if [[ $EXPERIMENT == fairness ]]; then
    # Zero-compute control plus two sustained compute costs; retain both peer counts.
    awk '$4 != 16384' "$OUT/cases.tsv" > "$OUT/cases-selected.tsv"
    mv "$OUT/cases-selected.tsv" "$OUT/cases.tsv"
fi
if [[ $EXPERIMENT == sustain || $EXPERIMENT == checkpoint || $EXPERIMENT == paced || $EXPERIMENT == chunks || $EXPERIMENT == canonical || $EXPERIMENT == stackful-paced || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then
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
if [[ $EXPERIMENT == paced || $EXPERIMENT == canonical || $EXPERIMENT == stackful-paced || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then
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
if [[ $EXPERIMENT == canonical || $EXPERIMENT == stackful-paced || $EXPERIMENT == owner-paced || $EXPERIMENT == dispatch-paced || $EXPERIMENT == wake-paced || $EXPERIMENT == service-paced || $EXPERIMENT == coroutine-paced ]]; then
    awk '$4==0 || ($4==2097152 && $5>=100)' "$OUT/cases.tsv" > "$OUT/cases-selected.tsv"
    mv "$OUT/cases-selected.tsv" "$OUT/cases.tsv"
fi
if [[ $MODE == profile ]]; then
    # All protocol/observer qualification above stays enabled. Attribute the
    # measured WF gap against native C manual and compact C++ continuations.
    profile_active=1
    forward=(base chbalanced16384 q16384 cpp-elide)
    reverse=(cpp-elide q16384 chbalanced16384 base)
    cp "$OUT/bin/echo-base" "$OUT/bin/echo-chbalanced16384" \
        "$OUT/bin/epoll_quantum" "$OUT/bin/cpp-elide" "$OUT/codegen/"
    cat > "$OUT/bin/profile-server" <<'PROFILE_SERVER'
#!/usr/bin/env bash
set -euo pipefail
diagnostics=$1
shift
exec "$@" 2> "$diagnostics"
PROFILE_SERVER
    chmod +x "$OUT/bin/profile-server"
fi
if [[ $client_experiment == 1 ]]; then
    forward=(base ch16384 chbalanced16384 q16384 cpp-elide)
    reverse=(cpp-elide q16384 chbalanced16384 ch16384 base)
fi
for ((pass=-WARMUP; pass<ROUNDS; pass++)); do
    order=("${forward[@]}")
    if (( (pass + WARMUP) % 2 )); then order=("${reverse[@]}"); fi
    cp "$OUT/cohorts.tsv" "$OUT/cohorts-order.tsv"
    if [[ $page_experiment == 1 || $client_experiment == 1 ]] && (( (pass + WARMUP) % 2 )); then
        # Alternate which policy runs first as well as representation order.
        tac "$OUT/cohorts.tsv" > "$OUT/cohorts-order.tsv"
    fi
  while IFS=$'\t' read -r cohort server_workers client_workers server_cpus client_cpus; do
    while read -r connections trips bytes compute_rounds light_per_second; do
      for admitted in "${admissions[@]}"; do
        for form in "${order[@]}"; do
            echo "network pass=$pass cohort=$cohort form=$form connections=$connections bytes=$bytes compute=$compute_rounds"
            network_case "$form" "$connections" "$trips" "$bytes" "$pass" 0
        done
      done
    done < "$OUT/cases.tsv"
  done < "$OUT/cohorts-order.tsv"
done

# Existing compiler-independent expected bytes from the Windows qualification.
# Warm positioned reads plus compute measure coexistence; they do not establish
# a bound on network latency while every worker runs a long computation.
cpu_programs=(compute mixed)
if [[ $EXPERIMENT != idle && $EXPERIMENT != checkpoint && $EXPERIMENT != chunks && $EXPERIMENT != canonical && $EXPERIMENT != owner && $EXPERIMENT != dispatch && $EXPERIMENT != wake && $EXPERIMENT != service && $EXPERIMENT != memory && $EXPERIMENT != storage && $EXPERIMENT != coroutine ]]; then cpu_programs=(); fi
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

# Keep raw samples; summarize ranges as well as medians. Combine's control
# is callee-small; other panels retain base. Both use the same pass and cohort.
paired_reference=base
if [[ $MODE == combine ]]; then paired_reference=callee-small; fi
awk -F '\t' -v reference="$paired_reference" '
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
          denominator=samples[c,reference];
          if(denominator<=0) { print "scheduler-bench: missing paired reference " reference > "/dev/stderr"; exit 1 }
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
if [[ $MODE == profile ]]; then
    mv "$OUT/network.tsv" "$OUT/profile.tsv"
    mv "$OUT/network-summary.txt" "$OUT/profile-summary.txt"
fi
