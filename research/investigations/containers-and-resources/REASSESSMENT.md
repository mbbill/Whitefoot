# Container architecture reassessment

This is a research result from the independent workload, critical-case, and semantic
reviews of 2026-09-06, including their cross-review. It does not select a replacement
architecture or amend the active specification. It serves the next container
architecture experiment; incorporate its surviving findings into `DESIGN.md` when
that experiment settles the choice, or remove it if the investigation is abandoned.

The implementation examined was `ea97222adc0aff481df320f4c39624eb6813e488`, rebased as
`eff095c701b473a0108822a16cd1e1274c43f621` onto main
`8a5ad14b1d9093117dff1bd437d4de9d7aa564c4`. The rebase adds exactly main's intervening
tree changes: container implementation, active and archived specification bytes,
scheduler, and completion code are unchanged from the examined container revision.

## Selection ground

[Project instructions](../../../AGENTS.md), the
[constitution](../../../docs/constitution.md), the
[active specification](../../../spec/kernel-spec.md), and the corrected
[development decisions](../../../mcts_mem/whitefoot/development-workflow.md) supply
the ground. Historical design selections and compiler convenience do not define a
new architecture's requirements. Current language behavior remains the active
specification's, including its proof rules; historical claim/trap language does not
authorize a writer-accessible escape.

- Performance is the reason to exist. Evaluate ordinary accepted source against an
  equivalent reference implementation, including its layout, algorithm, resource
  contract, and generated machine code. A smaller operation table is not evidence
  of a better design. Name the advantage over Rust in performance, checked safety
  without writer-accessible unsafe, or enforced fast source forms.
- Safety and proof authority cannot be traded away. Memory validity, exclusivity,
  initialization, ownership, and partial-operation domains must be established by
  the specified deterministic checker. No proof-search timeout, work budget, or
  runtime fallback selects acceptance.
- The default source form must reliably expose the appropriate fast representation.
  An expert-only rewrite does not establish that ordinary accepted forms are good.
  Necessary dynamic program state is allowed; redundant safety scaffolding is not
  justified merely because it makes a source program compile.
- An `existence-only` decision separates a derived need from a minimality-selected
  form. Read its rationale and reconsideration condition. New workloads can
  disprove the adequacy of the form without disproving the need.
- D17 commits to checked representation authority as a long-term lane. It does not
  require implementing a universal storage proof framework before the next useful
  program. Such machinery earns priority by resolving an immediate experiment.

Frequency determines optimization and investigation priority, not whether a needed
capability may exist. A rare workload can veto a representation for its own hard
contract; it does not thereby get to impose its metadata on every dense array.
Do not average away a hard-contract failure with wins on common cases. Conversely,
do not turn contiguous layout, stable addresses, no allocation, or no runtime tags
into universal requirements without a workload-specific derivation.

## Independent obligations

| Obligation | What must stay distinct |
| --- | --- |
| Store provenance | A store brand identifies the provider; it does not identify an allocation or a recycled object generation. |
| Object identity | Identity, address stability, and current ownership are different properties. |
| Ownership | Every live affine or linear element has exactly one owning responsibility, including failure and early-return paths. |
| Loans | Authorized ranges remain valid through their promised use; copying a descriptor does not extend backing lifetime. |
| Logical shape | Length, order, continuity, keys, and occupancy need not describe the same physical representation. |
| Initialization | Only constructed values may be read or destroyed; temporary holes need checked authority. |
| Abstraction | Useful relations must survive helpers, fields, variants, and elements through verified contracts, with sound invalidation. |
| Resources | Allocation failure, peak storage, backing reuse, and destruction are part of the complete operation. |

Runtime occupancy, a ring's head, and a genuine failure outcome can carry necessary
information. Rechecking a property already guaranteed by a returned capability or
verified contract is a different issue. Proof erasure must not discard semantic
facts needed to select efficient lowering.

## Workload evidence and coverage gaps

These are workload classes, not measured prevalence estimates. Existing Whitefoot
examples are biased toward byte storage and dense construction. The compiler's
own Rust implementation is an additional concrete demand source, not evidence that
a full self-hosting project should start now.

| Class | Complete trace to preserve | Local demand source |
| --- | --- | --- |
| Tiny temporary sequences | Empty, a few inserts, occasional spill, helper transfer, clear and reuse | `compiler/src/lowering/builder.rs`: block successors and builder collections |
| Fully initialized fixed objects | Construct, store in a record, return through a helper, repeatedly update | `tests/programs/raw_deflate_dynamic.wf`, `fir_filter.wf` |
| Growing nested owned values | Grow or fail, preserve existing elements and unconsumed inputs, clean up | `tests/programs/growable_vec.wf`; byte-only examples underexercise affine payloads |
| Bulk transformations | In-place shrinking, overlapping copy, and LZ back-reference expansion | `tests/programs/percent_decode.wf`, `utf8parse.wf`, `raw_deflate_dynamic.wf` |
| Retained tails and rings | Append, consume a prefix, wrap, inspect two spans, reuse backing | `tests/programs/wfgrep.wf`, `run_queue.wf` |
| Variable records and names | Many short or empty values, rare long values, nesting, retained references | `tests/programs/byte_string.wf`, `wfgrep.wf` |
| Graphs and coordinated tables | Adjacency, membership, discovery state, worklists, deterministic readiness | `compiler/src/semantic/entailment.rs`: SCC and postcondition scheduling |
| Sparse keyed collections | Insert, find, delete, reuse, maintain index relationships | `compiler/src/lowering/builder.rs`; `tests/programs/option_slots.wf` |
| Partial construction | Fail after several owned values, return every responsibility exactly once | `tests/programs/prefix_expression.wf` and ownership rules; broader failure trace needed |
| Fixed pools | Exhaust, lease, return in a different order, reuse without extra backing | `tests/programs/block_pool.wf` |
| Borrowed stable storage | Publish, suspend, resume, join, consume result, retire, then reuse | Runtime interface supplied by the I/O/runtime agent; container side only |
| Destruction | Early exit with a deep owned structure, bounded or explicit cleanup storage | `PROV-6` release graph; a measured deep-chain case is still needed |

Bulk operations need their own semantics: disjoint copying, overlap-preserving
movement, and LZ expansion reading newly written output are not interchangeable.
Likewise, an owned tree does not cover shared graphs or sparse stable identities.

DMA mapping/coherence, general cancellation, arbitrary strided splitting, and
concurrent reclamation remain boundary questions. Their possible future value is
not a reason to add pinning, atomic metadata, or a registry to every container now.

## What the existing implementation establishes

The following distinctions prevent an implementation gap from being mistaken for
a language design result:

1. **Confirmed implementation limitation:** forming an exclusive view of an inline
   run reports `SemanticUnsupported::ExclusiveViewOverInlineRun`. The relevant path
   is `compiler/src/semantic/check/expressions/flat_storage/slices.rs`. This is not a
   proof that the source operation is semantically invalid. A minimal probe is:

   ```whitefoot
   command fn main() -> status: own ExitStatus pure {
     let empty = fixed_vector::<u8, 4>();
     let built = place_back(vector: move empty, value: 7_u8);
     region {
       let view = mut_slice_of(&uniq built);
       set view[0_u64] = 9_u8;
     }
     let byte = built[0_u64];
     return exit_status(code: byte);
   }
   ```

2. **Confirmed lowering concern:** `compiler/src/backend/emitter/runs.rs` represents
   inline runs as aggregates and stages whole values through storage slots around
   element operations. A 4096-byte fill probe retained whole-aggregate work inside
   its loop after local Apple Clang 21 `-O2` lowering. This establishes a problematic
   machine shape for that probe, not a portable timing result. By-value source
   ownership does not itself require aggregate copying; destination construction
   and in-place updates must be evaluated separately from the source API.

3. **Specified composition gap:** `MSR-3` does not carry inner element measures
   across the run boundary. `CALL-4`/`FN-9` also limit routed result contracts. Pool
   examples expose the resulting pressure for fresh capacity checks. Improving
   aggregate lowering alone cannot supply a missing callable contract.

4. **Specified view restriction:** `VIEW-2` supplies a contiguous view of a
   nonwrapping window, while `VIEW-6` restricts repeated result view types/regions.
   A wrapped ring's two-span consumption needs an actual checked expression; a
   second full backing allocation is not an equivalent solution for a one-backing
   workload contract.

5. **Unimplemented proposal to reconsider:** `DESIGN.md`'s CALL-7/B6 proposal asks
   for nontrivial relations for every measure. Its own examples exclude useful
   front operations and measured generic results. Contract adequacy should follow
   the abstraction and the operation's required domain, not a demand to expose
   every representation coordinate. This is a design issue, not a claimed defect
   in an implemented B6 checker.

6. **Open falsifiers:** an empty run whose element type is explicitly linear,
   full-capacity generic permutation of owned elements, sparse-slot reuse, and
   deep destruction need complete positive and negative executable traces. Static
   reading identifies pressure; it does not establish all their verdicts or costs.

Do not generalize one bad fill loop into a ban on all initialization. The existing
`research/experiments/buffer-initialization-cost/RESULTS.md` measures a case where
one-time initialization is negligible for long-lived reusable storage. Repetition,
element width, lifetime, and the surrounding operation matter.

## Architecture alternatives after cross-review

The first review identified three families. Cross-review showed that they are not
three mutually exclusive complete solutions:

| Direction | Question answered | Strongest challenge |
| --- | --- | --- |
| Compiler-owned state machines | Does the compiler directly own full-array, prefix, ring, and any justified additional initialization states? | A new required layout must not need another trusted family merely to avoid extra storage or redundant work. |
| Checked library representation | Can libraries obtain precise backing and initialized-range authority through checked invariants and finite evidence? | Helpers and representation abstraction must compose without replaying long proofs, exposing private layouts everywhere, or requiring unbounded automatic reasoning. |
| Stable objects and owning stores | Where do stable identity and storage live when values are deleted, recycled, or borrowed across suspension? | Identity must not become mandatory indirection or allocation for dense scans; recycled IDs must not silently regain authority. |

The first two primarily compete over **where representation authority lives**.
The third is a **storage/identity choice** that can accompany either. Full arrays,
prefixes, rings, sparse slots, and stable objects should be compared on their own
required state and conversion costs; no universal layout is assumed.

All three also need compositional abstract contracts. Storage validity alone does
not establish `free + outstanding = total`, graph/worklist membership implications,
or atomic ownership recovery after a two-index insertion fails. This is a separate
language capability question, not something a storage descriptor solves.

A useful adversarial trace has three fixed slots: construct values in slots 0, 1,
and 2, keep 0 and 2 borrowed, discharge the owner in slot 1, then reuse only slot 1.
One contiguous live window plus stable IDs does not by itself express that state.
Ordinary tagged slots, independent slot capabilities, or chunked ownership may
solve it; each must demonstrate its complete trace and cost. This is a prospective
non-I/O sparse-lifecycle case. It does **not** require the current staged runtime
to retire source iterations out of order.

## Evidence needed to select a form

Compare candidates with the same behavior and declared resource contract. Each
candidate must show natural source, helper contracts, initialization transitions,
failure cleanup, final storage placement, and generated machine shape. Pair each
positive trace with a nearby invalid trace: raw read, duplicate ownership, aliased
exclusive ranges, stale identity, missing release, or premature backing reuse.

Record allocation count, bytes copied/initialized, peak live storage, stack demand,
steady-state machine work, proof size, diagnostic usefulness, and checker cost.
Measure only the dimensions relevant to that trace. Runtime state inherent to the
algorithm is not a proof overhead. Reference timings must use comparable algorithms
and layouts; source occurrence counts are not execution hotness.

The smallest useful selection experiment couples one common path with one critical
path that distinguishes the candidates, crosses a real helper boundary, and ends
the relevant storage lifetime. A choice becomes actionable when these programs
run, nearby invalid programs are rejected for their rules, and the architecture
tradeoff is visible in source and cost evidence. Remaining limits should be named;
an ever-growing scenario list or a universal framework is not the completion test.

## I/O boundary and validation

Keep the runtime's acquire/publish/join/read-result/release protocol opaque. A
published callee may change workers and suspend. Its storage must remain valid
through join return, result consumption, and the corresponding retirement; DONE
alone is insufficient. Form an address-bearing view only once its backing is in
the required stable location. Current slot sizes and experimental worker policies
are not language premises. This review adds no I/O, pinning, lease, or scheduling
interface and makes no change to the other agent's worktree.

Before rebase, the compiler build and the six ordinary `programs::runs` tests passed.
After rebase, the three changed unreadable-path program tests passed. The rebased
tree was checked against both input revisions. Full root `make check` has not run;
these results establish only the stated build and test coverage, not merge readiness.
