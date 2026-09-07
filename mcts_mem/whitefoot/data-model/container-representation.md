- Represent owned aggregates through general typed storage places, with explicit
  initialization/ownership state and lifetimes. Use caller-provided result
  destinations and consumed-place reuse when sound; source value transfer is not
  permission to duplicate owners or a requirement to copy an entire payload at
  every element operation. Fields, indices, and cell dereferences share the normal
  place path.
- Distinguish full fixed arrays, initialized prefixes, and circular windows.
  Persistent fixed extent and full initialization belong to the relevant type or
  state; variable length and head are not universal array metadata. Placement and
  stable identity are separate axes, with no mandatory handle/store indirection
  on dense values. Current specification bytes still define source behavior.
- Keep initial public storage transitions compiler-checked. Make finite ranges,
  loans, storage identity, and initialized responsibility explicit internally, but
  do not mistake a concrete checker model for a verified symbolic library or a
  production erasure path. General source-library representation authority remains
  the D17 direction, deferred until an exact checked implementation and a real
  workload establish its soundness and usefulness.
- Prefer inherent fixed-block type facts and existing verified count contracts
  for the measured pool. Do not infer general element-refinement requirements
  from a fact that a fixed-capacity type already preserves. Do not infer that
  dynamic capacities or graph relations can always be reduced to constants.
- A checked empty run may discharge its own backing responsibility after every
  element obligation is gone, with the provider and loan rules still satisfied.
  Emptiness of a field does not discharge an independently declared linear wrapper.
  This is a selected amendment, not a claim that today's empty-run rejection is
  already fixed.
- Finite projected result contracts use the normal entry snapshots, publication,
  and write invalidation. Two-span results must carry each physical range and
  origin through helper boundaries; a second descriptor without checked sibling
  disjointness and lifetime is insufficient. Do not require nontrivial published
  relations for every internal representation coordinate.
- Destination selection preserves source evaluation, allocation failure, and
  release order. Acquire backing first in source when allocation-first construction
  is required; do not hoist an allocation across observable initialization just to
  obtain destination passing. Retain every partial-construction responsibility.
- A published/borrowed address lasts through its actual use, including staged join
  return, result consumption, and corresponding retirement. DONE is not reuse
  authority. The runtime interface stays opaque; no private slot size or worker
  placement policy selects container correctness.

## Facts

- 2026-09-06 selection ground: main's `existence-only` correction separates a
  derived need from a minimality-selected form. The independent review and
  executable experiments compare forms against performance, checked authority,
  default source shape, and concrete resource contracts. Workload frequency
  selects priority, not whether a necessary capability exists. The complete
  selection and first implementation criteria are in
  [the container assessment](../../../research/investigations/containers-and-resources/REASSESSMENT.md).
- 2026-09-06 measurement: on one arm64 macOS/Clang 21 run, the dense construction
  plus four-update trace at 16/256/4096 `u64` elements measured approximately
  408 ns/176 us/43.9 ms in the existing compiler versus 65 ns/1.50 us/29.7 us in
  the same-layout native local-build/value-return control. Retained whole-payload
  transfers explain an adverse work shape; at 4096 elements the Whitefoot entry
  and its constructor have simultaneous static frames totaling 229,648 bytes,
  excluding platform helpers. This is a workload result, not a general language
  ranking. [Sources, 168 raw samples, controls, and limits](../../../research/experiments/container-representation/dense/RESULTS.md).
- 2026-09-06 control: native whole-value append also retains copies, whereas a
  separate noinline aggregate-return control writes directly to a caller's `sret`
  destination. Value semantics and physical aggregate copies are distinct; the
  result does not show that LLVM eliminates all move chains or that every ABI has
  identical cost. The native controls are not checked Whitefoot implementations.
- 2026-09-06 source evidence: five complete lifecycle programs run, including pool
  count conservation and static-capacity block checkout/mutation/return. Capacity
  survives through the field type; the neighboring full-initialization assertion
  does not. Boundary-element incidental measures, projected result contracts, and
  empty linear-owner discharge are separate source limitations. Two boxed-block
  cases expose incorrect compiler rejections for nested region inference and owned
  cell measures; they do not refute stable fixed storage. [Exact 15 outcomes and
  source](../../../research/experiments/container-representation/lifecycle/RESULTS.md).
- 2026-09-06 finite-model evidence: checked interval tokens account for all 510
  live sets at capacities 1..8, versus 184 single-window sets, and pass independent
  occupancy/conservation checks through cleanup. Two-span loans and middle-slot
  reuse are checked. The model has concrete endpoints, anonymous linear counts,
  and one storage context; it establishes neither public symbolic proof authority
  nor runtime token overhead. [Model and limits](../../../research/experiments/container-representation/authority/RESULTS.md).
- 2026-09-06 correction: removing the middle element of a capacity-three full run
  is not a sparse-window falsifier: remaining slots 0 and 2 form a circular window.
  Capacity four with initial live slots 0,1,2 and raw slot 3 supplies the actual
  `0101` separating state. An executable near-neighbor check corrected the paper
  claim. Cross-review also strengthened the model's independent oracle to retain
  the obligation count from trace entry instead of recomputing it before cleanup.

## Moves

- Selected general place/result-destination support as the first implementation,
  including the semantic field/index/cell/borrow support needed to turn the frozen
  wide-record and inline-view probes into executed positives. Require element-sized
  loop work, correct failure/cleanup, and the same-workload measurements; keeping a
  known unsupported verdict does not complete that slice. Broad migration and
  retirement wait for replacement capability.
- Superseded the earlier integrated container dossier's universal-window and
  aggregate-reconstruction defaults. Retained providers, checked ownership, and
  working contract transport. Did not replace the separate I/O/resource research
  or amend the active specification in this decision.
- Deferred generalized representation privileges, arbitrary sparse stores, global
  stable handles, and quantified element refinements as prerequisites. Reconsider
  when a real frozen workload cannot meet its behavior/resource/cost contract with
  the selected states, or a complete checked alternative demonstrates a better
  form. Minimal operation count, a successful workaround that changes the contract,
  and majority agreement among reviewers are not selection evidence.
