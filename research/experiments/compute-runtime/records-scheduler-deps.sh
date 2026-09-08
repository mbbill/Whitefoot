#!/bin/sh
set -eu
mode=${1:?fetch or build required}
case "$mode" in fetch|build) ;; *) exit 1;; esac
: "${OUT:?absolute output directory required}"
case "$OUT" in /*) ;; *) echo 'scheduler OUT must be absolute' >&2; exit 1;; esac
test "$OUT" != /
cache=${WHITEFOOT_SCRATCH_ROOT:-$HOME/do_not_scan}/whitefoot-compute-deps
tbb=${TBB_SOURCE:-$cache/onetbb-3046c8b0}
parlay=${PARLAY_SOURCE:-$cache/parlay-51017699}
tbb_pin=3046c8b0c29df995980003ea24f4d78c80ec0c8d
parlay_pin=51017699dcc421f80479cdb238d3092233ad0d26
verify() {
    test "$(git -C "$1" rev-parse HEAD)" = "$2"
    test -z "$(git -C "$1" status --porcelain --untracked-files=all)"
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
    fetch "$tbb" "$tbb_pin" https://github.com/uxlfoundation/oneTBB.git
    fetch "$parlay" "$parlay_pin" https://github.com/cmuparlay/parlaylib.git
    printf 'scheduler sources PASS: oneTBB=%s Parlay=%s\n' "$tbb_pin" "$parlay_pin"
    exit 0
fi
if test ! -d "$tbb/.git" || test ! -d "$parlay/.git"; then
    echo 'missing pinned schedulers: run make scheduler-fetch in this experiment first' >&2
    exit 1
fi
verify "$tbb" "$tbb_pin"
verify "$parlay" "$parlay_pin"
prefix=$OUT/scheduler-deps
build=$prefix/build
cmake=${CMAKE:-cmake}
cc=${CC:-/usr/bin/clang}
cxx=${CXX:-/usr/bin/clang++}
scalar='-fno-vectorize -fno-slp-vectorize -fno-lto'
mkdir -p "$prefix"
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
"$cmake" --build "$build" --target tbb --parallel 2
"$cmake" --install "$build" --config Release
"$cmake" -E copy_directory "$parlay/include/parlay" "$prefix/include/parlay"
diff -qr "$parlay/include/parlay" "$prefix/include/parlay"
diff -qr "$tbb/include/oneapi" "$prefix/include/oneapi"
diff -qr "$tbb/include/tbb" "$prefix/include/tbb"
case "$(uname -s)" in
    Darwin) library=$prefix/lib/libtbb.dylib;;
    Linux) library=$prefix/lib/libtbb.so;;
    *) echo 'scheduler dependency host is not qualified' >&2; exit 1;;
esac
test -f "$library"
{
    printf 'oneTBB=%s source=%s\nParlay=%s source=%s\n' "$tbb_pin" "$tbb" "$parlay_pin" "$parlay"
    printf 'library=%s\nC=%s\nCXX=%s\nscalar_flags=%s\n' "$library" "$cc" "$cxx" "$scalar"
    printf 'oneTBB: shared Release C++17 IPO=OFF hwloc=OFF malloc=OFF upstream_tests=OFF\n'
    "$cmake" --version
    "$cc" --version
    "$cxx" --version
    shasum -a 256 "$library" "$build/CMakeCache.txt" "$build/compile_commands.json"
    shasum -a 256 "$prefix/include/oneapi/tbb/version.h" "$prefix/include/parlay/scheduler.h"
} > "$prefix/metadata.txt"
printf 'scheduler dependencies PASS: scalar shared oneTBB and pinned Parlay at %s\n' "$prefix"
