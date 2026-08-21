# Reduces logs/results.tsv to one row per (configuration, implementation) cell:
# n, min, median, max, spread as a percentage of the minimum.
BEGIN { FS = "\t"; OFS = "\t" }
NR == 1 { next }
{
  cfg = $2 "_d" $3 "_w" $4
  impl = $6
  th = $7
  key = cfg SUBSEP impl SUBSEP th
  n[key]++
  v[key, n[key]] = $8 + 0
  if (!(key in seen)) { seen[key] = 1; order[++k] = key }
  if ($9 != 0) exits[key] = exits[key] " " $9
}
END {
  print "config", "impl", "threads", "n", "min", "median", "max", "spread_pct"
  for (i = 1; i <= k; i++) {
    key = order[i]
    m = n[key]
    for (a = 1; a <= m; a++) s[a] = v[key, a]
    # insertion sort
    for (a = 2; a <= m; a++) {
      x = s[a]; b = a - 1
      while (b >= 1 && s[b] > x) { s[b + 1] = s[b]; b-- }
      s[b + 1] = x
    }
    lo = s[1]; hi = s[m]
    if (m % 2 == 1) med = s[(m + 1) / 2]
    else med = (s[m / 2] + s[m / 2 + 1]) / 2
    split(key, part, SUBSEP)
    printf "%s\t%s\t%s\t%d\t%.4f\t%.4f\t%.4f\t%.1f\n", part[1], part[2], part[3], m, lo, med, hi, (hi - lo) / lo * 100
  }
}
