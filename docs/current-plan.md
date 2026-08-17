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
- **W3 (stretch) — wfgrep recursive-traversal slice** using W2's
  collections, only if W2's library lands with gate margin.
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

Task #44 owner rulings; DIAG-1 restructure and the conciseness ratchet;
parallelism (outline:PAR-1 gates on flagship profiling); activation of any
candidate; merging to main.
