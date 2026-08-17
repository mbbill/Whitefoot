# Current Plan — close the specified-language gap; resolve take/replace

Status: ACTIVE (owner direction in conversation, 2026-08-17: "把剩下所有事情
都放进计划里面全都实现了吧……为了防止卡我审评，你就在分枝上搞就好了，这样
spec修订也不卡。你可以开agent并行。" The direction authorizes this plan and
its batches; every specification byte and every protected-compliance change
this plan produces still lands only as a marked branch candidate awaiting
the owner's exact-byte morning approval.)

Derived from Direction Outline revision 41 and the 2026-08-17 capability
review. Supersedes the completed obligation-discharge plan in place.
Active language authority: v0.30 at `spec/kernel-spec.md`, SHA-256
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`.

Active language authority: v0.30 at `spec/kernel-spec.md`, SHA-256
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`; the
W2 v0.31 bytes on this branch are a declared candidate awaiting the
owner's exact-byte approval.

## Objective

Make the compiler implement the language the active v0.30 specification
already defines, then resolve the one recorded structural blocker in front
of the flagship: §5 take/replace and the first collections layer. Both
advance outline:CAND-8 (ripgrep-class search at 2.00x), whose every missing
functional leg (regex, traversal queue, result buffers) is blocked on
collections, which are blocked on take/replace.

## Workstreams

- **W1 — specified-but-unimplemented gap closure.** The 13 pending adapter
  rows and the one runnable failure (own3 outlives-store,
  RegionsAndBorrows): named-const array sizes, arena confinement and
  arena-origin slices, float `.strict` rows, polymorphic-recursion
  rejection, `propagate` execution, Result aggregate payloads,
  borrow-affine payload match, nested direct slices, cross-region borrow
  stores. Compiler-only; no spec bytes move. Evidence per gap: the pending
  case compiled and run directly (manifest untouched); the status flips are
  prepared as one marked protected candidate commit for morning approval.
- **W2 — §5 take/replace and first collections.** Consult mcts_mem, weigh
  the recorded alternatives, design the take/replace semantics against the
  obligation-discharge model (the hole's interaction with facts and kills
  is the novel part), draft the v0.31 candidate under candidate mode, run
  the grammar verifier, implement the semantic core, and build the minimal
  library layer (growable vector, byte-string over `buffer<T>`). Extend
  generics beyond Copy exactly as far as the container design forces
  (task #39's recorded trigger). Conformance case family prepared as a
  marked protected candidate.
- **W3 — wfgrep recursive-traversal slice** using W2's collections.
- **W5 — every remaining deferred and parked item** (owner expansion,
  2026-08-17: "之前所有'还没实现的'，包括pending的，deferred，以及其他各种
  零碎的东西，都要实现"): const arithmetic and struct/enum consts
  (CONST-1/2 deferred notes), the OWN-1/FN-8 repair conflict (#35),
  arithmetic-mode dissolution (#13), grandchild reborrows and
  call-result-borrow roots, non-ASCII diagnostics, escaped host-string
  display, affine-element buffers (rides W2), DIAG-1 restructure and the
  conciseness ratchet, representation Stage 2 extraction locks, O11
  boolean composition (from its four recorded findings), a minimal
  char/text slice over W2's byte-string, and wfgrep re-attribution (#17)
  after W3. Research-grade items that cannot be closed soundly tonight
  (complete-domain proof calculi; FN-3 contracts round-2, which gates on
  writer-tier evidence) end as blocking analyses, not invented semantics.
  All spec deltas integrate into the single v0.31 candidate through the
  lead; the spec file has one writer.
- **W4 — batch audit and morning packet:** adversarial audit, batch
  economics, and one review document enumerating every approval the owner
  owes.

## Boundaries and invariants

Candidate mode for all spec work; no activation, no chain line, no
`ACTIVE-SPEC:` append. No manifest row, verdict, status, or gate-wiring
change outside commits marked as protected candidates. Facts-off
acceptance, one normal path, no `unsafe`, English artifacts. Blockers stop
and get reported in the batch record, never absorbed.

## Acceptance and stop

Gate green at the branch tip; each W1 gap carries direct-run evidence; W2's
design is recorded in mcts_mem with rejected alternatives and its candidate
verifies; the audit ran and its findings are dispositioned. Stop rather
than weaken any check; unfinished workstreams report honestly.

## Exclusions

Task #44 owner rulings; parallelism (outline:PAR-1 gates on flagship profiling); activation of any
candidate; merging to main.
