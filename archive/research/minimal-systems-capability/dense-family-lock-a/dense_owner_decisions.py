#!/usr/bin/env python3
"""Owner-review choices that remain intentionally unresolved in Lock A."""

from __future__ import annotations


OWNER_DECISIONS = (
    {
        "decision_id": "OD-0",
        "question": "How are the shared ordinary-library prerequisites supplied to every dense candidate arm?",
        "recommended_option_id": "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
        "options": (
            {
                "option_id": "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
                "policy": "Freeze one candidate-neutral, experiment-only substrate and use its exact artifacts unchanged in all five arms. It provides erasable user-defined sealing, the already selected generic monomorphization and reborrow/result-provenance semantics, direct effectful behavior calls with exact retained-state ownership, one checked allocation-owner facade, and one affine single-live-interval storage owner for owning traversal. It selects no production spelling or implementation.",
                "performance_consequence": "Every arm receives the same representation-sealing, call, provenance, allocation, growth, and owning-interval interfaces. Their layout, emitted paths, compiler costs, owning-cursor costs, and protected B-FIX/B-P2 effects are charged and compared identically; no arm receives standard-library-only privilege or a private allocator route.",
                "contract_consequence": "An ordinary library can define an unforgeable dense or flat-set representation and instantiate it over the admitted payload and behavior envelope. The checked allocation facade transfers exact block ownership and failure results but grants no raw bytes, writer-controlled liveness, unchecked capacity change, manual deallocation, or container-specific recognition. The affine interval owner holds one master allocation and exactly one live range [front,back), kills an endpoint owner before yielding it, permits abandonment by dropping exactly the remaining interval and releasing once, counts ZST values by logical index, and exposes no hole, mutation, repair-to-Dense, second-range, or arbitrary liveness API.",
                "reopening_consequence": "Any semantic, spelling, tool, artifact, or cost change to the common substrate reopens every candidate binding and protected control. A later production design still requires its own explicit owner decision and cannot be inferred from this experimental control.",
            },
            {
                "option_id": "OD-0-SEPARATE-PREREQUISITE-LOCKS",
                "policy": "Close and adopt the required sealing, generic/direct-behavior, retained-state, reborrow/result-provenance, checked-allocation, and affine single-live-interval storage-owner prerequisites in separate locks before constructing a dense candidate.",
                "performance_consequence": "Prerequisite costs are selected independently before the dense comparison, which gives clean sequential attribution but postpones every dense candidate build, runner pilot, and score.",
                "contract_consequence": "Until every prerequisite closes, H-FLATSET cannot establish ordinary-library generativity, three operation-local candidates cannot satisfy owning traversal, and the dense family cannot close. The present dossier remains a conditional internal-state protocol only.",
                "reopening_consequence": "After the prerequisite locks close, their exact adopted public contracts and artifacts must replace the blocked references here, regenerate every candidate binding and performance control, and pass fresh hostile review before construction.",
            },
        ),
    },
    {
        "decision_id": "OD-1",
        "question": "Which dense operations expose recoverable allocation failure?",
        "recommended_option_id": "OD-1-RESERVE-FIRST",
        "options": (
            {
                "option_id": "OD-1-RESERVE-FIRST",
                "policy": "Only try-reserve operations return arithmetic or allocation failure. Ordinary push, insert, append, resize, collect, and clone routes use checked arithmetic and the current divergent OOM boundary. A caller needing recovery first performs try-reserve and then uses a no-grow operation.",
                "performance_consequence": "The default no-grow mutation route has no recoverable-error result branch. Growth-capable convenience operations retain only checked arithmetic plus the current OOM abort edge.",
                "contract_consequence": "Recoverable failure returns the unchanged dense owner from try-reserve. Offered payload owners are not yet consumed by the subsequent mutation. A divergent mutator edge has no normal result and must preserve the exact pre-abort invariant until abort.",
                "reopening_consequence": "Selecting this option activates the RESERVE_FIRST policy variant and excludes every RECOVERABLE_MUTATOR outcome and timing cell.",
            },
            {
                "option_id": "OD-1-RECOVERABLE-MUTATORS",
                "policy": "Every growth-capable mutator returns an explicit recoverable arithmetic or allocation failure carrying the unchanged dense owner and every offered affine owner.",
                "performance_consequence": "Every default growth-capable mutation carries a result branch and a larger ABI. The branch, code size, and owner-return path are mandatory scored endpoints.",
                "contract_consequence": "Each mutator gains precommit arithmetic-failure and allocation-failure outcomes. No destructive commitment may precede the last recoverable point.",
                "reopening_consequence": "Selecting this option activates the RECOVERABLE_MUTATOR policy variant and regenerates member outcomes, soundness traces, reference adapters, and performance cells before Lock approval.",
            },
        ),
    },
    {
        "decision_id": "OD-2",
        "question": "What native target scope is required for a production-selection claim?",
        "recommended_option_id": "OD-2-DUAL-NATIVE",
        "options": (
            {
                "option_id": "OD-2-DUAL-NATIVE",
                "policy": "Require the frozen AArch64 macOS runner and one independently frozen x86-64 Linux runner. Both targets pass every mandatory structural and timed intersection gate independently.",
                "performance_consequence": "Selection is architecture-general only across these two named targets; neither target can compensate for the other.",
                "contract_consequence": "The x86-64 machine, kernel, libc, allocator, toolchain, DataLayout, and commands remain construction-blocking until exact identities exist.",
                "reopening_consequence": "Changing either native target identity or dropping a target reopens the protocol and power calculation.",
            },
            {
                "option_id": "OD-2-MAC-TARGET-LOCAL",
                "policy": "Use only the frozen AArch64 macOS runner and label the result target-local.",
                "performance_consequence": "No architecture-general mechanism or production-performance claim is permitted from the result.",
                "contract_consequence": "Cross-target layout and 32-bit arithmetic soundness checks remain required, but x86-64 timing does not gate this target-local experiment.",
                "reopening_consequence": "Any later architecture-general adoption requires a new target lock and hostile review before timing.",
            },
        ),
    },
    {
        "decision_id": "OD-3",
        "question": "Does the first dense payload envelope include zero-sized affine values?",
        "recommended_option_id": "OD-3-INCLUDE-ZST",
        "options": (
            {
                "option_id": "OD-3-INCLUDE-ZST",
                "policy": "Include zero-sized affine values with logical capacity usize::MAX, no payload allocation or growth, index-based owner identity, and exactly len logical destructions.",
                "performance_consequence": "ZST cells are soundness and structural gates plus targeted operation-latency witnesses. Zero allocator calls and bytes are structural equality, never a memory benefit; allocator-movement and allocator-failure cells are inapplicable.",
                "contract_consequence": "Disjointness never relies on addresses. Length overflow remains checked even though payload bytes are zero.",
                "reopening_consequence": "Any finite-capacity or allocating ZST representation reopens soundness and same-shape equivalence.",
            },
            {
                "option_id": "OD-3-DEFER-ZST",
                "policy": "Exclude zero-sized affine values from the first payload envelope.",
                "performance_consequence": "No ZST performance or allocator cell exists.",
                "contract_consequence": "The family claim is limited to positive-size payloads and generic libraries must reject or route ZST instantiations to a later family decision.",
                "reopening_consequence": "Adding ZST later requires a new payload, ownership, disjointness, destruction, and capacity review.",
            },
        ),
    },
    {
        "decision_id": "OD-4",
        "question": "Which removal-consumption contracts are mandatory in the dense family?",
        "recommended_option_id": "OD-4-EAGER-AND-SCOPED-CONSUME",
        "options": (
            {
                "option_id": "OD-4-EAGER-ONLY",
                "policy": "Require only eager range removal, eager predicate extraction, and eager splice that return an owning removed-result sequence. Keep streaming removal and Rust-style lazy repair-bearing cursors as excluded evidence.",
                "performance_consequence": "The dense family measures eager owning-result operations only. Discarding or consuming removed values online may require an otherwise unnecessary O(k) result allocation and O(k) auxiliary storage and traffic, so this option cannot support a full removal-consumption capability claim.",
                "contract_consequence": "No public repair-bearing cursor or scoped consumer is required. Owning whole-sequence iteration remains independently mandatory. The family claim is explicitly limited to callers that need an owning collection of removed values.",
                "reopening_consequence": "Adding streaming consumption or a lazy operation later reopens member outcomes, lifecycle, stored-state, abandonment, cleanup, payload-scope, fact, and timing ledgers.",
            },
            {
                "option_id": "OD-4-EAGER-AND-SCOPED-CONSUME",
                "policy": "Require eager owning-result removal plus a nonescaping scoped consume or fold contract. The scoped form invokes a direct monomorphized consumer in deterministic source order, moves each removed owner exactly once, repairs the dense owner before every normal return, and allocates no removed-result sequence unless the caller explicitly collects. It creates no persistent repair-bearing cursor.",
                "performance_consequence": "Discarding or consuming removed values online has O(1) auxiliary container state and no mandatory removed-result allocation. Direct behavior-call overhead, retained callable state, exact repair work, and early-stop paths are primary gates. This option makes no cross-call lazy-cursor parity claim.",
                "contract_consequence": "The consumer and its captures are lexically retained by one effectful operation and cannot escape. Each call receives one removed owner and must return the declared retained state and control result. Early normal stop first restores a complete valid dense owner; traps abort without unwinding. Collect is an ordinary caller choice layered over the scoped form, not hidden mandatory storage.",
                "reopening_consequence": "Permitting consumer escape, persistent suspension, callback reentry into the same owner, a second repair state, or a different call/repair order reopens every scoped-consume contract, candidate binding, fact channel, and performance cell.",
            },
            {
                "option_id": "OD-4-PROMOTE-LAZY",
                "policy": "Promote Rust-style lazy drain, extract, and splice cursor contracts into the mandatory dense surface in addition to eager owning-result removal.",
                "performance_consequence": "Cursor construction, partial consumption, close, abandonment, tail repair, retained callable/range state, and code-size paths become primary gates.",
                "contract_consequence": "Every candidate must derive the exact repair and allocation-lifecycle semantics for all normal exits.",
                "reopening_consequence": "This option requires a new exact member/outcome and stored-borrow scope expansion before Lock approval.",
            },
        ),
    },
    {
        "decision_id": "OD-5",
        "question": "May the first experiment select a compile-time crossover among mechanisms?",
        "recommended_option_id": "OD-5-NO-CROSSOVER",
        "options": (
            {
                "option_id": "OD-5-NO-CROSSOVER",
                "policy": "Each of the five candidate configurations uses one mechanism over the complete frozen matrix. The selection function returns one candidate or NO-SELECTION.",
                "performance_consequence": "Candidate count remains k=5, the six-treatment Williams design is fixed, and no post-result size or payload threshold can combine arms.",
                "contract_consequence": "One writer route has one lifecycle mechanism in the first decision. A later static specialization proposal requires a separate Lock.",
                "reopening_consequence": "Any crossover or hybrid is a new candidate and reopens k, ordering, power, multiplicity, META-5, and hostile review before candidate-dependent observation.",
            },
            {
                "option_id": "OD-5-ENUMERATED-CROSSOVER",
                "policy": "Add one fully enumerated compile-time crossover candidate whose total target-by-payload-layout-by-static-size function is frozen before construction and contains no runtime dispatch.",
                "performance_consequence": "The crossover is a sixth candidate; the treatment count, Williams design, sample size, directed alpha, dominance graph, and every cell assignment must be regenerated.",
                "contract_consequence": "Every static region independently satisfies soundness and protected-baseline gates through one writer-facing route.",
                "reopening_consequence": "Selecting this option cannot use the current k=5 protocol; a revised Lock requires hostile review before construction.",
            },
        ),
    },
)


def flattened_decision_rows() -> list[dict[str, str]]:
    """Return one deterministic row per owner option without selecting it."""
    result: list[dict[str, str]] = []
    for decision in OWNER_DECISIONS:
        for option in decision["options"]:
            result.append(
                {
                    "decision_id": decision["decision_id"],
                    "question": decision["question"],
                    "option_id": option["option_id"],
                    "recommended": "YES" if option["option_id"] == decision["recommended_option_id"] else "NO",
                    "selected": "UNRESOLVED_OWNER_DECISION",
                    "policy": option["policy"],
                    "performance_consequence": option["performance_consequence"],
                    "contract_consequence": option["contract_consequence"],
                    "reopening_consequence": option["reopening_consequence"],
                }
            )
    return result
