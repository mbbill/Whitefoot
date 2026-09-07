#!/bin/sh
# Explicit caller for the checked O3 binaries. One process runs at a time;
# calibration rotates/reverses configuration order and retains every call.
set -eu
cd "$(dirname "$0")"
: "${OUT:?set OUT to the build-bench output directory}"
case "$OUT" in /*) ;; *) echo 'OUT must be absolute' >&2; exit 1;; esac
mode=${1:-check}
case "$mode" in check|calibrate) ;; *) echo 'expected check or calibrate' >&2; exit 1;; esac
rounds=${ROUNDS:-5}
case "$rounds" in ''|*[!0-9]*) exit 1;; esac
test "$rounds" -ge 1 && test "$rounds" -le 20
if test "$mode" = check; then rounds=1; fi
result=${RESULTS:-$OUT/$mode-$$}
# A run never overwrites an earlier table, including a failed one.
mkdir "$result"
printf '%s\n' "$result" > "$OUT/last-$mode-path.txt"
{
    printf 'mode=%s rounds=%s\n' "$mode" "$rounds"
    uname -a
    git rev-parse HEAD
    git status --short
    cat "$OUT/bench-toolchain.txt"
    cat "$OUT/fir-wf-flags.txt"
    if test "$(uname -s)" = Linux; then
        lscpu
        cat /proc/self/status
        for path in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/cpuset.cpus.effective; do
            printf '%s: ' "$path"
            if test -r "$path"; then cat "$path"; else printf 'not readable; unqualified\n'; fi
        done
    else
        sysctl hw.model hw.ncpu hw.physicalcpu hw.l1dcachesize hw.l2cachesize machdep.cpu.brand_string
    fi
    shasum -a 256 fir.wf fir_direct.wf fir_lanes.wf fir_host.ll fir_bench.c fir_native.c fir_native.h fir_check.c fir_wf_check.c \
        fir_static.c fir_static.h fir_static_check.c runtime.c runtime.h Makefile fir-bench.sh \
        ../../../compiler/src/backend/wf_floor.c ../../../compiler/src/backend/sched/core.c \
        ../../../compiler/src/backend/sched/prim_host.c ../../../compiler/src/backend/sched/entry.c \
        "$OUT/fir-host.ll" "$OUT/fir-wf.o" "$OUT/fir-native.o" "$OUT/fir-static.o" \
        "$OUT/bench-weak" "$OUT/bench-static" "$OUT/bench-recovered" "$OUT/bench-shared" \
        "$OUT/fir-lanes-host.ll" "$OUT/fir-wf-lanes16.o" "$OUT/fir-wf-lanes16-slp.o" \
        "$OUT/bench-weak-wf-lanes16" "$OUT/bench-recovered-wf-lanes16" "$OUT/bench-shared-wf-lanes16" \
        "$OUT/bench-weak-wf-lanes16-slp" "$OUT/bench-recovered-wf-lanes16-slp" "$OUT/bench-shared-wf-lanes16-slp"
} > "$result/manifest.txt"
git diff --binary HEAD -- . ../../../.github/workflows/compute-bench.yml > "$result/source.patch"
cp "$OUT/fir-native-release-check.log" "$result/native-qualification.txt"
for lanes in 1 2 4; do cp "$OUT/fir-static-release-$lanes.log" "$result/"; done
for variant in direct lanes16 lanes16-slp; do
    cp "$OUT/fir-wf-$variant-check.log" "$OUT/fir-wf-$variant-sanitized-check.log" "$result/"
done

if test "$mode" = check; then
    printf '3 0\n7 33\n64 4097\n' > "$result/cells.txt"
    tiles='17'
    widths='0 2 4'
else
    # Calibration only. The proposed held-out K/N/input families are NOT run
    # here and may not be promoted from this screen to confirmation results.
    for k in 3 15 64; do
        for n in 33 4097 262144; do printf '%s %s\n' "$k" "$n"; done
    done > "$result/cells.txt"
    printf '64 0\n' >> "$result/cells.txt"
    tiles='257 4096 65536'
    if test "$(uname -s)" = Linux; then cpus=$(nproc); else cpus=$(sysctl -n hw.ncpu); fi
    widths='0'
    if test "$cpus" -ge 2; then widths="$widths 2"; fi
    if test "$cpus" -ge 4; then widths="$widths 4"; fi
fi
{
    for kernel in direct lanes4 lanes8 lanes16; do printf 'weak %s 0 65536\n' "$kernel"; done
    for kernel in direct lanes4 lanes8 lanes16; do
        for width in $widths; do printf 'static static-%s %s 65536\n' "$kernel" "$width"; done
    done
    for kernel in wf wf-lanes16 wf-lanes16-slp; do
        for tile in $tiles; do
            printf 'weak %s 0 %s\n' "$kernel" "$tile"
            for runtime in recovered shared; do
                for width in $widths; do printf '%s %s %s %s\n' "$runtime" "$kernel" "$width" "$tile"; done
            done
        done
    done
} > "$result/configurations.txt"
printf 'runtime\tkernel\tworkers\tk\tn\ttile\tseed\tpass\tcalls\tcore_mean_ns\tcore_min_ns\tcore_max_ns\tcycle_mean_ns\tcycle_min_ns\tcycle_max_ns\n' > "$result/summary.tsv"
cell=0
while read -r k n; do
    cell=$((cell + 1))
    reps=$((8000000 / (k * n + 1)))
    if test "$reps" -lt 4; then reps=4; fi
    if test "$reps" -gt 256; then reps=256; fi
    if test "$mode" = check; then reps=2; fi
    seed=$((92821 + cell * 7919))
    pass=0
    while test "$pass" -lt "$rounds"; do
        # Shift by both cell and pass, reversing every second pass.
        awk -v shift="$((pass + cell))" -v reverse="$((pass % 2))" '
            { a[NR]=$0 } END { for (j=0;j<NR;j++) {
                i=(j+shift)%NR; if (reverse) i=NR-1-i; print a[i+1]
            }}' "$result/configurations.txt" > "$result/order.txt"
        while read -r runtime kernel width tile; do
            log="$result/k$k-n$n-$runtime-$kernel-w$width-t$tile-p$pass.tsv"
            suffix=
            case "$kernel" in wf-lanes16|wf-lanes16-slp) suffix=-$kernel;; esac
            WF_WORKERS=$width WF_SCHED_REPORT=1 "$OUT/bench-$runtime$suffix" \
                "$kernel" "$k" "$n" "$tile" "$reps" "$seed" "$pass" > "$log" 2>&1
            grep -Fxq "# FIR bench PASS: calls=$((reps + 1)) samples=$(((reps + 1) * n)) history=$(((reps + 1) * (k - 1)))" "$log"
            if test "$runtime" = static; then
                lanes=$width; if test "$lanes" -lt 2; then lanes=1; fi
                # The ordinary PASS precedes separately timed shutdown. A
                # complete comparison also requires exact capacity and the
                # final lifecycle report after successful pool destruction.
                awk -v lanes="$lanes" '
                    /^# FIR bench PASS:/ {passed=1}
                    /^# static_requested_lanes=/ {
                        if(!passed || reports++)bad=1;
                        expected="# static_requested_lanes=" lanes " static_actual_lanes=" lanes \
                          " static_helpers=" (lanes-1) " static_creation_error=0 static_idle=condvar static_spin=0" \
                          " static_startup_in_first_core=1 static_shutdown_outside_batch=1 static_shutdown_ns=";
                        if(substr($0,1,length(expected))!=expected ||
                           substr($0,length(expected)+1) !~ /^[0-9]+$/)bad=1;
                        last=NR
                    }
                    END {exit bad || reports!=1 || last!=NR}
                ' "$log"
            fi
            # Raw calls are retained. This table contains per-process warm
            # means/ranges, not confidence intervals or percentile claims.
            awk -F '\t' -v runtime="$runtime" -v kernel="$kernel" -v width="$width" \
                -v k="$k" -v n="$n" -v tile="$tile" -v seed="$seed" -v pass="$pass" -v reps="$reps" '
            /^#/ {next}
            $0=="runtime\tkernel\tworkers\tk\tn\ttile\tseed\tpass\tcall\tphase\tcore_ns\tcycle_ns" {
                if(header++ || rows)bad=1; next
            }
            {
                if(!header || NF!=12 || $1!=runtime || $2!=kernel || $3!=width || $4!=k ||
                   $5!=n || $6!=tile || $7!=seed || $8!=pass || $9!=rows ||
                   $10!=(rows==0?"first":"warm") || $11>$12)bad=1;
                for(i=3;i<=9;i++) if($i !~ /^[0-9]+$/)bad=1;
                if($11 !~ /^[0-9]+$/ || $12 !~ /^[0-9]+$/)bad=1;
                rows++
            }
            $10=="warm" {
                count++; core+=$11; cycle+=$12;
                if(count==1 || $11<cmin)cmin=$11; if($11>cmax)cmax=$11;
                if(count==1 || $12<ymin)ymin=$12; if($12>ymax)ymax=$12;
                key=$1 FS $2 FS $3 FS $4 FS $5 FS $6 FS $7 FS $8
            } END { if(bad || header!=1 || rows!=reps+1 || count!=reps)exit 1;
                printf "%s\t%d\t%.3f\t%.0f\t%.0f\t%.3f\t%.0f\t%.0f\n",
                    key,count,core/count,cmin,cmax,cycle/count,ymin,ymax
            }' "$log" >> "$result/summary.tsv"
        done < "$result/order.txt"
        pass=$((pass + 1))
    done
done < "$result/cells.txt"
expected=$(awk 'END {print NR}' "$result/cells.txt")
configs=$(awk 'END {print NR}' "$result/configurations.txt")
actual=$(awk 'END {print NR-1}' "$result/summary.tsv")
test "$actual" -eq "$((expected * configs * rounds))"
printf 'FIR %s PASS: processes=%s results=%s\n' "$mode" "$actual" "$result"
