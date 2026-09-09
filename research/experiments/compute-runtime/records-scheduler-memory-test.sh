#!/bin/sh
# Grammar regressions for the pinned Rayon lifecycle classifier, on every host.
# Called by check-scheduler-memory and check-quadrature; retire with the checker.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT}"
out="$OUT/sanitizer-reports"
mkdir -p "$out"
# Relocated semantic frames from retained Linux quadrature reports. These are
# parser fixtures, not a claim that this host ran Linux LeakSanitizer.
cat > "$out/full" <<'EOF'

=================================================================
==123==ERROR: LeakSanitizer: detected memory leaks

Direct leak of 384 byte(s) in 1 object(s) allocated from:
    #0 0x123 in posix_memalign (/fixture/sanitized+0x123) (BuildId: abc)
    #1 0x123 in std::sys::alloc::unix::aligned_malloc /rustc/abc/library/std/src/sys/alloc/unix.rs:83:32
    #2 0x123 in <std::alloc::System as core::alloc::global::GlobalAlloc>::alloc /rustc/abc/library/std/src/sys/alloc/unix.rs:28:22
    #3 0x123 in __rustc::__rdl_alloc /rustc/abc/library/std/src/alloc.rs:455:20
    #4 0x123 in <rayon_core::thread_pool::ThreadPool>::build::<rayon_core::registry::DefaultSpawn> wf_records_rayon.abc-cgu.0
    #5 0x123 in <std::sync::once::Once>::call_once_force::<<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::initialize<<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::get_or_init<wf_records_rayon::quadrature::quadrature::{closure#0}>::{closure#0}, !>::{closure#0}>::{closure#0} wf_records_rayon.abc-cgu.0
    #6 0x123 in <std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::initialize::<<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::get_or_init<wf_records_rayon::quadrature::quadrature::{closure#0}>::{closure#0}, !> wf_records_rayon.abc-cgu.0

Indirect leak of 1520 byte(s) in 1 object(s) allocated from:
    #0 0x123 in calloc (/fixture/sanitized+0x123) (BuildId: abc)
    #1 0x123 in <crossbeam_deque::deque::Block<rayon_core::job::JobRef>>::new (/fixture/sanitized+0x123) (BuildId: abc)
    #2 0x123 in <rayon_core::thread_pool::ThreadPool>::build::<rayon_core::registry::DefaultSpawn> wf_records_rayon.abc-cgu.0
    #3 0x123 in <std::sync::once::Once>::call_once_force::<<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::initialize<<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::get_or_init<wf_records_rayon::quadrature::quadrature::{closure#0}>::{closure#0}, !>::{closure#0}>::{closure#0} wf_records_rayon.abc-cgu.0
    #4 0x123 in <std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::initialize::<<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::get_or_init<wf_records_rayon::quadrature::quadrature::{closure#0}>::{closure#0}, !> wf_records_rayon.abc-cgu.0

SUMMARY: AddressSanitizer: 1904 byte(s) leaked in 2 allocation(s).
EOF
awk '/^Direct leak/ {exit} {print}' "$out/full" > "$out/header"
awk '/^Direct leak/ {copy=1} /^Indirect leak/ {exit} copy {print}' "$out/full" > "$out/worker"
awk '/^Indirect leak/ {copy=1} /^SUMMARY:/ {exit} copy {print}' "$out/full" > "$out/queue"
cat "$out/header" "$out/worker" > "$out/worker-only"
printf '%s\n' 'SUMMARY: AddressSanitizer: 384 byte(s) leaked in 1 allocation(s).' >> "$out/worker-only"
cat > "$out/injection" <<'EOF'
Direct leak of 257 byte(s) in 1 object(s) allocated from:
    #0 0x123 in malloc /fixture/probe.c:1:1
    #1 0x123 in records_scheduler_leak_probe /fixture/probe.c:2:1

EOF
check() {
    awk -v leak=1 -v pool_owner=quadrature -v object_basename=sanitized \
        -v extra="${2:-0}" -v prefix="$out/prefix" \
        -f records-scheduler-memory.awk "$out/$1" || return $?
    test ! -s "$out/prefix"
}
reject() {
    if check "$1" > "$out/$1.rejection" 2>&1; then
        echo "sanitizer grammar accepted $1" >&2; exit 1
    fi
    grep -q '^scheduler sanitizer report rejected:' "$out/$1.rejection"
}
for shape in full worker-only; do
    check "$shape"
    sed '$d' "$out/$shape" > "$out/missing-summary"; reject missing-summary
    sed 's/byte(s) leaked in [12] allocation/byte(s) leaked in 3 allocation/' "$out/$shape" > "$out/wrong-count"; reject wrong-count
    sed '/^SUMMARY:/s/[0-9][0-9]*/9999/' "$out/$shape" > "$out/wrong-bytes"; reject wrong-bytes
    sed 's/ThreadPool>::build/ThreadPool>::unexpected/' "$out/$shape" > "$out/wrong-stack"; reject wrong-stack
    sed 's/Direct leak of 384/Direct leak of 385/' "$out/$shape" > "$out/wrong-size"; reject wrong-size
    sed '/^    #3 /d' "$out/$shape" > "$out/missing-frame"; reject missing-frame
    cat "$out/$shape" > "$out/trailing"; printf 'unexpected diagnostic\n' >> "$out/trailing"; reject trailing
    sed '$d' "$out/$shape" > "$out/extra"
    cat "$out/injection" >> "$out/extra"
    bytes=641; count=2
    if test "$shape" = full; then bytes=2161; count=3; fi
    printf 'SUMMARY: AddressSanitizer: %s byte(s) leaked in %s allocation(s).\n' "$bytes" "$count" >> "$out/extra"
    check extra 257
    reject extra
done
cat "$out/header" "$out/queue" > "$out/queue-only"
printf 'SUMMARY: AddressSanitizer: 1520 byte(s) leaked in 1 allocation(s).\n' >> "$out/queue-only"
reject queue-only
cat "$out/header" "$out/worker" "$out/worker" > "$out/duplicate-worker"
printf 'SUMMARY: AddressSanitizer: 768 byte(s) leaked in 2 allocation(s).\n' >> "$out/duplicate-worker"
reject duplicate-worker
cat "$out/header" "$out/worker" "$out/queue" "$out/queue" > "$out/duplicate-queue"
printf 'SUMMARY: AddressSanitizer: 3424 byte(s) leaked in 3 allocation(s).\n' >> "$out/duplicate-queue"
reject duplicate-queue
sed 's/Block<rayon_core::job::JobRef>/Block<unknown::JobRef>/' "$out/full" > "$out/wrong-queue"
reject wrong-queue
printf 'sanitizer report grammar PASS: known worker with optional known queue; unknown allocations rejected\n'
