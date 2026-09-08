#!/bin/sh
set -eu
cd "$(dirname "$0")"
mode=${1:?mode required}
case "$mode" in check|calibrate) ;; *) echo 'unknown scheduler mode' >&2; exit 1;; esac
: "${OUT:?build output required}"
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
if test "$mode" = check; then rounds=1; fi
results=${RESULTS:-$OUT/scheduler-$mode-$$}
test ! -e "$results"
test -s "$OUT/scheduler-flags.txt"
test -s "$OUT/scheduler-deps/metadata.txt"
for unit in host native oracle work floor runtime wf-adapter static-adapter tbb-adapter parlay-adapter; do
    test -s "$OUT/scheduler-$unit.o"
done
for unit in native work; do test -s "$OUT/scheduler-$unit.s"; done
for scheduler in wf static tbb parlay; do test -x "$OUT/scheduler-$scheduler"; done
# Both common C computation and C++ dispatch are ordinary scalar release builds.
awk '
    /^C: / || /^C\+\+: / {
        if ($0 !~ /(^| )-fno-vectorize( |$)/ || $0 !~ /(^| )-fno-slp-vectorize( |$)/ ||
            $0 !~ /(^| )-fno-lto( |$)/ || $0 !~ /(^| )-DNDEBUG( |$)/) bad=1;
        if ($1=="C:") c++; else cpp++;
    }
    END { if (bad || c!=1 || cpp!=1) exit 1 }
' "$OUT/scheduler-flags.txt"
mkdir -p "$results"
printf '%s\n' "$results" > "$OUT/last-scheduler-$mode-path.txt"
{
    printf 'mode=%s rounds=%s kernel=scalar-records-state\n' "$mode" "$rounds"
    printf '%s\n' 'common_leaf=scheduler-native.o common_callback=scheduler-work.o' \
        'measured_executables=scheduler-wf,scheduler-static,scheduler-tbb,scheduler-parlay' \
        'interval=joined-dispatch first=lazy-start warm=separate post-timing-capacity=excluded'
    git rev-parse HEAD
    git status --short
    uname -a
    cat "$OUT/scheduler-flags.txt"
    cat "$OUT/scheduler-deps/metadata.txt"
    if test "$(uname -s)" = Linux; then
        lscpu
        cat /proc/self/status
        for path in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; else printf '%s: unqualified\n' "$path"; fi
        done
    else
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    fi
    shasum -a 256 records_scheduler.cpp records_scheduler.h records_work.c records_native.c records_native.h \
        records_oracle.c records_runtime.c records_static.c records_tbb.cpp records_parlay.cpp \
        runtime.c runtime.h records-scheduler-bench.sh records-scheduler-deps.sh Makefile \
        records.c records.wf records_host.ll ../../../compiler/src/backend/wf_floor.c
    shasum -a 256 "$OUT"/scheduler-*.o "$OUT"/scheduler-*.s "$OUT/scheduler-flags.txt" \
        "$OUT/scheduler-wf" "$OUT/scheduler-static" "$OUT/scheduler-tbb" "$OUT/scheduler-parlay"
    find "$OUT/scheduler-deps" -type f -exec shasum -a 256 {} +
} > "$results/manifest.txt"
git diff --binary HEAD > "$results/source.patch"
while read -r scheduler backend shutdown; do
    for width in 1 2 4; do printf '%s %s %s %s\n' "$scheduler" "$backend" "$width" "$shutdown"; done
done > "$results/configurations.txt" <<'CONFIG'
wf wf-runtime 0
static static-spin 1
tbb oneTBB-v2023.1.0-auto-grain1 0
parlay parlay-native-grain1 1
CONFIG
while read -r scheduler backend width shutdown; do
    qualifier="$OUT/scheduler-$scheduler-check-w$width.log"
    awk -v backend="$backend" -v width="$width" -v shutdown="$shutdown" '
        { if (NR!=1 || $8 !~ /^capacity_waves=[1-3]$/ ||
            $0 != "record scheduler qualification PASS: backend=" backend " width=" width " actual=" width " " $8 \
                  " batches=780 outputs=270660 explicit_shutdown=" shutdown) bad=1 }
        END { if (bad || NR!=1) exit 1 }
    ' "$qualifier"
    cp "$qualifier" "$results/"
done < "$results/configurations.txt"
awk '
    NR==1 { if ($0!="UTF8 leaf qualification PASS: calls=4595603 all-codepoints all-truncations two-byte-pairs") bad=1 }
    NR==2 { if ($0!="UTF8 batch qualification PASS: world=sequential calls=342 results=1821070") bad=1 }
    END { if (bad || NR!=2) exit 1 }
' "$OUT/scheduler-leaf-check.log"
cp "$OUT/scheduler-leaf-check.log" "$results/"

# Each input retains the same seed across grains, widths, backends and passes.
# The fifth field is the seed; the fourth is records per common C callback.
if test "$mode" = check; then
    cat > "$results/cells.txt" <<'CELLS'
ascii 0 0 1 820300
unicode 33 17 16 828219
skew 257 64 64 836138
CELLS
else
    input=0
    for shape in ascii unicode error-first error-last skew; do
        for size in '256 65536' '4097 128' '65536 16'; do
            input=$((input+1))
            seed=$((812381+input*7919))
            for grain in 16 64; do printf '%s %s %s %s\n' "$shape" "$size" "$grain" "$seed"; done
        done
    done > "$results/cells.txt"
fi
printf 'shape\trecords\tmax_length\tseed\tbytes\n' > "$results/input-bytes.tsv"
printf 'backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\twarm_calls\tcall_mean_ns\tcall_min_ns\tcall_max_ns\tuser_mean_us\tsystem_mean_us\tcpu_mean_us\tvoluntary_switches_sum\tinvoluntary_switches_sum\tprocess_maxrss_bytes\n' > "$results/summary.tsv"
cell=0
processes=0
previous_input=''
input_bytes=''
while read -r shape count limit grain seed; do
    cell=$((cell+1))
    key="$shape-$count-$limit-$seed"
    if test "$key" != "$previous_input"; then previous_input=$key; input_bytes=''; fi
    chunks=$(((count+grain-1)/grain))
    reps=$((8000000/(count*limit+1)))
    if test "$reps" -lt 4; then reps=4; fi
    if test "$reps" -gt 32; then reps=32; fi
    if test "$mode" = check; then reps=2; fi
    pass=0
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((pass+cell))" -v reverse="$((pass%2))" '
            { a[NR]=$0 } END { for(j=0;j<NR;j++) { i=(j+shift)%NR; if(reverse) i=NR-1-i; print a[i+1] } }
        ' "$results/configurations.txt" > "$results/order.txt"
        while read -r scheduler backend width shutdown; do
            file="$results/$shape-n$count-l$limit-g$grain-$scheduler-w$width-p$pass.tsv"
            WF_WORKERS="$width" "$OUT/scheduler-$scheduler" "$width" "$count" "$limit" "$shape" \
                "$grain" "$reps" "$seed" "$pass" > "$file" 2>&1
            awk -F '\t' -v backend="$backend" -v width="$width" -v shape="$shape" -v count="$count" \
                -v limit="$limit" -v grain="$grain" -v chunks="$chunks" -v seed="$seed" -v pass="$pass" \
                -v reps="$reps" -v shutdown="$shutdown" -v expected_bytes="$input_bytes" '
                function fail() { bad=1; exit 1 }
                /^# backend=/ {
                    if (NR!=1 || meta++ || split($0,p," ")!=13 || p[1]!="#" || p[2]!="backend=" backend ||
                        p[3]!="width=" width || p[4]!="shape=" shape || p[5]!="records=" count ||
                        p[6]!~/^bytes=[0-9]+$/ || p[7]!="max_length=" limit || p[8]!="grain=" grain ||
                        p[9]!="chunks=" chunks || p[10]!="seed=" seed || p[11]!="pass=" pass ||
                        p[12]!="reps=" reps || p[13]!~/^clock_pair_min_ns=[0-9]+$/) fail();
                    split(p[6],pair,"="); bytes=pair[2];
                    if (length(expected_bytes) && bytes!=expected_bytes) fail();
                    next
                }
                /^backend\t/ {
                    if (NR!=2 || meta!=1 || header++ ||
                        $0!="backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\tcall\tphase\tcall_ns\tuser_us\tsystem_us\tmaxrss_bytes\tvoluntary_switches\tinvoluntary_switches") fail();
                    next
                }
                /^# post_timing_capacity=/ {
                    if (header!=1 || seen!=reps+1 || capacity++ || passed || split($0,p," ")!=6 ||
                        p[1]!="#" || p[2]!="post_timing_capacity=" width || p[3]!="includes_caller=1" ||
                        p[4]!~/^capacity_waves=[1-3]$/ || p[5]!="explicit_shutdown=" shutdown || p[6]!~/^stop_ns=[0-9]+$/) fail();
                    next
                }
                /^record scheduler benchmark PASS:/ {
                    if (capacity!=1 || passed++ || $0!="record scheduler benchmark PASS: calls=" (reps+1) " outputs=" ((reps+1)*count)) fail();
                    next
                }
                {
                    if (header!=1 || capacity || passed || NF!=18 || $1!=backend || $2!=width || $3!=shape ||
                        $4!=count || $5!=bytes || $6!=limit || $7!=grain || $8!=chunks || $9!=seed || $10!=pass ||
                        $11!=seen || $12!=(seen?"warm":"first")) fail();
                    for(i=13;i<=18;i++) if($i!~/^[0-9]+$/) fail();
                    if (seen) {
                        for(i=13;i<=18;i++) sum[i]+=$i;
                        if(seen==1 || $13<minimum)minimum=$13;
                        if(seen==1 || $13>maximum)maximum=$13;
                        if(seen==1 || $16>rss)rss=$16;
                    }
                    seen++;
                }
                END {
                    if (bad || meta!=1 || header!=1 || capacity!=1 || passed!=1 || seen!=reps+1) exit 1;
                    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%d",backend,width,shape,count,bytes,limit,grain,chunks,seed,pass,reps;
                    printf "\t%.3f\t%.0f\t%.0f\t%.3f\t%.3f\t%.3f\t%.0f\t%.0f\t%.0f\n",sum[13]/reps,minimum,maximum,sum[14]/reps,sum[15]/reps,(sum[14]+sum[15])/reps,sum[17],sum[18],rss;
                }
            ' "$file" >> "$results/summary.tsv"
            if test -z "$input_bytes"; then
                input_bytes=$(awk -F '\t' 'NR==3 { print $5 }' "$file")
                printf '%s\t%s\t%s\t%s\t%s\n' "$shape" "$count" "$limit" "$seed" "$input_bytes" >> "$results/input-bytes.tsv"
            fi
            processes=$((processes+1))
        done < "$results/order.txt"
        pass=$((pass+1))
    done
done < "$results/cells.txt"
expected=$((cell*12*rounds))
test "$processes" -eq "$expected"
test "$(wc -l < "$results/summary.tsv")" -eq "$((expected+1))"
printf 'record scheduler %s PASS: processes=%s results=%s\n' "$mode" "$processes" "$results"
