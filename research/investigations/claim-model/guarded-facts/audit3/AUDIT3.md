# Round-3 replay audit of `[ENT-5.G]` — verdict: CONFIRMED

Target: `../DRAFT.md` (round 3, 2,505 lines). Compiler: the 0106 worktree
release binary, unmodified; `git status --porcelain` empty before and after.
`./run.sh probes/*.wf`, `./audit/run.sh`, `./audit2/run.sh` and
`./audit3/run.sh` reproduce all 58 verdicts.

## 1. Replay of both prior audits' witnesses against the final text

| witness | required | replayed outcome |
| --- | --- | --- |
| `a01`, `a04` | refuse | refuse at release (D1: one route; `+fits` not derivable — 2915(b)'s no-kill restriction voids the comparison origin, verified at `kernel-spec.md:2915`) |
| `c01`, `c02` | refuse | refuse at formation ((a)/(b) by the arm-write test; (a′) same-sign) |
| `b01`, `r09` | refuse | refuse identically: (a)/(b) at the outer continuation, L-G4(ii) at the carry-out |
| `b04` | refuse | refuse by derivation; §8.3's six steps re-executed and each is right |
| `a05`, `layout.wf` | discharge | formation admitted at every continuation; all 4 + all 9 releases re-walked and close |
| `a12`, `r07` | inside the bound | a12: 1 if node, T≈5, ≈10 members vs 60; r07: T≈4 vs 36 |
| E11/E11′ | one outcome | `layout.wf:67` and `:110` are both at 2-space indent, so `chain` is empty at both and the keys are literally identical; union at formation; site 133 discharges |
| omission divergence | impossible | the clause is gone; grep finds it only in historical prose. No live clause reads a fact's absence |

## 2. Law-by-law violation hunt

- **L-G1**: no live presence condition reads an absence. The arm-write test is
  syntax + kill events (identical in both flows); (a′) is two positive
  derivability claims and fires strictly more often in a stronger flow, including
  when an arm exit becomes contradictory. "Both edges reach `C`" is the syntactic
  3097 sentence, not semantic reachability — checked, and it matters.
- **L-G2**: every union in the draft is over a row present throughout the span.
  I could not build a union the lemma does not cover. But the lemma's *stated*
  premise is presence, while the carry-out consumer can only supply "no kill" —
  see residual R1.
- **L-G3**: no formation over a written support. Chain members are not tested at
  formation, and I could not turn that into a hole: a write to a chain member's
  support before the formation point makes the key unsatisfiable thereafter, and
  a write after it kills the entry.
- **L-G4**: `(ii)` is normative and consistent at all four sites. `(i)` has a gap
  — see attack 1.

## 3. Three new attacks

**A1 — `t01_goalless_chain_gap.wf` (REJECT, `[OP-4] x < len(deref(data))`;
control `t01c` ACCEPT).** A `chain(C)`-invisible enclosing branch
(`if classify(v: v)`, a user call, no goal origin per `kernel-spec.md:2923`).
Under the loose reading of L-G4(i) the entry is carried out of a branch nothing
in its key records and index 50 of a 4-byte buffer is read. The strict reading
is safe and is what the clause's own gloss means. **Refused, but the
disambiguation rests on prose where (ii)'s rests on a quoted phrase.**

**A2 — the entry bound's derivation.** A Bool commit nested in two arms is in the
(a′) candidate set of two continuations, so §3.7's "summing over continuations"
undercounts. The bound survives, by an injection that (a′)'s opposite-decision
condition supplies. **Refused for a reason the draft does not state.**
`t02_nested_bool_commit_count.wf` (ACCEPT) is the shape.

**A3 — the `give` clause.** §3.2's non-reaching sentence adds `give` to a
sentence it calls "3097's sentence, unchanged"; 3097 has no `give`, and
`kernel-spec.md:1032` makes a `give` edge reach the initializer's continuation.
**Refused as intent** (correct for an `if_stmt` continuation), wrong as written
for a `value_if`.

## 4. Size bound, re-derived

`2(N_if + S_B)` entries (a theorem, with A2's missing step), `1.5·T(T-1)` facts
per delta, `3(N_if + S_B)·T(T-1)` total. `render_line`: `N_if = 30` (counted
from source), `S_B = 0` → 60 entries, 37,800 facts. Dead entries are outside the
map and the diagnostic reconstructs them, so the bound is true as written.
Flagship A recount: **22 live entries at 114** (11 continuations × 2, with @84,
@107, @113 unioning and @98 forming nothing) — exact, factor 2.7 under. §6.3's
`|δ|` column sums to **251** over those 22 rows.

## 5. Probes

| probe | verdict |
| --- | --- |
| `t01_goalless_chain_gap.wf` | REJECT `[OP-4] x < len(deref(data))` |
| `t01c_joined_control.wf` | ACCEPT |
| `t02_nested_bool_commit_count.wf` | ACCEPT |
