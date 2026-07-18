# wfc — plan to self-hosting and beyond

Status: CURRENT (2026-07-13). Supersedes the 2026-07-08 pool/handle bootstrap
sketch — that architecture is preserved as a rejected alternative in
`mcts_mem/whitefoot/toolchain.alt/pool-based-wfc-plan.md` with the reasons it
lost. What survived from it: the whole-program single-unit model, the
LLVM-IR-text target, the trusted-shim I/O boundary, and the byte-identical
fixpoint definition. What replaced it: fixed-capacity structure-of-arrays
tapes bounded by source size (pattern P2), which is why stage 0 needs no
growable collections, no pool, and no generics.

## Where wfc stands (measured, not aspired)

- 32 Whitefoot source files, 23,962 lines and 477 functions, compiled by stage 0
  (`prototype/democ`). The exact unit currently parses to 211,374 tokens and
  105,550 AST nodes.
- Frontend COMPLETE for wfc's own source: lexer (canonical OPNAME
  discipline), full parser, AST structural validation, symbol tables with
  type/constructor namespace separation, whole-unit type resolution,
  function-scope indexing. Permanent gate: the exact `sources.txt` unit
  parses twice into identical token and AST tapes (`test_self_parse.py`).
- Body semantics now have a whole-unit source-order coverage driver. The
  measured baseline is 15 clean, 462 legal-unsupported, zero semantic rejects;
  the first frontier is `lexer_scan_op_suffix`. Legal non-profile functions are
  no longer conflated with type errors.
- LLVM lowering: by capability family — scalar, linear, buffer, checked
  scanner loops WITH proof facts — currently composing a 15-function module
  that compiles through clang and passes native-execution tests.
- Facts are OFF in stage-0-compiled wfc until wfc's own effect checking is
  complete (conservative ordering; the parity corpus guards democ's facts).

## E0 — expressiveness validation behind the capability-floor gate

The 23,962-line compiler exposed a more important question than source ceremony:
does the current language prevent a zero-cost data structure, abstraction, or
proof-carrying control shape and thereby force default AI-written code to be
slower or less maintainable? The initial plan split that question into five
ordered investigations. The E0.1 initialized-prefix review showed that the split
was premature: fixed-record construction cannot be selected before the project
establishes that common dense, sparse, keyed, stable-identity, and traversal
operations all have an efficient route.

D11 replaces the earlier monolithic G0 proposal with staged closure. A bounded
G0-Core freezes the global capability registry, coarse observable and
asymptotic contracts, safety/no-tax laws, family dependencies, reopening rules,
holdout identities, and later-gate schemas. Exact algorithms, workloads,
thresholds, soundness fixtures, and META-5 derivations freeze only in the
relevant Family Lock A immediately before candidate work. The five
investigations below remain useful evidence axes, but they do not advance merely
because a preceding local protocol closes. A closed family may seek a separate
owner-approved adoption without waiting for unrelated families; the complete
capability-floor claim has a later all-family gate. No G0-Core or family work is
yet authorized.

1. **E0.1 — data layout and owning sequences (SUSPENDED BEFORE LOCK A).** Preserve the current
   fixed-capacity SoA compiler as the baseline. First isolate compiler-verified
   flat records and fixed-capacity AoS. Flat aggregate storage neither enlarges
   the implicit-Copy class nor licenses explicit contraction/Clone; whole-record
   duplication remains unavailable unless separately designed and approved.
   Only after that decision closes, separately isolate
   an affine initialized-prefix owning sequence with capacity, no-grow push,
   explicit reserve/growth, and atomic replace. General partial initialization,
   hole-producing take, and arbitrary affine-element drop wait for their own
   soundness prerequisites. The comparison family covers layout {SoA, AoS} x
   storage policies {fixed/full-initialized, reserve-exact/initialized-prefix,
   doubling/initialized-prefix}; each comparison is reported as a total
   representation/storage-policy effect unless a genuine single-variable
   diagnostic exists. Workloads include
   append, single-column scan, mixed-field scan, full-row traversal, retained
   frontend reuse, and cold construction on the exact `sources.txt` unit. The
   goal is to add a safe choice, not to presume AoS replaces SoA.

E0.1 remains suspended before Lock A. Its paired declarative-Copy versus affine
fixed-builder protocol is now a historical draft and does not restart
automatically. G0-Core plus a dense-family Lock A are necessary but not
sufficient for any later owner-authorized reopening; that family lock must
explicitly retain, revise, or supersede every relevant E0.1 arm and
measurement. No G0-Core work, checker/compiler feature flag, candidate or
production implementation, scored run, or normative change is authorized. Any
later experimental candidate uses a separate single-semantics toolchain; dual
language modes never coexist.

The first detached E0.1a prototype closed only a feasibility question: on the
two frozen 64-bit targets, record-buffer field access can use target stride plus
row/field GEPs while unchanged SoA fixtures retain raw-IR identity. Hostile
review rejected the prototype itself: its fill loop duplicated an affine record,
the checker omitted ownership flow through index atoms, and the backend was not
32-bit/DataLayout-sound. A later paired protocol compared declarative Copy with a
full-initialization-only affine builder, but the builder cannot express the
compiler's unknown final lengths, prefix reads, backpatches, reset/reuse, growth,
or general deletion. The next discussion is therefore the bounded G0-Core
boundary, not an initialization spelling, timing, or production landing. `Flat`
never implies Copy, Repeat, Clone, or a general collection substrate.

2. **E0.2 — namespaces and zero-cost API abstraction.** Logical closed-unit
   modules, private-by-default fields, type-owned inherent `impl`, qualified
   variants, then a genuinely callable static `contract`/`conform` path. Calls
   remain monomorphized direct calls; no auto-borrow/deref, extension-method
   lookup, vtable, or implicit dynamic dispatch enters this phase.
3. **E0.3 — borrowed aggregates and fact-carrying control.** Region-generic
   aggregates with mode-bearing fields and sound holder provenance, disjoint
   slice splitting, then counted/index loops and integer/range match as real
   core nodes. Each sub-axis must separately account for retained bounds checks,
   alias facts, branch shape, and vectorization.
4. **E0.4 — byte constants and bulk output.** Canonical runtime `b"..."`
   constants plus a checked bulk-append operation. Compare rodata/bulk-copy
   lowering against the existing eight-byte scalar chunk path without changing
   output bytes or capacity/trap semantics.
5. **E0.5 — explicit SIMD and target dispatch.** Typed vectors, checked
   loads/stores, target-feature specialization, and deterministic dispatch are
   researched only after the scalar abstraction/data stack closes. The ordinary
   default call path must reach tuned kernels without writer-visible unsafe.

Every E0 phase uses the same non-regression discipline:

- **Unchanged-shape pin.** A program not using the candidate feature must retain
  byte-identical raw IR and the existing optimized-IR/assembly gate results.
  New capabilities are additive and explicit until their default-writer behavior
  is measured. In particular, E0.1 does not infer structural Copy and does not
  change the current SoA tapes.
- **Frozen factorial protocol.** Correctness corpus, source hashes, compiler and
  target flags, allocator/alignment, capacity/growth policy, measurement blocks,
  non-inferiority margin, and attribution counters are fixed before scored
  timing. Construction and retained steady state are reported separately.
- **Default-shape floor.** No feature becomes a taught/default pattern merely
  because an expert can use it well. Where a writer must choose among shapes, a
  separately authorized, benchmark-blind fixed low-tier writer panel must select
  the measured-appropriate shape without lowering correctness or the registered
  performance distribution. Until then the feature remains explicit/non-default.
- **Performance accounting.** Record wall time, instructions where available,
  allocation/reallocation count, initialized/touched bytes, peak RSS, bounds
  sites and traps, runtime alias guards, vectorizer remarks, IR/assembly size,
  vector width, and whole-record copies (`memcpy`, `byval`, `sret`). A win in one
  column cannot hide a material loss in another registered compiler phase.
- **Soundness before speed.** Any wrong output, uninitialized read, leak/double
  drop in production semantics, dangling element borrow across growth, invalid
  alias fact, missing required trap, observable padding, or failure-atomicity
  violation rejects the candidate before performance is considered.
- **Hostile review before landing.** The reviewer sees the protocol and artifacts,
  not a prose summary, and must audit equivalence, fairness, default-writer risk,
  ABI/copy/drop behavior, and every claimed attribution. Accepted changes land
  atomically across spec, stage 0, wfc, conformance, derivation record, teaching
  material, and code-shape pins; dual old/new canonical spellings never coexist.

The earlier surface observations remain recorded but are parked: 1,186 explicit
region blocks, 1,458 Bool matches, 4,935 local `own` annotations, and mandatory
single-line formatting are real source costs, not current evidence of lost
performance or architectural expressiveness. Local-own elision, Bool `if`,
multiline formatting, anonymous reborrow, argument punning, and expression
nesting do not run ahead of E0.1-E0.5.

## Stages to the fixpoint (each gated; no stage starts before the prior gate)

**S1 — body semantics parity on the compiler unit.**
Grow ownership/effect/type body checks until wfc's semantic layer renders a
verdict on every function in `sources.txt`.
The coverage baseline is complete. After the owner decides how the staged
capability work interleaves with wfc and any prerequisite family decision
closes, the next functional slice is an
independent acyclic-decision semantic family (not an enlargement of the loop
scanner profile) covering `lexer_scan_op_suffix` and `lexer_scan_word` together.
It needs nested `let`/`match`/early-return flow, primitive expression typing,
multi-argument user-call checking, named-argument mapping, and effect
containment. It adds no LLVM lowering; the current 15-function module remains
byte-identical. Freeze the new total only after implementation because any new
Whitefoot helper also enters the audited unit. The next expected source-order
frontier is `lexer_ampuniq_at`.
GATE: differential accept/reject parity with the stage-0 checker over (a) the
whole compiler unit and (b) every conformance case whose constructs fall
inside wfc's own subset — zero verdict disagreements; every disagreement
found on the way is either a wfc bug fixed or a documented stage-0 bug with
a conformance case added.

**S2 — lowering coverage of the compiler's own source.**
Extend the `llvm_*` families until every function in `sources.txt` lowers.
The tracker is `llvm_supported`: its composed-module count climbs from 15 to
all functions in the unit.
GATE: `wfc_compile(sources)` under stage 0 emits one complete `.ll` for the
whole unit; clang accepts it; the emitted module's functions execute their
existing native tests.

**S3 — stage 1 runs.**
The stage-0-compiled wfc binary compiles real programs.
GATE: the conformance runner gains a wfc adapter and the full suite runs
through stage-1 wfc with verdicts identical to democ's (the suite is the
acceptance oracle, as always). Differences triaged exactly as in S1.

**S4 — the fixpoint.**
wfc1 (stage-0-built) compiles `sources.txt` -> IR2 -> build wfc2; wfc2
compiles `sources.txt` -> IR3.
GATE: IR2 == IR3, byte-identical. This is the self-hosting definition and it
is deliberately byte-level: canonical form end to end, no tolerance band.
After S4, stage 0 is FROZEN, not deleted — democ remains the differential
oracle (independent implementations disagreeing is how soundness bugs
surface), but it stops growing.

**S5 — facts on.**
Complete wfc's effect checking; enable the fact channels in wfc's output.
GATE: the codegen-parity corpus (bounds cases + channel pins) runs against
wfc-emitted IR with the same per-site proof accounting results as democ, and
the byte-identical fixpoint is re-established WITH facts on.

**S6 — beyond subset S.**
Only after S5: grow the accepted language beyond what wfc's own source needed
— in evidence order, not wish order. Standing queue with recorded triggers:
reborrow deltas OWN-6/OWN-14 not selected by E0.3 (re-run the binary-trees pilot
if they land), the region arena, the gated I/O frame (first D4 boundary
instance; unlocks the coreutils ladder), multi-field aggregate enum payloads
and Result<aggregate, E> lowering (unlocks the streaming DecodeStep shape).

## Standing constraints

- Every stage lands in small commits, each keeping BOTH gates green
  (`make check` at root, `make check` here).
- Dogfood ergonomics changes are language changes, not opportunistic wfc
  refactors: update spec, stage 0, wfc parsing/semantics, conformance, and the
  derivation record together, one surface axis per commit.
- The self-parse gate and the byte-identical report/no-report property are
  never waived.
- Every E0 candidate and anything touching proof emission or fact channels gets
  hostile review before ship, per the standing process rule.
- Owner-gated items stay owner-gated: retained-site approvals need per-site
  cone identity first (review B1); GATE-1 claims need external repository
  controls.

## How this composes with THE-PLAN

The experiment track runs beside the build track but does not steer it before
measurement. D9a now scores one fixed low-tier model's first correctness-green
Whitefoot artifact against a previously untuned shipped Rust library; base64 is
ineligible because it already shaped PROOF-1/2, and QOI is deferred until its
aggregate-result and fixed-array needs arrive through the normal compiler
roadmap. The selected experiment must run on Whitefoot capabilities that already
exist when its protocol is frozen. Only a post-freeze result may motivate a
general compiler feature, which then follows the same staged, gated path as
everything else.
