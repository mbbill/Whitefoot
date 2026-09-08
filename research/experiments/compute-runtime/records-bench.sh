#!/bin/sh
set -eu
cd "$(dirname "$0")"
requested=${1:?mode required}
budget=0
case "$requested" in
    check|calibrate) mode=$requested;;
    budget-check) mode=check; budget=1;;
    budget-calibrate) mode=calibrate; budget=1;;
    *) echo 'unknown records mode' >&2; exit 1;;
esac
: "${OUT:?build output required}"
# The real WF and native anchors are a scalar, non-LTO cohort.
for flag in -fno-vectorize -fno-slp-vectorize -fno-lto; do
    head -n 1 "$OUT/records-flags.txt" | grep -Eq "(^| )$flag( |$)"
done
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
if test "$mode" = check; then rounds=1; fi
results=${RESULTS:-$OUT/records-$requested-$$}
test ! -e "$results"
if test "$budget" = 1; then
    grep -Fxq 'budget=cost,capacity,team; cadence=call,batch; same executable and emitted WF object; controls selected before floor entry' "$OUT/records-budget-flags.txt"
    budget_hash=$(shasum -a 256 "$OUT/records-budget")
    # This fixture has one map split. Derive its first affordable span from
    # the emitted weight, so a changed estimator does not inherit a stale
    # numeric threshold. The first call has no pending owner work.
    work_per_chunk=$(awk '$1=="#define" && $2=="WF_PAR_SPLIT_WORK_PER_CHUNK" {print $3; seen++} END {if(seen!=1)exit 1}' runtime.c)
    case "$work_per_chunk" in ''|*[!0-9]*) exit 1;; esac
    test "$work_per_chunk" -gt 0 && test "$work_per_chunk" -le 1000000000
    cost_minimum=$(awk -v work="$work_per_chunk" '
        /call i64 @wf__par_split_budget\(/ {
            sub(/^.* i64 /, ""); sub(/\).*$/, "")
            if($0 !~ /^[1-9][0-9]*$/)exit 1
            weight=$0+0; seen++
        }
        END {if(seen!=1 || weight<=0)exit 1; print 2*(weight>=work ? 1 : int((work+weight-1)/weight))}
    ' "$OUT/records-host.ll")
fi
mkdir -p "$results"
printf '%s\n' "$results" > "$OUT/last-records-$requested-path.txt"
{
    printf 'mode=%s rounds=%s\n' "$requested" "$rounds"
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
    if test "$budget" = 1; then
        cat "$OUT/records-budget-flags.txt"
        printf 'cost_first_parallel_span=%s\n' "$cost_minimum"
        printf '%s\n' "$budget_hash"
        shasum -a 256 "$OUT/records-budget-sanitized"
    fi
} > "$results/manifest.txt"
git diff --binary HEAD > "$results/source.patch"
for form in weak recovered sanitized; do cp "$OUT/records-$form-check.log" "$results/"; done
if test "$budget" = 1; then
    for image in budget budget-sanitized; do for policy in cost capacity team; do for cadence in call batch; do
        cp "$OUT/records-$image-$policy-$cadence-check.log" "$results/"
        cp "$OUT/records-$image-$policy-$cadence-boundary.log" "$results/"
    done; done; done
    for policy in cost capacity team; do for workers in 0 2 4; do for cadence in call batch; do
        printf 'budget wf %s %s %s\n' "$workers" "$policy" "$cadence"
    done; done; done > "$results/configurations.txt"
else
cat > "$results/configurations.txt" <<'CONFIG'
weak state 0
weak word 0
weak wf 0
recovered wf 0
recovered wf 2
recovered wf 4
CONFIG
fi
if test "$mode" = check; then
    cat > "$results/cells.txt" <<'CELLS'
ascii 0 0
unicode 33 17
skew 4097 64
unicode 256 65536
error-first 256 65536
CELLS
elif test "$budget" = 1; then
    for shape in ascii unicode error-first error-last skew; do
        printf '%s 256 65536\n%s 4097 128\n' "$shape" "$shape"
    done > "$results/cells.txt"
    printf '%s\n' 'ascii 33 64' 'error-first 33 64' 'ascii 0 0' >> "$results/cells.txt"
else
    # Keep existing cell ordinals/seeds when adding the long-record cases.
    for shape in ascii unicode error-first error-last skew; do
        for size in '1 32' '33 64' '4097 128' '65536 16'; do
            printf '%s %s\n' "$shape" "$size"
        done
    done > "$results/cells.txt"
    for shape in ascii unicode error-first error-last skew; do
        printf '%s 256 65536\n' "$shape"
    done >> "$results/cells.txt"
fi
printf 'runtime\tkernel\tworkers\tshape\trecords\tbytes\tmax_length\tseed\tpass\tcalls\tcore_mean_ns\tcore_min_ns\tcore_max_ns\tcycle_mean_ns\tcycle_min_ns\tcycle_max_ns' > "$results/summary.tsv"
if test "$budget" = 1; then
    printf '\tbudget_policy\tinput_check_cadence\tgap_mean_ns\tgap_min_ns\tgap_max_ns' >> "$results/summary.tsv"
fi
printf '\n' >> "$results/summary.tsv"
cell=0
processes=0
while read -r shape count limit; do
    cell=$((cell+1))
    seed=$((812381+cell*7919))
    reps=$((2000000/(count*limit+1)))
    if test "$reps" -lt 4; then reps=4; fi
    if test "$reps" -gt 64; then reps=64; fi
    if test "$budget" = 1; then reps=8; fi
    if test "$mode" = check; then reps=2; fi
    pass=0
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((pass+cell))" -v reverse="$((pass%2))" '
            { a[NR]=$0 } END { for(j=0;j<NR;j++) { i=(j+shift)%NR; if(reverse) i=NR-1-i; print a[i+1] } }
        ' "$results/configurations.txt" > "$results/order.txt"
        while read -r runtime kernel workers policy cadence; do
            file="$results/$shape-n$count-l$limit-$runtime-$kernel-w$workers-p$pass.tsv"
            if test "$budget" = 1; then file="$results/$shape-n$count-l$limit-$runtime-$kernel-w$workers-$policy-$cadence-p$pass.tsv"; fi
            WF_WORKERS="$workers" WF_BUDGET_CONTROL="$policy" WF_RECORD_CHECK_CADENCE="$cadence" \
                "$OUT/records-$runtime" "$kernel" "$count" "$limit" "$shape" "$reps" "$seed" "$pass" > "$file" 2>&1
            grep -Fxq "# UTF8 records PASS: calls=$((reps+1)) results=$(((reps+1)*count))" "$file"
            test "$(grep -c '^# UTF8 records PASS:' "$file")" -eq 1
            tail -n 1 "$file" | grep -Fxq "# UTF8 records PASS: calls=$((reps+1)) results=$(((reps+1)*count))"
            test "$(grep -c '^# runtime=' "$file")" -eq 1
            test "$(grep -c '^# batch_includes_checks=' "$file")" -eq 1
            world=sequential
            if test "$kernel" = wf && test "$workers" -ge 2; then world=parallel; fi
            grep -Eq "^# runtime=$runtime kernel=$kernel workers=$workers world=$world shape=$shape records=$count bytes=[0-9]+ max_length=$limit seed=$seed pass=$pass reps=$reps entry_ns=[0-9]+ clock_pair_min_ns=[0-9]+$" "$file"
            if test "$runtime" = recovered || test "$runtime" = budget; then
                actual=0; if test "$workers" -ge 2; then actual="(0|$workers)"; fi
                if test "$budget" = 1; then
                    minimum=$cost_minimum; if test "$policy" != cost; then minimum=2; fi
                    actual=0
                    if test "$workers" -ge 2 && test "$count" -ge "$minimum"; then actual=$workers; fi
                fi
                grep -Eq "^# actual_lanes=$actual steals=[0-9]+$" "$file"
                test "$(grep -c '^# actual_lanes=' "$file")" -eq 1
                if grep -q '^# actual_lanes=0 ' "$file"; then grep -Fxq '# actual_lanes=0 steals=0' "$file"; fi
            fi
            awk -F '\t' -v runtime="$runtime" -v kernel="$kernel" -v workers="$workers" \
                -v shape="$shape" -v count="$count" -v limit="$limit" -v seed="$seed" -v pass="$pass" -v reps="$reps" \
                -v budget="$budget" -v policy="$policy" -v cadence="$cadence" '
                function fail() { bad=1; exit 1 }
                /^# budget_policy=/ {
                    if(budget!=1 || (policy!="cost" && policy!="capacity" && policy!="team") || policy_seen++ ||
                       $0!="# budget_policy=" policy || NR!=1) fail(); next
                }
                /^# input_check_cadence=/ {
                    if(budget!=1 || (cadence!="call" && cadence!="batch") || cadence_seen++ ||
                       $0!="# input_check_cadence=" cadence || NR!=2) fail(); next
                }
                /^# runtime=/ { split($0,parts," "); split(parts[8],pair,"="); declared_bytes=pair[2]; next }
                /^# batch_includes_checks=/ {
                    if(resource_seen++ || seen!=reps+1) fail();
                    split($0,parts," "); split(parts[3],pair,"="); batch_ns=pair[2]; next
                }
                /^# actual_lanes=/ { if(actual_seen++ || !resource_seen) fail(); next }
                /^# call_gap / {
                    if(budget!=1 || !resource_seen || !actual_seen || passed ||
                       $0!~/^# call_gap call=[0-9]+ ns=[0-9]+$/) fail();
                    split($0,parts," "); split(parts[3],pair,"=");
                    if(pair[2]+0!=++gap_count || gap_count>reps) fail();
                    split(parts[4],pair,"="); gap=pair[2]+0; gap_sum+=gap;
                    if(gap_count==1 || gap<gap_min)gap_min=gap;
                    if(gap_count==1 || gap>gap_max)gap_max=gap;
                    next
                }
                /^# UTF8 records PASS:/ { if(passed++ || (budget && gap_count!=reps)) fail(); next }
                /^#/ { fail() }
                /^runtime\t/ {
                    if($0!="runtime\tkernel\tworkers\tshape\trecords\tbytes\tmax_length\tseed\tpass\tcall\tphase\tcore_ns\tcycle_ns" || header++) fail();
                    next
                }
                {
                    if(!header || resource_seen || passed || NF!=13 || $1!=runtime || $2!=kernel || $3!=workers || $4!=shape || $5!=count ||
                       $7!=limit || $8!=seed || $9!=pass || $10!=seen || $11!=(seen?"warm":"first") ||
                       $6!~/^[0-9]+$/ || $12!~/^[0-9]+$/ || $13!~/^[0-9]+$/ || $12+0>$13+0) fail();
                    if($6!=declared_bytes || (seen && $6!=bytes)) fail(); bytes=$6; total_cycle+=$13;
                    if(seen) { for(i=12;i<=13;i++) {v=$i+0;sum[i]+=v;if(seen==1 || v<min[i])min[i]=v;if(seen==1 || v>max[i])max[i]=v;} }
                    seen++;
                }
                END {
                    if(bad || header!=1 || seen!=reps+1 || resource_seen!=1 || passed!=1 ||
                       batch_ns+0<total_cycle+gap_sum || policy_seen!=budget || cadence_seen!=budget ||
                       gap_count!=budget*reps) exit 1;
                    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%d",runtime,kernel,workers,shape,count,bytes,limit,seed,pass,reps;
                    for(i=12;i<=13;i++) printf "\t%.3f\t%.0f\t%.0f",sum[i]/reps,min[i],max[i];
                    if(budget) printf "\t%s\t%s\t%.3f\t%.0f\t%.0f",policy,cadence,gap_sum/reps,gap_min,gap_max;
                    printf "\n";
                }
            ' "$file" > "$results/row.tsv"
            cat "$results/row.tsv" >> "$results/summary.tsv"
            grep -Eq '^# batch_includes_checks=1 batch_ns=[0-9]+ user_us=[0-9]+ system_us=[0-9]+ maxrss_bytes=[0-9]+ voluntary_switches=[0-9]+ involuntary_switches=[0-9]+$' "$file"
            processes=$((processes+1))
        done < "$results/order.txt"
        pass=$((pass+1))
    done
done < "$results/cells.txt"
configurations=6; if test "$budget" = 1; then configurations=18; fi
expected=$((cell*configurations*rounds))
test "$processes" -eq "$expected"
test "$(wc -l < "$results/summary.tsv")" -eq "$((expected+1))"
if test "$budget" = 1; then test "$(shasum -a 256 "$OUT/records-budget")" = "$budget_hash"; fi
printf 'UTF8 records %s PASS: processes=%s results=%s\n' "$requested" "$processes" "$results"
