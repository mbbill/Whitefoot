# Provenance and the subject-position gate — held design evidence

Status: **DESIGN EVIDENCE FOR THE GATE ACTIVATED AS v0.27** (stage 5a-R
review, 2026-08-09; gate activated 2026-08-10 at `5ab45aa`, exact-byte approval
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f` in
`governance/APPROVALS.md`). Non-authoritative history: this file is neither an
activation candidate nor an exact delta against the active specification. Its
header said "HELD DESIGN EVIDENCE ... the active language is v0.24" for six
days after the design it held was activated, contradicting the ledger;
corrected rather than left, because a record whose own status disagrees with
the ledger is the defect class this project keeps re-finding.

This record began as a complete exact-text delta against the then-active v0.22
specification. Sections 2, 6, and 9 retain that old header, replacement text,
anchors, and verification only as dated design provenance. They are not
instructions for patching v0.24 or a later active specification and must not be
retargeted by fuzzy matching.

The [ENT-5] continuing-kill correction that was once described here as a
companion is now installed in v0.24. It is a fixed premise of the measurement,
not an unactivated dependency of this held design.

## 0. Why it remains held after a live measurement

The old claim that the gate had no live instance is obsolete. Task 0041 ran the
held rules over the boundary-fed raw-DEFLATE compilation unit installed with
v0.24. The frozen unit contains 33 subscript obligations under 23 claims and
does exercise the three canonical-Huffman sites that motivated the gate:
`decode_table_symbol`'s `ordered_in_symbols`, and `build_huffman_table`'s
`order_slot_in_offsets` and `destination_in_symbols`.

That measurement was useful precisely because it was negative. The old place-
read rule caught two canonical sites but missed `destination_in_symbols`: the
selected value `offsets[count_index]` was classified only from the internal
`offsets` root and ignored its external selection offset. The old PRV-2 column
also retained a parameter position without retaining which concrete leaf
obligation that position protected. Stage 5a-R repairs only those two defects.
Section 8 records the complete A-only rewalk: canonical 3/3, exactly one newly
external obligation subject, and no new gate in the original boundary
controls.

This positive rule-review result still does **not** make the design activatable.
O3 remains material: moving the checked access into a callee `requires`
condition bypasses the gate. Stage 7 must close that hole before any later
exact-byte specification proposal may select this design. Until then this file
is evidence for review, not language authority.

## 1. What is ruled and what is open

Ruled for this held design by the completed measurements and stage-5a-R
selection:

- **R1 (was O2) — the provenance class of the four transfer counts: internal.**
  The program supplies the bound (`capacity`, or `count` for `write_once`) and
  [SYS-9] with [ENT-3] S10 already fix `count <= bound` as a declared operation
  contract, so only the position within `[0, bound]` is environment-chosen —
  the same standing a length has under T1. The strict alternative was measured
  and rejected: it illegalizes five of wfgrep's eight claims, every one of them
  a relation no environment can falsify, since the program computes
  `room = 4096 - carry` and passes it as `capacity`. Those are precisely the
  false positives `PROBE-TAINT.md` reported as zero. This ruling is part of the
  held design; a later design that reclassifies one of these results as
  external introduces a new narrowing that must be measured and reviewed (§5).
- **R2 (was O4) — the gate attaches to the obligation, not the claim.** Subject
  position is defined only by an obligation, so the obligation-side form is
  total, needs no claim-to-obligation mapping, and is decided by two closures
  of one state. The S2 (`check`) exclusion is kept: without it the gate is a
  one-line bypass. Cost accepted: the diagnostic lands at the exact protected
  subscript and carries its residual, provenance chain, and repairs; it does
  not relocate to an upstream claim, which may not exist.
- **R3 — place reads use A-only explicit dataflow.** A resolved place read
  joins the provenance of its storage root with every subscript-offset atom in
  that resolved place; field selection preserves the accumulated class and
  `len(P)` remains internal. This closes the measured selected-offset miss
  without adding branch, match, loop, or write-address implicit flow.
- **R4 — PRV-2 retains a finite parameter-datum-to-leaf relation.** A protected leaf
  is one concrete [FN-2] instantiation, one exact [ENT-6] obligation occurrence,
  and one normalized conjunct ordinal; this design's only conjunct has ordinal
  zero. Direct and call-composed edges reach one finite least fixed point. Call
  paths are reconstructed only after convergence for diagnostics; they are not
  members of the lattice.

Open, and deliberately unresolved:

- **O3 — the `requires` bypass (B1 of §7).** The material hole. Ship the gate
  with it named, or hold until a subject-position notion exists for an
  arbitrary `requires` condition? The naive closure over-fires badly. Lead
  ruling: keep open; this is a reason the batch is held, not a detail to settle
  in drafting. Stage 7, not this review, owns the closure.

Two former questions are now settled within this design. Provenance remains
derived rather than writer-annotated (O5), and storage remains flow-insensitive
per whole root (O6). The latter intentionally produces the five site-local
stored-block precision positives reported in §8. Whether later claim-ledger
tooling retains provenance (old O7) remains outside this design and adds no
current metadata guarantee.

Noted, not open here: [FN-8]'s foreign-entry adapter still executes the
`requires` prologue with "trap semantics unchanged in this version", while
`DOSSIER.md` §2.8 requires the adapter's failure path to follow the boundary's
error protocol, because foreign arguments are external by definition and the
environment has no trap authority. The gate makes that inconsistency explicit.
The current gated family remains a stub with no call form, so this design does
not alter [FN-8]. The foreign-entry behavior remains a finding for the later
requires-as-goal work rather than evidence that O3 is closed.

## 2. Historical v0.22 proposed version-header paragraph

The quotation below is the header drafted on 2026-08-07 for the abandoned
v0.22 exact-delta form. Its zero-site measurement and set-only PRV-2 summary
are superseded by §§4 and 8. It is retained verbatim as historical evidence,
not as current proposed text.

> Status: REVIEW CANDIDATE vNEXT (2026-08-07; provenance and the
> subject-position taint gate). Adds a derived provenance judgment and one
> gate: a value originating at a [SYS-2] system operation, at a kind-declaring
> entry's labelled standard input [FN-7], or at a gated-family member [GATE-1,
> LEDGER-1] carries external provenance; provenance propagates by dataflow
> through operands, place roots, and calls, never through control flow, so a
> `match` on external bytes yields program-chosen values; `len` and the four
> transfer counts whose [SYS-2] contract bounds them by a program-supplied
> argument are internal; and an obligation [ENT-6] whose constrained subject
> term is externally provenanced is discharged only in the unasserted fact
> state — the [ENT-4] closure with every S2 `check` and S3 `claim` fact omitted
> — so a dominating branch discharges it and a writer-authored runtime check
> does not. Provenance is a derived signature-visible column, never written
> source: each concrete function derives its result provenance, its `&uniq`
> write provenance, and the parameter positions its body's gated obligations
> require to be internal, and a call passing an externally-provenanced actual
> to such a position is rejected at the call site with the caller's own
> provenance chain, under one least fixed point over the closed call graph
> [PROG-1] — Whitefoot has no indirect calls, so the fixed point exists and is
> unique. Specification delta: numbered rules +3/-0 ([PRV-1] provenance,
> [PRV-2] the derived signature column, [PRV-3] the gate); seven existing rules
> modified at nine verbatim-anchored modification sites (a site is one
> contiguous verbatim-anchored replacement): FN-1 (the signature-content
> sentence gains the derived provenance column), OP-4 (the mechanical-fix
> sentence names the gate), SYS-2 (one provenance classification per operation
> result and per `&uniq` destination, as declaration data), CLM-1 (the deferral
> sentence is replaced by the live gate reference), ENT-1 (two sites: the
> judgment list gains the provenance judgment and the gate; the
> implementation-agreement sentence covers provenance), ENT-3 (the "nothing
> else is a fact" sentence stops denying a taint judgment and states that
> provenance is not a fact), ENT-6 (two sites: the discharge condition routes
> through [PRV-3]; the mechanical-fix sentence). Section 18's heading gains
> provenance. Tokens +0/-0; terminal spellings +0/-0; grammar productions
> +0/-0; operation-table rows +0/-0; source constructs +0/-0; sections +0. No
> rule carries an exception clause [META-3]: the provenance classification of
> each system result is [SYS-2] declaration data and the propagation judgment
> is a total case analysis over expression forms. The accepted-program set
> narrows by exactly one class: a program in which an externally-provenanced
> value indexes storage and only a `check` or `claim` bounds it, measured at
> zero sites across both corpora (144 claims, 2026-08-07). Selection ground:
> `DOSSIER.md` §2.5 as amended by `PROBE-W1.md` (provenance is not integrity;
> the mechanical gate is subject position) and `PROBE-TAINT.md` (zero taint
> false positives on the real boundary). These bytes are non-authoritative
> until the derived-material review, full-document hash, exact owner approval,
> and active-target installation complete.

## 3. Grammar delta

The held design needs no written source construct: provenance remains derived,
so A-only and the PRV-2 relation add no production, terminal predicate, token
form, or operation-table row. This is a design conclusion, not verification of
future active-spec bytes. Section 9 retains the old v0.22 assembly check only
as historical evidence; a future exact-byte proposal must verify the then-
current stable specification independently.

Provenance is deliberately **derived, not written** (O5).

## 4. Held rule design

The three rules below are normative only within this held design record. They
do not join the active specification. Their labels are working identifiers for
the later exact-byte workflow.

---

**[PRV-1]** Provenance is a derived property of storage and bindings, judged
per concrete function body and per [FN-2] instantiation, with exactly two
classes: **external** and **internal**. It is not a fact [ENT-3]: it never
enters the fact state, is never established, closed, killed, or joined, no
relation is derivable from it, and no [ENT-4] answer depends on it. Its sole
consumers are [PRV-2] and [PRV-3].

One **provenance datum** is one value in that two-point lattice. Every plain
value binding and every storage root has one aggregate datum. A value binding
whose type is a payload-carrying enum instead has one datum for each direct
`(variant, payload field)` projection; its aggregate is the derived join of
those finite projections rather than an independently stored datum. A direct
projection stores the aggregate class of its field atom. If that atom is
itself a payload-carrying enum, matching it into a new binder conservatively
seeds every direct projection of the inner binder with that one outer class;
selectors do not form recursive payload paths. A tag or the control choice of
a variant contributes no provenance by itself. Storage roots remain
whole-root only: they retain no variant, field, element, or path projections.
Storing an enum writes its derived aggregate into the whole root, and reading
an enum from storage seeds every direct payload projection from that selected
whole-root read. These value projections are not field-sensitive storage.

The judgment domain is every value binding of the body — parameters, `let`
bindings, match binders, requires-clause locals — and every **storage root**,
the root `pbase` binding of a resolved place [OWN-5]. The judgment is
per-binding and flow-insensitive. A plain binding or root is external exactly
when at least one of its writes carries external aggregate provenance. Each
enum payload projection instead joins only writes to that component, after
which the binding aggregate is derived as above. The writes are the initializer
together with every `set` whose resolved target overlaps the root under
[OWN-7]'s overlap relation, and every call argument position through which a
callee may write external content [PRV-2]. For a `set`, the written class is
the aggregate class of the right-hand-side value. Subscript offsets in the
resolved target locate the write and participate in overlap, but do **not**
become provenance of the written value or root. This is the least solution of
a monotone system over finite two-point datums, so it exists, is unique, is
reached in finitely many steps, and is independent of visit order.

The origins of external provenance are exactly:

- **E1.** Every labelled standard-input parameter of a kind-declaring entry
  [FN-7], for the `command` row `Args`, `DirectoryRead`, and `Output`. These
  are the values the environment supplies at program start.
- **E2.** The result value of a call to a system operation [SYS-2], in the
  aggregate or per-variant payload classes that rule's table fixes, whether
  that value is let-bound or flows directly into another expression, `give`,
  `set`, or `return`. A listed payload class initializes that exact projection
  and the aggregate is their join; a plain result initializes its one
  aggregate datum.
- **E3.** Every storage root a system operation may write through a `&uniq`
  parameter, in the provenance class [SYS-2] fixes for that parameter, taken
  through the ordinary [EFF-2] boundary projection of the call's `writes`
  occurrences onto caller places.
- **E4.** The result binding of, and every root written by, a call to a
  gated-family member [GATE-1, LEDGER-1]. The held design's gated family is a
  writer-visible stub with no call form, so E4 has no instances; it fixes the
  classification before one exists.

Propagation is a total case analysis over the forms a binding's write can take,
and the classes it produces are:

- **P1.** A literal, a named const [CONST-2], and the distinguished forms of
  [ENT-2] are internal in every aggregate or payload datum they produce.
- **P2.** A resolved place read starts with its storage root's class and joins
  the class of every subscript-offset atom encountered in the complete
  resolved place. A field step preserves the accumulated class. The selected
  value is therefore external exactly when the root or at least one such
  offset is external; an external offset selects an external value even from
  an otherwise internal root. The offset of a `set` target is not a place
  read's selected-value edge and remains excluded by the write rule above.
- **P3.** A call to a table operation [OP-1], other than the `cvt` and
  `reinterpret` transformations handled by P5, classifies each result datum
  separately. A plain result is the join of its operand atoms. The current
  table's payload-carrying results in this class are exactly the checked
  integer arithmetic operations returning `Result<T, Overflow>` or
  `Result<T, DivError>`: their `Ok(value:)` projection is the join of the
  arithmetic operands, while their `Err(error:)` projection is internal
  because the tag-only error value carries no operand atom and the control
  choice transfers no provenance. Their derived aggregate is the join of
  those two projections. One plain-result exception is fixed by the operation
  table: `len(P)` yields internal for every `P`; it does not apply P2 to `P`.
  A `buffer<T>` length is fixed at
  allocation and an `array<T, N>` or `slice<'r, T>` length by its type or
  creation [TYPE-2, OP-1], so a length is program-maintained metadata even when
  the contents it measures are environment-chosen — the T1 property.
- **P4.** A non-enum `construct` yields the join of its field atoms in its
  aggregate datum; it creates no persistent field projection. A nullary
  construct of a tag-only enum is internal. A nullary variant of a payload-
  carrying enum, such as `Option::None`, adds no edge to any payload projection
  and therefore has an internal derived aggregate when it is the binding's
  only write. An enum
  `construct` writes each field atom into that exact constructed
  `(variant, payload field)` projection using that atom's aggregate class,
  leaves other variant projections without an edge from that construct, and
  derives the aggregate by joining all projections. An enum `match` binder
  receives only its corresponding scrutinee projection, not the scrutinee
  aggregate or a different variant's payload. When that binder is itself a
  payload-carrying enum, the received class seeds all of its direct
  projections as fixed above. For `value_match` and `value_if`, each result
  projection is the union of the corresponding projections delivered by every
  reachable `give`; a plain result similarly unions the delivered aggregates.
  A delivered nested enum contributes its aggregate class to the enclosing
  direct projection. The scrutinee, condition, and arm choice add no
  provenance merely by controlling which `give` executes.
- **P5.** `move`, a `borrow_expr`, a `deref` wrapping, and `reinterpret`
  [OP-8] preserve the operand aggregate and every payload projection their
  result type retains. `cvt` [OP-6] preserves the operand class into its plain
  result or `Ok(value:)` projection; its `Err(error:)` projection is internal
  because `NarrowError` is tag-only and the success/error control choice adds
  no provenance. For `let x = propagate e`, `x` receives exactly
  `e`'s `Ok(value:)` projection; when `x` is a payload-carrying enum, that
  outer class seeds all of `x`'s direct projections. The `Err(error:)`
  projection is copied as one aggregate field class onto the enclosing
  function's corresponding `Err(error:)` result projection along the
  automatic return edge. The tag dispatch itself adds no class.
- **P6.** A call to a user function yields the plain result datum or each enum
  result projection that [PRV-2]'s derived result column fixes for that callee
  under the actual arguments' classes; its aggregate is the join of the
  returned projections. No call collapses distinct payload projections before
  a match or `propagate` consumes them.

Control flow transfers no provenance. No rule makes a binding external because
of the branch, match arm, or loop it occurs in, or because of the class of an
enclosing scrutinee: a value computed in an arm from internal operands is
internal however the arm was selected. Matching on external bytes therefore
yields program-chosen values, which is why a parser launders. Write-address
choice is excluded in the same explicit sense. This is an explicit-dataflow
policy, not noninterference or a complete implicit-flow analysis.

The following hostile shape is deliberately accepted. It is **PSEUDOCODE,
not source syntax**; only the stated provenance consequence is normative here:

```
let a = buffer_new(2_u64, 0_u64);
let in_range = ilt(external_i, 2_u64);
if in_range {
  set a[external_i] = 1_u64;
} else {
  return domain_error;
}
let selected = a[0_u64];
let selected_ok = ilt(selected, 1_u64);
claim selected_in_b: selected_ok because "writer assertion";
let value = b[selected];
```

The target offset is external, but the value written is the internal literal
`1`, so `a` remains internal; `a[0]`, `selected`, and the supporting claim are
internal and legal. Choosing `external_i == 0` makes the claim fire. Likewise,
an external branch or match may select which internally computed literal is
used without making that literal external. These are known write-address and
control implicit-flow under-blocks, not accidental consequences hidden by P2.

Payload projection has a separate hostile control. This too is
**PSEUDOCODE, not source syntax**:

```
let r = value_if external_choice {
  give Ok(value: 0_u64);
} else {
  give Err(error: external_error);
};
let value = propagate r;
```

The `Ok(value:)` projection is internal, the `Err(error:)` projection is
external, and the aggregate `r` is external by their join. The external
condition does not taint either projection. `propagate` therefore binds an
internal `value` on its success edge and copies only the external error
projection to the enclosing function result. If several `value_if` or
`value_match` arms deliver the same projection, that component unions their
delivered classes; an external condition or scrutinee still contributes no
edge of its own.

Provenance also never subtracts a fact: [ENT-3] source S1 establishes a
branch's relation regardless of operand class, because a safety relation is
relational — any storage is indexable below its own length, whoever chose that
length.

---

**[PRV-2]** Every concrete [FN-2] function instantiation `F` derives one
provenance column in three parts. The column is derived by the implementation,
never written in source, and adds no `fn_sig` syntax. This held rule states
what a later implementation must retain; it does not claim that current
checked metadata already contains these relations or witnesses.

One **parameter datum** is `(position, selector)`. A non-payload-carrying
parameter has the sole selector `plain`; a payload-carrying enum parameter has
one selector for each exact `(variant, payload field)` projection admitted by
its type. An enum aggregate has no independent selector: every use of it
expands definitionally to the union of its projection datums. Selector order is
`plain` first when present, then enum projections by variant declaration order
and payload-field declaration order. For each aggregate or payload-projection
datum `x` in `F`, its dependency is either *unconditionally external* or the
finite set of `F`'s parameter datums whose external class makes `x` external
under [PRV-1]. The empty set means internal for every call. At a call, each
callee parameter datum is substituted with the actual atom's corresponding
plain or payload projection; an enum aggregate is derived only after its
projections are instantiated. Propagation is union over positive edges, so
this representation is exact for the stated datums.

Before body propagation, each ordinary source parameter's plain or direct
payload datum has the identity dependency on its matching parameter datum.
The parameter binding's aggregate, and a formal root read through it, then
follow the ordinary aggregate rules above. An E1-labelled standard-input
parameter of a kind-declaring entry is unconditionally external instead; it
does not create a caller-substitutable sentinel or parameter dependency.

- **Result column.** A plain result has one aggregate dependency. A payload-
  carrying enum result has one dependency for each `(variant, payload field)`
  projection; its aggregate is derived by joining those entries and is not a
  separate column member. Every explicit `return e` contributes `e`'s plain
  datum or direct payload projections componentwise, and multiple explicit
  return edges union their contributions. P5's automatic `propagate` error
  return contributes the selected `Err(error:)` projection by the same rule.
  A call and P6 instantiate the converged entries independently.
- **Write column.** For each `&uniq` parameter, the dependency of content that
  `F` may write into the root to which that parameter's actual resolves. It is
  the union of the aggregate dependency of every `set` right-hand side whose
  resolved target overlaps that formal root, every E3 system write projected
  there (unconditionally external), and every callee write-column entry
  substituted through a call whose [EFF-2] boundary projection reaches that
  root. A target address alone contributes no dependency.
- **Internal-required column.** A finite relation `Req(F)` from a parameter
  datum to a **protected leaf**. A protected leaf is the triple of one
  concrete [FN-2] instantiation `G`, one exact [ENT-6] obligation occurrence in
  `G`'s checked body, and one ordinal in that obligation's normalized conjunct
  order. The occurrence is identified by its complete node path, not merely by
  a claim name or source line; two concrete generic instantiations of the same
  source obligation are distinct leaves. This design's relation has one
  conjunct per obligation, so every current leaf carries ordinal zero.

The internal-required relation has exactly these generators:

1. **Direct edge.** For each local obligation leaf `L = (F, O, k)` that the
   unasserted fact state does **not** discharge, add `(d, L)` for every
   parameter datum `d` in the dependency of conjunct `k`'s subject term. An
   unasserted-proven leaf adds no edge. Such an unasserted-undischarged subject,
   when unconditionally external, is a local [PRV-3] violation; it is not
   represented by a sentinel datum.
2. **Call-composed edge.** At a concrete call `c: F -> G`, for each
   `(d_q, L)` in `Req(G)`, select from the actual atom at `d_q.position` the
   plain or payload projection named by `d_q.selector`. For every caller
   parameter datum `d_p` in that selected actual datum's dependency, add
   `(d_p, L)` to `Req(F)`. If the selected actual datum is unconditionally
   external, the call has a local violation at that argument; again no sentinel
   is added to the relation.

Result dependencies, write dependencies, and `Req` call-composition are solved
together as the least fixed point over the closed compilation unit [PROG-1].
There are finitely many concrete instantiations, parameter datums, value
bindings, their finite enum payload projections, storage roots, concrete call
occurrences, obligation occurrences, and normalized conjuncts, and every
transfer only adds a member, so the product lattice is finite and monotone. The
least fixed point exists, is unique, is order-independent, and terminates for
recursive and mutually recursive groups. A purely recursive cycle with no
direct protected leaf remains empty. Call
paths are deliberately absent from this lattice.

At a resolved call `c: F -> G`, `Targets(c, q)` contains every pair
`(d_q, L)` in `Req(G)` whose datum is at position `q` and whose corresponding
actual plain or payload projection is external. If that set is nonempty, the
compiler emits exactly
one PRV-2 event for argument `q`, not one event per datum or leaf. Adding a
second leaf to the same required parameter datum changes targets but does not
narrow acceptance again; adding a different payload datum at the same position
may narrow a previously unrequired projection. The event cites PRV-2 at the
argument atom's complete checked half-open source extent and retains its finite
`Targets` set.

Only after the relation has converged does the diagnostic choose one simple
witness from the retained target pairs. It minimizes call boundaries. Ties
compare lexicographically the complete sequence of boundary states, each
containing the call node path, argument node path, and both the callee and
caller parameter datums. Parameter positions use declaration order and
selectors use the order above. The protected leaf's complete node path and
conjunct ordinal follow that route key. If the remaining
tie is solely between concrete instantiations of the same source route and
leaf, [DIAG-1]'s implementation-defined but stable deterministic concrete-
instantiation order selects one; it must be fixed for that compiler executable
and must not depend on hash iteration, worklist order, or which predecessor was
first discovered. Within each function, a PRV-1 predecessor segment minimizes
binding edges and orders ties lexicographically by the complete sequence of
predecessor node paths and their selectors under the same order.
Reconstruction tracks visited
`(concrete function, parameter datum, protected leaf)` states, so recursive
cycles terminate rather than appearing in the witness.

The rendered chain runs from the exact leaf backward through callee parameter
datums and the selected call boundaries to the rejecting caller actual, then
through
the caller's chosen PRV-1 predecessor chain to its external origin. Thus one
argument produces one deterministic event while preserving all targets for
later tooling. Judging the violation at the call site is the same caller-side
placement that [OP-4] discharge uses: that point owns the actual's origin
vocabulary.

---

**[PRV-3]** The **unasserted fact state** at a program point is the [ENT-4]
closure of the [ENT-3] flow to that point with every S2 fact and every S3 fact
omitted — the two sources whose sole warrant is a runtime check the writer
authored in the same body. Every other source, S1, S4, S5, S6, S7, S9, and S10,
participates unchanged.

An obligation [ENT-6] is **externally subjected** when its **subject term** is
external under [PRV-1]. For the one obligation family this held design attaches,
the subject term of `i < len(P)` is `i`, the offset atom of the subscript; the
bound `len(P)` is internal at every site by [PRV-1] P3. When a later version
attaches an obligation whose normalized relation is a conjunction, each
conjunct is judged separately under this rule: a conjunct whose subject is
internal is discharged from the closed fact state as [ENT-6] fixes, and a
conjunct that is externally subjected is discharged only in the unasserted
state. The held family's relation has exactly one conjunct, so the partition clause
has no instances here and fixes the semantics before one exists.

An externally subjected obligation is discharged exactly when the **unasserted**
fact state at its node derives it. An externally subjected obligation the
unasserted state does not derive is a hard error citing PRV-3 at that
subscript's `psuffix` node, and it is this rejection, not the [OP-4] one, that
the program receives. Its diagnostic carries exactly:

1. the residual, rendered exactly as [ENT-6] fixes;
2. the **provenance chain**: for a local error, the shortest [PRV-1] binding
   chain from the subject term back to its origin, with complete-node-path tie
   breaking; for a call error, the post-convergence [PRV-2] simple witness from
   the exact protected leaf through callee parameter datums and call boundaries to
   the caller actual, concatenated with that caller actual's shortest PRV-1
   chain to its origin. The origin names the [SYS-2] operation, [FN-7] input
   label, or gated member [GATE-1] and its source coordinate; and
3. the two legal repairs, in these terms: a dominating branch [ENT-3 S1]
   establishing the relation, whose false edge does not reach this subscript
   and whose else the writer fills with the domain outcome; or a restructure in
   which the external value no longer occupies the subject position, the
   operation returning a value the caller matches rather than indexing by
   environment-chosen data.

This rule gates exactly the S2 and S3 sources. It does not gate the [FN-8]
`requires` prologue, whose final `check` is a trap the callee executes and
whose subject position the held obligation family does not define; it
does not gate the trapping arithmetic modes [OP-2], whose obligations this
version does not attach; it does not gate an external value in a bound
position, where the relation stays true for every value the environment may
supply of that magnitude; and it does not gate a claim [CLM-1] that supports no
obligation, because subject position is defined by an obligation. Each of the
four is a stated boundary of the mechanical classifier, not an unnoticed gap:
they are the residue the fired-claim lifecycle and contract tests own.

## 5. Held [SYS-2] provenance classification (declaration data)

The classification below is operation-table data of the same kind as each
operation's effect row within this design: never derived from a body, narrowed
by a proof, or selected by a call site. It is not active declaration data until
a later specification workflow adopts it.

| operation | result aggregate or payload-projection class | `&uniq` destination class |
|---|---|---|
| `args_count` | external | — |
| `arg_get` | `Ok(value:)` external; `Err(error:)` external | — |
| `host_bytes_len` | external | — |
| `host_copy_bytes` | `Ok(value:)` internal; `Err(error:)` external | `destination` external |
| `host_utf8_len` | `Ok(value:)` external; `Err(error:)` external | — |
| `host_copy_utf8` | `Ok(value:)` internal; `Err(error:)` external | `destination` external |
| `relative_path` | `Ok(value:)` external; `Err(error:)` external | — |
| `open_read` | `Ok(value:)` external; `Err(error:)` external | — |
| `read_once` | `ReadBytes(count:)` internal; `ReadFailed(error:)` external | `destination` external, `file` external |
| `write_once` | `Ok(value:)` internal; `Err(error:)` external | `output` external |
| `exit_status` | internal | — |

For a plain result, the table cell fixes its aggregate datum. For a payload
enum, every named direct `Variant(field:)` entry fixes that exact projection
and the result aggregate is their join. An `Err(error:)` class therefore fixes
the aggregate class of its error atom; if source later matches that atom as a
payload-carrying error enum, the class seeds each of its direct projections.
Thus `CopyTooSmall(required:)` and `Utf8CopyTooSmall(required:)` are external
without pretending they are direct projections of `Result`. The four internal
payload projections are
exactly the **transfer counts**: the quantity of units one attempt moved. Each
is bounded by an argument the program chose —
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

This is R1 of §1. A future candidate that reclassifies any listed result datum
from internal to external must report that as an acceptance narrowing, not
merely a checker strengthening.

## 6. Historical v0.22 replacement deltas and verbatim anchors

Everything in this section is the abandoned 2026-08-07 exact delta against
v0.22. It is retained to explain the design's origin. It is not replacement
text for the stable active specification. In particular, the old [FN-1]
sentence below is superseded by §4: adding another leaf for the same parameter
datum already in `domain(Req)` changes diagnostic targets but does not further
strengthen caller acceptance; requiring a different payload datum at the same
position can strengthen it.

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
followed by the then-current 2026-08-07 classification table as inventory data.
That historical table is preserved in revision `5998b87`; this sentence does
not dynamically refer to the revised held table in §5 above.

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

## 7. Open bypasses and deliberate classifier boundaries

An owner should approve the gate knowing what it does not catch.

**B1 — the `requires` prologue.** [FN-8]'s final `check` is a writer-authored
trap the callee executes, and [ENT-3] S4 turns it into a body-entry fact that
the unasserted state keeps. A writer therefore moves any gated claim behind a
callee contract:

```
fn get['d](data: &'d buffer<u8>, i: own u64) -> own u8 reads('d), traps
  requires {
    let room = len(deref(data));
    let in_range = ilt(i, room);
    check in_range else trap "index outside data";
  }
{
  return deref(data)[i];
}
```

A caller passing an externally-provenanced `i` compiles and traps on hostile
input — the original defect, one call away. Closing it needs a subject-position
notion for an arbitrary `requires` condition, which the held design's single
obligation family does not supply, and the naive closure (put every parameter a
requires condition mentions into the internal-required column) over-fires
badly. `store_dynamic_length`'s real requires prologue mentions only
`literal_lengths` and `literal_count`; a mention-only closure would gate an
external `literal_count` even when the caller's
`buffer_new(literal_count, 0_u8)` allocation equality makes
`literal_count <= len(literal_lengths)` derivable in the unasserted state. That
would reject a goal the program already proves without a writer assertion.
This is O3, and it is the material one.

**B2 — trapping arithmetic.** `external_value + k` traps on overflow of
environment-chosen data. The held design attaches obligations to subscripts
only, so the gate has nothing to attach to. Closing it is the arithmetic half
of `DOSSIER.md` §2.9 and remains deferred outside the owner-selected completion
boundary.

**B3 — bound position.** `claim n: ilt(clean_i, external_count) because …`
is legal. This is the under-block `PROBE-W1.md` named when it amended the gate
to subject-position form, and it is deliberate: provenance is not integrity,
and rejecting every claim that mentions an external value rejects the truthful
relational shapes the corpus depends on. For the one obligation family this
held design attaches the case has no instances, because the bound is always
`len(P)`, internal by P3.

**B4 — a claim supporting no obligation.** Subject position is defined by an
obligation, so a free-standing `claim` over external data is ungated. Every
such claim remains outside this gate by construction.

**D1 — control and write-address implicit flow.** Section 4's hostile
`set a[external_i] = 1` witness is accepted even though the environment can
choose whether the later claim fires. A branch- or match-selected internal
literal is similarly internal. These are deliberate limits of A-only
explicit-dataflow provenance. They are not claimed as noninterference, and
they must remain visible in any activation review rather than being described
as classifier completeness.

B3 and B4 are the residue the fired-claim lifecycle of `DOSSIER.md` §2.7 owns
by design; B2 is deferred outside the owner-selected completion boundary; D1
is the selected policy boundary; and B1/O3 is a real hole in the gate's own
shape. O3 therefore still blocks
activation after the positive A-only measurement.

## 8. Installed-v0.24 frozen measurement and A-only rewalk (2026-08-09)

### Frozen basis

The rewalk preserves task 0041's installed basis: activation
`f4c7e60c47bdea620eea5a00be89ff54d7678cc9`, active-spec SHA-256
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`, and
the compilation-unit order `raw_deflate.wf`, `raw_deflate_dynamic.wf`,
`raw_deflate_dynamic_decode.wf`, `raw_deflate_boundary.wf`. Their respective
SHA-256 digests are
`c8fa0d58301e5346041c1886eaa3e277f9d3926212b6a5420e52b22eada300f0`,
`cca35bbd3c5985c1e6753e0b0ca5311be7287d2021c01b46f14506b06734fcee`,
`56c3bc84858849a27e4d493e6db0445056d36e2a7b3e864bb86d35bb22b792b7`,
and `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`.

The denominator is the whole statically checked boundary unit, not the dynamic
fixture that happens to run: 23 claims and 33 obligations. The decoder alone
accounts for 21 claims and 29 obligations; the boundary helpers add two claims
and four obligations. Stored, fixed, and dynamic paths all remain in scope.
The old root-only measurement used held-candidate SHA-256
`62f9fbb98d69777f5cacacb8f63fd4a922eed4bef6e49d7a66f71df7827fb47b`;
the A-only column below is a literal rewalk under §4, not an installed checker
export.

### Obligation and claim result

| rule | external subjects | unasserted-proven | rejected obligations | affected claims | internal subjects | canonical |
|---|---:|---:|---:|---:|---:|---:|
| old root-only P2 | 18/33 | 6 | 12 | 10 | 15 | 2/3 |
| A-only P2 | **19/33** | **6** | **13** | **11** | **14** | **3/3** |

Exactly one obligation subject changes: `destination_in_symbols`, whose
subject is `destination` from `offsets[count_index]`. The `offsets` root and
its external `count_index` selection are now both explicit P2 inputs, so the
subject is external and its claim cannot discharge the obligation in the
unasserted state. The other 32 obligation subjects retain their old class.
The three canonical sites are therefore all gated:

- `order_slot_in_offsets`;
- `destination_in_symbols`; and
- `ordered_in_symbols`.

Twelve claim declarations have external subjects. `count_slot_in_counts`
protects two obligations that the unasserted state proves, so it passes. The
other eleven declarations account for thirteen rejected obligation nodes:
the five stored-block claims, `length_symbol_in_tables`,
`match_copy_in_history`, `order_slot_in_offsets`,
`destination_in_symbols`, `ordered_in_symbols`, and
`distance_position_in_lengths`. `length_symbol_in_tables` and
`order_slot_in_offsets` each protect two rejected accesses; each other gated
declaration protects one. The remaining eleven claims and fourteen obligation
subjects are internal.

The original five boundary-bearing controls remain unchanged, with no gate in
their fifteen claims:

| program | claims | gated under A-only |
|---|---:|---:|
| `tests/programs/wfgrep.wf` | 8 | 0 |
| `tests/conformance/cases/run-sysfile-multichunk.wf` | 4 | 0 |
| `run-syshost-copybytes-toosmall-unchanged.wf` | 1 | 0 |
| `run-syshost-copyutf8-toosmall-unchanged.wf` | 1 | 0 |
| `run-syshost-copyutf8-invalid-unchanged.wf` | 1 | 0 |

### Precision and honest repair cost

Under the rule's own whole-root, flow-insensitive definition, every external
classification has a written propagation edge, so the formal false-positive
count is zero. Under a site-local path-and-time precision lens, the five
stored-block claims remain precision positives: at each claim, the indexed
header/copy offset is still a program-maintained counter, but an external write
to a sibling state field and a later external update classify the entire
`InflateState` root across paths and time. This is an explicit cost of O6, not
silently relabelled as five formal errors.

Eight of the eleven gated declarations are outside the three registered
canonical sites and cover nine obligation nodes. That noncanonical precision
spill is unchanged by A-only. Ten of the eleven gated claim families admit a
local honest repair: replace the claim with an exact value branch returning
the existing `Truncated`, `InvalidHuffmanCode`, `InvalidDistance`, or
`InvalidHuffmanTree` outcome. The newly gated `destination_in_symbols` belongs
to this local group. `distance_position_in_lengths` remains the expensive
case: `store_dynamic_length` must return `Result<unit, InflateError>` and three
callers must propagate it, or all three callers must duplicate the domain
branch. Moving the access into a `requires` check remains a cheaper bypass,
which is why O3 still blocks activation.

### Complete binding, root, and signature delta

The following is the complete internal-to-external binding/root delta from
root-only P2 to A-only P2 on the frozen unit:

- In `decode_length`, `length_base_word`, `length_base`,
  `length_extra_byte`, and `length_extra_count` change through the two
  const-table reads selected by external `length_index`.
- In `copy_distance`, `distance_base_word`, `distance_base`,
  `distance_extra_byte`, and `distance_extra_count` change through the two
  const-table reads selected by external `distance_symbol`.
- In `build_huffman_table`, `previous_count`, both later `count` bindings,
  `left`, `oversubscribed`, `incomplete`, `offset`, `destination`, and
  `destination_ok` change. The `counts` and `offsets` storage roots change at
  all three frozen calls. The function's `Ok(value:)` projection for local
  `table`, and therefore the caller's `code_table` binder selected by
  `propagate`, change; the literal and distance tables were already external
  through external `symbol_count` actuals.
- In the code-table invocation of `decode_table_symbol`, the `table` actual,
  `empty`, `count`, `first_after_count`, `first`, `symbol_index`,
  `last_length`, and `decoded` change because the table is now external.
  `ordered` was already external through decoded bits.
- At that call's success projection in `decode_dynamic`, `decoded_symbol`,
  `length_symbol`, `direct_length`, the direct `length` match binder,
  `previous_length`, `repeat_previous`, `short_zero`, and `long_zero` change as
  the code-table result propagates.

No other binding aggregate, storage root, or enum payload projection changes
class. In particular, the literal `Err` projections and conversion-error
binders do not inherit the class of a distinct `Ok(value:)` projection. The
`symbols` root does not acquire provenance merely because `destination` is an external
write address: the stored RHS `symbol` is internal. The code-table `symbols`
root remains internal; the literal/distance `symbols` roots were already
external through their external allocation sizes. `code_lengths` was already
external through external stored length values, and `literal_lengths` and
`distance_lengths` were already external through their external allocation
sizes.

The exact derived-column delta is:

| column | function / destination | added parameter-datum dependencies |
|---|---|---|
| result `Ok(value:)` projection | `decode_length` | `symbol` |
| result `Ok(value:)` projection | `build_huffman_table` | `lengths` |
| result `Ok(value:)` projection | `decode_table_symbol` | `state`, `input` through selected `ordered` |
| write | `decode_length` / `state` | `symbol` through selected `length_extra_count` and `read_bits` |
| write | `copy_distance` / `state` | `distance_symbol` through selected `distance_extra_count` and `read_bits` |
| write | `copy_range` / `destination` | `from` |
| write | `copy_distance` / `out` | `state`, `input`, `distance_symbol` |

`read_bits` gains another state-offset predecessor edge but no write-column
membership change because that parameter was already present. No other result
or write dependency changes. In particular, all three functions' error payload
projections retain their held dependencies; an external `Ok(value:)` projection
changes the result aggregate by join but does not taint an `Err(error:)`
projection. No write dependency is inferred from a target address alone.

All frozen parameters named below are non-payload types, so their sole
parameter datum has selector `plain`, and the 19/33 and 14/24 measurements are
unchanged by making
the datum key explicit. The internal-required relation changes in two exact
ways. `build_huffman_table` already required the `(lengths, plain)` datum for
the two exact `order_slot_in_offsets` obligation occurrences; it now also
contains
`((lengths, plain), destination_in_symbols leaf)`. That datum was already in
`domain(Req)`, so each affected build argument remains one event while its
finite `Targets` set gains the new leaf. Separately,
the `(distance_symbol, plain)` datum of `copy_distance` now reaches the existing
`match_copy_in_history` leaf through the selected distance-base/extra values
and `source_index`; this is a genuine new domain member.

The frozen call-site projection remains fourteen rejecting call statements.
External required-argument atoms increase from 21 to **24**: one new `table`
actual at the code-table `decode_table_symbol` call, plus the external
`distance_symbol` actual at each of two `copy_distance` calls. The other two
table-decode calls already had external `state`, `input`, and `table` actuals.
There is no other `Req` domain change.

The relation design was also checked against hostile finite cases: one
parameter datum protecting multiple leaves still emits one argument event; the
same source obligation at two concrete generic instantiations remains two leaves;
ordinary, recursive, and mutually recursive composition reaches the same least
fixed point in any visit order; a purely recursive component with no direct
leaf remains empty; and an unconditionally external actual is a local
violation rather than a synthetic relation position. Current compiler metadata
does not yet retain this relation or its witness paths.

### Disposition

The A-only rewalk closes the two stage-5a-R review defects: canonical coverage
is 3/3 with exactly the predicted single new obligation subject, and PRV-2 now
has a finite deterministic parameter-datum-to-leaf meaning. The design remains
held
behind stage 7/O3. Nothing in this result activates PRV-1, PRV-2, or PRV-3 or
changes the active specification, compiler, or protected corpus.

## 9. Historical v0.22 verification record

The checks below ran on 2026-08-07 against the then-active v0.22 spec at
`8f91ede`. They verify the historical delta retained in §§2 and 6; they do not
verify the revised A-only/PRV-2 design text in §4 against active v0.24.

1. **Anchor exactness.** The historical work matched ten verbatim anchors —
   nine rule sites plus the §18 heading — as fixed strings against
   `spec/kernel-spec-v0.22.md`. Every one matched exactly one line.
2. **Grammar containment.** The v0.22 spec's fenced blocks span lines 98–126,
   130–139, 143–165, 169–182, 660–662, 706–740, 766–826, 830–842, and
   1050–1093. Every modification site (lines 386, 400, 846, 1000, 1004, 1008,
   1016, 1044) lies outside all of them, so no site touches a grammar
   production, a terminal, the operation table, or the worked example.
3. **Additive assembly.** The historical draft's three [PRV] rules, [SYS-2]
   table, and changed §18 heading were appended to a scratch copy and the native
   verifier run on it: **65 productions, 75 decisions, 76 terminal predicates,
   exit code 0**, identical to that v0.22 baseline. This is evidence that the
   old draft did not alter grammar, not an exact-byte assembly for v0.24.

### Historical FLOOR-5 anchor note

Verified textually 2026-08-07 against `spelling-relief-candidate.md`. **Exactly
one of the historical draft's ten anchors would have been disturbed if that batch activated
first:**

- **[ENT-6] second site** — the mechanical-fix sentence — *is* FLOOR-5's only
  [ENT-6] site. It becomes "in canonical ANF, one `let` binding `len(P)`
  followed by one `claim` on, or `if` over, the admitted comparison [CLM-1,
  ENT-3]". Both the historical draft's anchor and its replacement text would have had to be
  re-taken: the anchor against the respelled bytes, and the replacement's
  closing phrase from "the `match` half is the fix" to "the `if` half is the
  fix".

The other nine survive verbatim, each for a checked reason: FLOOR-5 modifies
neither FN-1, OP-4, SYS-2, CLM-1, nor ENT-1, and its seven [ENT-3] sites are
S1's origin clause, S1's establishment sentence, S4, S5, S6, S7, and S9 — not
the "Nothing else is a fact" sentence the historical draft took. [ENT-6]'s first
site (the discharge condition) is likewise untouched by FLOOR-5.

Those anchors now have no activation procedure. A future selected proposal
must edit `spec/kernel-spec.md` directly under the current exact-byte workflow,
run the native grammar verifier on the complete proposed stable-spec bytes,
and archive only the outgoing active bytes at activation. The historical
v0.22 strings are never fuzzy-matched onto the stable specification.

This held-evidence revision modifies no active specification, compiler,
protected corpus, generated datum, or approval record. O3 remains the required
precondition for any future provenance-gate proposal.
