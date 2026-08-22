#!/bin/zsh
# Builds every markdown table in RESULTS.md from logs/results.tsv.
# Re-runnable with one command; overwrites logs/t_*.md in place.
set -u
ROOT=${0:A:h}
cd "$ROOT"
R=rust/target/release/paired-layout

awk -f cells.awk logs/results.tsv > logs/cells.tsv

# Configuration inventory with node counts. "leaf work" is the per-leaf
# arithmetic each family varies: metric-table words for the layout shapes, the
# orbit iteration cap for the index-split grid.
{
  print "| config | shape | nodes | leaves | leaf work | reps |"
  print "|---|---|---:|---:|---:|---:|"
  while read -r shape depth words reps; do
    case "$shape" in ''|\#*) continue ;; esac
    info=$($R nodes $shape $depth $words 1)
    n=${${info#nodes=}%% *}
    l=${${info##*leaves=}%% *}
    print "| \`${shape}_d${depth}_w${words}\` | $shape, depth $depth | $n | $l | $words | $reps |"
  done < configs.txt
} > logs/t_inventory.md

awk -F'\t' '
NR == 1 { next }
{
  key = $1 SUBSEP $2 SUBSEP $3
  lo[key] = $5; med[key] = $6; hi[key] = $7; nn[key] = $4; spread[key] = $8
  if (!($1 in cfgseen)) { cfgseen[$1] = 1; cfgs[++nc] = $1 }
  if ($8 + 0 > worst) { worst = $8 + 0; worstcell = $1 " " $2 "/" $3 }
  tot_spread += $8; cells++
  if ($8 + 0 > 20) over20++
  if ($8 + 0 > 50) over50++
  reps_n = $4
}
function g(c, i, t) { return lo[c SUBSEP i SUBSEP t] }
function m(c, i, t) { return med[c SUBSEP i SUBSEP t] }
function sp(c, i, t) { return spread[c SUBSEP i SUBSEP t] }
# The protocol: a difference under 20% is not resolved by this measurement.
function verdict(r) { return (r > 0.833333 && r < 1.2) ? " unres." : "" }
END {
  # --- sequential parity: the codegen-quality axis ---
  f = "logs/t_seq_parity.md"
  print "| config | WF-seq min | med | spread | Rust-seq min | med | spread | WF/Rust (min-of-N) |" > f
  print "|---|---:|---:|---:|---:|---:|---:|---:|" > f
  for (i = 1; i <= nc; i++) {
    c = cfgs[i]
    w = g(c, "wf_seq", 0); r = g(c, "rs_seq", 0)
    printf "| `%s` | %.3f | %.3f | %.0f%% | %.3f | %.3f | %.0f%% | %.2fx%s |\n", \
      c, w, m(c,"wf_seq",0), sp(c,"wf_seq",0), r, m(c,"rs_seq",0), sp(c,"rs_seq",0), w/r, verdict(w/r) > f
  }

  # --- WF --par scaling: against WF sequential ---
  f = "logs/t_wf_par.md"
  print "| config | WF-seq | --par w=1 | w=2 | w=4 | w=8 | seq/w2 | seq/w4 | seq/w8 | opt-in cost w1/seq |" > f
  print "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|" > f
  for (i = 1; i <= nc; i++) {
    c = cfgs[i]
    s = g(c,"wf_seq",0); p1 = g(c,"wf_par",1); p2 = g(c,"wf_par",2); p4 = g(c,"wf_par",4); p8 = g(c,"wf_par",8)
    printf "| `%s` | %.3f | %.3f | %.3f | %.3f | %.3f | %.2fx%s | %.2fx%s | %.2fx%s | %.2fx%s |\n", \
      c, s, p1, p2, p4, p8, s/p2, verdict(s/p2), s/p4, verdict(s/p4), s/p8, verdict(s/p8), p1/s, verdict(p1/s) > f
  }
  f = "logs/t_wf_par_spread.md"
  print "| config | WF-seq | w=1 | w=2 | w=4 | w=8 |" > f
  print "|---|---:|---:|---:|---:|---:|" > f
  for (i = 1; i <= nc; i++) {
    c = cfgs[i]
    printf "| `%s` | %.0f%% | %.0f%% | %.0f%% | %.0f%% | %.0f%% |\n", c, \
      sp(c,"wf_seq",0), sp(c,"wf_par",1), sp(c,"wf_par",2), sp(c,"wf_par",4), sp(c,"wf_par",8) > f
  }

  # --- rayon scaling: against Rust sequential ---
  f = "logs/t_rayon.md"
  print "| config | Rust-seq | rayon t=2 | t=4 | t=8 | seq/t2 | seq/t4 | seq/t8 |" > f
  print "|---|---:|---:|---:|---:|---:|---:|---:|" > f
  for (i = 1; i <= nc; i++) {
    c = cfgs[i]
    s = g(c,"rs_seq",0); a = g(c,"rs_rayon",2); b = g(c,"rs_rayon",4); d = g(c,"rs_rayon",8)
    printf "| `%s` | %.3f | %.3f | %.3f | %.3f | %.2fx%s | %.2fx%s | %.2fx%s |\n", \
      c, s, a, b, d, s/a, verdict(s/a), s/b, verdict(s/b), s/d, verdict(s/d) > f
  }

  f = "logs/t_rayoncut.md"
  print "| config | Rust-seq | cutoff t=2 | t=4 | t=8 | seq/t2 | seq/t4 | seq/t8 |" > f
  print "|---|---:|---:|---:|---:|---:|---:|---:|" > f
  for (i = 1; i <= nc; i++) {
    c = cfgs[i]
    s = g(c,"rs_seq",0); a = g(c,"rs_cut",2); b = g(c,"rs_cut",4); d = g(c,"rs_cut",8)
    printf "| `%s` | %.3f | %.3f | %.3f | %.3f | %.2fx%s | %.2fx%s | %.2fx%s |\n", \
      c, s, a, b, d, s/a, verdict(s/a), s/b, verdict(s/b), s/d, verdict(s/d) > f
  }

  # --- cross-language headline, min-of-N throughout ---
  f = "logs/t_headline.md"
  print "| config | best WF | which | best Rust | which | WF/Rust | best WF vs Rust-seq |" > f
  print "|---|---:|---|---:|---|---:|---:|" > f
  nwf = split("wf_seq:0 wf_par:1 wf_par:2 wf_par:4 wf_par:8", WI, " ")
  nrs = split("rs_seq:0 rs_rayon:2 rs_rayon:4 rs_rayon:8 rs_cut:2 rs_cut:4 rs_cut:8", RI, " ")
  for (i = 1; i <= nc; i++) {
    c = cfgs[i]
    bw = 1e9; bwn = ""
    for (j = 1; j <= nwf; j++) {
      split(WI[j], q, ":"); t = g(c, q[1], q[2])
      if (t > 0 && t < bw) { bw = t; bwn = q[1] "/" q[2] }
    }
    br = 1e9; brn = ""
    for (j = 1; j <= nrs; j++) {
      split(RI[j], q, ":"); t = g(c, q[1], q[2])
      if (t > 0 && t < br) { br = t; brn = q[1] "/" q[2] }
    }
    rs = g(c, "rs_seq", 0)
    sub(/\/0$/, "", bwn); sub(/\/0$/, "", brn)
    printf "| `%s` | %.3f | %s | %.3f | %s | %.2fx%s | %.2fx%s |\n", \
      c, bw, bwn, br, brn, bw/br, verdict(bw/br), bw/rs, verdict(bw/rs) > f
  }

  f = "logs/t_spread.md"
  printf "N = %d repetitions per cell, %d cells.\n\n", reps_n, cells > f
  printf "Worst cell spread (max-min as a share of min): %.0f%% at %s.\n\n", worst, worstcell > f
  printf "Mean cell spread: %.0f%%. Cells whose spread exceeds 20%%: %d of %d. Exceeding 50%%: %d.\n", \
    tot_spread/cells, over20, cells, over50 > f
}
' logs/cells.tsv

# Full per-cell spread listing, worst first.
{
  print "| config | impl | threads | n | min (s) | median (s) | max (s) | spread |"
  print "|---|---|---:|---:|---:|---:|---:|---:|"
  tail -n +2 logs/cells.tsv | sort -t$'\t' -k8 -rn \
    | awk -F'\t' '{printf "| `%s` | %s | %s | %s | %s | %s | %s | %s%% |\n", $1,$2,$3,$4,$5,$6,$7,$8}'
} > logs/t_allspreads.md

print "tables written to logs/t_*.md"
