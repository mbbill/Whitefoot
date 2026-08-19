# Current Plan — the reusable container, and the generics it requires

Status: ACTIVE (owner direction in conversation, 2026-08-19: "开一个新的
worktree开始做可增长容器吧。这个事情要做好的话涉及到标准库的设计实现，不单单
是可增长容器。把它当成标准库改进来做", followed by "开始" on the derivation
below. Every specification byte and every protected-compliance change this
plan produces still lands only as a marked branch candidate awaiting the
owner's exact-byte approval.)

Derived from `docs/constitution.md` directly, and from the batch-0072
outcome. Supersedes the completed searching-wfgrep plan in place.
Active language authority: v0.32 at `spec/kernel-spec.md`, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.

## Why this, from the constitution

Not from taste and not from what other languages have. The chain:

- **W1 (default shape is optimal shape).** A program that must collect an
  unknown number of elements has no container to reach for, so it
  preallocates a fixed buffer and grows a constant instead. `wfgrep`
  reached 64 entries per directory and 16 levels of depth that way. The
  language made the truncating program the default shape and left the
  correct shape to be hand-built. That is W1's stated failure.
- **R4 (shift left; silent corruption forbidden).** Those bounds truncate
  and return normally: exit 0, empty stderr, a hit set that looks
  complete. The current state lands on R4's forbidden rung.
- **R2 (a cut that harms AI codegen is a wrong cut; simplicity is never
  sufficient) with its own recorded precedent — generics, round-2
  checker-collapse, natural experiment Go before 1.18.** A monomorphic
  container makes every element type copy the machinery, which is
  exactly the state that precedent names as the wrong cut. So the
  container must be generic, and the generics-with-regions gap is a
  PREREQUISITE of this plan rather than a follow-up.
- **R1/R5 rule OUT one thing.** A qualified namespace, module syntax, or
  any reuse mechanism argued from readability earns nothing: R1 admits a
  construct only for P0 or P1, and R5 makes readability a non-goal. Such
  a mechanism enters this plan only if it is derived from P0 or from
  W1/W3, and PROG-1's closed-world law — itself derived from measured
  cross-module visibility and dispatch-opacity results — is not reopened
  by this plan.
- **R0.** The delta over Rust to name at approval: a container whose
  capacity discipline is checked rather than trusted, in a language with
  no writer-accessible unsafe, so the growth path itself carries proof
  obligations instead of `unsafe { ... }`.

## Objective

Make a growable, generic container expressible and used, so that a
program collecting an unknown number of elements is written the correct
way by default. Prove it by removing `wfgrep`'s fixed bounds, not by
raising them.

## Workstreams

- **W1 — generics with regions.** Close the recorded capability stop
  (`compiler/src/semantic/check/generics.rs:198`: a declaration carrying
  both generic parameters and region parameters is unsupported). Design
  first — what a region parameter means under monomorphization, how
  instance identity and the existing exact-goal machinery compose with
  it — then implement behind a default-off switch with a candidate delta
  if the specification must move.
- **W2 — the container.** A growable sequence over an arbitrary element
  type, built on the landed [SET-2] atomic replacement, with its
  capacity and length discipline carried by contracts rather than by
  comment. Its shape is decided by what W3's consumers actually need,
  not by a survey of other languages' collection APIs.
- **W3 — the consumers, which decide the shape.** `wfgrep`'s entry
  collection first: the 64-entry and depth-16 bounds disappear and the
  search is complete on a directory of four thousand entries. Then
  whichever other corpus program is forced onto a hand-built shape by
  the same absence.
- **W4 — reuse form, only if a consumer forces it.** How a container
  reaches a second program at all. Today a compilation unit spans source
  files with one flat name domain, so a `main`-free file is the whole
  mechanism. If W3 shows that mechanism failing on evidence — name
  collision in a real pair of programs, or an obligation that cannot
  cross the file boundary — that evidence opens the question. Absent it,
  this workstream lands nothing.
- **W5 — batch audit and owner packet.**

## Boundaries and invariants

Candidate mode for all spec work; no activation without exact-byte owner
approval. No protected conformance change outside marked candidate
commits. Facts-off acceptance, one normal path, no `unsafe`, English
artifacts. A container API is not designed ahead of a consumer that
needs it; migration cost is never an argument for or against a design.

## Acceptance and stop

`wfgrep` searches a directory of four thousand entries completely, with
its bounds gone rather than raised, and the completeness is pinned by a
test that fails if truncation returns. The container is generic — the
same declaration serves at least two element types in a real program.
Gate green at every landing; the audit ran and its findings are
dispositioned. Stop rather than weaken any check.

## Exclusions

The `requires`/`ensures` surface redesign (owner deferred it explicitly);
the arithmetic-trap audit; activation of any candidate; any module,
import, or separate-compilation mechanism, which PROG-1 decides and this
plan does not reopen.
