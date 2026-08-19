# The contract surface: the design space

Research document. Nothing here is approved. It changes no specification byte,
no compiler line, no test, and no conformance case. Written against
`spec/kernel-spec.md` v0.32, SHA-256 `5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`
(re-verified with `shasum -a 256` on branch `batch-0072`).

This replaces the four-proposal framing at
`research/investigations/contract-surface/PROPOSAL-{A,B,C,D}-*.md`. Those are
four points somebody imagined. This is the space, what the rules already
decided in it, and what is left.

The spec already says this surface is unvalidated. Its own R3-PROVISIONAL
register (`spec/kernel-spec.md:9`) lists "the `requires { requires_entry* }`
surface spelling with its FN-8-checked ordinary-let/final-check subset (FN-8 —
semantics selected, spelling not yet compared)". Semantics selected, spelling
not yet compared. That is the whole subject.

**Line numbers.** Every spec line cited below was read directly from the v0.32
file at that digest. Several citations that reached me from the per-axis
analysts were off by one to five lines; where mine differ from theirs, mine are
what the file says.

**Populations.** `tests/programs` (8 blocks in 6 files) and
`research/experiments` (3 blocks in 3 files) are program-shaped: 11 blocks
total. `tests/conformance/cases` (58 in 44), `tests/codegen/cases` (37 in 37 —
mutants of one contract), and `research/investigations/*/probes` (16 in 13) are
test-shaped and are evidence about the test suite, never about writer demand.
Counts re-measured this session; they match the lead's 122 blocks in 103 files.
Every corpus number below names its population.

**One correction to the brief, load-bearing.** The brief states that every trap
message in `tests/programs` "says something the predicate does not". It does
not. I read all eight:

| predicate (after erasure) | message |
|---|---|
| `ige(len(deref(out)), len(src))` | "output too short" |
| `ige(len(deref(out)), len(src))` | "output too short" |
| `ile(filled, len(deref(destination)))` ×2 | "append filled exceeds destination" |
| `ile(result, len(deref(destination)))` ×2 | "append result exceeds destination" |
| `ile(literal_count, len(deref(literal_lengths)))` | "literal lengths shorter than literal_count" |
| `ile(result, mask)` | "read bits result exceeds mask" |

Eight of eight are the predicate's negation re-spelled in English. What the
message adds over the predicate is one fault-ascribing word — "too",
"exceeds", "shorter" — and nothing else. No message states a fact the predicate
does not. Deleting the message channel loses no stated fact in the
program-shaped population. This matters because it removes the strongest
intuitive argument for axis 5.

---

## 1. The frame

Eight axes. I add no ninth. I do flag two things inside existing axes that the
frame's wording hides, and say so where they arise (§3.1 and §3.6).

**1. BINDING FACILITY** — whether the contract clause contains a way to name an
intermediate value, and if so which. `none` | `statement lets` (today) |
`expression-local binding` | `unit-level named propositions referenced by
contracts`. This is the root of the dependency graph: it determines axis 8
entirely, forces axis 3 in one direction, and gates axis 2.

**2. TERMINAL MARKING** — how the line that carries the proposition is
recognized. `check` (today) | a different keyword | positional, no keyword |
the whole clause is one expression. The last value is not a marking choice at
all but a deletion of the block, which is why it drags axes 1, 3 and 5 with it.

**3. CARDINALITY** — how many propositions one clause may state. Exactly one
(today, so several preconditions are squeezed into one `band` tree) | several
entries, juxtaposition meaning conjunction. This is the axis that changes
which programs compile.

**4. SHARED FORM** — whether `requires` and `ensures` are one clause form or
two. They share a form today. The only enforced difference between them in
v0.32 is whether a non-comparison Boolean root is admitted.

**5. PROSE CHANNEL** — where the English about an obligation lives. The trap
STRING (today, inert) | a `doc` entry inside the block | none.

**6. IDENTITY** — whether a contract carries a writer-chosen token of its own.
Anonymous (today) | named.

**7. EXTRA JUDGMENTS** — what the compiler says about the contract itself, as
opposed to about calls to it. None (today) | refutation of a self-contradictory
contract | redundancy advisory | both.

**8. GOAL-TEMPLATE SOURCE** — the tree left after erasing the bindings (today,
via alpha expansion) | the written tree directly. As worded this axis is a
trap: under three of axis 1's four values the two options name the same object
or one of them is unusable. Restated so it survives (§3.6), it is the
structural-versus-nominal identity question for propositions, and it is live in
exactly one corner of the space.

---

## 2. What the constraints already decided

Six determinations survived adversarial challenge. The owner does not need to
think about these.

### 2.1 Expression-local binding is dead

`[GRAM-9]` at `spec/kernel-spec.md:299-300` is unconditional and not
position-enumerated:

> A computed value is forwarded to another operation only by binding it with a
> preceding `let` (whose mode and type are derived [TYPE-5]) and referencing
> the binding. Nesting and let-splitting are not two spellings of one
> computation; there is no expression-nesting alternative [FORM-1].

`[GRAM-6]:252` adds "composition is by `let`". An enclosing `let x = e in P` is
neither preceding nor the statement `let`, so it is a second spelling of
binding, barred by `[FORM-1]:29` ("There is exactly one spelling per semantic
construct"). Scoping it to contract position only is precisely what
`[META-2]:3258` forbids: "No context-dependent spellings or rule variants: no
rule's meaning depends on surrounding context." The in-spec precedent is
`[GRAM-8]:294` — a name-only-when-ambiguous alternative was rejected on exactly
this ground.

To revive it you would replace the statement `let` language-wide. That is a
change to the body language, not to the contract surface.

### 2.2 Unit-level propositions in the opaque reading are dead

A proposition reference that never unfolds anywhere has no discharge route.
`[ENT-3]:2799`: "No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`,
`ene`, user-function results, and deeper indirection chains contribute no L0
comparison origin in this version." `[ENT-4]:2882` leaves `+G` derivable only
from that exact positive fact or from an exact comparison projection.

The half that is usually missed is the callee side, and I measured it. In
`s4_yes.wf` a `requires` whose expanded root is `ilt(i, len(deref(out)))` — a
comparison over two admitted `[ENT-2]:2757` terms — discharges the body's own
`set deref(out)[i]` bounds obligation and the file compiles. In `s4_no.wf`,
identical except that one root operand is an operation result (`i +wrap 1`) so
the root no longer projects, the same body is rejected `[OP-4]
UndischargedBoundsObligation { residual: "i < len(deref(out))" }`. An opaque
proposition never projects, so under the opaque reading all 8 `tests/programs`
contracts lose the thing they exist for.

What survives is the *expanding* reading, where the reference is erased into
its definition before the template exists — and, separately, the nominal
reading of §3.6, which unfolds for projection while retaining the reference for
goal identity. Both are live. Only "opaque, never unfolds" is dead.

### 2.3 The trap STRING is dead as a prose channel

Its complete reader set in v0.32 is four rules, all verified:

- `[OP-5]:889` — "The final `check_stmt` in a `requires` block uses this exact
  condition judgment, decoded message, and dynamic-boundary failure behavior,
  but [FN-8] owns its execution: it is no ordinary-callee runtime check, and
  only program start plus a later implemented gated adapter evaluate it."
- `[PROG-3]:1561` — "A false result emits the final `check_stmt`'s exact [OP-5, DIAG-3] trap record".
- `[GATE-1]:2070` — "A false result must retain the final `check_stmt`'s
  [OP-5, DIAG-3] trap semantics."
- `[DIAG-3]:2048` — "For an [FN-8] program-start goal, `rule_id` is `OP-5` and
  `message` is the final `check_stmt`'s STRING value decoded by [FORM-5]."

Owner ruling 1 deletes the PROG-3 and GATE-1 evaluations; ruling 2 deletes the
program entry. No rule then reads the STRING. `[FN-9]:1295` already ran this
experiment on the other half: "its message, clause-local spelling, and sharing
have no identity, it contributes no `traps`, it never executes, and it emits no
[DIAG-3] record."

Be precise about what is killed: the value *as stated* — the trap STRING
serving as the prose channel — because after the rulings no rule reads it.
Retaining the bytes would violate no rule I can name; `[FORM-4]:77` is enforced
lexically only, and `[CLM-1]:2692` proves inert prose can be legal ("mandatory
compile-time review data retained by the checked program [DIAG-2], absent from
runtime behavior, and never semantics-selecting"). But to save the message you
would have to give it a CLM-1-style DIAG-2 retention sentence, which
reclassifies it as a justification — a different axis-5 value, not this one.

**Composition consequence worth seeing.** Ruling 1 read as written also deletes
`[GATE-1]:2070`'s admission path, so the deferred foreign-boundary design loses
its specified way to admit a contract-bearing function: a foreign caller cannot
be statically discharged. That is a real cost of ruling 1, and it is not
recoverable by any axis choice here.

### 2.4 Redundancy as a hard rejection is dead; as an advisory it is not

`[ENT-1]:2741` is law: "Version monotonicity of fact-source and closure
strengthening is law with one enumerated exception: a later specification
version may add fact sources and closure rules, and that strengthening removes
none, so it never converts a discharged obligation, call goal, or selected-return
relation into an undischarged one and never converts a claim into a
redundancy-ground rejection."

A contract-redundancy rejection is the same non-monotone edge: a stronger
closure would newly derive a previously underivable contract and newly reject a
previously accepted program. `[CLM-2]:2699` states the surviving form and its
monotonicity argument verbatim, and `[CLM-2]:2703` leaves the channel
implementation-owned. This is a derivation from the law's stated rationale plus
an explicit in-spec precedent, not a literal application of a sentence about
contracts; I flag that so the owner can check it.

### 2.5 Erasure is forced wherever a clause-local binder exists

Three rules together. `[ENT-2]:2759`: "Two places are the same term exactly
when their roots resolve to the same declaration event [TYPE-6, DIAG-1] and
their canonical source spellings [FORM-2] are byte-identical."
`[TYPE-6]:394`: "a requires-block let is visible only to later requires entries
and not to the ensures block or function body [FN-8]." `[FN-8]:1247`: "It then
substitutes each formal datum in the concrete GoalTemplate with that actual's
pre-transfer value image" — **only formals**.

So a template retaining a clause-local datum names a declaration event that
exists in neither the caller nor the callee body. `[ENT-4]:2882`'s route (a)
has no source, because `[ENT-2]:2778` fixes the caller's goal universe as
"exactly the goals formed from its written Bool conditions, checks, claims,
requirement S4, and ordinary-call requirements". Route (b) needs operands that
are admitted terms of the querying function. The shared-subtree escape is barred
by `[ENT-2]:2772` — "A concrete goal is one finite typed expression tree" — a
DAG is not a tree.

The callee side fails independently, and `s4_yes`/`s4_no` above is the
measurement: the projection reaches the body only because `capacity` has been
erased to `len(deref(out))`, a term `[ENT-2]:2757(b)` admits. Note also that
`tests/programs/raw_deflate_boundary.wf` binds the byte-identical
`let capacity = len(deref(destination));` at line 18 (requires), line 22
(ensures) and line 26 (body) — three distinct terms under `[ENT-2]:2759`. Only
the erasure makes those one goal.

The trap is well-formedness, not ill-formedness: `[ENT-2]:2757(a)` explicitly
admits "a requires-clause local" as a term root, so a written-tree template
would parse, type-check, and fail silently at discharge. The compiler could not
diagnose the design's own defect.

One case correction the original analysis overstated: `[ENT-4]:2886` makes
everything derivable at a contradictory point, so the precise statement is
"undischargeable at every **non-contradictory** call site".

To revive the written tree here, `[ENT-2]:2759` would have to admit a
callee-clause-local declaration event as a caller-substitutable datum. That is
a change to the fragment's identity core.

### 2.6 The `ensures` half of axis 8 is determined outright

Not derived — stated. `[FN-9]:1305` "Recursively alpha-expand the ensures
locals by their unique definitions exactly as FN-8 expands requires locals";
`[FN-9]:1309` "No operation result, clause local, arithmetic expression,
subscript, ephemeral actual datum, Boolean connective, nested result
projection, or body local becomes a relation term"; `[FN-9]:1313` "It excludes
binder spelling, let spelling or sharing, message bytes, clause-local
NodePaths, and callee-instance identity."

The deeper reason, which matters because it survives every axis-1 choice:
`[FN-9]:1310` publishes an L0 RelationTemplate, and L0's vocabulary is terms
and difference bounds (`[ENT-2]:2785-2786`). A nominal proposition has no L0
denotation except through unfolding. So even under the unit-level-proposition
design of §3.6, `ensures` must unfold. Axis 8 splits: determined for `ensures`,
live for `requires` in one corner.

---

## 3. What is genuinely open

Ranked by how much the choice changes the language. Axis 1 leads because it is
upstream: it forces axis 8, forces axis 3 in one direction, and gates axis 2.

### 3.1 Axis 1 — BINDING FACILITY (the root choice)

**Live values.** `none` | `statement lets` (today) | `unit-level named
propositions` (expanding or nominal).

**What `statement lets` makes true.** Contracts can state a relation whose
operands are computed. Measured: `req_arith.wf` — a `len` → `*wrap` → `+wrap` →
`ile` chain of four lets — compiles, rc=0. It is also not an imported habit:
`[GRAM-2]:166` makes `requires_entry := doc | stmt | check_stmt` where `stmt`
is the body's own `stmt`, and `[GRAM-9]:299` names `let` the unique forwarding
mechanism in the language.

**What it makes false.** The `let` keyword binds nothing at any time in this
position. `[FN-8]:1233` erases every clause local before the template exists;
after owner ruling 1 nothing in the clause is ever computed. The spelling
asserts an evaluation that does not occur — which is the same defect as
`else trap`, one construct over.

**And it admits several byte-distinct spellings of one goal.** The spec says so
itself at `[FN-8]:1238`: "Clause-local spellings, clause-local NodePaths, and
whether identical subexpressions were shared through one let are absent after
expansion." The corpus splits on it: 5 of the 8 `tests/programs` blocks bind
the final condition (`check sufficient`, `check admitted` ×2,
`check enough_literals`, `check enough_output`) and 3 inline it
(`check ile(result, capacity)`, `check ile(result, mask)`) — and
`tests/programs/wfgrep.wf` uses the bound form at line 126 and the inline form
at line 129, four lines apart in the same declaration. In every one of those 5,
the Bool binding was **not** forced by GRAM-9: both comparison operands were
already atoms. Against `[FORM-1]:29` that is one construct with at least two
admitted spellings.

**What `none` makes true.** No binder, so no name to check, no let order to
vary, no `let` keyword asserting a non-event, and no clause-local scope — which
deletes `[TYPE-6]:394`'s three-way isolation and the triple restatement it
forces (`raw_deflate_boundary.wf` writes the same `len` binding three times in
one declaration; `wfgrep.wf` does the same at lines 124, 128, 132).

**What `none` costs, and this is the crux.** `[GRAM-9]:298` caps the clause at
one operation over atoms: "a `call` or `construct` in an atom position does not
derive under the grammar and is a hard error citing GRAM-9." Probe `nested.wf`
— `requires { check ile(filled, len(deref(destination))) else trap "too long"; }`
→ `Parsing/Source [GRAM-9]`. So under `none` **no contract in the language can
mention a length**, and `[FN-9]:1307` admits `len(P)` as a relation operand
outright, making a specified operand class unreachable surface.

**The measurement that decides this axis, and it is sharp.** In the
program-shaped population, `tests/programs`, I read all 8 blocks. Seven bind at
least one clause-local. **Every single value that is genuinely forwarded into
another operation is a `len(P)`.** Nothing else. Zero blocks forward arithmetic.
The remaining bindings are Bool names for the comparison itself, which GRAM-9
does not require.

And `len(P)` is not a computation in the fragment's own vocabulary. It is a
primitive: `[ENT-2]:2757(b)` "a length term `len(P)`, of fragment type u64";
`[FN-9]:1307` admits it directly as a relation operand. The entire binding
facility, in every contract the corpus actually discharges, exists to route one
term around GRAM-9 — a term the entailment fragment already carries
first-class.

**Flagged, not a ninth axis.** `none` is relevance-refuted at 7 of 8 unless the
`atom` production admits `len(P)`. That widening is a precondition on an
existing axis-1 value, not a new axis. It is language-wide (so META-2-clean),
and it makes the surface match `[ENT-2]`'s term grammar rather than the
statement grammar. Under it, all 8 program-shaped blocks become single flat
comparisons over `{parameter datum, len(P), literal, const}` — which is
*exactly* the operand vocabulary `[FN-9]:1307-1309` already fixes for `ensures`.
The ensures rule already names the term grammar a contract needs; the surface
just does not use it.

**The only program-shaped evidence for arithmetic in a contract** is the 3
`research/experiments` kernels — `b64.wf` (a depth-4 arithmetic derivation),
`huffman_literals.wf` (16 lets), `match_copy.wf` (27 lets, a hand-split 128-bit
product-overflow chain). Its weight is weak on two counts I verified myself.
None of the three parses under v0.32 — all fail `Parsing/Source [GRAM-4]` at
their first typed clause-local (`let x: own T = ...`; `[GRAM-4]:205` is
`let_stmt := "let" IDENT "="`, no annotation). And each declares one function
with **zero call sites** anywhere in the tree (`inflate_match_copy`,
`inflate_huffman_literals`, `encode`: 0 each; against 18 for `append_slice`, 14
for `read_bits`, 3 for `store_dynamic_length`, 1 each for `decode` and `parse`).
Not one of those three contracts has ever been discharged as an obligation.

**On `unit-level propositions`.** This value is not a rival to statement lets;
it relocates them. A prop body must still forward `len(deref(d))` into `ile`,
and GRAM-9 admits one mechanism for that. What it uniquely makes true: one
predicate written once and referenced from several declarations — which is the
answer to the triple-restatement finding above — and a predicate writable in
`fn_sig` position, which the DEFERRED work at `[FN-8]:1231` will need. What it
costs: `[FORM-1]:29` forbids admitting both a prop reference and an equivalent
inline let-chain, since they expand to the identical GoalTemplate, so props must
*replace* the in-clause facility; and a `prop` whose body ends in a Bool is a
second declaration spelling of "a named pure total computation" beside `fn`.
The cleanest derivation is not a new declaration kind but a widening of
`[FN-8]:1228`'s admitted callee class to a restricted class of ordinary `fn`
callees with mandatory expansion.

**Discriminator.** Does any contract the language must be able to write forward
a value other than `len(P)`? Everything shipped and called says no, 7 for 7.
Everything that says yes is uncalled and does not parse. The owner scope ruling
is: **is the contract surface for length and bound relations, or must it also
carry computed capacity and overflow-safety preconditions of the
`b64`/`match_copy` kind?**

**Half of the axis is already rule-capped and needs no measurement.** For
`ensures`, `[FN-9]:1309` closes the operand vocabulary, so an `ensures`
clause-local can only ever be a `len(P)` or a degenerate rename. Verified by
probe: `ens_arith.wf` (an arithmetic local) → `[FN-9] InvalidPostconditionRelation`;
the `len` local form → accepted. The `ensures` binding facility has exactly one
admitted non-degenerate use, forced by rule, and both program-shaped `ensures`
blocks that bind use exactly that one.

### 3.2 Axis 3 — CARDINALITY (the axis that changes which programs compile)

**Live values.** Exactly one proposition (today) | several entries,
juxtaposition meaning conjunction.

**Nothing forces either.** The frame's fixed constraint "[FN-8] admits no
composition — a contract's goal is one indivisible tree" is `[FN-8]:1243`: "a
complete `band`, `bor`, `bxor`, or `bnot` tree is one goal that no evidence for
its children ever composes: discharging the whole requires the exact whole
tree". That governs the shape of *one* goal, not how many goals a declaration
carries. Under several entries each entry is one indivisible tree and the
sentence is untouched. `[ENT-2]:2778` is already plural. Neither owner ruling
mentions cardinality.

**What exactly-one makes true.** Every multi-conjunct precondition must be
`band`-rooted. `[ENT-2]:2782` gives a non-comparison root no L0 projection and
`[ENT-4]:2883` states "Deriving the two children of a Boolean operation never
derives its parent, and derivability never decomposes". So a conjunctive
contract is dischargeable **only** when the caller already holds the
byte-identical `+G` — by a dominating branch on a reconstructed `band`, by a
`claim` (a runtime check `[CLM-1]` never elides), or by an identical inherited
S4. The fragment's entire L0 reasoning power is unavailable to every contract
with more than one conjunct, in both directions of strength.

**`band` is a one-way valve, and I measured both sides.** Body side: a
`band`-rooted `requires` establishes both conjuncts inside the callee
(`[ENT-3]:2810-2811` decomposition), so the body loses nothing. Call side:
`band.wf` — a caller establishing `ige(x, 1_u64)` and `ile(y, 9_u64)` in two
nested dominating branches, both facts strictly available — cannot discharge
`band(ige(a,1), ile(b,9))` and is rejected `[FN-8] UndischargedCallRequirement`,
`disposition: Unproved`, `instantiated_goal: "Boolean(And)<...>"`. The identical
two conjuncts written as two single-comparison contracts (`split.wf`) are
accepted. Facts flow out of a conjunction into the body and none flow in to
discharge it.

**The falsifier on a real shipped program.** `tests/programs/percent_decode.wf`
compiles clean, rc=0. Add one second conjunct that the fragment proves at every
program point — `ige(source_length, 0_u64)`, an `[ENT-2]:2788` implicit fact for
a u64 term — squeezed under `band` exactly as cardinality-1 requires. The single
call site in `main` is rejected: `Semantics/Source [FN-8] ...
UndischargedCallRequirement`. **Adding a precondition the checker already knows
to be true breaks the call site.** The rejection is structural, not epistemic.

**What several entries makes true.** A conjunctive contract is discharged
conjunct-by-conjunct against L0 — which is what `[ENT-4]:2888` already does for
an L0 relation ("discharged when every normalized conjunct of R is derivable")
and what `[ENT-6]:2984, :2992-2993` already does for the compiler's own
overflow and division obligations, down to "the least undischarged conjunct" in
the diagnostic and a "normalized conjunct ordinal" in the identity at
`[ENT-6]:3012`. Establishment is unchanged, since `[ENT-3]:2810` already
decomposes `+band`. A two-sided postcondition becomes spellable at all, which
under cardinality 1 it is not (`[FN-9]:1306` requires the expanded root to be
exactly one of `ieq`/`ine`/`ilt`/`ile`/`igt`/`ige`; probe `ens_band.wf` →
`InvalidPostconditionRelation`). And the n−1 invented conjunction names
disappear.

**The writer-facing contract is the only obligation in the language capped at
one conjunct.** Every obligation the compiler forms for itself is a normalized
conjunct list. Cardinality-1 is the Eiffel/JML shape, not a shape re-derived
from this fragment. The plural apparatus is already in the spec text:
`[DIAG-2]:1956` writes the requirement occurrence as "(concrete callee
instance, final-check NodePath, conjunct ordinal 0)" — a checked-program
identity component with exactly one possible value.

**Sub-choice inside "several entries", flagged as a sub-decision not an axis.**
Whether a `band` root stays legal at an entry. After the change a root `band`
states the same body facts and strictly loses at the call site, so it is
dominated surface (FORM-1 pressure, not a FORM-1 violation — the probes show
the two spellings genuinely differ in meaning). `bor`/`bnot` roots are not
dominated: disjunction has no juxtaposition reading. In the program-shaped
population, of 12 `band` lets in contract blocks, 10 are top-level conjunction
that juxtaposition deletes and 2 (`huffman_literals` `zero_case`, `tail_case`)
sit under a `bor` and survive.

**Corpus, program-shaped only.** 9 of 11 blocks are single-conjunct
comparison-rooted. The only 2 `band`-rooted blocks are
`research/experiments/zlib-core-kernels/{match_copy,huffman_literals}.wf`, and
both have zero call sites. So every contract the corpus actually discharges is
single-conjunct, and the `band`-root discharge path has never been exercised by
a program-shaped call. Green is not coverage: the corpus is silent because its
only conjunctive contracts are uncalled, and the one call-site test that exists
for a conjunctive contract is the one I built, and it fails.

**Discriminator.** Must a contract be dischargeable at an ordinary call site
from facts the fragment already holds, or is it acceptable that a multi-conjunct
contract be dischargeable only by the caller rebuilding the callee's predicate
as one Boolean value and branching on it? Owner ruling 1 removed runtime code
from the callee. The question is whether that justifies forcing runtime code
into every caller of every multi-conjunct contract — because the compiler's own
mechanical fix text says exactly that: "establish the complete callee
requirement with one dominating branch, check, or claim before the call", and
probe `branchfix.wf` confirms the branch route is the one that works.

### 3.3 Axis 7 — EXTRA JUDGMENTS (the axis that changes what "accepted" means)

**Live values.** none (today) | refutation of a self-contradictory contract |
redundancy advisory | both.

**The measurement, reproduced independently this session.** Two files differing
in one literal:

```
fn store['r](out: &uniq 'r buffer<u8>, i: own u64, k: own u64) -> own u64 writes('r) requires {
  let lo = ige(k, 1_u64);
  let hi = ile(k, 9_u64);          // vac_contra: ile(k, 0_u64)
  let both = band(lo, hi);
  check both else trap "k range";
} {
  set deref(out)[i] = 7_u8;
  return 0_u64;
}
```

`vac_sat` (satisfiable): rejected, `[OP-4] UndischargedBoundsObligation
{ residual: "i < len(deref(out))" }`, rc=1. `vac_contra` (contradictory): rc=0,
accepted. The parameter `k` appears nowhere in the body. A contract about `k`
discharges a memory-safety bounds obligation on `i`, by ex falso —
`[ENT-4]:2886`, "At a contradictory point every L0 relation and both signs of
every goal in the finite universe are derivable, every obligation, call goal,
and FN-9 selected-return relation is discharged."

The emitted code, `whitefootc --emit-llvm vac_contra.wf`:

```
define internal i64 @wf_store({ ptr, i64 } %v0, i64 %v1, i64 %v2) {
entry:
  %v3 = select i1 true, i8 7, i8 7
  %t0 = extractvalue { ptr, i64 } %v0, 0
  %t1 = getelementptr inbounds i8, ptr %t0, i64 %v1
  store i8 %v3, ptr %t1
```

No bounds check, no trap block, authorized by `[OP-4]:868` ("A discharged
subscript reads or writes with no runtime bounds check in every build mode").
This is not a soundness break — the function is uncallable, and `[PROG-1]`/
`[PROG-2]` close the unit. It is a verification-integrity break plus an
unchecked store in the module.

**The asymmetry.** This defect is already a hard rejection in the `ensures`
half: `[FN-9]:1338` makes complete discharge at every selected return mandatory,
and an impossible `ensures` alone is rejected with `disposition: Refuted`. Add a
contradictory `requires` to the same declaration and the `[FN-9]` rejection
switches off. So value `none` is not the absence of a judgment. It is the
presence of the judgment on one half of a shared form, with the unjudged half
able to disable the judged one.

**What refutation makes true.** Both clauses carry the same lifecycle judgment,
which is what a shared form claims. It adds no fragment power and no new state:
it is one `[ENT-4]:2885` contradiction query on the S4 body-entry state the
checker already builds (`[FN-8]:1258`). Its completeness exactly equals the hole
it closes, because both are the same predicate. Its cost is a rule cost, not a
migration cost: `[ENT-1]:2742` currently reads "The one exception is claim
refutation", and this is a second permanently non-monotone edge, so that
sentence and `[ENT-1]:2748`'s amendment-kind list both change.

**What the redundancy advisory makes true, and why alone it is incoherent.**
`[ENT-4]:2886` makes everything derivable at a contradictory point, so an
advisory-only design classifies an impossible contract as "redundant"
(constrains nothing) when the truth is that it excludes every caller. To avoid
that it must be phrased "when the state is non-contradictory and derives P" —
`[CLM-2]:2700`'s shape — which means the spec computes the contradiction test
and declines to state its answer. That is computed-but-unstated surface, the
same defect the owner is removing from the trap string. Its cheap half is real:
non-rejecting, `[ENT-1]` untouched, mechanism already shipped (the compiler
emits `advisory [CLM-2]: ... is redundant` today).

**No measured demand for the advisory half.** Zero of the 8 `tests/programs`
blocks is redundant. Under R1 ("a construct exists only if it serves P0 or P1")
the advisory is currently unearned. That is honest evidence against urgency, and
weak evidence about the target population, since all 8 were hand-written and the
language is for AI-written code.

**Two corrections to the reachability pass, both of which I checked.** (i)
Refutation does *not* force several entries. Contract contradiction is a
property of the *state* the predicate establishes, not of the predicate's root,
and `[ENT-3]:2810-2811` gives every comparison-rooted member of a `band` tree
its exact L0 relation at establishment — which is exactly why `vac_contra`'s
bounds obligation vanished at cardinality 1. The judgment has full content
today. (ii) The redundancy forcing is only half right: correct for the
whole-proposition reading (a `band` root has no comparison projection, so the
reading is blind), wrong for the member-wise reading over the signed
decomposition set, which has content at cardinality 1.

**A sub-choice the frame does not name.** Which state the judgment queries.
`[FN-8]:1233` erases clause locals, so a comparison whose operand was a computed
clause local has no `[ENT-2]` term after expansion and therefore no comparison
origin. A contradiction one `+wrap` away from the parameters is invisible on the
expanded goal and visible in clause-local scope — and `[ENT-2]:2757(a)` admits
"a requires-clause local" as a term, so a judgment run in the block's own scope
sees strictly more and is sound. Note the dependency: axis 7's detection power
is a function of **axis 1**, not axis 3.

**Discriminator.** Does constitution W3 cover a contract made *unsatisfiable*?
W3 verbatim (`docs/constitution.md:13`): "the writer cannot hack around the
checker — no writer-emittable unsafe or trust; **contracts cannot be weakened to
make a failing body pass**; exhaustiveness cannot be silenced; **checks are
elidable only by proof**; failures trap with reports, never silently. Some AIs
cheat when stuck; cheating is made unrepresentable, not detected later." The
measured channel is one literal wide and it is *strengthening to inconsistency*
— the same failure mode by the opposite move from the one W3 names. R4
(`docs/constitution.md:29`) ranks "check-time rejection with rule-citing
diagnostics" above silence. The scope call is the owner's.

### 3.4 Axis 2 — TERMINAL MARKING

**Live values.** (a) `check` | (b) a different keyword (`holds` in Proposals A,
B, C) | (c) positional, bare `expr ";"` | (d) the whole clause is one
expression.

**(a) vs (b) is decided on one fact and the corpus cannot help.** v0.32's check
dissolution already removed `check_stmt` from the `stmt` alternation
(`[GRAM-4]:202-204`; probe `bodycheck.wf` → a body `check` is
`Parsing/Source [FORM-3]`, i.e. `check` is now only a reserved word in an IDENT
slot). Owner ruling 1 removes the program-start evaluation that `[OP-5]:889` and
`[PROG-3]` are the last readers of. So keeping `check` retains a keyword whose
name describes an action nothing in the language does — and it does not even
avoid the acceptance widening, because `trap` leaves the grammar either way. No
rule forbids this. `holds`, `check`, and `trap` all have zero identifier uses in
the tree, so the corpus does not separate them. `holds` is the only value that
generalizes unchanged to statement position, which matters for the DEFERRED
"loop invariants, ranges" at `[OP-5]:891`.

**(c) positional mints no atom and deletes a production.** The block head
(`requires` / `ensures <selector>`) becomes the sole modality marker and no line
repeats it. `[FORM-2]:56` already lists three keyword-free line-bearing
productions (`expr_stmt`, `field`, `fn_bind`), so nothing in the language
requires a keyword-led proposition line.

**Its forced consequence, and it is real not hypothetical.**
`requires_entry`/`ensures_entry` must narrow from `doc | stmt | check_stmt`,
because `expr_stmt := call ";"` (`[GRAM-4]:214`) and a positional
`prop_entry := expr ";"` both accept an `(IDENT, "(")` start and would compete
at one grammar decision, which `[GRAM-1]:146` forbids. Probe `bareexpr.wf`:
`requires { ige(a, 1_u64); }` → `Resolution/Source [FN-8] ...
RequiresShape(InvalidEntry)`. It reached *resolution*, so those bytes already
have a derivation inside `requires_entry` as `stmt → expr_stmt`. The collision
exists today. The narrowing deletes `[FN-8]:1219`'s stated reason for existing
("syntax formation does not encode the block's semantic subset") and moves those
rejections into the parser. `[FN-8]:1219` states the over-admission as a
deliberate property of the current design, not as a law, so narrowing it is
permitted.

**(d) is where the analysts disagree, and I report the disagreement rather than
average it.** The reachability pass killed the nested-applicative reading via
`[GRAM-9]:300`. The axis-2 analyst objects that this is an exegesis of GRAM-9's
scope, not a rule application — Proposal D's competing reading is that GRAM-9's
enumerated positions are all `expr`/`stmt`/`for_stmt`, and "forwarding a
computed value" is an execution notion that a never-evaluated proposition has no
instance of; owner ruling 1 strengthens that reading rather than weakening it,
since the requires predicate's last executable role is exactly what the ruling
removes. Both agree (d)-nested needs a GRAM-9 amendment. The honest statement is
**settled by a ruling on GRAM-9's scope**, not "killed".

**The axis-2 analyst also found a cleaner kill the pass missed, and it holds.**
If GRAM-9 is language-wide: (d) has no statement position, so axis 1 ∈ {none,
props}; under 1 = none the pass's own forcing requires several entries, which
(d) forbids; so (d) requires axis 1 = props. That is a stronger and cleaner
result than the pass reported, and it is what makes §4's point P7 the only home
for (d).

**The flat reading of (d)** (`requires ile(result, mask);`, one operation over
atoms) violates nothing but is relevance-refuted at 1 of 8 in the
program-shaped population — `tests/programs/raw_deflate.wf:26-27` is the single
block with zero clause-locals and no `len`. The other 7 all need `len(...)`
nested inside the comparison. (This corrects the brief's "all 8 read `len(...)`":
it is 7 of 8.) With the `len`-as-atom widening of §3.1 the flat reading carries
all 8.

**What is not a discriminator.** Proposal A's own defense — that FN-8's
missing-final rejection needs a keyword to name — is false and A concedes it:
`[FN-8]:1220` already reports the `requires_block` node for an empty or all-let
block, uniformly under every value.

**Discriminators, in dependency order.** D1 settles (d) against the rest: is
`[GRAM-9]:298-300` language-wide or scoped to the executable surface? D2 settles
(c) against (a)/(b): must the DEFERRED vocabulary at `[OP-5]:891` attach in
statement position? If yes, positional is dead — a positional proposition in
`stmt` would compete with `expr_stmt` at the one `stmt` decision, `expr_stmt` is
load-bearing in `tests/programs` (bare-call statements in `wfgrep.wf`,
`growable_vec.wf`, `option_slots.wf`, `byte_string.wf`), and a language whose
contract terminal is positional but whose loop-invariant terminal needs a
keyword has two markings for one concept. D3 settles (a) against (b) on the
"does the surface state what is true" criterion, which is an owner call.

### 3.5 Axis 4 — SHARED FORM

**Live values.** share a form (today) | they differ.

**Nothing forces either, and I checked the four candidates.** `[FORM-1]:29` is
per-construct and silent across two constructs. The `ensures_selector` is in the
header, not the entry list — `requires_entry` and `ensures_entry` are
byte-identical at `[GRAM-2]:166` and `:169`, and the frame already labels
today's state "share a form" with the selector present. The predicate-language
restriction coexists with a shared form today. Owner ruling 1 *removes* the one
form-level asymmetry v0.32 has (the trap message is live in `requires` and dead
in `ensures`), and removing a difference is not a rule requiring sameness.

**The one enforced difference, measured.** The byte-identical clause body
`let low = ige(...); let high = ile(...); let both = band(low, high); check both else trap "…";`
is accepted under `requires` and rejected under `ensures` as `[FN-9]
InvalidPostconditionRelation`. That is the whole live difference: whether a
non-comparison Boolean root is admitted. Probe: `req_arith.wf` rc=0 (arbitrary
arithmetic tree admitted in `requires`); `ens_band.wf` and `ens_arith.wf` both
`InvalidPostconditionRelation`. `[FN-8]:1228` admits any tree of non-trapping
total pure operation-table rows; `[FN-9]:1306-1309` closes the vocabulary.

**Half of that is already answered by rule.** `ensures` cannot admit a tree
while S12 publishes into the caller's L0 component, because a Boolean tree has
no L0 projection (`[ENT-2]:2782`) and the caller's goal universe
(`[ENT-2]:2778`) contains only goals the caller itself formed, so a published
tree fact could match nothing. Permanent unless a later version adds a
signed-goal S12 channel — which `[ENT-1]:2741` permits as an addition.

**So axis 4 reduces to axis 3.** Rule the Boolean tree out of `requires` and the
two positions admit the identical predicate language, no form difference
remains, and "share a form" is the only value with content. Rule it in and "they
differ" is true on exactly that one point and on nothing else — on binding,
terminal marking, prose channel, and identity the rules and the corpus give both
positions the same answer.

**The measured writer evidence.** In the program-shaped population, 6 of 8
`requires` are already comparison-rooted; `match_copy.wf` is a pure conjunction
of comparisons that juxtaposition dissolves entirely; only
`huffman_literals.wf:80` needs a root no conjunction split removes — one genuine
disjunction, `bor(band(remainder_zero, input_exact), band(remainder_nonzero, input_with_tail))`.
Neither Boolean-tree contract has a call site. Every `requires` goal actually
discharged at a source call in a real program is comparison-rooted.

**Verified duplication that binds the drafting either way.** `[FN-8]:1220` and
`[FN-9]:1289` state the same structural pass in full, which under
`[META-4]:3260` ("Every normative fact is stated once") is one fact stated
twice. Four drifts have already appeared in that duplication: `[FN-9]:1291`
names `claim` inadmissible and `[FN-8]:1223` does not (the fact lives in a third
rule, `[CLM-2]:2693`); `[FN-8]:1228` orders the admitted row "non-trapping,
total ... `pure`" and `[FN-9]:1292` orders it "pure, total, non-trapping";
`[FN-8]:1229` lists rejected constructs in a separate sentence and `[FN-9]:1291`
folds them in; and the identity-exclusion lists differ by one item —
`[FN-9]:1313` excludes "message bytes" and `[FN-8]:1238` does not, because FN-8's
message is load-bearing at program start, which is exactly what ruling 2
removes. In the compiler the two passes are already one mechanism
(`compiler/src/resolution/engine/admission.rs` is a single loop over both
productions; `compiler/src/semantic/check/ensures.rs` imports `ClauseKind` and
the shared clause builders from `requires.rs`).

**The strongest program-shaped observation for "share".** In
`tests/programs/wfgrep.wf:123-130` and identically `raw_deflate_boundary.wf`,
the two clauses of one function are the *same proposition over two different
values* — the requires expands to `ile(filled, len(deref(destination)))` and the
ensures to `ile(result, len(deref(destination)))` — and they chain: the ensures
of call N is what discharges the requires of call N+1. Two forms would spell one
thought two ways.

### 3.6 Axis 8 (restated) — PROPOSITION IDENTITY

**The frame's wording is a trap and should be repaired.** As stated ("the tree
left after erasing the bindings | the written tree directly"), the axis carries
no choice under axis 1 ∈ {none, statement lets, expression-local binding}: under
`none` the erasure is the identity map and the two values name the same tree
(this is what Proposal D actually is — its draft rule deletes only the expansion
sentence and keeps every datum- and node-identity sentence verbatim, because its
grammar has no clause locals); under the binding values §2.5 forces erasure.

The original analyst concluded from this that axis 8 carries no choice under
**any** axis-1 value and should be struck. That was challenged and the challenge
holds. The three rules in §2.5 are each about a *clause-local* declaration
event. Axis 1's fourth value has no clause-locals: a proposition's declaration
event is a top-level item, visible throughout the closed compilation unit under
the same `[TYPE-6]:380` rule that makes every top-level function signature
visible. The killer does not run there. Three of four values were proved and the
conclusion was quantified over all four.

**Restate the axis so it survives the axis-1 choice:**

> **PROPOSITION IDENTITY:** structural (the template is the tree after every
> binding and definition is erased) | nominal (the template retains the
> reference; identity is the declaration event plus arguments).

**The design that makes it live**, which nobody proposed:

```
prop fits(filled: own u64, destination: &'d buffer<u8>)
  = ile(filled, len(deref(destination)));

fn append_slice['d,'m](destination: &uniq 'd buffer<u8>, filled: own u64,
                       text: own slice<'m,u8>) -> own u64 …
  requires fits(filled: filled, destination: destination);
{ … }
```

The `[FN-8]` GoalTemplate is `PropRef(<prop declaration event>, [formal#1,
formal#0])` — not the inlined body. Node identity is (prop declaration event,
concrete type/const substitution, written argument order), exactly parallel to
`[FN-8]:1237`. Goal equality stays exact tree equality (`[ENT-2]:2772, :2777`);
a PropRef is one indivisible tree and `[FN-8]:1243`'s no-composition sentence is
untouched. A prop application becomes a direct goal-origin shape
(`[ENT-3]:2801`), so a caller establishes the exact goal by writing
`if fits(filled: f, destination: d) { … }`. And *one definitional unfolding*
supplies comparison origin and L0 projection when the substituted body root is a
comparison over admitted terms.

**It needs three spec additions, all monotone** (`[ENT-1]:2741` permits adding
fact sources and closure rules): `[ENT-3]:2801`'s goal-origin shapes,
`[ENT-3]:2797-2799`'s comparison-origin enumeration, and `[FN-8]:1237`'s node
vocabulary. Note what it does **not** need: no `[ENT-2]:2759` change, because its
leaves are parameters, consts, literals, and `len(P)`; and **no GRAM-9
amendment**, because a prop body can itself be a flat ANF let-chain — the
nesting sublanguage Proposal D needs is optional here, not required. Corpus
compatibility: all 8 program-shaped roots are one comparison over admitted
terms, so all 8 keep both their caller discharge and their S4 relation.

**The two identity values disagree on real observables**, measured: (i) a caller
that reproduces the callee's tree with different binder spellings is accepted
under structural identity and would be rejected under nominal (the caller must
name the prop); (ii) two props with byte-identical bodies are one goal under
structural and two under nominal; (iii) the `[FN-8]` rejection can print the
writer's own `fits(filled, destination)` rather than an expanded tree over
`BindingId(0)`/`BindingId(1)`, structurally rather than as a diagnostics patch.
Two different accepted sets, both coherent, both constraint-legal. That is a
decision.

**The precedent that was not weighed.** `[FN-3]:1047-1048`: "a nominal instance
equals only an instance of the same nominal declaration identity ... Layout
equality, member equality, spelling equality across distinct declarations, and
implicit conversion never establish type identity." The kernel has already taken
the nominal side of this axis for types. Structural goal identity is the
outlier, and it survives today only because a contract has no declaration site
to be nominal about. The moment axis 1 supplies one, the house style pulls the
other way.

**What is determined.** Structural for `ensures` under every axis-1 value
(§2.6). Structural under axis 1 ∈ {none — where it is vacuous — statement lets,
expression-local binding} (§2.5). **Live for `requires` under axis 1 =
unit-level propositions.** Note the robustness of the frame repair: if one
insists a prop is not a "binding" and so there is nothing for erasure to act on,
then striking the axis leaves the frame with no slot at all for the
inline-or-not decision. Either reading forbids the strike.

**One genuinely open item that belongs elsewhere.** The `[FN-8]` rejection today
renders the *expanded* tree over parameter ordinals, so a writer is shown a form
they did not write. `[FN-8]:1239` already permits retaining the source occurrence
for diagnostics ("identify the requirement occurrence for diagnostics and checked
metadata but are not part of predicate equality"), so better diagnostics cost
nothing and change no goal identity. That is a compiler defect, not an axis.

**And one thing that looks like a live alternative and is not:** keeping the
written tree in the compiler and expanding at instantiation. `[ENT-2]:2772` fixes
the *concrete* goal as a tree, and the concrete goal is what discharge, S4,
`[DIAG-1]` and `[DIAG-2]` all read. Identical concrete goal, identical
observables. Implementation schedule, not a language value.

### 3.7 Axis 5 — PROSE CHANNEL

**Live values.** none | a `doc` entry inside the block. (The trap STRING is
dead, §2.3.)

**This axis is settled by axis 3, not by preference.** If the contract stays one
proposition, a block-level `doc` reaches exactly what the declaration's `doc`
already reaches — one sentence about one function — so the second slot buys a
label, not a capability, and `none` wins on parsimony. If the contract becomes
several juxtaposed entries, a `doc` can sit next to the entry it explains and
carries something no existing channel can: *which conjunct* a sentence is about.

**The demand is real, unmet, and bound-level.**
`research/experiments/zlib-core-kernels/match_copy.wf:1-29` encodes 32768 (the
DEFLATE window), 3 and 258 (the match-length range), and a hand-split 128-bit
product overflow test; `huffman_literals.wf:74` requires
`ile(symbol_count, 16397105843297379213_u64)` — the largest n for which
n + (n>>3) does not overflow u64. Neither the trap message nor the body `doc`
(`match_copy.wf:31`, `huffman_literals.wf:87`) says why any of those numbers is
that number. In a language whose premise is human approval of AI-written code,
the reviewer re-derives 16397105843297379213 unaided.

**Writer behavior with the channel that already exists.** Program-shaped
population, roughly 80 `doc` strings: exactly one mentions a caller-facing
constraint (`wfgrep.wf:299`, about aliasing) and zero state or explain a
precondition. Of the 9 contract-bearing functions, 5 carry a body `doc` and not
one of those docs explains its contract; the other 4 carry no doc at all. Two
readings survive and this data cannot separate them: writers have nothing to say
about contracts, or the available slot is the wrong one.

**The `doc` value costs no grammar and no formatting rule.** `[GRAM-2]:166`
and `:169` already derive `doc` as an entry; `[FORM-2]:56` already lists `doc` as
line-bearing and `:59` lists both blocks as block-bearing, so it renders at depth
plus one with no new sentence; `[FN-8]:1219` states the admission is deliberate.
Probe `docentry.wf` (a doc as the first requires entry) → `Resolution/Source
[FN-8] ... RequiresShape(InvalidEntry)` — a *semantic* rejection, not a grammar
non-derivation. The whole value is one structural-pass sentence in FN-8 and FN-9
plus adding "doc bytes" to `[FN-8]:1238`'s exclusion list.

**FORM-1 does not decide this**, contrary to Proposal D. `doc` already occupies
five positions with one spelling (`[GRAM-2]:156, :158, :163, :170, :174`);
position is not spelling. Nor does admitting it commit the language to prose
inside statement blocks: `[FN-8]:1273` and `[FN-9]:1286` draw the line —
requires/ensures blocks are declaration surface, while loop, if, region and arm
blocks are executable statements.

**Secondary and independent question.** Must the rationale be machine-retained
review data? `[DIAG-2]:1962` retains a CLM-1 justification STRING; `[DIAG-2]`
retains no `doc`, and a whole-spec search finds exactly two prose rules
(`[FORM-4]:78`, `[CLM-1]:2692`). If the owner wants a toolchain that can
enumerate every contract with its rationale, `doc` does not satisfy that without
a new retention sentence, and the axis reopens onto a justification-shaped
value the frame does not list.

**A fourth value I considered and rejected on mechanism**, flagged as an
addition: a CLM-1-shaped mandatory justification STRING on the terminal.
`[CLM-1]:2692` gives a claim a justification because a claim asserts what the
compiler cannot prove; under ruling 1 a contract asserts nothing and is
discharged at every call, so there is no belief to justify. It reduces to the
`doc` value made mandatory, and mandatory is refuted by the measurement above —
a reader-less mandatory string degenerates (in the test-shaped population, 16 of
16 `ensures` messages contain the slot's own name: "postcondition", "relay
postcondition", "bounded postcondition", …).

**Cross-axis gate.** Axis 2 = (d) forbids both surviving values — an expression
clause has no entry slot — and collapses axis 5 to `none`.

### 3.8 Axis 6 — IDENTITY

**Live values.** anonymous (today) | named as compile-time review data, in
`[CLM-1]:2689`'s non-declaration carrier class.

**Two sub-readings are dead by rule.** *Named-and-referenced* (a name a caller
or conformance could mention to discharge): `[ENT-1]:2735` closes the fact-source
list, and discharge by naming is in none of those, so a referencing name would
be a second discharge path. *Named-as-identity*: `[ENT-6]:3014` already fixes the
requirement occurrence identity as `(concrete function instance, final-check
NodePath, 0)` and `:3015` adds that these do not participate in goal equality, so
a name can never distinguish two occurrences the tuple confuses. It is provably
redundant as identity, not merely unnecessary.

**What survives is a name that addresses a human reader and nothing else.** And
that is exactly what `[DIAG-1]:1912` already ruled against in the only
diagnostic where the question arises: the FN-8 rejection payload "contains the
concrete callee instance, the requirement occurrence's final-check NodePath, the
complete instantiated typed goal, and exactly one disposition ... **it does not
select a predicate by clause-local spelling**." A writer label was considered and
refused at the finest granularity available.

**Ruling 1 removes the precedent's justification.** The claim name — the
language's one free-choice IDENT with no source referent — is redeemed by
mandatory observable output (`[DIAG-3]:2049`, "`message` is the claim's exact
IDENT spelling"). A contract that generates no runtime code in any position can
never reach that reader. Every writer-chosen IDENT in v0.32 is read by a source
reference, a closed-table check, or mandatory observable output. After ruling 1 a
contract name is in none of the three.

**Measured writer behavior.** Writers name *propositions*, not contracts. 7 of
the 11 program-shaped blocks spend an optional `let` purely to name the
proposition (`sufficient`, `admitted` ×2, `enough_literals`, `enough_output`,
plus 17 Bool names in `match_copy.wf` and 11 in `huffman_literals.wf` — 33 in
all), and in each of the 5 `tests/programs` `requires` blocks the binding was not
forced by GRAM-9. Every one of the 33 is erased by `[FN-8]:1238`. Zero of the 11
shows any demand for a block-level name. And the block-level label channel that
already exists — the trap STRING — carries nothing the rendered proposition does
not (see the table at the top of this document); three of eleven simply restate
the function name as a topic, which is precisely what a name is.

**Discriminator.** Enumerate every post-ruling consumer of a contract's identity
— the FN-8 rejection payload, `[DIAG-2]:1956` retention, `[ENT-6]:3012-3016`'s
occurrence identity and its PRV-2/PRV-3 bridge, and any axis-7 advisory. None
needs content it cannot state from `(concrete instance, final-check NodePath,
conjunct ordinal, rendered instantiated goal)`. So the only ground left is
addressing a human reader, and the owner rules on exactly that, against
constitution R1 (`docs/constitution.md:26`, "Earn your place. A construct exists
only if it serves P0 or P1. Serving human authorship counts for nothing") and R5
(":30", "Readability is a non-goal; auditability of the trusted base is the
deliberate exception" — which does not reach here, since a contract is checked
rather than trusted). Note W1 (":11") forecloses settling it by model runs: "a
model score is neither evidence for nor against W1."

---

## 4. The reachable points

Dependencies that constrain the enumeration, all verified:

- axis 1 = none **forces** axis 3 = several. With no binder,
  `check band(ige(a, 1_u64), ile(b, 9_u64))` puts two calls in atom positions;
  probe `conj.wf` → `Parsing/Source [GRAM-9]`. Conjunction is inexpressible in
  one entry, so juxtaposition becomes mandatory.
- axis 1 = none **collapses** axis 8; axis 1 = lets **forces** axis 8 =
  structural; axis 1 = props **frees** axis 8 for `requires` only.
- axis 2 = (d) **forces** axis 3 = one and axis 5 = none, and **forbids** axis
  1 = lets (no statement position). With the first dependency this leaves
  axis 2 = (d) reachable only under **axis 1 = props**.
- axis 2 = positional **requires** narrowing `requires_entry` (GRAM-1 collision,
  probe `bareexpr.wf`).
- axis 1 = none is **relevance-refuted at 7 of 8** unless GRAM-9's `atom`
  production admits `len(P)`.

Axes 4, 5, 6, 7 are near-orthogonal riders on the structural skeleton below,
with the two couplings already noted (axis 4 reduces to axis 3; axis 5 is
decided by axis 3 and gated off by axis 2 = (d)).

| # | axis 1 | axis 2 | axis 3 | axis 8 | who proposed it |
|---|---|---|---|---|---|
| **P1** | statement lets | keyword | one | structural | **Proposal A** (`holds`); **B** and **C** land here too after abandoning their stances. The `check`-retaining variant is unwritten. |
| **P2** | statement lets | keyword | **several** | structural | **nobody** |
| **P3** | none + `len` atom | **positional** | several | n/a | **nobody** |
| **P4** | none + `len` atom | keyword | several | n/a | **nobody** |
| **P5** | none, no atom widening | any block form | several | n/a | reachable; carries 1 of 8 program-shaped blocks (`raw_deflate.wf:26` only), since no contract can mention a length |
| **P6** | **props** | one expression | one | structural | **nobody** (Proposal D's *shape*, but D uses a nested grammar, not a declaration) |
| **P7** | **props** | one expression | one | **nominal** | **nobody** — the only point where axis 8 is a choice |
| **P8** | **props** | keyword or positional | several | either | **nobody** |
| **P9** | none | (d) nested applicative grammar | one | n/a (collapse) | **Proposal D** — lives or dies on the GRAM-9 scope ruling |

**The headline.** All four existing proposals sit on two skeletons: P1 (A, B, C)
and P9 (D). Every one of them keeps **cardinality 1**. No proposal mentions
juxtaposition, and no proposal mentions unit-level propositions — I searched all
four for both terms and for "several/multiple/more than one entry/proposition";
the only hit is Proposal B's conditional aside that "when a contract block admits
more than one proposition, revisit the name", which proposes nothing.

So the two structural changes with the largest measured consequences — the one
that fixes the call-site discharge asymmetry (§3.2) and the one that fixes the
triple-restatement and makes axis 8 a real choice (§3.1, §3.6) — are both
unproposed. The four proposals explored the *terminal marking* axis thoroughly
and the *cardinality* and *binding-location* axes not at all.

**Three of the unproposed points deserve naming.**

**P2 is the minimal delta against the largest measured defect.** Change nothing
but cardinality: keep the block, keep the lets, keep a keyword, admit several
terminal entries. The structural passes at `[FN-8]:1220` and `[FN-9]:1289`
change "exactly one final" to "one or more"; the requirement occurrence becomes
plural where `[DIAG-2]:1956` already writes "conjunct ordinal 0"; and FN-9's
publication machinery is already written per-relation ("per-relation `M(c,q)`",
"it does not suppress a relation at another call"), pinned to one by the
cardinality rule alone. It fixes the `percent_decode` falsifier and leaves every
other axis untouched. It is the cheapest point on the board and no proposal
contains it.

**P3 is the smallest surface that carries the whole shipped corpus.** No binder,
no keyword, several bare propositions, `len(P)` admitted as an atom
language-wide:

```
requires {
  ile(filled, len(deref(destination)));
}
```

All 8 `tests/programs` blocks fit it, because every operand they use is already
in `[FN-9]:1307`'s admitted vocabulary. It deletes `check_stmt`, the trap tail,
the clause-local scope rules at `[TYPE-6]:394`, the alpha expansion at
`[FN-8]:1233` and `[FN-9]:1305`, the `[FN-8]:1220`/`[FN-9]:1289` structural-pass
duplication and its four drifts, and the FORM-1 many-to-one of §3.1 — and it
restores conjunct-wise discharge. What it costs is the arithmetic contracts of
`b64`/`match_copy`, none of which parses or is called today, and one GRAM-9
amendment.

**P7 is the only point where the frame's axis 8 is a live decision**, and it is
also the only point that answers the triple-restatement finding and the
DEFERRED `fn_sig` work at `[FN-8]:1231`. It costs three monotone spec additions
and a new declaration kind (or a widening of `[FN-8]:1228`'s admitted callee
class).

---

## 5. What would settle each open axis

**Axis 1 — binding facility.** *Owner scope ruling.* Is the contract surface for
length and bound relations, or must it carry computed capacity and
overflow-safety preconditions of the `b64`/`match_copy` kind? Everything shipped
and called answers the first, 7 for 7. *Cheap falsifier before ruling:* port
`match_copy.wf`'s `requires` to v0.32 spelling and write one caller that
discharges it. `[ENT-2]:2777` makes goal equality exact tree equality and
`[ENT-3]:2802` gives a user-function call no goal origin, so the caller must
reproduce the whole tree byte-identically. If it cannot, the "yes" evidence
collapses to zero and the axis settles at `none` + `len`-as-atom.

**Axis 3 — cardinality.** *Owner ruling, factual half already settled.* Run: take
any `tests/programs` contract, add one conjunct the fragment proves everywhere
(`ige(x, 0_u64)` on a u64 term), squeeze it under `band`. I ran it on
`percent_decode.wf`: clean → `[FN-8] UndischargedCallRequirement`. If the owner
accepts that adding a known-true precondition may break every call site, keep
cardinality 1. If not, several entries is the only other value. Sub-decision if
several: does a `band` root stay legal at an entry?

**Axis 7 — extra judgments.** *Owner ruling on W3's scope.* Does "cheat-proofness"
cover a contract made *unsatisfiable*, given that the identical defect in
`ensures` is already a hard `[FN-9]` rejection and that a contradictory
`requires` switches that rejection off? The reproduction is `vac_sat.wf` vs
`vac_contra.wf`, one literal apart. Yes → refutation as a hard declaration-site
rejection, and `[ENT-1]:2742`'s exception list grows from one entry to two. No →
the language accepts that a body under an impossible contract is accepted
without being verified. The redundancy half is a separate, subordinate question
with a measured answer of zero occurrences in the program-shaped population; do
not bundle it into the same ruling. Sub-decision either way: does the judgment
query the expanded goal or the block's own clause-local scope? The second sees
strictly more and is sound.

**Axis 2 — terminal marking.** Three rulings in dependency order. **D1:** is
`[GRAM-9]:298-300` language-wide or scoped to the executable surface? This is a
spec ruling and it is the only thing keeping (d) alive; it also decides whether
the axis-1-none → axis-3-several forcing holds. **D2:** must the DEFERRED
"loop invariants, ranges" at `[OP-5]:891` attach in statement position or as a
clause block on the loop? Statement position kills positional. **D3:** after
ruling 1, is there any construct spelled `check` that performs a check at
runtime? No — so keeping `check` is decided on whether the surface must state
what is true. The corpus cannot decide any of the three: zero identifier
collisions for `holds`, `check`, and `trap` alike.

**Axis 4 — shared form.** No separate ruling needed. Answer axis 3 and the
Boolean-tree question, and axis 4 falls out: rule the tree out of `requires` and
the two positions admit the identical predicate language and "share" is the only
value with content; rule it in and "differ" is true on that one point and
nothing else. The one thing that binds either way is `[META-4]:3260`: the shared
facts must be stated once, and today they are not.

**Axis 8 — proposition identity.** Only live if axis 1 = props. Then it is a
straight nominal-versus-structural call, and `[FN-3]:1047-1048` is the house
precedent for nominal. The observation that separates them: under structural
identity a caller discharges by reproducing the callee's tree; under nominal it
must name the prop. Decide which of those two accepted sets you want. Note
`ensures` unfolds either way (§2.6), so choosing nominal accepts an asymmetry
axis 4 then has to state.

**Axis 5 — prose channel.** Decided by axis 3 (see §3.7), with one independent
question the owner should answer separately: must a contract's rationale be
machine-retained review data (`[DIAG-2]`-style), or are canonical source bytes
enough? If retained, neither surviving value satisfies it and the axis reopens.

**Axis 6 — identity.** *Owner ruling, one sentence:* when a contract entry is
unproved, refuted, or advised redundant, does the diagnostic address the writer
with the writer's label or with the rendered instantiated proposition?
`[DIAG-1]:1912` has already ruled for the rendered proposition once. Ruling
"rendered proposition" closes the axis permanently — no later axis-3 or axis-7
choice reopens it, since `[ENT-6]:3012` already supplies a conjunct ordinal and
the payload renders the failing conjunct's whole goal. Ruling "writer's label"
admits the value in the CLM-1 carrier class only, with the measured expectation
that writers will use it the way they used the trap message.

---

## Appendix — probes run for this document

All against the prebuilt `compiler/target/debug/whitefootc` at the v0.32 digest
above. Sources at `/Users/bytedance/do_not_scan/syn/`; the repository is
unmodified apart from this file (`git status --porcelain` clean before the
commit).

| probe | result |
|---|---|
| `vac_sat.wf` / `vac_contra.wf` | one literal apart; rc=1 `[OP-4] UndischargedBoundsObligation` vs rc=0 accepted with `getelementptr inbounds` + `store` and no bounds check |
| `pd_base.wf` / `pd_conj.wf` | shipped `percent_decode.wf` rc=0; + one implicit-fact conjunct under `band` → `[FN-8] UndischargedCallRequirement` |
| `split.wf` / `band.wf` | two single-comparison contracts accepted; same two conjuncts under one `band`, caller holding both facts, rejected `Unproved`, `instantiated_goal: "Boolean(And)<…>"` |
| `s4_yes.wf` / `s4_no.wf` | projecting root discharges the body's bounds obligation; non-projecting root → `[OP-4]` residual `i < len(deref(out))` |
| `nested.wf` | `check ile(filled, len(deref(destination)))` → `Parsing/Source [GRAM-9]` |
| `conj.wf` | `check band(ige(a,1_u64), ile(b,9_u64))` with no binder → `Parsing/Source [GRAM-9]` |
| `req_arith.wf` | four-let `len`→`*wrap`→`+wrap`→`ile` chain in `requires` → rc=0 |
| `ens_band.wf`, `ens_arith.wf` | `[FN-9] InvalidPostconditionRelation` |
| `docentry.wf`, `bareexpr.wf` | both `Resolution/Source [FN-8] RequiresShape(InvalidEntry)` — both *parsed* |
| `bodycheck.wf` | body `check` → `Parsing/Source [FORM-3]`; check dissolution complete |
| `research/experiments/{b64,huffman_literals,match_copy}.wf` | all three `Parsing/Source [GRAM-4]` under v0.32 — stale, not current demand |

Corpus counts re-measured: `tests/programs` 5 requires + 3 ensures in 6 files;
`tests/conformance/cases` 42 + 16 in 44; `tests/codegen/cases` 37 + 0 in 37;
`research` 14 + 5 in 16 files, of which only 3 blocks in 3 files (under
`research/experiments`) are program-shaped and 16 blocks in 13 files are probe
files under `research/investigations`. Call
sites in `tests/programs`: `append_slice` 18, `read_bits` 14,
`store_dynamic_length` 3, `decode` 1, `parse` 1; in `research/experiments`,
`inflate_match_copy` 0, `inflate_huffman_literals` 0, `encode` 0.

No migration count appears anywhere in this document.
