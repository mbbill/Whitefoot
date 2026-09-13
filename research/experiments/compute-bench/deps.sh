#!/bin/sh
# Serves compute-bench: the pinned third-party schedulers the native reference
# rows are built from. `fetch` is the only network step; `build` is offline.
#
#   OUT=<absolute dir> sh deps.sh fetch     clone oneTBB and ParlayLib at their
#                                           pins and populate the cargo cache
#   OUT=<absolute dir> sh deps.sh build     install into OUT, offline
#
# Carried from the research bundle's records-scheduler-deps.sh with the ten
# named changes the specification lists; each is marked CHANGE below.
set -eu
# CHANGE 1: the original only worked when the caller's cwd was the bundle
# directory. The cargo manifest path below is relative to this script.
cd "$(dirname "$0")"
mode=${1:?fetch or build required}
case "$mode" in fetch|build) ;; *) exit 1;; esac
cache=${WHITEFOOT_SCRATCH_ROOT:-${TMPDIR:-/tmp}/whitefoot}/whitefoot-compute-deps
tbb=${TBB_SOURCE:-$cache/onetbb-3046c8b0}
parlay=${PARLAY_SOURCE:-$cache/parlay-51017699}
tbb_pin=3046c8b0c29df995980003ea24f4d78c80ec0c8d
parlay_pin=51017699dcc421f80479cdb238d3092233ad0d26
# CHANGE 2: the original failed on a bare `test` with no output, leaving a
# cached clone that could not repair itself and had to be deleted by hand.
verify() {
    if test "$(git -C "$1" rev-parse HEAD)" != "$2"; then
        echo "compute-bench deps: $1 is not at pin $2; delete it and rerun fetch" >&2
        exit 1
    fi
    if test -n "$(git -C "$1" status --porcelain --untracked-files=all)"; then
        echo "compute-bench deps: $1 has local modifications; delete it and rerun fetch" >&2
        exit 1
    fi
}
fetch() {
    if test ! -e "$1"; then
        mkdir -p "$(dirname "$1")"
        git init "$1"
        git -C "$1" remote add origin "$3"
        git -C "$1" fetch --depth=1 origin "$2"
        git -C "$1" checkout --detach "$2"
    fi
    verify "$1" "$2"
}
if test "$mode" = fetch; then
    # CHANGE 7: `fetch` writes only into the cache root, so the OUT checks
    # belong to the build branch. The original ran them before the mode branch
    # and a bare `deps.sh fetch` exited 1 before fetching anything.
    fetch "$tbb" "$tbb_pin" https://github.com/uxlfoundation/oneTBB.git
    fetch "$parlay" "$parlay_pin" https://github.com/cmuparlay/parlaylib.git
    # CHANGE 4: the crate directory is `rayon`, and one build produces one
    # staticlib carrying both entry points. The original's quadrature feature
    # builds and cargo-clean loop are cut.
    cargo fetch --locked --manifest-path rayon/Cargo.toml
    printf 'compute-bench sources PASS: oneTBB=%s Parlay=%s\n' "$tbb_pin" "$parlay_pin"
    exit 0
fi
: "${OUT:?absolute output directory required}"
case "$OUT" in /*) ;; *) echo 'compute-bench OUT must be absolute' >&2; exit 1;; esac
test "$OUT" != /
if test ! -d "$tbb/.git" || test ! -d "$parlay/.git"; then
    echo 'missing pinned schedulers: run `make deps` (its fetch step needs the network) first' >&2
    exit 1
fi
verify "$tbb" "$tbb_pin"
verify "$parlay" "$parlay_pin"
# CHANGE 6: install into $OUT itself. The original used $OUT/scheduler-deps and
# cargo's own release/ directory, one level deeper than the Makefile's
# -isystem $(DEPS)/include, -L$(DEPS)/lib and $(DEPS)/rayon/... expect.
prefix=$OUT
build=$prefix/build
cmake=${CMAKE:-cmake}
cc=${CC:-/usr/bin/clang}
cxx=${CXX:-/usr/bin/clang++}
# `shasum -a 256` is present on macOS and on Linux, but a Linux host without
# Perl has only coreutils. The Makefile probes for it and falls back to
# sha256sum for the image hashes; the metadata hashes below do the same, so
# one missing tool does not fail the whole dependency build.
if command -v shasum > /dev/null 2>&1; then sha256='shasum -a 256'; else sha256=sha256sum; fi
# CHANGE 10: the scalar flag string and BENCH_ARCH arrive from the Makefile
# through the environment, so every compiler flag is written in one place.
scalar=${DEPS_SCALAR_FLAGS:-'-fno-vectorize -fno-slp-vectorize -fno-lto'}
bench_arch=${BENCH_ARCH:-}
mkdir -p "$prefix"
rm -f "$prefix/parlay-unavailable.txt"
# Install only this pinned private dependency, never into the system prefix.
# No hwloc discovery, optional allocator or upstream test target is selected.
"$cmake" -S "$tbb" -B "$build" \
    -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_STANDARD=17 \
    -DCMAKE_C_COMPILER="$cc" -DCMAKE_CXX_COMPILER="$cxx" \
    -DCMAKE_C_FLAGS="$scalar" -DCMAKE_CXX_FLAGS="$scalar" \
    -DCMAKE_INTERPROCEDURAL_OPTIMIZATION=OFF -DTBB_ENABLE_IPO=OFF \
    -DCMAKE_EXPORT_COMPILE_COMMANDS=ON -DBUILD_SHARED_LIBS=ON \
    -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_INSTALL_LIBDIR=lib \
    -DTBB_TEST=OFF -DTBB_EXAMPLES=OFF -DTBBMALLOC_BUILD=OFF \
    -DTBB_DISABLE_HWLOC_AUTOMATIC_SEARCH=ON
# CHANGE 3: the original built with --parallel 2 on every host.
"$cmake" --build "$build" --target tbb --parallel "$(getconf _NPROCESSORS_ONLN)"
"$cmake" --install "$build" --config Release
"$cmake" -E copy_directory "$parlay/include/parlay" "$prefix/include/parlay"
diff -qr "$parlay/include/parlay" "$prefix/include/parlay"
diff -qr "$tbb/include/oneapi" "$prefix/include/oneapi"
diff -qr "$tbb/include/tbb" "$prefix/include/tbb"
case "$(uname -s)" in
    Darwin) library=$prefix/lib/libtbb.dylib;;
    Linux) library=$prefix/lib/libtbb.so;;
    *) echo 'compute-bench dependency host is not qualified' >&2; exit 1;;
esac
test -f "$library"
# CHANGE 9: probe ParlayLib by compiling it. ParlayLib is header-only, so the
# original's header copy could never detect the real exposure, which is at
# compile time of backend_parlay.cpp in its #error guards and its
# PARLAY_ELASTIC_STEAL_TIMEOUT assert. The probe carries the same guards. On
# failure it records the reason and exits 0, and the build then defines
# WFB_NO_PARLAY, which removes the two structs, the two table entries and the
# object together; the table prints both rows n/a with the recorded reason.
cat > "$build/parlay_probe.cpp" <<'PROBE'
/* Serves compute-bench deps: the ParlayLib availability probe. It carries the
   same guards and asserts as backend_parlay.cpp, so a host on which that file
   cannot compile is detected here rather than at build time. */
#if defined(PARLAY_CILKPLUS) || defined(PARLAY_OPENCILK) || defined(PARLAY_OPENMP) || \
    defined(PARLAY_TBB) || defined(PARLAY_SEQUENTIAL)
#error "The compute-bench Parlay reference requires the native scheduler"
#endif
#include <parlay/parallel.h>
#ifndef PARLAY_USING_PARLAY_SCHEDULER
#error "The compute-bench Parlay reference did not select the native scheduler"
#endif
static_assert(PARLAY_ELASTIC_PARALLELISM, "Keep Parlay's default elastic policy");
static_assert(PARLAY_ELASTIC_STEAL_TIMEOUT == 10000, "Keep Parlay's default idle timeout");
int main() {
    parlay::internal::scheduler_type scheduler(2);
    return scheduler.num_workers() >= 2 ? 0 : 1;
}
PROBE
# The probe is compiled AND run: constructing parlay::internal::scheduler_type
# is where a host that compiles the header but cannot start the native
# scheduler fails, and metadata.txt below claims the probe compiled and ran.
# Either failure records its own reason and exits 0, and the build then
# defines WFB_NO_PARLAY.
parlay_reason=
if ! "$cxx" -std=c++17 -DNDEBUG -pthread $scalar $bench_arch -isystem "$prefix/include" \
        "$build/parlay_probe.cpp" -o "$build/parlay_probe" > "$build/parlay_probe.log" 2>&1; then
    parlay_reason='ParlayLib did not compile on this host'
elif ! "$build/parlay_probe" >> "$build/parlay_probe.log" 2>&1; then
    parlay_reason='The ParlayLib probe compiled but its scheduler did not start on this host'
fi
if test -z "$parlay_reason"; then
    printf 'ParlayLib probe PASS: compiled and ran\n'
else
    {
        printf '%s (%s %s)\n' "$parlay_reason" "$(uname -s)" "$(uname -m)"
        cat "$build/parlay_probe.log"
    } > "$prefix/parlay-unavailable.txt"
    printf 'ParlayLib probe FAILED; the parlay rows will print n/a\n' >&2
fi
# Cargo's lock pins the full Rayon dependency graph. Both crate types retain
# the ordinary public C-callable functions in the static archive without an
# unsafe no_mangle/export_name attribute. Fail closed if either emitted symbol
# is not unique; no assumption about a persistent Rust-mangled name escapes
# this exact build. All dependency crates receive the same scalar flags.
rayon=$prefix/rayon
mkdir -p "$rayon"
rayon_flags=${DEPS_RUSTFLAGS:-'-C no-vectorize-loops -C no-vectorize-slp -C lto=off -C symbol-mangling-version=v0'}
# CHANGE 11 (not in the specification's list of ten, and recorded here as an
# addition with its reason): when this one package is rebuilt it is rebuilt
# from scratch. The emitted .ll carries cargo's metadata hash in its name, so
# a second build with a different hash leaves the first beside it and the
# exact-match symbol recovery below then sees the same entry twice and fails.
# Cleaning only this package keeps every dependency crate cached, so the cost
# is one crate.
#
# The rebuild is skipped when the previous one is still complete: exactly one
# emitted .ll, the staticlib at both its cargo path and its fixed name, and
# the recovered symbols header. `build` is a prerequisite of `make build`, and
# a from-scratch crate build plus its LLVM IR on every `make build` is the
# whole reason this script felt slow. The recovery below still runs against
# the kept .ll, so rayon-binding.h is regenerated -- never inherited -- on
# every invocation, and `make deps` stays idempotent. `make clean-deps`
# removes this tree and rebuilds everything.
# The kept build must also be the build of THESE sources: the stamp written
# after every rebuild is the hash of the crate's three files, and a changed
# adapter (the width ceiling moved once) rebuilds instead of being kept.
rayon_sources=$($sha256 rayon/Cargo.toml rayon/Cargo.lock rayon/adapter.rs)
rayon_modules=$(ls "$rayon"/target/release/deps/wf_compute_bench_rayon-*.ll 2>/dev/null | wc -l | tr -d "[:space:]")
if test "$rayon_modules" -eq 1 &&
        test "$(cat "$rayon/sources.sha256" 2>/dev/null)" = "$rayon_sources" &&
        test -f "$rayon/target/release/libwf_compute_bench_rayon.a" &&
        test -f "$rayon/libwf_compute_bench_rayon.a" &&
        test -f "$rayon/rayon-binding.h" && test -f "$prefix/rayon-binding.h"; then
    printf 'Rayon staticlib and recovered symbols already built; keeping them\n'
else
    cargo clean --locked --offline --release --package wf-compute-bench-rayon \
        --manifest-path rayon/Cargo.toml --target-dir "$rayon/target"
    rm -f "$rayon"/target/release/deps/wf_compute_bench_rayon-*.ll
    RUSTFLAGS="$rayon_flags" cargo rustc --locked --offline --release \
        --manifest-path rayon/Cargo.toml --target-dir "$rayon/target" -- --emit=llvm-ir,link
    printf '%s\n' "$rayon_sources" > "$rayon/sources.sha256"
fi
# CHANGE 8: recover two entry symbols, not one. v0 mangling writes the name
# length before the name, so the two patterns are 3map and 5fork2; the
# original's single 3run matches neither and one match can never bind two.
# CHANGE 5: print the pattern, the match count and rustc -vV before failing,
# so a toolchain change does not read as a missing symbol.
: > "$prefix/rayon-binding.h"
printf '/* Generated by deps.sh from the crate%s emitted LLVM IR. */\n' "'s" \
    >> "$prefix/rayon-binding.h"
case "$(uname -s)" in Darwin) label_prefix=_;; *) label_prefix=;; esac
for entry in map fork2; do
    # The opening paren is a bracket expression rather than a backslash escape:
    # awk -v processes escapes in its value, and BSD awk then reads a bare
    # "3map(" as an unbalanced group and stops with "illegal primary".
    pattern="^define .* @_RNv[^ (]*$(printf '%s' "$entry" | wc -c | tr -d ' ')$entry[(]"
    matches=$(awk -v pattern="$pattern" '
        $0 ~ pattern { split($0,a,"@"); symbol=a[2]; sub(/\(.*/,"",symbol); print symbol }
    ' "$rayon"/target/release/deps/wf_compute_bench_rayon-*.ll)
    count=$(printf '%s\n' "$matches" | grep -c . || true)
    if test "$count" != 1; then
        echo "compute-bench deps: Rayon entry '$entry' matched $count symbols" >&2
        echo "  awk pattern: $pattern" >&2
        printf '%s\n' "$matches" >&2
        rustc -vV >&2
        exit 1
    fi
    case "$entry" in
        map) macro=RAYON_MAP_SYMBOL;;
        fork2) macro=RAYON_FORK2_SYMBOL;;
    esac
    printf '#define %s "%s%s"\n' "$macro" "$label_prefix" "$matches" >> "$prefix/rayon-binding.h"
done
# CHANGE 6 (continued): fixed names, so no consumer has to know cargo's layout.
cp "$rayon/target/release/libwf_compute_bench_rayon.a" "$rayon/libwf_compute_bench_rayon.a"
cp "$prefix/rayon-binding.h" "$rayon/rayon-binding.h"
{
    printf 'RUSTFLAGS=%s\n' "$rayon_flags"
    rustc -vV
    cargo --version
    cargo tree --locked --offline --manifest-path rayon/Cargo.toml
    $sha256 rayon/Cargo.toml rayon/Cargo.lock rayon/adapter.rs \
        "$rayon/libwf_compute_bench_rayon.a" "$rayon/rayon-binding.h"
} > "$rayon/metadata.txt"
{
    printf 'oneTBB=%s source=%s\nParlay=%s source=%s\n' "$tbb_pin" "$tbb" "$parlay_pin" "$parlay"
    printf 'library=%s\nC=%s\nCXX=%s\nscalar_flags=%s\nbench_arch=%s\n' \
        "$library" "$cc" "$cxx" "$scalar" "$bench_arch"
    printf 'oneTBB: shared Release C++17 IPO=OFF hwloc=OFF malloc=OFF upstream_tests=OFF\n'
    if test -f "$prefix/parlay-unavailable.txt"; then
        printf 'ParlayLib: unavailable\n'
    else
        printf 'ParlayLib: probe compiled and ran\n'
    fi
    "$cmake" --version
    "$cc" --version
    "$cxx" --version
    $sha256 "$library" "$build/CMakeCache.txt" "$build/compile_commands.json"
    $sha256 "$prefix/include/oneapi/tbb/version.h" "$prefix/include/parlay/scheduler.h"
    cat "$rayon/metadata.txt"
} > "$prefix/metadata.txt"
printf 'compute-bench dependencies PASS: scalar shared oneTBB and pinned Parlay at %s\n' "$prefix"
