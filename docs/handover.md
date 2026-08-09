# Handover — 2026-08-09

Written because a session ended mid-track. It is not a task list; the task list
is `docs/roadmap.md` and `docs/current-plan.md`. This explains **why the work is
where it is**, so whoever picks it up knows what they are in the middle of
rather than only what is left.

Delete this file once its content has been absorbed into the outline and the
plan.

---

## 1. Where this all started, and why everything traces back to it

The whole arc began with a criticism of Whitefoot's trap semantics: *a language
whose partial operations trap means critical applications crash at unpredictable
moments*. The owner had no good answer and asked for one.

The first instinct — "convert traps into error returns" — was argued down over
several rounds and the arguments are worth keeping, because they will be
re-proposed:

- **`traps` is contagious.** Once one function can trap, every caller inherits
  the row, and in a large program that is nearly every function. A property that
  is universally true carries no information.
- **A caller cannot know when a callee traps.** `requires` clauses can do
  arithmetic on inputs, so whether a call traps depends on values the caller may
  not have.
- **Result-everywhere is not obviously better.** It forces the writer to author
  an error arm for a condition that is *impossible when the code is correct*,
  and a catchable internal error lets a stuck writer swallow a violated
  invariant at the innermost frame.
- **The classification cannot be delegated to judgment.** The owner's point:
  you cannot trust a human or an AI to decide "this is a trap, that is an
  error", and the probability that a given predicate fails is unverifiable.

What came out of that is **obligation discharge**, and it is the design the
project is now executing:

> Every partial operation carries a **proof goal**. At each use site the goal is
> either **discharged** from facts the checker already has, or carried by an
> explicit **`claim`** — named, with a `because` string — or the program is
> **rejected** with the residual obligation printed.

Three consequences that make it answer the original criticism:

1. **`claim` is the only writer-reachable trap source.** Every remaining trap in
   a program is a named, justified, ledgered assertion. "Where can this crash?"
   becomes a greppable question.
2. **The classification is modal, not probabilistic.** A predicate whose falsity
   is reachable in a correct program is a *value* (return a `Result`); one whose
   falsity means the program's own reasoning is broken is a *trap*. Probability
   never enters, which is what makes it decidable by rule instead of by taste.
3. **`traps` stops saturating.** Claims live in callers, so a leaf function with
   a goal but no claims has a clean effect row. The row regains signal.

The design is frozen in
`research/investigations/obligation-discharge/DOSSIER.md` together with the
falsifier it was tested against.

## 2. What has actually shipped on that track

Design → specification → implementation → measurement is **complete**. This is
further along than the day's noise suggests.

- **v0.21** shipped the `claim` construct, the L0 entailment fragment (ENT-1..6),
  and caller-side discharge for index obligations.
- **v0.22** settled the index surface (`a[i]` subscripts, element-type
  derivation).
- **The implementation** landed across four tasks now in `docs/done/`: the ENT
  engine, its fact sources, the discharge flip plus claim semantics, and a
  **preregistered acceptance run**.
- **The acceptance run measured**: `utf8parse` landed on its predicted buckets
  exactly; `sha256` came in one claim over; **`deflate` diverged badly** — 5 of
  29 sites proven against 17 of 30 predicted, and 21 claims where about 8 were
  expected.

That divergence is the thread everything else hangs from.

## 3. Why the divergence happened, and what fixes it (this is the live work)

The dominant cause was isolated to **ENT-5's loop rule**. Read the rule and the
defect is visible in six lines:

```
loop @scan {
  check ilt(ordered_symbol, len(code_lengths)) else trap "…";
  let v = code_lengths[ordered_symbol];   # discharged — no runtime check
}
```

Move one `return` into the body and the same index stops discharging:

```
loop @scan {
  check ilt(ordered_symbol, len(code_lengths)) else trap "…";
  if done { return unit; }                # ← only this line is new
  let v = code_lengths[ordered_symbol];   # now rejected
}
```

**Why.** ENT-5 discards, at each iteration head, every fact whose support any
kill event inside the body *may* kill. Kill event (d) is *an edge leaving the
lexical scope of a support binding* — and a `return` leaves the scope of **every
binding in the function**. So one `return` anywhere in a loop discards every
pre-loop fact, including `requires` axioms and allocation-length equalities that
**no execution can invalidate**. An array's length cannot change; a `return`
certainly cannot change it.

`break` alone does not trigger it, because `break` leaves only the loop's scope
while the supports live outside it. `sha256` was unaffected because its three
loops contain no `return`. The reach is every deflate function with a loop that
returns inside it — eight of them.

**The fix** scans only the kill events an execution can carry into a *later
iteration head of the same loop*. A `return`, a `break`, or a `propagate` error
edge never reaches the next iteration head, so counting its kills there removes
facts nothing can observe as false.

The candidate is
`governance/spec-evolution/ent5-loop-fix-v024-candidate.md`, owner-approved
2026-08-07 and **re-cut against v0.23** on 2026-08-09. Its digest is
`9afd7fd57390b688ba0a2c7d91573d9d2cd3cbb8a8244a440e9120b73f73481e` — recompute
it, do not trust this line.

## 4. Why a spelling batch consumed a whole day in front of it

FLOOR-5 (the v0.23 "spelling relief" batch) is a **companion** to the trap work,
not a detour, and it had to land first for one concrete reason: the approved
**stable-specification-filename** switchover is sequenced onto *the first
activation with no EBNF change*, and that is ENT-5. v0.23 changes EBNF; ENT-5
does not. So v0.23 had to activate the old way before ENT-5 could carry the file
model.

v0.23 **is activated** (`spec/kernel-spec-v0.23.md`, chain at 15 links). What it
changed:

- deleted written type arguments from operations outside a small retained class;
- deleted the mode-and-type annotation from every `let`;
- replaced the Bool-scrutinee `match` with `if`/`else`;
- made the hot integer arithmetic infix.

**The owner cancelled the infix comparisons** mid-batch, and the reasoning
matters because it will come up again. The candidate had made four of six
comparisons infix (`== != <= >=`) while `<` and `>` stayed named calls, because
`<` in expression position collides with a type-argument list. The owner rejected
the asymmetry. Investigation showed the collision is irreducible only for a
*user-generic call* (`f<i32>(x)` versus `a < b`), and no bounded lookahead
resolves it because types nest. Cancelling all four gave a clean **grammar-class
rule** — arithmetic is infix, comparison is a call — and made the delta *smaller*
(64 → 62 modification sites), because two replacements became byte-identical to
v0.22 and stopped being modifications at all.

## 5. What is on `main` right now, honestly

`make check` **exits 2**, with two failures, both diagnosed:

1. **`every_canonical_corpus_file_re_renders_to_itself`** asserts that only the
   FORM-2 negative cases may be non-canonical, naming exactly two files. That
   premise is now false: **20 files do not derive under the active grammar**.
   They are the tail the migration tool refused — negative cases for FORM-3,
   FORM-4, FORM-5 and GRAM-9 whose whole subject is that they do not parse. The
   fix is the same one this batch applied twice elsewhere: **derive the excluded
   class by rule from the manifest instead of maintaining a hand list**. This is
   a bounded slice and nobody has started it.
2. **`own3-pos-outlives-store`** in the conformance adapter (389 / 1 / 13). This
   one is deliberate: the approved v0.23 bytes *name* it. See §6.

A third failure was found and fixed while writing this: the v0.23 activation
added the 15th chain link without bumping the literal in
`recorded_chain_ends_at_the_embedded_specification`. **It sat unseen because two
readers each checked one `test result` line and `make check` runs several
crates.** If you take one operational lesson from this handover, take that one.

## 6. Open language questions, with the reason each is open

None of these blocks ENT-5. Each is recorded with a trigger.

**A3 removed an ability nobody noticed.** v0.22 admitted
`let q: &'s i32 = &'r a;` — an annotation naming a *destination region* the
right-hand side did not produce. A3 deletes the annotation, so v0.23 cannot state
a destination region for a local binding. A sweep found **exactly one** such site
in 1954 annotated bindings. The approved bytes name it as "an expressible form
removed, whose effect on the accepted-program set is not established" — *not* as
a fourth narrowing, because an outer-region borrow satisfies an inner
destination by outlives, so the equivalent program may always be writable. That
one site is `own3-pos-outlives-store`, which is why it still fails.

**A generic that returns its own parameter has no legal form at any copy type.**
`fn pick<Held>(value: own Held) -> own Held` works at `box<i32>` and is
uninstantiable at `i32`, `Bool`, or any copy type: the body is checked
generically (unknown `Held` ⇒ affine ⇒ `move` required) *and* at the
instantiation (`move` of a copy is an error). Two defensible rules, no legal
spelling between them.

A nine-agent adversarial investigation
(`research/investigations/move-on-copy/REPORT.md`) established that the obvious
fix — allowing `move` on copy values — **is not a one-clause deletion** (OWN-1
defines "consuming use" only for affine places, so deleting the prohibition
leaves `move` with no defined effect), that its minimal form **leaks memory**
(one generic body frees 2 at `box<i32>` and 0 at `i32`), that it **does not fix
multi-use bodies**, and that a strictly better option exists that nobody had
considered: extending FN-2's bound vocabulary with a **copy marker**. Measured
demand today is **zero** — every generic in real programs is already bounded.
**The owner ruled: wait.** The trigger to revisit is the first real program
needing one body at both a copy and an affine instantiation.

Note also that "just fix the diagnostic" is *not* available here: both mechanical
fixes are stated in OWN-1's own normative bytes, so a misleading diagnostic can
only be corrected by amending the rule. That is a real cost of putting repairs in
normative text and it has now bitten twice.

**O11 (boolean composition) is undrafted, not approved.** What carries approval
is its queue position. It flips a declared conformance verdict, touches CLM-2
(whose worked example the correction makes false), and its depth-1 form would
read no corpus site at all because the grammar forbids nesting connectives except
through a `let`. It was **de-paired from ENT-5** so the file-model switchover
would not ride behind an undrafted rule change.

**OWN-3's holder-scope predicate** tests scope-parent identity where the rule
means *outlives*. The slice path already accepts the shape the scalar path
forbids, so the question inverted: not "is widening safe" but "why is the scalar
path narrower". Not authorized; a check is removed only by proof.

## 7. What this session learned about how to work here

These are in `docs/WORKFLOW.md` under "Evidence discipline" and "The failures
that look like success". They are here because they cost real time to find.

**The defects that hurt are the ones that look like success.** A conformance case
that passes while testing nothing; a check that cannot fail; a transform verified
against its own output; an operation performed against a baseline that no longer
describes reality. Nothing that watches for *failure* sees any of them. A
deliberate sweep found at least two green conformance cases testing nothing, and
three more testing a different concern than they record — and the asymmetry is a
property of our process: cases whose subject died turn red if they were negative
and green if they were positive, so ordinary work catches half of them.

**Prefer the observation that separates two hypotheses** over one consistent with
the hypothesis you hold. Before running a check, ask what result would make you
believe the *other* thing; if no result would, the check is decorative.

**A remembered conclusion feels exactly like a measured one** and nothing in the
feeling distinguishes them. This is why the evidence rule has to be mechanical
rather than a matter of care.

**Green figures mean less than they look.** "128/128 rules covered" counts
*naming*: a rule is covered when any case merely lists it, whether or not any
verdict cites it. 83 of 128 have no negative at all, so a rule that silently
stopped rejecting would not be caught. That is not a defect tally — many of those
rules have no negative form — but it is not coverage either.

**Tooling traps that produced wrong measurements here:** `grep -E` treats `\b` as
a literal `b`; the installed `grep` is ugrep, where `-oc` counts occurrences
while GNU `grep -c` counts lines; short identifiers matched as substrings are
mostly artefacts (`ine` matches inside *retained*, *scrutinee*, *multi-line*);
and exit codes read through a pipe report the pipe's last command.

## 8. The immediate next three things

1. **Finish ENT-5.** The archive-integrity gate must be taught the stable-file
   model *first, in its own commit, proven to fail in both directions* — a gate
   that has never failed is decorative. Then the owner approves the candidate's
   bytes, then one activation commit that also carries the filename switchover.
   `docs/ongoing/0042` has the step-by-step and the blocker that stopped the
   first attempt.
2. **Re-derive the canonical-corpus exclusion by rule**, closing the first
   failure in §5.
3. **Re-run the acceptance measurement** once ENT-5 is active. That is the payoff
   and the thing the whole trap track has been waiting for: deflate should move
   from 5-of-29 toward the predicted 17-of-30, and those 21 claims should fall.
   **Claims are the only trap source, so that number falling *is* the answer to
   the criticism this all started from.**
