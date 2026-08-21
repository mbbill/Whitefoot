#!/bin/zsh
# Interleaved min-of-N timer. Every cell is visited once per round; rounds
# repeat N times, so drift is spread across cells rather than concentrated.
# Usage: timeit.zsh N "label=WF_WORKERS:binary" ...
# Exit status of every run is read directly from the process, never a pipeline.
zmodload zsh/datetime
emulate -L zsh
setopt err_return

typeset -i N=$1; shift
typeset -a cells; cells=("$@")
typeset -A best
typeset -A worst
typeset -A fails
typeset -A outsum

mkdir -p out
for c in $cells; do best[$c]=999999; worst[$c]=0; fails[$c]=0; done

for ((r = 1; r <= N; r++)); do
  for c in $cells; do
    label=${c%%=*}; rest=${c#*=}
    workers=${rest%%:*}; bin=${rest#*:}
    of=out/${label}.r${r}
    t0=$EPOCHREALTIME
    if [[ $workers == "-" ]]; then
      "$bin" > $of
    else
      WF_WORKERS=$workers "$bin" > $of
    fi
    rc=$?
    t1=$EPOCHREALTIME
    (( rc != 0 )) && (( fails[$c]++ ))
    d=$(( t1 - t0 ))
    (( d < best[$c] )) && best[$c]=$d
    (( d > worst[$c] )) && worst[$c]=$d
    outsum[$c]=$(shasum -a 256 < $of | cut -c1-16)
  done
done

for c in $cells; do
  label=${c%%=*}
  printf "%-22s min %8.4fs  max %8.4fs  spread %5.1f%%  fails %d  sha %s\n" \
    "$label" ${best[$c]} ${worst[$c]} \
    $(( 100.0 * (worst[$c] - best[$c]) / best[$c] )) ${fails[$c]} ${outsum[$c]}
done
