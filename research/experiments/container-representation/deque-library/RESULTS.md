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
[`measurements.csv`](measurements.csv). Both normal and retained executables
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
| Rebase transfer loop | 1 x 256-byte memmove | 1 x 256-byte memcpy |
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

The fact-only variant also permits initial-fill vectorization. This isolates
why these metadata operations survive optimization; it supplies no measured
timing recovery percentage. Construction and all three oracle checks took
2.26 seconds under the shared guard. Production lowering is unchanged. A
general improvement still needs qualification over all admitted storage/index
domains, including zero-sized elements, a compatible path for older LLVM,
and the same-source timing matrix. The bounded positive-stride u64 probe does
not establish that those other cases may receive the flag.

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
