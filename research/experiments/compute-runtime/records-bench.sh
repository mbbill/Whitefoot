#!/bin/sh
set -eu
cd "$(dirname "$0")"
mode=${1:?mode required}
case "$mode" in check|calibrate) ;; *) echo 'unknown records mode' >&2; exit 1;; esac
: "${OUT:?build output required}"
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
if test "$mode" = check; then rounds=1; fi
results=${RESULTS:-$OUT/records-$mode-$$}
test ! -e "$results"
mkdir -p "$results"
printf '%s\n' "$results" > "$OUT/last-records-$mode-path.txt"
{
    printf 'mode=%s rounds=%s\n' "$mode" "$rounds"
    git rev-parse HEAD
    uname -a
    cat "$OUT/records-flags.txt"
    if test "$(uname -s)" = Linux; then
        lscpu
        cat /proc/self/status
        for path in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; else printf '%s: unqualified\n' "$path"; fi
        done
    else
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    fi
    shasum -a 256 records.wf records_host.ll records.c records_oracle.c records_native.c records_native.h records-bench.sh Makefile \
        runtime.c runtime.h ../../../compiler/src/backend/wf_floor.c
    shasum -a 256 "$OUT/records-host.ll" "$OUT/records-wf.o" "$OUT/records-native.o" \
        "$OUT/records-weak" "$OUT/records-recovered"
} > "$results/manifest.txt"
git diff --binary HEAD > "$results/source.patch"
for form in weak recovered sanitized; do cp "$OUT/records-$form-check.log" "$results/"; done
cat > "$results/configurations.txt" <<'CONFIG'
weak state 0
weak word 0
weak wf 0
recovered wf 0
recovered wf 2
recovered wf 4
CONFIG
if test "$mode" = check; then
    cat > "$results/cells.txt" <<'CELLS'
ascii 0 0
unicode 33 17
skew 4097 64
CELLS
else
    for shape in ascii unicode error-first error-last skew; do
        for size in '1 32' '33 64' '4097 128' '65536 16'; do
            printf '%s %s\n' "$shape" "$size"
        done
    done > "$results/cells.txt"
fi
printf 'runtime\tkernel\tworkers\tshape\trecords\tbytes\tmax_length\tseed\tpass\tcalls\tcore_mean_ns\tcore_min_ns\tcore_max_ns\tcycle_mean_ns\tcycle_min_ns\tcycle_max_ns\n' > "$results/summary.tsv"
cell=0
processes=0
while read -r shape count limit; do
    cell=$((cell+1))
    seed=$((812381+cell*7919))
    reps=$((2000000/(count*limit+1)))
    if test "$reps" -lt 4; then reps=4; fi
    if test "$reps" -gt 64; then reps=64; fi
    if test "$mode" = check; then reps=2; fi
    pass=0
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((pass+cell))" -v reverse="$((pass%2))" '
            { a[NR]=$0 } END { for(j=0;j<NR;j++) { i=(j+shift)%NR; if(reverse) i=NR-1-i; print a[i+1] } }
        ' "$results/configurations.txt" > "$results/order.txt"
        while read -r runtime kernel workers; do
            file="$results/$shape-n$count-l$limit-$runtime-$kernel-w$workers-p$pass.tsv"
            WF_WORKERS="$workers" "$OUT/records-$runtime" "$kernel" "$count" "$limit" "$shape" "$reps" "$seed" "$pass" > "$file" 2>&1
            grep -Fxq "# UTF8 records PASS: calls=$((reps+1)) results=$(((reps+1)*count))" "$file"
            test "$(grep -c '^# UTF8 records PASS:' "$file")" -eq 1
            tail -n 1 "$file" | grep -Fxq "# UTF8 records PASS: calls=$((reps+1)) results=$(((reps+1)*count))"
            test "$(grep -c '^# runtime=' "$file")" -eq 1
            test "$(grep -c '^# batch_includes_checks=' "$file")" -eq 1
            world=sequential
            if test "$kernel" = wf && test "$workers" -ge 2; then world=parallel; fi
            grep -Eq "^# runtime=$runtime kernel=$kernel workers=$workers world=$world shape=$shape records=$count bytes=[0-9]+ max_length=$limit seed=$seed pass=$pass reps=$reps entry_ns=[0-9]+ clock_pair_min_ns=[0-9]+$" "$file"
            if test "$runtime" = recovered; then
                actual=0; if test "$workers" -ge 2; then actual="(0|$workers)"; fi
                grep -Eq "^# actual_lanes=$actual steals=[0-9]+$" "$file"
                test "$(grep -c '^# actual_lanes=' "$file")" -eq 1
                if grep -q '^# actual_lanes=0 ' "$file"; then grep -Fxq '# actual_lanes=0 steals=0' "$file"; fi
            fi
            awk -F '\t' -v runtime="$runtime" -v kernel="$kernel" -v workers="$workers" \
                -v shape="$shape" -v count="$count" -v limit="$limit" -v seed="$seed" -v pass="$pass" -v reps="$reps" '
                function fail() { bad=1; exit 1 }
                /^# runtime=/ { split($0,parts," "); split(parts[8],pair,"="); declared_bytes=pair[2]; next }
                /^# batch_includes_checks=/ { split($0,parts," "); split(parts[3],pair,"="); batch_ns=pair[2]; next }
                /^# (actual_lanes=|UTF8 records PASS:)/ { next }
                /^#/ { fail() }
                /^runtime\t/ {
                    if($0!="runtime\tkernel\tworkers\tshape\trecords\tbytes\tmax_length\tseed\tpass\tcall\tphase\tcore_ns\tcycle_ns" || header++) fail();
                    next
                }
                {
                    if(!header || NF!=13 || $1!=runtime || $2!=kernel || $3!=workers || $4!=shape || $5!=count ||
                       $7!=limit || $8!=seed || $9!=pass || $10!=seen || $11!=(seen?"warm":"first") ||
                       $6!~/^[0-9]+$/ || $12!~/^[0-9]+$/ || $13!~/^[0-9]+$/ || $12+0>$13+0) fail();
                    if($6!=declared_bytes || (seen && $6!=bytes)) fail(); bytes=$6; total_cycle+=$13;
                    if(seen) { for(i=12;i<=13;i++) {v=$i+0;sum[i]+=v;if(seen==1 || v<min[i])min[i]=v;if(seen==1 || v>max[i])max[i]=v;} }
                    seen++;
                }
                END {
                    if(bad || header!=1 || seen!=reps+1 || batch_ns+0<total_cycle) exit 1;
                    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%d",runtime,kernel,workers,shape,count,bytes,limit,seed,pass,reps;
                    for(i=12;i<=13;i++) printf "\t%.3f\t%.0f\t%.0f",sum[i]/reps,min[i],max[i];
                    printf "\n";
                }
            ' "$file" >> "$results/summary.tsv"
            grep -Eq '^# batch_includes_checks=1 batch_ns=[0-9]+ user_us=[0-9]+ system_us=[0-9]+ maxrss_bytes=[0-9]+ voluntary_switches=[0-9]+ involuntary_switches=[0-9]+$' "$file"
            processes=$((processes+1))
        done < "$results/order.txt"
        pass=$((pass+1))
    done
done < "$results/cells.txt"
expected=$((cell*6*rounds))
test "$processes" -eq "$expected"
test "$(wc -l < "$results/summary.tsv")" -eq "$((expected+1))"
printf 'UTF8 records %s PASS: processes=%s results=%s\n' "$mode" "$processes" "$results"
