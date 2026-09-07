#!/usr/bin/env bash
# Linux-only qualification of the existing netload protocol with bounded
# Rayon offload. Experiment 42 owns this caller; retire with that comparison.
# Four short samples qualify operation, not a comparative performance claim.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=${ROOT:-$(cd "$HERE/../../.." && pwd)}
OUT=${OUT:-${WHITEFOOT_SCRATCH_ROOT:-$HOME/do_not_scan}/whitefoot-mixed-rayon}
CLANG=${CLANG:-/usr/bin/clang}
[[ $(uname -s) == Linux ]] || { echo 'mixed-rayon-smoke: Linux netload required' >&2; exit 2; }
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$OUT/cargo-target}
server_pid=''
cleanup() {
    if [[ -n $server_pid ]]; then kill -- "-$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT
{
    git -C "$ROOT" rev-parse HEAD
    git -C "$ROOT" status --short
    uname -a
    rustc -vV
    cargo -V
    "$CLANG" --version
    lscpu
    awk '/Cpus_allowed_list:/ {print}' /proc/self/status
    printf 'qualification_only=1 total_threads=2,4 io_threads=1 queue=2*(total-1) connections=16 duration_ms=500 light_rate_per_peer=100 heavy_rounds=1048576\n'
    printf 'affinity=unrestricted; client shares host; normal and observed builds are separate\n'
} > "$OUT/host.txt"
"$CLANG" -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -pthread \
    -DWF_NETLOAD_SERVICE_ROUNDS=8 "$HERE/netload.c" -o "$OUT/netload"
for observed in 0 1; do
    features=mixed
    if [[ $observed == 1 ]]; then features=mixed-observe; fi
    cargo build --release --features "$features" --bin mixed-rayon --locked --offline \
        --manifest-path "$HERE/rayon-baseline/Cargo.toml"
    cp "$CARGO_TARGET_DIR/release/mixed-rayon" "$OUT/mixed-$observed"
    for threads in 2 4; do
        port=$((42000 + threads * 2 + observed))
        stem="$OUT/b$threads-observed$observed"
        setsid timeout --signal=TERM --kill-after=5s 30s /usr/bin/time \
            -f '%U\t%S\t%M\t%w\t%c' -o "$stem.resources.tsv" \
            "$OUT/mixed-$observed" "$port" 16 --threads "$threads" \
            > "$stem.server.out" 2> "$stem.server.err" &
        server_pid=$!
        listening=0
        for ((attempt=0; attempt<500; attempt++)); do
            if ss -H -ltn "sport = :$port" | awk 'END {exit NR==0}'; then listening=1; break; fi
            kill -0 "$server_pid" 2>/dev/null || { cat "$stem.server.err" >&2; exit 1; }
            sleep 0.01
        done
        [[ $listening == 1 ]] || { echo 'mixed-rayon-smoke: listener timeout' >&2; exit 1; }
        timeout --signal=TERM --kill-after=5s 30s "$OUT/netload" "$port" 16 32768 64 \
            --threads 1 --compute 1048576 --heavy-every 4 --admit \
            --duration-ms 500 --light-per-second 100 > "$stem.client.tsv" 2> "$stem.client.err"
        if ! wait "$server_pid"; then cat "$stem.server.err" >&2; exit 1; fi
        server_pid=''
        [[ ! -s $stem.server.out && ! -s $stem.client.err ]]
        if [[ $observed == 0 ]]; then
            [[ ! -s $stem.server.err ]]
        else
            awk -v threads="$threads" '/^mixed-rayon:/ {
                for(i=2;i<=NF;i++) {split($i,a,"=");v[a[1]]=a[2]+0}; seen++
            } END {exit !(seen==1 && v["total_threads"]==threads && v["io_threads"]==1 &&
                v["cpu_threads"]==threads-1 && v["queue"]==2*(threads-1) &&
                v["submitted"]>0 && v["submitted"]==v["completed"] && v["inflight"]==0 &&
                v["waiting"]==0 && v["active"]==0 && v["inflight_peak"]<=v["queue"] &&
                v["active_peak"]<=threads-1 && v["light_while_saturated"]>0)}' "$stem.server.err"
        fi
        printf 'mixed-rayon-smoke: PASS total_threads=%s observed=%s\n' "$threads" "$observed"
        cat "$stem.client.tsv" "$stem.server.err"
    done
done
cp "$HERE/rayon-baseline/Cargo.lock" "$OUT/Cargo.lock"
shasum -a 256 "$OUT/mixed-0" "$OUT/mixed-1" "$OUT/netload" \
    "$HERE/rayon-baseline/src/mixed.rs" "$HERE/rayon-baseline/Cargo.toml" \
    "$HERE/rayon-baseline/Cargo.lock" "$HERE/netload.c" "$HERE/compute_protocol.h" >> "$OUT/host.txt"
