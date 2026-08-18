# Current Plan — wfgrep functional legs and trap-endpoint closure

Status: ACTIVE (owner approval in conversation, 2026-08-18: "批了",
approving this plan exactly as proposed at 6d9eb4bc.)

Derived from Direction Outline revision 41 and the batch-0070 outcome.
Supersedes the completed gap-closure/take-replace plan in place.
Active language authority: v0.32 at `spec/kernel-spec.md`, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`
(activated 2026-08-18 under this plan's batch-0071 approval).

## Objective

Two thrusts, both advancing outline:CAND-8 (ripgrep-class search) and the
trap thesis. First, make wfgrep functionally real on the v0.31 collection
layer: recursive directory traversal, byte-string handling, and the
affine-element buffer lowering the searcher's result storage needs.
Second, close the remaining designed-but-open language endpoints so that
the accepted language matches its own doctrine: claim as the sole
writer-reachable trap source, zero-divisor obligations, and
declaration-site rejection of ambiguous-provenance borrow returns.

## Workstreams

- **W1 — wfgrep functional legs (main).** The recursive-traversal slice
  and the byte-string program over the landed growable-vector layer, plus
  affine-element buffer lowering: `buffer_vacant` construction, element
  replace/vacate lowering, and the per-element drop loop that closes the
  recorded explicit-unsupported capability. Evidence: wfgrep compiles and
  runs a real recursive search over a directory tree end to end.
- **W2 — trap-endpoint spec batch (parallel; one v0.32 candidate).**
  Three deltas through the lead into one candidate under candidate mode:
  division/remainder zero-divisor obligations by the constant-operand
  recipe (#48; the b != 0 goal is already expressible in the fragment);
  check dissolution (#47) — retire OP-5 `check` so claim is literally the
  sole writer-reachable trap source, S2 establishment migrating to the
  richer S1/S3 sources; and declaration-site rejection of
  ambiguous-provenance borrow returns (#50; owner-driven law: at most one
  parameter may share the result borrow's region and kind — a declaration
  whose result no caller can use is itself the error). Grammar verified
  natively; conformance family prepared as marked protected candidates;
  the #50 activation also records its mcts_mem decision and the
  decision-not-access writer idiom in `docs/patterns.md`.
- **W3 — evidence-gated follow-ups (claim only when triggered).** Option
  niche layout (#46) after affine buffers land, measurement first; the
  generic vector when the recorded generics+regions gap is designed;
  nested-slice CheckedType interning if a consumer forces it.
- **W4 — batch audit and owner packet.** Adversarial exit audit, batch
  economics, and one review document enumerating every approval owed:
  the v0.32 exact-byte packet and the protected conformance boundary.

## Boundaries and invariants

Candidate mode for all spec work; no activation, no chain line without
exact-byte owner approval. No protected conformance change outside marked
candidate commits. Facts-off acceptance, one normal path, no `unsafe`,
English artifacts. Blockers stop and are reported in the batch record.

## Acceptance and stop

Gate green at every landing; wfgrep's traversal slice carries a real
run over a directory tree as evidence; the v0.32 candidate verifies and
the compiler implements it behind switches before the approval packet;
the audit runs and its findings are dispositioned. Stop rather than
weaken any check.

## Exclusions

Task #44 owner rulings (separate owner session); #8 M4 packet; #17
wfgrep re-attribution until the traversal slice lands; parallelism
(outline:PAR-1 gates on flagship profiling); activation of any candidate.
