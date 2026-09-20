#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: observe.sh ARTIFACT_ROOT REPORT_ROOT" >&2
  exit 2
fi

artifact_root=$1
report_root=$2
baseline="$artifact_root/performance-baseline/records"
candidate="$artifact_root/performance-candidate/records"
baseline_sha=904028379508662fc988cb256896f90dbb4f24a82746c771b0e1cc6024412ce5
candidate_sha=7d0f38f2f61e109cd2260c8ba335beaff4eae44a47a0d7ec8e1b06bc954fbf68

mkdir -p "$report_root/logs" "$report_root/gdb"
command -v gdb >/dev/null
test -x "$baseline"
test -x "$candidate"
printf '%s  %s\n%s  %s\n' \
  "$baseline_sha" "$baseline" "$candidate_sha" "$candidate" \
  > "$report_root/expected-sha256.txt"
sha256sum --check "$report_root/expected-sha256.txt"
sha256sum "$baseline" "$candidate" > "$report_root/sha256-before.txt"

observe_one() {
  local arm=$1
  local width=$2
  local image=$3
  local payload_offset=$4
  local commands="$report_root/gdb/${arm}-w${width}.gdb"
  local log="$report_root/logs/${arm}-w${width}.log"

  cat > "$commands" <<EOF
set pagination off
set confirm off
set verbose off
set disable-randomization off
set \$ordinal = 0
set \$payload_offset = $payload_offset
break wf_record_result_release
commands
  silent
  set \$cell = (unsigned long long)\$rdi
  set \$payload = \$cell + \$payload_offset
  set \$word0 = *(unsigned long long *)\$cell
  printf "WF_ADDR arm=$arm width=$width ordinal=%llu cell=0x%llx cell_mod64=%llu payload=0x%llx payload_mod64=%llu word0=%llu\\n", \$ordinal, \$cell, \$cell & 63, \$payload, \$payload & 63, \$word0
  set \$ordinal = \$ordinal + 1
  if \$ordinal >= 8
    disable 1
  end
  continue
end
run
quit
EOF

  WF_WORKERS="$width" gdb --batch --quiet -nx -x "$commands" \
    --args "$image" measure "$arm" "$width" 0 > "$log" 2>&1
  test "$(grep -c '^WF_ADDR ' "$log")" -eq 6
  test "$(grep -c $'^records\t' "$log")" -eq 6
  grep -q 'exited normally' "$log"
}

for width in 1 2 4; do
  observe_one baseline "$width" "$baseline" 0
  observe_one candidate "$width" "$candidate" 8
done

awk '
  /^WF_ADDR / {
    arm = width = ordinal = cell = cell_mod = payload = payload_mod = word0 = ""
    for (i = 1; i <= NF; ++i) {
      split($i, pair, "=")
      if (pair[1] == "arm") arm = pair[2]
      else if (pair[1] == "width") width = pair[2]
      else if (pair[1] == "ordinal") ordinal = pair[2]
      else if (pair[1] == "cell") cell = pair[2]
      else if (pair[1] == "cell_mod64") cell_mod = pair[2]
      else if (pair[1] == "payload") payload = pair[2]
      else if (pair[1] == "payload_mod64") payload_mod = pair[2]
      else if (pair[1] == "word0") word0 = pair[2]
    }
    print arm "\t" width "\t" ordinal "\t" cell "\t" cell_mod "\t" payload "\t" payload_mod "\t" word0
  }
' "$report_root"/logs/*.log > "$report_root/addresses.tsv"

test "$(wc -l < "$report_root/addresses.tsv")" -eq 36
test "$(awk -F '\t' '$1 == "candidate" && $8 == 131072 { count += 1 } END { print count + 0 }' "$report_root/addresses.tsv")" -eq 18
awk -F '\t' '$1 == "baseline" && $3 != 0 { print $7 }' "$report_root/addresses.tsv" \
  | sort -n -u > "$report_root/baseline-kept-payload-residues.txt"
awk -F '\t' '$1 == "candidate" && $3 != 0 { print $7 }' "$report_root/addresses.tsv" \
  | sort -n -u > "$report_root/candidate-kept-payload-residues.txt"
awk -F '\t' '$1 == "baseline" && $3 == 0 { print $2 "=" $7 }' "$report_root/addresses.tsv" \
  | sort -n > "$report_root/baseline-warmup-payload-residues.txt"
awk -F '\t' '$1 == "candidate" && $3 == 0 { print $2 "=" $7 }' "$report_root/addresses.tsv" \
  | sort -n > "$report_root/candidate-warmup-payload-residues.txt"

baseline_count=$(wc -l < "$report_root/baseline-kept-payload-residues.txt")
candidate_count=$(wc -l < "$report_root/candidate-kept-payload-residues.txt")
verdict=INCONCLUSIVE
reason=mixed-residues
if [[ $baseline_count -eq 1 && $candidate_count -eq 1 ]]; then
  baseline_residue=$(cat "$report_root/baseline-kept-payload-residues.txt")
  candidate_residue=$(cat "$report_root/candidate-kept-payload-residues.txt")
  if [[ $candidate_residue -eq $(((baseline_residue + 8) % 64)) ]]; then
    verdict=STABLE_PLUS_8
    reason=unique-residues-match-layout-shift
  else
    reason=stable-residues-do-not-match-layout-shift
  fi
fi

sha256sum "$baseline" "$candidate" > "$report_root/sha256-after.txt"
cmp "$report_root/sha256-before.txt" "$report_root/sha256-after.txt"
{
  printf 'verdict=%s\nreason=%s\n' "$verdict" "$reason"
  printf 'baseline_requested_bytes=1048576\n'
  printf 'candidate_requested_bytes=1048584\n'
  printf 'processes=6\nrelease_observations=36\n'
  printf 'warmup_observations=6; archived but excluded from residue selection\n'
  printf 'kept_observations=30\n'
  printf 'timing_interpretation=discarded; pointer provenance only\n'
  printf 'baseline_kept_residues='
  paste -sd, "$report_root/baseline-kept-payload-residues.txt"
  printf 'candidate_kept_residues='
  paste -sd, "$report_root/candidate-kept-payload-residues.txt"
  printf 'baseline_warmup_residues_by_width='
  paste -sd, "$report_root/baseline-warmup-payload-residues.txt"
  printf 'candidate_warmup_residues_by_width='
  paste -sd, "$report_root/candidate-warmup-payload-residues.txt"
} > "$report_root/RESULT.txt"
cat "$report_root/RESULT.txt"
