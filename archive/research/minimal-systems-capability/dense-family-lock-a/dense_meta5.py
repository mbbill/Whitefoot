#!/usr/bin/env python3
"""Cumulative candidate-specific META-5 rows for the dense research lock."""

from __future__ import annotations


META5_FIELDS = (
    "delta_id",
    "candidate_ids",
    "public_spelling",
    "normative_rule",
    "grammar",
    "type_ownership_borrow",
    "effect_exit_drop",
    "diagnostic",
    "checker_proof_state",
    "trusted_fact_path",
    "lowering_codegen",
    "artifact_reporting",
    "tests_and_hostile_review",
    "derivation_and_necessity",
    "protected_baseline_effect",
    "authorization_status",
)


META5_ROWS = (
    {
        "delta_id": "META-DENSE-COMMON",
        "candidate_ids": "ALL",
        "public_spelling": "No common writer-visible spelling. Each isolated candidate toolchain admits only its candidate-specific row below.",
        "normative_rule": "Common DenseState, ownership events, member/outcome contracts, payload envelope, failure classes, ZST policy branch, algorithms, and protected controls are observational requirements rather than one shared mechanism.",
        "grammar": "No common production.",
        "type_ownership_borrow": "All arms preserve affine single ownership, explicit move, no writer-visible uninitialized T, root-and-version provenance, and rejection of incompatible live borrows.",
        "effect_exit_drop": "All arms preserve current effect exhibits and trap-aborts-without-unwind. Every normal exit leaves one valid owner, returns the frozen owners, or performs the arm's exact structural action.",
        "diagnostic": "DENSE-COMMON-UNKNOWN-STATE, DENSE-COMMON-WRONG-OWNER, DENSE-COMMON-WRONG-VERSION, DENSE-COMMON-DEAD-SLOT, DENSE-COMMON-OVERLAP, and DENSE-COMMON-DEFERRED-PAYLOAD are mandatory diagnostic identities.",
        "checker_proof_state": "Owner identity, allocation root, structural version, capacity bound, live ranges, dead ranges, active borrows, and offered/returned/destroyed owner sets are explicit proof components.",
        "trusted_fact_path": "Only the reviewed fact ledger may authorize payload access or optimizer metadata. Facts-off accepts the same programs and retains checks.",
        "lowering_codegen": "No common runtime field, flag, branch, dispatch, allocator path, or container recognition is permitted. Candidate-specific lowering is charged independently.",
        "artifact_reporting": "Every accepted source emits schema-versioned transition, fact-producer, fact-consumer, invalidation, cleanup/drop, and structural-cost reports; absent events are explicit zero rows.",
        "tests_and_hostile_review": "Every arm binds every candidate-neutral trace and fact attack, plus B-FIX, B-P2, W-SMALL, W-GAP, and H-FLATSET. Fact channels and final bytes receive independent hostile review.",
        "derivation_and_necessity": "Current fixed fully initialized Copy buffers cannot express affine spare capacity, move-out, relocation, partial live sets, or exact live-set destruction. This row selects no mechanism.",
        "protected_baseline_effect": "Exactly zero: the protected registry requires source verdict, layout, raw IR, normalized optimized code, calls, traps, facts, and structural counters to match its frozen baseline.",
        "authorization_status": "RESEARCH_ONLY_NOT_NORMATIVE_NOT_IMPLEMENTATION_AUTHORITY",
    },
    {
        "delta_id": "META-DENSE-OD0-COMMON-SUBSTRATE",
        "candidate_ids": "ALL",
        "public_spelling": "Conditional experiment-only common substrate under OD-0-COMMON-EXPERIMENTAL-SUBSTRATE: ordinary-library opaque/private construction, the already selected generic/reborrow/result contracts, direct retained-state behavior, checked F-ALLOC, and one closed affine owning-interval carrier. No production spelling is selected.",
        "normative_rule": "All five measured arms bind the identical observable contracts and adapters. Opaque construction protects ordinary-library invariants without standard-library privilege. F-ALLOC transfers one checked allocation-owner token and exposes no raw/manual authority. The owning cursor holds one master allocation and exactly [front,back); one endpoint dies before yield and affine abandonment drops exactly the remainder then releases once.",
        "grammar": "Conditional experiment grammar exposes only candidate-neutral opaque/private library construction, checked allocation calls, and the closed owning-interval constructor/next/next_back/drop interface. The carrier has no hole, mutation, repair-to-Dense, second-range, arbitrary-liveness, raw-byte, or manual-deallocation production.",
        "type_ownership_borrow": "Opaque types retain their private construction authority. AB-GENERIC, BR-REBORROW, and BR-RESULT keep their selected semantics. F-ALLOC success returns exactly one nonforgeable allocation owner. The common cursor is affine and owns one root/version plus exactly one live interval; payload and stored behavior leaves retain exact external roots.",
        "effect_exit_drop": "Direct behavior calls exhibit exact ordered effects and retained-state transitions. Allocation failure follows the selected exact failure policy. Common cursor abandonment destroys exactly [front,back) and releases once; trap aborts without unwind cleanup. No candidate lifecycle action runs inside the common carrier.",
        "diagnostic": "DENSE-COMMON-SEAL-VIOLATION, DENSE-COMMON-ALLOC-OWNER, DENSE-COMMON-ALLOC-FAILURE, DENSE-COMMON-CURSOR-FORGED-INTERVAL, DENSE-COMMON-CURSOR-STALE-VERSION, DENSE-COMMON-CURSOR-SECOND-RANGE, and DENSE-COMMON-CURSOR-DOUBLE-RELEASE.",
        "checker_proof_state": "Track opaque construction authority, exact generic/reborrow/result summaries, retained behavior owner/root/leaf state, one F-ALLOC owner credit, and the cursor tuple (master owner,root,version,capacity,front,back) with 0<=front<=back<=capacity. No arbitrary live-set proposition is admitted.",
        "trusted_fact_path": "Only exact owner/root/version/range facts from the reviewed common ledgers transfer. Cursor endpoint transitions invalidate the old interval before result liveness. F-ALLOC facts convey capacity/root ownership only, never slot liveness. Stored leaves never root in carrier storage or a call frame.",
        "lowering_codegen": "Opaque construction and selected proof summaries erase. Every arm uses one byte-identical checked allocation adapter and one byte-identical cursor adapter. Region-free retained-state instantiations emit zero provenance fields/branches. The carrier lowers only front/back updates, exact remainder drop, and one release; candidate-private cursor lowering rejects the arm.",
        "artifact_reporting": "Emit common substrate option ID, opaque authority edges, allocation acquire/transfer/failure/release credits, retained-state root/leaf transitions, cursor interval transitions, endpoint yields, abandonment drops, ZST logical counts, adapter hashes, and per-arm equality evidence.",
        "tests_and_hostile_review": "Attack ordinary-library construction bypass, standard-library privilege, forged/raw/manual allocation authority, owner loss/double release, call-frame borrow leaves, stale roots, cursor endpoint order, forged/stale/second-range cursor state, ZST underdrop, candidate-private cursor substitution, and every protected no-tax dimension.",
        "derivation_and_necessity": "This conditional row closes prerequisites orthogonal to operation-local partial-state candidates. Under OD-0-SEPARATE-PREREQUISITE-LOCKS it is absent and H-FLATSET/family closure plus three arms' owning traversal remain blocked. It selects no production design.",
        "protected_baseline_effect": "Exactly zero for programs not using the conditional substrate: identical source verdict, parser/type artifacts, layout, raw/optimized IR, facts, calls, traps, fields, branches, code bytes, and structural counters in every arm.",
        "authorization_status": "CONDITIONAL_RESEARCH_ONLY_NOT_IMPLEMENTATION_AUTHORITY",
    },
    {
        "delta_id": "META-DENSE-OD4-SCOPED-CONSUME",
        "candidate_ids": "ALL",
        "public_spelling": "Conditional experiment-only scoped consume/fold under OD-4-EAGER-AND-SCOPED-CONSUME. The exact surface is a nonescaping lexical call over a checked range and direct monomorphized consumer; no first-class repair cursor is returned and no production syntax is selected.",
        "normative_rule": "The scoped operation moves one selected owner at a time into the direct consumer in source order, retains exact consumer/control state, permits declared early normal stop, repairs the dense owner completely before every normal return, and allocates no removed-result storage unless the caller explicitly chooses a collecting consumer.",
        "grammar": "Conditional closed scoped-consume/fold form with atom receiver/range/consumer and an explicit early-stop result. The scope, control authority, receiver borrow, and consumer state cannot be returned, stored, captured, reentered, or converted into a cursor. Exact production remains an owner decision.",
        "type_ownership_borrow": "One lexical authority owns BASE, range progress, current moved item, and consumer state. Consumer leaves retain exact external roots. The consumer receives only the current item and its declared state; it never receives master allocation, spare capacity, liveness, or repair authority.",
        "effect_exit_drop": "Calls occur exactly once per visited item in increasing source order. Continue advances after exact item disposition; early stop performs no later call and repairs before return. Normal errors follow the exact result contract. Trap abort preserves the pre-abort partition and performs no unwind cleanup.",
        "diagnostic": "DENSE-SCOPED-CONSUME-ESCAPE, DENSE-SCOPED-CONSUME-CAPTURE, DENSE-SCOPED-CONSUME-REENTRY, DENSE-SCOPED-CONSUME-CALL-ORDER, DENSE-SCOPED-CONSUME-UNREPAIRED-RETURN, and DENSE-SCOPED-CONSUME-RESULT-ALLOCATION.",
        "checker_proof_state": "Track BASE root/version, original range, visited prefix, retained suffix, current item role, consumer state/root/leaf ledger, call ordinal, early-stop edge, and exact repair completion on every normal exit.",
        "trusted_fact_path": "Range/item facts are valid only for the exact current step. Each move invalidates the source slot before consumer entry; no consumer call or branch join retains stale positional facts. Repair emits successor Dense facts only after the final owner transition.",
        "lowering_codegen": "Directly monomorphize the consumer loop and erase lexical authority. Emit O(1) auxiliary container state, no persistent cursor, no indirect dispatch, and no removed-result allocation unless the selected consumer explicitly collects. Charge repair movement/cold edges exactly.",
        "artifact_reporting": "Emit call ordinal, input/result owner roles, consumer state before/after, external borrow roots, early-stop edge, repair transitions, final Dense owner, allocation events, escape/reentry checks, and explicit zero result-allocation row for noncollecting consumers.",
        "tests_and_hostile_review": "Exhaust call order and zero/one/many lengths, early stop at every position, state/capture/return/escape attempts, nested reentry, behavior abort, owner loss/duplication, borrow-root rebasing, unrepaired normal exits, hidden result allocation, ZST logical counts, and region-free zero-tax artifacts.",
        "derivation_and_necessity": "The row prevents eager owning-result removal from becoming the only materially slower workaround for streaming/discard use. It is conditional on the unresolved three-way OD-4 decision and creates no persistent lazy repair cursor.",
        "protected_baseline_effect": "Exactly zero for programs without the conditional scoped form: no parser/type/checker state, retained environment field, cleanup edge, allocation, branch, call, fact, code-size, or final-byte delta.",
        "authorization_status": "CONDITIONAL_RESEARCH_ONLY_NOT_IMPLEMENTATION_AUTHORITY",
    },
    {
        "delta_id": "META-C-ATOMIC",
        "candidate_ids": "C-ATOMIC-TRANSITIONS",
        "public_spelling": "`transition PLACE as IDENT { stmt* }`; inside that scope the only state-changing operations are the closed table-op set `transition_init`, `transition_move_out`, `transition_relocate`, `transition_destroy`, `transition_replace`, `transition_swap`, `transition_allocate`, `transition_release`, and `transition_commit`.",
        "normative_rule": "PLACE is moved into one scope-bound transition authority. The authority cannot escape, be returned, stored, captured, borrowed as payload, or passed to a user function. Every normal path executes transition_commit exactly once in a ValidDense state; an open or doubly committed path is rejected. The scope may contain loops and effectful behavior calls that cannot access the authority.",
        "grammar": "Add `transition_stmt` to `stmt`; `transition_stmt := \"transition\" place \"as\" IDENT \"{\" stmt* \"}\"`. The binder is fresh under TYPE-6. The closed operations are table calls and retain GRAM-9 atom arguments.",
        "type_ownership_borrow": "The binder has sealed lexical type `Transition<T>` and unique authority over one allocation. Each operation checks its exact live/dead precondition and updates one abstract state. No partial-state value is first class. Existing payload borrows must end before entry; transition-created borrows must end before the next conflicting event or commit.",
        "effect_exit_drop": "The scope exhibits the union of its body and transition events. Fallthrough, return, break, give, and try edges are legal only after commit; behavior stop/error follows the same rule. Trap aborts without commit or cleanup after preserving the pre-abort invariant. There is no automatic normal-exit action.",
        "diagnostic": "DENSE-ATOMIC-OPEN-EXIT, DENSE-ATOMIC-ESCAPE, DENSE-ATOMIC-CAPTURE, DENSE-ATOMIC-STATE-MISMATCH, DENSE-ATOMIC-DOUBLE-COMMIT, and DENSE-ATOMIC-BORROW-CONFLICT.",
        "checker_proof_state": "A lexical transition map records allocation root, owner, version, live ranges, dead ranges, and each ownership event. Loop headers require an invariant over this map; joins require identical owner/root and compatible range propositions. Commit proves ValidDense.",
        "trusted_fact_path": "Checked events produce versioned live-range and capacity facts. Entry invalidates all prior positional facts; each event invalidates the affected range before mutation. No open transition fact crosses a call, return, or branch join without proof.",
        "lowering_codegen": "Erase the lexical authority. Lower events to direct checked payload operations and commit to no instruction. Reject the arm if a container-specific opcode, runtime transition tag, implicit cleanup block, or persistent field appears.",
        "artifact_reporting": "Emit the exact state before/after every event, loop invariant, join proof, fact dependency, invalidation, and commit site. Report zero implicit cleanup edges.",
        "tests_and_hostile_review": "Bind every trace to the lexical enforcement point; include open exits through every control construct, capture/store/return attempts, callback attempts, incompatible joins, double commit, and postcommit use. Hostile review checks loop and callback soundness.",
        "derivation_and_necessity": "A callback-free sequence of one-step operations cannot derive stable O(n), O(1)-scratch compaction after the first hole. The loop-capable lexical authority is the minimum distinguishing delta for this arm.",
        "protected_baseline_effect": "B-FIX and B-P2 contain no transition statement and must produce no new parser node in their trees, proof state, fact, branch, field, code, or diagnostic delta.",
        "authorization_status": "DESCRIPTION_FROZEN_CONSTRUCTION_NOT_AUTHORIZED",
    },
    {
        "delta_id": "META-C-LINEAR",
        "candidate_ids": "C-LINEAR-REBUILD",
        "public_spelling": "Add the mode `exact`. A rebuild begins with `let NAME: exact rebuild<T> = rebuild_begin<T>(move OWNER);`; every rebuild operation consumes one exact value and returns one new exact rebuild value; `rebuild_commit<T>(move NAME)` or a frozen explicit failure constructor consumes the final value.",
        "normative_rule": "An exact binding is consumed exactly once on every normal control-flow path. Affine abandonment, implicit drop, duplicate consumption, and conversion to own are rejected. Exactness is transitive through structs, enums, function parameters/results, calls, closures/environment structs, match, loop, try, give, and returns; wrapping cannot weaken it.",
        "grammar": "Extend `mode := ... | \"exact\"`. Add built-in `rebuild<T>` to the gated candidate prelude; all operations remain ordinary GRAM-9 calls with named arguments for user helpers and positional arguments for table ops.",
        "type_ownership_borrow": "One exact rebuild value owns the master allocation, source/destination live ranges, offered values, and state proof. It may move through an exact-typed helper but cannot be borrowed into longer-lived storage. Commit is legal only for ValidDense; failure constructors return the exact owner sets frozen by the outcome.",
        "effect_exit_drop": "Exact values have no drop action. Every fallthrough, return, break, give, try, behavior stop/error, or helper return proves one consumption. Trap aborts without consumption after the pre-abort invariant. A loop invariant states which exact value reaches the next iteration or exit.",
        "diagnostic": "DENSE-LINEAR-ABANDON, DENSE-LINEAR-DUPLICATE, DENSE-LINEAR-WRAP-WEAKEN, DENSE-LINEAR-JOIN, DENSE-LINEAR-HELPER-SIGNATURE, and DENSE-LINEAR-INVALID-COMMIT.",
        "checker_proof_state": "The ownership flow graph gains an exact-use obligation per binding and a state proposition per rebuild value. Each normal CFG edge carries exactly one successor consumption; path joins require one compatible exact value and state.",
        "trusted_fact_path": "State facts transfer only with the exact value. Moving it changes the binding identity but preserves allocation root and version; calls require exact parameter/result summaries; no fact survives abandonment because abandonment is rejected.",
        "lowering_codegen": "Erase exactness and state proofs. Lower rebuild operations to direct checked moves/relocations. Commit and exact transfers emit no runtime tag or cleanup. Reject any implicit drop flag or persistent metadata.",
        "artifact_reporting": "Emit exact-use dependency cones, each transfer, every CFG-edge obligation, state proposition, fact transfer, and final consumption. Missing or multiply credited edges are hard findings.",
        "tests_and_hostile_review": "Attack transitivity through every aggregate and control construct, helper and behavior boundary, nested exact values, recursion rejection, loop joins, and normal exits. Review the type/flow seam independently.",
        "derivation_and_necessity": "Current own is affine and may be abandoned. Exact-use is the arm's irreducible delta; if the proposal instead confines authority lexically it collapses to C-ATOMIC.",
        "protected_baseline_effect": "No protected fixture contains exact mode. Their parser trees, ownership facts, diagnostics, IR, and code remain exact matches; merely adding an enum variant or runtime mode field fails.",
        "authorization_status": "DESCRIPTION_FROZEN_CONSTRUCTION_NOT_AUTHORIZED",
    },
    {
        "delta_id": "META-C-REPAIR",
        "candidate_ids": "C-DERIVED-REPAIR",
        "public_spelling": "`repair PLACE as IDENT { stmt* }`; the closed table-op set is the atomic set plus `repair_commit`. The binder is lexical and cannot escape, but an open normal exit is accepted only when the compiler has derived the one registered repair for its exact partial state.",
        "normative_rule": "Every partial state has exactly one total, nonallocating, nontrapping, behavior-free repair/destruction function. It may move or destroy already-owned values and release the owned allocation but may not call user code. Explicit repair_commit in ValidDense suppresses cleanup exactly once. Moving the lexical scope is impossible; nested scopes repair innermost first.",
        "grammar": "Add `repair_stmt` to `stmt`; `repair_stmt := \"repair\" place \"as\" IDENT \"{\" stmt* \"}\"`. The binder is fresh. Closed operations remain table calls under GRAM-9.",
        "type_ownership_borrow": "The binder has sealed `RepairState<T>` with unique allocation authority and exact live ranges. It cannot escape, be captured, or be placed in data. Each state selects one registered repair proof. A state with no total repair is rejected before lowering.",
        "effect_exit_drop": "Normal open exits execute the derived repair, then continue with the repaired owner or the outcome-specific owner disposition. Trap aborts with no cleanup. Cleanup effects are included in the enclosing function's exact effect row even when cold. Repair never catches a trap or recovers OOM.",
        "diagnostic": "DENSE-REPAIR-NO-TOTAL-ACTION, DENSE-REPAIR-ESCAPE, DENSE-REPAIR-EFFECTFUL-ACTION, DENSE-REPAIR-DOUBLE-ACTION, DENSE-REPAIR-STATE-MISMATCH, and DENSE-REPAIR-UNSURFACED-EDGE.",
        "checker_proof_state": "Each CFG edge carries the partial-state ID and the registered total repair proof. Explicit commit kills the repair obligation. Joins require the same repair ID or distinct predecessor cleanup blocks before the join.",
        "trusted_fact_path": "All open-state facts invalidate before a cleanup edge. Repair emits a new versioned ValidDense fact only after its final ownership event. No fact licenses speculation across the cleanup boundary.",
        "lowering_codegen": "Emit one explicit cold cleanup block per distinct repair state and share it only when state and owner disposition are identical. Cleanup contains no allocator acquisition, user call, indirect call, or trap. Artifact-visible blocks are part of code-size and latency accounting.",
        "artifact_reporting": "Emit every implicit edge, repair ID, proof of totality, instruction/call/effect set, owner disposition, fact invalidation, and suppression site. Hidden cleanup is a hard failure.",
        "tests_and_hostile_review": "Abandon after every partial transition through every normal exit; attack double cleanup, move/capture/escape, effectful repair, nested ordering, join conflation, and fact use before/after cleanup. Review generated CFG and code size.",
        "derivation_and_necessity": "This arm admits affine abandonment by compiler action. If open exits are rejected it collapses to C-ATOMIC; if a first-class exact token is required it collapses to C-LINEAR.",
        "protected_baseline_effect": "Protected fixtures contain no repair statement and must emit zero cleanup edges, repair tables, runtime flags, effects, branches, fields, or code bytes.",
        "authorization_status": "DESCRIPTION_FROZEN_CONSTRUCTION_NOT_AUTHORIZED",
    },
    {
        "delta_id": "META-C-PROOF",
        "candidate_ids": "C-PROOF-CARRYING-STATE",
        "public_spelling": "Add the gated type `partition<T>`. `partition_begin<T>(move OWNER)` returns one partition; state-changing table ops consume one partition plus explicit index/range atoms and return one partition and any moved-out owner through a named result type; `partition_commit<T>(move PARTITION)` returns a dense owner only when the proof state is one prefix.",
        "normative_rule": "One partition owns the noncopyable master allocation authority and all live payload ranges. Range proofs may be passed through ordinary own partition values, but a base or suffix owner never escapes separately. At most two disjoint live ranges are admitted in this candidate. Dropping a partition destroys exactly those ranges, then releases the master once; commit is the sole conversion back to a dense owner.",
        "grammar": "Extend `type` with `partition \"<\" type \">\"`; no new statement or expression production. All operations are closed table calls under existing GRAM-5/9. State indices are explicit atom arguments, not new type-level syntax.",
        "type_ownership_borrow": "partition<T> is affine, noncopyable, and structurally droppable. The checker binds each value to one master root/version and zero to two disjoint live ranges. The base cannot escape, reallocate, release, or be destroyed independently. Borrow results are range-rooted and block overlapping transitions.",
        "effect_exit_drop": "Ordinary abandonment invokes the built-in structural drop for the proved live ranges; it does not reconstruct a prior dense invariant. Structural drop is nonallocating and has only payload-destruction and release effects. Trap aborts without drop. Calls transfer the complete partition proof summary.",
        "diagnostic": "DENSE-PROOF-RANGE-OVERLAP, DENSE-PROOF-WRONG-MASTER, DENSE-PROOF-BASE-ESCAPE, DENSE-PROOF-UNPROVED-DROP, DENSE-PROOF-THIRD-RANGE, and DENSE-PROOF-INVALID-COMMIT.",
        "checker_proof_state": "A partition proposition contains master root, version, capacity, and an ordered set of at most two half-open live ranges whose union owns every payload. Each operation proves arithmetic, disjointness, exact owner transfer, and the successor proposition. Calls require exact proposition summaries.",
        "trusted_fact_path": "Live-range facts are owner/root/version-bound and transfer only with the partition. Facts authorize access within proved ranges. All affected range facts invalidate before a move/drop; joins retain only equal propositions or require an explicit checked normalization.",
        "lowering_codegen": "Erase proof identities. Carry explicit index/range values already present in the program; add no state tag or per-slot metadata. Structural drop lowers to at most two range loops plus one release. Any hidden third range, runtime tag, or implicit repair-to-dense block rejects the arm.",
        "artifact_reporting": "Emit master authority, range propositions, operation proofs, call summaries, drop ranges, release credit, fact transfers, and proof erasure. Report the exact runtime values retained solely for range destruction.",
        "tests_and_hostile_review": "Attack master escape, early base release/reallocation, double release, overlapping or forged ranges, stale versions, third-range pressure, call-summary weakening, ZST same-address values, rejoin, and abandonment after every split/hole state.",
        "derivation_and_necessity": "Always-valid state with statically proved live ranges avoids lexical rejection and repair-to-dense cleanup. If a runtime topology tag chooses ranges it collapses to C-RUNTIME; if normal exit reconstructs Dense it collapses to C-DERIVED.",
        "protected_baseline_effect": "Protected fixtures use buffer, not partition. They must gain no partition drop glue, proof parameter, runtime range field, fact dependency, code, or diagnostic delta.",
        "authorization_status": "DESCRIPTION_FROZEN_CONSTRUCTION_NOT_AUTHORIZED",
    },
    {
        "delta_id": "META-C-RUNTIME",
        "candidate_ids": "C-RUNTIME-TOPOLOGY",
        "public_spelling": "Add the gated type `topology<T>`. `topology_begin<T>(move OWNER)` returns one topology owner; closed table ops consume and return it; `topology_commit<T>(move TOPOLOGY)` returns a dense owner only in Dense state.",
        "normative_rule": "One topology owner holds the master allocation and an exact runtime descriptor for either Dense `[0,len)` or Hole `[0,hole_start) + [hole_end,len)`. No per-slot tag or bitmap is permitted. Every descriptor state is structurally droppable. The descriptor is sealed; ordinary code cannot forge or write its fields.",
        "grammar": "Extend `type` with `topology \"<\" type \">\"`; no new statement or expression production. Closed operations use existing table-call grammar and GRAM-9 atoms.",
        "type_ownership_borrow": "topology<T> is affine and structurally droppable. Metadata and payload share one sealed authority and version. Operations check descriptor bounds and exact ownership transitions. Borrow results are master-rooted and versioned; conflicting transitions reject.",
        "effect_exit_drop": "Ordinary abandonment destroys the one or two runtime live ranges selected by validated metadata and releases once. It never repairs to Dense. Trap aborts without drop. Metadata validation precedes any payload load or drop and is not elided without the exact fact.",
        "diagnostic": "DENSE-RUNTIME-FORGED-TOPOLOGY, DENSE-RUNTIME-STALE-VERSION, DENSE-RUNTIME-PAYLOAD-METADATA-ORDER, DENSE-RUNTIME-THIRD-RANGE, DENSE-RUNTIME-INVALID-COMMIT, and DENSE-RUNTIME-UNSEALED-FIELD.",
        "checker_proof_state": "The checker knows only the sealed owner/root/version and the descriptor invariant; exact Dense versus Hole may be dynamic. Checked transitions produce successor facts. User code receives no field authority.",
        "trusted_fact_path": "A validated descriptor fact authorizes only the named live ranges for the same owner/root/version. Metadata must become a readable successor state before any normal interruption of payload motion. Wrong/stale/forged metadata and speculative loads are mandatory attacks.",
        "lowering_codegen": "Use one transient tag plus `len`, `hole_start`, and `hole_end` only while topology<T> is live; commit returns the ordinary ptr/len/cap dense representation. Per-slot metadata, a persistent dense-mode field, hidden repair block, or more than two live ranges rejects the arm.",
        "artifact_reporting": "Emit descriptor layout, every metadata and payload update in order, validation checks, facts, invalidations, drop branches/ranges, release credit, and proof-elision sites.",
        "tests_and_hostile_review": "Interrupt after every metadata/payload substep; attack forged/stale descriptors, wrong owner/root/version, payload-before-metadata and metadata-before-safe-payload order, speculative dead-slot loads, ZST aliasing, third-range pressure, and every abandonment edge.",
        "derivation_and_necessity": "The runtime descriptor makes every partial state valid and abandonable without exact-use or repair-to-dense. If topology is compile-time-only it collapses to C-PROOF; if open state is lexical it collapses to C-ATOMIC.",
        "protected_baseline_effect": "topology<T> is transient and explicit. Existing buffers and SoA pools must gain no tag, range field, validation, branch, fact, drop glue, code, or source change.",
        "authorization_status": "DESCRIPTION_FROZEN_CONSTRUCTION_NOT_AUTHORIZED",
    },
)


def validate_meta5_rows() -> None:
    """Fail closed on missing columns, duplicate IDs, or authorization drift."""
    ids: set[str] = set()
    candidates: set[str] = set()
    for row in META5_ROWS:
        missing = [field for field in META5_FIELDS if not row.get(field)]
        if missing:
            raise ValueError(f"META-5 row {row.get('delta_id')} misses {missing}")
        if row["delta_id"] in ids:
            raise ValueError(f"duplicate META-5 delta: {row['delta_id']}")
        ids.add(row["delta_id"])
        if row["candidate_ids"] != "ALL":
            candidates.add(row["candidate_ids"])
        if "NOT_AUTHORIZED" not in row["authorization_status"] and "NOT_IMPLEMENTATION_AUTHORITY" not in row["authorization_status"]:
            raise ValueError(f"META-5 row exceeds authorization: {row['delta_id']}")
    expected = {
        "C-ATOMIC-TRANSITIONS",
        "C-LINEAR-REBUILD",
        "C-DERIVED-REPAIR",
        "C-PROOF-CARRYING-STATE",
        "C-RUNTIME-TOPOLOGY",
    }
    if candidates != expected:
        raise ValueError(f"META-5 candidate set mismatch: {sorted(candidates)}")
    required_common = {
        "META-DENSE-COMMON",
        "META-DENSE-OD0-COMMON-SUBSTRATE",
        "META-DENSE-OD4-SCOPED-CONSUME",
    }
    if not required_common <= ids:
        raise ValueError(f"META-5 common delta set mismatch: {sorted(required_common-ids)}")
    by_id = {row["delta_id"]: row for row in META5_ROWS}
    for delta_id in required_common:
        if by_id[delta_id]["candidate_ids"] != "ALL":
            raise ValueError(f"META-5 common delta is not cumulative: {delta_id}")
    od0_text = " ".join(str(value) for value in by_id["META-DENSE-OD0-COMMON-SUBSTRATE"].values()).lower()
    for required_text in (
        "ordinary-library", "f-alloc", "[front,back)", "byte-identical",
        "no production", "standard-library privilege",
    ):
        if required_text not in od0_text:
            raise ValueError(f"OD-0 META-5 omits {required_text}")
    od4_text = " ".join(str(value) for value in by_id["META-DENSE-OD4-SCOPED-CONSUME"].values()).lower()
    for required_text in (
        "call ordinal", "early stop", "cannot be returned", "reentry",
        "no removed-result allocation", "conditional",
    ):
        if required_text not in od4_text:
            raise ValueError(f"OD-4 META-5 omits {required_text}")


def cumulative_meta5_ids(candidate_delta_id: str) -> tuple[str, ...]:
    """Return the exact cumulative research deltas for one measured arm."""
    common = (
        "META-DENSE-COMMON",
        "META-DENSE-OD0-COMMON-SUBSTRATE",
        "META-DENSE-OD4-SCOPED-CONSUME",
    )
    ids = {row["delta_id"] for row in META5_ROWS}
    if candidate_delta_id not in ids:
        raise KeyError(candidate_delta_id)
    return common + (candidate_delta_id,)
