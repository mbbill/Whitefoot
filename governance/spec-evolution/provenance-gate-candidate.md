# Provenance and the subject-position taint gate — specification-change candidate

Status: CANDIDATE, **HELD FOR MEASUREMENT** (2026-08-07, lead ruling).
Non-authoritative and not proposed for approval. This document is the complete
delta against the exact text of the active `spec/kernel-spec-v0.22.md`
(installed 8f91ede; SHA-256
`b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8`; roadmap
revision 18). It implements `DOSSIER.md` §8 item 5, priority-advanced by the
owner on the acceptance evidence.

Split out of `semantics-v024-candidate.md` (drafted and withdrawn 2026-08-07).
Its companion — the [ENT-5] loop-rule fix — is
`ent5-loop-fix-v024-candidate.md`, owner-approved and ready for activation. The
two share no rule text; that batch does not depend on this one and must not
wait for it.

## 0. Why it is held: the gate has no live instance

**The gate cannot fire on the sites that motivated it, and the corpus contains
nothing else for it to fire on.**

`ACCEPTANCE.md` advanced this item because three canonical-Huffman sites in the
deflate decoder became aborting claims where the design demands recoverable
`Err` branches: `decode_table_symbol`'s `ordered_in_symbols`, and
`build_huffman_table`'s `order_slot_in_offsets` and `destination_in_symbols`.
Those are exactly the shape the gate is for — an index derived from decoded
file content.

They stay legal under this candidate. `tests/programs/raw_deflate_vectors.wf`
builds the compressed input with `make_dynamic_input()`, an ordinary heap
buffer, and its `main` is unlabelled, so the unit is not system-admitted
[SYS-3] and **no value in any deflate program has a boundary origin at all**.
Provenance is empty there; the gate has nothing to attach to.

The rest of the corpus gives the same answer for a different reason. Across 144
claims, only five programs have a boundary, and in every one the claim subjects
are counter chains, literals, or transfer counts — never environment-chosen
data. Measured rejections: **zero** (§7).

So the semantics below are, as far as any measurement here can tell, correct
and inert. The evidence that the gate does the thing it was advanced for does
not exist yet, and the candidate says so rather than inferring it from the
design. §8 records the measurement condition that would change this.

Two further reasons the hold is the right call, both from the drafting itself:

- The gate **narrows** acceptance, unlike its companion. A narrowing shipped
  without a live instance buys nothing and risks the false-positive class
  `PROBE-TAINT.md` measured at zero.
- The gate has four named bypasses (§6), one of them material: any gated claim
  moves one call away into a callee's `requires` contract and compiles.

## 1. What is ruled and what is open

Ruled by the lead 2026-08-07, on the drafted reasoning and measurements:

- **R1 (was O2) — the provenance class of the four transfer counts: internal.**
  The program supplies the bound (`capacity`, or `count` for `write_once`) and
  [SYS-9] with [ENT-3] S10 already fix `count <= bound` as a declared operation
  contract, so only the position within `[0, bound]` is environment-chosen —
  the same standing a length has under T1. The strict alternative was measured
  and rejected: it illegalizes five of wfgrep's eight claims, every one of them
  a relation no environment can falsify, since the program computes
  `room = 4096 - carry` and passes it as `capacity`. Those are precisely the
  false positives `PROBE-TAINT.md` reported as zero. This ruling binds future
  versions too: reclassifying a result from internal to external is a
  narrowing (§5).
- **R2 (was O4) — the gate attaches to the obligation, not the claim.** Subject
  position is defined only by an obligation, so the obligation-side form is
  total, needs no claim-to-obligation mapping, and is decided by two closures
  of one state. The S2 (`check`) exclusion is kept: without it the gate is a
  one-line bypass. Cost accepted: the diagnostic lands at the subscript, and
  the drafted message names the supporting claim as well.

Open, and deliberately unresolved:

- **O3 — the `requires` bypass (B1 of §6).** The material hole. Ship the gate
  with it named, or hold until a subject-position notion exists for an
  arbitrary `requires` condition? The naive closure over-fires badly. Lead
  ruling: keep open; this is a reason the batch is held, not a detail to settle
  in drafting.
- **O5 — derived versus written provenance (drafted: derived).** A written
  annotation would be a grammar change, would hand a trust-bearing declaration
  to the untrusted writer, and would need a variance rule the moment indirect
  calls exist. Recorded as a rejected alternative, not omitted.
- **O6 — flow-insensitive per binding (drafted: yes).** A flow-sensitive
  judgment would keep a binding internal before its first external write, at
  the cost of a join rule, a kill rule, and a loop rule of its own — a second
  dataflow system beside [ENT-3] — for a precision gain no corpus site needs
  under R1. Revisit only if R1 is ever reopened.
- **O7 — whether [DIAG-2] should retain the provenance column.** The claim
  ledger of `DOSSIER.md` §2.7 wants provenance per row. The ledger is not in
  this batch, so [DIAG-2] is untouched; adding retention later is additive.

Noted, not open here: [FN-8]'s foreign-entry adapter still executes the
`requires` prologue with "trap semantics unchanged in this version", while
`DOSSIER.md` §2.8 requires the adapter's failure path to follow the boundary's
error protocol, because foreign arguments are external by definition and the
environment has no trap authority. The gate makes that inconsistency explicit.
It has zero instances (the gated family is a stub with no call form), so this
candidate does not touch [FN-8]. **Recorded as a finding for the
requires-as-goal batch** (`DOSSIER.md` §8 item 7), per the lead's ruling.

## 2. Proposed version-header paragraph

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

None. This batch adds, removes, and reshapes no production, terminal
predicate, token form, operation-table row, or source construct. No
modification site lies inside a fenced grammar block, and the three new rules
are prose rules in §18. §9 records the mechanical confirmation, including a
verifier run on an assembly carrying every new byte.

Provenance is deliberately **derived, not written** (O5).

## 4. New rules

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
the call site, in the caller's own vocabulary, is the same placement [OP-4]
discharge already uses: the call site is the only point that knows where its
data came from.

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

## 5. [SYS-2] provenance classification (declaration data)

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

This is R1 of §1. It binds future versions: reclassifying any result from
internal to external is a narrowing, not a checker strengthening.

## 6. Modified rules (complete replacement deltas, verbatim anchors)

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
followed by the classification table of §5 as inventory data.

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

## 7. The four bypasses, with witnesses

An owner should approve the gate knowing what it does not catch.

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
requires condition mentions into the internal-required column) over-fires
badly: it would gate `store_dynamic_length`'s `position` parameter, whose
contract is about `literal_count` and cannot trap on `position` at all. This is
O3, and it is the material one.

**B2 — trapping arithmetic.** `iadd.trap<u64>(external, k)` traps on overflow
of environment-chosen data. This version attaches obligations to subscripts
only, so the gate has nothing to attach to. Closing it is the arithmetic half
of `DOSSIER.md` §2.9, explicitly deferred there pending [OP-4] experience.

**B3 — bound position.** `claim n: ilt<u64>(clean_i, external_count) because …`
is legal. This is the under-block `PROBE-W1.md` named when it amended the gate
to subject-position form, and it is deliberate: provenance is not integrity,
and rejecting every claim that mentions an external value rejects the truthful
relational shapes the corpus depends on. For the one obligation family this
version attaches the case has no instances, because the bound is always
`len(P)`, internal by P3.

**B4 — a claim supporting no obligation.** Subject position is defined by an
obligation, so a free-standing `claim` over external data is ungated. Every
claim in the current corpus supports at least one obligation, so B4 has no
instances today, but nothing prevents one.

B3 and B4 are the residue the fired-claim lifecycle of `DOSSIER.md` §2.7 owns
by design; B2 is a scheduled later batch; B1 is a real hole in the gate's own
shape.

## 8. Corpus measurement (2026-08-07) and the condition for un-holding

**Under this candidate's rules, zero claims become illegal.**

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
counter chain seeded by a literal or by a transfer count, internal under R1.
The values wfgrep does hold externally — `arguments` from `args_count`, the
pattern and input bytes, every `IoError` — reach only branch guards and
`ieq<u8>` comparisons, never a subject position. That reproduces
`PROBE-TAINT.md`'s finding against the real rule rather than by inspection:
wfgrep compares bytes but never indexes by one.

**Under the rejected strict alternative** (every system result external,
including the four transfer counts) the same walk gives five rejections, all in
wfgrep. `read_once`'s count `taken` flows to `available`, thence to
`terminator`, `scan`, `probe`, `spot`, `source_index`, `tail`, and `carry`, so
`carry_in_input`, `probe_in_input`, `spot_in_input`, and
`shift_read_in_input` become externally subjected; and `copy_range` is called
with `from: scan`, so its `from` parameter enters the internal-required column
and `copy_read_in_source` fails at that call site under [PRV-2].
`ahead_in_pattern` and `shift_write_in_input` survive (their subjects `ahead`
and `moved` are literal-seeded), as does `copy_write_in_destination`. Recorded
so R1 can be reopened against a real number rather than re-argued.

### The measurement condition

`docs/planned/0037-deflate-boundary-driver.md` is the registered task for
steps 1 and 2 below and cites this candidate as its authority; the two records
are cross-linked deliberately, and the ordering note at the end of this section
is that task's dependency on the companion [ENT-5] batch.

This candidate leaves HELD until **all** of the following hold, in order:

1. A deflate driver is wired through a real boundary: the compressed input
   reaches `decode_dynamic` from `read_once` under a kind-declaring `command`
   entry [FN-7, SYS-3], rather than from `make_dynamic_input()`. This is a
   corpus change, not a language change, and it needs its own plan authority.
2. The gate is measured on that program: which obligations become externally
   subjected, and specifically whether the three canonical-Huffman sites —
   `decode_table_symbol`'s `ordered_in_symbols`, `build_huffman_table`'s
   `order_slot_in_offsets` and `destination_in_symbols` — are among them.
3. The result is recorded either way. If the three sites are gated, the gate
   has done the thing it was advanced for and the batch returns for approval
   with a live instance. If they are not, the reason is a finding about the
   drafted propagation rules — most likely the laundering path through the
   canonical-Huffman table construction — and the rules return to review, not
   the corpus.
4. The false-positive count on the same program is reported alongside. A gate
   that catches the three sites but also rejects a dozen truthful counter
   claims is not ready, and `PROBE-TAINT.md`'s zero is the standard it is held
   to.

Steps 1 and 2 are cheap relative to activating a narrowing rule with no
instance. Note that step 1 also depends on the companion [ENT-5] fix: the
deflate path's discharge is currently dominated by the loop-rule defect, so
measuring the gate before that lands would attribute to provenance what the
loop rule caused.

## 9. Verification record

All checks run 2026-08-07 against the active spec at 8f91ede.

1. **Anchor exactness.** Each of the ten verbatim anchors quoted in §4 and §6 —
   nine rule sites plus the §18 heading — was matched as a fixed string against
   `spec/kernel-spec-v0.22.md`. Every one matches exactly one line. No anchor
   is paraphrased and none is ambiguous.
2. **Grammar containment.** The active spec's fenced blocks span lines 98–126,
   130–139, 143–165, 169–182, 660–662, 706–740, 766–826, 830–842, and
   1050–1093. Every modification site (lines 386, 400, 846, 1000, 1004, 1008,
   1016, 1044) lies outside all of them, so no site touches a grammar
   production, a terminal, the operation table, or the worked example.
3. **Additive assembly.** The only new bytes this candidate introduces are the
   three [PRV] rules, the [SYS-2] provenance table, and the changed §18
   heading; the nine rule sites are prose-for-prose replacements. Those new
   bytes were appended to a scratch copy of the active spec and the native
   verifier run on it: **65 productions, 75 decisions, 76 terminal predicates,
   exit code 0** — identical to the baseline for the unmodified spec. The
   grammar impact of this batch is nil, mechanically confirmed rather than
   asserted. The complete ten-anchor assembly is produced by the activation
   task, whose byte comparison is the authoritative one.

### Anchor re-take against the FLOOR-5 spelling batch

Verified textually 2026-08-07 against `spelling-relief-candidate.md`. **Exactly
one of this candidate's ten anchors is disturbed if that batch activates
first:**

- **[ENT-6] second site** — the mechanical-fix sentence — *is* FLOOR-5's only
  [ENT-6] site. It becomes "in canonical ANF, one `let` binding `len(P)`
  followed by one `claim` on, or `if` over, the admitted comparison [CLM-1,
  ENT-3]". Both this candidate's anchor and its replacement text must be
  re-taken: the anchor against the respelled bytes, and the replacement's
  closing phrase from "the `match` half is the fix" to "the `if` half is the
  fix".

The other nine survive verbatim, each for a checked reason: FLOOR-5 modifies
neither FN-1, OP-4, SYS-2, CLM-1, nor ENT-1, and its seven [ENT-3] sites are
S1's origin clause, S1's establishment sentence, S4, S5, S6, S7, and S9 — not
the "Nothing else is a fact" sentence this candidate takes. [ENT-6]'s first
site (the discharge condition) is likewise untouched by FLOOR-5.

Re-verification procedure for the activation task, whichever order obtains:
re-run check 1 above against the then-active spec before applying any delta. A
candidate whose anchor no longer matches exactly one line stops for review
rather than being fuzzy-matched.

No file under `spec/`, `docs/`, `tests/`, or `compiler/` is modified by this
candidate.
