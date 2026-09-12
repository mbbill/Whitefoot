# Removal sequence and verification

This is the execution proposal for C1, not implementation work. The work
remains on PR #30's existing line. The paused D8 library, gate and regression
drafts are excluded; no change to the active specification or compiler was
made for C1. The [inventory](INVENTORY.md) pins the baseline and sizes;
[CASES.md](CASES.md) gives each conformance migration and its rule change.

## First settle the ordinary interfaces

The owner selects the alternatives in [DECISIONS.md](DECISIONS.md), especially
linearity, factory vs reservation, input aggregation, TCP half cleanup and
loss of the current PAR-3 permission. The fixed absolute rule is not reopened.
The proposed source boundary must have no dependence on whether a function's
definition is written in WF or supplied by linking.

Before deleting a path, write its ordinary counterpart using only existing
forms. These are the decisive paired witnesses:

| Witness | What would falsify the replacement |
| --- | --- |
| Same ordinary signature/row/contract supplied as a WF definition and a prelude definition, including a behavior actual | Different type, loan, obligation, effect-kill or overlap verdict based on definition provenance. |
| Owned Box/run/opaque transfer through locals, a returned value and an exclusive referent replacement | An effect or postcondition still requires FN-1 owned ancestry, or a real borrow origin is lost with that ancestry. |
| Explicit file close on success, failed open, early return and partial aggregate move | A linear owner is lost, or a native close occurs through opaque drop rather than an ordinary call. |
| Two opens through one factory, and the ordinary reservation alternative if selected | Hidden factory mutation, retained early-release permission, refusal losing an owner/credit, or a throughput claim unsupported by measurement. |
| Read/write range requirements plus nonzero successful endpoints and failure leaving data unchanged | Any SystemRange/S10 operation-name fact remains, or a forced impossible runtime branch replaces a static precondition. |
| Two connections whose direction halves are crossed, then closed in both orders | Ordinary construction leads to mismatched release, double close or wrong credit. |
| Standard streams redirected together, two opened handles to one file, stateful listener accept | Native shared changing state is treated as independent solely because two opaque wrappers are different values. |
| Ordinary function entry with a Heap region argument, and a contract-bearing ordinary selected function | A hidden command label/brand or entry proof bypass is required. The launcher must meet the callable's ordinary preconditions, not execute a special requires fallback. |
| Existing staged traversal/fanout, sequential and each retained parallel mode | Deleted PAR-3 permission silently reappears through a native tag, a loan ends before return, or output/refusal behavior changes. Performance loss is measured separately from source legality. |

No counterexample to expressibility has been established in C1. Cross-paired
TCP halves defeat the unchanged *API*, and the staged examples lose an
optimization; neither establishes that ordinary values/functions cannot
express the program. If a concrete witness does establish that limit, stop
and bring that program to the owner instead of adding a replacement mechanism.

## Spec and compiler changes, in dependency order

One future specification amendment must contain the complete consistent
language change, bump the active version, and archive the outgoing exact
bytes. Do not change archived versions, distribute the semantic change over
inconsistent specification states, or count C1 prose as language authority.
Implementation commits can separate invariant-bearing repairs underneath that
amendment; the final revision must pass canonical `make check`.

| Work | Exact surfaces | Size estimate and completion evidence |
| --- | --- | --- |
| Ordinary declaration boundary | PRE-1, TYPE-2/5/6, FORM-8, GRAM-11, OP-1, FN-3/4/5/8, PROG-1; delete SYS-1/2/3 and GATE/LEDGER boundary classification. `resolution/catalog.rs`, domain installation/lookup/roles, checked nominal/function model, ordinary call checker and behavior binding. | 30 nominals + 29 signatures re-homed; remove the 307-record system identity. Roughly 1–2 KLOC of mixed resolver/type/call edits, plus removal of the dedicated 413-line system-call checker and most system catalog data. Match ordinary-definition/prelude-definition verdicts. |
| Ordinary entry and brands | GRAM-2/FN-7/PROG-3, PROV-1/STOR-1/BLK-4 references; syntax entry form, semantic entry form, driver/launcher, grammar generator and generated syntax. | Dedicated entry modules: 154 + 462 LOC. About 0.5–1 KLOC of integration changes; generated table churn is mechanical and not hand-written feature size. Verify ordinary signature/region/contract behavior and launcher inputs/status. |
| Empty opaque release and place effects | SYS-5/STOR-3/EFF-1..5/PROV-6/FN-1/FN-4; remove state_origins.rs (1,235), result_state_origin.rs (1,382), release-owner/target metadata and ReleaseEffectMismatch. Update effects, cleanup, assignments, kernel calls, behavior checks and IR consumers. | 2,617 LOC of dedicated routing plus about 2–4 KLOC of mixed edits. Retain ordinary provider-write walks and loan provenance. Test replacement values, exact rows, all close/refusal outcomes, and ordinary scalar/run/Box negatives. |
| Ordinary range proof | SYS-8/9, ENT-3.S10, ENT-2/ENT-6, CALL-3/5, DIAG-1/2, ERR-4; remove SystemRange, system-result origins and per-op result bounds in entailment and backend target validation. | About 0.5–1.5 KLOC of mixed entailment/IR deletion and 10 explicit view-range signatures/contracts. Prove ordinary requirements and Result.Ok bounds using existing FN-8/FN-9, with exact negative snippets retained. |
| Generic call runtime | Delete target_action.rs (225), qualification.rs (1,897) as the external contract table, and staged_permission.rs (1,860). Rewrite PAR-1/2; delete this PAR-3. Remove dedicated SystemCall emission/completion adaptation (4,321 + 1,391 LOC); migrate ordinary storage/ABI/call/parallel consumers. | 3,982 LOC of dedicated summary/qualification/staged modules; 5,712 LOC of native emitter code must leave compiler operation dispatch, with useful bodies reimplemented as ordinary library functions. Native engine/scheduler LOC is retained implementation, not claimed deletion. Verify ordinary direct-call ABI and loan duration. |
| Native bodies and API fixtures | All current 29 operations and selected replacement closes/results; host-string/path/error routines; bootstrap and completion adapters; native Linux/macOS/Windows harnesses. | Roughly 4–7 KLOC of moved/adapted implementation, highly dependent on factory/TCP/runtime selections. This is a cost estimate, not newly invented language support. Preserve no-allocation/short-transfer/encoding/refusal behavior where the selected API promises it; measure changed behavior explicitly. |
| Conformance and generated source | Every row in CASES.md; compiler fixtures; maintained corpus/program generators and their launcher assumptions. | 779 cases inspected; 754 contain an exact command main header. Most are scaffold-only; 91 have additional migration codes in this register. Compiler fixture modules are counted separately in INVENTORY. Expect several thousand mechanical source-line edits, with semantic re-derivation concentrated in the classified cases. |
| Documentation and ledger | Active rule index/grounds in `spec/derivation/derivation-ledger.md`; compiler README, docs/patterns, constitution and affected live guidance in INVENTORY. Historical research and decision alternatives are preserved with supersession pointers as appropriate. | About 0.5–1.5 KLOC of active prose/index changes. No new memory node for an unsettled option. `docs/roadmap.md` remains a reference map, not a work item or capability authority. |

The specification itself has approximately 630 lines in the complete SYS/HOST/
PATH/QUAL block plus entry, routing, release, diagnostic, gated-boundary and
parallel clauses elsewhere. Expect roughly 800–1,100 old lines to be deleted
or rewritten, with ordinary prelude declarations and rule edits replacing
part of them. Eight dedicated compiler modules total **7,628 LOC**; that is
the old audit footprint, not net savings. Catalog and emitter removal is
additional; ordinary code extracted from them and native implementations still
have to exist. The estimates overlap where a mixed file serves two steps and
must not be added into a precise total or schedule.

## Evidence and completion

1. In the amended specification and checked IR, no system domain, declaration
   ID, qualification identity, native release row, SystemCall, SystemRange,
   state-route summary or suspension/milestone summary survives. Search is a
   final check on the dependency inventory, not its replacement. Ordinary
   kernel storage primitives and source loan suspension are not false targets.
2. Re-derive every CASES.md migration under the selected ordinary declarations.
   Keep original failing values/operation chains and native fixture outcomes.
   Retire old rule-specific assertions with the named rule change in the same
   revision. No source rejection is replaced by an unsupported/backend result.
3. Verify emitted ordinary call ABI, opaque layout ceiling, explicit closes,
   view proof erasure, and facts-off correctness. Keep scalar, aggregate,
   cross-worker and three-mode witnesses; no special native-call test path.
4. Run full canonical `make check`, then native host/runtime CI on the exact
   pushed revision. Record all verdict changes, retired assertions and source
   migrations in the PR under the repository's rule 4. Only the owner merges.
5. Measure existing traversal/TCP/staged workloads and container witnesses with
   retained helpers and existing controls. Attribute ordinary call overhead,
   lost PAR-3 overlap, factory contention and native implementation costs
   separately. A legal sequential program is not evidence of performance parity.

The C1 proposal is complete when its inventory, design, owner selections and
execution plan are reviewed and pushed. It does not complete D8, select these
API alternatives for the owner, or authorize an implementation claim.
