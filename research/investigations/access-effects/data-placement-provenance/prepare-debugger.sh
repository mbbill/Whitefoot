#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: prepare-debugger.sh ARTIFACT_ROOT REPORT_ROOT" >&2
  exit 2
fi

artifact_root=$1
report_root=$2
baseline="$artifact_root/performance-baseline/records"
candidate="$artifact_root/performance-candidate/records"
ubuntu_sources=/etc/apt/sources.list.d/ubuntu.sources

mkdir -p "$report_root"
test "$(uname -s)" = Linux
test "$(uname -m)" = x86_64
test -x "$baseline"
test -x "$candidate"
test -f "$ubuntu_sources"

capture_allocator_identity() {
  local destination=$1
  {
    dpkg-query -W -f='package=${Package}\tversion=${Version}\tarch=${Architecture}\n' \
      libc6 libc-bin
    /usr/bin/ldd --version | sed -n '1p'
    for entry in "baseline:$baseline" "candidate:$candidate"; do
      local label=${entry%%:*}
      local image=${entry#*:}
      local mapped
      local mapped_real
      local interpreter
      local libc_build_id
      local interpreter_build_id
      mapped=$(/usr/bin/ldd "$image" | awk '$1 == "libc.so.6" { print $3; exit }')
      test -n "$mapped"
      mapped_real=$(readlink -f "$mapped")
      test -f "$mapped_real"
      interpreter=$(/usr/bin/readelf -l "$image" \
        | sed -n 's@.*Requesting program interpreter: \(.*\)]@\1@p')
      test -n "$interpreter"
      libc_build_id=$(/usr/bin/readelf -n "$mapped_real" \
        | sed -n 's/^[[:space:]]*Build ID: //p')
      interpreter_build_id=$(/usr/bin/readelf -n "$(readlink -f "$interpreter")" \
        | sed -n 's/^[[:space:]]*Build ID: //p')
      test -n "$libc_build_id"
      test -n "$interpreter_build_id"
      printf '%s_image=%s\n' "$label" "$image"
      printf '%s_libc_path=%s\n' "$label" "$mapped_real"
      printf '%s_libc_sha256=' "$label"
      sha256sum "$mapped_real" | awk '{ print $1 }'
      printf '%s_libc_build_id=%s\n' "$label" "$libc_build_id"
      printf '%s_interpreter_path=%s\n' "$label" "$(readlink -f "$interpreter")"
      printf '%s_interpreter_sha256=' "$label"
      sha256sum "$(readlink -f "$interpreter")" | awk '{ print $1 }'
      printf '%s_interpreter_build_id=%s\n' "$label" "$interpreter_build_id"
    done
  } > "$destination"
}

{
  printf 'workflow_run=%s\nworkflow_sha=%s\nworkflow_ref=%s\n' \
    "$GITHUB_RUN_ID" "$GITHUB_SHA" "$GITHUB_REF"
  printf 'prior_infrastructure_failure=35532641001; gdb unavailable; no image executed\n'
  printf 'departure=install GNU gdb only; official Ubuntu source; no recommends\n'
  date -u '+date=%Y-%m-%dT%H:%M:%SZ'
  uname -a
  nproc
  lscpu
  taskset -pc $$
  printf 'ubuntu_sources_sha256='
  sha256sum "$ubuntu_sources" | awk '{ print $1 }'
} > "$report_root/PROVENANCE.txt"
cp "$ubuntu_sources" "$report_root/ubuntu.sources"

capture_allocator_identity "$report_root/allocator-before.txt"
sudo apt-get -o Dir::Etc::sourcelist="$ubuntu_sources" \
  -o Dir::Etc::sourceparts=- update 2>&1 | tee "$report_root/apt-update.log"
sudo apt-get -o Dir::Etc::sourcelist="$ubuntu_sources" \
  -o Dir::Etc::sourceparts=- install -y --no-install-recommends gdb \
  2>&1 | tee "$report_root/apt-install.log"
command -v gdb >/dev/null
{
  gdb --version
  dpkg-query -W -f='package=${Package}\tversion=${Version}\tarch=${Architecture}\n' gdb
  apt-cache policy gdb
} > "$report_root/debugger.txt"
capture_allocator_identity "$report_root/allocator-after.txt"
cmp "$report_root/allocator-before.txt" "$report_root/allocator-after.txt"
printf 'allocator_identity=unchanged_after_gdb_install\n' >> "$report_root/PROVENANCE.txt"
