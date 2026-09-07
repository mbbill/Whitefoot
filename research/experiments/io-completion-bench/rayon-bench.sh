#!/usr/bin/env bash
# CPU-only external control for the existing par_layout.wf workload.
# Owned by io-model/SCHEDULER-EXPERIMENT.md; remove with that comparison.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=${ROOT:-$(cd "$HERE/../../.." && pwd)}
OUT=${OUT:-${WHITEFOOT_SCRATCH_ROOT:-$HOME/do_not_scan}/whitefoot-rayon-bench}
CLANG=${CLANG:-/usr/bin/clang}
WFC=${WFC:-$ROOT/compiler/target/gate/whitefootc}
ROUNDS=${ROUNDS:-5}
WARMUP=${WARMUP:-1}
CALIBRATION_ROUNDS=${CALIBRATION_ROUNDS:-3}
BATCHES=${BATCHES:-1}
RAYON_THREADS=${RAYON_THREADS:-"1 2 4"}
RAYON_GRAINS=${RAYON_GRAINS:-"1 4 16"}
RAYON_PROFILE=${RAYON_PROFILE:-0}

RESOURCE_CONTROLS=${RESOURCE_CONTROLS:-0}
EXPECTED='420a993efa7437a1 41fa962893d45299'
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$OUT/cargo-target}

bounded_integer() {
    [[ $2 =~ ^[0-9]+$ ]] && (( 10#$2 >= $3 && 10#$2 <= $4 )) || {
        echo "rayon-bench: $1 must be $3..$4" >&2
        exit 2
    }
}
bounded_integer ROUNDS "$ROUNDS" 1 128
bounded_integer WARMUP "$WARMUP" 0 128
bounded_integer CALIBRATION_ROUNDS "$CALIBRATION_ROUNDS" 1 128
bounded_integer BATCHES "$BATCHES" 1 16
bounded_integer RAYON_PROFILE "$RAYON_PROFILE" 0 1

bounded_integer RESOURCE_CONTROLS "$RESOURCE_CONTROLS" 0 1
if [[ $RESOURCE_CONTROLS == 1 && $RAYON_PROFILE == 1 ]]; then
    echo 'rayon-bench: resource controls and perf captures are separate experiments' >&2
    exit 2
fi
read -r -a threads <<< "$RAYON_THREADS"
read -r -a grains <<< "$RAYON_GRAINS"
(( ${#threads[@]} > 0 && ${#grains[@]} > 0 )) || exit 2
for width in "${threads[@]}"; do bounded_integer THREADS "$width" 1 256; done
for grain in "${grains[@]}"; do bounded_integer GRAIN "$grain" 1 64; done

{
    git -C "$ROOT" rev-parse HEAD
    git -C "$ROOT" status --short
    uname -a
    rustc -vV
    cargo -V
    printf 'runner_compiler=%s\n' "$CLANG"
    "$CLANG" --version
    # whitefootc.rs::clang_executable fixes the non-Windows native linker to
    # this path; CLANG above controls only this experiment's C timing runner.
    printf 'wf_native_link_compiler=/usr/bin/clang (compiler-owned selection)\n'
    /usr/bin/clang --version
    printf 'RUSTFLAGS=%s\nCARGO_ENCODED_RUSTFLAGS=%s\n' \
        "${RUSTFLAGS:-}" "${CARGO_ENCODED_RUSTFLAGS:-}"
    if [[ $RESOURCE_CONTROLS == 0 ]]; then
        printf 'threads=%s grains=%s batches=%s calibration=%s confirmation=%s warmup=%s\n' \
            "$RAYON_THREADS" "$RAYON_GRAINS" "$BATCHES" "$CALIBRATION_ROUNDS" "$ROUNDS" "$WARMUP"
    else
        printf 'threads=4 grain=4 batches=1,4,16 stacks=12,1100 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    fi
    printf 'timing=whole-process; Rayon pool created once, install once; no concurrent I/O\n'
    printf 'resource_controls=%s\n' "$RESOURCE_CONTROLS"
    if [[ $(uname -s) == Linux ]]; then
        lscpu
        awk '/Cpus_allowed_list:/ {print}' /proc/self/status
    else
        sysctl hw.model hw.physicalcpu hw.logicalcpu hw.memsize
    fi
} > "$OUT/host.txt"
cargo build --release --locked --offline --manifest-path "$HERE/rayon-baseline/Cargo.toml"
cp "$CARGO_TARGET_DIR/release/whitefoot-rayon-baseline" "$OUT/rust-layout"
cp "$HERE/rayon-baseline/Cargo.lock" "$OUT/Cargo.lock"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror "$HERE/runner.c" -o "$OUT/runner"
if [[ ! -x $WFC ]]; then
    echo 'rayon-bench: build whitefootc or set WFC to the compiler for this revision' >&2
    exit 2
fi
"$WFC" --no-overlap "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-seq"
"$WFC" --par "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-par"
shasum -a 256 "$WFC" "$OUT/wf-seq" "$OUT/wf-par" "$OUT/rust-layout" \
    "$ROOT/tests/programs/par_layout.wf" "$ROOT/compiler/src/bin/whitefootc.rs" \
    "$HERE/rayon-baseline/src/main.rs" "$HERE/rayon-baseline/Cargo.toml" \
    "$HERE/rayon-baseline/Cargo.lock" >> "$OUT/host.txt"

if [[ $RESOURCE_CONTROLS == 1 ]]; then
    # Experiment 45 freezes the earlier four-worker calibration. Do not tune
    # against these samples: only WF stack capacity and batch count vary.
    printf 'resource_panel=workers4; Rayon grain4; batches1,4,16; WF_STACKS12,1100\n' >> "$OUT/host.txt"
    printf 'resource_budget=WF main+3 workers; Rayon 4 workers+sleeping caller; no CPU affinity\n' >> "$OUT/host.txt"
    : > "$OUT/resource.plan"
    for batches in 1 4 16; do
        printf 'rayon.w4.g4.b%s\t\t%s\trayon\t4\t4\t%s\n' \
            "$batches" "$OUT/rust-layout" "$batches" >> "$OUT/resource.plan"
        for stacks in 12 1100; do
            printf 'wf.w4.s%s.b%s\tWF_WORKERS=4,WF_STACKS=%s,WF_SCHED_REPORT=0\t%s' \
                "$stacks" "$batches" "$stacks" "$OUT/wf-par" >> "$OUT/resource.plan"
            for ((batch=1; batch<batches; batch++)); do printf '\tbatch' >> "$OUT/resource.plan"; done
            printf '\n' >> "$OUT/resource.plan"
        done
    done
    WF_BENCH_RAW="$OUT/resource.tsv" "$OUT/runner" "$OUT/resource.plan" \
        "$ROUNDS" "$WARMUP" "$EXPECTED" > "$OUT/resource.txt" 2> "$OUT/resource.err"

    # Existing observer, separately linked and never timed. The normal WF
    # executable above remains the compiler's ordinary native output. The
    # completion units match the ordinary module's write_once output path
    # and provide grant_observer.c's bridge-report dependency. Final stdout
    # output initializes the bridge even though the layout work is CPU-only.
    backend="$ROOT/compiler/src/backend"
    "$WFC" --par --emit-llvm "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-par.ll"
    observer_sources=()
    for unit in wf_floor.c sched/core.c sched/prim_host.c sched/entry.c \
        completion/runtime.c completion/wait_host.c completion/file_adapter.c \
        completion/file_posix.c completion/bridge.c completion/linux_io_uring.c \
        sched/grant_observer.c; do observer_sources+=("$backend/$unit"); done
    observer_command=(/usr/bin/clang -std=c11 -O2 -pthread -I "$backend" \
        -I "$backend/completion" -DWF_SCHED_OBSERVE=1 -x c "${observer_sources[@]}" \
        -x ir "$OUT/wf-par.ll" -Wno-override-module -lm -o "$OUT/wf-par-observed")
    printf '%q ' "${observer_command[@]}" > "$OUT/observer.command"
    printf '\n' >> "$OUT/observer.command"
    "${observer_command[@]}"
    shasum -a 256 "$OUT/wf-par.ll" "$OUT/wf-par-observed" "${observer_sources[@]}" \
        "$backend/sched/core.h" "$backend/sched/prim.h" "$backend/sched/switch.h" \
        "$HERE/runner.c" "$HERE/rayon-bench.sh" >> "$OUT/host.txt"
    printf 'observation=untimed WF_SCHED_OBSERVE=1; exhausted_compute counts no-target join turns, not peak stacks\n' >> "$OUT/host.txt"
    for batches in 1 4 16; do
        observed_args=("$OUT/wf-par-observed")
        for ((batch=1; batch<batches; batch++)); do observed_args+=(batch); done
        for stacks in 12 1100; do
            record="$OUT/observed-s$stacks-b$batches"
            WF_WORKERS=4 WF_STACKS=$stacks WF_SCHED_REPORT=1 "${observed_args[@]}" \
                > "$record.out" 2> "$record.err"
            printf '%s\n' "$EXPECTED" | cmp - "$record.out"
            awk '/^sched:/ {seen++; for(i=2;i<=NF;i++) {split($i,p,"=");v[p[1]]=p[2]+0}}
                END {exit !(seen==1 && v["observed"]==1 && v["threads"]==4 &&
                    v["workers_started"]==3 && v["steals"]>0 && ("exhausted_compute" in v))}' "$record.err"
        done
    done
    cat "$OUT/resource.txt"
    printf 'rayon-bench: resource samples and separate observations retained in %s\n' "$OUT"
    exit 0
fi

# Each native process requests BATCHES explicitly. WF's complete argument
# count includes its executable, so BATCHES - 1 dummy args select the same work.
wf_args=("$OUT/wf-seq")
for ((batch=1; batch<BATCHES; batch++)); do wf_args+=(batch); done
"${wf_args[@]}" > "$OUT/wf-seq.out"
printf '%s\n' "$EXPECTED" | cmp - "$OUT/wf-seq.out"
wf_args[0]="$OUT/wf-par"
for width in "${threads[@]}"; do
    WF_WORKERS=$width WF_STACKS=1100 "${wf_args[@]}" > "$OUT/wf-par-$width.out"
    printf '%s\n' "$EXPECTED" | cmp - "$OUT/wf-par-$width.out"
done

# Calibration is an independent alternating cohort. The best grain at each
# pool width is frozen before any confirmation sample is collected.
printf 'rust-seq\t\t%s\tseq\t1\t64\t%s\n' "$OUT/rust-layout" "$BATCHES" > "$OUT/calibration.plan"
for width in "${threads[@]}"; do
    for grain in "${grains[@]}"; do
        printf 'rayon.w%s.g%s\t\t%s\trayon\t%s\t%s\t%s\n' \
            "$width" "$grain" "$OUT/rust-layout" "$width" "$grain" "$BATCHES" >> "$OUT/calibration.plan"
    done
done
WF_BENCH_RAW="$OUT/calibration.tsv" "$OUT/runner" "$OUT/calibration.plan" \
    "$CALIBRATION_ROUNDS" 1 "$EXPECTED" > "$OUT/calibration.txt" 2> "$OUT/calibration.err"
awk '$1 ~ /^rayon[.]w/ {
    split($1,p,"."); width=substr(p[2],2); grain=substr(p[3],2);
    if (!(width in best) || $2 < best[width]) {best[width]=$2; selected[width]=grain}
} END {for(width in selected) print width "\t" selected[width]}' \
    "$OUT/calibration.txt" | sort -n > "$OUT/selected.tsv"
[[ $(wc -l < "$OUT/selected.tsv") -eq ${#threads[@]} ]] || {
    echo 'rayon-bench: calibration did not select every requested pool width' >&2
    exit 1
}

awk '$1=="rust-seq"' "$OUT/calibration.plan" > "$OUT/confirmation.plan"
while IFS=$'\t' read -r width grain; do
    awk -F '\t' -v name="rayon.w$width.g$grain" '$1==name' "$OUT/calibration.plan" >> "$OUT/confirmation.plan"
done < "$OUT/selected.tsv"
printf 'wf-seq\t\t%s' "$OUT/wf-seq" >> "$OUT/confirmation.plan"
for ((batch=1; batch<BATCHES; batch++)); do printf '\tbatch' >> "$OUT/confirmation.plan"; done
printf '\n' >> "$OUT/confirmation.plan"
for width in "${threads[@]}"; do
    printf 'wf-par.w%s\tWF_WORKERS=%s,WF_STACKS=1100\t%s' "$width" "$width" "$OUT/wf-par" >> "$OUT/confirmation.plan"
    for ((batch=1; batch<BATCHES; batch++)); do printf '\tbatch' >> "$OUT/confirmation.plan"; done
    printf '\n' >> "$OUT/confirmation.plan"
done
WF_BENCH_RAW="$OUT/confirmation.tsv" "$OUT/runner" "$OUT/confirmation.plan" \
    "$ROUNDS" "$WARMUP" "$EXPECTED" > "$OUT/confirmation.txt" 2> "$OUT/confirmation.err"
cat "$OUT/calibration.txt" "$OUT/selected.tsv" "$OUT/confirmation.txt"
printf 'rayon-bench: raw samples and exact-byte qualifications retained in %s\n' "$OUT"

if [[ $RAYON_PROFILE == 1 ]]; then
    [[ $(uname -s) == Linux ]] || { echo 'rayon-bench: profiling requires Linux' >&2; exit 2; }
    perf_tool=${PROFILE_PERF:-perf}
    profile_user=$(id -un)
    sudo -n true
    mkdir -p "$OUT/profile"
    {
        "$perf_tool" --version
        echo 'profile_batches=4; ordinary binaries; separate CPU and scheduler captures; no profiled timings enter confirmation.tsv'
        echo "workload_user=$profile_user; privileged recorder only; workload inherits the normal job affinity"
        cat /proc/sys/kernel/perf_event_paranoid
        cat /proc/sys/kernel/kptr_restrict
    } > "$OUT/profile/host.txt"
    # The recorder needs scheduler tracepoint access. Drop back to the normal
    # job user before exec, preserve the real workload PID, and separate its
    # strict output channels from profiler diagnostics.
    cat > "$OUT/profile/run" <<'PROFILE_RUN'
#!/bin/sh
record=$1
shift
printf '%s\n' "$$" > "$record.pid"
exec "$@" > "$record.out" 2> "$record.err"
PROFILE_RUN
    chmod +x "$OUT/profile/run"
    profile_case() {
        local kind=$1 label=$2 record
        shift 2
        record="$OUT/profile/$kind-$label"
        printf '%q ' "$@" > "$record.command"
        printf '\n' >> "$record.command"
        if [[ $kind == cpu ]]; then
            sudo -n "$perf_tool" record -e cpu-clock -F 999 -o "$record.data" -- \
                sudo -n -u "$profile_user" -- "$OUT/profile/run" "$record" "$@" \
                > "$record.recorder.out" 2> "$record.recorder.err"
        else
            sudo -n "$perf_tool" sched record -a -o "$record.data" -- \
                sudo -n -u "$profile_user" -- "$OUT/profile/run" "$record" "$@" \
                > "$record.recorder.out" 2> "$record.recorder.err"
        fi
        printf '%s\n' "$EXPECTED" | cmp - "$record.out"
        [[ ! -s $record.err && -s $record.pid ]]
        sudo -n chown "$(id -u):$(id -g)" "$record.data"
        if [[ $kind == cpu ]]; then
            "$perf_tool" report --stdio --header --show-nr-samples --no-children \
                --pid "$(cat "$record.pid")" --sort pid,dso,symbol -i "$record.data" > "$record.report" 2> "$record.report.err"
            "$perf_tool" script --show-lost-events -i "$record.data" \
                -F comm,pid,tid,time,event,ip,sym,dso,period > "$record.samples" 2> "$record.script.err"
        else
            "$perf_tool" sched timehist -i "$record.data" --pid "$(cat "$record.pid")" \
                --state --wakeups --migrations --with-summary > "$record.timehist" 2> "$record.timehist.err"
            "$perf_tool" script --show-lost-events -i "$record.data" > "$record.samples" 2> "$record.script.err"
            [[ -s $record.timehist ]]
        fi
        [[ -s $record.samples ]]
        echo "rayon-profile: $kind $label captured; inspect lost events before attributing costs"
    }
    # Four identical source batches amortize startup in observations without
    # changing the preceding one-batch calibration or confirmation panel.
    for kind in cpu sched; do
        profile_case "$kind" rust-seq "$OUT/rust-layout" seq 1 64 4
        profile_case "$kind" wf-seq "$OUT/wf-seq" batch batch batch
        while IFS=$'\t' read -r width grain; do
            profile_forms=(wf rayon)
            if [[ $kind == sched ]]; then profile_forms=(rayon wf); fi
            for form in "${profile_forms[@]}"; do
                if [[ $form == wf ]]; then
                    profile_case "$kind" "wf-w$width" env "WF_WORKERS=$width" WF_STACKS=1100 \
                        "$OUT/wf-par" batch batch batch
                else
                    profile_case "$kind" "rayon-w$width-g$grain" "$OUT/rust-layout" rayon "$width" "$grain" 4
                fi
            done
        done < "$OUT/selected.tsv"
    done
fi
