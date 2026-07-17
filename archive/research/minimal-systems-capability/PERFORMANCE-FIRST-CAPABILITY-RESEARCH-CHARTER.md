# Performance-first capability research charter

Status: active research direction, 2026-07-15. This charter supersedes the
mechanism-first sequencing of the static-privilege owner packet. It does not
rewrite that packet or its hash-pinned evidence, select a production design,
authorize an experiment, or authorize a language, specification, checker,
compiler, runtime, standard-library, fact-channel, migration, or teaching
change.

## Objective

Find the smallest coherent xlang capability sets that let ordinary checked
programs obtain required native representations and machine-code shapes without
unavoidable initialization, zeroing, copying, relocation, tags, metadata,
allocation, indirection, dynamic dispatch, checks, or extra machine events.

Memory and thread safety, no writer-accessible `unsafe`, proof-only check
elision, a compact regular specification, a feasible checker, stable AI use,
and ordinary-library systems coverage without a shipped standard library remain
binding constraints. They are not substitutes for the performance objective.

Isolation is neither the objective nor the derivation starting point. It is a
conditional implementation constraint only for a surviving capability whose
semantics cannot be expressed as an ordinary safe language or checker rule.

## Required order

1. Freeze a finite ledger of performance-relevant expressiveness gaps.
2. Derive common semantic requirements across independent gaps.
3. Construct at least three materially different candidate capability sets.
4. Compare each set's performance coverage, safety, composition, language and
   checker cost, implementation shape, and extension risk.
5. Recheck ordinary-library derivability, structural costs, and held-outs.
6. Hostile-review correctness and hidden performance taxes.
7. Return the evidence and candidate comparison to the owner before any
   production design or experiment.

No gate, registry, intrinsic form, keyword, or prior P/Q basis is selected
before this sequence.

## Performance-gap ledger contract

Each gap must record:

- a stable gap identity and at least one exact demand identity;
- the required caller-observable contract;
- the current xlang rule that blocks the representation or code shape;
- the unavoidable cost or impossibility under current semantics;
- protected asymptotic time and space;
- layout, contiguity, and allocation topology;
- initialized and zeroed bytes;
- copies, moves, relocations, and destruction;
- mandatory tags, bitmaps, headers, counters, generations, or leases;
- indirections, dynamic dispatch, checks, branches, code size, and machine or
  external event counts;
- ownership, borrow provenance, failure, cleanup, interference, and fact
  invalidation obligations;
- the evidence strength: measured, structurally derived, model-required,
  unmeasured, or open; and
- an exact falsifier that would remove or revise the gap.

An API name, container name, Rust mechanism, or one benchmark does not establish
a gap by itself.

## Candidate-set deliverable

The final packet must contain at least three materially different complete
candidate sets. They must differ in semantic factoring or responsibility, not
only spelling or packaging. No candidate is preselected.

For every candidate set, record:

1. every language, type-state, ownership/checker, proof, ordinary-library,
   builtin, runtime, or target capability in the set;
2. the exact semantics and composition rules of each capability;
3. the independent gaps each capability closes and its removal witness;
4. how the set changes layout, initialization, zeroing, copying, relocation,
   allocation, metadata, indirection, checks, code shape, and machine events;
5. the containers and systems facilities derivable by ordinary checked
   libraries;
6. the expected checker, compiler, backend, and runtime responsibilities;
7. safety, failure, cleanup, fact-production, and invalidation rules;
8. costs imposed on weaker protected shapes;
9. unresolved problems and explicit falsifiers; and
10. why the set is one coherent alternative rather than an arbitrary bundle.

Candidate forms may include direct language constructs, uniform type-state or
ownership rules, checked proof mechanisms, ordinary libraries over fixed
semantics, compiler builtins, and exact runtime or target leaves. Safe ordinary
writer access is a benefit when it closes the gap under the binding constraints;
it is not an authority defect.

## Uniform comparison

The comparison must explain, rather than merely score, every candidate's pros,
cons, and trade-offs across:

- gap and held-out coverage;
- native representation and zero-cost code shape;
- unavoidable tax on weaker shapes;
- semantic concept and normative-rule count;
- composition across storage, ownership, concurrency, external, and target
  domains;
- checker complexity and feasibility;
- compiler and backend special cases;
- runtime metadata and transitions;
- AI generation stability and canonical teaching burden;
- specification size and regularity;
- no-standard-library operation;
- target portability;
- extension discipline and risk of capability growth; and
- safety, proof, fact-channel, and review burden.

The result may select one candidate, select an exact combination, or conclude
that no candidate wins on current evidence. Any recommendation must name the
conditions that eliminate the alternatives, its residual risks, and the new
evidence that would reverse it.

## Evidence carried forward

The Rust 1.97.0 census, 276 coverage clusters, 49 capability obligations,
26-domain systems envelope, dense exact obligations, topology witnesses,
held-outs, production-language mechanism census, prior P/Q proposal, and
structural-cost fields remain evidence inputs. None is a selected language
mechanism or candidate basis.

The sealed compiler-embedded registry remains one possible implementation
technique for residual compiler-defined machine semantics. The prior gate
comparison proves neither that such a residual set exists in its proposed form
nor that isolation should organize the public capability basis.

Exact D-2 and P-1 remain fail-closed. A routed category earns no exact
derivability credit, a paper construction earns no measured performance credit,
and a structural cost argument is not a formal safety proof.

## Hostile-review focus

Review first for hidden initialization or zeroing, universal tags or bitmaps,
hidden `Copy`/`Clone`/`Default`, extra allocations, headers, reference counts,
leases, whole-buffer copies, indirection, dynamic dispatch, additional machine
events, surviving checks, code-size multiplication, state explosion, owner
loss, duplicate destruction, uninitialized reads, wrong borrow provenance,
interference, and stale facts.

Name, path, flag, plugin, artifact, and symbol admission attacks apply only to a
candidate capability that actually requires compiler-private definition.

## Owner decision surface

The final packet must let the owner decide independently:

1. whether the performance gaps and protected costs are correct;
2. which candidate sets remain live;
3. whether to accept a recommended set as a falsifiable research hypothesis;
4. which separately scoped safety model, structural prototype, machine-event
   model, or performance measurement to authorize; and
5. whether any later syntax, specification, checker, compiler, backend, or
   runtime design may begin.

The research requests no production decision in advance.
