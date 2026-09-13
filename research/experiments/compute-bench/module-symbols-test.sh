#!/bin/sh
# Serves compute-bench's module isolation. Keep with module-symbols.awk; this
# gate test links both emissions and checks calls, imports and weak overrides.
set -eu
work=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-module-symbols.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
cc=${CC:-/usr/bin/clang}
cat > "$work/input.ll" <<'IR'
@text = private constant [10 x i8] c"@helper\22\5C\00"
declare i32 @imported()
define weak i32 @runtime() {
  ret i32 1000
}
define i32 @helper() {
  %x = call i32 @imported()
  ret i32 %x
}
define internal i32 @helper_extra() {
  %x = call i32 @"helper"()
  %y = call i32 @runtime()
  %r = add i32 %x, %y
  ret i32 %r
}
define i32 @wf_bench_test() {
  ; @helper is a comment, and @text's contents are not symbol references.
  %x = call i32 @helper_extra()
  ret i32 %x
}
define void @wf_bench_test_release() {
  ret void
}
IR
for mode in par seq; do
    awk -v prefix="test_$mode" -v api=wf_bench_test -v mode="$mode" \
        -f module-symbols.awk "$work/input.ll" > "$work/$mode.ll"
    # Source linkage must survive: internalizing a strong helper changes the
    # optimization experiment even if the resulting executable returns 42.
    grep -q "^define i32 @\"test_${mode}_helper\"()" "$work/$mode.ll"
    grep -q '^define weak i32 @runtime()' "$work/$mode.ll"
    grep -Fq '@text = private constant [10 x i8] c"@helper\22\5C\00"' "$work/$mode.ll"
    grep -Fq '; @helper is a comment' "$work/$mode.ll"
    "$cc" -O2 -Wno-override-module -c "$work/$mode.ll" -o "$work/$mode.o"
done
cat > "$work/caller.c" <<'C'
int wf_bench_test_par(void);
int wf_bench_test_seq(void);
void wf_bench_test_par_release(void);
void wf_bench_test_seq_release(void);
int imported(void) { return 40; }
int runtime(void) { return 2; }
int main(void) {
    wf_bench_test_par_release();
    wf_bench_test_seq_release();
    return wf_bench_test_par() == 42 && wf_bench_test_seq() == 42 ? 0 : 1;
}
C
"$cc" -std=c11 -O2 "$work/caller.c" "$work/par.o" "$work/seq.o" -o "$work/check"
"$work/check"
# A missing adapter cannot silently emit a module whose harness calls a stale
# symbol. This input is the original module with the adapter removed.
sed '/^define i32 @wf_bench_test()/,$d' "$work/input.ll" > "$work/missing.ll"
if awk -v prefix=test_par -v api=wf_bench_test -v mode=par \
    -f module-symbols.awk "$work/missing.ll" > "$work/missing.out" 2> "$work/missing.err"; then
    echo 'module-symbols-test: missing adapter was accepted' >&2
    exit 1
fi
grep -q 'missing adapter' "$work/missing.err"
echo 'module-symbols-test: PASS (paired calls, linkage, imports, weak override, missing adapter)'
