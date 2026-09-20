#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 2 ]]; then
  echo "usage: prepare-controlled.sh COMPUTE_ARTIFACT OUTPUT" >&2
  exit 2
fi
artifact=$1
output=$2
source_ll=$artifact/performance-candidate/records.ll
candidate=$artifact/performance-candidate
here=$(cd -- "$(dirname -- "$0")" && pwd)
expected_ll=8b60bc3da92f0914de1c90dbdeac5aaa99427136ed6093626894154a0ca7ac63
expected_candidate=7d0f38f2f61e109cd2260c8ba335beaff4eae44a47a0d7ec8e1b06bc954fbf68
expected_baseline=904028379508662fc988cb256896f90dbb4f24a82746c771b0e1cc6024412ce5
expected_runner=414de402069c32cca26b55ba05a0ea2c70f5c1f6bdb4b1dbbf632ea36a4b2707
expected_records_object=f7d4c41599ffec03a9f77d59bc3713037002bb5ce620c4effcb39dca8d003768
expected_oracle=3f38a448805d6c9376b19c7cab33328068cea053f6e212834259d725ce1c42ef
expected_identity=6d7afa2b0722edd5a48a8c1c61b44607e71e0aa17c4d58c5208b57b9a02c77d6
mkdir -p "$output"
test -f "$source_ll"
check_sha() {
  local expected=$1
  local file=$2
  test "$(sha256sum "$file" | awk '{print $1}')" = "$expected"
}
check_sha "$expected_identity" "$artifact/performance-results/identity.txt"
test "$(sha256sum "$source_ll" | awk '{print $1}')" = "$expected_ll"
test "$(sha256sum "$candidate/records" | awk '{print $1}')" = "$expected_candidate"
test "$(sha256sum "$artifact/performance-baseline/records" | awk '{print $1}')" = "$expected_baseline"
test "$(sha256sum "$candidate/runner.o" | awk '{print $1}')" = "$expected_runner"
test "$(sha256sum "$candidate/records.o" | awk '{print $1}')" = "$expected_records_object"
test "$(sha256sum "$candidate/records_oracle.o" | awk '{print $1}')" = "$expected_oracle"
check_sha b4d465213da606f2ea9e10bbab270f1f2dae0ffc9257306ff3e90d32be40c246 \
  "$artifact/performance-baseline/mandelbrot"
check_sha 904028379508662fc988cb256896f90dbb4f24a82746c771b0e1cc6024412ce5 \
  "$artifact/performance-baseline/records"
check_sha 3ba99b5d88818f3b6c0bd648f73f3facded50d2aa7b4cdff2a7d29efcb404410 \
  "$artifact/performance-baseline/fir"
check_sha 5cd90a0e80699800322394da7515db7bcbfbadc78be982470ac8ea49e3d99136 \
  "$artifact/performance-baseline/quadrature"
check_sha c63c2656c9a49d5b03c4d79a0d1f7b0abd6c01e4f2513d36abf6234be3bae67b \
  "$artifact/performance-baseline/stencil"
check_sha d71fe5141455b44fa2e8df9e51a9a27bad0f0164abdba4b410a54a91d7f46e3b \
  "$artifact/performance-candidate/mandelbrot"
check_sha 7d0f38f2f61e109cd2260c8ba335beaff4eae44a47a0d7ec8e1b06bc954fbf68 \
  "$artifact/performance-candidate/records"
check_sha 234a5805ffd6e4ecab517b140dff1a3856f9a9c96d3e97aa8de9676272e650cf \
  "$artifact/performance-candidate/fir"
check_sha 5cd90a0e80699800322394da7515db7bcbfbadc78be982470ac8ea49e3d99136 \
  "$artifact/performance-candidate/quadrature"
check_sha 89b9e80dea97bee12007382940f440845e2087b037733c3df0997d672d8b7912 \
  "$artifact/performance-candidate/stencil"

# Artifact download need not retain executable permission. Restore it only on
# the ten byte-qualified images that the comparison will invoke.
for arm in performance-baseline performance-candidate; do
  for kernel in mandelbrot records fir quadrature stencil; do
    chmod +x "$artifact/$arm/$kernel"
  done
done

# Rewrite exactly the allocation and release in the compiler-produced result
# cell instance. C-oracle allocations and every other runtime allocation stay
# on the artifact's ordinary allocator.
awk '
  /^define ptr @wf_box_array_filled\$instance\$34\(/ { in_alloc=1 }
  /^define i64 @wf_record_result_release\(/ { in_release=1 }
  in_alloc && /call ptr @malloc\(i64 %t1\)/ {
    sub(/@malloc/, "@wf_records_result_allocate"); allocations++
  }
  in_release && /call void @free\(ptr %v0\)/ {
    sub(/@free/, "@wf_records_result_release"); releases++
  }
  { print }
  /^}/ { in_alloc=0; in_release=0 }
  END { if (allocations != 1 || releases != 1) exit 9 }
' "$source_ll" > "$output/records-controlled.ll"
printf '\ndeclare noalias align 8 ptr @wf_records_result_allocate(i64)\ndeclare void @wf_records_result_release(ptr)\n' >> "$output/records-controlled.ll"

test "$(grep -c '^define ptr @wf_box_array_filled\$instance\$34(' "$output/records-controlled.ll")" -eq 1
test "$(grep -c 'call ptr @wf_box_array_filled\$instance\$34(' "$output/records-controlled.ll")" -eq 2
test "$(grep -c '^define i64 @wf_record_result_release(' "$output/records-controlled.ll")" -eq 1
test "$(grep -c 'call i64 @wf_record_result_release(' "$output/records-controlled.ll")" -eq 1
awk '
  /^define / { function_name=$0; sub(/^.*@/, "", function_name); sub(/\(.*/, "", function_name) }
  /call ptr @wf_box_array_filled\$instance\$34\(/ { print function_name }
' "$output/records-controlled.ll" | sort > "$output/allocation-callers.txt"
printf '%s\n' wf__par_seq_summarize_records wf_summarize_records \
  | sort > "$output/expected-allocation-callers.txt"
cmp "$output/expected-allocation-callers.txt" "$output/allocation-callers.txt"
awk '
  /^define / { function_name=$0; sub(/^.*@/, "", function_name); sub(/\(.*/, "", function_name) }
  /call i64 @wf_record_result_release\(/ { print function_name }
' "$output/records-controlled.ll" > "$output/release-callers.txt"
grep -Fxq wf_bench_records_release "$output/release-callers.txt"
test "$(wc -l < "$output/release-callers.txt")" -eq 1

clang -O2 -Wno-override-module -x ir -c "$output/records-controlled.ll" -o "$output/records-controlled.o"
clang -std=c11 -O2 -Wall -Wextra -Werror -Wpedantic -c \
  "$here/controlled-result-allocation.c" -o "$output/controlled-result-allocation.o"
objects=("$output/records-controlled.o" "$candidate/records_oracle.o" \
         "$candidate/runner.o" "$output/controlled-result-allocation.o")
while IFS= read -r object; do objects+=("$object"); done < <(find "$candidate/native" -type f -name '*.o' | sort)
clang -O2 "${objects[@]}" -pthread -lm -o "$output/records-controlled"

# All four timed residue arms execute this same ELF image.
sha256sum "$output/records-controlled" > "$output/CONTROLLED-IMAGE.sha256"
# The original textual malloc call has no explicit call attributes. Its fresh
# return guarantee is retained as `noalias`; the replacement guarantees eight-
# byte alignment, the maximum required by this result cell. Audit the hot worker
# itself byte-for-byte against the downloaded candidate after linking.
extract_worker() {
  local image=$1
  local symbols
  symbols=$(nm -S --defined-only "$image" \
    | awk '$4 ~ /^wf__par_chunk_[0-9]+$/ { print $1, $2, $4 }')
  test "$(printf '%s\n' "$symbols" | sed '/^$/d' | wc -l)" -eq 1
  local address size name
  read -r address size name <<< "$symbols"
  test "$size $name" = '00000000000001b8 wf__par_chunk_38'
  local start=$((16#$address))
  local stop=$((start + 440))
  # Bound disassembly by the ELF symbol's address and size. GNU objdump prints
  # inter-function alignment after the final instruction unless given a stop
  # address; those bytes are not part of the worker. AWK consumes the complete
  # bounded stream so pipefail cannot turn an early exit into SIGPIPE.
  objdump -d --start-address="$start" --stop-address="$stop" "$image" | awk '
    /^[[:space:]]*[0-9a-f]+ <[^>]+>:/ {
      inside = ($0 ~ /<wf__par_chunk_38>:/)
      next
    }
    inside && /^[[:space:]]*[0-9a-f]+:/ {
      line=$0; sub(/^[[:space:]]*[0-9a-f]+:[[:space:]]*/, "", line)
      count=split(line, fields, /[[:space:]]+/)
      for (i=1; i<=count && fields[i] ~ /^[0-9a-f][0-9a-f]$/; ++i) {
        printf "%s", fields[i]
        emitted += 1
      }
    }
    END { print ""; if (emitted != 440) exit 9 }
  '
}
extract_worker "$candidate/records" > "$output/original-worker.hex"
extract_worker "$output/records-controlled" > "$output/controlled-worker.hex"
test "$(wc -c < "$output/original-worker.hex")" -eq 881
test "$(wc -c < "$output/controlled-worker.hex")" -eq 881
cmp "$output/original-worker.hex" "$output/controlled-worker.hex"
printf 'worker_bytes=identical; bytes=440\n' > "$output/WORKER-AUDIT.txt"
for residue in 16 24 48 56; do
  wrapper="$output/records-r$residue"
  cat > "$wrapper" <<WRAPPER
#!/usr/bin/env bash
export WF_RECORDS_PAYLOAD_RESIDUE=$residue
exec "\$(dirname -- "\$0")/records-controlled" "\$@"
WRAPPER
  chmod +x "$wrapper"
done
