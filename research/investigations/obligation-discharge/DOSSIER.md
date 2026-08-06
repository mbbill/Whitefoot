# Obligation-discharge semantics: trap as the checker's runtime backstop

Status: DISCUSSION-STAGE DESIGN (recorded 2026-08-06). This dossier freezes the
owner/assistant design discussion of 2026-08-05..06 on trap semantics. It is
design evidence only: no specification authority, no roadmap item, no
implementation authorization. If the direction is adopted, the language-change
loop in `docs/WORKFLOW.md` owns it and this file gains a supersession pointer;
if the validation plan below refutes it, the refutation is recorded here and
the dossier is parked. Removal condition: superseded by an active spec version
that lands or rejects the design.

Relation to current law: this design **contradicts v0.17 in deliberate,
enumerated ways** (it dissolves OP-4 implicit checks, FN-8 unconditional entry
execution, and the `.trap`/`.checked` operation modes). Nothing here weakens
any current rule until the spec loop says so.

## 1. Problem

The starting criticism: Whitefoot programs trap. A `requires` violation or
bounds failure aborts the process, so a bad parameter that reaches a contract
looks like a random crash — unacceptable in critical software. Pressure-testing
that criticism surfaced four real defects in the current story, each sharper
than the original complaint:

1. **Discoverability.** A caller cannot completely know when a callee will
   trap. The contract is machine-readable (FN-8) but nothing forces the
   caller to consult or satisfy it before runtime.
2. **No enforceable classification.** ERR-4 fixes trap-vs-value per table
   operation, but for user functions the trap/error split is doctrine (P9),
   not mechanism. An untrusted AI writer will misclassify, and "expected
   failure probability" is unverifiable in principle (a broken kernel API
   turns a "can't happen" into a 100% event), so any probability-based
   standard is wrong by construction.
3. **Unreviewable assertions.** "This can't fail" (Rust `unwrap` culture) is
   an unaccompanied claim. Whole-program reachability of a trap is beyond any
   author, human or AI.
4. **Effect-row saturation.** `traps` in EFF-1/EFF-2 rows is syntactic and can
   never tighten, so in a large program the bit is on nearly everywhere and
   carries no caller-usable information.

Rejected wholesale fixes, with the arguments of record:

- **Result-everywhere** (make every fallible operation return a value).
  Rejected: every checker-incompleteness site then forces the writer to author
  an error arm for a condition that is impossible when the code is correct.
  Those arms are untestable-in-truth, reachable-in-bug, and written by the
  writer we do not trust — the silent-wrong-answer floor. A catchable internal
  error is also a W3 hole: a stuck writer swallows `Err(Internal)` at the
  innermost frame and goes green. The lazy floor of trap semantics is a loud
  deterministic stop; the lazy floor of Result-everywhere is a plausible
  wrong answer.
- **Global prove-or-handle** (unproven assertion = compile error, always).
  Rejected as global law: with a deterministic no-search checker the
  unprovable-but-true residue is large, and forcing it into Result camouflages
  the discharge worklist inside genuine fallibility. Adopted instead as an
  opt-in per-partition/per-function mode (empty claim ledger).
- **Assume-without-check** (SPARK `pragma Assume` shape: prover accepts a
  stated fact with no runtime check). Categorically rejected; see the W3
  keystone in §3.

## 2. The design

Three constructs replace today's implicit-trap machinery.

### 2.1 goal — proof obligations pinned to definitions

Every partial operation carries a predicate that makes it safe: the operation
table's predicates (`i < len`, divisor nonzero, product fits u64, sum fits T)
and, for user functions, the `requires` block. A goal is written once, by the
definer, in the existing FN-8 pure/total fragment.

### 2.2 Callee single-form; call-site discharge

A function compiles in exactly one form: **uncheck**. Its body takes the goal
as an axiom (used to discharge the body's own obligations, not merely to
optimize). There is no entry check, no checked adapter, no dual entries.

Every call site must **statically discharge** the callee's goal, instantiated
with the actual arguments. Evidence sources: dominating branch conditions,
prior claims, the enclosing function's own requires, and (future) ensures and
witness types. Discharge failure is a **compile error that prints the
residual** — the exact unproven remainder after the checker consumed all
available facts. The checker never auto-inserts checks; obligations stay
explicit in source.

Because Whitefoot is ANF, actuals are atoms, so instantiation stays inside the
goal fragment: **the residual is always expressible as a claim or branch
condition.** The zero-skill fallback — paste the printed residual as a claim
at the call site — always closes discharge in one step and reproduces today's
entry-check semantics exactly, except per-site specialized: each call site
pays only what the checker could not derive there, from zero (fully proven) to
full price (no facts). Today's FN-8 entry check is the one-size-fits-all
worst case of this scheme.

### 2.3 claim — the only trap source

`claim <name>: <predicate> because "<justification>"` compiles to a runtime
check of the predicate; a false predicate is a deterministic trap (EFF-4-style
abort with a DIAG-3-style record citing the claim's name and operand values).
A passing claim contributes its fact to dominated code.

- A claim is legal **only where the checker cannot already prove the
  predicate** (a provable claim is a redundancy warning → delete).
- A claim is illegal on **tainted** predicates (§2.5).
- Placement is free subject to fact stability (§2.6): claims may be hoisted
  out of loops. Hoisting is a semantic edit (it strengthens the program's
  failure condition — eager trap), visible in review by the claim's name.
- Trap therefore has one meaning: **a claim violated at runtime**. A claim is
  a branch whose else the writer refuses to write; the language, not the
  writer, authors the failure path.

### 2.4 branch — ordinary control flow as discharge evidence

`if R { call } else { ... }` discharges the same residual with both outcomes
written. The else belongs to the writer's domain semantics (return a domain
error, retry, degrade, default). There is no ContractError type and no derived
`.checked` function form in the kernel: failure classification happens at the
call site, the only place that knows the data's provenance, in the caller's
own vocabulary. (A `.checked` convenience spelling may exist as pure sugar.)

### 2.5 taint — the mechanical trap/value classifier

Values originating at the gated boundary (§14 imports — the only entry for
outside data) carry provenance, propagated by dataflow.

- **Tainted residual → branch is mandatory, claim is illegal.** A trap
  conditioned on outside data is "bad input crashes the program," the
  original defect. Mixed residuals are partitioned mechanically by conjunct:
  claim the clean conjuncts, branch the tainted ones.
- **Clean residual → claim is the reference shape.** Not mechanically
  forceable (see §4, W1 item), floor-audited instead.
- Laundering is what keeps taint from saturating: (a) control flow does not
  propagate taint — matching on tainted bytes yields program-chosen values,
  so parsing is the laundering machine; (b) structure metadata is always
  clean — T1 means lengths, tags, and layout are program-maintained even when
  contents are attacker-chosen, so `len(x)` always tells the truth; (c) a
  branch establishes its predicate regardless of operand taint, because
  safety predicates are relational (any array is indexable below its own
  length, whoever chose the length). What stays tainted: raw content values
  carried forward (file-claimed sizes, offsets, text) — historically the CVE
  surface (decoder size arithmetic, font index lookups), which is exactly
  where the rule forces checked arithmetic and branches.

Consequence: **the trap-vs-value standard is modal and largely mechanical** —
"can this predicate be false in a correct program?" Environment-origin facts
can (→ branch/value); internal facts cannot (→ claim/trap). No probability
judgment anywhere.

### 2.6 Fact stability (kill rules)

A fact established at one point licenses an uncheck use later only while it
survives. Facts are killed by: (a) assignment to a free variable of the
predicate; (b) any call whose effect row writes a region containing a free
place of the predicate; (c) expiry of the borrow the fact reads through.
Exact, bidirectionally-checked effect rows make (b) a signature lookup — the
analysis that drowns in aliasing elsewhere is modular here. This judgment is
new normative machinery and must be specified to v0.17 precision.

### 2.7 Claim lifecycle and ledger

The compiler enumerates every claim (and every branch-discharge) with: site,
predicate, taint status, residual provenance, justification text. This is the
discharge worklist and the review surface.

- Checker upgrade proves a claim → **warning** (delete the line). Warning,
  not error, is what keeps acceptance monotone across spec versions.
- Checker upgrade refutes a claim → **hard error** (bug found at compile time).
- A claim that ever fires (test, fuzz, production) is by definition not a
  necessary truth → auto-escalated for reclassification.
- Review is of ledger deltas, not code: rows carry predicates and
  justifications; approvals anchor to claim name + predicate hash (canonical
  bytes make invalidation-on-edit mechanical). A wrongly approved claim costs
  a loud bounded stop, never corruption — which is why sampled, risk-weighted
  review is sound here while the unsafe/gated ledger (LEDGER-1) requires
  exhaustive review.
- Partition policies: `deny-claims` (critical code: ledger empty — the
  prove-or-handle mode, achieved by proof and branches only),
  `review-required`, `audited-free`. Foreign-reachable entries always get a
  synthesized boundary adapter (§2.8) regardless of policy.

### 2.8 Boundary adapters

Caller-side discharge requires a call site; foreign entries have none. The
§14 gate synthesizes a checking adapter for every externally reachable
function. Foreign arguments are tainted by definition, so the adapter's
failure path follows the boundary's error protocol (error return to the
foreign caller), not a trap: the environment has no trap authority.

### 2.9 Operation-table collapse

`.trap` and `.checked` modes dissolve: each partial operation has one uncheck
form plus a goal, discharged like any call (claim or branch on the residual;
`iadd` overflow, `idiv` zero/overflow, `index` bounds, `buffer_new` size all
uniform). Genuinely different total semantics remain as operations:
`.wrap`, `.sat`, rotate-masking. Codegen must guarantee **claim fusion**: a
claim adjacent to its sole use compiles to today's fused check (hardware
overflow flag, single bounds compare), byte-for-byte cost parity — otherwise
the design regresses P0 on hot paths.

### 2.10 What Result becomes

Result returns to plain data modeling of expected outcomes (ERR-4's value
category: parse failures, environment failures, not-found). It is no longer a
contract mechanism or trap's counterpart. The trap/Result boundary question
that opened the discussion dissolves: trap is the checker's runtime backstop
at its provability frontier; Result is what an else-arm or an expected-outcome
API returns.

### 2.11 Effect-row consequence

Claims live in callers, so a leaf function with a goal but no internal claims
has a clean row. `traps` stops saturating and starts meaning "an undischarged
assertion sits under here" — the row regains signal, answering defect 4 of §1.

## 3. Arguments of record

- **W3 keystone: no construct may introduce a fact without either a proof or
  an executed runtime check.** This is the door SPARK's `pragma Assume` leaves
  open and the single invariant that keeps the prover honest against a
  cheating writer. Every extension must preserve it.
- **Authorship factoring.** The AI writes predicates (data) and justifications
  (auditable prose); the language writes failure behavior (uniform,
  spec-fixed); the owner writes partition policy; the human signs ledger
  deltas. The untrusted party never authors what happens when it is wrong.
- **Dead-arm argument.** Forcing writers to fill impossible-error arms is a
  structural demand for lies; trap semantics makes honesty the cheapest shape
  and lies an auditable extra (W1 floor logic).
- **Interface honesty.** requires-as-goal completes FN-1 for value
  constraints: "compiles ⟹ arguments are legal." The class of runtime
  regressions where a signature stays stable while the implementation's
  implicit expectations move becomes compile errors with printed residuals.
  Contract changes become reviewable canonical-byte diffs. Interface ripple
  on a strengthened requires is the interface telling the truth; with AI
  writers patching from residuals, the friction is cheap. Named delta: Rust
  cannot express this; SPARK expresses it with nondeterministic solver
  discharge.
- **Checker as teacher.** The residual-printing loop (fail → read residual →
  claim or branch → compiles ⟹ satisfied) answers defect 1: the contract is
  conversational at compile time, strictly better than `# Panics` prose and
  than runtime discovery.
- **TCB stance.** The entailment fragment joins the TCB exactly like the type
  and borrow checkers; the language does not hedge against its own compiler.
  A replay build mode (re-materialize every discharged check, trap on
  divergence) exists as a test instrument for finding entailment bugs, not as
  language semantics.

## 4. Costs and open problems

1. **W1 floor: if-instead-of-claim** (owner-flagged top test item). A lazy
   writer can discharge every clean residual with a branch whose else
   fabricates a default — reintroducing dead arms voluntarily. Not
   mechanically preventable: the checker's incompleteness is precisely what
   hides the else's deadness at authorship time. Treatment: (a) hard rule in
   one direction only (taint → branch); (b) floor audit pattern — branch on a
   clean residual whose else only constructs a default/error is a shape
   divergence from the claim reference shape; (c) retroactive lint — on
   checker upgrade, a provably-true branch condition flags the else as dead.
   Damage is quality (misclassification, dead code, lost elimination), never
   soundness. Needs a designed floor test.
2. **The entailment fragment becomes normative.** Kill rules, congruence,
   interval arithmetic, taint partitioning — all spec text at v0.17
   precision; every checker strengthening is a spec version; acceptance
   monotonicity depends on redundant-claim being a warning.
3. **Entailment correctness is now T1-critical at every call site** (a wrong
   discharge compiles a raw out-of-bounds access). Owner ruling: this is a
   compiler bug class, owned by testing (conformance corpus for the fragment,
   replay mode as oracle), not by language-level hedging.
4. **Loop-carried facts still need induction.** A loop-head claim matches
   today's per-iteration cost; real hoisting of loop-carried bounds waits on
   a loop-induction (or interval-widening) extension. Claims relocate the
   checker gap and price it visibly; they do not close it.
5. **Vertical threading tax.** Deep chains must forward requires clauses
   through intermediate signatures; brittle under goal changes (though
   honestly so). Pressure valves: witness types (facts travel in values;
   needs constructor privacy, absent in v0.17) and **ensures** — the missing
   symmetric construct (callee establishes, caller uses). The construct
   family: claim (same scope establishes and uses), requires (caller
   establishes, callee uses), ensures (callee establishes, caller uses).
6. **Claim-vintage texts.** Residual-shaped claims reflect the checker
   version they were written under; the redundancy lint is the cleanup path.
7. **Indirect calls (future)** force goals into function types with variance
   rules (an implementation may require less, never more). v0 has no
   indirect calls, so deferred, but the direction is fixed.
8. **Taint granularity engineering.** The laundering rules of §2.5 are what
   stand between this design and taint saturation on input-processing
   programs (a browser-shaped workload is the stress case). Needs the probe
   in §6.
9. **ContractError/justification bikeshed.** Whether `because` is grammar or
   policy; what a claim record carries; how goals interact with contract
   members (FN-8 currently defers requires on `fn_sig`).

## 5. Prior art and the null hypothesis

Lineage, borrowed deliberately: hybrid type checking (Flanagan; Sage) — static
where decidable, dynamic checks where not; Dafny `expect` (runtime-checked
assumption); liquid/refinement-type call-site discharge; Deputy/CCured
(inserted runtime checks for unproved dependent facts); SPARK/GNATprove
justification workflow and assumption registers (seL4, DO-178C) for the
ledger shape.

**Standing discipline: "SPARK + LLM-written annotations" is the null
hypothesis for this entire lane.** Any claim of value here must name its
delta, R0-style. Deltas of record: (a) no writer-accessible assume — the W3
keystone is structural, where SPARK's discipline is social and an AI writer
makes `pragma Assume` audit load explode; (b) deterministic spec-pinned
discharge vs SMT portfolios with timeout/replay instability — also a clean
reward signal for iterating AI writers; (c) taint as a compile gate vs
process; (d) exact effect rows as the kill-rule oracle; (e) one fact base
feeding prover and optimizer. Plausibly novel as a composition: caller-side
discharge with single-form callees and per-site residuals, taint-forced
trap/value classification, and claims as the sole trap source with a
ledgered lifecycle, under an untrusted-writer governance frame.

## 6. Validation plan (cheap falsifiers first)

1. **Hand-simulation** on one or two real programs from the native corpus:
   classify every current trap site into proven / claim / branch under a
   v0.17-strength entailment; record bucket counts, residual sizes, hoist
   opportunities, threading-tax occurrences. Kills or grounds the design for
   two days' cost. Prediction to falsify: most bounds obligations discharge
   or reduce to one-line residuals; threading tax appears but stays shallow.
2. **W1 floor probe**: have a writer solve discharge errors lazily; measure
   whether if-instead-of-claim divergences are mechanically detectable
   against the reference shape.
3. **Taint saturation probe** on an input-processing program (parser-shaped):
   measure what fraction of values stay tainted past the parse layer.
4. **Codegen parity spot-check**: fused claim vs today's bounds/overflow
   check, byte-level.

Owner's stated concern to keep honest: the discussion may overfit to itself —
"到真实场景里面试一下发现根本不work." The validation plan exists to answer
exactly that before any spec motion.
