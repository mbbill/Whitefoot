#!/bin/sh
# Build a private diagnostic Rayon fork for the scheduler event experiment.
# Retire this build and its patch when that pinned event boundary is retired.
set -eu
cd "$(dirname "$0")"
case "${1:-build}" in build) ;; *) echo 'expected build' >&2; exit 1;; esac
: "${OUT:?absolute experiment output required}"
case "$OUT" in /*) ;; *) echo 'Rayon event OUT must be absolute' >&2; exit 1;; esac
test "$OUT" != /
prefix=$OUT/scheduler-events-deps
source=$prefix/rayon-source
vendor=$source/vendor
fork=$source/fork/rayon-core
project=$source/adapter
target=$prefix/rayon
mkdir -p "$source" "$source/fork" "$project" "$target"

# The ordinary lock and registry remain read-only. Retain the exact vendored
# originals, then modify only a separate path dependency for rayon-core.
cargo vendor --locked --offline --versioned-dirs \
    --manifest-path records-rayon/Cargo.toml "$vendor" > "$source/vendor-config.toml"
test -f "$vendor/rayon-core-1.13.0/src/job.rs"
test -f "$vendor/rayon-1.12.0/src/lib.rs"
rm -rf "$fork"
cp -R "$vendor/rayon-core-1.13.0" "$fork"
# A path dependency is intentionally modified; its original checksum file
# remains in vendor, where Cargo still checks the untouched registry source.
rm "$fork/.cargo-checksum.json"
patch_file=$(pwd)/records-rayon-events.patch
(cd "$fork" && git apply --check "$patch_file" && git apply "$patch_file")

# Use the same adapter and run ABI. Only the private diagnostic copy has the
# getter or direct rayon-core dependency; the normal build has neither.
cp records-rayon/Cargo.toml "$project/Cargo.toml"
cp records-rayon/Cargo.lock "$project/Cargo.lock"
cp records-rayon/adapter.rs "$project/adapter.rs"
cat >> "$project/Cargo.toml" <<'TOML'

[dependencies.rayon-core]
version = "=1.13.0"

[patch.crates-io]
rayon-core = { path = "../fork/rayon-core" }
TOML
cat >> "$project/adapter.rs" <<'RUST'

// Bind this ordinary C ABI symbol from the exact diagnostic build's LLVM IR.
pub extern "C" fn event(worker: u32, event: u32) -> u64 {
    rayon_core::diagnostic_count(worker as usize, event as usize)
}
RUST
# Offline resolution sees only the versioned packages in this locked vendor
# set; the local lock additionally records the direct patched-core dependency.
cargo --config "$source/vendor-config.toml" generate-lockfile --offline \
    --manifest-path "$project/Cargo.toml"
rayon_flags='-C no-vectorize-loops -C no-vectorize-slp -C lto=off -C symbol-mangling-version=v0'
RUSTFLAGS="$rayon_flags" cargo --config "$source/vendor-config.toml" rustc --locked --offline --release \
    --manifest-path "$project/Cargo.toml" --target-dir "$target" -- --emit=llvm-ir,link
case "$(uname -s)" in Darwin) label_prefix=_;; Linux) label_prefix=;; *) echo 'Rayon event host is unqualified' >&2; exit 1;; esac
for name in run event; do
    case "$name" in run) length=3;; event) length=5;; esac
    awk -v suffix="$length$name" '
        /^define .* @_RNv/ {
            split($0,a,"@"); symbol=a[2]; sub(/\(.*/,"",symbol)
            if (symbol ~ (suffix "$")) { found=symbol; count++ }
        }
        END { if(count!=1)exit 1;print found }
    ' "$target"/release/deps/wf_records_rayon-*.ll > "$source/export-$name.txt"
done
{
    printf '#ifndef WF_RAYON_EVENTS_BINDING_H\n#define WF_RAYON_EVENTS_BINDING_H\n'
    printf '#define RAYON_SYMBOL "%s%s"\n' "$label_prefix" "$(cat "$source/export-run.txt")"
    printf '#define RAYON_EVENT_SYMBOL "%s%s"\n' "$label_prefix" "$(cat "$source/export-event.txt")"
    printf '#define RAYON_EVENT_COUNT 13\n#define RAYON_EVENT_BANKS 5\n#endif\n'
} > "$prefix/rayon-binding.h"
cp "$prefix/rayon-binding.h" "$prefix/events-binding.h"
{
    printf '%s\n' 'diagnostic-only=rayon-1.12.0/rayon-core-1.13.0; one pool per process' \
        'workers=0..3; bank4=external submitters; counters=cumulative relaxed atomics; reset=absent' \
        'events=publish,local_pop,foreign_steal,stack_complete,inline_complete,execute_complete,inject_publish,inject_pop,broadcast_publish,broadcast_pop,heap_entry,arc_entry,fifo_entry' \
        'qualification=join-only; inject/broadcast/non-stack events must be zero for its conservation equation' \
        'instrumentation=separate per-worker banks; completion counts precede latch publication; includes run_inline' \
        'runtime-job-events are instrumented observations, not ordinary timing or OS context-switch events'
    printf 'RUSTFLAGS=%s\n' "$rayon_flags"
    rustc -vV
    cargo --version
    cargo --config "$source/vendor-config.toml" tree --locked --offline --manifest-path "$project/Cargo.toml"
    shasum -a 256 records-rayon-events.patch records-rayon-events.sh \
        records-rayon/Cargo.toml records-rayon/Cargo.lock records-rayon/adapter.rs \
        "$project/Cargo.toml" "$project/Cargo.lock" "$project/adapter.rs" \
        "$source/vendor-config.toml" "$prefix/rayon-binding.h" "$prefix/events-binding.h" \
        "$target/release/libwf_records_rayon.a"
    find "$vendor" "$fork" -type f -exec shasum -a 256 {} +
} > "$prefix/metadata.txt"
printf 'Rayon event build PASS: archive=%s header=%s\n' \
    "$target/release/libwf_records_rayon.a" "$prefix/rayon-binding.h"
