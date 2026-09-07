# Dense construction, value transfer, and destination storage

This experiment isolates a representation question: can fixed dense storage be
constructed, returned by a helper, updated repeatedly, and consumed without
moving the whole payload for every element operation? It does not implement a
new checked storage design, select the container API, or measure I/O/runtime
performance. The native variants are research references, not trusted or checked
Whitefoot implementations. This fixture remains owned by the container
representation experiment; supersede its measurements and remove obsolete
variants when the representation question changes.

The two retained runs compare the pre-implementation baseline at `eff095c7` with
the owned-storage checkpoint at `f5dab70c`. The latter removes whole-payload
transfers from the element loops and passes all three scalar sizes, the N=16
four-field record, and the inline exclusive-view program. It still retains a
one-time aggregate transfer and excess frame storage. The full repository gate
and parallel aggregate integration remain pending; these measurements establish
this experiment's behavior and cost, not workload prevalence or merge readiness.

## Reproduction and scope

From this directory, after building the repository compiler:

```sh
cargo build --manifest-path ../../../../compiler/Cargo.toml --locked --offline \
  --profile gate --bin whitefootc
make check
make measure
build/driver summarize measurements.csv
build/driver summarize owned-storage-measurements.csv
```

`WHITEFOOTC`, `CLANG`, and `RUSTC` can be overridden. `make clean` removes only
this experiment's generated `build/` artifacts. Use a clean build after changing
compiler flags or tools. `make measure` writes new results under `build/`; it
does not overwrite either retained CSV. `measurements.csv` owns the baseline
samples; `owned-storage-measurements.csv` owns the first implementation samples.
Keep the pair while this before/after comparison is used, and supersede it when
the experiment changes rather than accumulating unexamined runs.

Recorded run: 2026-09-06, arm64 macOS 26.6.2 (25G83), Apple Clang 21.0.0
(clang-2100.1.1.101), Rust 1.98.1. The sandbox did not permit querying the CPU
model; no particular Apple chip is assumed. Compiler implementation baseline:
`eff095c7`; the compiler was freshly built at `0f537c7757db90019f83b48faf7873aa674571dd`
(subsequent changes there were documentation). Its binary SHA-256 was
`c69fc5e17f651f1db3febdd6783e913410aeaab63e8a74d517c903c5690f8eaf`.

Both Whitefoot IR and native C use Clang `-O2`. C uses C17 with all enabled
warnings as errors. No LTO, native assembly replacement, fact injection,
compiler patch, or per-function optimization attributes affect the primary
timed variants. The complete generated Whitefoot command programs are built
and executed through `whitefootc` itself.

For in-process timing only, the driver changes two linkage declarations in the
emitted module: it exports `wf_dense(u64, u64) -> u64` and renames the generated
C `main` to `wf_fixture_main`. It changes no function body or attribute. This
private ABI adapter is necessary because source functions otherwise have
internal linkage. The native harness is compiled separately from the measured
kernels, receives its initial seed through `argv`, varies the seed on each call,
and consumes each returned checksum through an observable volatile sink.

## Work performed and correctness

The shared trace is: construct N `u64` elements using a runtime-seeded recurrence;
return the container from a helper; perform R complete update passes where each
next state depends on the previous state and the old element; hash every final
element in order. R=0 isolates construction plus consumption; R=4 includes
repeated updates. Arithmetic is explicitly modulo 2^64 in both languages. All
payload words contribute to the result. The variants use the same payload
layout followed by two `u64` descriptor fields; native static assertions check
the resulting size and field offset.

- `whitefoot`: normal `FixedVector<u64,N>` construction by `place_back`, helper
  return, indexed updates, and indexed consumption.
- `c_value`: construct in a helper's local aggregate and return that aggregate
  by value. Ordinary compiler optimization may inline the helper.
- `c_destination`: construct directly into a caller-owned aggregate through a
  pointer, then execute the same updates and consumption.
- `c_append_value`: pass and return the entire aggregate through a small append
  helper for every element. This deliberately retains the consume/return-shaped
  source chain as a separate control; it is not conflated with an ordinary
  by-value constructor return.

All native variants initially zero the full aggregate. This experiment does
not award the references an initialization-elision advantage in their source.
The native optimizer retains a one-time memset for the larger direct-storage
variants, while their element loops remain linear.

The baseline `make check` passed. For each N in {16,256,4096}, 12 runtime seeds and R in
{0,1,3,4,8} give 60 differential inputs per variant. Every Whitefoot and native
result agrees. Three complete Whitefoot command executables also agree with an
independent safe-Rust oracle at seed=19, R=3. The timed run originally verified
{0,1,3,8}; R=4 was subsequently added to the verification set and passed without
changing the measured kernels or timing procedure. Verification contains no
timing threshold.

## Baseline measurements

The retained CSV has 168 samples: 3 sizes × 2 round counts × 4 variants × 7
samples. Each sample reports:

```text
sample,N,lanes,variant,rounds,sample_index,iterations,elapsed_ns,checksum
```

Calibration doubles iteration counts until a batch reaches 20 ms or 1,048,576
calls. The smallest native construction-only batches reach that call cap first.
Samples rotate variant order. Compiler builds and coordinated agent workloads
were paused during timing. This is one local run, not a confidence interval or
a cross-platform performance claim.

Median nanoseconds per complete kernel call:

| Elements | Payload bytes | Update passes | Whitefoot | C value return | C destination | C whole-value append |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 128 | 0 | 90.22 | 11.81 | 11.34 | 177.48 |
| 16 | 128 | 4 | 408.39 | 64.89 | 64.29 | 224.90 |
| 256 | 2,048 | 0 | 32,205.08 | 618.56 | 617.52 | 14,642.58 |
| 256 | 2,048 | 4 | 176,023.44 | 1,503.97 | 1,519.10 | 15,681.15 |
| 4096 | 32,768 | 0 | 8,766,750.00 | 13,341.80 | 13,358.40 | 3,635,500.00 |
| 4096 | 32,768 | 4 | 43,866,000.00 | 29,727.54 | 29,864.26 | 3,679,000.00 |

For four update passes, Whitefoot/C-value median ratios are approximately
6.29, 117.04, and 1,475.60. The raw data includes outliers: at N=4096/R=4,
C-value spans 29,431.64–61,250.98 ns and Whitefoot spans
43,335,000–61,049,000 ns. The large gap is also present in the minima; the small
differences between C-value and C-destination do not justify ranking them.

## Baseline storage and copy evidence

Clang `-fstack-usage` reports these static entry-function frames in bytes:

| Elements | Whitefoot `wf_dense` | C value return | C destination | C whole-value append |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 656 | 160 | 160 | 320 |
| 256 | 10,464 | 2,112 | 2,112 | 4,208 |
| 4096 | 164,160 | 32,832 | 32,832 | 65,664 |

These are per-function reports, not total process RSS or all transitive stack
usage. At N=4096, `wf_dense` additionally calls `wf_build`, whose frame is
65,488 bytes: those two simultaneous frames sum to 229,648 bytes, excluding
platform helper stack use. At N=16 a native update helper can add 16 bytes.
No measured kernel allocates from the heap.

The artifacts are reproducible with `make shape` at the corresponding revision
and retained under `build/`. A current build contains the new checkpoint's shape,
not the historical baseline's:

```sh
cat build/dense*.su build/native*.su
sed -n '/^LBB0_1:/,/b.ne.*LBB0_1/p' build/dense16.s
rg -n '^define|memcpy|alloca' build/native256.opt.ll
rg -n '^define|sret|call.*boundary|memcpy' build/boundary256.ll
```

The second command is specific to this recorded arm64 assembly. Its first
construction-loop iteration stores the existing 128-byte payload, stores the
new 8-byte word, and reloads the 128-byte payload: 264 bytes of explicit memory
operands per append, before considering subsequent phases. This is derived
instruction-level transfer volume, not a hardware-counter measurement. Larger
optimized Whitefoot functions likewise retain expanded payload transfers and
spills in their element loops. The source IR's aggregate store/load is therefore
not merely harmless notation eliminated by this backend.

Conversely, `c_value` and `c_destination` have direct element stores and no
whole-array memcpy in their construction/update loops. The C whole-value-append
control has **two 2,064-byte memcpy operations per append** at N=256, and two
32,784-byte copies per append at N=4096. Thus C/LLVM also fails to eliminate the
repeated aggregate chain in that source form. Static assembly line counts are
not used as performance evidence.

The primary C helper-return comparison permits ordinary inlining. A separate,
untimed N=256 control forces constructor calls to remain `noinline` and is
executed against the same 60-input correctness matrix. In `boundary256.ll`,
`return_across_boundary` writes through `sret(Dense)` directly into the caller's
single aggregate allocation. `construct_across_boundary` receives that same
single allocation as an ordinary pointer. Neither control has an intervening
aggregate memcpy. This establishes feasibility of a direct destination across
these retained internal calls, not timing parity for every ABI or aggregate.

## Owned-storage checkpoint

The compiler was freshly built from the clean implementation commit
`f5dab70cf785a75122e96dfdaf835c4e8e6364aa` on 2026-09-06 local time
(2026-09-07 UTC), with binary SHA-256
`9fc3a257dfb148ee2f330954a2ef0cc9dd9b575a0ab03c1ff022d40a3be570db`.
The OS, Rust, Clang, flags, seed and measurement procedure match the baseline.
`make clean && make measure` rebuilt every artifact. The three scalar and one
wide-record checks each passed 60 inputs against all four variants; the retained
N=256 boundary control passed against all six variants. All command executables
and the inline-view executable returned 0. The wide fixture correction below
does not change any timed scalar kernel.

The new CSV contains the same 168-sample matrix. Compiler builds finished before
timing, and this task's agents were idle. Other host activity was not controlled;
this is one local measurement, not a cross-platform or statistical speed claim.

Median nanoseconds per complete kernel call:

| Elements | Update passes | Whitefoot | C value return | C destination | C whole-value append |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 0 | 11.23 | 10.96 | 10.91 | 168.69 |
| 16 | 4 | 108.38 | 60.66 | 61.25 | 208.15 |
| 256 | 0 | 491.04 | 607.22 | 606.17 | 14,317.87 |
| 256 | 4 | 1,522.03 | 1,480.22 | 1,479.98 | 15,321.78 |
| 4096 | 0 | 9,077.64 | 13,236.82 | 13,140.62 | 3,618,125.00 |
| 4096 | 4 | 25,380.86 | 28,943.36 | 29,020.51 | 3,626,875.00 |

For four passes, Whitefoot's before/after median ratios are approximately
3.77, 115.65 and 1,728.31. Against the C-value control in the new run, its ratios
are 1.79, 1.03 and 0.88. The small case remains slower. These comparisons isolate
this algorithm/layout; they do not rank the languages or establish a cause for
every remaining difference.

| Elements | Baseline Whitefoot entry frame | New Whitefoot entry frame | New C value/destination entry frame |
| ---: | ---: | ---: | ---: |
| 16 | 656 | 448 | 160 |
| 256 | 10,464 | 6,240 | 2,112 |
| 4096 | 164,160 | 125,872 | 32,832 |

The new constructor is inlined in all three optimized kernels. At N=4096 there
is therefore no additional 65,488-byte constructor frame; the former two-frame
sum was 229,648 bytes. These remain static function reports, not RSS or a complete
stack bound. The wide entry frame is 1,664 bytes and has no retained timing run.

The raw constructor writes each new word and its run metadata directly into
`%wf.result`; no payload snapshot crosses a construction-loop edge. Optimized
update and checksum loops address one word at a time, with no aggregate transfers
in those loops. There is no heap allocation in the measured kernels.

The result destination and the subsequently addressable binding are still
separate: raw `wf_dense` loads the completed aggregate from `%wf.slot.0` and
stores it into `%v3` once before the update loops. The third slot is a snapshot
loaded for final cleanup. This `u64` run has no element cleanup, so optimization
removes that final transfer but retains the empty middle field in the unified
frame. There are not three live source owners.

In optimized N=4096 IR the initial transfer expands into payload loads and stores
even though there is no `memcpy` spelling. The assembly reserves 98,352 bytes for
the three aggregate fields, 27,360 further local/spill bytes and 160 saved-register
bytes, totaling 125,872. Absence of `memcpy` is not evidence of absence of copying.
No old aggregate snapshot is required by this particular source: construction,
helper return and binding initialization transfer one owner, and element reads
copy only `u64`. The remaining copies reflect conservative placement, rather than
a source ownership requirement. This checkpoint removes the repeated work shape
without claiming optimal placement or a single physical payload. Result-to-binding
reuse and frame compaction require a general lifetime justification that also
preserves actual snapshots in other programs.

## Wide-record boundary and conclusions

`inline-view.wf` is a separate complete source probe: append a byte, form an
exclusive view of its inline owner, update through the view, and verify the
owner sees the new byte.
The baseline reported `ExclusiveViewOverInlineRun`. The owned-storage path now
compiles and executes this program; `make check` requires exit 0 and no longer
accepts an unsupported result.

The same generator also produces a complete N=16 source with four distinct
`u64` fields per element. Current records are affine, so it reads an element
through a shared borrow, computes the replacement fields, ends the borrow, and
replaces the owned element. The baseline stopped at
`Semantics/Unsupported: RegionsAndBorrows` for that inline-element borrow.
After implementing the borrow, a redundant explicit region in the final checksum
loop became visible as a FORM-8 source defect. Removing that sole-statement
wrapper preserves the loop's implicit region and the algorithm; the update
loop's short borrow region remains necessary and remains present. Scalar timed
kernels were unchanged by this fixture correction.

`make check` now builds and executes the wide program and checks its checksum
against all three native controls over the same 60-input matrix. The four-field
case remains outside the retained 168 scalar timing samples. No failure branch
or alternate container algorithm was introduced to satisfy the checker.

The measured result supports an implementation requirement for this dense
workload: owned values need a representation that preserves destination storage
through construction and update. A source-level helper returning a value does
not inherently require a payload copy, including in the demonstrated retained
call control. Equally, an aggregate consume/return chain is not reliably repaired
by ordinary LLVM optimization. None of this selects compiler-owned container
state machines over checked library representations, settles stable handles,
establishes arbitrary-record support, or proves any native implementation safe
enough to become a Whitefoot kernel.
