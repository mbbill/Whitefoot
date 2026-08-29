# Adversarial audit of `[ENT-5.G]` — guarded facts

Target: `/tmp/claude-0/-home-user-Whitefoot/6a4209eb-2cad-5504-9f06-67307ee32037/scratchpad/wf-0111-guarded-facts/DRAFT.md` (905 lines).
Baseline: the 0108 design at
`research/investigations/claim-model/DESIGN.md` on `batch/0106-claim-model-design`
(read only; `git status` in that worktree was clean before and after).
All probes compiled with the worktree compiler at
`wf-0111-guarded-facts/target/release/whitefootc`; `audit/run.sh` reproduces
every verdict below.

## Verdict

**REFUTED**, on a memory-safety witness, with a cheap and identified repair.

The refutation is narrow and surgical: it is **not** the formation rule, **not**
the kill discipline, **not** the join, **not** the loop seam, and **not** the
identity release route. Those survived every attack I could construct. It is the
**second** release route — `[ENT-5.G5]`'s *projection* clause — which admits a
guarded fact on the strength of a relation whose terms the entry's support does
not protect. `audit/probes/a01_projection_retest.wf` is a program the drafted
rule accepts and which reads 46 bytes past the end of a 4-byte buffer.

A second, independent finding is a **coverage claim that does not hold**: the
draft's own flagship is the one shape in this family the rule handles, and the
adjacent shape — flags computed as comparisons but *stored into a record* and
re-tested from the record — is discharged **zero** times by the rule and is not
in §5.1's price list. `a05`/`a07` are the witnesses.

Neither finding kills the idea. Both are repairable inside the drafted frame,
and §7 below states the repairs.

---

## 1. SOUNDNESS — REFUTED. The projection release route admits false facts

### 1.1 The witness

`audit/probes/a01_projection_retest.wf`, compiled, **REJECT today** with exactly
`[OP-4] residual: idx < len(deref(data))`:

```whitefoot
fn broken['d](data: &'d buffer<u8>, small: own u64) -> result: own u8 reads(data) contract {
  define room = len(deref(data));
  requires ige(room, 4_u64);
} {
  let m = small;
  let fits = ige(m, 8_u64);
  let idx = if fits {
    let three = 3_u64;   give three;
  } else {
    let fifty = 50_u64;  give fifty;
  }
  set m = 64_u64;
  let out = 0_u8;
  let fits_again = ige(m, 8_u64);
  if fits_again {
    let byte = deref(data)[idx];
    set out = byte;
  }
  return out;
}
```

Called as `broken<'r>(data: &'r input, small: 0_u64)` on a 4-byte buffer.

### 1.2 Hand execution of the drafted rules, step by step

**Formation, `[ENT-5.G2]`(a) at the `value_if` continuation.**
`fits` has goal origin `{fits, ige(m, 8_u64)}` and comparison origin
`R = 8 - m <= 0`. The then edge's delivery image gives `idx = 3`; the else edge
gives `idx = 50`. `J` therefore derives `idx - Z <= 50` and `Z - idx <= -3`,
and `A` derives `idx - Z <= 3`, a strict improvement. So:

    key        [ +fits ]
    projection [ 8 - m <= 0 ]
    delta      { idx - Z <= 3 }

No commit image `[ENT-3.S5]` is needed anywhere in this program: the delta comes
from the `value_if` delivery clause of `[ENT-5.G2]`(a), which is precisely why
the draft calls the `g03`/`g04` pair its clean isolating witness. **The witness
therefore refutes `[ENT-5.G]` alone**, on top of today's checker, with no other
0108 rule in the way.

**Transport, `[ENT-5.G4]`.** The entry's support is
`support(delta) ∪ support(key)` = `{idx, Z}` ∪ `support(+fits)`. By
`[ENT-5.G1]` the key is the **direct** goal, and DRAFT §3.4 fixes its support as
the binding — the whole point of that subsection is that a write to an origin
place kills the *expanded* goal and not the *binding* goal. **`m` is in neither
half of the support.** So `set m = 64_u64;` kills nothing: not the delta fact
(support `{idx, Z}`), not the key member (support `{fits}`).

**Release, `[ENT-5.G5]` step (2') on the then edge of `if fits_again`.** S1
publishes the comparison origin of `ige(m, 8_u64)`, i.e. `8 - m <= 0`. That is
byte-for-byte the stored projection `R_1`. `[ENT-5.G5]`: *"A key member `s_i` is
satisfied … when its projection `R_i` is present and L0 derives `R_i`."*
Satisfied. Step (2') adds `idx - Z <= 3` to the state as an ordinary L0 fact.

**Discharge.** `idx <= 3` with the `requires`'s `len(deref(data)) >= 4` closes
`idx < len(deref(data))` in one `[ENT-4]` transitivity step. **ACCEPTED.**

**Execution.** `small = 0`, so `fits` is false, the *else* arm delivered
`idx = 50`, and the buffer is 4 bytes. The accepted program reads
`deref(data)[50]`. Memory corruption, from an accepted program, with no `unsafe`
and no `claim`.

### 1.3 Both load-bearing premises are machine-checked, not asserted

| probe | verdict | what it pins |
| --- | --- | --- |
| `a01_projection_retest.wf` | **REJECT** `[OP-4] idx < len(deref(data))` | the only premise the read lacks today is the guarded release; every other step of the program is legal now |
| `a02_control_reprojection.wf` | **ACCEPT** | after `set m = 64_u64;` a *fresh* binding `ige(m, 8_u64)` still publishes `m >= 8` to L0 at its then entry, over the same term `m`. The stored projection is derivable at the release site |
| `a03_control_discharge.wf` | **ACCEPT** | with `idx - Z <= 3` present (both arms deliver 3) and `room >= 4`, the subscript discharges. The arithmetic of a01's discharge is confirmed |

a01 = a03 with one arm's literal changed from `3` to `50`. The only difference
between an accepted program and a refuted one is whether `idx - Z <= 3` is a
joined fact or a released guarded fact.

### 1.4 Where `[ENT-5.G6]`'s proof breaks, exactly

`[ENT-5.G6]` argues:

> `ε` live at `P` means no kill event on `J`→`P` reached the support of `G`, so
> by `[ENT-5]`'s over-approximating overlap relation `[OWN-7]` no place `G`
> reads was written, so `G`'s value at `P` equals its value at the branch.
> `ε` satisfied at `P` means `G` is true at `P`.

Two sentences, each false under the draft's own other choices:

1. *"no place `G` reads was written."* Only under the **expansion** reading of
   `support(+G)`. §3.4 mandates the **binding** reading, precisely so that the
   flagship's `+f10_canvasfit` key survives the sink call's write to `room`'s
   origin. Under the binding reading, support(`+G`) = `{fits}`, and every place
   the *expansion* reads is unprotected.
2. *"`ε` satisfied at `P` means `G` is true at `P`."* False by construction:
   `[ENT-5.G5]` gives satisfaction **two** routes, and the second one — the
   projection — establishes that `R` is true at `P`, not that `G` is. `R` at `P`
   equals `G` at the branch only if `R`'s terms are unwritten in between, and
   nothing in `[ENT-5.G1]` or `[ENT-5.G4]` requires that.

The proof is written for the identity route and silently applied to both. The
identity route is genuinely sound — see §2.

### 1.5 The dilemma the draft is standing on

The two readings of `support(+G)` are not a drafting slip that can be settled by
picking the safe one. They are in direct opposition on the draft's own flagship:

| reading of `support(+G)` | a01 | `layout.wf` line 139 (`rtl_at`) |
| --- | --- | --- |
| **binding** (§3.4's ruling, and what `[ENT-5.G1]`'s "direct goal" implies) | **UNSOUND** — a01 accepted, OOB read | discharged: the sink's write to `deref(canvas)` does not kill `+f10_canvasfit` |
| **expansion** (what `[ENT-5.G6]`'s proof needs, and what `[ENT-5.G4]`'s "the ordinary `[ENT-5]` support of that signed goal" literally reads as) | sound | **LOST** — the sink kills the key, the depth-3 entry dies, one of the nine sites fails |

So the draft as written is not merely unsound; it is **ambiguous in a way that
changes the accepted set**, which is an independent `[ENT-1]` 2835–2836 defect.
Two conforming implementations, reading `[ENT-5.G4]`'s one sentence differently,
accept different programs — one of them a01.

### 1.6 The route menu walks the writer into the witness

DRAFT §5.4 replaces `[ENT-6]`'s route menu with:

> the route is **to test that same condition again at the use** — `[ENT-5.G]`
> admits the correlation on the edge where the condition is re-established

a01 is a writer doing exactly that. `if fits_again { … }` *is* "the same
condition again". The rule cannot distinguish "the same condition" from "the
same syntax over a changed input", and the menu gives no warning that the
difference is load-bearing. This is the worst possible shape for a defect in an
invisible mechanism: the failure has no syntax of its own.

---

## 2. Soundness attacks that FAILED — what the design got right

I ran the prompt's whole seed list against the drafted text and could break
nothing except the projection route. This is worth recording as precisely as the
break, because it says the repair is local.

| attack | outcome | why the draft survives |
| --- | --- | --- |
| **write through a borrow to a delta term** | **refused** | a delta fact carries its ordinary `[ENT-5]` support, kill (b) uses `[EFF-2]`'s boundary projection over the actuals, and the delta fact dies exactly as an ordinary fact would. No clause needed |
| **the middle work calls a callee writing the delta term through `&uniq`** | **refused** | same mechanism; `[OWN-7]` over-approximates, so it kills at least as much |
| **the flag recomputed from changed inputs, old binding still live, released by *identity*** | **refused** | `g13`/`g14` (compiled, REJECT) pin that L0 does **not** back-derive a second binding's signed goal. `+fits_again` never satisfies key `+fits` by identity. It is only the *projection* route that crosses (§1) |
| **a guarded fact about a place whose storage is reused** | **refused** | `[ENT-2]` makes a fresh binding reusing an expired spelling a distinct term and a distinct goal; kill (d) removes the entry on the edge leaving the old scope. Both mechanisms fire, and §3.4 is right that stating either is enough |
| **shadowing** | **refused** | same two mechanisms |
| **the flag written between the sites** | **refused** | the key member's support contains the flag's place, so kill (a) or (b) removes the whole entry |
| **a struct-field flag, callee writes a sibling field** | **refused, and correctly permissive** | `a11` (compiled, ACCEPT) pins that a struct field is an `[ENT-2]` term carrying L0 bounds; field-granular `writes(h.seen)` does not reach `h.has_body` |
| **contradictory-arm entry loss as a monotonicity break** | **refused** | I built this attack and it dies: if a stronger prover makes `A` contradictory, then `J' = B'`, and `B'` derives `-G` by S1, which transports to `P` on the same immutable binding and makes `P`'s state contradictory too, so every obligation there discharges. `[ENT-5.G2]`(a)'s "and is not contradictory" clause is safe |
| **`C_G` as a prover-dependent cap** | **refused** | key length is the syntactic nesting depth between formation and the current point, entries are independent, and a stronger prover only *adds* entries. §3.6's claim 2 holds |
| **release inside a loop body breaking `[ENT-5.R8]`'s induction** | **refused** | §3.5(c)'s two-case argument (formed-in-body, or survived-the-head-subtraction) is correct, and neither case consults the induction hypothesis |

The identity route of `[ENT-5.G5]` — release because S1 republishes the same
signed goal — is sound, and I could not construct a counterexample to it. That
route alone carries **all nine of the flagship's nine sites**. Reading DRAFT
§4.4's table column by column: 115 E1 identity, 119 E3 identity, 123→124 E7
reconstruction, 129 E9 identity, 133 E11' identity, 137→139 three by identity,
145→147 identity plus reconstruction, 153 identity on a negative key, 164 E5
identity. The projection route satisfies *extra* entries at 115 and 119 whose
deltas no obligation there consumes. **The projection route discharges nothing
on the design's own flagship.** Its sole stated motivation is `g13`/`g14` — two
bindings of one comparison — a shape that appears nowhere in `layout.wf`, in my
second flagship, or in any corpus site the draft names.

---

## 3. The loop seam — the same break, in loop form; the rest holds

`audit/probes/a04_loop_projection.wf`, compiled, **REJECT today** with
`[OP-4] residual: idx < len(deref(data))`. Same entry as a01, formed *before*
the loop, with the projection's term `m` written **inside the body on every
pass**:

```whitefoot
  let fits = ige(m, 8_u64);
  let idx = if fits { … give three; } else { … give fifty; }
  loop @scan {
    let done = ige(k, 2_u64);
    if done { break @scan; }
    set m = 64_u64;
    let again = ige(m, 8_u64);
    if again { let byte = deref(data)[idx]; set out = byte; }
    set k = k +wrap 1_u64;
  }
```

`[ENT-5.G3]`'s head rule subtracts *"every entry having a support member that a
continuing kill event of that loop may kill"*. `@scan`'s continuing kills are
`m`, `k`, `out`. The entry's support is `{idx, Z, fits}`. **`m` is not a support
member, so the head subtraction keeps the entry at every iteration head**, and
step (2') releases a false `idx - Z <= 3` on every pass. The loop rule adds no
protection because it, too, is written over `support`, and the projection's terms
are outside `support`.

**Everything else about the loop seam checks out.** I attacked it four ways:

- *a guarded entry as an `[ENT-5.R]` retention candidate.* §3.5(b) excludes them
  and the exclusion is correctly drawn: `C(@l)` ranges over atomic facts.
- *a flag computed inside the body.* The head rule resets the guarded component
  to the pre-loop component, so an entry formed in iteration `i` is gone at the
  head of `i+1`; it can only be consumed inside its own iteration or carried out
  on a `break` edge, where it is the actual path taken. Sound, and the draft's
  silence here is adequate because `[ENT-5.G3]` already says it.
- *does the deletion argument survive?* Yes. §3.5(c)'s case split is right, and
  `[ENT-5.R9]`'s A1 immunity is untouched — release introduces no constant the
  arm-exit state did not already hold.
- *the transitive route the draft does not mention.* A **released** fact is an
  ordinary L0 fact, so it enters `E(@l)` at the preheader edge and **is** an
  `[ENT-5.R2]` candidate. §3.5(b)'s sentence "a guarded entry is not an
  `[ENT-5.R]` retention candidate" is true and misleading: guarded content
  reaches the retention fixed point one step later, through release. This is
  sound (retention's own base case is `E(@l)`), but it has two consequences the
  draft owes a sentence on — see §7.2 and §7.3.

---

## 4. Determinism — three points where two conforming implementations diverge

`[ENT-1]`'s law is *closed, deterministic, search-free*; `[ENT-1]` 2835-2836
requires byte-identical derivation. `[ENT-5.G7]` asserts *"Two conforming
implementations hold the same guarded component at every point."* Three defects
make that false as drafted.

### 4.1 `support(+G)` is not defined (the §1.5 dilemma, restated as determinism)

`[ENT-5.G4]` says the support of a key member is *"the ordinary `[ENT-5]`
support of that signed goal — the union of the resolved places its complete
typed expression reads"*. §3.4 then rules that the key is the **direct** goal and
that a write to an origin place does **not** kill it. "Complete typed expression"
and "direct goal" pull in opposite directions, and the two readings differ on
`layout.wf` line 139 and on a01. One sentence must settle it, and which sentence
decides both the soundness of §1 and one of the flagship's nine sites.

### 4.2 The guarded component is not stated to be key-indexed, and the draft's
own flagship holds two live entries with the same key

DRAFT §4.3 has `E11` (key `[+band(f09_indented, f07_head)]`, surviving with
`Z - n <= -4` and `style.indent != Z` after `set ind_at = widened;`) and then
forms `E11'` at line 113 **with the identical key**. Both are live from line 113
onward. Now read `[ENT-5.G2]`(c) at the very next branch continuation (line 117):

> Each guarded entry present with an identical key in the states on **every**
> reaching edge, with its delta joined

With two entries sharing key `K` on each of two reaching edges, the clause does
not say whether the result is one entry (all four deltas joined), two entries
(paired how?), or one per edge-pair. Under a "join all deltas per key" reading
the surviving delta is the **weakest** of the four and `E11'`'s useful
`ind_at - n <= -4` **is lost**, because `E11` does not hold it. Under a
"per-entry" reading it survives. **DRAFT §4.4's site-133 discharge depends on
which reading an implementation picks** — and §4.3 asserts the favourable one
without licence from the rule text.

This is not hypothetical: it is realised at line 117 of the draft's own
flagship, and it decides one of the nine.

### 4.3 The omission clause is a monotonicity hazard

`[ENT-5.G2]`: *"A fact that a live entry with the identical key already holds
with a constant `c' <= c` is omitted from a new delta under (a) or (b)."*

The omission is unconditional on the omitting entry's **future**. Two same-key
entries can diverge, because clause (d) fires on *"each guarded entry present on
exactly one reaching edge"* and delta-emptiness is prover-dependent:

1. `ε_old` and `ε_new` share key `[+f]`. `ε_new` omitted `φ` because `ε_old`
   held it.
2. At a later branch, `ε_old`'s delta empties on the else edge (its facts'
   supports are written there) but `ε_new`'s does not. `ε_old` is now present on
   exactly one reaching edge.
3. Clause (c) keeps `ε_new` with key `[+f]`; clause (d) key-extends `ε_old` to
   `[+g, +f]`.
4. At a point where `+f` holds and `+g` does not, `ε_new` releases — **without
   `φ`** — and `ε_old` does not release at all.

In the weaker flow `ε_old` never held `φ`, `ε_new` kept it, and `φ` is released.
**Stronger prover, fewer facts, and a program that compiled can stop compiling.**
This is the `[CLM-2]`-class hazard §3.6 claims is gone, reintroduced by a size
optimisation. The clause buys nothing measurable (§4.2 measures 39 facts) and
should be deleted.

Note that §4.3 of the draft *also* contradicts this clause in prose: it says
`E11'` is formed "with the same key and the same delta", when the clause requires
`E11'` to omit `Z - n <= -4` and `style.indent != Z`, which `E11` still holds.

### 4.4 Two proof views, one guarded component?

`[ENT-3]`'s pipeline (0108 §3.14 step 3) walks each function **once per proof
view**, `complete` and `s4_blinded`. Formation reads closed states, which differ
per view; release writes into the state, which differs per view. The draft never
says the guarded component is per-view. If it is shared, a fact whose delta was
derived under S4 `requires` can be released into the `s4_blinded` view, which is
the view `[PRV]` uses for the external-subject partition. One sentence, and it is
load-bearing for step 8, not decoration.

---

## 5. The size bound — arithmetic correct, gap misattributed, fallback unsafe

### 5.1 Re-derivation

`[ENT-5.G7]`: at most `2·N_if` entries; each delta at most `T(T-1)` bounds and
`T(T-1)/2` disequalities; total `3·N_if·T(T-1)`.

`2 · N_if · (T(T-1) + T(T-1)/2) = 2 · N_if · 1.5 · T(T-1) = 3·N_if·T(T-1)`. ✓
`3 × 30 × 21 × 20 = 37,800`. ✓ The entry count is right: clauses (a) and (b)
form at most two entries per branch **node**, and clause (d) transforms rather
than duplicates.

Two omissions: the bound does not cover clause (a)'s `value_if` delivery
relations (which are not pair-bounds and are not counted), nor the key and
projection lists. Both are small; the bound should say so rather than read as
total.

### 5.2 The maximiser — the bound is tight, and the gap is not what §4.2 says

DRAFT §4.2 explains the 39-vs-37,800 gap as *"an arm changes two or three terms
and the entry stores a difference"*. That is the wrong invariant, and the whole
F-G2 fallback rests on it.

Take a chain `t1 <= t2 <= … <= tT` live at a branch and the single guard
`if ile(tT, t1) { … }`. The then-edge closure collapses the chain: for **every**
ordered pair `(ti, tj)` with `i < j`, `A` derives `tj - ti <= 0` and `J` does
not. That is `T(T-1)/2` delta members from an arm that **commits nothing at
all**. `audit/probes/a12_dbm_collapse.wf` (compiled, **ACCEPT**) pins the
closure step at `T = 3`. Repeat over `N_if` guards on disjoint chains and the
state reaches the stated bound within a constant factor.

So the bound is tight, not loose, and the realistic figure is small because
*ordinary programs do not collapse their zones*, not because deltas are
commit-shaped.

### 5.3 Consequence: the drafted fallback is not safe

§3.2 names the available narrowing — *"restrict the delta to ordered pairs at
least one of whose terms the arm commits"* — and calls it *"nearly lossless —
the facts it drops are consequences of the guard's own relation `R`, which S1
re-establishes at the release site anyway"*. F-G2 names it as **the** repair if
cost is refuted.

On the collapse shape the narrowing drops **100%** of the delta, because the arm
commits nothing. And the "S1 re-establishes it anyway" argument is exactly
backwards: S1 re-establishes `R`, but re-deriving the chain from `R` needs the
chain premises `ti - t(i+1) <= 0` to still be live at the release site. If they
are, the entry was never needed; if they are not — which is the only case where
storing the fact mattered — the narrowing loses it. **The narrowing is lossless
precisely when the entry is useless, and lossy precisely when it matters.**

That does not make the narrowing wrong as a *deliberate* ceiling; it makes it
wrong as a *free* one. If the owner wants it, it must be priced as a ceiling with
its own excluded shape, not held in reserve as a no-cost repair.

### 5.4 The nine-site trace does not reproduce as printed

I re-executed `[ENT-5.G2]` over `probes/layout.wf` and reproduce the nine
discharges of §4.4 — with one correction and one omission.

**Correction.** §4.1 states *"The `-band(…)` rows form nothing. A negated
conjunction publishes no conjunct and no numeric content, so the false-edge delta
is empty and no entry is formed."* This is wrong, and the reason is that a delta
is not what the *guard* publishes — it is what the *join* is about to lose. At
`@62`'s false edge `col_at` keeps its constructor default `0`, while `A` gives
`col_at = n - 2` with `n` unbounded above; the join therefore has no upper bound
on `col_at` and the false edge's `col_at - Z <= 0` is a strict improvement. A
negative entry **is** formed there, and likewise at `@66` (`hyph_at - Z <= 0`)
and `@70` (`ind_at - Z <= 0`), and at `@83` (`n - just_at <= 0`), and on the
false edge of the second `ind_at` branch at `@113` (`ind_at - Z <= 255`, from
`cvt<u8,u64>`).

**Consequence.** §4.2's measured figure — *"Seventeen live entries holding
thirty-nine atomic facts"* — is an undercount by roughly a third; the true figure
is about 24 entries. The order-of-magnitude claim survives; the measurement does
not, and F-G2's thresholds are calibrated against it.

---

## 6. The second flagship — a shape in the same family the rule misses entirely

The prompt asked for one more flagship of my own design, structurally different.
`audit/probes/a05_record_inline.wf` is a record parser: the flags **and** the
offsets they correlate with live in one struct the function itself writes, there
is an **early `return`** between the measure and use phases, the struct is handed
to a `&uniq` mutator in between, one site is under a **negative key**, and one is
**inside a loop**. No offset is re-tested at its use, no impossible `else`, no
enum, no helper carrying a `requires`.

### 6.1 What today's checker says

Neutralising each rejecting site in turn to its fixed point
(`a05_record_inline_neutralised.wf`, **ACCEPT**) enumerates the whole surface:

```
  h.body_at < len(deref(data))     if h.has_body { deref(data)[h.body_at] }
  h.wide_at < len(deref(data))     if h.wide    { deref(data)[h.wide_at] }
  h.wide_at < len(deref(data))     else         { deref(data)[h.wide_at] }
  h.body_at < len(deref(data))     inside @paint, if h.has_body
```

Four residuals, all flag correlation, nothing else — the same result `layout.wf`
produces, on a different structure. The family is real.

### 6.2 The drafted rule discharges **none** of them

The measure phase writes the flag into the record under the comparison's branch:

```whitefoot
  let room4 = ige(n, 4_u64);
  if room4 {
    set h.has_body = yes;
    let b = n -wrap 4_u64;
    set h.body_at = b;
  }
```

`[ENT-5.G2]`(a) forms `key [+room4]`, `projection 4 - n <= 0`,
`delta { h.body_at - n <= -4, Z - n <= -4 }`. The use phase tests
`if h.has_body`. `[ENT-5.G5]` asks whether `+room4` is derivable there, or
whether `4 - n <= 0` is L0-derivable there. **Neither is.** `h.has_body` is an
opaque `Bool` place datum: S1 publishes the signed goal `+h.has_body` and no
comparison projection at all.

Machine-checked as the isolating pair:

| probe | verdict | what it pins |
| --- | --- | --- |
| `a07_opaque_field_key.wf` | **REJECT** `[FN-8] instantiated_goal Unproved` | a flag *computed* as `ige(n, 4)` and *stored* into a place under that branch publishes nothing numeric when the place is tested |
| `a08_control_localflag.wf` | **ACCEPT** | the identical program with the use site testing `room4` itself |

`a06_record_localflag.wf` is the same flagship with every use site rewritten to
test the local comparison flag; it rejects with the identical four residuals, and
I hand-executed the drafted rule over it: all four discharge (identity route,
entries surviving the early `return`, the `writes(h.seen)` call, and `@paint`'s
head subtraction). **So the draft handles the local-flag rewrite and misses the
record form.**

### 6.3 Why this matters more than a missing case

1. **It is not in §5.1's price list.** The seven excluded shapes there do not
   include it. The nearest, *"a signed goal in the delta"*, is Q2 — and Q2 does
   **not** fix this. Q2 would add `+h.has_body` to the delta of the `[+room4]`
   entry, which lets the checker derive the flag *given* the branch. The program
   needs the converse. Buying Q2 buys the wrong direction.
2. **§5.4's amended route menu tells the writer to do what the program already
   does.** "Test that same condition again at the use" — `if h.has_body` *is*
   the writer's own re-test of the same condition. It does not release.
3. **The rewrite that works is plumbing.** `a06` keeps a shadow local for every
   flag already stored in the record — a second name for the same predicate,
   carried across the phases for no reason a reader can see. That is exactly the
   class of workaround §4.5 is proud that `layout.wf` does not contain, and it
   fails the intent test in §8.
4. **A prerequisite the draft does not name.** `a09_field_kill.wf` and its
   no-call control both REJECT while `a10` (a plain local literal) ACCEPTs:
   today's checker publishes **no per-field image from a struct construction**.
   Both the record flagship and `layout.wf`'s `E15`/`E16` deltas (which mention
   `style.columns`) need `[ENT-3.S5]` to cover field destinations and
   constructions. 0108 §3.4 does not obviously promise that, and DRAFT §4.0's
   prerequisite note mentions only `set x = t;` on locals.

### 6.4 The repair that would cover it, and it is cheap

The correlation the record form needs is a **biconditional**: `h.has_body` was
set true on the then edge and left false on the false edge, so `+h.has_body` at a
later point — with `h.has_body` unwritten since — implies the then arm ran. That
is a sound key, and it is derivable by the same argument `[ENT-5.G6]` already
makes:

> `[ENT-5.G2]`(a'), proposed. When the then edge derives a signed goal `+S` and
> every other reaching edge derives `-S`, additionally form an entry with key
> `[+S]`, absent projection, and the same delta; symmetrically for `-S`.

Soundness is `[ENT-5.G6]` verbatim with `S` for `G`: `+S` at `P` and no kill of
`support(S)` on `J`→`P` gives `S`'s value at `P` equal to its value at the
continuation, and only the then edge leaves it true. It needs `[ENT-3.S5]` over
`Bool` commits, which the design already owes, and it costs at most one more
entry per branch. It is the single cheapest thing that turns this rule from
"handles `layout.wf`" into "handles the family".

---

## 7. Monotonicity and the `[IND]` seam

### 7.1 The headline claim survives, with one exception

§3.6's *"no program that compiles before compiles less after"* is right in
outline, and I could break it only at §4.3's omission clause. In particular:

- **`[IND-7]` slots.** Released facts are ordinary L0 facts, so they *do* fill
  and tighten `[IND-7]` certificate slots — `[ENT-5.G1]`'s "no `[IND]` check
  reads them" is true of *entries* and false of *releases*, and the draft should
  say which it means. That is safe as it stands: 0108 §2.4's fifth repair made
  the slot list, the visit set and the elimination-term list **syntactic**, with
  contents supplied by the ambient prover and no hard error reachable from
  inside the check, and *"a slot that fills or tightens never loses a
  certificate."* `[ENT-5.G]` is exactly an ambient strengthening, so the
  N1/N2/N3/N4/N5/N6 family does not recur. I attacked this directly and found
  nothing.
- **The contradictory-arm clause.** Attacked and refused (§2).
- **`C_G`.** Attacked and refused (§2).

### 7.2 `[ENT-5.R2]`'s constant ladder now depends on release, and nothing says
whether it is recomputed

`K` is *"every bound constant appearing in any `E(@l)` of the function"*, and
`[ENT-5.R]`'s algorithm computes `K := ladder(F)` once, at line 1, before the
universe iteration. Release runs on the preheader edge (step (2') runs on every
edge), so `E(@l)` now contains released facts and their constants. Either

- `K` is computed from a no-release flow — then a candidate whose constant only a
  released fact contributes is unreachable, deterministically but arbitrarily; or
- `K` is recomputed per round — then `[ENT-5.R5]`'s termination argument, which
  rests on the fixed finite set `pairs(F) × K`, needs restating.

The draft chooses neither. One sentence, in `[ENT-5.R2]` or in `[ENT-5.G5]`.

### 7.3 The guarded component is inside `[ENT-5.R]`'s fixed point and the
algorithm has no line for it

`[ENT-5.G3]` derives a loop head's guarded component from *"the state before the
loop"*. Under `[ENT-5.R5]`'s outer universe iteration that state changes between
rounds, so the guarded component must be recomputed each round — and the
fourteen-line algorithm of 0108 §3.6.2 has no step that does it. `[ENT-5.R3]`'s
fixed order at the head (*preheader establishment and closure; continuing-kill
subtraction; retention; S11 bounds; bound publication; closure*) also has no slot
for the guarded-component subtraction. Both need editing, and §3.14's pipeline
step 2 needs a clause. The draft says "nothing below asks any of those to move";
this asks two of them to move.

### 7.4 `[ENT-5.G7]`'s monotonicity argument has a gap it does not notice

The argument runs: *"a fact released in the weaker flow is, in the stronger flow,
either in `δ` again … or already in `J` and transported unconditionally under the
identical kill set."* The third case is missing: in the stronger flow the fact
may be in `δ` of a **differently-keyed** entry — §4.3's scenario — or omitted
from `δ` by the omission clause. Deleting the omission clause closes the gap and
makes the two-case argument true.

---

## 8. The intent test — is any of this plumbing the writer must write?

**Annotations: none.** Zero. That part of the design is unimpeachable, and it is
the strongest thing about it. `[ENT-5.G]` adds no keyword, no attribute, no
`because`, no statement, and no obligation on the writer. `layout.wf` is 186
lines of ordinary layout code with no ceremony in it.

**But invisibility cuts both ways, and the draft only counts one side.**

1. *The failure mode has no syntax.* When a release does not happen, nothing in
   the program shows why. §5.2's `correlation` diagnostic is the right instrument
   and I think it is the most valuable single paragraph in the draft — but it is
   named as a diagnostic improvement, when it is actually the **only** channel
   through which this mechanism is legible at all. It should be treated as part
   of the rule, not as polish, and it needs a third string for the case in §6.2
   (the key exists, the writer tested a *different* proposition that means the
   same thing).
2. *The record rewrite is plumbing.* `a06` versus `a05`: the writer must keep a
   local shadow of every flag they already stored in the record, because the
   rule keys on the branch condition and not on the stored flag. That is a second
   name for one predicate, carried across two phases, existing only to satisfy
   the checker. It is the same class of thing as `g16`'s re-test, which §4.5
   correctly refuses. §6.4's `(a')` clause removes it.
3. *The unsound case is also invisible.* a01 is not exotic code. "Compute a flag,
   branch, later re-test the flag, meanwhile the input moved" is a normal thing
   to write, and the rule silently converts it into an out-of-bounds read with no
   syntax to inspect and no diagnostic to read. **A rule this invisible must be
   sound by construction**, because the writer has no way to audit it and the
   reviewer has nothing to review.

Point 3 is why I do not think the projection route should be repaired by a
liveness side-condition and kept. See §9.

---

## 9. Repairs

### R1 — the soundness repair (required)

Two forms, and I recommend the first.

**R1a — delete the projection route.** `[ENT-5.G5]`'s satisfaction becomes
"`s_i` is derivable there", full stop; `[ENT-5.G1]` drops the projection list.
Cost, measured against the draft's own trace: **zero of the flagship's nine
sites** (§2), zero of my second flagship's four, and the loss is exactly `g13`'s
shape — two distinct bindings of one comparison — which no program in either
flagship or in any named corpus site exhibits. Gain: `[ENT-5.G6]`'s proof becomes
true as written, a01 and a04 are refused, and the rule loses a whole state
component.

**R1b — give the projection its own liveness.** Keep the route, and add: *a key
member may be satisfied through `R_i` only when no kill event since the entry's
formation has killed the support of `R_i`* — equivalently, put `support(R_i)`
into the entry's support. Sound: `R`'s terms unwritten ⇒ `R` at `P` equals `R` at
the branch, and S1's exactness makes `R` at the branch equivalent to `G` there.
Costs the flagship nothing (`n = len(deref(glyphs))` is never written, so `E5`
still releases by projection at 115/119, and the `f10_canvasfit` key still
survives the sink call, because the projection is over the local `room` and not
over `deref(canvas)`).

R1a is better because of §8's point 3: R1b keeps a route whose correctness
depends on a condition the writer cannot see and whose failure is a memory-safety
bug, in exchange for a shape nobody writes.

### R2 — define `support(+G)` (required either way)

One sentence in `[ENT-5.G4]`: the support of a key member is the support of the
**direct** goal — the resolved places its own binding or place reads — and not
the places of its origin expansion. This is what §3.4 argues for and what the
flagship needs; under R1 it is also sound.

### R3 — delete the omission clause in `[ENT-5.G2]` (required)

It is a size optimisation that buys nothing at the measured figure and costs the
monotonicity theorem (§4.3).

### R4 — say the guarded component is key-indexed, or say it is not (required)

`[ENT-5.G2]`(c) and (d) are ambiguous the moment two live entries share a key,
which happens at line 117 of the draft's own flagship (§4.2). My recommendation:
key-indexed, with formation **unioning** into an existing key's delta rather than
forming a second entry, and (c) joining per key. That also makes R3 free, since
there is then no second entry to omit against.

### R5 — buy `(a')`, the committed-flag key (recommended, not required)

§6.4. Without it the rule covers `layout.wf`'s idiom and misses the
record-of-flags idiom, which is at least as common. With it, both flagships
compile. Its prerequisite — `[ENT-3.S5]` over `Bool` commits and field
destinations — is owed anyway (§6.3 point 4).

### R6 — the three seam sentences (required)

`[ENT-5.R2]`'s ladder (§7.2), `[ENT-5.R3]`/`§3.6.2`'s per-round recomputation
(§7.3), and per-proof-view guarded components (§4.4).

---

## 10. What the owner must decide

The draft's own Q1–Q6 are the small questions. These are the ones this audit
puts on the table, in the order they block.

**O1 — Delete the projection release route, or keep it with a liveness
side-condition?** (R1a vs R1b.) This is the soundness decision and it is not a
drafting matter: it trades a shape nobody has written (`g13`) against a route
whose failure mode is an out-of-bounds read with no visible syntax. My
recommendation is delete. The owner's call because it is the one place where this
design's reach is deliberately narrowed.

**O2 — Is the key the direct goal or the expansion?** (R2.) One sentence, and it
decides both a01's fate and `layout.wf` line 139's. They cannot both be had
without O1 being answered first.

**O3 — Does the rule key on the branch condition only, or also on a committed
flag?** (R5, `(a')`.) This is the difference between a rule that handles the
draft's flagship and a rule that handles the family. It is where the ledger row's
"buys" column should be re-costed: as drafted, V8 buys `layout.wf`'s idiom, not
"the whole flag-correlation family" as §5.3 claims.

**O4 — Is `[ENT-1]`'s determinism law satisfied by a rule whose state component
is not defined as a set, a multiset, or a map?** (R4.) Formally this is a defect
either way; the owner's decision is only which reading, and whether the resulting
loss at `layout.wf` line 133 is acceptable if the per-key-join reading wins.

**O5 — Is the `correlation` diagnostic part of the rule or part of the polish?**
§8. I hold it is part of the rule and should ship in the same batch, because it
is the only thing that makes an invisible mechanism auditable. That is a
scheduling decision the owner owns.

**O6 — Should F-G2's fallback be re-specified?** §5.3 shows the syntactic
narrowing is not the free repair the draft holds in reserve. Either it is priced
as a ceiling with its own excluded shape, or F-G2 needs a different repair named.

---

## 11. Probe ledger

Thirteen files in `audit/probes/`, all compiled against the unmodified worktree
compiler. `audit/run.sh` reproduces every verdict.

| probe | verdict | role |
| --- | --- | --- |
| `a01_projection_retest.wf` | REJECT `[OP-4] idx < len(deref(data))` | **the refutation.** Accepted under drafted `[ENT-5.G]`; reads index 50 of a 4-byte buffer |
| `a02_control_reprojection.wf` | ACCEPT | premise 1: the stored projection is L0-derivable after the term is written |
| `a03_control_discharge.wf` | ACCEPT | premise 2: the released fact discharges the subscript |
| `a04_loop_projection.wf` | REJECT, same residual | the same break through `[ENT-5.G3]`'s head subtraction |
| `a05_record_inline.wf` | REJECT, four correlation residuals | **second flagship**; the drafted rule discharges none |
| `a05_record_inline_neutralised.wf` | ACCEPT | the fixed point: the whole rejection surface is the correlation |
| `a06_record_localflag.wf` | REJECT, same four residuals | the local-flag rewrite; the drafted rule discharges all four |
| `a07_opaque_field_key.wf` | REJECT `[FN-8]` | a stored `Bool` flag publishes no comparison projection |
| `a08_control_localflag.wf` | ACCEPT | the same program testing the comparison flag |
| `a09_field_kill.wf` | REJECT | struct construction publishes no per-field image today |
| `a10_local_literal.wf` | ACCEPT | the contrast to a09: a plain local literal *does* carry an L0 fact |
| `a11_field_is_term.wf` | ACCEPT | a struct field *is* an `[ENT-2]` term carrying L0 bounds, so a09's rejection isolates construction, not fields |
| `a12_dbm_collapse.wf` | ACCEPT | one guard collapsing a chain — the size maximiser's closure step |

**What this ledger does not establish.** No line of `[ENT-5.G]` is implemented.
a01 is a hand-execution of the drafted rules over a state no compiler prints. It
is refuting because every premise it needs is machine-checked separately (a02,
a03) and the only missing step is the one the draft specifies. If the owner
wants it converted to a measurement, implementing `[ENT-5.G5]` step (2') alone —
no formation, one hand-supplied entry — is a half-day and would settle it.
