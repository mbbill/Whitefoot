# xlc — plan to self-hosting and beyond

Status: CURRENT (2026-07-13). Supersedes the 2026-07-08 pool/handle bootstrap
sketch — that architecture is preserved as a rejected alternative in
`mcts_mem/xlang/toolchain.alt/pool-based-xlc-plan.md` with the reasons it
lost. What survived from it: the whole-program single-unit model, the
LLVM-IR-text target, the trusted-shim I/O boundary, and the byte-identical
fixpoint definition. What replaced it: fixed-capacity structure-of-arrays
tapes bounded by source size (pattern P2), which is why stage 0 needs no
growable collections, no pool, and no generics.

## Where xlc stands (measured, not aspired)

- 32 xlang source files, 23,962 lines and 477 functions, compiled by stage 0
  (`prototype/democ`). The exact unit currently parses to 211,374 tokens and
  105,550 AST nodes.
- Frontend COMPLETE for xlc's own source: lexer (canonical OPNAME
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
- Facts are OFF in stage-0-compiled xlc until xlc's own effect checking is
  complete (conservative ordering; the parity corpus guards democ's facts).

## E0 — dogfood ergonomics checkpoint (active, bounded)

xlc is now large enough to test xlang's source design against real code rather
than toy examples. This checkpoint is not a request to make xlang resemble a
human-first language. A surface fact is worth keeping when it independently
prevents an AI error, supports local checking, or carries optimizer/soundness
information. Repetition that carries no independent fact is design debt.

Measured signals in the current compiler unit:

- 1,186 explicit `region` blocks; 1,142 (96.3%) wrap exactly one source line.
  They pair with roughly 1.2k explicit uniq reborrows. Boundary regions are
  useful; arbitrary fresh names around statement-scoped reborrows are suspect.
- 1,458 `True()/False()` matches; at least 1,240 (85.0%) have an empty arm.
  Match-only Bool control is therefore a high-frequency surface cost, and the
  no-`if` choice is already R3-provisional in the specification.
- 4,935 local `let` bindings all spell `own`; 4,628 bind `Bool`, `u64`, or `u8`.
  The local mode token carries no variation in this corpus. Full local mode/type
  annotation is already a provisional, unratified part of TYPE-5/OWN-2.
- 3,788 lines exceed 80 columns, 1,428 exceed 120, and the maximum is 456.
  FORM-2's mandatory one-statement line and no-wrapping rule directly causes
  this; canonical bytes do not require canonical bytes to be unreadable.
- 72.3% of local bindings are used once. Some are forced by GRAM-9 ANF, but
  `match expr` and `return expr` already accept direct calls, so some are merely
  old bootstrap style. Do not blame the language until those are normalized.
- Function names average 25.7 bytes and 124 exceed 30 because the closed unit
  has no module namespace. This is real debt, but adding modules before the
  fixpoint would expand the bootstrap target substantially.

Classification and action:

1. **Prototype now, one axis at a time:** canonical Bool `if/else` versus
   Bool `match`; local-`own` elision (`let x: T = ...` means exactly `own`, with
   no copy/move inference); and deterministic multiline formatting backed by a
   formatter. Measure source reduction, low-tier model syntax/type errors, and
   checker complexity before changing the normative grammar. A Bool surface
   change must keep one canonical spelling and exhaustive control.
2. **Soundness prototype before any syntax promise:** anonymous call-scoped
   uniq reborrow. The current one-line region ceremony is costly, but implicit
   uniq reborrow changes borrow end points and has an unresolved OWN-5/OWN-6/10
   specification boundary. Keep boundary lifetimes and escaping borrows explicit.
3. **Keep for now:** named arguments, exact effect rows, explicit signature
   modes/regions, and ANF in trapping/allocating/borrowing expressions. They
   carry checked information or preserve evaluation order. Argument punning and
   pure-expression nesting are candidates only after the first three tests.
4. **Implementation debt, not language evidence:** fixed-capacity SoA tapes,
   capability-specific semantic runners, manual report plumbing, and prefixed
   global names during bootstrap. Refactor these as coverage grows; do not add a
   language feature merely to hide a temporary compiler architecture.

E0 exit gate: choose, reject, or defer each of the three surface prototypes with
recorded evidence; add conformance cases for every chosen change; keep the old
and new grammar from coexisting as two accepted spellings. This is a short gate
before the next large S1 semantic family, not a new open-ended project.

## Stages to the fixpoint (each gated; no stage starts before the prior gate)

**S1 — body semantics parity on the compiler unit.**
Grow ownership/effect/type body checks until xlc's semantic layer renders a
verdict on every function in `sources.txt`.
The coverage baseline is complete. After E0, the next functional slice is an
independent acyclic-decision semantic family (not an enlargement of the loop
scanner profile) covering `lexer_scan_op_suffix` and `lexer_scan_word` together.
It needs nested `let`/`match`/early-return flow, primitive expression typing,
multi-argument user-call checking, named-argument mapping, and effect
containment. It adds no LLVM lowering; the current 15-function module remains
byte-identical. Freeze the new total only after implementation because any new
xlang helper also enters the audited unit. The next expected source-order
frontier is `lexer_ampuniq_at`.
GATE: differential accept/reject parity with the stage-0 checker over (a) the
whole compiler unit and (b) every conformance case whose constructs fall
inside xlc's own subset — zero verdict disagreements; every disagreement
found on the way is either an xlc bug fixed or a documented stage-0 bug with
a conformance case added.

**S2 — lowering coverage of the compiler's own source.**
Extend the `llvm_*` families until every function in `sources.txt` lowers.
The tracker is `llvm_supported`: its composed-module count climbs from 15 to
all functions in the unit.
GATE: `xlc_compile(sources)` under stage 0 emits one complete `.ll` for the
whole unit; clang accepts it; the emitted module's functions execute their
existing native tests.

**S3 — stage 1 runs.**
The stage-0-compiled xlc binary compiles real programs.
GATE: the conformance runner gains an xlc adapter and the full suite runs
through stage-1 xlc with verdicts identical to democ's (the suite is the
acceptance oracle, as always). Differences triaged exactly as in S1.

**S4 — the fixpoint.**
xlc1 (stage-0-built) compiles `sources.txt` -> IR2 -> build xlc2; xlc2
compiles `sources.txt` -> IR3.
GATE: IR2 == IR3, byte-identical. This is the self-hosting definition and it
is deliberately byte-level: canonical form end to end, no tolerance band.
After S4, stage 0 is FROZEN, not deleted — democ remains the differential
oracle (independent implementations disagreeing is how soundness bugs
surface), but it stops growing.

**S5 — facts on.**
Complete xlc's effect checking; enable the fact channels in xlc's output.
GATE: the codegen-parity corpus (bounds cases + channel pins) runs against
xlc-emitted IR with the same per-site proof accounting results as democ, and
the byte-identical fixpoint is re-established WITH facts on.

**S6 — beyond subset S.**
Only after S5: grow the accepted language beyond what xlc's own source needed
— in evidence order, not wish order. Standing queue with recorded triggers:
reborrow deltas OWN-6/OWN-14 (re-run the binary-trees pilot when they land),
copy-struct tier for AoS buffers, the region arena, the gated I/O frame
(first D4 boundary instance; unlocks the coreutils ladder), multi-field
aggregate enum payloads and Result<aggregate, E> lowering (unlocks the
streaming DecodeStep shape).

## Standing constraints

- Every stage lands in small commits, each keeping BOTH gates green
  (`make check` at root, `make check` here).
- Dogfood ergonomics changes are language changes, not opportunistic xlc
  refactors: update spec, stage 0, xlc parsing/semantics, conformance, and the
  derivation record together, one surface axis per commit.
- The self-parse gate and the byte-identical report/no-report property are
  never waived.
- Anything touching proof emission or fact channels gets hostile review
  before ship, per the standing process rule.
- Owner-gated items stay owner-gated: retained-site approvals need per-site
  cone identity first (review B1); GATE-1 claims need external repository
  controls.

## How this composes with THE-PLAN

The experiment track runs beside the build track but does not steer it before
measurement. D9a now scores one fixed low-tier model's first correctness-green
xlang artifact against a previously untuned shipped Rust library; base64 is
ineligible because it already shaped PROOF-1/2, and QOI is deferred until its
aggregate-result and fixed-array needs arrive through the normal compiler
roadmap. The selected experiment must run on xlang capabilities that already
exist when its protocol is frozen. Only a post-freeze result may motivate a
general compiler feature, which then follows the same staged, gated path as
everything else.
