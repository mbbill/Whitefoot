#!/bin/sh
# Gate tests of the source adapter and its fail-closed library profile selection.
# Retire with baseline-entry.sh when the old entry boundary is no longer used.
set -eu
work=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-baseline-entry.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
cp programs/quadrature.wf "$work/current.wf"
sh baseline-entry.sh adapt ordinary "$work/current.wf" "$work/ordinary.wf"
cmp "$work/current.wf" "$work/ordinary.wf"
sh baseline-entry.sh adapt command "$work/current.wf" "$work/old.wf"
test "$(sh baseline-entry.sh profile "$work/current.wf")" = ordinary
test "$(sh baseline-entry.sh profile "$work/old.wf")" = command
sed 's/^command fn main/fn main/' "$work/old.wf" > "$work/reversed.wf"
cmp "$work/current.wf" "$work/reversed.wf"
sed '/^fn main/d' "$work/current.wf" > "$work/missing.wf"
cat "$work/current.wf" "$work/current.wf" > "$work/duplicate.wf"
for bad in missing duplicate; do
  if sh baseline-entry.sh adapt command "$work/$bad.wf" "$work/result.wf" > "$work/error" 2>&1; then
    echo "accepted $bad main" >&2; exit 1
  fi
  if sh baseline-entry.sh profile "$work/$bad.wf" > "$work/error" 2>&1; then
    echo "classified $bad main" >&2; exit 1
  fi
done
if sh baseline-entry.sh adapt unknown "$work/current.wf" "$work/result.wf" > "$work/error" 2>&1; then
  echo 'accepted unknown profile' >&2; exit 1
fi
# An ordinary profile with only the old four components must not fall back.
mkdir -p "$work/native/sched"
for unit in core prim_host entry; do : > "$work/native/sched/$unit.c"; done
: > "$work/native/wf_floor.c"
if ${MAKE:-make} --no-print-directory baseline-check WF_B_INTERFACE=ordinary \
  WF_B_SCHED_DIR="$work/native/sched" WF_B_FLOOR="$work/native/wf_floor.c" \
  WF_B_WFC=/usr/bin/true > "$work/error" 2>&1; then
  echo 'accepted incomplete ordinary library' >&2; exit 1
fi
grep -q 'missing baseline input' "$work/error"
for bad in '%' 'ordinary command' '' 'ordinary '; do
  if ${MAKE:-make} --no-print-directory baseline-check "WF_B_INTERFACE=$bad" > "$work/error" 2>&1; then
    echo "accepted invalid interface [$bad]" >&2; exit 1
  fi
  grep -q 'WF_B_INTERFACE must be' "$work/error"
done
echo 'baseline entry and incomplete-library checks passed'
