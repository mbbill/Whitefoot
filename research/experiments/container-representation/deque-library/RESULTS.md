# Deque library costs

This explicit experiment bundles [`deque.wf`](../../../../lib/containers/deque.wf).
It is outside daily correctness CI. `make check` verifies the operation traces;
`make measure` checks them first and writes interleaved timing samples under
`.build/`. Retire this experiment when a maintained successor covers the same
contract and preserves its comparison evidence.

## Contract and criterion before timing

The selected Deque has reference-taking endpoint mutations and an explicit
consuming rebase that creates a genuinely new backing. It does not promise
automatic growth or a zero-copy two-span interface. Compare the actual library
with C using the same header, inline payloads, allocation policy, callback
order, and retained helper boundaries. The additional bulk C control replaces
the element-by-element rebase with at most two contiguous copies, preserving
that conversion's contract. Its difference from the loop C control prices
source composition separately from WF lowering.

The three traces are forward churn (`pop_front`, `push_back`), reverse churn
(`pop_back`, `push_front`), and rebase after a half-length rotation has wrapped
the live window. Churn reuses one allocation. Rebase grows from capacity `n`
to `2*n+1`, preserves logical order, resets the head, drains, and releases both
owners. Every payload word contributes to an order-sensitive checksum. An
independent arithmetic oracle derives logical element order without using a
deque. Timed traces include setup and final cleanup; per-round figures are
amortized whole-trace costs, not isolated operation latency.

The matrix uses lengths 16, 256, and 4096 and 8-byte scalars or 256-byte
`nocopy` inline records. Correctness adds empty, singleton, and irregular
lengths, zero rounds, and wrapping arithmetic seeds. Allocation counts,
requested bytes, peak bytes, and final zero live bytes must match before a
timing is accepted. Normal and retained modes save optimized WF/C IR; retained
helper calls must survive. Element transfers are read from those artifacts
separately from layout and timing. No universal parity threshold or application
frequency is assumed.

A separate `setup-cleanup` measurement uses zero churn rounds and repeats
`32768 / count` complete allocate/fill/drain/free traces per sample. This
prices construction, initial materialization and cleanup together. Rebase at
zero rounds allocates nothing and is not used as a construction baseline.
Setup timings are not subtracted from whole traces to claim isolated operation
latencies.

## Source admission observations

The first caller selected direction inside each iteration:

```wf
let removed = if path != 0_u64 {
  let value = deque_pop_back::<T>(values: &values);
  invariant popped: values.inner.len + 1_u64 == count;
  give move value;
} else {
  let value = deque_pop_front::<T>(values: &values);
  invariant popped: values.inner.len + 1_u64 == count;
  give move value;
}
DequeElement::accept(env: &digest, value: move removed);
let next = base +wrap index;
let value = DequeElement::make(seed: next);
if path != 0_u64 {
  let length = deque_push_front::<T>(values: &values, value: move value);
  invariant restored: values.inner.len == count;
} else {
  let length = deque_push_back::<T>(values: &values, value: move value);
  invariant restored: values.inner.len == count;
}
```

The v0.63 baseline compiler rejected `restored` under INV-1 after both
`popped` assertions were proved. Without those assertions it instead rejected
the enclosing loop's same-length backedge invariant. Independent reduction
classified this as a limit of the written proof formulation: ENT-5's counted
continuing kill removes the earlier equality, and the loop headers publish
affine relations over immutable measure images. Each branch mutation creates
a different image; the same-spelled `popped` assertions do not become one
canonical relation under ENT-6's intersection and INV-1's capture rule. A
direct common equality available to the automatic equality family is a
different case. This does not establish a general Deque interface limit.
The current caller chooses its immutable direction before the complete loops,
removing that proof join. The C traces use the same hoist; endpoint calls,
payload construction and callback order are unchanged. No additional runtime
validation or fake fallback was added.

## Results

The corrected baseline has 6,336 accepted timing samples in
[`measurements.csv`](measurements.csv), preserved at checkpoint `5a5481b1`
before the merge of main `7127bcb6`. Both normal and retained executables
passed 432 correctness configurations, each through all three implementations:
2,592 executions in total. Every execution checked the independent checksum,
allocation requests, requested/peak bytes and final zero live bytes. Retained
optimized IR contains 20 WF helper call sites and 40 C sites across the two
controls.

### Storage and allocation

All implementations allocate a 24-byte header and inline payloads of stride
8 or 256 bytes. Churn has exactly one allocation of `24 + n*stride` requested
and peak bytes. Each rebase trace round allocates both the initial capacity
`n` and replacement capacity `2*n+1`: its requests are `2*rounds`, requested
bytes are `rounds * (48 + (3*n+1)*stride)`, and peak bytes are one such pair
(zero for zero rounds). Setup repetition multiplies requests and total bytes,
not peak bytes. Observer headers and allocator metadata are excluded equally.
These formulas are checked independently for every accepted sample.

### Amortized trace costs

The time column is WF nanoseconds per logical endpoint step or rebased element
at `count = 4096`, including setup and cleanup. Ratio ranges cover all three
counts and the four run/cohort medians. Values above 1 mean WF took longer.
Ranges describe measured cohorts, not confidence intervals.

| Mode | Payload bytes | Trace | WF ns/step, n=4096 | WF / loop C | WF / bulk C |
| --- | ---: | --- | ---: | ---: | ---: |
| normal | 8 | forward churn | 3.204 | 2.283–2.341 | 2.283–2.341 |
| normal | 8 | reverse churn | 3.571–3.723 | 1.175–1.220 | 1.188–1.219 |
| normal | 8 | wrapped rebase | 4.883 | 1.650–1.943 | 1.669–2.672 |
| retained | 8 | forward churn | 4.791–4.944 | 1.000–1.015 | 0.993–1.015 |
| retained | 8 | reverse churn | 5.157–5.310 | 1.074–1.088 | 1.064–1.101 |
| retained | 8 | wrapped rebase | 8.209–8.270 | 1.051–1.082 | 1.093–1.183 |
| normal | 256 | forward churn | 34.363–34.576 | 0.975–1.011 | 0.981–1.000 |
| normal | 256 | reverse churn | 34.393–35.034 | 0.982–1.005 | 0.984–1.007 |
| normal | 256 | wrapped rebase | 47.150–47.363 | 0.985–1.007 | 1.018–1.086 |
| retained | 256 | forward churn | 38.177–38.452 | 0.956–1.017 | 0.959–0.993 |
| retained | 256 | reverse churn | 38.055–38.452 | 0.939–0.976 | 0.939–0.981 |
| retained | 256 | wrapped rebase | 57.190–59.448 | 0.953–0.975 | 0.978–1.030 |

The wide-record workload is competitive under both boundary treatments;
scalar normal-mode churn and rebase are not at native parity. The scalar
rebase loop/bulk comparison exposes a separate source-composition opportunity,
but these complete traces do not price just the copy phase. Keeping the
explicit consuming rebase is useful as a library capability without claiming
an efficient automatic growth policy or a borrowed two-span interface.

### Zero-round setup and cleanup

Times are WF microseconds per allocate/fill/drain/free trace. Ratios span all
three counts. Cleanup visits every payload word through the checksum callback.

| Mode | Payload bytes | n=16, us/trace | n=256, us/trace | n=4096, us/trace | WF / loop C |
| --- | ---: | ---: | ---: | ---: | ---: |
| normal | 8 | 0.047–0.048 | 0.539–0.562 | 8.500–8.875 | 1.702–2.000 |
| retained | 8 | 0.067 | 1.211–1.219 | 19.625 | 0.958–1.000 |
| normal | 256 | 0.545–0.549 | 8.656–8.914 | 141.375–148.000 | 0.961–1.006 |
| retained | 256 | 0.686–0.693 | 10.805–10.953 | 175.625–181.375 | 0.957–0.992 |

### Optimized transfers and scalar loop evidence

| Retained record path | WF copies per element | Loop C copies per element |
| --- | --- | --- |
| Pop front or back | 1 x 256-byte memcpy | 1 x 256-byte memcpy |
| Push front or back | 1 x 256-byte memmove | 1 x 256-byte memcpy |
| Rebase transfer loop | 1 x 256-byte memcpy | 1 x 256-byte memcpy |
| Drain to consumer | 1 x 256-byte memcpy | 1 x 256-byte memcpy |

The WF retained functions are `wf_deque_pop_front$instance$50`,
`wf_deque_push_back$instance$51`, `wf_deque_rebase$instance$52`,
`wf_deque_drain$instance$53`, `wf_deque_pop_back$instance$55`, and
`wf_deque_push_front$instance$56`. Neither retained trace adds another record
bulk copy at the endpoint boundary. Bulk C uses at most two dynamic extent
copies during rebase. WF memory operands often have alignment 1 where native
C has alignment 8; the generated ordinary WF ABI and C's internal fastcc
helpers also differ. Equal copy counts therefore do not imply equal lowering.

The scalar forward gap persists after the instrumentation correction and
reversed-order repeat. In normal optimized WF IR,
`wf_deque_library_trace$instance$37` block `bb32` has four i64 loads (head,
capacity, payload, length) and four stores (decremented length, advanced head,
payload, restored length) per hot-loop iteration. In optimized C,
`word_loop_library_trace` block `272` keeps header state in SSA, performs one
payload load and one payload store, and commits the header after the inner
loop. Both hot loops are scalar; SIMD vectorization is not their difference.
C's initial fill does vectorize, while WF retains per-element descriptor
traffic there too. Correct allocator information permits more source-header
hoisting in rebase, but destination-header traffic remains in WF's loop.

This is a concrete lowering improvement opportunity, not an attribution of
every nanosecond to four stores. Normal versus retained changes call visibility,
inlining and native conventions as well as transfers; it is not a pure
copy-cost experiment. The ordinary reference-taking interface can stay while
that issue is tested, and the measured baseline must remain available for the
same-source comparison.

A subsequent controlled IR probe isolated the relevant optimization fact for
this scalar specialization. Starting from the corrected normal LLVM, it either
added only `nuw` to the four dynamic u64 Ring payload GEPs, or split those same
GEPs into a header projection and payload indexing without adding any facts.
After Apple Clang 21 O2:

| IR variant | Scalar forward hot-loop loads / stores | Independent correctness executions |
| --- | --- | ---: |
| Baseline | 4 / 4 | 1,296 passed |
| Unsigned non-wrapping GEP fact only | 1 payload load / 1 payload store; header state in SSA | 1,296 passed |
| Split GEP without new fact | 4 / 4 | 1,296 passed |

The fact-only variant also permits initial-fill vectorization. It identifies
an omitted fact sufficient to eliminate this specialization's metadata traffic;
it supplies no measured timing recovery percentage. Construction and all three
oracle checks took 2.26 seconds under the shared guard. Production lowering is unchanged. A
general improvement still needs qualification over all admitted storage/index
domains, including zero-sized elements, a compatible path for older LLVM,
and the same-source timing matrix. The bounded positive-stride u64 probe does
not establish that those other cases may receive the flag.

The maintained optional reproduction is
`make -C research/experiments/container-representation/deque-library probe-scalar-gep`,
under the shared guard. It refuses drift from the exact four selected u64
GEPs, saves the optimized IR variants and their oracle results under
`.build/`, and requires a local Clang that accepts GEP `nuw`. It is outside
both daily checks and the ordinary benchmark targets. It changes no compiler
emission. The target was added after baseline timing: the measured Makefile
hash below predates it; the original three-variant target's Makefile was
`1facd9eafce1baabac65e456df6ba962b46abd45a944af4eeeaf5f0afbd4e020`.

#### Nonnegative-index assumption candidate

The next bounded probe adds a fourth variant to the same target. Immediately
before each of the same four positive-stride u64 payload GEPs, it inserts
`icmp sge i64 <actual normalized address index>, 0` and
`call void @llvm.assume(i1 <comparison>)`, leaving the original `inbounds`
GEP intact. One intrinsic declaration is added. The target requires exactly
four replacements and refuses an input already containing `llvm.assume` or
the probe's temporary-name suffix. This is a transformation of a fixed
baseline module, not a general lowering implementation.

The 2026-09-22 run used the unchanged Deque library, workload and C driver
identified in this report, and the v0.65 baseline compiler corresponding to merged
`1b916975`, SHA-256
`0ab0f5730590828c511d0a0d4d90d3131654377ea020e5634f5a0800ff9edb96`.
Its raw WF output is
`a32c7d14640f6f855f67149281fa839c65d3d30597c96bc50d8de8e012fa32b6`.
The extended Makefile is
`c5d94d9e71160a37894d11f3deca02b2ec659b8a017a1543646a3d739b9c77bc`.
Reproduction pins that baseline compiler rather than a future compiler that
may already emit the assumption:

```sh
BASELINE_WHITEFOOTC=/path/to/baseline/whitefootc
perl .github/run-check.pl ring-gep-portable-probe \
  make -C research/experiments/container-representation/deque-library \
  probe-scalar-gep \
  WHITEFOOTC="$BASELINE_WHITEFOOTC" \
  CLANG=/usr/bin/clang
```

Set the variable to a saved compiler with the recorded baseline identity. Apple
Clang 21.0.0 (`clang-2100.3.34.2`) on the recorded arm64 host optimized,
linked and executed all four variants. Each passed 432 configurations and
1,296 independent checksum/allocation-ledger executions, for 5,184 executions
in total. No timing samples were collected. Probe construction and correctness
took 4.07 seconds, including rebuilt runtime objects; the complete shared
guard interval took 4.56 seconds including assembly generation. Assembly was
generated directly from each transformed input with
`clang -O2 -Wno-override-module -S -x ir`, matching the native executable's
input rather than reoptimizing the already optimized module.

Here `trace` means `wf_deque_library_trace$instance$37`, and the hot loop is
its forward-churn `bb32`. Branch counts are static instruction counts, not
dynamic branch frequencies. The full trace includes fill, reverse churn,
rebase and cleanup as well as that hot loop.

| Variant | Hot-loop i64 loads / stores | Optimized IR assume calls, trace / hot loop | AArch64 trace calls / conditional branches | AArch64 hot-loop calls / branches |
| --- | --- | --- | --- | --- |
| Baseline | 4 / 4 | 0 / 0 | 6 / 24 | 0 / 1 |
| GEP `nuw` | 1 / 1 | 0 / 0 | 6 / 30 | 0 / 1 |
| Split only | 4 / 4 | 0 / 0 | 6 / 24 | 0 / 1 |
| Nonnegative-index `assume` | 1 / 1 | 22 / 1 | 6 / 30 | 0 / 1 |

The assumption variant retains 34 intrinsic calls across the optimized
module, but none becomes an assembly call or a branch checking the sign.
The two fact-bearing variants have identical forward hot-loop instructions:
one payload load, one payload store, no call, and one loop backedge branch.
Both also allow initial-fill vectorization; their scalar trace has eight
`<2 x i64>` store instructions, versus none in the baseline or split control.
Their additional trace branches come with the same vectorized/loop forms,
not an extra assumption check. The complete scalar functions are nevertheless
different: the `nuw` variant has 310 machine instructions and the assumption
variant 305, with other-path register allocation and scheduling differences.
This is not a whole-function equivalence or performance claim. The record
trace's complete assembly is equal after removing comments and blank lines
(481 instructions), since this bounded probe does not change record payload
addresses.

| Variant | Optimized LLVM SHA-256 |
| --- | --- |
| Baseline | `912fa2d161292a9d600f423d582ccbb3a20ba18606096b3dc55ebc86caa856c9` |
| GEP `nuw` | `a59d9fec4de6587daff422e9aca58590bb98b40fdde1c2137eae8a205e256116` |
| Split only | `2a8736557f16554218531368eb22d429a95225045bbf044fb3a9ca64070e2f7d` |
| Nonnegative-index `assume` | `a06f39c34ed65e80cd1c6fd1eaeea72668da4020771a588ad5ea85b4b817fb13` |

This evidence selects the normalized-index assumption as the production
candidate to evaluate: it communicates the useful local fact without
requiring newer GEP-flag syntax or making emitted syntax depend on a
build-time compiler that may differ from the actual IR consumer. It does not
establish that the two candidates have equal full-matrix cost. The target's
four-case `nuw` comparison still needs a recent LLVM; this run does not test
LLVM 18 or Apple Clang 15 compatibility of production output. The full
normal/retained scalar/record timing comparison and actual older-consumer
qualification are still pending. General padded-header/extent/index
qualification needs its own proof; this four-site experiment does not
establish it. These selected u64 sites do not qualify every alignment,
zero-stride step or maximum admitted index, and the preserved historical
CSV files are not measurements of this candidate.

### Instrumentation correction

The first adapter renamed `malloc` to an accounting wrapper while leaving
WF's declaration as `declare ptr @wf_cost_allocate(i64)`. Clang inferred
`noalias nonnull` from the C wrapper's body but the separately optimized WF
module lost the allocator information associated with the `malloc` name.
Its rebase then had to consider aliasing between the old and fresh backings.
The provisional timing gap cannot be called an ordinary compiler cost.
The adapter now preserves the wrapper's true fresh, nonnull return contract
with `declare noalias nonnull ptr @wf_cost_allocate(i64)`: the wrapper adds
a private header to a new allocation and exits on failure. No argument alias
attribute or memory-effect promise is added. Independent checksum and exact
allocation formulas are rerun before accepting corrected measurements.

### Reproduction and units

The host is Apple M1 Pro, eight logical CPUs, arm64 macOS 26.6.2 / Darwin
25.6.0, with Apple Clang 21.0.0 (`clang-2100.3.34.2`). The guarded local command
is `make -C research/experiments/container-representation/deque-library measure`.
Normal mode permits O2 inlining. Retained mode marks the library, workload,
make and consume helpers noinline; primitive operations inside rebase/drain
remain ordinary optimized element operations. C endpoint helpers take the
address of the pointer owner, just as WF takes `&Box<Ring<T>>`, and return the
new length where WF does. Both retain the final `free_empty` boundary. C keeps
one external trace wrapper in normal mode and the additional generic trace
boundary in retained mode, matching the WF translation-unit boundary.

Two complete runs reverse normal/retained executable order. Each includes
two cohorts with reversed implementation order and eleven samples rotating
the first implementation. The CSV's `run`, `cohort`, and `traces` fields
preserve those distinctions. Nonzero-round figures divide elapsed time by
`rounds * count` and describe amortized complete churn/rebase traces per
element, including setup and final cleanup. Setup figures divide by `traces`
and describe one complete zero-round trace. Ratios use medians within a
run/cohort. Every word of a wide record enters the checksum, whose cost is
included; close record ratios are not a universal native-parity result.

Incremental experiment construction took 0.55 seconds (WF emission, native
compilation/linking and optimized IR), then `make measure` including its
correctness checks took 6.63 seconds. This excludes a Rust compiler build and
a cold runtime-object build. `CLOCK_MONOTONIC` samples on this host have
microsecond granularity; normalized decimal places are not clock precision.
Ratios around one percent from equality are not a precise gain or parity
claim: quantization, cohort variation and the checksum's substantial share of
record work limit that inference.

The measured compiler SHA-256 was
`4311534e72d2df6ccb2f6043465a801f42f98a70f940f1249755267da2b8c31e`.
The subsequent const-forwarding repair's compiler
`5d3a09f7f07b09ac333615830340878a24daa81ef42d5e61c3a5200098aa84c5`
emitted byte-identical raw LLVM for this source. No specification change is
part of this experiment. Artifact SHA-256 values fix the corrected baseline:

| Artifact | SHA-256 |
| --- | --- |
| `spec/kernel-spec.md` | `2b4df9e688befb4d7ea1a58d7b95dfe56f436ab0739caee9db934763abd21b35` |
| `lib/containers/deque.wf` | `4f4e617e4e33733cea6d57367e1933e0d82a07d3983c47e00fb144e1c812aa82` |
| `deque-library.wf` | `5ba91e4e58c9eddd89c1d781ae1496ba673a6ac684c14890fc331579217344f2` |
| `deque-costs.c` | `85e1d18de6445a76763054b4e5d367a63e61a724593c5a367287b208e186c0bc` |
| `Makefile` | `7fa03e4534e1e3062276b2a40008485d6d1783bcb82171bb901015b4ba54580c` |
| Raw compiler LLVM | `723ab1d7b246e941e8ec4199619068cc9e59171c1ca4d2944cf70f49acce9c08` |
| `measurements.csv` | `42b44b68328776a6f12259c25f1f1ee449b01629cf60b852f56f961197919736` |

### Integration verification at dcbfdc0f

After merging main `7127bcb6`, the freshly built compiler at `dcbfdc0f` had
SHA-256 `9653362a116997dd34558b9da6436dfb343018631b4fabcaa255e420a0c80005`.
The library, workload and C driver retain their baseline source hashes above.
The raw WF output and both normal/retained optimized modules are byte-identical
to the preserved baseline, as are both C controls' optimized IR. Each old/current
optimized WF module was also compiled with Apple Clang 21.0.0 on the same
arm64 host using `clang -O2 -Wno-override-module -S -x ir`; the complete
assembly files compare byte-for-byte equal, without normalization.

| Current artifact | SHA-256 |
| --- | --- |
| Normal optimized LLVM | `fa94c2c058791b408fdff75be414408bd830f54e4bfa532e12ad21325ca61cb0` |
| Retained optimized LLVM | `9a314178b5abc2f1b631f65427bd7ce16b985bfc11b5521138c2ac60391349fd` |
| Normal old/current assembly | `6ffaa7605a55f50d54cf89308f9f12e92cd31e1d1835e8cc7fc3afcb03c3ec4d` |
| Retained old/current assembly | `bbb59f2653b7818e0f78b3ae857c933776e96384b09565b7f3ba1372f28cc096` |

This is emitted-module evidence from optimized IR, not a byte-identity claim
for the complete linked native image. The runtime's C, header and LLVM
sources and `compiler/runtime.mk` were unchanged. Current normal/retained
harnesses passed all 2,592 correctness executions again. Incremental
construction took 0.80 seconds and the executions plus retained-call checks
took 0.73 seconds. No new timings were collected: the original CSV remains
the dated baseline, with no claim of a fresh measurement, a different
toolchain's output, or untested instantiations.

The maintained `probe-scalar-gep` target was executed once in the same guarded
interval, taking 2.13 seconds for construction and checks. Its baseline,
fact-only and split-only variants each passed 1,296 oracle executions; reading
the current scalar `bb32` reproduces 4/4, 1/1 and 4/4 i64 loads/stores. The
target saves `deque-gep-{baseline,nuw,split}.opt.ll` and corresponding
`.check.txt` files under `.build/`. Their optimized-IR hashes are:

| Probe variant | SHA-256 |
| --- | --- |
| Baseline | `50b2c6766470eea22411ad6664d9bc97cbb5835b1ff5bcbeb997f023df490a4a` |
| Unsigned fact only | `5b42af64e26d2717df662a72cb9346b0419b83f9fb50a63eb87802f047591e2c` |
| Split only | `db139c31763151222cbb33b39630699a548eded6192d1ad939c53db028e4a5e6` |

The complete Slab/Deque integration interval took 7.51 seconds. This confirms
the optional reproduction wiring and bounded code-generation observation;
it does not add the flag to production lowering or measure a speedup.

Requalification after main PR #80 used merge `4da1710e` (main `95b21cfd`)
and compiler SHA-256
`cb399df104e975c607973f151dd87a4caf2e4a0e9606a468b4ea69d683345a48`.
The library, workload, C driver, Makefile, runtime inputs and CSV were
unchanged. Fresh raw, normal-optimized and retained-optimized WF LLVM were
byte-identical to the saved `dcbfdc0f` artifacts; both C-control optimized
modules retained their exact hashes. The recorded raw/optimized identities
therefore also identify this fresh emission, without normalization.

The guarded `slab-deque-pr80-emission` command requested these Make targets:

```sh
make -C research/experiments/container-representation/deque-library \
  .build/deque-library-normal.opt.ll .build/deque-library-retained.opt.ll \
  .build/control-normal.opt.ll .build/control-retained.opt.ll
```

Deque emission/optimization took 0.28 seconds; the combined Slab/Deque guard
interval took 1.03 seconds. The C targets were already current. The earlier
native checks, assembly comparison and optional-probe evidence are reused
because their inputs are unchanged. No native relink, execution, probe or
timing was repeated. The original CSV remains historical timing evidence,
with no fresh performance or wider toolchain claim.

#### v0.64 predecessor-lowering comparison

Main PR #88, merged as `ce9a3870` from `e8e1c411`, releases v0.64 and changes
`place_front`'s predecessor calculation to `(head == 0 ? capacity : head) - 1`.
This avoids an overflowing intermediate for large header-only capacities,
but also changes positive-stride scalar and record endpoint code. Unlike the
earlier integrations, this requires a fresh timing comparison. The library,
workload, C driver and controls have not changed. The old
[`measurements.csv`](measurements.csv) remains the pre-v0.64 baseline.

The gate compiler SHA-256 is
`b68db16443f0606ef5d3217014fbdb1bdae0ea01bc23452eff27e2626bbc24ff`;
the active specification is
`bc4d465d698a63518d4c768bfa0b4147afa15e328e32f8adac3f27980c7a21ed`.
The current Makefile is
`1facd9eafce1baabac65e456df6ba962b46abd45a944af4eeeaf5f0afbd4e020`:
its difference from the original measured Makefile is the optional probe,
not the check or measurement matrix.

| v0.64 artifact | SHA-256 |
| --- | --- |
| Raw compiler LLVM | `a32c7d14640f6f855f67149281fa839c65d3d30597c96bc50d8de8e012fa32b6` |
| Normal optimized LLVM | `e36eeb449413e522d28c2faeccb5fedf5e0f28e98de76888a826cda25e08727e` |
| Retained optimized LLVM | `42db1665f36ee102687df07507c7278a51d07af906a08d1357a36d482e5301dd` |
| Normal current assembly | `ab1f40e2a8871e2377b2999d330d9bf910663cda828ac0f30a8c47ed3bc06040` |
| Retained current assembly | `bf028f3e74c9f816827456d4fa6e04f8de5a5ab2671930e3804d8bab65cfe08b` |

Comparison with the saved `dcbfdc0f` optimized modules isolates retained
changes to `wf_deque_push_front$instance$47` (scalar),
`wf_deque_push_front$instance$56` (record), and their primitive bodies
`wf_place_front$instance$63` and `$68`. Their predecessor arithmetic and
address scheduling differ in generated assembly; normal mode contains the
corresponding inlined reverse-churn change. The retained record-copy counts
above are unchanged. The new raw module also contains the POSIX resource
writer's EINTR retry helper, which is removed by optimization under the
accounting allocator's nonnull contract; it is not a timed-path difference.
Both C-control optimized modules and the linked runtime's C/header/LLVM
sources and `compiler/runtime.mk` are unchanged. The assembly comparison
uses the same optimized-IR-to-arm64 command as above and concerns the emitted
WF module, not the complete linked image.

The current scalar forward hot block still has four i64 loads and four stores.
The maintained probe still structurally selects exactly four positive-stride
u64 Ring payload GEPs. The historical `dcbfdc0f` probe result is reused, not
rerun or relabelled as a v0.64 execution: the changed predecessor is in the
other endpoint path and does not supply the missing unsigned address fact.
No production GEP flag or container API change is part of this comparison.

Compiler construction with `make -C compiler build` took 46.94 seconds;
Deque emission/optimization took 0.28 seconds. The complete guarded
Slab/Deque build, emission and conditional assembly comparison took
48.38 seconds. These construction figures are separate from the native
check and timing run below.

The new [`measurements-v0.64.csv`](measurements-v0.64.csv), collected on
2026-09-22 on the same host/toolchain, contains all 6,336 samples from the
unchanged matrix. Its SHA-256 is
`e297a90a7ce0f56f116c9697aece4f10dcf0a4d6c855e97e0a7b1cb89db26638`.
Reproduce through the existing target, then preserve its output separately:

```sh
perl .github/run-check.pl deque-v64-measure \
  make -C research/experiments/container-representation/deque-library measure
cp research/experiments/container-representation/deque-library/.build/measurements.csv \
  research/experiments/container-representation/deque-library/measurements-v0.64.csv
```

Both modes again passed 2,592 correctness executions in total, and retained
IR still has 20 WF and 40 C helper call sites. Every timing row passed the
same checksum and allocation ledger checks. Incremental native construction
took 0.58 seconds. Observing the existing `make measure` output boundary
after its retained-call check separates 0.790 seconds for correctness from
5.740 seconds for the timing matrix, including each phase's Make/output
overhead; total check plus measurement was 6.529 seconds. The whole guarded
construction/check/measurement/copy interval took 7.14 seconds.

The following table uses the same units and per-run/cohort median method as
the original table. It is a new complete cohort, not a replacement of the
historical values or an isolated instruction-latency comparison.

| Mode | Payload bytes | Trace | WF ns/step, n=4096 | WF / loop C | WF / bulk C |
| --- | ---: | --- | ---: | ---: | ---: |
| normal | 8 | forward churn | 3.204 | 2.256–2.405 | 2.283–2.349 |
| normal | 8 | reverse churn | 3.601–3.754 | 1.196–1.230 | 1.194–1.240 |
| normal | 8 | wrapped rebase | 4.883–5.066 | 1.648–1.909 | 1.648–2.683 |
| retained | 8 | forward churn | 4.791–5.127 | 0.994–1.035 | 1.000–1.028 |
| retained | 8 | reverse churn | 5.157–5.554 | 1.032–1.158 | 1.013–1.158 |
| retained | 8 | wrapped rebase | 8.240–8.392 | 1.028–1.098 | 1.089–1.201 |
| normal | 256 | forward churn | 34.363–34.637 | 0.978–1.026 | 0.987–1.014 |
| normal | 256 | reverse churn | 34.424–34.821 | 0.986–1.008 | 0.977–1.006 |
| normal | 256 | wrapped rebase | 46.997–48.676 | 0.976–1.025 | 0.993–1.108 |
| retained | 256 | forward churn | 38.086–39.795 | 0.948–0.991 | 0.952–0.992 |
| retained | 256 | reverse churn | 37.994–38.422 | 0.932–0.973 | 0.937–0.979 |
| retained | 256 | wrapped rebase | 57.159–58.472 | 0.919–0.989 | 0.977–1.041 |

The zero-round setup/cleanup cohort is preserved too; times are microseconds
per complete trace, with no subtraction from the operation traces:

| Mode | Payload bytes | n=16, us/trace | n=256, us/trace | n=4096, us/trace | WF / loop C |
| --- | ---: | ---: | ---: | ---: | ---: |
| normal | 8 | 0.047–0.048 | 0.539 | 8.500 | 1.690–1.971 |
| retained | 8 | 0.067–0.069 | 1.211–1.273 | 19.625–21.250 | 0.959–1.024 |
| normal | 256 | 0.546–0.565 | 8.625–8.695 | 142.250–143.875 | 0.942–0.996 |
| retained | 256 | 0.689–0.711 | 10.852–10.906 | 175.875–181.500 | 0.948–0.970 |

The changed reverse path still has a scalar normal-mode gap (1.196–1.230
against loop C, previously 1.175–1.220), while record traces remain
competitive in this matrix. The unchanged scalar forward code still costs
2.256–2.405 times loop C. These ranges do not isolate a causal percentage
for the predecessor rewrite: the old and new cohorts ran at different times,
retained scalar reverse results vary more in the new cohort, and the samples
still have one-microsecond granularity. The evidence supports retaining the
same lowering-improvement questions, not asserting a speedup, regression
threshold or general native parity from small ratio differences.

Requalification after main PR #87 used merge `fd41dcc8` (main `8d6da723`),
which releases v0.65, and compiler SHA-256
`0ab0f5730590828c511d0a0d4d90d3131654377ea020e5634f5a0800ff9edb96`.
Fresh normal and retained optimized WF modules compare byte-for-byte equal
to the saved v0.64 artifacts in the table above, without normalization;
both C-control modules are unchanged too. Library, workload, harness,
native runtime inputs and both CSVs retain their identities. Gate-profile
compiler construction took 46.75 seconds; Deque emission/optimization
through the same four Make targets took 0.27 seconds, and the shared
`slab-deque-v65-emission` interval took 1.14 seconds. This confirms emitted
WF module correspondence on the recorded Apple Clang 21 / arm64
configuration. No native relink, execution, probe or timing was repeated;
the v0.64 CSV remains the dated measured cohort, not a fresh v0.65 result.
