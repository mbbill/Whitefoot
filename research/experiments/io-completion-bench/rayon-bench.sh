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
    "$CLANG" --version
    printf 'threads=%s grains=%s batches=%s calibration=%s confirmation=%s warmup=%s\n' \
        "$RAYON_THREADS" "$RAYON_GRAINS" "$BATCHES" "$CALIBRATION_ROUNDS" "$ROUNDS" "$WARMUP"
    printf 'timing=whole-process; Rayon pool created once, install once; no concurrent I/O\n'
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
    "$ROOT/tests/programs/par_layout.wf" >> "$OUT/host.txt"

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
