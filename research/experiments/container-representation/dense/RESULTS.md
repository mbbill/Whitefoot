# Dense construction, value transfer, and destination storage

This experiment isolates a representation question: can fixed dense storage be
constructed, returned by a helper, updated repeatedly, and consumed without
moving the whole payload for every element operation? It does not implement a
new checked storage design, select the container API, or measure I/O/runtime
performance. The native variants are research references, not trusted or checked
Whitefoot implementations. This fixture remains owned by the container
representation experiment; supersede its measurements and remove obsolete
variants when the representation question changes.

## Reproduction and scope

From this directory, after building the repository compiler:

```sh
cargo build --manifest-path ../../../../compiler/Cargo.toml --locked --offline \
  --profile gate --bin whitefootc
make check
make measure
build/driver summarize measurements.csv
```

`WHITEFOOTC`, `CLANG`, and `RUSTC` can be overridden. `make clean` removes only
this experiment's generated `build/` artifacts. Use a clean build after changing
compiler flags or tools. `make measure` writes new results under `build/`; it
does not overwrite the retained `measurements.csv`.

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

`make check` passes. For each N in {16,256,4096}, 12 runtime seeds and R in
{0,1,3,4,8} give 60 differential inputs per variant. Every Whitefoot and native
result agrees. Three complete Whitefoot command executables also agree with an
independent safe-Rust oracle at seed=19, R=3. The timed run originally verified
{0,1,3,8}; R=4 was subsequently added to the verification set and passed without
changing the measured kernels or timing procedure. Verification contains no
timing threshold.

## Measurements

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

## Storage and copy evidence

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

The artifacts are reproducible with `make shape` and retained under `build/`:

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

## Wide-record boundary and conclusions

`inline-view.wf` is a separate complete source probe: append a byte, form an
exclusive view of its inline owner, update through the view, and verify the
owner sees the new byte. It currently reports `ExclusiveViewOverInlineRun`.
The check records only that exact unsupported result or, once supported,
compiles and executes the program. The implementation target requires the
successful execution; retaining an unsupported observation is not its completion.

The same generator also produces a complete N=16 source with four distinct
`u64` fields per element. Current records are affine, so it reads an element
through a shared borrow, computes the replacement fields, ends the borrow, and
replaces the owned element. The compiler reports
`Semantics/Unsupported: RegionsAndBorrows` for that inline-element borrow.
`make check` records this honest capability result in `build/wide-probe.txt`;
it does not call the source invalid or include it in runtime performance data.
If the capability becomes supported, the probe reports that the wide runtime
measurement should be added. No fake failure branch or spare-buffer algorithm
was inserted to get this case through the current compiler.

The measured result supports an implementation requirement for this dense
workload: owned values need a representation that preserves destination storage
through construction and update. A source-level helper returning a value does
not inherently require a payload copy, including in the demonstrated retained
call control. Equally, an aggregate consume/return chain is not reliably repaired
by ordinary LLVM optimization. None of this selects compiler-owned container
state machines over checked library representations, settles stable handles,
establishes arbitrary-record support, or proves any native implementation safe
enough to become a Whitefoot kernel.
