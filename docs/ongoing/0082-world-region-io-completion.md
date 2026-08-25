# Batch 0082 — world-region I/O and completion backends

Branch: `codex/io-model-completion`, from `main` at
`eab81a335addfb0ae060735771d4e98891dec2ea`.

Status: IN PROGRESS. Phase A started 2026-08-25.

## Charter

The owner directed this branch to execute
`research/investigations/io-model/HANDOFF.md` from Phase A through the final
phase. The batch is complete only when the specification migration, compiler
and conformance migration, macOS prototype and measurement, production macOS,
Linux, and Windows completion backends, cross-host harness, CI coverage, and
the deterministic `--io-ledger` audit surface, scripted schedule evidence, and
canonical repository verification are complete.

No revision from this branch may enter `main` without the owner's approval of
the exact revision. This record does not authorize such a merge.

## Binding material read before implementation

The implementation pass read these sources in the order fixed by the handoff:

1. `AGENTS.md`;
2. `docs/constitution.md`, including W3 and theorem T3;
3. `research/investigations/io-model/DESIGN.md`, revision 2;
4. all three reports in `research/investigations/io-model/reviews/`;
5. `docs/current-plan.md` W2 and the active v0.36 rules [FN-7], [EFF-1..5],
   [PAR-1], [PAR-2], [TRAP-1], and [SYS-1..14];
6. `docs/done/0081-loan-column.md`.

T3 controls the yield direction: a correct program never loses permission in
order to stabilize a defective execution's observables. The world-window rule
must widen the erroneous-execution promise and must not restore the overruled
trap-free gate.

## Fixed implementation boundaries

- Phase A uses DESIGN section 3e option 1: every operation formerly carrying
  `external` joins one conservative global world-order domain, preserving the
  v0.36 order promise.
- Different capability values never prove world disjointness. Missing origin,
  alias, projection, or target evidence denies overlap.
- `blocks` becomes trusted completion/blocking metadata for every target
  action, including compiler-derived release and transitive user wrappers.
- The language exposes completion semantics only. kqueue readiness, io_uring,
  and IOCP remain backend and trusted-base choices.
- Phase C covers macOS, Linux, and Windows against one shared C contract.
- No active source, test, tool, or build path may depend on `archive/`.

## Phase A evidence

### Parent behavior reproduced

The Phase A baseline is the branch parent, specification v0.36 at
`eab81a335addfb0ae060735771d4e98891dec2ea`.

- Case-insensitive word-boundary counts in `spec/kernel-spec.md` reproduce the
  review exactly: 136 `external` occurrences, 31 `blocks` occurrences, on 117
  physical lines.
- The same search under `tests/conformance/cases/` names exactly 42 `.wf`
  files.
- `cargo test --locked --offline system_effects`: 10 passed, 0 failed.
- `cargo test --locked --offline semantic::tests::permission`: 28 passed,
  0 failed. This pins both adjacent boundaries: an `external` row is denied by
  the current row gate, while a claim-bearing closure remains eligible under
  T3.
- `cargo test --test conformance --locked --offline -- --ignored --nocapture`:
  `Pass=500  Skip=1`, including the same-sink EFF-5 runtime witness and the
  seven release/effect-row verdict records.

### Candidate and ledgers

`research/investigations/io-model/SPEC-CANDIDATE.md` now contains the complete
non-authoritative delta against v0.36 under the recommended selections. It
includes:

- D1 through D5 with consequences, including all three observable
  `WF_WORKERS=1` mapping deltas;
- an explicit disposition for all 163 distinct `K` anchors cited by the
  specification sweep and all sixteen final amendments;
- all fifteen kinded operation signatures, all eight release rows, exact
  declaration counts, origin/alias rules, operation points, outcomes,
  progress, abort, qualification, and target-action rules;
- a one-to-one ledger for the 42 source cases and the exact seven
  verdict-sensitive manifest records, with no proposed verdict change, plus
  an explicit inventory of the 27 files that actually write an `external`
  effect row; and
- the nineteen additional tracked `.wf` workloads outside conformance that
  use world-bearing system types or operations, plus the one deliberate
  `open_read` name-collision control that receives no syntax rewrite;
- compiler work sizing that isolates capability world-vector representation
  and EFF-2 world projection as the two substantial pieces, followed by their
  syntax, catalog, release, permission, entry, provenance, lowering, and
  activation satellites.

The draft remains research material. No active specification, conformance
case, compiler behavior, or runtime mapping changes until the owner selects
D1 through D5 and approves activation of the resulting exact candidate.

### Phase A acceptance audit

Mechanical checks over the completed draft establish:

- sweep coverage is 163 candidate anchors for 163 review anchors, with no
  missing or extra ID;
- the conformance table is 42 candidate IDs for the 42 case files found by the
  parent atom search, with no missing or extra case;
- stripping string literals identifies exactly the separately enumerated 27
  source files that really write an `external` row;
- the broader world-family search identifies exactly the separately
  enumerated nineteen `.wf` workloads outside conformance;
- parsing the candidate operation block yields fifteen operations, fifty-four
  operation region parameters, and thirty-eight value parameters; with
  sixteen nominals, fourteen nominal world parameters, forty-two constructors,
  and sixty-seven fields, the declared preorder total is 246; and
- `make repository-invariants` passes, and both new documents contain no
  trailing whitespace, personal home path, TODO, TBD, or FIXME marker.

The only remaining Phase A acceptance item is the owner decision itself. That
boundary is intentional evidence, not an implementation gap.

## Phase B evidence

Pending: kqueue prototype, directory-walk measurement, readiness-service
overhead, and loan-shape findings.

## Phase C evidence

Pending: shared contract, production backends, C harness, cross-host CI, and
target-qualification boundary verification.

## Closure

Pending. On closure this record moves to `docs/done/`, the handoff file is
deleted, and the exact branch revision passes `make check`. Closure does not
merge the revision into `main`.
