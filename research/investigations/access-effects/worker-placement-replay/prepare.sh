#!/usr/bin/env bash
# Construct records binaries whose only pairwise code-placement difference is
# the location of the already-emitted parallel worker body. This script never
# runs the images or performs timings.
set -euo pipefail

if [[ $# != 2 ]]; then
    echo 'usage: prepare.sh AC34_ARTIFACT_ROOT OUTPUT' >&2
    exit 2
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || {
    echo 'construction requires native Linux x86_64 binutils and linker' >&2
    exit 2
}

artifact=$(realpath "$1")
output=$(realpath -m "$2")
candidate="$artifact/performance-candidate"
source_ir="$candidate/records.ll"
worker=wf__par_chunk_38
body=${worker}_placement_body

for tool in /usr/bin/clang readelf objcopy objdump nm sha256sum perl; do
    command -v "$tool" >/dev/null || { echo "missing tool: $tool" >&2; exit 2; }
done
for path in "$source_ir" "$candidate/records.o" "$candidate/records_oracle.o" \
    "$candidate/runner.o"; do
    [[ -f $path ]] || { echo "missing input: $path" >&2; exit 2; }
done
[[ $(grep -c "^define i8 @$worker(" "$source_ir") == 1 ]] || {
    echo "expected exactly one $worker definition" >&2
    exit 2
}
! grep -Eq '^attributes #(1|2) =' "$source_ir" || {
    echo 'scratch attribute group numbers collide with the downloaded IR' >&2
    exit 2
}

mkdir -p "$output/build" "$output/images"
cp "$source_ir" "$output/build/records-placement.ll"

# Keep every original call site pointed at a fixed noinline trampoline. Move
# only the original worker definition into a named section. The records worker
# is leaf code, so its emitted bytes contain no placement-dependent external
# call relocations; the checks below prove the copied body itself is unchanged.
perl -0pi -e 's!define i8 \@wf__par_chunk_38\(i8 %v0, i64 %v1, i64 %v2, \{ ptr, i64 \} %v3, \{ ptr, i64 \} %v4, i64 %v5, i64 %v6, i64 %v7, ptr %v8\) #0 \{!define i8 \@wf__par_chunk_38(i8 %v0, i64 %v1, i64 %v2, { ptr, i64 } %v3, { ptr, i64 } %v4, i64 %v5, i64 %v6, i64 %v7, ptr %v8) #1 {\nentry:\n  %placement.result = tail call i8 \@wf__par_chunk_38_placement_body(i8 %v0, i64 %v1, i64 %v2, { ptr, i64 } %v3, { ptr, i64 } %v4, i64 %v5, i64 %v6, i64 %v7, ptr %v8)\n  ret i8 %placement.result\n}\n\ndefine i8 \@wf__par_chunk_38_placement_body(i8 %v0, i64 %v1, i64 %v2, { ptr, i64 } %v3, { ptr, i64 } %v4, i64 %v5, i64 %v6, i64 %v7, ptr %v8) #2 section ".wf_par_worker_body" {!' "$output/build/records-placement.ll"
printf '\nattributes #1 = { noinline nounwind }\nattributes #2 = { noinline "probe-stack"="inline-asm" }\n' >> "$output/build/records-placement.ll"
[[ $(grep -c "^define i8 @$body(" "$output/build/records-placement.ll") == 1 ]] || {
    echo 'IR rewrite did not produce exactly one body' >&2
    exit 2
}

# Removing only this module's unwind tables prevents moved-PC FDE bytes from
# becoming a second pairwise difference. The native runtime objects are reused
# byte-for-byte. This flag is identical in all placement arms.
/usr/bin/clang -O2 -Wno-override-module -fno-asynchronous-unwind-tables \
    -fno-unwind-tables -x ir -c "$output/build/records-placement.ll" \
    -o "$output/build/records-placement.o"

body_size_hex=$(nm -S --defined-only "$output/build/records-placement.o" |
    awk -v symbol="$body" '$4 == symbol { print $2 }')
[[ -n $body_size_hex ]] || { echo 'worker body symbol is missing' >&2; exit 2; }
body_size=$((16#$body_size_hex))
[[ $body_size == 440 ]] || {
    echo "worker body changed size: expected 440, got $body_size" >&2
    exit 2
}

# The input section contains only the body. Compare it to the precise symbol
# bytes from the downloaded candidate object before linking any variant.
objcopy --dump-section .wf_par_worker_body="$output/build/body.bin" \
    "$output/build/records-placement.o"
original_value_hex=$(nm -S --defined-only "$candidate/records.o" |
    awk -v symbol="$worker" '$4 == symbol { print $1 }')
original_size_hex=$(nm -S --defined-only "$candidate/records.o" |
    awk -v symbol="$worker" '$4 == symbol { print $2 }')
[[ -n $original_value_hex && -n $original_size_hex ]] || {
    echo 'downloaded worker symbol is missing' >&2
    exit 2
}
[[ $((16#$original_size_hex)) == body_size ]] || {
    echo 'downloaded and relocated worker sizes differ' >&2
    exit 2
}
objcopy --dump-section .text="$output/build/original-text.bin" "$candidate/records.o"
dd if="$output/build/original-text.bin" of="$output/build/original-body.bin" \
    bs=1 skip=$((16#$original_value_hex)) count="$body_size" status=none
cmp "$output/build/original-body.bin" "$output/build/body.bin"

native_objects=(
    native/ordinary_values.o
    native/sched/core.o
    native/sched/entry.o
    native/completion/runtime.o
    native/completion/file_adapter.o
    native/completion/bridge.o
    native/wf_floor.o
    native/sched/prim_host.o
    native/completion/wait_host.o
    native/completion/file_posix.o
    native/completion/linux_io_uring.o
    native/ordinary_values_ir.o
)
link_inputs=("$output/build/records-placement.o" "$candidate/records_oracle.o" "$candidate/runner.o")
for object in "${native_objects[@]}"; do
    [[ -f "$candidate/$object" ]] || { echo "missing input: $candidate/$object" >&2; exit 2; }
    link_inputs+=("$candidate/$object")
done

for pad in 0 16 32 48; do
    script="$output/build/placement-$pad.ld"
    cat > "$script" <<EOF
SECTIONS
{
  .wf_par_experiment ALIGN(64) :
  {
    __wf_par_experiment_start = .;
    . = __wf_par_experiment_start + $pad;
    KEEP(*(.wf_par_worker_body))
    . = __wf_par_experiment_start + 512;
    __wf_par_experiment_end = .;
  }
}
INSERT AFTER .text;
EOF
    /usr/bin/clang -O2 "${link_inputs[@]}" -pthread -lm \
        -Wl,--build-id=none -Wl,-T,"$script" -o "$output/images/records-pad$pad"
done

report="$output/STATIC-CHECKS.txt"
: > "$report"
printf 'artifact=%s\nworker=%s size=%s\n' \
    "$artifact" "$worker" "$body_size" >> "$report"

reference="$output/images/records-pad0"
for pad in 0 16 32 48; do
    image="$output/images/records-pad$pad"
    body_address_hex=$(nm -S --defined-only "$image" |
        awk -v symbol="$body" '$4 == symbol { print $1 }')
    entry_address_hex=$(nm -S --defined-only "$image" |
        awk -v symbol="$worker" '$4 == symbol { print $1 }')
    [[ -n $body_address_hex && -n $entry_address_hex ]] || {
        echo "placement symbols missing in pad$pad" >&2
        exit 2
    }
    body_address=$((16#$body_address_hex))
    ((body_address % 64 == pad)) || {
        echo "pad$pad body has residue $((body_address % 64))" >&2
        exit 2
    }
    printf 'pad=%s entry=0x%s body=0x%s residue=%s\n' \
        "$pad" "$entry_address_hex" "$body_address_hex" "$((body_address % 64))" >> "$report"

    # Dump the body out of the fixed-size output section and recheck it after
    # final relocations. This worker is leaf code, so equality must be exact.
    objcopy --dump-section .wf_par_experiment="$output/build/experiment-$pad.bin" "$image"
    dd if="$output/build/experiment-$pad.bin" of="$output/build/linked-body-$pad.bin" \
        bs=1 skip="$pad" count="$body_size" status=none
    cmp "$output/build/body.bin" "$output/build/linked-body-$pad.bin"
done

# Pairwise invariants. The experiment section has one fixed 512-byte extent,
# so every symbol except the moved body and linker markers must retain its
# address. Every ordinary allocated section must also remain byte-identical.
normalize_symbols() {
    readelf -Ws "$1" |
        awk -v body="$body" '$8 != body && $8 !~ /^__wf_par_experiment_/ { print $2, $3, $4, $5, $6, $7, $8 }'
}
normalize_symbols "$reference" > "$output/build/symbols-reference.txt"
readelf -lW "$reference" > "$output/build/program-headers-reference.txt"
readelf -SW "$reference" > "$output/build/section-headers-reference.txt"
sections=(.init .plt .plt.got .fini .rodata .eh_frame_hdr .eh_frame \
          .init_array .fini_array .data.rel.ro .dynamic .got .got.plt .data)
for pad in 16 32 48; do
    image="$output/images/records-pad$pad"
    normalize_symbols "$image" > "$output/build/symbols-$pad.txt"
    cmp "$output/build/symbols-reference.txt" "$output/build/symbols-$pad.txt"
    readelf -lW "$image" > "$output/build/program-headers-$pad.txt"
    readelf -SW "$image" > "$output/build/section-headers-$pad.txt"
    cmp "$output/build/program-headers-reference.txt" "$output/build/program-headers-$pad.txt"
    cmp "$output/build/section-headers-reference.txt" "$output/build/section-headers-$pad.txt"
    for section in "${sections[@]}"; do
        objcopy --dump-section "$section=$output/build/reference-${section#.}.bin" "$reference"
        objcopy --dump-section "$section=$output/build/pad$pad-${section#.}.bin" "$image"
        cmp "$output/build/reference-${section#.}.bin" "$output/build/pad$pad-${section#.}.bin"
    done
done

# The direct caller sees one fixed five-byte tail-jump in every arm. Its rel32
# displacement is the only unavoidable ordinary-.text byte difference: the
# target moves while the instruction stays put. Verify that exact shape and
# compare the whole section after zeroing only those four displacement bytes.
entry0=$(nm -S --defined-only "$reference" | awk -v symbol="$worker" '$4 == symbol { print $1, $2 }')
seq0=$(nm -S --defined-only "$reference" | awk '$4 == "wf__par_seq__par_chunk_38" { print $1, $2 }')
[[ ${entry0##* } == 0000000000000005 ]] || {
    echo "entry trampoline is not exactly five bytes: $entry0" >&2
    exit 2
}
text_address_hex=$(objdump -h "$reference" | awk '$2 == ".text" { print $4 }')
entry_address_hex=${entry0%% *}
[[ -n $text_address_hex && -n $entry_address_hex ]] || exit 2
entry_text_offset=$((16#$entry_address_hex - 16#$text_address_hex))
for pad in 16 32 48; do
    image="$output/images/records-pad$pad"
    [[ $(nm -S --defined-only "$image" | awk -v symbol="$worker" '$4 == symbol { print $1, $2 }') == "$entry0" ]]
    [[ $(nm -S --defined-only "$image" | awk '$4 == "wf__par_seq__par_chunk_38" { print $1, $2 }') == "$seq0" ]]
done
for pad in 0 16 32 48; do
    image="$output/images/records-pad$pad"
    disassembly="$output/build/trampoline-$pad.txt"
    objdump -d --disassemble="$worker" "$image" > "$disassembly"
    [[ $(grep -Ec '^[[:space:]]*[[:xdigit:]]+:[[:space:]]+e9 ' "$disassembly") == 1 ]]
    [[ $(grep -Ec "jmp.*<$body>" "$disassembly") == 1 ]]
    objcopy --dump-section .text="$output/build/text-$pad.bin" "$image"
    printf '\0\0\0\0' | dd of="$output/build/text-$pad.bin" bs=1 \
        seek=$((entry_text_offset + 1)) count=4 conv=notrunc status=none
done
for pad in 16 32 48; do
    cmp "$output/build/text-0.bin" "$output/build/text-$pad.bin"
done
cat "$output/build/trampoline-0.txt" >> "$report"
sha256sum "$output/images"/* >> "$report"
printf 'STATIC CHECKS PASSED; no image was executed.\n' >> "$report"
cat "$report"
