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
        'interval=joined-dispatch first=lazy-start warm=separate post-timing-capacity=excluded' \
        'outputs=distinct-preallocated-per-call batch=all-calls-and-gaps final-verification=excluded'
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

# Each input retains the same seed across grains, cadences, widths and passes.
# Fields: shape, count, max length, records per callback, seed, cadence.
if test "$mode" = check; then
    while read -r shape count limit grain seed; do
        for cadence in checked dense sleep-100us sleep-1ms; do
            printf '%s %s %s %s %s %s\n' "$shape" "$count" "$limit" "$grain" "$seed" "$cadence"
        done
    done > "$results/cells.txt" <<'CELLS'
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
            for grain in 16 64; do
                cadences='checked dense'
                case "$shape $size $grain" in
                    'error-first 256 65536 16'|'error-first 4097 128 16'|'unicode 4097 128 16')
                        cadences="$cadences sleep-100us sleep-1ms";;
                esac
                for cadence in $cadences; do
                    printf '%s %s %s %s %s\n' "$shape" "$size" "$grain" "$seed" "$cadence"
                done
            done
        done
    done > "$results/cells.txt"
fi
printf 'shape\trecords\tmax_length\tseed\tbytes\n' > "$results/input-bytes.tsv"
printf 'backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\twarm_calls\tcall_mean_ns\tcall_min_ns\tcall_max_ns\tuser_mean_us\tsystem_mean_us\tcpu_mean_us\tvoluntary_switches_sum\tinvoluntary_switches_sum\tprocess_maxrss_bytes\tcadence\trequested_gap_ns\tgap_mean_ns\tgap_min_ns\tgap_max_ns\tbatch_ns\tbatch_user_us\tbatch_system_us\tbatch_cpu_us\tbatch_maxrss_bytes\tbatch_voluntary_switches\tbatch_involuntary_switches\tbatch_includes_checks\n' > "$results/summary.tsv"
cell=0
processes=0
previous_input=''
input_bytes=''
while read -r shape count limit grain seed cadence; do
    cell=$((cell+1))
    key="$shape-$count-$limit-$seed"
    if test "$key" != "$previous_input"; then previous_input=$key; input_bytes=''; fi
    chunks=$(((count+grain-1)/grain))
    reps=$((8000000/(count*limit+1)))
    if test "$reps" -lt 4; then reps=4; fi
    if test "$reps" -gt 32; then reps=32; fi
    if test "$mode" = check; then reps=2; fi
    checks=0
    case "$cadence" in
        checked) requested_gap=0; checks=1;;
        dense) requested_gap=0;;
        sleep-100us) requested_gap=100000;;
        sleep-1ms) requested_gap=1000000;;
        *) exit 1;;
    esac
    pass=0
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((pass+cell))" -v reverse="$((pass%2))" '
            { a[NR]=$0 } END { for(j=0;j<NR;j++) { i=(j+shift)%NR; if(reverse) i=NR-1-i; print a[i+1] } }
        ' "$results/configurations.txt" > "$results/order.txt"
        while read -r scheduler backend width shutdown; do
            file="$results/$shape-n$count-l$limit-g$grain-$cadence-$scheduler-w$width-p$pass.tsv"
            WF_WORKERS="$width" "$OUT/scheduler-$scheduler" "$width" "$count" "$limit" "$shape" \
                "$grain" "$reps" "$seed" "$pass" "$cadence" > "$file" 2>&1
            awk -F '\t' -v backend="$backend" -v width="$width" -v shape="$shape" -v count="$count" \
                -v limit="$limit" -v grain="$grain" -v chunks="$chunks" -v seed="$seed" -v pass="$pass" \
                -v reps="$reps" -v shutdown="$shutdown" -v expected_bytes="$input_bytes" \
                -v cadence="$cadence" -v requested_gap="$requested_gap" -v checks="$checks" '
                function fail() { bad=1; exit 1 }
                /^# backend=/ {
                    if (NR!=1 || meta++ || split($0,p," ")!=15 || p[1]!="#" || p[2]!="backend=" backend ||
                        p[3]!="width=" width || p[4]!="shape=" shape || p[5]!="records=" count ||
                        p[6]!~/^bytes=[0-9]+$/ || p[7]!="max_length=" limit || p[8]!="grain=" grain ||
                        p[9]!="chunks=" chunks || p[10]!="seed=" seed || p[11]!="pass=" pass ||
                        p[12]!="reps=" reps || p[13]!~/^clock_pair_min_ns=[0-9]+$/ ||
                        p[14]!="cadence=" cadence || p[15]!="requested_gap_ns=" requested_gap) fail();
                    split(p[6],pair,"="); bytes=pair[2];
                    if (length(expected_bytes) && bytes!=expected_bytes) fail();
                    next
                }
                /^backend\t/ {
                    if (NR!=2 || meta!=1 || header++ ||
                        $0!="backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\tcall\tphase\tcall_ns\tuser_us\tsystem_us\tmaxrss_bytes\tvoluntary_switches\tinvoluntary_switches\tcadence\tgap_ns") fail();
                    next
                }
                /^# batch_includes_first=/ {
                    if (header!=1 || seen!=reps+1 || batch++ || capacity || passed || split($0,p," ")!=9 ||
                        p[1]!="#" || p[2]!="batch_includes_first=1" || p[3]!="batch_includes_checks=" checks ||
                        p[4]!~/^batch_ns=[0-9]+$/ || p[5]!~/^user_us=[0-9]+$/ || p[6]!~/^system_us=[0-9]+$/ ||
                        p[7]!~/^maxrss_bytes=[0-9]+$/ || p[8]!~/^voluntary_switches=[0-9]+$/ ||
                        p[9]!~/^involuntary_switches=[0-9]+$/) fail();
                    for(i=4;i<=9;i++) { split(p[i],pair,"="); totals[i]=pair[2]+0; }
                    if (all[13]+gap_sum>totals[4] || all[14]>totals[5] || all[15]>totals[6] ||
                        all_rss>totals[7] || all[17]>totals[8] || all[18]>totals[9]) fail();
                    next
                }
                /^# post_timing_capacity=/ {
                    if (batch!=1 || seen!=reps+1 || capacity++ || passed || split($0,p," ")!=6 ||
                        p[1]!="#" || p[2]!="post_timing_capacity=" width || p[3]!="includes_caller=1" ||
                        p[4]!~/^capacity_waves=[1-3]$/ || p[5]!="explicit_shutdown=" shutdown || p[6]!~/^stop_ns=[0-9]+$/) fail();
                    next
                }
                /^record scheduler benchmark PASS:/ {
                    if (capacity!=1 || passed++ || $0!="record scheduler benchmark PASS: calls=" (reps+1) " outputs=" ((reps+1)*count)) fail();
                    next
                }
                {
                    if (header!=1 || batch || capacity || passed || NF!=20 || $1!=backend || $2!=width || $3!=shape ||
                        $4!=count || $5!=bytes || $6!=limit || $7!=grain || $8!=chunks || $9!=seed || $10!=pass ||
                        $11!=seen || $12!=(seen?"warm":"first") || $19!=cadence ||
                        $20!~/^[0-9]+$/ || (!seen && $20!=0)) fail();
                    for(i=13;i<=18;i++) if($i!~/^[0-9]+$/) fail();
                    for(i=13;i<=18;i++) all[i]+=$i;
                    if(!seen || $16>all_rss)all_rss=$16;
                    gap_sum+=$20;
                    if (seen) {
                        for(i=13;i<=18;i++) sum[i]+=$i;
                        if(seen==1 || $13<minimum)minimum=$13;
                        if(seen==1 || $13>maximum)maximum=$13;
                        if(seen==1 || $16>rss)rss=$16;
                        if(seen==1 || $20<gap_minimum)gap_minimum=$20;
                        if(seen==1 || $20>gap_maximum)gap_maximum=$20;
                    }
                    seen++;
                }
                END {
                    if (bad || meta!=1 || header!=1 || batch!=1 || capacity!=1 || passed!=1 || seen!=reps+1) exit 1;
                    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%d",backend,width,shape,count,bytes,limit,grain,chunks,seed,pass,reps;
                    printf "\t%.3f\t%.0f\t%.0f\t%.3f\t%.3f\t%.3f\t%.0f\t%.0f\t%.0f",sum[13]/reps,minimum,maximum,sum[14]/reps,sum[15]/reps,(sum[14]+sum[15])/reps,sum[17],sum[18],rss;
                    printf "\t%s\t%.0f\t%.3f\t%.0f\t%.0f",cadence,requested_gap,gap_sum/reps,gap_minimum,gap_maximum;
                    printf "\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%d\n",totals[4],totals[5],totals[6],totals[5]+totals[6],totals[7],totals[8],totals[9],checks;
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
