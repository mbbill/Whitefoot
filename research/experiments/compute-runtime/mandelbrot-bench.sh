#!/bin/sh
set -eu
cd "$(dirname "$0")"
mode=${1:?mode required}
case "$mode" in check|calibrate|grain-check|grain-calibrate|split-check|split-calibrate) ;; *) exit 1;; esac
: "${OUT:?build output required}"
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
for flag in -fno-vectorize -fno-slp-vectorize -fno-lto -ffp-contract=off; do
    head -n 1 "$OUT/mandelbrot-flags.txt" | grep -Fq -- "$flag"
done
results=${RESULTS:-$OUT/mandelbrot-$mode-$$}
test ! -e "$results"
mkdir -p "$results"
printf '%s\n' "$results" > "$OUT/last-mandelbrot-$mode-path.txt"
{
    git rev-parse HEAD
    uname -a
    cat "$OUT/mandelbrot-flags.txt" "$OUT/scheduler-flags.txt" "$OUT/scheduler-deps/metadata.txt"
    if test "$(uname -s)" = Linux; then
        lscpu
        cat /proc/self/status
        for path in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            if test -r "$path"; then printf '%s: ' "$path"; cat "$path"; else printf '%s: unqualified\n' "$path"; fi
        done
    else
        sysctl hw.model hw.ncpu hw.physicalcpu hw.logicalcpu hw.memsize
    fi
    shasum -a 256 mandelbrot.wf mandelbrot_host.ll mandelbrot.c mandelbrot-bench.sh Makefile runtime.c runtime.h \
        records_scheduler.h records_selector.c records_runtime.c records_static.c records_tbb.cpp records_parlay.cpp records_rayon.c \
        records-scheduler-deps.sh records-scheduler-memory.awk records-scheduler-memory.sh records-rayon/adapter.rs \
        records-rayon/Cargo.toml records-rayon/Cargo.lock ../../../compiler/src/backend/wf_floor.c
    shasum -a 256 "$OUT"/mandelbrot*.o "$OUT"/mandelbrot*.ll "$OUT"/scheduler-layout-*.o "$OUT/scheduler-floor.o" \
        "$OUT/scheduler-deps/rayon/release/libwf_records_rayon.a" "$OUT"/scheduler-deps/lib/libtbb.* \
        "$OUT/mandelbrot" "$OUT/mandelbrot-sanitized"
} > "$results/manifest.txt"
git diff --binary HEAD > "$results/source.patch"
cat > "$results/configurations.txt" <<'CONFIG'
wf-seq 1 cost 0
wf-auto 1 cost 0
wf-auto 4 cost 0
wf-auto 4 capacity 0
wf-auto 4 team 0
CONFIG
grains=64;configurations=19
case "$mode" in grain-*) grains='1 16 64 256 1024';configurations=75;; esac
for backend in wf static tbb parlay parlay-auto rayon-join rayon-iter; do
    for grain in $grains; do
        printf '%s 1 cost %s\n%s 4 cost %s\n' "$backend" "$grain" "$backend" "$grain" >> "$results/configurations.txt"
    done
done
case "$mode" in split-*)
    cat > "$results/configurations.txt" <<'CONFIG'
wf-seq 1 cost 0
wf-auto 4 cost 0
wf-auto 4 capacity 0
wf-auto 4 chunks16 0
wf-auto 4 chunks256 0
CONFIG
    for backend in wf tbb rayon-join; do
        for grain in 16 64 256; do printf '%s 4 cost %s\n' "$backend" "$grain"; done
    done >> "$results/configurations.txt"
    configurations=14
;; esac
test "$(wc -l < "$results/configurations.txt")" -eq "$configurations"
if test "$mode" = check || test "$mode" = split-check; then
    # Full qualification supplies its own grain16 fixtures. Distinct timing
    # grains of one backend/width/policy need only one identical qualification.
    awk '!seen[$1 " " $2 " " $3]++' "$results/configurations.txt" > "$results/qualification-configurations.txt"
    for image in mandelbrot mandelbrot-sanitized; do
        while read -r backend width policy grain; do
            log="$results/$image-$backend-w$width-$policy-check.log"
            status=0
            WF_WORKERS="$width" WF_BUDGET_CONTROL="$policy" LSAN_OPTIONS=exitcode=23 \
                "$OUT/$image" "$backend" "$width" check > "$log" 2>&1 || status=$?
            leak=0
            case "$(uname -s)/$(uname -m)/$image/$backend" in
                Linux/x86_64/mandelbrot-sanitized/rayon-*)
                    if grep -Eq '^sanitizer=.*address' "$OUT/mandelbrot-flags.txt";then leak=1;fi;;
            esac
            expected_status=0;if test "$leak" = 1;then expected_status=23;fi
            test "$status" -eq "$expected_status"
            : > "$results/functional-prefix.txt"
            awk -v leak="$leak" -v extra=0 -v object_basename=mandelbrot-sanitized \
                -v prefix="$results/functional-prefix.txt" -f records-scheduler-memory.awk "$log"
            grep -Fxq '# Mandelbrot qualification PASS: leaves=185155 calls=252 outputs=184422' "$results/functional-prefix.txt"
            test "$(wc -l < "$results/functional-prefix.txt")" -eq 1
        done < "$results/qualification-configurations.txt"
    done
fi
if test "$mode" = split-check; then
    rounds=1
    printf '%s\n' 'plane 0 0' 'trailing 4096 256' 'trailing 4097 256' 'exterior 33 16' > "$results/cells.txt"
elif test "$mode" = split-calibrate; then
    for shape in plane trailing interior exterior; do
        for count in 4096 4097; do printf '%s %s 256\n' "$shape" "$count"; done
    done > "$results/cells.txt"
    printf '%s\n' 'exterior 33 16' >> "$results/cells.txt"
elif test "$mode" = check; then
    rounds=1
    printf '%s\n' 'plane 0 0' 'boundary 33 16' 'clustered 4097 256' 'trailing 4097 256' 'exterior 33 16' > "$results/cells.txt"
elif test "$mode" = grain-check; then
    rounds=1
    printf '%s\n' 'trailing 4097 16' 'exterior 33 16' > "$results/cells.txt"
elif test "$mode" = grain-calibrate; then
    for shape in plane clustered trailing interior exterior; do
        printf '%s 4097 256\n' "$shape"
    done > "$results/cells.txt"
    for shape in plane exterior; do printf '%s 33 16\n' "$shape"; done >> "$results/cells.txt"
else
    for shape in plane boundary clustered interleaved interior exterior trailing; do
        printf '%s 4097 256\n' "$shape"
    done > "$results/cells.txt"
    for shape in plane boundary interior exterior; do printf '%s 33 16\n' "$shape"; done >> "$results/cells.txt"
fi
image_hash=$(shasum -a 256 "$OUT/mandelbrot")
weight=$(awk '/call i64 @wf__par_split_budget\(/ {sub(/^.* i64 /, "");sub(/\).*$/, "");print;seen++} END{if(seen!=1)exit 1}' "$OUT/mandelbrot-host.ll")
work=$(awk '$1=="#define" && $2=="WF_PAR_SPLIT_WORK_PER_CHUNK"{print $3;seen++} END{if(seen!=1)exit 1}' runtime.c)
case "$weight:$work" in *[!0-9:]*|:*|*:) exit 1;;esac
test "$weight" -gt 0 && test "$work" -gt 0
minimum=$(awk -v weight="$weight" -v work="$work" 'BEGIN{print 2*(weight>=work?1:int((work+weight-1)/weight))}')
printf 'backend\twidth\tpolicy\tshape\tpoints\tlimit\tgrain\tseed\tpass\titerations\twarm_calls\tcore_mean_ns\tcore_min_ns\tcore_max_ns\tcycle_mean_ns\tcycle_min_ns\tcycle_max_ns\n' > "$results/summary.tsv"
cell=0;processes=0
while read -r shape count limit; do
    cell=$((cell+1));pass=0
    expected_iterations=
    escaped=1;if test "$limit" = 0;then escaped=0;fi
    case "$shape" in
        interior) expected_iterations=$((count*limit));;
        exterior) expected_iterations=$((count*escaped));;
        clustered|interleaved|trailing) inside=$(((count+3)/4));expected_iterations=$((inside*limit+(count-inside)*escaped));;
    esac
    while test "$pass" -lt "$rounds"; do
        awk -v shift="$((cell+pass))" -v reverse="$((pass%2))" '{a[NR]=$0} END{for(j=0;j<NR;j++){i=(j+shift)%NR;if(reverse)i=NR-1-i;print a[i+1]}}' \
            "$results/configurations.txt" > "$results/order.txt"
        while read -r backend width policy grain; do
            file="$results/$shape-n$count-l$limit-$backend-w$width-$policy-g$grain-p$pass.tsv"
            reps=8;case "$mode" in *check) reps=2;if test "$shape" = exterior;then reps=256;fi;;esac
            expected_lanes=0
            if test "$width" -gt 1;then
                if test "$backend" = wf && test "$count" -gt "$grain";then expected_lanes=$width;fi
                if test "$backend" = wf-auto;then
                    threshold=$minimum;if test "$policy" != cost;then threshold=2;fi
                    if test "$count" -ge "$threshold";then expected_lanes=$width;fi
                fi
            fi
            WF_WORKERS="$width" WF_BUDGET_CONTROL="$policy" "$OUT/mandelbrot" "$backend" "$width" \
                "$shape" "$count" "$limit" "$grain" "$reps" 828219 "$pass" > "$file" 2>&1
            awk -F '\t' -v backend="$backend" -v width="$width" -v policy="$policy" -v shape="$shape" \
                -v count="$count" -v limit="$limit" -v grain="$grain" -v reps="$reps" -v pass="$pass" \
                -v expected_lanes="$expected_lanes" -v expected_iterations="$expected_iterations" '
                function fail(){bad=1;exit 1}
                /^# backend=/ {
                    prefix="# backend=" backend " width=" width " policy=" policy " shape=" shape " points=" count " limit=" limit " grain=" grain " reps=" reps " seed=828219 pass=" pass " iterations=";
                    if(NR!=1 || metadata++ || substr($0,1,length(prefix))!=prefix)fail();
                    iterations=substr($0,length(prefix)+1);if(iterations!~/^[0-9]+$/ || iterations+0>count*limit ||
                        (expected_iterations!="" && iterations+0!=expected_iterations+0))fail();next
                }
                /^call\t/ {if(NR!=2 || header++ || $0!="call\tphase\tcore_ns\tcycle_ns")fail();next}
                /^# batch_ns=/ {
                    if(resources++ || calls!=reps+1 || passed || $0!~/^# batch_ns=[0-9]+ user_us=[0-9]+ system_us=[0-9]+ voluntary=[0-9]+ involuntary=[0-9]+ wf_pool_lanes=[0-9]+ maxrss_bytes=[0-9]+$/)fail();
                    split($0,a," ");split(a[2],b,"=");batch_ns=b[2]+0;
                    split(a[7],b,"=");lanes=b[2]+0;
                    if(lanes!=expected_lanes)fail();next
                }
                /^# gap / {
                    if(!resources || passed || $0!~/^# gap call=[0-9]+ ns=[0-9]+$/)fail();
                    split($0,a," ");split(a[3],b,"=");if(b[2]+0!=++gaps || gaps>reps)fail();
                    split(a[4],b,"=");gap_sum+=b[2];next
                }
                /^# Mandelbrot PASS:/ {
                    if(passed++ || !resources || gaps!=reps || $0!="# Mandelbrot PASS: calls=" reps+1 " outputs=" (reps+1)*count)fail();next
                }
                {
                    if(!header || resources || passed || NF!=4 || $1!=calls || $2!=(calls?"warm":"first") || $3!~/^[0-9]+$/ || $4!~/^[0-9]+$/ || $3+0>$4+0)fail();
                    total_cycle+=$4;
                    if(calls)for(i=3;i<=4;i++){v=$i+0;sum[i]+=v;if(calls==1||v<min[i])min[i]=v;if(calls==1||v>max[i])max[i]=v;}
                    calls++
                }
                END {
                    if(bad || metadata!=1 || header!=1 || calls!=reps+1 || resources!=1 || passed!=1 || gaps!=reps || batch_ns<total_cycle+gap_sum)exit 1;
                    printf "%s\t%d\t%s\t%s\t%d\t%d\t%d\t828219\t%d\t%.0f\t%d",backend,width,policy,shape,count,limit,grain,pass,iterations,reps;
                    for(i=3;i<=4;i++)printf "\t%.3f\t%.0f\t%.0f",sum[i]/reps,min[i],max[i];printf "\n"
                }' "$file" >> "$results/summary.tsv"
            if test -z "$expected_iterations";then expected_iterations=$(tail -n 1 "$results/summary.tsv" | cut -f 10);fi
            processes=$((processes+1))
        done < "$results/order.txt"
        pass=$((pass+1))
    done
done < "$results/cells.txt"
test "$processes" -eq "$((cell*configurations*rounds))"
test "$(wc -l < "$results/summary.tsv")" -eq "$((processes+1))"
test "$(shasum -a 256 "$OUT/mandelbrot")" = "$image_hash"
printf 'Mandelbrot %s PASS: processes=%s results=%s\n' "$mode" "$processes" "$results"
