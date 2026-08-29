#!/bin/sh
# Reproduces every verdict in AUDIT2.md's probe ledger.
WFC=/tmp/claude-0/-home-user-Whitefoot/6a4209eb-2cad-5504-9f06-67307ee32037/scratchpad/wf-0111-guarded-facts/target/release/whitefootc
D=$(dirname "$0")/probes
for f in "$D"/*.wf; do
  out=$($WFC "$f" 2>&1)
  if [ -z "$out" ]; then
    echo "ACCEPT  $(basename $f)"
  else
    res=$(printf '%s' "$out" | sed -n 's/.*residual: "\([^"]*\)".*/\1/p')
    rule=$(printf '%s' "$out" | sed -n 's/.*Semantics\/Source \[\([A-Z0-9-]*\)\].*/\1/p')
    if [ -n "$res" ]; then echo "REJECT  $(basename $f)  [$rule] residual: $res"
    else echo "REJECT  $(basename $f)  [$rule] $(printf '%s' "$out" | sed -n 's/.*kind: \([A-Za-z]*\).*/\1/p')"; fi
  fi
done
