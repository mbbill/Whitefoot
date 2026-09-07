#!/usr/bin/env bash
# CPU-only external control for the existing par_layout.wf workload.
# Owned by io-model/SCHEDULER-EXPERIMENT.md; remove with that comparison.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=${ROOT:-$(cd "$HERE/../../.." && pwd)}
OUT=${OUT:-${WHITEFOOT_SCRATCH_ROOT:-$HOME/do_not_scan}/whitefoot-rayon-bench}
CLANG=${CLANG:-/usr/bin/clang}
WFC=${WFC:-$ROOT/compiler/target/gate/whitefootc}
ROUNDS=${ROUNDS:-5}
WARMUP=${WARMUP:-1}
CALIBRATION_ROUNDS=${CALIBRATION_ROUNDS:-3}
BATCHES=${BATCHES:-1}
RAYON_THREADS=${RAYON_THREADS:-"1 2 4"}
RAYON_GRAINS=${RAYON_GRAINS:-"1 4 16"}
RAYON_PROFILE=${RAYON_PROFILE:-0}

RESOURCE_CONTROLS=${RESOURCE_CONTROLS:-0}
CPU_PHASE_TRACE=${CPU_PHASE_TRACE:-0}
EXPECTED='420a993efa7437a1 41fa962893d45299'
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$OUT/cargo-target}

bounded_integer() {
    [[ $2 =~ ^[0-9]+$ ]] && (( 10#$2 >= $3 && 10#$2 <= $4 )) || {
        echo "rayon-bench: $1 must be $3..$4" >&2
        exit 2
    }
}
bounded_integer ROUNDS "$ROUNDS" 1 128
bounded_integer WARMUP "$WARMUP" 0 128
bounded_integer CALIBRATION_ROUNDS "$CALIBRATION_ROUNDS" 1 128
bounded_integer BATCHES "$BATCHES" 1 16
bounded_integer RAYON_PROFILE "$RAYON_PROFILE" 0 1

bounded_integer RESOURCE_CONTROLS "$RESOURCE_CONTROLS" 0 6
if [[ $RESOURCE_CONTROLS != 0 && $RAYON_PROFILE == 1 ]]; then
    echo 'rayon-bench: resource controls and perf captures are separate experiments' >&2
    exit 2
fi
bounded_integer CPU_PHASE_TRACE "$CPU_PHASE_TRACE" 0 1
if [[ $CPU_PHASE_TRACE == 1 && $RESOURCE_CONTROLS != 3 ]]; then
    echo 'rayon-bench: coarse phase traces require the startup control panel' >&2
    exit 2
fi
if [[ $RESOURCE_CONTROLS == 2 || $RESOURCE_CONTROLS == 3 || $RESOURCE_CONTROLS == 4 || $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
    # The default row must really inherit no override; the disabled row sets
    # the existing control only in its own child process.
    unset WF_IO_NO_NATIVE_RING
fi
read -r -a threads <<< "$RAYON_THREADS"
read -r -a grains <<< "$RAYON_GRAINS"
(( ${#threads[@]} > 0 && ${#grains[@]} > 0 )) || exit 2
for width in "${threads[@]}"; do bounded_integer THREADS "$width" 1 256; done
for grain in "${grains[@]}"; do bounded_integer GRAIN "$grain" 1 64; done

{
    git -C "$ROOT" rev-parse HEAD
    git -C "$ROOT" status --short
    uname -a
    rustc -vV
    cargo -V
    printf 'runner_compiler=%s\n' "$CLANG"
    "$CLANG" --version
    # whitefootc.rs::clang_executable fixes the non-Windows native linker to
    # this path; CLANG above controls only this experiment's C timing runner.
    printf 'wf_native_link_compiler=/usr/bin/clang (compiler-owned selection)\n'
    /usr/bin/clang --version
    printf 'RUSTFLAGS=%s\nCARGO_ENCODED_RUSTFLAGS=%s\n' \
        "${RUSTFLAGS:-}" "${CARGO_ENCODED_RUSTFLAGS:-}"
    if [[ $RESOURCE_CONTROLS == 0 ]]; then
        printf 'threads=%s grains=%s batches=%s calibration=%s confirmation=%s warmup=%s\n' \
            "$RAYON_THREADS" "$RAYON_GRAINS" "$BATCHES" "$CALIBRATION_ROUNDS" "$ROUNDS" "$WARMUP"
    elif [[ $RESOURCE_CONTROLS == 1 ]]; then
        printf 'threads=4 grain=4 batches=1,4,16 stacks=12,1100 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    elif [[ $RESOURCE_CONTROLS == 2 ]]; then
        printf 'threads=4 grain=4 batches=1,16 stacks=12 no_native_ring=unset,1 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    elif [[ $RESOURCE_CONTROLS == 3 ]]; then
        printf 'threads=4 grain=4 batches=1,16 stacks=12 init_used_lanes=0,1 compact_stacks=0 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    elif [[ $RESOURCE_CONTROLS == 4 ]]; then
        printf 'threads=4 grain=4 batches=1,16 stacks=12 idle_spin_rounds=256 idle_yield_rounds=16,0 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    elif [[ $RESOURCE_CONTROLS == 5 ]]; then
        printf 'threads=4 grain=4 batches=1,16 stacks=12 tree_build=parallel,sequential idle_spin_rounds=256 idle_yield_rounds=16 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    fi
    if [[ $RESOURCE_CONTROLS == 6 ]]; then
        printf 'threads=4 grain=4 batches=1,16 stacks=12 layout_fork_depth=full,4 tree_build=parallel idle_spin_rounds=256 idle_yield_rounds=16 confirmation=%s warmup=%s calibration=none\n' \
            "$ROUNDS" "$WARMUP"
    fi
    printf 'timing=whole-process; Rayon pool created once, install once; no concurrent I/O\n'
    printf 'resource_controls=%s\n' "$RESOURCE_CONTROLS"
    if [[ $RESOURCE_CONTROLS == 4 || $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
        echo 'runner_rusage=1; raw rows include wait4 context switches and peak RSS in KiB'
    fi
    if [[ $(uname -s) == Linux ]]; then
        lscpu
        awk '/Cpus_allowed_list:/ {print}' /proc/self/status
    else
        sysctl hw.model hw.physicalcpu hw.logicalcpu hw.memsize
    fi
} > "$OUT/host.txt"
cargo build --release --locked --offline --manifest-path "$HERE/rayon-baseline/Cargo.toml"
cp "$CARGO_TARGET_DIR/release/whitefoot-rayon-baseline" "$OUT/rust-layout"
cp "$HERE/rayon-baseline/Cargo.lock" "$OUT/Cargo.lock"
runner_command=("$CLANG" -std=c11 -O2 -Wall -Wextra -Werror)
if [[ $RESOURCE_CONTROLS == 4 || $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then runner_command+=(-DWF_BENCH_RUSAGE=1); fi
runner_command+=("$HERE/runner.c" -o "$OUT/runner")
"${runner_command[@]}"
if [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
    printf '%q ' "${runner_command[@]}" > "$OUT/runner.command"
    printf '\n' >> "$OUT/runner.command"
fi
if [[ ! -x $WFC ]]; then
    echo 'rayon-bench: build whitefootc or set WFC to the compiler for this revision' >&2
    exit 2
fi
"$WFC" --no-overlap "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-seq"
"$WFC" --par "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-par"
shasum -a 256 "$WFC" "$OUT/wf-seq" "$OUT/wf-par" "$OUT/rust-layout" "$OUT/runner" \
    "$ROOT/tests/programs/par_layout.wf" "$ROOT/compiler/src/bin/whitefootc.rs" \
    "$HERE/rayon-baseline/src/main.rs" "$HERE/rayon-baseline/Cargo.toml" \
    "$HERE/rayon-baseline/Cargo.lock" >> "$OUT/host.txt"

if [[ $RESOURCE_CONTROLS != 0 ]]; then
    # Experiments 45/47/50/59/61/63 freeze the earlier four-worker calibration. No
    # panel tunes against its confirmation samples or changes runtime sources.
    if [[ $RESOURCE_CONTROLS == 1 ]]; then
        resource_batches=(1 4 16)
        resource_forms=(12 1100)
        printf 'resource_panel=workers4; Rayon grain4; batches1,4,16; WF_STACKS12,1100\n' >> "$OUT/host.txt"
    elif [[ $RESOURCE_CONTROLS == 2 ]]; then
        resource_batches=(1 16)
        resource_forms=(default disabled)
        printf 'resource_panel=workers4; Rayon grain4; batches1,16; WF_STACKS12; WF_IO_NO_NATIVE_RING unset/1\n' >> "$OUT/host.txt"
    else
        resource_batches=(1 16)
        if [[ $RESOURCE_CONTROLS == 3 ]]; then
            resource_forms=(ordinary manual lanes)
            control_name=startup
            candidate_name=lanes
            candidate_defines=(-DWF_SCHED_INIT_USED_LANES=1 -DWF_SCHED_COMPACT_STACKS=0)
            printf 'resource_panel=workers4; Rayon grain4; batches1,16; WF_STACKS12; ordinary/manual-default/manual-used-lanes\n' >> "$OUT/host.txt"
        elif [[ $RESOURCE_CONTROLS == 4 ]]; then
            resource_forms=(ordinary manual idle0)
            control_name=idle
            candidate_name=idle0
            candidate_defines=(-DWF_SCHED_IDLE_YIELD_ROUNDS=0u -DWF_SCHED_INIT_USED_LANES=0 -DWF_SCHED_COMPACT_STACKS=0)
            printf 'resource_panel=workers4; Rayon grain4; batches1,16; WF_STACKS12; ordinary/manual-default/manual-idle-yield0; all other yield sites unchanged\n' >> "$OUT/host.txt"
        elif [[ $RESOURCE_CONTROLS == 5 ]]; then
            resource_forms=(ordinary manual seqbuild)
            control_name=build
            candidate_name=seqbuild
            candidate_defines=(-DWF_SCHED_IDLE_YIELD_ROUNDS=16u -DWF_SCHED_INIT_USED_LANES=0 -DWF_SCHED_COMPACT_STACKS=0)
            printf 'resource_panel=workers4; Rayon grain4; batches1,16; WF_STACKS12; ordinary/manual-default/manual-sequential-build; lazy worker start moves to first layout\n' >> "$OUT/host.txt"
        fi
        if [[ $RESOURCE_CONTROLS == 6 ]]; then
            resource_forms=(ordinary manual grain4)
            control_name=grain
            candidate_name=grain4
            candidate_defines=(-DWF_SCHED_IDLE_YIELD_ROUNDS=16u -DWF_SCHED_INIT_USED_LANES=0 -DWF_SCHED_COMPACT_STACKS=0)
            printf 'resource_panel=workers4; Rayon grain4; batches1,16; WF_STACKS12; ordinary/manual-default/manual-four-fork-levels; default parallel build and lazy startup\n' >> "$OUT/host.txt"
        fi
        completion_flags='-std=c11 -O2 -g -Wall -Wextra -Werror -Wpedantic -pthread'
        if [[ $RESOURCE_CONTROLS != 5 && $RESOURCE_CONTROLS != 6 ]]; then completion_flags+=" ${candidate_defines[*]}"; fi
        if ! make -C "$ROOT/compiler" completion-test CC=/usr/bin/clang \
            COMPLETION_TMP="$OUT/$control_name-check" \
            COMPLETION_BASE_CFLAGS="$completion_flags" \
            > "$OUT/$control_name-check.log" 2>&1; then
            cat "$OUT/$control_name-check.log"
            exit 1
        fi
        backend="$ROOT/compiler/src/backend"
        "$WFC" --par --emit-llvm "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-par.ll"
        candidate_ir="$OUT/wf-par.ll"
        if [[ $RESOURCE_CONTROLS == 5 ]]; then
            # A workload control, never a compiler rule: select the existing
            # sequential clone at exactly the main function's one build call.
            candidate_ir="$OUT/wf-seqbuild.ll"
            awk '
                /^define .* @wf_main\(/ {mains++; inside=1}
                /^define .* @wf_build\(/ {builds++}
                /^define .* @wf__par_seq_build\(/ {clones++}
                inside && /call ptr @wf__par_seq_build\(/ {bad=1}
                inside && /call ptr @wf_build\(/ {
                    calls+=gsub(/call ptr @wf_build\(/, "call ptr @wf__par_seq_build(")
                }
                {print}
                /^}/ {inside=0}
                END {exit !(mains==1 && builds==1 && clones==1 && calls==1 && !bad)}
            ' "$OUT/wf-par.ll" > "$candidate_ir"
            # Reverse only that same scoped substitution, then compare every
            # byte, including all hot layout definitions and unrelated calls.
            awk '
                /^define .* @wf_main\(/ {inside=1}
                inside {sub(/call ptr @wf__par_seq_build\(/, "call ptr @wf_build(")}
                {print}
                /^}/ {inside=0}
            ' "$candidate_ir" > "$OUT/ir-roundtrip.ll"
            cmp "$OUT/wf-par.ll" "$OUT/ir-roundtrip.ll"
            rm "$OUT/ir-roundtrip.ll"
            diff -u "$OUT/wf-par.ll" "$candidate_ir" > "$OUT/build-ir.diff" || [[ $? == 1 ]]
            echo 'main_build_call=one exact replacement; all other IR bytes unchanged' > "$OUT/build-ir-check.txt"
        fi
        if [[ $RESOURCE_CONTROLS == 6 ]]; then
            candidate_ir="$OUT/wf-grain4.ll"
            # These bounded workload copies change no frame ABI or runtime.
            # Reverse each copy to its source template, then restore the full
            # original module independently from the saved candidate file.
            cat > "$OUT/grain-transform.awk" <<'GRAIN_TRANSFORM'
function need(ok, why) {if (!ok) {print "grain transform: " why > "/dev/stderr"; exit 1}}
function replace(text, from, to, expected, pos, result, count) {
    while ((pos=index(text, from))) {
        result=result substr(text,1,pos-1) to
        text=substr(text,pos+length(from)); count++
    }
    need(count==expected, "replacement count for " from)
    return result text
}
function hot(kind, depth) {return "wf__grain4_" kind "_d" depth}
function thunk(kind, depth) {return hot(kind,depth) "_thunk"}
function child(kind, depth) {
    return depth==1 ? "wf__par_seq_" kind : hot(kind,depth-1)
}
function forward(kind, depth, is_thunk, original, copy) {
    original=is_thunk ? original_thunk[kind] : "wf_" kind
    copy=body[original]
    copy=replace(copy, "@" original "(", "@" (is_thunk ? thunk(kind,depth) : hot(kind,depth)) "(", is_thunk ? 1 : 3)
    if (!is_thunk) {
        copy=replace(copy, "call double @" hot(kind,depth) "(", "call double @" child(kind,depth) "(", 2)
        copy=replace(copy, "ptr @" original_thunk[kind] ")", "ptr @" thunk(kind,depth) ")", 1)
    } else {
        copy=replace(copy, "call double @wf_" kind "(", "call double @" child(kind,depth) "(", 1)
    }
    return copy
}
function reverse(kind, depth, is_thunk, original, copy) {
    original=is_thunk ? original_thunk[kind] : "wf_" kind
    copy=body[is_thunk ? thunk(kind,depth) : hot(kind,depth)]
    copy=replace(copy, "call double @" child(kind,depth) "(", "call double @wf_" kind "(", is_thunk ? 1 : 2)
    copy=replace(copy, "@" (is_thunk ? thunk(kind,depth) : hot(kind,depth)) "(", "@" original "(", 1)
    if (!is_thunk) copy=replace(copy, "ptr @" thunk(kind,depth) ")", "ptr @" original_thunk[kind] ")", 1)
    need(copy==body[original], "reversed template " original " depth " depth)
}
BEGIN {
    kinds[1]="layout"; kinds[2]="layout_banded"
    original_thunk["layout"]="wf__par_thunk_1"
    original_thunk["layout_banded"]="wf__par_thunk_2"
    marker="; grain4 specialization begins\n"
}
{
    all=all $0 "\n"
    if ($0=="; grain4 specialization begins") {region++; need(region==1,"one specialization region")}
    if (region) suffix=suffix $0 "\n"; else prefix=prefix $0 "\n"
    if (/^define /) {
        need(owner=="", "nested definition")
        owner=$0; sub(/^.*@/,"",owner); sub(/\(.*/,"",owner)
        seen[owner]++; need(seen[owner]==1,"unique definition " owner)
    }
    if (owner!="") body[owner]=body[owner] $0 "\n"
    if (/^}/) owner=""
}
END {
    # An earlier failure exits through END too; no output is accepted unless
    # the caller sees success and the separate full-module cmp also passes.
    need(owner=="" && seen["wf_main"]==1,"complete main")
    for (k=1;k<=2;k++) {
        kind=kinds[k]
        need(seen["wf_" kind]==1 && seen["wf__par_seq_" kind]==1 && seen[original_thunk[kind]]==1,"source templates " kind)
    }
    if (mode=="generate") {
        need(region==0 && !index(all,"@wf__grain4_"),"unspecialized input")
        main=body["wf_main"]
        for (k=1;k<=2;k++) main=replace(main,"call double @wf_" kinds[k] "(","call double @" hot(kinds[k],4) "(",1)
        printf "%s", replace(all,body["wf_main"],main,1) marker
        for (k=1;k<=2;k++) for (depth=1;depth<=4;depth++) {
            printf "%s\n%s\n", forward(kinds[k],depth,0), forward(kinds[k],depth,1)
        }
    } else {
        need(mode=="verify" && region==1,"verification mode/region")
        expected=marker
        for (k=1;k<=2;k++) for (depth=1;depth<=4;depth++) {
            reverse(kinds[k],depth,0); reverse(kinds[k],depth,1)
            expected=expected body[hot(kinds[k],depth)] "\n" body[thunk(kinds[k],depth)] "\n"
        }
        need(suffix==expected,"exact sixteen-copy suffix")
        main=body["wf_main"]
        for (k=1;k<=2;k++) main=replace(main,"call double @" hot(kinds[k],4) "(","call double @wf_" kinds[k] "(",1)
        printf "%s", replace(prefix,body["wf_main"],main,1)
    }
}
GRAIN_TRANSFORM
            awk -v mode=generate -f "$OUT/grain-transform.awk" "$OUT/wf-par.ll" > "$candidate_ir"
            awk -v mode=verify -f "$OUT/grain-transform.awk" "$candidate_ir" > "$OUT/grain-roundtrip.ll"
            cmp "$OUT/wf-par.ll" "$OUT/grain-roundtrip.ll"
            rm "$OUT/grain-roundtrip.ll"
            diff -u "$OUT/wf-par.ll" "$candidate_ir" > "$OUT/grain-ir.diff" || [[ $? == 1 ]]
            echo 'main_layout_calls=two replacements; sixteen reversed templates; all original IR bytes restored' > "$OUT/grain-ir-check.txt"
        fi
        manual_sources=()
        for unit in wf_floor.c sched/core.c sched/prim_host.c sched/entry.c \
            completion/runtime.c completion/wait_host.c completion/file_adapter.c \
            completion/file_posix.c completion/bridge.c completion/linux_io_uring.c; do
            manual_sources+=("$backend/$unit")
        done
        # Mirror whitefootc's source order, C11/-pthread/-O2/-lm and stdin IR.
        # Storage macros spell the existing defaults. The idle panel varies
        # only the existing yield-round override in its candidate link.
        for used in 0 1; do
            manual_command=(/usr/bin/clang -std=c11 -pthread -I "$backend" \
                -I "$backend/completion")
            if [[ $RESOURCE_CONTROLS == 3 ]]; then
                manual_command+=(-DWF_SCHED_COMPACT_STACKS=0 "-DWF_SCHED_INIT_USED_LANES=$used")
            elif [[ $RESOURCE_CONTROLS == 4 ]]; then
                manual_command+=(-DWF_SCHED_COMPACT_STACKS=0 -DWF_SCHED_INIT_USED_LANES=0 \
                    "-DWF_SCHED_IDLE_YIELD_ROUNDS=$((16 * (1 - used)))u")
            else
                manual_command+=("${candidate_defines[@]}")
            fi
            for source in "${manual_sources[@]}"; do manual_command+=(-x c "$source"); done
            manual_command+=(-x ir - -Wno-override-module -O2 -lm -o "$OUT/wf-manual-$used")
            printf '%q ' "${manual_command[@]}" > "$OUT/manual-$used.command"
            manual_ir="$OUT/wf-par.ll"
            if [[ ( $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ) && $used == 1 ]]; then manual_ir="$candidate_ir"; fi
            printf '< %q\n' "$manual_ir" >> "$OUT/manual-$used.command"
            "${manual_command[@]}" < "$manual_ir"
            shasum -a 256 "$OUT/wf-manual-$used" >> "$OUT/host.txt"
            WF_WORKERS=4 WF_STACKS=12 WF_SCHED_REPORT=0 "$OUT/wf-manual-$used" > "$OUT/manual-$used.out"
            printf '%s\n' "$EXPECTED" | cmp - "$OUT/manual-$used.out"
            if [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
                qualified_args=("$OUT/wf-manual-$used")
                for ((batch=1; batch<16; batch++)); do qualified_args+=(batch); done
                WF_WORKERS=4 WF_STACKS=12 WF_SCHED_REPORT=0 "${qualified_args[@]}" > "$OUT/manual-$used-b16.out"
                printf '%s\n' "$EXPECTED" | cmp - "$OUT/manual-$used-b16.out"
            fi
        done
        if cmp -s "$OUT/wf-par" "$OUT/wf-manual-0"; then
            echo 'manual_default_binary=byte-identical to ordinary compiler output' > "$OUT/link-comparison.txt"
        else
            echo 'manual_default_binary=differs; retain both controls and inspect code/layout before attribution' > "$OUT/link-comparison.txt"
        fi
        if [[ $(uname -s) == Linux ]]; then
            objdump --version > "$OUT/objdump-version.txt"
            for form in wf-par wf-manual-0 wf-manual-1; do
                objdump -t "$OUT/$form" > "$OUT/$form.symbols"
                objdump -d "$OUT/$form" > "$OUT/$form.disassembly"
            done
            if [[ $RESOURCE_CONTROLS == 5 ]]; then
                # Decode the actual ELF call instructions. Optimizers may
                # inline wf_main into the ABI body, so inspect both owners.
                for used in 0 1; do
                    awk -v candidate="$used" '
                        /^[0-9a-f]+ <[^>]+>:/ {inside=($0 ~ /<(wf_main|wf__main_body)>:/)}
                        inside && /[[:space:]]callq?[[:space:]].*<wf_build>/ {parallel++}
                        inside && /[[:space:]]callq?[[:space:]].*<wf__par_seq_build>/ {sequential++}
                        END {print "parallel=" parallel+0, "sequential=" sequential+0;
                            exit !(candidate ? parallel==0 && sequential>0 : parallel==1)}
                    ' "$OUT/wf-manual-$used.disassembly" > "$OUT/build-call-$used.txt"
                    for symbol in wf_layout wf_layout_banded; do
                        objdump -d --no-show-raw-insn --disassemble="$symbol" "$OUT/wf-manual-$used" \
                            > "$OUT/$symbol-$used.disassembly"
                        # Preserve mnemonics, operands, target names and
                        # function-relative offsets; remove only relocated
                        # instruction addresses and resolved RIP displacements.
                        awk '
                            /^[[:space:]]*[0-9a-f]+:/ {
                                sub(/^[[:space:]]*[0-9a-f]+:[[:space:]]*/, "")
                                gsub(/-?0x[0-9a-f]+\(%rip\)/, "RIP(%rip)")
                                gsub(/[[:space:]][0-9a-f]+ </, " <")
                                print; instructions++
                            }
                            END {exit !(instructions>0)}
                        ' "$OUT/$symbol-$used.disassembly" > "$OUT/$symbol-$used.normalized"
                    done
                done
                : > "$OUT/hot-layout-check.txt"
                for symbol in wf_layout wf_layout_banded; do
                    if cmp -s "$OUT/$symbol-0.normalized" "$OUT/$symbol-1.normalized"; then
                        printf '%s=identical offset-normalized decoded instructions\n' "$symbol" >> "$OUT/hot-layout-check.txt"
                    else
                        printf '%s=differs; retained code requires attribution review\n' "$symbol" >> "$OUT/hot-layout-check.txt"
                    fi
                done
            fi
            if [[ $RESOURCE_CONTROLS == 6 ]]; then
                # Retain actual call sites, including copies that LLVM inlines
                # into another depth. The observed publication totals below
                # qualify the executed grain; names alone cannot do that.
                for used in 0 1; do
                    awk -v candidate="$used" '
                        /^[0-9a-f]+ <[^>]+>:/ {
                            owner=$0; sub(/^.*</,"",owner); sub(/>:.*/,"",owner)
                        }
                        /[[:space:]]callq?[[:space:]].*<(wf__grain4_|wf_layout|wf__par_seq_layout|wf__par_acquire_lane|wf__par_publish)/ {
                            print owner ": " $0
                            if (owner ~ /^(wf_main|wf__main_body|wf__grain4_)/) {
                                if ($0 ~ /<wf__grain4_/) grain++
                                if ($0 ~ /<wf__par_seq_layout/) sequential++
                                if ($0 ~ /<wf__par_publish>/) publish++
                            }
                        }
                        END {exit !(candidate ? grain>0 && sequential>0 && publish>0 : grain==0)}
                    ' "$OUT/wf-manual-$used.disassembly" > "$OUT/grain-calls-$used.txt"
                done
            fi
        elif [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
            echo 'ELF call and hot-layout checks require the native Linux cohort' > "$OUT/hot-layout-check.txt"
        fi
        shasum -a 256 "$OUT/wf-par.ll" "${manual_sources[@]}" >> "$OUT/host.txt"
    fi
    resource_settings() {
        resource_binary="$OUT/wf-par"
        if [[ $RESOURCE_CONTROLS == 1 ]]; then
            resource_label="wf.w4.s$1"
            resource_environment="WF_WORKERS=4,WF_STACKS=$1"
        elif [[ $RESOURCE_CONTROLS == 2 ]]; then
            resource_label="wf.w4.s12.r$1"
            resource_environment='WF_WORKERS=4,WF_STACKS=12'
            if [[ $1 == disabled ]]; then resource_environment+=',WF_IO_NO_NATIVE_RING=1'; fi
        else
            resource_label="wf.w4.s12.$1"
            resource_environment='WF_WORKERS=4,WF_STACKS=12'
            if [[ $1 == manual ]]; then resource_binary="$OUT/wf-manual-0"; fi
            if [[ $1 == lanes || $1 == idle0 || $1 == seqbuild || $1 == grain4 ]]; then resource_binary="$OUT/wf-manual-1"; fi
        fi
    }
    printf 'resource_budget=WF main+3 workers; Rayon 4 workers+sleeping caller; no CPU affinity\n' >> "$OUT/host.txt"
    : > "$OUT/resource.plan"
    for batches in "${resource_batches[@]}"; do
        printf 'rayon.w4.g4.b%s\t\t%s\trayon\t4\t4\t%s\n' \
            "$batches" "$OUT/rust-layout" "$batches" >> "$OUT/resource.plan"
        for form in "${resource_forms[@]}"; do
            resource_settings "$form"
            printf '%s.b%s\t%s,WF_SCHED_REPORT=0\t%s' \
                "$resource_label" "$batches" "$resource_environment" "$resource_binary" >> "$OUT/resource.plan"
            for ((batch=1; batch<batches; batch++)); do printf '\tbatch' >> "$OUT/resource.plan"; done
            printf '\n' >> "$OUT/resource.plan"
        done
    done
    if [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
        # Snapshot launched inputs before timing, then verify them again after
        # the separate observations. The retained compiler is rehashable too.
        cp "$WFC" "$OUT/whitefootc"
        : > "$OUT/$control_name-source.sha256"
        for source in "${manual_sources[@]}" "$backend/sched/grant_observer.c" \
            "$ROOT/tests/programs/par_layout.wf" "$HERE/runner.c" "$HERE/rayon-bench.sh" \
            "$HERE/Makefile" "$ROOT/.github/workflows/io-scheduler.yml" \
            "$HERE/rayon-baseline/src/main.rs" "$HERE/rayon-baseline/Cargo.toml" \
            "$HERE/rayon-baseline/Cargo.lock"; do
            shasum -a 256 "$source" >> "$OUT/$control_name-source.sha256"
        done
        # Header coverage includes all platform branches, not just this host.
        while IFS= read -r source; do
            shasum -a 256 "$ROOT/$source" >> "$OUT/$control_name-source.sha256"
        done < <(git -C "$ROOT" ls-files 'compiler/src/backend/*.h' 'compiler/src/backend/**/*.h')
        (cd "$OUT" && shasum -a 256 whitefootc runner rust-layout wf-seq wf-par wf-manual-0 wf-manual-1 \
            wf-par.ll "${candidate_ir##*/}" > "$control_name-ordinary.sha256")
    fi
    WF_BENCH_RAW="$OUT/resource.tsv" "$OUT/runner" "$OUT/resource.plan" \
        "$ROUNDS" "$WARMUP" "$EXPECTED" > "$OUT/resource.txt" 2> "$OUT/resource.err"

    # Existing observer, separately linked and never timed. The normal WF
    # executable above remains the compiler's ordinary native output. The
    # completion units match the ordinary module's write_once output path
    # and provide grant_observer.c's bridge-report dependency. Final stdout
    # output initializes the bridge even though the layout work is CPU-only.
    backend="$ROOT/compiler/src/backend"
    if [[ $RESOURCE_CONTROLS != 3 && $RESOURCE_CONTROLS != 4 && $RESOURCE_CONTROLS != 5 && $RESOURCE_CONTROLS != 6 ]]; then
        "$WFC" --par --emit-llvm "$ROOT/tests/programs/par_layout.wf" -o "$OUT/wf-par.ll"
    fi
    observer_sources=()
    for unit in wf_floor.c sched/core.c sched/prim_host.c sched/entry.c \
        completion/runtime.c completion/wait_host.c completion/file_adapter.c \
        completion/file_posix.c completion/bridge.c completion/linux_io_uring.c \
        sched/grant_observer.c; do observer_sources+=("$backend/$unit"); done
    observer_command=(/usr/bin/clang -std=c11 -O2 -pthread -I "$backend" \
        -I "$backend/completion" -DWF_SCHED_OBSERVE=1 -x c "${observer_sources[@]}" \
        -x ir "$OUT/wf-par.ll" -Wno-override-module -lm -o "$OUT/wf-par-observed")
    printf '%q ' "${observer_command[@]}" > "$OUT/observer.command"
    printf '\n' >> "$OUT/observer.command"
    "${observer_command[@]}"
    if [[ $RESOURCE_CONTROLS == 3 || $RESOURCE_CONTROLS == 4 || $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
        observer_used_command=(/usr/bin/clang -std=c11 -O2 -pthread -I "$backend" \
            -I "$backend/completion" -DWF_SCHED_OBSERVE=1 "${candidate_defines[@]}" \
            -x c "${observer_sources[@]}" \
            -x ir "$candidate_ir" -Wno-override-module -lm -o "$OUT/wf-$candidate_name-observed")
        printf '%q ' "${observer_used_command[@]}" > "$OUT/observer-$candidate_name.command"
        printf '\n' >> "$OUT/observer-$candidate_name.command"
        "${observer_used_command[@]}"
        shasum -a 256 "$OUT/wf-$candidate_name-observed" "$HERE/rayon-phase-trace.sh" >> "$OUT/host.txt"
    fi
    shasum -a 256 "$OUT/wf-par.ll" "$OUT/wf-par-observed" "${observer_sources[@]}" \
        "$backend/sched/core.h" "$backend/sched/prim.h" "$backend/sched/switch.h" \
        "$HERE/runner.c" "$HERE/rayon-bench.sh" >> "$OUT/host.txt"
    if [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
        shasum -a 256 "$candidate_ir" >> "$OUT/host.txt"
        (cd "$OUT" && shasum -a 256 whitefootc runner rust-layout wf-seq wf-par wf-manual-0 wf-manual-1 \
            wf-par.ll "${candidate_ir##*/}" wf-par-observed "wf-$candidate_name-observed" > "$control_name-artifact.sha256")
    fi
    printf 'observation=untimed WF_SCHED_OBSERVE=1; exhausted_compute counts no-target join turns, not peak stacks\n' >> "$OUT/host.txt"
    for batches in "${resource_batches[@]}"; do
        observed_args=("$OUT/wf-par-observed")
        for ((batch=1; batch<batches; batch++)); do observed_args+=(batch); done
        for form in "${resource_forms[@]}"; do
            if [[ ( $RESOURCE_CONTROLS == 3 || $RESOURCE_CONTROLS == 4 || $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ) && $form == manual ]]; then continue; fi
            resource_settings "$form"
            observed_args[0]="$OUT/wf-par-observed"
            if [[ $RESOURCE_CONTROLS == 3 && $form == lanes ]]; then observed_args[0]="$OUT/wf-lanes-observed"; fi
            if [[ $RESOURCE_CONTROLS == 4 && $form == idle0 ]]; then observed_args[0]="$OUT/wf-idle0-observed"; fi
            if [[ ( $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ) && $form == "$candidate_name" ]]; then observed_args[0]="$OUT/wf-$candidate_name-observed"; fi
            record="$OUT/observed-${resource_label#wf.w4.}-b$batches"
            IFS=, read -r -a observed_environment <<< "$resource_environment,WF_SCHED_REPORT=1"
            if [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
                printf '%q ' env "${observed_environment[@]}" "${observed_args[@]}" > "$record.command"
                printf '\n' >> "$record.command"
            fi
            env "${observed_environment[@]}" "${observed_args[@]}" \
                > "$record.out" 2> "$record.err"
            printf '%s\n' "$EXPECTED" | cmp - "$record.out"
            awk '/^sched:/ {seen++; for(i=2;i<=NF;i++) {split($i,p,"=");v[p[1]]=p[2]+0}}
                END {exit !(seen==1 && v["observed"]==1 && v["threads"]==4 &&
                    v["workers_started"]==3 && v["steals"]>0 && ("exhausted_compute" in v))}' "$record.err"
            if [[ $RESOURCE_CONTROLS == 3 ]]; then
                awk -v form="$form" '/^sched:/ {for(i=2;i<=NF;i++) {split($i,p,"=");v[p[1]]=p[2]+0}}
                    END {exit !(("compact_stacks" in v) && v["compact_stacks"]==0 &&
                        ("init_used_lanes" in v) && v["init_used_lanes"]==(form=="lanes"))}' "$record.err"
            fi
            if [[ $RESOURCE_CONTROLS == 4 || $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
                awk -v form="$form" '/^sched:/ {for(i=2;i<=NF;i++) {split($i,p,"=");v[p[1]]=p[2]+0}}
                    END {exit !(("spin_rounds" in v) && v["spin_rounds"]==256 &&
                        ("yield_rounds" in v) && v["yield_rounds"]==(form=="idle0" ? 0 : 16) &&
                        ("compact_stacks" in v) && v["compact_stacks"]==0 &&
                        ("init_used_lanes" in v) && v["init_used_lanes"]==0 &&
                        ("idle_steps" in v) && ("idle_looks" in v) && ("idle_waits" in v))}' "$record.err"
            fi
            if [[ $RESOURCE_CONTROLS == 6 ]]; then
                # Grants are steals, not all successful acquisitions. Owner
                # inline executions account for the rest of these CPU tasks.
                awk -v form="$form" -v batches="$batches" '
                    /^grants=/ {grants++; split($0,p,"="); count=p[2]+0}
                    /^sched:/ {seen++; for(i=2;i<=NF;i++) {split($i,p,"=");v[p[1]]=p[2]+0}}
                    END {expected=63+1600*batches*(form=="grain4" ? 15 : 63);
                        print "published_executions=" v["steals"]+v["inline_runs"], "expected=" expected;
                        exit !(grants==1 && seen==1 && count==v["steals"] &&
                            ("inline_runs" in v) && v["steals"]+v["inline_runs"]==expected)}
                ' "$record.err" > "$record.publications"
            fi
            if [[ $RESOURCE_CONTROLS == 2 && $(uname -s) == Linux ]]; then
                # Qualify the mechanism outside timing. A Linux host without
                # a usable ring cannot answer this particular comparison.
                awk -v form="$form" '/^ring:/ {seen++; for(i=2;i<=NF;i++) {split($i,p,"=");v[p[1]]=p[2]+0}}
                    END {exit !(form=="disabled" ? seen==0 :
                        seen==1 && ("submissions" in v) && ("submission_enters" in v) && ("completions" in v) &&
                        v["submissions"]==0 && v["submission_enters"]==0 && v["completions"]==0)}' "$record.err" || {
                    echo "rayon-bench: Linux ring control qualification failed for $form; see $record.err" >&2
                    exit 1
                }
            fi
        done
    done
    if [[ $RESOURCE_CONTROLS == 5 || $RESOURCE_CONTROLS == 6 ]]; then
        (cd "$OUT" && shasum -a 256 -c "$control_name-ordinary.sha256") > "$OUT/$control_name-ordinary-check.txt"
        (cd "$OUT" && shasum -a 256 -c "$control_name-artifact.sha256") > "$OUT/$control_name-artifact-check.txt"
        shasum -a 256 -c "$OUT/$control_name-source.sha256" > "$OUT/$control_name-source-check.txt"
    fi
    cat "$OUT/resource.txt"
    printf 'rayon-bench: resource samples and separate observations retained in %s\n' "$OUT"
    if [[ $CPU_PHASE_TRACE == 1 ]]; then OUT="$OUT" bash "$HERE/rayon-phase-trace.sh"; fi
    exit 0
fi

# Each native process requests BATCHES explicitly. WF's complete argument
# count includes its executable, so BATCHES - 1 dummy args select the same work.
wf_args=("$OUT/wf-seq")
for ((batch=1; batch<BATCHES; batch++)); do wf_args+=(batch); done
"${wf_args[@]}" > "$OUT/wf-seq.out"
printf '%s\n' "$EXPECTED" | cmp - "$OUT/wf-seq.out"
wf_args[0]="$OUT/wf-par"
for width in "${threads[@]}"; do
    WF_WORKERS=$width WF_STACKS=1100 "${wf_args[@]}" > "$OUT/wf-par-$width.out"
    printf '%s\n' "$EXPECTED" | cmp - "$OUT/wf-par-$width.out"
done

# Calibration is an independent alternating cohort. The best grain at each
# pool width is frozen before any confirmation sample is collected.
printf 'rust-seq\t\t%s\tseq\t1\t64\t%s\n' "$OUT/rust-layout" "$BATCHES" > "$OUT/calibration.plan"
for width in "${threads[@]}"; do
    for grain in "${grains[@]}"; do
        printf 'rayon.w%s.g%s\t\t%s\trayon\t%s\t%s\t%s\n' \
            "$width" "$grain" "$OUT/rust-layout" "$width" "$grain" "$BATCHES" >> "$OUT/calibration.plan"
    done
done
WF_BENCH_RAW="$OUT/calibration.tsv" "$OUT/runner" "$OUT/calibration.plan" \
    "$CALIBRATION_ROUNDS" 1 "$EXPECTED" > "$OUT/calibration.txt" 2> "$OUT/calibration.err"
awk '$1 ~ /^rayon[.]w/ {
    split($1,p,"."); width=substr(p[2],2); grain=substr(p[3],2);
    if (!(width in best) || $2 < best[width]) {best[width]=$2; selected[width]=grain}
} END {for(width in selected) print width "\t" selected[width]}' \
    "$OUT/calibration.txt" | sort -n > "$OUT/selected.tsv"
[[ $(wc -l < "$OUT/selected.tsv") -eq ${#threads[@]} ]] || {
    echo 'rayon-bench: calibration did not select every requested pool width' >&2
    exit 1
}

awk '$1=="rust-seq"' "$OUT/calibration.plan" > "$OUT/confirmation.plan"
while IFS=$'\t' read -r width grain; do
    awk -F '\t' -v name="rayon.w$width.g$grain" '$1==name' "$OUT/calibration.plan" >> "$OUT/confirmation.plan"
done < "$OUT/selected.tsv"
printf 'wf-seq\t\t%s' "$OUT/wf-seq" >> "$OUT/confirmation.plan"
for ((batch=1; batch<BATCHES; batch++)); do printf '\tbatch' >> "$OUT/confirmation.plan"; done
printf '\n' >> "$OUT/confirmation.plan"
for width in "${threads[@]}"; do
    printf 'wf-par.w%s\tWF_WORKERS=%s,WF_STACKS=1100\t%s' "$width" "$width" "$OUT/wf-par" >> "$OUT/confirmation.plan"
    for ((batch=1; batch<BATCHES; batch++)); do printf '\tbatch' >> "$OUT/confirmation.plan"; done
    printf '\n' >> "$OUT/confirmation.plan"
done
WF_BENCH_RAW="$OUT/confirmation.tsv" "$OUT/runner" "$OUT/confirmation.plan" \
    "$ROUNDS" "$WARMUP" "$EXPECTED" > "$OUT/confirmation.txt" 2> "$OUT/confirmation.err"
cat "$OUT/calibration.txt" "$OUT/selected.tsv" "$OUT/confirmation.txt"
printf 'rayon-bench: raw samples and exact-byte qualifications retained in %s\n' "$OUT"

if [[ $RAYON_PROFILE == 1 ]]; then
    [[ $(uname -s) == Linux ]] || { echo 'rayon-bench: profiling requires Linux' >&2; exit 2; }
    perf_tool=${PROFILE_PERF:-perf}
    profile_user=$(id -un)
    sudo -n true
    mkdir -p "$OUT/profile"
    {
        "$perf_tool" --version
        echo 'profile_batches=4; ordinary binaries; separate CPU and scheduler captures; no profiled timings enter confirmation.tsv'
        echo "workload_user=$profile_user; privileged recorder only; workload inherits the normal job affinity"
        cat /proc/sys/kernel/perf_event_paranoid
        cat /proc/sys/kernel/kptr_restrict
    } > "$OUT/profile/host.txt"
    # The recorder needs scheduler tracepoint access. Drop back to the normal
    # job user before exec, preserve the real workload PID, and separate its
    # strict output channels from profiler diagnostics.
    cat > "$OUT/profile/run" <<'PROFILE_RUN'
#!/bin/sh
record=$1
shift
printf '%s\n' "$$" > "$record.pid"
exec "$@" > "$record.out" 2> "$record.err"
PROFILE_RUN
    chmod +x "$OUT/profile/run"
    profile_case() {
        local kind=$1 label=$2 record
        shift 2
        record="$OUT/profile/$kind-$label"
        printf '%q ' "$@" > "$record.command"
        printf '\n' >> "$record.command"
        if [[ $kind == cpu ]]; then
            sudo -n "$perf_tool" record -e cpu-clock -F 999 -o "$record.data" -- \
                sudo -n -u "$profile_user" -- "$OUT/profile/run" "$record" "$@" \
                > "$record.recorder.out" 2> "$record.recorder.err"
        else
            sudo -n "$perf_tool" sched record -a -o "$record.data" -- \
                sudo -n -u "$profile_user" -- "$OUT/profile/run" "$record" "$@" \
                > "$record.recorder.out" 2> "$record.recorder.err"
        fi
        printf '%s\n' "$EXPECTED" | cmp - "$record.out"
        [[ ! -s $record.err && -s $record.pid ]]
        sudo -n chown "$(id -u):$(id -g)" "$record.data"
        if [[ $kind == cpu ]]; then
            "$perf_tool" report --stdio --header --show-nr-samples --no-children \
                --pid "$(cat "$record.pid")" --sort pid,dso,symbol -i "$record.data" > "$record.report" 2> "$record.report.err"
            "$perf_tool" script --show-lost-events -i "$record.data" \
                -F comm,pid,tid,time,event,ip,sym,dso,period > "$record.samples" 2> "$record.script.err"
        else
            "$perf_tool" sched timehist -i "$record.data" --pid "$(cat "$record.pid")" \
                --state --wakeups --migrations --with-summary > "$record.timehist" 2> "$record.timehist.err"
            "$perf_tool" script --show-lost-events -i "$record.data" > "$record.samples" 2> "$record.script.err"
            [[ -s $record.timehist ]]
        fi
        [[ -s $record.samples ]]
        echo "rayon-profile: $kind $label captured; inspect lost events before attributing costs"
    }
    # Four identical source batches amortize startup in observations without
    # changing the preceding one-batch calibration or confirmation panel.
    for kind in cpu sched; do
        profile_case "$kind" rust-seq "$OUT/rust-layout" seq 1 64 4
        profile_case "$kind" wf-seq "$OUT/wf-seq" batch batch batch
        while IFS=$'\t' read -r width grain; do
            profile_forms=(wf rayon)
            if [[ $kind == sched ]]; then profile_forms=(rayon wf); fi
            for form in "${profile_forms[@]}"; do
                if [[ $form == wf ]]; then
                    profile_case "$kind" "wf-w$width" env "WF_WORKERS=$width" WF_STACKS=1100 \
                        "$OUT/wf-par" batch batch batch
                else
                    profile_case "$kind" "rayon-w$width-g$grain" "$OUT/rust-layout" rayon "$width" "$grain" 4
                fi
            done
        done < "$OUT/selected.tsv"
    done
fi
