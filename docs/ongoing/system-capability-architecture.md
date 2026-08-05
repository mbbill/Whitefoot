# System-capability architecture

This is a temporary coordination record, not execution authority. Delete it
when the task is integrated, parked, replaced, or abandoned.

- **Status:** `WAITING` — owner architecture selection
- **Authority:** active [`docs/current-plan.md`](../current-plan.md), `BOUND-1`
  Goal, Work 1–5, Verification, and Done-when conditions; derived from
  `CAND-8`, `BOUND-1`, `PAR-4`, and `VERIFY-1` in Outline revision 6
- **Owner / workspace:** primary agent / `main` integration workspace
- **Base revision:** `03fc7b2`

## Goal

Give the owner enough evidence to select one complete Whitefoot
system-capability architecture and one exact first command-program slice. That
slice must unblock `wfgrep` without choosing argv/file/stdout APIs that later
networking, cancellation, waiting, or worker support would have to replace.

## Direction and invariants

- Exact static entry imports provide authority; typed affine resources define
  ownership, aliasing, cleanup, transfer, cancellation, and concurrency.
- Authority, observable effects, and trusted provider identity stay separate.
  Native hot I/O retains a direct static path without mandatory dispatch,
  centralized serialization, or avoidable copies.
- Whole-process abort remains the containment boundary; general foreign-code
  FFI remains `BOUND-2`; this task changes no specification or compiler code.

## Method

Use the architecture dossier to audit WASI and native constraints, compare
complete alternatives, instantiate representative resource protocols, trace
command, parallel-search, and network-service witnesses, inventory v0.17 and
compiler deltas, and run hostile semantic and performance-shape reviews.

## Progress

- **Done:** audited WASI and native costs; compared four architecture
  families; drafted the candidate, capability map, representative protocols,
  three hostile witnesses, and exact first command slice; completed hostile
  reviews with no remaining design blocker; committed evidence in `03fc7b2`.

- **Current:** owner review and architecture selection.

- **Next:** after selection, record the durable decision, update the Direction
  Outline, and replace the Current Plan with a `PROPOSED` implementation slice.
  Specification and compiler work wait for that plan's approval.

## Scope and expected touch set

This scope is advisory and non-exclusive. It identifies semantic overlap and
likely rebase pressure; it does not reserve files.

- Primary evidence:
  `research/investigations/system-capability-architecture/DOSSIER.md`
- Expected decision-closure paths: `docs/current-plan.md`, `docs/roadmap.md`,
  and relevant nodes under `mcts_mem/whitefoot/`.
- Excluded write scope: compiler source, numbered specification files,
  conformance expectations, and `wfgrep` source.

## Dependencies and integration order

- **Prerequisites:** none currently.
- A task changing `BOUND-1` language, provider, ABI, effect, resource, or entry
  semantics depends on owner selection and lands after this decision closure;
  it then refreshes its base, rereads the decision and plan, rebases, and reruns
  its gates.
- Unrelated work may land in either order. Textual overlap is not a lock;
  system-capability semantic overlap requires cross-links in both records.

## Validation, stop, and closure

- **Validate:** owner selects or rejects the candidate; open questions have
  explicit dispositions; hostile reviews, document checks, and MCTS lint pass.
- **Stop:** do not expand if the owner requests another alternative or review
  exposes a semantic question outside the active plan.
- **Close:** record the decision in its canonical homes, update the Outline,
  replace the Current Plan, and delete this file in the same integration
  change. A parked or superseded task records that disposition and is deleted.
