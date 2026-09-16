#!/bin/sh
# Crafted tables exercise the actual reducer/verdict, without compiling or
# timing WF. The original fourteen decisions remain, with matrix and precision
# controls for defects found during extraction. Retire with this instrument.
set -eu
here=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
work=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-performance-test.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
cases=0
cpus=4
matrix() {
    awk 'BEGIN {split("mandelbrot records fir quadrature stencil",k);for(i=1;i<=5;i++)for(w=1;w<=4;w*=2)print k[i],w,1,0,5,1}' > "$work/table"
}
row() {
    awk -v k="$1" -v w="$2" -v r="$3" -v n="$4" -v c="$5" \
        '{if($1==k && $2==w){$3=r;$4=n;$6=c}print}' "$work/table" > "$work/new"
    mv "$work/new" "$work/table"
}
expect() {
    name=$1; want=$2; pattern=$3; program=${4:-verdict}
    set +e
    awk -v cpus="$cpus" -f "$here/$program.awk" "$work/table" > "$work/out" 2>&1
    status=$?
    set -e
    if [ "$status" -ne "$want" ] || ! grep -F -q "$pattern" "$work/out"; then
        echo "FAIL $name: status=$status expected=$want pattern=$pattern" >&2
        cat "$work/out" >&2
        exit 1
    fi
    cases=$((cases + 1))
    echo "PASS $name"
}
matrix
row fir 4 0.96 2 0.99
expect pass 0 'VERDICT: PASS'
matrix; row fir 2 0.94 4 0.995; row fir 4 0.948 5 0.991
expect two-block-fail 1 'VERDICT: FAIL'
matrix; row records 1 0.938 5 0.996
expect one-block-suspect 0 'suspect: records W=1'
matrix; row fir 1 0.94 4 0.995; row records 1 0.951 4 0.998
expect two-kernels-one-block-each 0 '2 suspect(s)'
matrix; row fir 2 0.9 2 1
expect wall-band-without-adverse-pairs 0 'VERDICT: PASS'
matrix; row fir 2 0.985 4 1
expect adverse-pairs-inside-band 0 'VERDICT: PASS'
printf '# no baseline rows\n' > "$work/table"
expect no-twin 2 'REFUSED: missing'
matrix; row fir 4 0.5 5 1; cpus=2
expect oversubscribed-excluded 0 'VERDICT: PASS'
cpus=4; matrix; printf 'fir 8 0.5 5 5 1\n' >> "$work/table"
expect unrecorded-width-ignored 0 'skipped 1 unrecorded/oversubscribed'
printf 'fir 8 0.5 5 5 1\n' > "$work/table"
expect only-unrecorded-widths 2 'REFUSED: missing'
matrix; sed 's/fir 2 1 0 5 1/fir 2 1 0 2 1/' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect under-powered 2 'underpowered paired row'
matrix; row fir 2 1 4 0.5
expect cpu-reported-never-failing 0 'cpu report: fir W=2'
matrix; awk '$2==1' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect one-readable-block-per-kernel 2 'REFUSED: missing mandelbrot W=2'
: > "$work/table"
expect empty-table 2 'REFUSED: missing'

matrix; awk '$1!="stencil"' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect missing-whole-kernel 2 'REFUSED: missing stencil'
matrix; head -1 "$work/table" >> "$work/table"
expect duplicate-width 2 'REFUSED: duplicate kernel/width'
matrix; row fir 1 0.96999999 4 1; row fir 2 0.96999999 4 1
expect unrounded-below-threshold 1 'VERDICT: FAIL'
matrix; row fir 1 0.97000001 4 1; row fir 2 0.97000001 4 1
expect unrounded-above-threshold 0 'VERDICT: PASS'
matrix; cpus=1
expect insufficient-host 2 'at least two available CPUs'
cpus=4

raw() {
    awk 'BEGIN {
        split("mandelbrot records fir quadrature stencil",k)
        for(i=1;i<=5;i++)for(w=1;w<=4;w*=2)for(a=0;a<2;a++)for(p=0;p<5;p++)for(s=0;s<=5;s++) {
            t=(a && p<4 ? 1100 : 1000)+s
            if(s==0)t=9999999
            print k[i],a?"candidate":"baseline",w,p,s,t,t,17
        }
    }' > "$work/table"
}
raw
expect reduce-paired-process-medians 0 'mandelbrot' reduce
cp "$work/out" "$work/paired"
# Four slower candidate passes must survive both medians; warmup is ignored.
awk '$1=="mandelbrot" && $2==1 {if($3>0.911 || $3<0.909 || $4!=4 || $5!=5)exit 1;found=1}END{if(!found)exit 1}' "$work/paired"
raw; awk '$1!="stencil"' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect reduce-missing-kernel 2 'REFUSED: missing stencil' reduce
raw; awk '$2!="baseline"' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect reduce-missing-arm 2 'REFUSED: missing mandelbrot baseline' reduce
raw; awk '!($1=="fir" && $2=="candidate" && $3==4 && $4==2 && $5==3)' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect reduce-missing-sample 2 'REFUSED: missing fir candidate' reduce
raw; head -1 "$work/table" >> "$work/table"
expect reduce-duplicate-sample 2 'REFUSED: duplicate raw row' reduce
raw; awk 'NR==1{$8=18} {print}' "$work/table" > "$work/new"; mv "$work/new" "$work/table"
expect reduce-output-extent 2 'REFUSED: inconsistent output extent' reduce
printf 'broken\n' > "$work/table"
expect reduce-malformed 2 'REFUSED: invalid raw row' reduce
# Match compare.sh's sequential status propagation: partial reducer output
# must never allow the verdict or a later tee/cat to replace the failure.
if sh -ec 'awk -v cpus=4 -f "$1/reduce.awk" "$2/table" > "$2/paired"; touch "$2/verdict-ran"' sh "$here" "$work" 2>/dev/null; then
    echo 'FAIL reducer error was swallowed' >&2; exit 1
fi
test ! -e "$work/verdict-ran"
cases=$((cases + 1))
echo "performance instrument: $cases cases PASS"
