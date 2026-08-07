# Semantics batch — vNEXT candidate (ENT-5 loop-rule fix + taint gate)

Status: CANDIDATE, DRAFT (2026-08-07). Non-authoritative. This document is the
complete semantic delta against the exact text of the active
`spec/kernel-spec-v0.22.md` (installed 8f91ede; SHA-256
`b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8`; roadmap
revision 18). It authorizes nothing: activation runs the `docs/WORKFLOW.md`
loop.

Two items, both semantic, both grammar-free:

- **Item A — the [ENT-5] loop-rule fix.** The loop rule's kill scan considers
  only kill events on edges that can reach a later iteration head of the same
  loop. Owner-explained, pending final word; drafted here. Evidence:
  `research/investigations/obligation-discharge/ACCEPTANCE.md` (2026-08-07),
  which isolates this rule as the dominant cause of the deflate divergence
  (5/29 sites proven against 17/30 predicted) with the minimal witness `D1h`
  / `D1i` — one `return` moved across the loop boundary, nothing else changed.
- **Item B — the taint gate, subject-position form.** `DOSSIER.md` §8 item 5,
  priority-advanced by the owner on the acceptance evidence. Adds a derived,
  signature-visible provenance property and one gate: an obligation whose
  constrained subject carries external provenance may not be discharged by a
  writer-authored runtime check.

**Numbering.** The file name reserves v0.24 on the assumption that the FLOOR-5
spelling batch (`spelling-relief-candidate.md`, planned task 0036) takes v0.23.
If that batch is deferred or reordered this candidate takes the next free
number instead; per `docs/WORKFLOW.md` step 2 an occupied candidate path stops
for an owner choice rather than being silently merged or skipped. Ordering
dependency on that batch is stated in §8 (O9); it is textual, not semantic, and
the one shared rule ([ENT-5]) is edited at two non-overlapping sites.

**Three honest statements up front, before the detail:**

1. Item A only widens acceptance under checker strengthening, with one
   enumerated edge already in [ENT-1]: a pre-loop fact that now survives can
   newly *refute* a claim under [CLM-2]. That is the lifecycle's existing
   deliberate non-monotone edge, and it rejects only a program proven to trap
   on every execution reaching that claim. §4.4 states it exactly.
2. Item B, as drafted, **rejects zero claims in the current corpus** (144
   claims; 15 of them in the 5 programs that have a boundary at all). It does
   **not** by itself convert the three canonical-Huffman sites that motivated
   it into `Err` branches, because the deflate corpus's input is built by
   `make_dynamic_input()` and never crosses a boundary, so no value in that
   program carries external provenance. §7 measures this.
3. Item B has **four named bypasses** — the [FN-8] `requires` prologue, the
   trapping arithmetic modes, bound-position external values, and a claim
   supporting no obligation. Each is stated in §5.4 with a witness. The gate
   raises the cost of the "bad input crashes the program" defect and makes its
   common shape a compile error; it does not eliminate the defect. An owner
   approving this batch should approve it on that basis.

## 1. Proposed version-header paragraph

> Status: REVIEW CANDIDATE vNEXT (2026-08-07; semantics batch: the [ENT-5]
> loop-rule scope-exit fix and the subject-position taint gate). Restates
> [ENT-5]'s loop rule so that the kill scan at a loop head considers exactly
> the kill events an execution can carry into a later iteration head of the
> same loop: an event inside the body is scanned when some path of the
> conservative structural graph [FN-1] leads from its edge back to that
> loop's body entry without leaving the body, and the events reachable only
> through a `break` naming that loop or an enclosing one, a `return`, or a
> `propagate` error edge are not scanned, because no later iteration head
> observes them — today a single `return` anywhere in a body discards every
> pre-loop fact at the head, including `requires` axioms and allocation-length
> equalities that no execution can invalidate. Adds a derived provenance
> judgment and one gate: a value originating at a [SYS-2] system operation, at
> a kind-declaring entry's labelled standard input [FN-7], or at a gated-family
> member [GATE-1, LEDGER-1] carries external provenance; provenance propagates
> by dataflow through operands, place roots, and calls, never through control
> flow, so a `match` on external bytes yields program-chosen values; `len` and
> the four transfer counts whose [SYS-2] contract bounds them by a
> program-supplied argument are internal; and an obligation [ENT-6] whose
> constrained subject term is externally provenanced is discharged only in the
> unasserted fact state — the [ENT-4] closure with every S2 `check` and S3
> `claim` fact omitted — so a dominating branch discharges it and a
> writer-authored runtime check does not. Provenance is a derived
> signature-visible column, never written source: each concrete function
> derives its result provenance, its `&uniq` write provenance, and the
> parameter positions its body's gated obligations require to be internal, and
> a call passing an externally-provenanced actual to such a position is
> rejected at the call site with the caller's own provenance chain, under one
> least fixed point over the closed call graph [PROG-1] — Whitefoot has no
> indirect calls, so the fixed point exists and is unique. Specification delta:
> numbered rules +3/-0 ([PRV-1] provenance, [PRV-2] the derived signature
> column, [PRV-3] the gate); eight existing rules modified at ten
> verbatim-anchored modification sites (a site is one contiguous
> verbatim-anchored replacement): FN-1 (the signature-content sentence gains
> the derived provenance column), OP-4 (the mechanical-fix sentence names the
> gate), SYS-2 (one provenance classification per operation result and per
> `&uniq` destination, as declaration data), CLM-1 (the deferral sentence is
> replaced by the live gate reference), ENT-1 (two sites: the judgment list
> gains the provenance judgment and the gate; the implementation-agreement
> sentence covers provenance), ENT-3 (the "nothing else is a fact" sentence
> stops denying a taint judgment and states that provenance is not a fact),
> ENT-5 (one site: the loop rule), ENT-6 (two sites: the discharge condition
> routes through [PRV-3]; the mechanical-fix sentence). Section 18's heading
> gains provenance. Tokens +0/-0; terminal spellings +0/-0; grammar
> productions +0/-0; operation-table rows +0/-0; source constructs +0/-0;
> sections +0. No rule carries an exception clause [META-3]: the provenance
> classification of each system result is [SYS-2] declaration data and the
> propagation judgment is a total case analysis over expression forms. The
> accepted-program set widens by every program whose loop-head facts survive
> under the corrected scan — measured on the deflate unit as the dominant
> cause of a 12-site discharge miss — and narrows by exactly one class: a
> program in which an externally-provenanced value indexes storage and only a
> `check` or `claim` bounds it, measured at zero sites across both corpora
> (144 claims, 2026-08-07). Selection ground: `ACCEPTANCE.md` (2026-08-07) for
> Item A, with the `D1h`/`D1i` witness; `DOSSIER.md` §2.5 as amended by
> `PROBE-W1.md` and `PROBE-TAINT.md` for Item B. These bytes are
> non-authoritative until the derived-material review, full-document hash,
> exact owner approval, and active-target installation complete.

## 2. Grammar delta

None. Neither item adds, removes, or reshapes a production, a terminal
predicate, a token form, an operation-table row, or a source construct. No
modification site in §4 or §5 lies inside a fenced grammar block, and the three
new rules are prose rules in §18. A full-document assembly of this candidate
reproduces v0.22's §3 bytes exactly.

Provenance is deliberately **derived, not written**. The alternative — a
written provenance annotation on parameters, in the shape of an effect
category — would be a grammar change, would put a trust-bearing declaration in
the untrusted writer's hands, and would need a variance rule the moment
indirect calls exist. It is recorded as a rejected alternative in §8 (O5).

## 3. Native grammar-verifier evidence

Baseline, run 2026-08-07 against the active spec at 8f91ede:

```sh
cargo run -q --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  spec/kernel-spec-v0.22.md
# -> grammar-preserving candidate verified by the active compiler:
#    65 productions, 75 decisions, 76 terminal predicates
# exit code 0
```

A full-document assembly of this candidate must reproduce those three counts
and exit code exactly; any other result is a drafting defect in this file, not
a language change. The assembly check is recorded in §9.

## 4. Item A — the [ENT-5] loop rule

### 4.1 The modification (complete replacement delta, verbatim anchor)

**[ENT-5]** One site: the rule's final paragraph. Verbatim anchor —

> Loops carry no induction in this version: the fact state at the head of each
> iteration of `loop @l { … }` is the state before the loop minus every fact
> having a support member that any kill event (a)–(d) occurring anywhere inside
> the loop body, at any nesting depth, may kill. The surviving facts hold at
> every iteration head; establishment and kills then proceed ordinarily within
> the iteration, and no fact established inside an iteration survives to the
> next iteration's head. Loop induction is a later version's
> [ENT-1]-monotone extension.

becomes —

> Loops carry no induction in this version: the fact state at the head of each
> iteration of `loop @l { … }` is the state before the loop minus every fact
> having a support member that a continuing kill event of `@l` may kill. A kill
> event (a)–(d) placed inside `@l`'s body, at any nesting depth, is continuing
> for `@l` exactly when some path of the conservative structural normal-control
> graph [FN-1] leads from the edge carrying that event to `@l`'s body entry
> without leaving `@l`'s body — that is, exactly when an execution taking that
> edge can reach a later iteration head of the same loop. Every other kill
> event inside the body is not continuing and is not scanned: an event on or
> reachable only through a `break` edge naming `@l` or any enclosing loop, a
> `return` edge, or a `propagate` error edge leaves `@l` for the loop's
> continuation or the function-return sink [FN-1, ERR-3], and no iteration head
> of `@l` is reached from it without first re-entering `@l` from outside, where
> the enclosing flow supplies the state. A kill inside a nested `loop @m` whose
> continuation lies inside `@l`'s body is continuing for `@l`, including the
> kills carried on `@m`'s own `break` edges, because `@l`'s body entry is
> reached from `@m`'s continuation without leaving `@l`. The surviving facts
> hold at every iteration head; establishment and kills then proceed ordinarily
> within the iteration, and no fact established inside an iteration survives to
> the next iteration's head. A fact a non-continuing edge kills is still
> removed on that edge: the continuation join above takes each `break` edge
> after that edge's scope-exit kills, and an edge to the function-return sink
> reaches no queried program point, so narrowing this scan opens no path on
> which a dead fact is read. Loop induction is a later version's
> [ENT-1]-monotone extension.

No other clause changes. The three other places in [ENT-3] and [ENT-5] that
mention kill events — the comparison-origin clause (b), S7's checked-arithmetic
origin, and S10's boundary-count origin — each quantify over *paths from an
initializer to a use*. A `return` or `propagate` error edge lies on no such
path, because nothing follows it, so none of the three is affected by this
change and none is restated.

### 4.2 Soundness

**Claim.** If a fact `F` holds in the state before `loop @l` and no continuing
kill event of `@l` may kill a support member of `F`, then `F` holds at every
iteration head of `@l`.

**Proof.** By induction on the iteration index.

*Base.* Iteration 1's head state is the state before the loop, in which `F`
holds by hypothesis.

*Step.* Suppose `F` holds at iteration `k`'s head. Take any execution that
reaches iteration `k+1`'s head. On the conservative structural graph its
control traverses a path `π` from `@l`'s body entry to the loop-body normal
exit and thence to the body entry again, and every edge of `π` lies inside
`@l`'s body — a path that left the body would reach `@l`'s continuation or the
function-return sink, and neither reaches `@l`'s body entry without a fresh
entry to the whole `loop_stmt` from the enclosing flow, which is a different
program point whose state the enclosing flow (and, when `@l` is nested, the
enclosing loop's own instance of this rule) supplies. Every edge `e` of `π`
therefore has `@l`'s body entry reachable from it along the remainder of `π`
without leaving the body, so every kill event on `π` is continuing for `@l` by
the definition above. By hypothesis no such event kills a support member of
`F`, and establishment along `π` only adds facts ([ENT-3] sources are additive
and [ENT-4] closure is monotone), so `F` survives `π` and holds at iteration
`k+1`'s head. ∎

The contrapositive is the property that matters: the scan omits an event
exactly when no execution can both take that event's edge and be observed at a
later head of the same loop. Nothing survives that could be false where it is
read. In particular the omission is not "returns are ignored" — a `return`
inside a loop still kills on its own edge, and that edge reaches the
function-return sink, which is not a queried point.

The rule is strictly stronger than the narrower reading "exclude the `return`,
`break`, and `propagate`-error edges themselves". Consider
`loop @l { set p = e; return v; }`: the `set`'s kill (a) is on an ordinary
statement edge, so the narrow reading scans it, while the reachability
formulation does not, because `@l`'s body entry is not reachable from that edge
without leaving the body. Both are sound by the proof above; the reachability
form is the one that states the actual reason, needs no enumeration of
statement kinds, and generalizes when a later version adds a control form. The
choice between the two is ruled open at §8 (O1) rather than taken silently,
because it is a widening beyond the owner's stated wording.

### 4.3 The break edge opens no hole

The concern the change must answer: `break @l` carries scope-exit kills, and if
the loop-head scan stops looking at them, some fact might survive into the
loop's continuation after the binding it depends on has died.

It does not, because the continuation state is computed by a different clause
that this delta does not touch. [ENT-5]'s join paragraph already says: "The
continuation of a `loop_stmt` is the join over the states on its `break` edges,
each likewise taken after its scope-exit kills and closed." Every `break` edge
contributes its own post-kill state, the join keeps only what all of them hold,
and a loop with no `break` naming its label yields the contradictory
all-derivable state. So:

- a fact killed on a `break` edge is absent from that edge's contribution and
  therefore absent from the join;
- a fact killed on a `return` or `propagate` error edge is never read, because
  those edges reach the function-return sink and no queried point follows;
- a fact killed on an edge inside the body that continues is still scanned by
  the modified loop rule, so the iteration head never sees it.

The three exits are exhaustive over the ways control leaves a loop body in
v0.22: `break_stmt` reaching `normal_successor` of its resolved target loop,
`return_stmt` and `propagate_let_rhs`'s `Err` edge reaching the function-return
sink, and `value_match`'s `give`/return/break edges, which [FN-1] routes to
exactly those same targets. There is no `continue` form.

### 4.4 Monotonicity under [ENT-1]

The change only removes kills from one scan, so at every loop head the fact
state is a superset of the state v0.22 computes. [ENT-4]'s closure is monotone
and derivability is upward-closed in the state, so:

- every obligation v0.22 discharges is still discharged, and more are;
- no [OP-4] rejection is newly created on discharge grounds;
- a claim v0.22 accepts as non-redundant may become redundant, which [CLM-2]
  makes a non-rejecting advisory precisely so this direction stays monotone.

The one edge that is not monotone is the one [ENT-1] already enumerates.
[CLM-2] rejects a claim whose exact negation the non-contradictory state
derives. A pre-loop fact that now survives to the loop head can supply that
negation, so a program accepted under v0.22 can be newly rejected as a refuted
claim. This is the lifecycle's single deliberate non-monotone edge, already law
in v0.22 ("Refutation is the lifecycle's one deliberate non-monotone edge"),
and it fires only on a claim proven to trap on every execution reaching it — a
defect found at compile time, which is the outcome the rule exists to produce.
The candidate therefore states the property as: **no program that compiles
today loses acceptance on discharge, redundancy, or any [OP-4] ground; the sole
newly reachable rejection is [CLM-2] refutation of a claim that cannot pass.**
The looser sentence "no program that compiles today can break" is not accurate
and is not used.

Note also that a larger surviving state can be contradictory, and [ENT-4]
already fixes that case: at a contradictory point every obligation discharges
and no claim is refuted, so the refutation edge cannot fire there.

### 4.5 Expected effect, and what is not measured here

`ACCEPTANCE.md` establishes the cause and the witness: `D1h` (a loop indexing a
const table with one early `return` in the body) leaves
`ordered_symbol < len(code_lengths)` undischarged, and `D1i` — the identical
`return` hoisted just outside the loop — discharges it. The reach is every
deflate function that has a loop and returns inside it: `read_bits`,
`decode_fixed_symbol`, `decode_fixed`, `inflate`, `copy_distance`,
`build_huffman_table`, `decode_table_symbol`, `decode_dynamic`. sha256's three
loops contain no `return`, which is why sha256 matched prediction.

This candidate does **not** assert a recovered site count. The honest number is
produced by re-running the dark-checker probe of `ACCEPTANCE.md` against a
checker carrying the corrected rule, which is compiler work outside a drafting
task. The acceptance criterion in §6 names that run and its falsifier.

## 5. Item B — the taint gate, subject-position form

### 5.1 New rules

The three rules below join §18, whose heading becomes
`## 18. Obligation discharge: provenance, claims, and the entailment fragment
(normative)`.

---

**[PRV-1]** Provenance is a derived property of storage and bindings, judged
per concrete function body and per [FN-2] instantiation, with exactly two
classes: **external** and **internal**. It is not a fact [ENT-3]: it never
enters the fact state, is never established, closed, killed, or joined, no
relation is derivable from it, and no [ENT-4] answer depends on it. Its sole
consumers are [PRV-2] and [PRV-3].

The judgment domain is every value binding of the body — parameters, `let`
bindings, match binders, requires-clause locals — and every **storage root**,
the root `pbase` binding of a resolved place [OWN-5]. The judgment is
per-binding and flow-insensitive: a binding or root is external exactly when at
least one of its writes carries external provenance, where its writes are its
initializer together with every `set` whose resolved target overlaps it under
[OWN-7]'s overlap relation, and every call argument position through which a
callee may write external content [PRV-2]. This is the least solution of a
monotone system over a two-point lattice, so it exists, is unique, is reached
in finitely many steps, and is independent of the order in which an
implementation visits the body.

The origins of external provenance are exactly:

- **E1.** Every labelled standard-input parameter of a kind-declaring entry
  [FN-7], for the `command` row `Args`, `DirectoryRead`, and `Output`. These
  are the values the environment supplies at program start.
- **E2.** The result binding of a call to a system operation [SYS-2], in the
  provenance class that rule's table fixes for that operation's result.
- **E3.** Every storage root a system operation may write through a `&uniq`
  parameter, in the provenance class [SYS-2] fixes for that parameter, taken
  through the ordinary [EFF-2] boundary projection of the call's `writes`
  occurrences onto caller places.
- **E4.** The result binding of, and every root written by, a call to a
  gated-family member [GATE-1, LEDGER-1]. This version's gated family is a
  writer-visible stub with no call form, so E4 has no instances; it fixes the
  classification before one exists.

Propagation is a total case analysis over the forms a binding's write can take,
and the classes it produces are:

- **P1.** A literal, a named const [CONST-2], and the distinguished forms of
  [ENT-2] are internal.
- **P2.** A place read yields its root's class. A field or subscript of an
  external root is external; this over-approximates, which is the safe
  direction for a gate.
- **P3.** A call to a table operation [OP-1] yields external exactly when some
  operand atom is external, with one datum fixed by the operation table:
  `len<T>(P)` yields internal for every `P`. A `buffer<T>` length is fixed at
  allocation and an `array<T, N>` or `slice<'r, T>` length by its type or
  creation [TYPE-2, OP-1], so a length is program-maintained metadata even when
  the contents it measures are environment-chosen — the T1 property.
- **P4.** A `construct` yields external exactly when some field atom is
  external; a match binder yields the class of the scrutinee's corresponding
  payload, which for a scrutinee that is one binding is that binding's class.
- **P5.** `move`, a `borrow_expr`, a `deref` wrapping, `cvt` [OP-6], and
  `reinterpret` [OP-8] each yield their operand's class.
- **P6.** A call to a user function yields the class [PRV-2]'s derived result
  column fixes for that callee under the actual arguments' classes.

Control flow transfers no provenance. No rule makes a binding external because
of the branch, match arm, or loop it occurs in, or because of the class of an
enclosing scrutinee: a value computed in an arm from internal operands is
internal however the arm was selected. Matching on external bytes therefore
yields program-chosen values, which is why a parser launders. Provenance also
never subtracts a fact: [ENT-3] source S1 establishes a branch's relation
regardless of its operands' classes, because a safety relation is relational —
any storage is indexable below its own length, whoever chose that length.

---

**[PRV-2]** Every concrete function derives one provenance column, in three
parts, and that column is part of what callers rely on [FN-1]. It is derived by
the implementation, never written in source, and adds no `fn_sig` syntax.

The body is judged under [PRV-1] with every parameter internal. Because
propagation is a union over a reachability relation with no negation and no
meet, each derived part is exactly a disjunction over parameter positions, and
the set representation below is exact rather than an approximation.

- **Result column.** Either *unconditionally external*, or the set of parameter
  positions whose external actual makes the result external; the empty set
  means the result is internal at every call site.
- **Write column.** For each `&uniq` parameter, the same shape, stating whether
  the callee may write external content into the root that actual resolves to.
- **Internal-required column.** The set of parameter positions `p` such that
  the body contains an obligation [ENT-6] whose subject term depends on `p`
  under [PRV-1] propagation and which [PRV-3] would gate if `p` were external.

At a `call` whose callee resolves to a user function, for every position in the
callee's internal-required column, the actual atom must be internal. An
external actual in such a position is a hard error citing PRV-2 at that
argument `atom` node, with `SourceCoordinate` equal to that atom's complete
checked half-open source extent, carrying the caller-side provenance chain
[PRV-3] and the callee obligation the position protects. Judging the gate at
the call site, in the caller's own vocabulary, is the same placement
[OP-4] discharge already uses: the call site is the only point that knows where
its data came from.

The three columns are one least fixed point over the call graph of the closed
compilation unit [PROG-1]. The unit is closed, every callee is resolved
statically, and no indirect call form exists, so the call graph is finite and
known; the system is monotone over a finite lattice, so the least fixed point
exists, is unique, and is reached in finitely many steps, including for a
recursive or mutually recursive group. Two conforming implementations therefore
derive the same three columns for every function [ENT-1].

---

**[PRV-3]** The **unasserted fact state** at a program point is the [ENT-4]
closure of the [ENT-3] flow to that point with every S2 fact and every S3 fact
omitted — the two sources whose sole warrant is a runtime check the writer
authored in the same body. Every other source, S1, S4, S5, S6, S7, S9, and S10,
participates unchanged.

An obligation [ENT-6] is **externally subjected** when its **subject term** is
external under [PRV-1]. For the one obligation family this version attaches,
the subject term of `i < len(P)` is `i`, the offset atom of the subscript; the
bound `len(P)` is internal at every site by [PRV-1] P3. When a later version
attaches an obligation whose normalized relation is a conjunction, each
conjunct is judged separately under this rule: a conjunct whose subject is
internal is discharged from the closed fact state as [ENT-6] fixes, and a
conjunct that is externally subjected is discharged only in the unasserted
state. This version's family has exactly one conjunct, so the partition clause
has no instances here and fixes the semantics before one exists.

An externally subjected obligation is discharged exactly when the **unasserted**
fact state at its node derives it. An externally subjected obligation the
unasserted state does not derive is a hard error citing PRV-3 at that
subscript's `psuffix` node, and it is this rejection, not the [OP-4] one, that
the program receives. Its diagnostic carries exactly:

1. the residual, rendered exactly as [ENT-6] fixes;
2. the **provenance chain**: the ordered sequence of binding sites [PRV-1]
   propagation passes through, from the subject term back to its origin,
   naming at the origin the [SYS-2] operation, the [FN-7] input label, or the
   gated member [GATE-1] and that origin's source coordinate, and naming at
   each call boundary crossed the [PRV-2] column that carried it; and
3. the two legal repairs, in these terms: a dominating branch [ENT-3 S1]
   establishing the relation, whose false edge does not reach this subscript
   and whose else the writer fills with the domain outcome; or a restructure in
   which the external value no longer occupies the subject position, the
   operation returning a value the caller matches rather than indexing by
   environment-chosen data.

This rule gates exactly the S2 and S3 sources. It does not gate the [FN-8]
`requires` prologue, whose final `check` is a trap the callee executes and
whose subject position this version's obligation family does not define; it
does not gate the trapping arithmetic modes [OP-2], whose obligations this
version does not attach; it does not gate an external value in a bound
position, where the relation stays true for every value the environment may
supply of that magnitude; and it does not gate a claim [CLM-1] that supports no
obligation, because subject position is defined by an obligation. Each of the
four is a stated boundary of the mechanical classifier, not an unnoticed gap:
they are the residue the fired-claim lifecycle and contract tests own.

### 5.2 [SYS-2] provenance classification (declaration data)

The classification below is operation-table data of the same kind as each
operation's effect row: fixed by this specification, never derived from a body,
never narrowed by a proof, never selected by a call site.

| operation | result class | `&uniq` destination class |
|---|---|---|
| `args_count` | external | — |
| `arg_get` | external | — |
| `host_bytes_len` | external | — |
| `host_copy_bytes` | `Ok(value:)` internal; `CopyTooSmall(required:)` external | `destination` external |
| `host_utf8_len` | external | — |
| `host_copy_utf8` | `Ok(value:)` internal; `Utf8CopyTooSmall(required:)` external | `destination` external |
| `relative_path` | external | — |
| `open_read` | external | — |
| `read_once` | `ReadBytes(count:)` internal; `ReadFailed(error:)` external | `destination` external, `file` external |
| `write_once` | `Ok(value:)` internal; `Err(error:)` external | `output` external |
| `exit_status` | internal | — |

The four internal results are exactly the **transfer counts**: the quantity of
units one attempt moved. Each is bounded by an argument the program chose —
`capacity` for `read_once`, `host_copy_bytes`, and `host_copy_utf8`, `count`
for `write_once` — and [SYS-9] and [ENT-3] S10 already fix that bound as a
declared operation contract. The program chose the bound; the environment chose
only where in `[0, bound]` the transfer stopped, which is the same standing as
a length under T1. Every other magnitude a system operation yields is external,
including `args_count`, `host_bytes_len`, `host_utf8_len`, and the `required:`
payloads, because no program-supplied argument bounds them.

`exit_status` is internal because its operand is the program's own `u8`;
`relative_path` and `arg_get` are external because they carry environment code
units [HOST-1, PATH-1].

This is the line the batch most needs a ruling on, and it is O2 in §8. The
strict alternative — every system result external, including the four transfer
counts — is measured in §7 and costs five of wfgrep's eight claims.

### 5.3 Modified rules (complete replacement deltas, verbatim anchors)

**[FN-1]** One site. "Signatures state everything callers need: parameter modes
and types, return mode and type, effect row, and region parameters." becomes
"Signatures state everything callers need: parameter modes and types, return
mode and type, effect row, region parameters, and the derived provenance column
[PRV-2]. The provenance column is derived from the body, never written; like
the effect row it is a caller-visible part of the interface, so a change that
adds a member to a callee's internal-required column is an interface change
its callers see, exactly as a strengthened `requires` is."

**[OP-4]** One site. "A subscript whose obligation is not discharged is a
compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying
the residual obligation rendered exactly per [ENT-6]; the mechanical fix is a
dominating `claim` of the residual [CLM-1] or a dominating branch establishing
it [ENT-3]." becomes "A subscript whose obligation is not discharged is a
compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying
the residual obligation rendered exactly per [ENT-6]; the mechanical fix is a
dominating `claim` of the residual [CLM-1] or a dominating branch establishing
it [ENT-3]. A subscript whose obligation is externally subjected [PRV-3] is
rejected citing PRV-3 at the same node, and its only mechanical fix is the
dominating branch: a claim does not discharge an obligation whose subject the
environment chose."

**[SYS-2]** One site. "The rows above are exactly that derivation together with
each operation's fixed external, blocking, and trapping classification; a
system operation's row is declaration data and is never derived from a body,
narrowed by a proof, or selected by a call site [ERR-4]." becomes the same
sentence with ", and each operation's result and `&uniq` destination carry one
fixed provenance class [PRV-1]" inserted after "trapping classification",
followed by the classification table of §5.2 as inventory data.

**[CLM-1]** One site. "This version defines no taint judgment: no predicate is
illegal by operand provenance; the subject-position gate is DEFERRED with
recorded delta." becomes "No predicate is illegal by operand provenance: a
claim's legality is judged by [CLM-2], and the subject-position gate [PRV-3]
constrains not the claim but the obligation, rejecting a discharge that rests
on a runtime check the writer authored over a subject the environment chose. A
claim that supports no obligation is therefore ungated, and a claim whose
predicate carries external operands in bound position remains legal."

**[ENT-1]** Two sites. First, "Its judgments are source-acceptance judgments:
obligation discharge [ENT-6], claim redundancy, and claim refutation [CLM-2]
are post-resolution semantic judgments under [DIAG-1], identical in facts-on
and facts-off compilation, and are not an optional optimizer-fact family."
becomes "Its judgments are source-acceptance judgments: obligation discharge
[ENT-6], the subject-position gate [PRV-3], the call-site provenance judgment
[PRV-2], claim redundancy, and claim refutation [CLM-2] are post-resolution
semantic judgments under [DIAG-1], identical in facts-on and facts-off
compilation, and are not an optional optimizer-fact family." Second, "two
conforming implementations derive the same closed fact state at every program
point and the same disposition for every obligation and claim" becomes "two
conforming implementations derive the same closed fact state at every program
point, the same provenance class for every binding and storage root [PRV-1],
the same three columns for every function [PRV-2], and the same disposition for
every obligation and claim".

**[ENT-3]** One site. "Nothing else is a fact: no ensures, struct invariant,
loop induction, user-function postcondition, or taint judgment exists in this
version." becomes "Nothing else is a fact: no ensures, struct invariant, loop
induction, or user-function postcondition exists in this version. Provenance
[PRV-1] is a judgment over bindings and storage roots, not a fact: it
establishes nothing, is derivable from nothing, and no [ENT-4] answer depends
on it."

**[ENT-6]** Two sites. First, "The obligation is discharged exactly when the
closed fact state at that node derives it [ENT-4, ENT-5]." becomes "An
obligation whose subject term is internal [PRV-1] is discharged exactly when
the closed fact state at that node derives it [ENT-4, ENT-5]; an externally
subjected obligation is discharged exactly when the unasserted fact state at
that node derives it [PRV-3]." Second, "The mechanical fix is one dominating
claim or branch establishing the relation — in canonical ANF, one `let` binding
`len<T>(P)` followed by one `claim` on, or `match` over, the admitted
comparison [CLM-1, ENT-3]." becomes the same sentence followed by "For an
externally subjected obligation the `claim` half of that fix is unavailable and
the `match` half is the fix [PRV-3]."

### 5.4 The four bypasses, with witnesses

Stated here because an owner should approve the gate knowing what it does not
catch. Each has a repair path recorded in §8.

**B1 — the `requires` prologue.** [FN-8]'s final `check` is a writer-authored
trap the callee executes, and [ENT-3] S4 turns it into a body-entry fact that
the unasserted state keeps. A writer therefore moves any gated claim behind a
callee contract:

```
fn get['d](data: &'d buffer<u8>, i: own u64) -> own u8 reads('d), traps
  requires { check ilt<u64>(i, len<u8>(deref(data))); }
{ return deref(data)[i]; }
```

A caller passing an externally-provenanced `i` compiles and traps on hostile
input — the original defect, one call away. Closing it needs a subject-position
notion for an arbitrary `requires` condition, which this version's single
obligation family does not supply, and the naive closure (put every parameter a
requires condition mentions into the internal-required column) over-fires badly:
it would gate `store_dynamic_length`'s `position` parameter, whose contract is
about `literal_count` and cannot trap on `position` at all. Ruled open at O3.

**B2 — trapping arithmetic.** `iadd.trap<u64>(external, k)` traps on overflow
of environment-chosen data. This version attaches obligations to subscripts
only, so the gate has nothing to attach to. Closing it is the arithmetic half
of DOSSIER §2.9, explicitly deferred there pending [OP-4] experience.

**B3 — bound position.** `claim n: ilt<u64>(clean_i, external_count) because …`
is legal. This is the under-block `PROBE-W1.md` named when it amended the gate
to subject-position form, and it is deliberate: provenance is not integrity, and
rejecting every claim that mentions an external value rejects the truthful
relational shapes the corpus depends on. For the one obligation family this
version attaches the case has no instances, because the bound is always
`len(P)`, internal by P3.

**B4 — a claim supporting no obligation.** Subject position is defined by an
obligation, so a free-standing `claim` over external data is ungated. Every
claim in the current corpus supports at least one obligation, so B4 has no
instances today, but nothing prevents one.

B1 is the material one. B3 and B4 are the residue the fired-claim lifecycle of
DOSSIER §2.7 owns by design; B2 is a scheduled later batch; B1 is a real hole in
the gate's own shape and the owner should rule on it before activation.

## 6. Acceptance-set delta, monotonicity, and acceptance criterion

**Item A widens.** Every loop head's fact state is a superset of v0.22's, so
discharge only widens. The sole newly reachable rejection is [CLM-2] refutation
(§4.4), which fires only on a claim that traps on every execution reaching it.

**Item B narrows, by exactly one class:** a program in which an
externally-provenanced value occupies an obligation's subject position and only
an S2 `check` or S3 `claim` bounds it. Measured at zero sites across both
corpora (§7). Under [ENT-1] this narrowing is an ordinary version amendment,
not a violation: that rule's removal prohibition is scoped to *checker
strengthening* — "a later specification version may add fact sources and
closure rules, and checker strengthening removes none" — and the gate adds no
fact source and removes none. This is the same accounting the index-surface
batch used for the S8 strike, under the owner's 2026-08-07 ruling that
cross-version compatibility promises are deferred wholesale.

**Item B is itself monotone in the right direction under checker
strengthening.** A later version that adds a fact source makes more obligations
derivable in the unasserted state, so it turns gate rejections into acceptances
and never the reverse. The non-monotone direction is provenance refinement: a
later version that adds an origin, or that reclassifies a [SYS-2] result from
internal to external, newly rejects. O2's ruling is therefore load-bearing for
future versions, not only for this one.

**Acceptance criterion.** Activation is not closed by conformance cases alone.
Both items are closed by re-running the `ACCEPTANCE.md` probe — the test-only
dark checker retaining each function's complete `FunctionEntailment` summary,
with the claim-blinding transform — on the same three programs at the same
denominators, and recording:

1. for Item A, the new proven/claim-supported split on the deflate unit against
   the 5/24 baseline, with the `D1h` / `D1i` pair as the pinned witness; the
   falsifier is `D1h` still failing to discharge, or any site regressing from
   proven to claim-supported;
2. for Item B, the count of gate rejections across both corpora against the
   zero this candidate predicts (§7); the falsifier is any rejection, which
   would mean this document's provenance walk is wrong.

Both are measurements the corrected checker produces directly; neither is a
conformance verdict edit.

## 7. Corpus impact (measured 2026-08-07)

**Item A: no program loses acceptance**, by §4.4. No corpus measurement of
recovered sites is asserted here (§4.5).

**Item B, under this candidate's rules: zero claims become illegal.**

The corpus holds 144 claims: 70 in `tests/programs/*.wf`, 74 in
`tests/conformance/cases/*.wf`, and 0 in `research/experiments`. Provenance is
empty in every program that has no boundary, and only a kind-declaring entry
[FN-7] admits the system inventory [SYS-3], so the walk reduces to the five
boundary-bearing programs that contain a claim at all:

| program | claims | externally subjected |
|---|---|---|
| `tests/programs/wfgrep.wf` | 8 | 0 |
| `tests/conformance/cases/run-sysfile-multichunk.wf` | 4 | 0 |
| `run-syshost-copybytes-toosmall-unchanged.wf` | 1 | 0 |
| `run-syshost-copyutf8-toosmall-unchanged.wf` | 1 | 0 |
| `run-syshost-copyutf8-invalid-unchanged.wf` | 1 | 0 |

The four `run-sysfile-multichunk` claims have literal subjects (`0`, `3`, `4`,
`7`), internal by P1. The three copy cases share one shape — subject `offset`,
a `0`-initialized `+1` counter. wfgrep's eight subjects are `cursor`, `target`,
`carry`, `probe`, `spot`, `ahead`, `source_index`, and `moved`; every one is a
counter chain seeded by a literal or by a transfer count, which §5.2 classifies
internal. The values wfgrep does hold externally — `arguments` from
`args_count`, the pattern and input bytes, every `IoError` — reach only branch
guards and `ieq<u8>` comparisons, never a subject position. That reproduces
`PROBE-TAINT.md`'s finding on the real boundary rather than by inspection:
wfgrep compares bytes but never indexes by one.

**Under the strict alternative of O2** — every system result external,
including the four transfer counts — the same walk gives **five rejections, all
in wfgrep**. `read_once`'s count `taken` flows to `available`, thence to
`terminator`, `scan`, `probe`, `spot`, `source_index`, `tail`, and `carry`,
so `carry_in_input`, `probe_in_input`, `spot_in_input`, and
`shift_read_in_input` all become externally subjected; and `copy_range` is
called with `from: scan`, so its `from` parameter enters the internal-required
column and `copy_read_in_source` fails at that call site under [PRV-2].
`ahead_in_pattern` and `shift_write_in_input` survive (their subjects `ahead`
and `moved` are literal-seeded), as does `copy_write_in_destination`. Five of
eight is the number the owner is choosing against when ruling O2, and every one
of the five is a relation no environment can falsify: the program computes
`room = 4096 - carry` and passes it as `capacity`, so `available <= 4096` holds
by construction. They are exactly the false positives `PROBE-TAINT.md` reported
as zero.

**The motivating sites are not caught, in either reading.** The three
canonical-Huffman claims that `ACCEPTANCE.md` names — `decode_table_symbol`'s
`ordered_in_symbols`, `build_huffman_table`'s `order_slot_in_offsets` and
`destination_in_symbols` — are exactly the shape the gate is for: an index
derived from decoded file content. They stay legal because
`tests/programs/raw_deflate_vectors.wf` builds the compressed input with
`make_dynamic_input()`, an ordinary heap buffer, and its `main` is unlabelled,
so the unit is not system-admitted and no value in it has an origin at all. The
gate would fire on those three sites the day the decoder is wired to a real
`read_once`. Until then Item B is a rule with correct semantics and no live
instance, and the evidence that it does the intended thing is the reasoning in
§5, not a corpus measurement. That is a weaker evidential position than Item A
occupies, and the honest options are recorded as O8.

## 8. Ruled and open list

Nothing is ruled by this document. Item A is drafted on the owner's
explanation, pending the final word; every item below is a real fork the draft
takes a position on and does not resolve silently.

- **O1 — Item A's exact scan (drafted: reachability).** The candidate excludes
  every kill event from which the loop's body entry is unreachable within the
  body, which subsumes the owner's stated form ("edges leaving the loop or the
  function are excluded") and additionally drops the kills on ordinary edges
  that only a leaving edge follows. Both are sound (§4.2). Alternative: the
  narrower enumerated form, which is easier to implement as a syntactic scan
  and gives up the `set`-then-`return` case. **Needs a ruling because the
  drafted form is a widening beyond the wording the owner gave.**
- **O2 — the provenance class of the four transfer counts (drafted:
  internal).** §5.2 and §7 give both readings and their measured cost: internal
  costs zero corpus claims and leaves a subject-position under-block on any
  claim that bounds a count more tightly than its contract does; external costs
  five of wfgrep's eight claims, all of them relations no environment can
  falsify. This is the batch's load-bearing ruling, and it binds future
  versions too (§6).
- **O3 — the `requires` bypass B1.** Ship the gate with the hole named, or hold
  Item B until a subject-position notion for `requires` conditions exists? The
  naive closure over-fires (§5.4). A third option: gate the `requires`
  condition itself once obligations attach to a second family.
- **O4 — where the gate is judged (drafted: at the obligation).** The DOSSIER
  and the task frame the gate claim-side ("a claim is rejected when …"). The
  draft attaches it to the obligation instead, because subject position is
  defined only by an obligation and the obligation-side form is total,
  needs no notion of which claim supports which obligation, and is decided by
  two closures of the same state. The cost is that the diagnostic lands at the
  subscript rather than at the claim; the drafted diagnostic names both.
- **O5 — derived versus written provenance (drafted: derived).** A written
  annotation is a grammar change, hands a trust-bearing declaration to the
  untrusted writer, and needs a variance rule when indirect calls arrive.
  Recorded as rejected, not omitted.
- **O6 — flow-insensitive per binding (drafted: yes).** A flow-sensitive
  judgment would keep a binding internal before its first external write. It
  needs a join rule, a kill rule, and a loop rule of its own — a second
  dataflow system beside [ENT-3] — for a precision gain no corpus site
  currently needs. Revisit if O2 rules the strict way, where precision starts
  to matter.
- **O7 — whether [DIAG-2] should retain the provenance column.** The claim
  ledger of DOSSIER §2.7 wants provenance per row. The ledger is not in this
  batch, so the draft does not touch [DIAG-2]; adding retention later is
  additive.
- **O8 — Item B's evidence position.** The gate has no live instance in the
  corpus (§7). Options: activate on the reasoning and let the deflate wiring
  supply the instance later; hold Item B and ship Item A alone, which is the
  item with a direct measurement and a pinned witness; or first port the
  deflate decoder onto a real `read_once` boundary so the three sites become
  live and the gate is measured doing the thing it was advanced for. The third
  is the only one that produces evidence before approval.
- **O9 — batch composition and ordering.** Item A is small, measured, and
  monotone; Item B is large, unmeasured on live instances, and narrowing. They
  ride together here because the task asked for both. Splitting them is
  available at no cost: they share no rule text except [ENT-1]'s judgment list,
  and Item A's [ENT-5] site does not overlap the FLOOR-5 batch's [ENT-5] site
  (that one inserts before the join paragraph's `loop_stmt` sentence; this one
  replaces the final paragraph). If FLOOR-5 lands first, §5's [ENT-6] and
  [OP-4] anchors must be re-taken against the respelled bytes.
- **O10 — [FN-8]'s foreign-entry adapter.** FN-8 says a gated foreign entry
  executes the `requires` prologue with "trap semantics unchanged in this
  version", while DOSSIER §2.8 requires the adapter's failure path to follow
  the boundary's error protocol, because foreign arguments are external by
  definition and the environment has no trap authority. The gate makes that
  inconsistency explicit. It has zero instances (the gated family is a stub
  with no call form), so the draft does not touch [FN-8]; naming it here is the
  point.

## 9. Assembly and verification record

All three checks were run 2026-08-07 against the active spec at 8f91ede.

1. **Anchor exactness.** Each of the eleven verbatim anchors quoted in §4.1,
   §5.1, and §5.3 — ten rule sites plus the §18 heading — was matched as a
   fixed string against `spec/kernel-spec-v0.22.md`. Every one matches exactly
   one line. No anchor is paraphrased and none is ambiguous.
2. **Grammar containment.** The active spec's fenced blocks span lines
   98–126, 130–139, 143–165, 169–182, 660–662, 706–740, 766–826, 830–842, and
   1050–1093. Every modification site (lines 386, 400, 846, 1000, 1004, 1008,
   1016, 1042, 1044) lies outside all of them, so no site touches a grammar
   production, a terminal, the operation table, or the worked example.
3. **Additive assembly.** The only new bytes this candidate introduces are the
   three [PRV] rules, the [SYS-2] provenance table, and the changed §18
   heading; the ten rule sites are prose-for-prose replacements. Those new
   bytes were appended to a scratch copy of the active spec and the native
   verifier re-run on it: **65 productions, 75 decisions, 76 terminal
   predicates, exit code 0** — identical to the §3 baseline. The grammar impact
   of this batch is therefore nil, mechanically confirmed rather than asserted.
   The complete ten-anchor assembly is produced by the activation task, whose
   byte comparison is the authoritative one.

No file under `spec/`, `docs/`, `tests/`, or `compiler/` is modified by this
candidate; the scratch assembly lives outside the repository and is deleted.
