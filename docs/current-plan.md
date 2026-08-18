# Current Plan — the searching wfgrep

Status: ACTIVE (owner direction in conversation, 2026-08-18: "批准，不过你需要
帮我像上次一样生成一个解释文档。然后开始继续下一批实现". The direction
authorizes this plan and its batches; every specification byte and every
protected-compliance change this plan produces still lands only as a marked
branch candidate awaiting the owner's exact-byte approval.)

Derived from Direction Outline revision 41 and the batch-0071 outcome.
Supersedes the completed wfgrep-legs/trap-endpoint plan in place.
Active language authority: v0.32 at `spec/kernel-spec.md`, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.

## Objective

Close the one honest gap batch 0071 left: wfgrep enumerates and descends a
directory tree but cannot open what it finds, because no operation opens a
file by an enumerated name. Deliver the operation, then make `wfgrep.wf`
a real recursive search — the flagship outline:CAND-8 deliverable — and
re-attribute its measurement against ripgrep on the same corpus. Clear the
residue batch 0071 recorded while that main line runs.

## Workstreams

- **W1 — file-open-by-name and the searching wfgrep (main).** The [SYS-8]
  name-range `open_read` sibling that takes a caller-owned single path
  component instead of a `RelativePath`, mirroring `open_directory`'s
  validation exactly; then wfgrep's traversal front end over the v0.32
  enumeration surface, reading and matching each file it reaches.
  Evidence: `wfgrep` compiles and runs a real recursive search over a
  directory tree, byte-for-byte matching a reference tool's hit set on the
  same corpus.
- **W2 — flagship re-attribution (#17).** With the search real, re-measure
  against ripgrep on the recorded corpus and record where the time goes:
  traversal, read, match, or checked-semantics tax. This is the number the
  project's thesis is judged by; no optimization work is authorized by this
  plan beyond what the measurement itself demands.
- **W3 — v0.33 residue candidate (parallel; one candidate).** Retire the
  strict-in-U [OP-4]/[OP-2] rejection clauses the batch-0071 audit proved
  unreachable; state the Linux enumeration mapping or record its absence
  as a qualification failure with the same honesty the darwin binding got;
  fold in whatever W1 needs from the specification. Conformance family
  prepared as marked protected candidates.
- **W4 — recorded residue and owner rulings.** Delete `whitefoot-migrate`
  (a v0.22 one-shot ten versions past its purpose, unable to migrate the
  constructs it names); the deferred conformance case renames; the OWN-6
  and OP-5 attribution questions the audit surfaced; task #44's standing
  ruling list. Each lands as its own small change or an owner packet item.
- **W5 — batch audit and owner packet.** Adversarial exit audit, batch
  economics, and one review document enumerating every approval owed.

## Boundaries and invariants

Candidate mode for all spec work; no activation without exact-byte owner
approval. No protected conformance change outside marked candidate
commits. Facts-off acceptance, one normal path, no `unsafe`, English
artifacts. Measurement is reported as measured — a slower number is the
result, not a reason to change the measurement.

## Acceptance and stop

Gate green at every landing; W1 carries a real recursive search run with
its hit set compared against a reference tool; W2 carries the
re-attribution table; the audit ran and its findings are dispositioned.
Stop rather than weaken any check.

## Exclusions

Option niche layout (#46) and the generics+regions vector gap (#39) stay
parked until a consumer forces them; parallelism (outline:PAR-1) still
gates on flagship profiling; activation of any candidate.
