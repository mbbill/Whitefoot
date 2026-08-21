#!/bin/zsh
# Byte comparison over the real published bytes of every run.
#   1. all Whitefoot outputs of one configuration, pairwise, across the default
#      compilation and --par at every worker count and every round;
#   2. all Rust outputs of one configuration, likewise;
#   3. Whitefoot against Rust.
# Uses cmp on the files themselves, not on remembered strings.
set -u
ROOT=${0:A:h}
cd "$ROOT"

fail=0
wfcells=0
rscells=0

while read -r shape depth words reps; do
  case "$shape" in ''|\#*) continue ;; esac
  tag="${shape}_d${depth}_w${words}"

  wf=(out/${tag}__wf_*.txt)
  rs=(out/${tag}__rs_*.txt)
  if (( ${#wf} == 0 || ${#rs} == 0 )); then
    print "MISSING outputs for $tag"
    fail=1
    continue
  fi

  ref=${wf[1]}
  for f in "${wf[@]}"; do
    if ! cmp -s "$ref" "$f"; then
      print "WF DIVERGENCE $tag: $ref vs $f"
      fail=1
    fi
    wfcells=$(( wfcells + 1 ))
  done

  rref=${rs[1]}
  for f in "${rs[@]}"; do
    if ! cmp -s "$rref" "$f"; then
      print "RUST DIVERGENCE $tag: $rref vs $f"
      fail=1
    fi
    rscells=$(( rscells + 1 ))
  done

  if cmp -s "$ref" "$rref"; then
    print "$tag: WF == Rust, bytes $(tr -d '\n' < $ref), ${#wf} WF runs, ${#rs} Rust runs"
  else
    print "$tag: CROSS-LANGUAGE DIFFERENCE  WF=$(tr -d '\n' < $ref)  Rust=$(tr -d '\n' < $rref)"
    fail=1
  fi
done < configs.txt

print "compared $wfcells WF outputs and $rscells Rust outputs"
if (( fail == 0 )); then
  print "RESULT: every run of every configuration published identical bytes, in both languages and across them."
else
  print "RESULT: divergences reported above."
fi
exit $fail
