#!/bin/zsh
# Paired browser-layout benchmark harness.
#
# Protocol: every (configuration, implementation) cell is visited exactly once
# per round, in a fixed rotation, and the rounds repeat N times. No cell is ever
# run back-to-back with itself. Exit codes are read directly from the process
# (output goes to a file; no pipeline is involved). Every run's stdout is kept
# so the byte comparison is over the real published bytes.
#
# usage: run_bench.zsh [ROUNDS]
set -u
zmodload zsh/datetime

ROOT=${0:A:h}
cd "$ROOT"
# Children inherit the harness's scheduling state. If this shell was started in
# PRIO_DARWIN_BG (which biases Apple-silicon placement towards efficiency
# cores), move it out, so that every measured process is scheduled the same way.
taskpolicy -B -p $$ 2>/dev/null || true
# The `default` worker cell measures a genuinely unset WF_WORKERS, so an
# inherited one would silently turn it into a configured cell. Every explicit
# cell sets the variable on its own command line, so clearing it here costs
# nothing and closes that hole.
unset WF_WORKERS
ROUNDS=${1:-9}
OFFSET=${2:-0}
R=rust/target/release/paired-layout
OUTDIR=out
mkdir -p "$OUTDIR"
RESULTS=logs/results.tsv

if (( OFFSET == 0 )); then
  print -r -- $'round\tshape\tdepth\twords\treps\timpl\tthreads\tseconds\texit\toutput' > "$RESULTS"
fi

typeset -a CONFIGS
while read -r shape depth words reps; do
  case "$shape" in ''|\#*) continue ;; esac
  CONFIGS+=("$shape $depth $words $reps")
done < configs.txt

# Fixed rotation of implementations inside each configuration visit. The
# `default` cells are the shipped defaults of the two languages: WF_WORKERS
# genuinely unset, and rayon's own global pool with no thread count named.
typeset -a IMPLS
IMPLS=(
  wf_seq:0
  wf_par:1
  wf_par:2
  wf_par:4
  wf_par:8
  wf_par:default
  rs_seq:0
  rs_rayon:2
  rs_rayon:4
  rs_rayon:8
  rs_rayon:default
  rs_cut:2
  rs_cut:4
  rs_cut:8
)

total=$(( ROUNDS * ${#CONFIGS} * ${#IMPLS} ))
done_n=0

for (( round = OFFSET + 1; round <= OFFSET + ROUNDS; round++ )); do
  print -u2 -- "round $round load: $(sysctl -n vm.loadavg)"
  for cfg in "${CONFIGS[@]}"; do
    set -- ${=cfg}
    shape=$1; depth=$2; words=$3; reps=$4
    tag="${shape}_d${depth}_w${words}"
    for spec in "${IMPLS[@]}"; do
      impl=${spec%%:*}
      threads=${spec##*:}
      outfile="$OUTDIR/${tag}__${impl}_${threads}__r${round}.txt"
      case "$impl" in
        wf_seq)
          t0=$EPOCHREALTIME
          ./bin/${tag}_seq > "$outfile"
          e=$?
          t1=$EPOCHREALTIME
          ;;
        wf_par)
          if [[ $threads == default ]]; then
            t0=$EPOCHREALTIME
            ./bin/${tag}_par > "$outfile"
            e=$?
            t1=$EPOCHREALTIME
          else
            t0=$EPOCHREALTIME
            WF_WORKERS=$threads ./bin/${tag}_par > "$outfile"
            e=$?
            t1=$EPOCHREALTIME
          fi
          ;;
        rs_seq)
          t0=$EPOCHREALTIME
          $R seq $shape $depth $words $reps 1 > "$outfile"
          e=$?
          t1=$EPOCHREALTIME
          ;;
        rs_rayon)
          t0=$EPOCHREALTIME
          $R rayon $shape $depth $words $reps $threads > "$outfile"
          e=$?
          t1=$EPOCHREALTIME
          ;;
        rs_cut)
          t0=$EPOCHREALTIME
          $R rayoncut $shape $depth $words $reps $threads > "$outfile"
          e=$?
          t1=$EPOCHREALTIME
          ;;
      esac
      secs=$(( t1 - t0 ))
      hex=$(< "$outfile")
      printf '%d\t%s\t%s\t%s\t%s\t%s\t%s\t%.6f\t%d\t%s\n' \
        $round $shape $depth $words $reps $impl $threads $secs $e "${hex//[$'\n']/}" >> "$RESULTS"
      done_n=$(( done_n + 1 ))
      if (( done_n % 48 == 0 )); then
        print -u2 -- "progress: $done_n / $total"
      fi
    done
  done
done

print -u2 -- "done: $done_n runs"
