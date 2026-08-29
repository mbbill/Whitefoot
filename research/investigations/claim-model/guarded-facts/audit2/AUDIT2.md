# Round-2 re-audit of `[ENT-5.G]` — guarded facts

Target: `wf-0111-guarded-facts/DRAFT.md` (1,869 lines, round 2).
Round-1 audit: `wf-0111-guarded-facts/audit/AUDIT.md` (740 lines) and the
`wf_b79ed153-b94` journal's second result line — both enforced below.
Baseline: `research/investigations/claim-model/DESIGN.md` and
`spec/kernel-spec.md` on `batch/0106-claim-model-design`, read only;
`git status --porcelain` in that worktree is empty before and after.
Every probe compiled with `wf-0111-guarded-facts/target/release/whitefootc`;
`audit2/run.sh` reproduces every verdict.

## Verdict

**REFUTED**, on a memory-safety witness in the formation clause itself, with a
cheap and identified repair.

`audit2/probes/c01_arm_writes_flag.wf` — compiled, **REJECT today with exactly
`[OP-4] residual: x < len(deref(data))`**, so the only premise it lacks is the
drafted guarded release — is accepted by round 2 and reads index 50 of a 4-byte
buffer. `audit2/probes/c01c_joined_control.wf` is the identical program with one
literal changed from `50` to `3` and **ACCEPTs**, so the discharge arithmetic is
machine-checked and the sole difference between an accepted program and the
refuted one is whether `x <= 3` is a joined fact or a released guarded fact.
`c02_negative_key_arm_writes.wf` is the same break through clause (b).

This is **not** the projection route (deleted, and D1 is right), **not** the
key-extension route (repaired, and `r09` is a real finding), **not** the
support ruling (D2 is right), **not** the record route (D3 is right and I could
not break it), **not** the union, **not** the loop seam and **not** the
monotonicity argument. Those survived every attack below. It is
`[ENT-5.G2]` clauses **(a) and (b)**, which round 2 did not touch, and it is the
*same mechanism as `r09`* one nesting level shallower — where the repair round 2
wrote into `[ENT-5.G3]`(d) does not reach.

Round 2's §0.4 says of `r09`: *"the condition in the second paragraph of (d) is
the repair, and it is the minimum one."* It is not the whole one. §0.4 also says
this is *"the second time this design has shipped a rule whose soundness
paragraph was restated instead of re-derived."* It is the third: `[ENT-5.G6]`'s
*"The last member"* paragraph asserts a premise that only clause (a′) enforces,
and (a) and (b) were written without it.

A second, independent finding is a **reading defect in the `r09` repair itself**
(`b01`), which decides memory safety and which the draft nowhere settles.

Both are repairable inside the drafted frame by one instrument, and §9 states it.

---

## 1. THE REFUTATION — `[ENT-5.G2]`(a) and (b) never check that the arm left the flag alone

### 1.1 The witness

`audit2/probes/c01_arm_writes_flag.wf`, compiled, **REJECT** with exactly
`[OP-4] residual: x < len(deref(data))`:

```whitefoot
fn broken['d](data: &'d buffer<u8>, f0: own Bool) -> result: own u8 reads(data) contract {
  define room = len(deref(data));
  requires ige(room, 4_u64);
} {
  let f = f0;
  let x = if f {
    let three = 3_u64;   give three;
  } else {
    let yes = True();
    set f = yes;                    // the ELSE arm makes the branch flag true
    let fifty = 50_u64;  give fifty;
  }
  let out = 0_u8;
  if f { let b = deref(data)[x]; set out = b; }
  return out;
}
```

Called `broken<'r>(data: &'r input, f0: no)` on a 4-byte buffer.

### 1.2 Hand execution of the round-2 rules

**Formation, `[ENT-5.G2]`(a) at the `value_if` continuation `C`.** `chain(C)` is
empty. The condition `f` is a place datum, hence a direct goal origin
(`g06` ACCEPT pins the shape). `A` is the then arm's **exit** state — §2.2's own
words, *"the closed states on the then and else (or false) edges as `[ENT-5.P0]`
step (4) leaves them"* — holding `+f` and, from the `value_if` delivery image,
`x = 3`. `B` is the else arm's **exit** state: `[ENT-3.S1]` established `-f` at
the else *entry*, `set f = yes;` is a `[SET-1]` commit whose resolved target
overlaps `support(-f) = {f}`, so kill (a) **removes `-f` on that edge**; `B`
delivers `x = 50`. `J` therefore gives `3 <= x <= 50`, and
`diff(A, J) ∋ x - Z <= 3` — a strict improvement, by the same delivery clause
DRAFT §5.7 uses to form a01's entry. So:

    key   [ +f ]
    delta { x - Z <= 3 }

Clause (a′) forms **nothing**: it requires `A ⊢ +S` and `B ⊢ -S`, and `B` does
not derive `-f`. **(a′) carries the condition; (a) and (b) do not.**

**Transport, `[ENT-5.G4]`.** `support(+f) = {f}` by D2; `support(x - Z <= 3) =
{x, Z}`. Nothing after `C` writes `f` or `x`. The entry is live at the use.

**Carry-out, `[ENT-5.G3]`(d): not involved.** The entry is not formed inside an
arm and is present on the continuation, not on one edge of a later branch. Round
2's `r09` repair is never consulted.

**Release, `[ENT-5.G5]`.** At `if f`, `[ENT-3.S1]` publishes `+f` at the then
entry. The single route is satisfied by identity. Step (2′) adds `x - Z <= 3`.

**Discharge.** `x <= 3` with `room >= 4` closes `x < len(deref(data))` in one
`[ENT-4]` step. **ACCEPTED.**

**Execution.** `f0 = False`, so the else arm runs: `x` is 50 and `f` becomes
true. The second `if f` **is** taken. `deref(data)[50]` on a 4-byte buffer.
Memory corruption from an accepted program, no `unsafe`, no `claim`.

### 1.3 Both premises machine-checked, not asserted

| probe | verdict | what it pins |
| --- | --- | --- |
| `c01_arm_writes_flag.wf` | **REJECT** `[OP-4] x < len(deref(data))` | every step of the program is legal today; the one premise the read lacks is the guarded release |
| `c01c_joined_control.wf` | **ACCEPT** | c01 with the else arm delivering `3`: with `x <= 3` joined and `room >= 4`, the subscript discharges. The arithmetic of c01's discharge is confirmed |
| `c02_negative_key_arm_writes.wf` | **REJECT**, same residual | the same break through clause **(b)**: the then arm writes the flag false and the `[-f]` entry releases on the else edge of the later branch |

c01 = c01c with one literal changed from `3` to `50`. As in round 1's a01/a03,
the only difference between an accepted program and a refuted one is whether the
fact is joined or released.

**No prerequisite is in the way.** The delta comes from the `value_if` delivery
clause, so neither P-S5L nor P-S5B is needed: c01 refutes `[ENT-5.G]` alone, on
top of today's checker. The same program written with `set x = …` needs P-S5L,
exactly as flagship A's nine sites do, and breaks the same way.

### 1.4 Where `[ENT-5.G6]` breaks, exactly

`[ENT-5.G6]`'s first paragraph:

> *The last member.* `s_k` is either `±G` for the branch's own direct goal `G`
> (clauses (a), (b)) or `±S` for a `Bool` place the arm commits (clause (a′)). In
> both cases the clause forming `ε` established that `s_k` holds on the edge whose
> difference `δ` is, **and that its complement holds on the other reaching edge** —
> for (a) and (b) by `[ENT-3.S1]`, for (a′) by the clause's own condition.

The emphasised half is **false for (a) and (b)**. `[ENT-3.S1]` establishes `-G`
at the else *block entry*; `A` and `B` are the arm *exit* states, and every kill
event on the arm applies in between. A commit to `G`'s support inside the arm
removes `-G` before `B` is read, and the rule has no clause that notices. The
citation *"for (a) and (b) by `[ENT-3.S1]`"* is a category error: S1 is an entry
source and the premise is an exit property.

Clause (a′) states the exit property as an explicit side condition and is
therefore sound. **The proof paragraph was written for (a′) and applied to all
three** — structurally the same mistake round 1 made when it wrote the proof for
the identity route and applied it to the projection route, and the same mistake
round 2 found in (d). Three rounds, three instances, one habit.

### 1.5 Why this shape is ordinary code, not a corner

The idiom is *"the fast path did not apply, so force the general path"*:

```whitefoot
  if fits { off = n - 8; }
  else    { let yes = True(); set fits = yes; off = 0; }   // fall back
  …
  if fits { … deref(data)[off] … }
```

and its mirror, *"do the special case once, then clear the flag"*, which is c02.
Neither has any syntax marking it, neither is exotic, and §7.4's amended route
menu — *"test the same proposition again at the use"* — tells the writer to write
exactly the second `if fits`. As in round 1, **the failure has no syntax of its
own**, and `[ENT-5.G8]` emits nothing: the entry is neither killed nor
unsatisfied, it is satisfied and wrong.

### 1.6 The repair, and it costs both flagships nothing

Clauses (a) and (b) need the property `[ENT-5.G6]` already claims for them, and
it has a syntactic form:

> **R1.** `[ENT-5.G2]`(a) and (b) form an entry at `C` only when **no kill event
> on any path from the branch through either arm to `C` applies to
> `support(G)`** — the same test `[ENT-5.G3]`(d) applies to a carried entry,
> applied to a formed one.

The derivability form — *"(a) forms `chain(C) ++ [+G]` only when `B` derives
`-G`, and (b) only when `A` derives `+G`"* — is equally sound and is the exact
transplant of (a′)'s own condition, which is the argument for it: one condition,
stated once, for all three clauses, and `[ENT-5.G6]`'s first paragraph becomes
true as written instead of true for one clause in three. R1 is preferable only
because it is syntactic; §4.4 shows the domain law has to be weakened either way,
so that advantage is smaller than it looks. **Note that R1 is not available for
(a′)**: (a′)'s arm *does* write `S`, so a kill-event test would delete the clause.

Both flagships pay **nothing**: no arm of `layout.wf` writes any of
`f01…f10` (the `set` targets there are `tail_at`, `mark_at`, `head_end`,
`col_at`, `hyph_at`, `ind_at`, `rtl_at`, `just_at`, `col_w`, `acc`, `k`, `out`,
`pass` — no flag among them), and no arm of `a05_record_inline.wf` writes
`room4`, `room64` or `bad`. All nine of §5.6's discharges and all four of §6.2's
survive unchanged.

Whichever form is chosen, the phrase must be *"on any path from the branch
through that arm"* and not *"the reaching edge's kill events"*, for the reason
§2 gives.

---

## 2. THE SECOND DEFECT — the `r09` repair has two readings and they differ on memory safety

`[ENT-5.G3]`(d), round 2's own soundness repair:

> A carried entry is put through **every reaching edge's kill events**, not only
> `e`'s: an entry that another reaching edge's events would have killed had it
> been present there is not carried…

`[ENT-5]` places kill events on edges with precision — a `[SET-1]` commit's kill
is an event of that commit's own edge, and *"scope exits are edge events: kills
(c) and (d) apply on every edge leaving the scope"*. Under that vocabulary, the
**reaching edge** of a branch continuation is the arm-*exit* edge, and a `set`
in the middle of the arm is an event of an *interior* edge, not of it.

**Reading L (literal).** Only the events the reaching edge itself carries —
scope-exit kills — are applied. **Reading W (whole-arm).** Every kill event on
any path through that arm is applied.

`audit2/probes/b01_carry_out_interior.wf`, compiled, **REJECT** with exactly
`[OP-4] residual: x < len(deref(data))`, separates them. It is `r09` with the
flag write moved off the arm's last statement:

```whitefoot
  if f { if g { set x = 3_u64; } }            // key [+f, +g] ↦ {x - Z <= 3}
  else  { let yes = True();
          set f = yes;                        // now INTERIOR
          let pad = 1_u64;
          set spare = spare +wrap pad; }      // the reaching edge's own events
  if f { if g { let b = deref(data)[x]; … } }
```

Under **L**, the else arm-exit edge carries scope-exit kills for `yes` and `pad`
only, neither of which touches `support(+f) = {f}` or `support(x - Z <= 3) =
{x, Z}`; the entry is carried and index 50 of a 4-byte buffer is read. Under
**W** it is not carried and the program stays a rejection. Nothing in DRAFT §2.3,
§5.3 or §0.4 chooses. §2.3's own gloss — *"`set f = yes;` **on the else edge**
kills `support(+f)`"* — is `r09`'s last statement and so is consistent with both,
and §5.3's table only ever meets bare false edges (`@77`, `@78`, `@84`), which
disambiguate nothing.

Reading W is plainly intended and is the correct one. But this is the same defect
class the audit raised as O2 and that round 2 fixed by **quoting one sentence
verbatim at five sites**: two conforming implementations, one sentence, different
accepted sets, one of them unsound. The sentence that is round 2's headline
soundness repair needs the same treatment. The wording that settles it is not
"reaching edge's kill events" but *"every kill event on any path from the branch
through that arm to the continuation"* — which is also the wording R1 (§1.6)
needs, so one phrase fixes both.

---

## 3. Every round-1 attack, replayed against the repaired text

### 3.1 `a01` and `a04` — REFUSED, and the argument holds

At `if fits_again`, `[ENT-3.S1]` publishes the condition's goal-origin set:
`+fits_again`, plus its `[ENT-3]` 2924 expansion `+ige(m, 8_u64)`. It does
**not** publish `+fits`, and `[ENT-4]` does not back-derive it — `g13` and `g14`
(compiled, REJECT) against `g15` (ACCEPT) pin exactly that. The single
derivability route of `[ENT-5.G5]` is unsatisfied, nothing is released, and the
program rejects as it does today. **Confirmed.**

Two things strengthen the draft's own §5.7 argument and are worth recording,
because they mean D1 is not the only thing standing between a01 and acceptance:

1. `fits`'s comparison origin is *already* gone at the use. Spec 2915(b) admits
   the origin of a bare `own Bool` IDENT only when *"no `[ENT-5]` kill event
   (a)–(d) applies to a fact supported by an operand term of `R` on any path from
   that initializer to the use"*, and `set m = 64_u64;` is such an event. So even
   an implementation that kept a projection list would find `R` unavailable at
   the use for `fits`. This does not rescue round 1 — round 1 stored `R` in the
   entry and re-derived it from `fits_again`'s *fresh* origin, which is exactly
   the crossing — but it does mean the draft can quote 2915 as well as 2924.
2. `a04`'s loop route is closed for the release reason, not the head reason, and
   §5.7 says so correctly. I re-checked `[ENT-5.G3]`'s head subtraction against
   it: the entry's key support `{fits}` and delta support `{idx, Z}` contain no
   continuing-kill target of `@scan`, so the entry does survive every head —
   and releases nothing, because `again` is a third binding. **Confirmed.**

**The route menu no longer walks the writer into a01** (§7.4 now says "the same
`[ENT-2]` goal, not the same spelling"), but it *does* walk the writer into c01:
"test the same proposition again at the use" is precisely the second `if f`.

### 3.2 The E11/E11′ collision — one defined outcome, and it is the favourable one

Verified against the source. `layout.wf`'s two `if band(f09_indented, f07_head)`
branches are at lines 67–70 and 110–113, **both at top level**, so
`chain(C)` is empty at both and both keys are the identical
`[+band(f09_indented, f07_head)]`. Round 2's key-indexed map with union at
formation gives one row; the per-pair tighter constant restores
`ind_at - n <= -4` after line 109's `set ind_at = widened;` killed it, and site
133 discharges. **One defined outcome. The audit's §4.2 is answered.**

But **§5.4's justification of union is not the true argument, and the true one is
the sentence §2 shows is ambiguous.** §5.4 says union is sound because *"each was
established under the same key on a path reaching here, so each holds wherever
the key is satisfied"* — which assumes what is to be proved. The argument that
works is: the entry's **continuous liveness** from the first formation to the
second means no kill touched any key member's support across that span, so the
key's truth value is constant across it, so the key true at `P` implies *both*
arms ran, so both deltas hold. That premise is delivered by `[ENT-5.G3]`(c)'s
presence-on-every-edge test and (d)'s kill condition — the clause with two
readings. Union is sound under reading W and under R1, and unsound without them.
The draft should state the premise where it states the operator.

### 3.3 The omission hazard — closed; I could not build the divergence

Round 2 deletes the omission clause and generalises to the domain law. I
constructed the audit's recipe against the repaired text and it does not build:
keys are `chain(C) ++ [member]`, fixed at formation, so the two entries the
recipe needs cannot exist with one key and different futures; there is no
omission, so there is no `φ` for one to lack; and clause (d) no longer changes a
key, so the divergence step has nothing to act on. **The `[CLM-2]`-class hazard
is gone.** §4.4 records the one place the *law* is broken and why it is
nevertheless monotone.

### 3.4 `a05`/`a07`/`a08` — the record flagship does discharge under the repaired route

I re-executed all four sites of `a05_record_inline.wf` against §2's text.

- **@30.** `A ⊢ +h.has_body` (P-S5B items 2 and 3 from `+yes`), `B ⊢ -h.has_body`
  (P-S5B item 1, unwritten on that edge). (a′) fires; key `[+h.has_body]` carries
  `h.body_at - n <= -4`. `A` and `B` differ on `h.has_body` at the arm exits, so
  §1's defect does **not** reach flagship B: the arm writes `h.has_body`, not the
  branch condition `room4`.
- **@38.** Same for `h.wide`; the else arm's `set h.wide_at = 0_u64;` is what puts
  `h.wide_at - Z <= 0` in `diff(B, J)` — `n` is unbounded above on the then edge,
  so the **upper** bound is what the join loses. §6.1 is right and §5.1's counting
  rule is the correct one.
- **@42, the early `return`.** One reaching edge, `[ENT-5.G2]` forms nothing, and
  `[ENT-5.G3]`(c) over one contributing edge passes every key through. Free.
- **The `&uniq` mutator at 44.** This is the one step round 2 asserts and does not
  probe (*"unprobed only because the goal form needs §3.3.3's prerequisite"*). It
  **is** probeable without the prerequisite, and I did:

| probe | verdict | what it pins |
| --- | --- | --- |
| `b02_goal_survives_uniq_borrow.wf` | **ACCEPT** | the signed goal `+h.flag`, published by S1, survives a whole-struct `&uniq` borrow, a call declaring `writes(h.seen)`, and the region exit, and still discharges a `requires f` |
| `b02c_goal_dies_on_flag_write.wf` | **REJECT** `[FN-8]` | the identical program with the callee's `[EFF-2]` row naming `h.flag`: kill (b) does reach the goal through the `&uniq` actual |

  So the projection is field-granular on the **goal** half as well as the L0 half
  (`r06`), and §3.3.2's three bullets are now all machine-checked. **§6.1's "every
  key survives" is confirmed, not asserted.**
- **@64, inside `@paint`.** Continuing kills are `set out` and `set pass`; the
  key support `{h.has_body}` and the surviving delta over `h.body_at`, `n`, `Z`
  meet neither, so the head subtraction removes nothing and (2′) releases every
  iteration. **Confirmed.**

**Four residuals, four discharges. D3 is right, and `a06`'s shadow-local rewrite
is genuinely eliminated.** The coverage claim the audit refuted is now defensible.

### 3.5 `a12` and the size bound — the bound covers it

Re-derived from scratch in §5. `a12`'s maximiser lives inside the bound with a
factor of six to spare, and the bound is tight up to that constant. **Confirmed.**

### 3.6 The K-ladder and the per-round seams — walked

Walked 3.6.2's fourteen lines with a released candidate; findings in §7.

### 3.7 The per-view case — the worked example is right, the rule is contradicted elsewhere

§3.7.3's `s4_blinded` walk of flagship B checks out: the `[-h.wide]` entry forms
in both views because `h.wide_at - Z <= 0` does not depend on `requires
ige(room, 1_u64)`, releases in both, and the obligation is unproved in the
blinded view because the *other* premise is missing — which is exactly what the
blinded view is for. **Confirmed.** §7.3 records where the per-view rule is
contradicted by §3.14's step 2.

---

## 4. My three new attacks

All three are on the repaired reading of support and on the record route, as
directed. Two are refused; the third is refused for a reason the draft does not
state and that matters.

### 4.1 Aliasing through a borrowed struct — REFUSED

Four routes, each decided by rules already written and now each pinned:

| route | outcome | mechanism |
| --- | --- | --- |
| callee takes `&uniq` of the **whole** struct, declares `writes(sibling)` | **refused** — the entry survives, correctly | `[EFF-2]` projects onto the resolved place `h.seen`, which does not overlap `h.flag` under `[OWN-7]`. `b02` ACCEPT (goal half), `r06` ACCEPT (L0 half) |
| the same callee declares `writes(h.flag)` | **kills the entry, correctly** | kill (b) through the `&uniq` actual. `b02c` REJECT |
| `set deref(p).flag = …` through a borrow | **kills** | `[SET-1]`/`[OWN-5]`/`[OWN-6]` resolve the target to `h.flag`; kill (a) |
| the borrow **itself** | kills nothing | a borrow is neither a commit nor a projected write; the region exit kills only facts whose support contains the *holder*, and `+h.flag`'s resolved place is reached without `deref` |

The interesting half is the **snapshot** case, and it is where D2 earns its keep
in the record idiom exactly as it does in `layout.wf`: with
`let flag = h.has_body;` and key `[+flag]`, a later `set h.has_body = no;` does
**not** kill the entry, and that is right — `flag` is a snapshot of the value at
the copy, and `+flag` at `P` still says the arm ran. The converse crossing is
blocked twice over: `[ENT-3]` 2924's expansion of `flag` to `h.has_body` is
invalid the moment `h.has_body` is a `set` target on the path, so the later
`if h.has_body` publishes nothing that satisfies `[+flag]`, and `r05` (REJECT)
pins that the expansion is one-directional anyway. **No attack.**

**One precision loss worth naming.** Goal identity is by exact typed-expression
tree (`[FN-8]`), support is by resolved place. So an entry formed under
`h.flag` and a use written `deref(p).flag` for a borrow `p` of the same `h` are
the *same proposition over the same place* and **different goals**, and the key
does not release. That is sound and conservative, it is not in §7.1's price list,
and `[ENT-5.G8]`'s third string does not fire for it either (the two are not
related by ordinary-let expansion). A record parser that hands `&uniq h` down and
tests through the borrow will be silently refused.

### 4.2 A flag field overwritten between store and test — REFUSED, by an unstated mechanism

`audit2/probes/b04_flag_cleared_between.wf`, compiled, **REJECT** with exactly
`[OP-4] h.at < len(deref(data))`. The sharpest form I could build:

```whitefoot
  if h.flag { set h.at = 3_u64; }                    // @C1: [+h.flag] ↦ {h.at <= 3, h.at >= 3}
  if h.flag { set h.at = 50_u64; }
  else      { let no = False(); set h.flag = no;     // clears the flag
              let pad = 1_u64; set spare = spare +wrap pad; }
  if h.flag { let b = deref(data)[h.at]; }
```

At `@C2` the inherited entry is present on the then edge only and its key member
`+h.flag` **is** the goal S1 establishes there, so (d)'s first condition is met
and everything turns on the second — the clause §2 shows has two readings. Under
reading **L** the entry is carried; under **W** it is dropped.

**It is safe under both**, and the reason is a mechanism the draft never states:
(d) carries the entry *as it stands on `e`*, and `@C2`'s then arm writes `h.at`,
so the stale `h.at <= 3` is already dead by kill (a) when (d) reads it. The
union at `@C2` therefore combines an **empty** inherited delta with
`{h.at <= 50, h.at >= 50}`, which is true.

Had the union seen both deltas it would have taken the per-pair tighter constant
in **both** directions — `h.at - Z <= 3` from `@C1` and `Z - h.at <= -50` from
`@C2` — an **inconsistent** delta that is not the syntactic `⊥` and that
`[ENT-5.G2]`'s Union operator has no consistency check for. Union is defined as
three per-pair extrema with no closure step; nothing in §2.2 or §5.4 observes
that a union can be inconsistent, and §2.6's `⊥` clause (*"if `δ` is `⊥`, no
execution reaches `P`"*) does not cover an inconsistent-but-not-`⊥` delta.
On the flagships this cannot arise, for the liveness reason §3.2 gives; it is a
gap in the *statement*, not a live break.

**The general form is refused too.** For two formation sites to union a stale
delta into one live key, the flag's own place must be unwritten between them —
and (a′) cannot fire twice on one key without a write, because it needs
`B ⊢ -S` and after the first continuation the join leaves `S` undecided. Every
route I tried closed on that. Writing the flag kills the entry (kill (a)); not
writing it makes the flag constant, so both arms ran and union is sound.

### 4.3 Two struct instances sharing a field spelling — REFUSED, and the refusal is `[ENT-2]`'s not this rule's

| probe | verdict | what it pins |
| --- | --- | --- |
| `b03_two_instances_same_field.wf` | **REJECT** `[FN-8]` | `if h2.flag` does not discharge a `requires` on `h1.flag`: two instances of one struct type are two resolved places and two goals |
| `b03c_same_instance_control.wf` | **ACCEPT** | the identical two-instance program tested on `h1` itself, so the shape is not the obstacle |

So an `[ENT-5.G2]`(a′) key `[+h1.flag]` cannot be satisfied by `+h2.flag` and no
delta about `h1.at` crosses instances. Shadowing and storage reuse are refused by
the same rule plus kill (d), as §2.4 says. **No attack** — but note that
`[ENT-5.G]` contributes nothing to this: the whole defence is `[ENT-2]`'s term
identity, and if a future change interned place datums by field path rather than
by resolved place, this rule would silently become unsound. Worth a sentence in
`[ENT-5.G1]` naming the dependency.

### 4.4 A fourth attack, on the domain law itself — the law is broken, but benignly

`[ENT-5.G1]`'s **domain law** reads: *"Which keys carry an entry at a program
point is a function of the syntax tree and of `[ENT-5]`'s kill events alone. …
No rule below may make the presence of a key depend on a derivability question."*

`[ENT-5.G2]`(a′) makes the presence of a key depend on a derivability question,
in as many words: *"when `A` derives `+S` and `B` derives `-S`"*. And
`[ENT-5.G7]`'s determinism paragraph rests on the law — *"The key set is
syntactic (`[ENT-5.G1]`'s domain law)"* — which is therefore false as drafted.

I tried to turn that into a `[CLM-2]`-class divergence and could not:

- Both of (a′)'s cases are conjunctions of derivability claims, so a stronger
  prover fires them at least as often: keys only grow.
- Both cases firing at once requires `A ⊢ ±S` and `B ⊢ ±S`, i.e. both arm-exit
  states contradictory, in which case both deltas are `⊥` and the continuation is
  unreachable.
- Determinism proper is unaffected: `[ENT-4]` 3033–3035 makes derivability a
  unique least-closure answer, so *"A derives `+S`"* is determinate and two
  conforming implementations agree.

So (a′) is monotone and deterministic; the **law** is stated too strongly, and
it cannot be repaired by making (a′) syntactic — (a′)'s whole point is that the
arm *does* write `S`, so the kill-event test R1 gives (a) and (b) is exactly
wrong for it. The law must be weakened to what is actually needed and what all
three clauses have: **monotone in the entering state** — a stronger prover
produces a key set at least as large. That is the property `[ENT-1]` 2853's
"fact-source and closure strengthening" class requires; "syntactic" was a
stronger claim than the design can keep, and `[ENT-5.G7]`'s determinism paragraph
must stop leaning on it (determinism is delivered by `[ENT-4]`'s least closure,
not by the law).

---

## 5. The size bound, re-derived, and both flagships recounted

### 5.1 Re-derivation from scratch

`[ENT-5.G7]`: at most `2(N_if + S_B)` keys; each delta at most `T(T-1)` bounds
and `T(T-1)/2` disequalities; total `3(N_if + S_B)·T(T-1)`.

- **Entries.** Every key is `chain(C) ++ [m]` for a formation continuation `C`
  and `m` drawn from `{+G, -G}` (clauses (a), (b)) or `{+S, -S}` for a `Bool`
  place `S` a commit in `C`'s arms writes (clause (a′)). Summing over `C` gives at
  most `2·N_if + 2·S_B` such pairs; coinciding keys share one row, so the count is
  an upper bound. If one arm commits `S` twice, that is two commits and one key
  pair, so counting commits over-counts and the bound still holds. **Correct, and
  it is a theorem rather than an estimate — the entry count no longer depends on
  nesting depth, which is what removing `C_G` bought.**
- **Delta.** `T` terms give `T(T-1)` ordered pairs and `T(T-1)/2` unordered ones,
  so `1.5·T(T-1)` atomic members. **Correct.** The `value_if` delivery relations
  are relations over live terms and are inside that count, which closes one of the
  two omissions round 1's bound had.
- **Total.** `2(N_if + S_B) · 1.5 · T(T-1) = 3(N_if + S_B)·T(T-1)`. **Correct.**
  `3 × 30 × 21 × 20 = 37,800` for `layout.wf` with `S_B = 0`. **Correct.**
- **Key-member storage.** `+ at most one slot per entry per enclosing branch arm`
  — bounded by the grammar's finite nesting depth. **Correct.**

**Does it cover `a12`?** Yes, with room. The maximiser is a chain
`t1 <= … <= tT` under one guard `if ile(tT, t1)`, whose then-edge closure gives
`T(T-1)/2` delta members from an arm that commits nothing; the per-delta bound is
`1.5·T(T-1)`, a factor of six above it. Repeated over `N_if` disjoint chains the
state reaches the total bound within that same constant. `a12` (ACCEPT) pins the
positive closure step at `T = 3` and `r07` (ACCEPT) the negative form.
**The bound is tight up to a constant, §7.2's attribution is now correct, and
round 1's "an arm changes two or three terms" is properly retracted.**

**Two things the bound still does not say.** A `⊥` delta is not a set of facts
and costs nothing, which is fine but unstated. And `[ENT-5.G8]`'s clause 5b fires
for *"a guarded entry that was **killed** at some point dominating `O`"* — which
requires retaining dead entries and their killing events. That history is outside
`[ENT-5.G1]`'s *"finite partial map"* and outside this bound. It is a diagnostic
structure, so it can be reconstructed on demand rather than carried, but the
draft should say which.

### 5.2 Flagship A recounted

I re-executed §5.2's table against `probes/layout.wf`. **The entry count is
right.** At line 114 the live rows are the two each from @51, @55, @58, @62, @66,
@70, @78, @77, @76, @83 and @90 — eleven continuations, **22 entries** — with
@84, @107 and @113 unioning into earlier rows and @98's `[±more]` removed by kill
(d) on the edge leaving `@scan`. Against the entry bound of
`2(N_if + S_B) = 60`, that is a factor of 2.7. **§7.2's "within a factor of
three" is confirmed.**

**The fact count is a formation-time sum presented as a line-114 measurement.**
Summing §5.2's `|δ|` column over those 22 rows gives **251**, close enough to the
stated "about 257" at the declared ±20%. But it is the sum of the deltas **as
formed**, and §5.5's own kills are not deducted: the sink call at 105 removes the
three `x - room <= -32` members of the `[+f10_canvasfit]` row (listed as 4) and
the `room - x <= 31` members of its negative twin (listed as 5), so the true
figure at line 114 is nearer 245 before @113's union adds back. The
order-of-magnitude claim survives and the number should be labelled.

**Two orders of magnitude below the bound**: 251 against 37,800 is a factor of
150. **Confirmed.** Against the ambient closed L0 state — `T = 21` gives at most
420 ordered-pair bounds — the guarded component is about **0.6×** the state it
rides beside, so F-G2's threshold of ten times has ample headroom.

**One sentence in §7.2 is wrong and should go.** *"The guarded component costs at
most `2(N_if + S_B)` extra zones **over the whole function**"* — it costs that
many extra zones **at every program point**, because the component is carried
along the flow. On `layout.wf` that is a 60× state-size ceiling per point, not a
constant per branch node. The *measured* factor is 0.6, so the conclusion holds;
the reassurance does not.

### 5.3 Flagship B recounted

`parse_record` has seven branch nodes (`room4`, `room64`, `bad`, `h.has_body`,
`h.wide`, `done`, and `h.has_body` inside `@paint`) and two `Bool` commits in
arms, so `N_if = 7`, `S_B = 2` and the entry bound is `2(7 + 2) = 18`. §6.1's
eight entries at @30 and @38, plus the render-phase and loop continuations, sit
inside it. The live terms at the measure phase are the nine §6.1 names, so
`T = 9` and the fact bound is `3 × 9 × 9 × 8 = 1,944` against §6.1's four
eight-member deltas. **Both figures check out.** Flagship B is a smaller program
and its interest is coverage, not cost.

---

## 6. Monotonicity, restated in my own words against the repaired text

The claim to be defended is `[ENT-1]` 2853's: *no program that compiles with
`[ENT-5.G]` switched off, or under a weaker ambient prover, compiles less with it
switched on or under a stronger one.* Take two flows of one function, `F0` and
`F1`, where `F1`'s entering state at every point derives everything `F0`'s does.
The argument is an induction over `[FN-1]` order with three obligations.

**(i) The key set never shrinks.** Keys are `chain(C) ++ [m]`. `chain(C)` is
syntax. `m` comes from clauses (a) and (b), whose trigger is *"the condition has a
goal origin"* — a function of the syntax tree and of `[ENT-5]`'s kill events, both
prover-independent — or from clause (a′), whose trigger is a conjunction of two
derivability claims, monotone upward. So `keys(F1) ⊇ keys(F0)`. Under R1 (§1.6)
the (a)/(b) trigger gains a kill-event conjunct, still prover-independent; under
the derivability form of R1 it gains a monotone conjunct. **Either repair
preserves this.**

**(ii) Every fact `F0` has at a point, `F1` has.** The only new way to acquire a
fact is step (2′). Suppose `F0` releases `φ` at `P` from entry `ε` with key `K`.
Then `K ∈ keys(F1)` by (i); `ε` is live at `P` in `F1` because liveness is decided
by kill events, which are identical in both flows; and `K` is satisfied at `P` in
`F1` because satisfaction is derivability of the key members, monotone upward.
So `F1` releases `δ_{F1}(K)` at `P`, and the question is whether `φ ∈ δ_{F1}(K)`.

Not necessarily — and this is the subtle step the draft states correctly.
`δ = diff(X, J)` and `δ ∪ J = closure(X)` over the live pairs. `X` is monotone in
the entering state, so `closure(X)` grows; but `J` may grow faster and absorb `φ`
out of `δ`. In that case `φ` is in `J`, hence in the ordinary state at the
formation continuation, and **transported strictly better than it would have been
in `δ`**: a delta fact dies under exactly the events that would kill it as an
ordinary fact (`[ENT-5.G4]`) *and* additionally dies when the key's support dies
or when `[ENT-5.G3]`(c)/(d) decline to inherit the entry. A `J`-held fact is
subject to the first of those and to neither of the other two. So the `δ`→`J`
migration is a strict improvement at every downstream point. **This is the case
the audit found missing from round 1's two-case argument, and round 2 closes it by
construction rather than by adding a third case: there is no differently-keyed
entry to hide the fact in, because a key is a syntactic function of its formation
site, and there is no omission clause.** I could not find a fourth case.

**(iii) Nothing consumes a state and returns less from more.** Formation writes a
new component and deletes no L0 fact. Union and join are per-pair extrema,
monotone in both arguments. `⊥` is the top of the delta order and is reached only
by the stronger flow. Release is a monotone fixed point: releasing one entry only
enlarges the state, and a larger state satisfies at least as many keys, so the
round order is unobservable and the result is the least fixed point above the
input. The head subtraction and the `[ENT-5]` join each derive a subset of every
input, which is why (2′) is correctly *not* run at either — a point I checked
against the contradictory-input rule (spec 3097's *"a contradictory all-derivable
input imposes no constraint"*), and it holds there too, because a contradictory
input derives everything and so is a superset of the join.

**Where it lands in `[ENT-5.R]`.** (ii) is exactly the premise `[ENT-5.R6]` needs
— *"every source, kill, join and closure derives at least as much from a larger
input state"* — extended to formation and release, so a larger family still makes
each preheader state larger, each universe larger and each greatest inductive
family at least as large. `[ENT-5.R5]`'s ascending sequence and its
`|pairs(F) × K| + 1` termination bound therefore survive **provided `K` is one
fixed finite set**, which §3.7.1's decision preserves. **The monotonicity story
is sound.** Its one soft spot is `⊥` (§7.1).

**And the `⊥` case is sound for a better reason than the draft gives.** §2.2
argues that an entry whose delta is `⊥` releases a contradiction only at an
unreachable point, *"because a contradictory arm-exit state means no execution
leaves that arm"*. That is true of the formation site but says nothing about a
`⊥` reaching a live key by union from a *different* site. The argument that
works: an entry survives from one formation to another only if no kill touched
any key member's support in between, so the key members' truth values are
constant across the span; a contradictory arm-exit state at the first site proves
those values cannot be the ones the key names; so any point where the key is
satisfied is unreachable, whichever site the `⊥` came from. Same conclusion,
different and complete reason — and it is the same liveness premise §3.2 shows
union needs and §5.4 does not state.

---

## 7. The seams

### 7.1 `[ENT-5.R2]`'s ladder — the decision is right, the disclosure is short one case

§3.7.1 computes `ladder(F)` once from the retention-free flow **in which step
(2′) runs**, and refuses per-round recomputation to keep `pairs(F) × K` fixed.
Walked against 3.6.2's fourteen lines with a released candidate: line 1 computes
`K` from the family-empty flow; line 4's `flowU` and line 8's `flow` both carry
the guarded component under §3.7.2's Edit 2; a fact released on a preheader edge
is in `E(@l)` at line 5, its constant is in `K` from line 1, and it becomes an
ordinary `[ENT-5.R2]` candidate at line 6, deletable at line 10 like any other.
**The algorithm needs no fifteenth line and Edit 2 is the right shape.**

Two gaps in the disclosure.

- **A `⊥` release makes `E(@l)` contradictory, and `ladder(F)` is then undefined.**
  `K` is *"every bound constant appearing in any `E(@l)`"*; a contradictory state
  derives every bound, so either the phrase means the syntactic representation
  (implementation-visible, hence an `[ENT-1]` problem) or it means derivability
  (in which case `K` is infinite). F-G5's new refutation condition — *"`ladder(F)`
  computed with step (2′) is not a superset of `ladder(F)` computed without it"* —
  is vacuous or ill-posed in exactly that case. One sentence: a contradictory
  `E(@l)` contributes no constant.
- **The residual is understated.** §3.7.1 says a candidate whose constant appears
  only in a round-`i` state is unreachable, and that this *"is `[ENT-5.R2]`'s
  existing property, not something release introduces"*. Half right: release makes
  the class strictly larger, because a delta is `diff(A, J)` over states that
  themselves move between rounds, so retention at round `i` can create a delta
  member with a constant no round-0 state held. The direction is safe and
  deterministic; the sentence should say "enlarges" rather than "does not
  introduce".

### 7.2 The per-round slot — Edit 1's justification is incoherent

Edit 1 puts the guarded-component head subtraction between the continuing-kill
subtraction and retention, *"because retention must see the head's guarded
component in order for a released fact to be a candidate."*

That reason is false. `[ENT-5.G5]` says step (2′) *"runs on every edge and at no
other point. It is not run at a merge or at a loop head."* So the head's guarded
component releases nothing at the head, and retention's candidates come from
`E(@l)` — the closed **preheader** state, where (2′) already ran — and from
`H0(@l)`. Retention never reads the head's guarded component at all.

The placement is harmless because it is unobservable, but a fixed order in
`[ENT-5.R3]` justified by a false reason invites an implementation to reason
from it, and this is a rule whose whole determinism case is that the order is
fixed and the reason for each slot is stated. Give the slot its real reason: the
subtraction reads the same continuing-kill predicate the step before it computes,
and nothing downstream reads its output, so it may as well sit there.

### 7.3 Per proof view — §3.7.3 is right and §3.14's step 2 contradicts it

§3.7.3 states, correctly and load-bearingly, that *"no entry, delta or release
crosses views"*, and the worked `s4_blinded` case for flagship B is right (§3.6).

But §3.14's **step 2 computes the ladder and the `[ENT-5.R5]` retention family
once, before step 3's per-view loop**. Under Edit 2 those flows run step (2′).
So a single family is derived from a flow that used **one** view's guarded
releases and is then installed at every loop head in **both** views. That is a
release crossing views by another route, and it contradicts §3.7.3's sentence
directly.

§3.7.4 calls this *"one seam this design inherits rather than creates"* and defers
it to Q6. That is not available here. Before `[ENT-5.G]`, step 2's view-ambiguity
was a *precision* under-specification (S4 facts in `E(@l)`). After it, §3.7.3 is
stated as a **rule** — "no release crosses views" — and step 2 violates it. Either
§3.7.3 must be qualified (*"except through `[ENT-5.R]`'s family, which is computed
once, in the complete view"*, and then the `[PRV]` partition argument must be
re-made) or Q6's recommendation must be taken **in this batch** rather than
deferred. This is the one seam round 2 does not actually own.

### 7.4 `[IND-7]` — confirmed

§3.5's split (entries invisible, released facts ordinary and therefore filling and
tightening slots) is right, and the safety argument is 0108 §2.4's fifth repair
rather than anything in this rule: the slot list, visit set and elimination-term
list are syntactic, the contents are the ambient prover's, and *"a slot that fills
or tightens never loses a certificate"*. `[ENT-5.G]` is exactly an ambient
strengthening. I attacked this again and found nothing new. §3.6(b)'s rewrite
says plainly what round 1's sentence obscured. **Both answered.**

---

## 8. The intent test, both ways

**Annotations: still zero.** Nothing in round 2 asks the writer for a keyword, an
attribute, a `because`, a statement or an obligation. Flagship A is 186 lines of
ordinary layout code and flagship B is a record parser; neither contains one
character of ceremony. D3 removes the one piece of plumbing round 1 forced — the
`a06` shadow local for every flag already stored in the record — and that removal
is the strongest thing round 2 did. **This half is unimpeachable and better than
round 1's.**

**Auditability: `[ENT-5.G8]` is in the rule, and it is still not enough.**
Making the `correlation` diagnostic normative was the right call, the third
string is the right addition, and both of its named customers (`g13`/`g14` and
`a07`) are compiled. But the token is the only channel through which an invisible
mechanism is legible, and it is silent in **four** cases, three of them created or
enlarged by round 2:

1. **A `[ENT-5.G3]`(d) refusal.** Round 2's own soundness repair works by *not
   carrying* an entry, and that is neither a kill nor an unsatisfied key — clause
   5b fires for neither branch. So the one place where the rule's soundness bites
   hardest is the one place the writer is told nothing. `b04` is that program: it
   rejects with a bare `[OP-4]`, and the reason (the else arm cleared the flag) is
   exactly what a `correlation` string could say. **This needs a fourth string.**
2. **A `[ENT-5.G3]`(c) refusal at a merge** — a key absent on some contributing
   edge. Same gap, same fix.
3. **Fallback N1's excluded shape**, which §7.2 discloses honestly: no entry is
   formed, so nothing fires.
4. **The goal-identity gap** (§4.1): an entry keyed on `h.flag` and a use written
   `deref(p).flag` are the same proposition and different goals. The third string
   fires only when the two are related by ordinary-let expansion, which these are
   not.

**The unsound case is invisible, and that is now a two-round pattern.** c01 is
normal code — a fallback arm that forces the general path — and under the drafted
text it becomes an out-of-bounds read with no syntax to inspect and no diagnostic
to read. The audit's §8 point 3 was: *"a rule this invisible must be sound by
construction, because the writer has no way to audit it and the reviewer has
nothing to review."* Round 2 quoted that principle and then shipped a formation
clause whose soundness paragraph asserts a premise the clause does not check.
The lesson `[ENT-5.G]` keeps re-learning is narrower than "re-derive the proof":
**every clause that names a signed goal as a key must state, in the clause, what
makes that goal record the arm — and (a′) is the only one of the four that
does.**

---

## 9. Repairs

**R1 — the soundness repair (required).** §1.6. Clauses (a) and (b) form an entry
only when no kill event on any path through either arm applies to `support(G)`;
or, equivalently and more uniformly, only when the *other* arm-exit state still
derives the complement of the key member — (a′)'s own condition, stated once for
all three clauses. Cost: **zero on both flagships**, verified against the `set`
targets of `layout.wf` and `a05_record_inline.wf`. Gain: `[ENT-5.G6]`'s first
paragraph becomes true as written, and c01, c02 and the whole family of
"fall back and set the flag" idioms are refused.

**R2 — settle (d)'s reading (required).** §2. Replace *"every reaching edge's kill
events"* with *"every kill event on any path from the branch through that arm to
the continuation"*, at the two places it appears (`[ENT-5.G3]`(d) and §0.4). The
same phrase serves R1, so one wording fixes both. Cost: zero on both flagships.

**R3 — weaken the domain law to monotonicity (required).** §4.4. The law as
written is falsified by `[ENT-5.G2]`(a′) in the draft's own text, and
`[ENT-5.G7]`'s determinism paragraph leans on it. The property the design needs
and has is *monotone in the entering state*; determinism comes from `[ENT-4]`'s
least closure, not from the law.

**R4 — state union's premise where union is stated (required).** §3.2. Union is
sound because the entry's continuous liveness makes the key's truth value
constant across the two formation sites, not because *"each holds wherever the
key is satisfied"*. Add the consistency question too: `[ENT-5.G2]`'s Union has no
closure step and can in principle produce an inconsistent delta that is not `⊥`
(§4.2). Under R1 and R2 it cannot; say so.

**R5 — the three seam sentences (required).** §7.1's contradictory-`E(@l)`
sentence and the "enlarges" correction; §7.2's real reason for Edit 1's slot;
and §7.3 — either qualify §3.7.3 or take Q6 in this batch. The third is the only
one with a decision in it.

**R6 — a fourth and fifth `[ENT-5.G8]` string (recommended).** §8 cases 1 and 2:
the entry existed and was not inherited, naming the branch and the other arm's
write. Without it, round 2's own repair is the least legible thing in the rule.

**R7 — name the `[ENT-2]` dependency (recommended).** §4.3. This rule's defence
against two struct instances sharing a field spelling is entirely `[ENT-2]`'s term
identity by resolved place. One sentence in `[ENT-5.G1]` makes the dependency
visible to whoever next edits `[ENT-2]`.

**Retained from round 1's audit, and correctly discharged by round 2:** R1a
(delete the projection route) = D1 ✓; R2 (define `support(+G)`) = D2 ✓; R3
(delete the omission clause) ✓; R4 (key-indexed) ✓; R5 (buy (a′)) = D3 ✓; R6
(the seam sentences) — two of three ✓, §7.3 outstanding.

---

## 10. What the owner must decide

The draft adopted D1, D2 and D3 as adopted-and-flagged and I confirm all three:
delete the projection route, key on the direct goal with the direct goal's
support, and buy the committed-flag key. Nothing below reopens them.

**P1 — Which form of R1?** The syntactic kill-event test, or the transplant of
(a′)'s arm-exit derivability condition. Both cost the flagships nothing and both
are sound. The kill test is prover-independent; the transplant is one condition
for all three clauses and makes `[ENT-5.G6]` read as one argument. This is the
only choice in the soundness repair and it is a genuine one.

**P2 — Is the domain law weakened, or is (a′) re-drafted?** R3 weakens it to
monotonicity, which is honest and is what `[ENT-1]` 2853 actually requires. The
alternative is to find a syntactic (a′), which I do not believe exists: the clause
must know which arm the flag records, and only the arm-exit states know.

**P3 — §3.14 step 2, now.** §7.3. Either `[ENT-5.G]` qualifies its own per-view
sentence, or `[ENT-5.R]`'s family goes per view in this batch. Deferring to Q6
leaves a stated rule contradicted by a stated pipeline, which is the one thing
`[ENT-1]` cannot absorb.

**P4 — Does `[ENT-5.G8]` gain the not-inherited strings?** §8. My view is that it
must, because round 2 moved the rule's principal loss mode from "killed" to "not
carried" and the diagnostic did not follow.

**P5 — Is F-G1 run before or after R1 lands?** The draft is honest that no line of
`[ENT-5.G]` is implemented and that *"every acceptance claimed in §5.6 and §6.2
is a hand-execution over a state the compiler prints no view of. That is exactly
the gap that let round 1 ship a memory-safety bug in a table of nine confident
discharges."* It let round 2 ship a second one in the same table. Implementing
`[ENT-5.P0]` step (2′) alone with hand-supplied entries — the audit's own
suggestion, half a day — would have turned c01 into a machine verdict, and it is
now the cheapest thing that would stop this recurring.

---

## 11. Probe ledger

Nine files in `audit2/probes/`, all compiled against the unmodified worktree
compiler. `audit2/run.sh` reproduces every verdict; `./run.sh probes/*.wf` and
`./audit/run.sh` reproduce round 1's and round 2's, and I re-ran all three.

| probe | verdict | role |
| --- | --- | --- |
| `c01_arm_writes_flag.wf` | **REJECT** `[OP-4] x < len(deref(data))` | **the refutation.** Accepted under round 2; reads index 50 of a 4-byte buffer. Clause (a), no carry-out, no prerequisite |
| `c01c_joined_control.wf` | **ACCEPT** | c01 with one literal changed 50→3: the discharge arithmetic, machine-checked |
| `c02_negative_key_arm_writes.wf` | **REJECT**, same residual | the same break through clause (b), on a negative key |
| `b01_carry_out_interior.wf` | **REJECT** `[OP-4] x < len(deref(data))` | `r09` with the flag write made interior to the else arm: separates the two readings of `[ENT-5.G3]`(d), one of which reads out of bounds |
| `b04_flag_cleared_between.wf` | **REJECT** `[OP-4] h.at < len(deref(data))` | attack (ii), a flag field overwritten between store and test — refused, by the unstated mechanism of §4.2 |
| `b02_goal_survives_uniq_borrow.wf` | **ACCEPT** | attack (i): the signed goal on a `Bool` field survives a whole-struct `&uniq` borrow and a sibling-field `writes` row. Supplies the probe §3.3.2 says it lacks |
| `b02c_goal_dies_on_flag_write.wf` | **REJECT** `[FN-8]` | its control: with the `[EFF-2]` row naming the flag field, kill (b) does reach the goal |
| `b03_two_instances_same_field.wf` | **REJECT** `[FN-8]` | attack (iii): two instances of one struct are two goals; a key on `h1.flag` is not satisfied by `+h2.flag` |
| `b03c_same_instance_control.wf` | **ACCEPT** | its control: the two-instance shape is not itself the obstacle |

**What this ledger does not establish.** As in round 1, no line of `[ENT-5.G]` is
implemented, so c01 is a hand-execution of the drafted rules over a state no
compiler prints. It is refuting because every premise it needs is machine-checked
separately — the formation step is the one DRAFT §5.7 performs itself on a01,
the release step is `[ENT-3.S1]` publishing the branch goal, and the discharge is
`c01c` — and the only step in between is the one the draft specifies.
