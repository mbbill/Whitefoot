# Comparison CORE-2: three core models with their rules written out

Research date: 2026-09-16. Second round on the core model: the model
[VERDICT-CORE.md](VERDICT-CORE.md) selected (value semantics with projections
and pool handles), the runner-up (window-scoped fine names over coarse store
names), and the earlier Candidate A with condition terms bounded to enclosing
guards, each forced to write its rules for creating and ending storage, for
splitting and rejoining state, and for the bridge between an identity and the
runtime values that determine it, then derived line by line on P1, P3, P4, P7,
P8 and the twenty-six cases of [CASES.md](CASES.md), cross-examined, and
compared without selection. The completeness review follows. Record in
EVIDENCE-debate-core2-2026-09-16.md (`EVIDENCE-debate-core2-2026-09-16.md`, removed in the cleanup, recoverable at commit 4ca62f8db758).
Nothing here is selected; the owner has not ruled. Supersede in place.


Round: core2. Date 2026-09-16. Sources: `rules-{value-semantics,window-focus,brand-context-bounded}.md`,
the nine `derive-*.md` files, the nine `critique-*.md` files. Requirements from
`VERDICT-D0.md` §2/§7; hazard ladder style from `MECHANISM-MAP.md` §3; programs
from `PROGRAMS.md`; cases from `CASES.md`; meta-finding from `VERDICT-CORE.md` §6.

**This file selects nothing and recommends nothing.** Short keys: **VS** =
`value-semantics`, **WF** = `window-focus`, **BCB** = `brand-context-bounded`.

---

## 1. Verdict matrix

### 1.1 Legend

A cell states the candidate's disposition of the item **relative to the item's
own expected verdict** (`CASES.md` for S/B/L rows, `PROGRAMS.md`'s required
properties for P rows):

| Label | Meaning |
|---|---|
| `accept` | the expected verdict is derived, nothing written |
| `accept+cost` | the expected verdict is derived, with written annotations, a restructuring, or one stated property not delivered |
| `refuse+repair` | the candidate differs from the expected verdict deliberately, and names a repair |
| `CL` | capability loss — differs, and no local repair exists |
| `underivable` | no rule of sets I/II/III reaches the line |

Markers:

| Mark | Meaning |
|---|---|
| `†` | **corrected cell** — a critique's "verdicts overturned" table overturned the derivation's own verdict; the label shown is the corrected one |
| `‡` | the verdict stands but rests on a rule the candidate does not state (a derivation's own "rule missing", or a critique's missing-rule finding) |
| `⚠` | a critique finds a hazard (R1/R2/R3/R4/R5/R11) or an M2(ii) breach admitted at this item |

One direction note, because it recurs: for VS's **DV-4 family** (S03, S06, B03,
B04, B07, B09, L02, and P7 line 1) `refuse+repair` means *VS declines the case's
expected REJECT* — `replace(inout s, None)` is total, so the hole read accepts
and returns `None` — and names the repair (declare the slot `Option`, write the
arm). It is a refusal of the case's refusal, not of the program.

### 1.2 The matrix

| Item | `value-semantics` | `window-focus` | `brand-context-bounded` |
|---|---|---|---|
| **P4** cursor over a growing vector | `CL` ⟨1⟩ | `accept+cost` †‡⚠ ⟨2⟩ | `accept+cost` †⚠ ⟨3⟩ |
| **P7** take/put/replace through aliases | `refuse+repair` † ⟨4⟩ | `accept+cost` †⚠ ⟨5⟩ | `accept+cost` † ⟨6⟩ |
| **P8** conditional release, loop exits | `CL` †⚠ ⟨7⟩ | `underivable` † ⟨8⟩ | `accept` †⟨9⟩ |
| **P1** container split at a runtime index | `accept+cost` †⚠ ⟨10⟩ | `accept+cost` †‡ ⟨11⟩ | `accept+cost` †‡ ⟨12⟩ |
| **P3** graph with back edges in a pool | `underivable` † ⟨13⟩ | `accept+cost` ‡⚠ ⟨14⟩ | `accept+cost` ‡⚠ ⟨15⟩ |
| **S01** aliases, stable copies | `accept+cost` † ⟨16⟩ | `accept` †‡ ⟨17⟩ | `accept` ‡ ⟨18⟩ |
| **S02** replacement transfers the old value | `accept+cost` † ⟨16⟩ | `accept` ‡ ⟨19⟩ | `accept` ‡ ⟨19⟩ |
| **S03** hole is a property of the target | `refuse+repair` ‡ ⟨20⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨22⟩ |
| **S04** another alias restores the hole | `accept+cost` † ⟨16⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨22⟩ |
| **S05** hole need not be restored before end | `accept+cost` † ⟨16⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨22⟩ |
| **S06** replace requires old content | `refuse+repair` ‡ ⟨20⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨23⟩ |
| **S07** release is permanent, every path | `accept` | `accept` ‡ ⟨24⟩ | `accept` ‡ ⟨18⟩ |
| **B01** one locator, alternative targets | `accept+cost` † ⟨25⟩ | `accept` | `accept` ‡ ⟨26⟩ |
| **B02** target/initialization correlated | `refuse+repair` ⟨27⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨22⟩ |
| **B03** write initializes the selected target | `refuse+repair` ‡ ⟨20⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨22⟩ |
| **B04** copied uncertain target, take/put | `refuse+repair` † ⟨28⟩ | `accept` ‡ ⟨21⟩ | `accept` † ⟨29⟩ |
| **B05** correlated targets, selected release | `CL` † ⟨30⟩ | `underivable` † ⟨31⟩ | `accept` † ⟨32⟩ |
| **B06** rebinding does not retarget copies | `CL` † ⟨33⟩ | `underivable` † ⟨31⟩ | `accept` ‡ ⟨18⟩ |
| **B07** conditional hole, scalar overwrite | `refuse+repair` ⟨20⟩ | `accept` ‡ ⟨21⟩ | `accept` ‡ ⟨22⟩ |
| **B08** repeating an unchanged condition | `accept+cost` † ⟨34⟩ | `accept` ‡ ⟨21⟩ | `accept` † ⟨35⟩ |
| **B09** conditions are captured values | `refuse+repair` † ⟨36⟩ | `accept` † ⟨37⟩ | `accept` ‡ ⟨22⟩ |
| **B10** join must not erase the obligation | `refuse+repair` ‡ ⟨38⟩ | `underivable` † ⟨39⟩ | `accept` ‡ ⟨40⟩ |
| **B11** locator names the remaining obligation | `CL` (pool) / `refuse+repair` (binding) ⟨41⟩ | `underivable` † ⟨39⟩ | `accept` ‡ ⟨26⟩ |
| **B12** two conditional releases | `refuse+repair` ⟨42⟩ | `underivable` † ⟨39⟩⟨37⟩ | `accept` ‡ ⟨22⟩ |
| **B13** conditional scope cleanup | `refuse+repair` ⟨43⟩ | `refuse+repair` † ⟨44⟩ | `refuse+repair` † ⟨45⟩ |
| **L01** take/restore preserve the head | `accept+cost` ‡ ⟨46⟩ | `accept` ‡ ⟨21⟩ | `accept` |
| **L02** empty exit becomes the next input | `refuse+repair` ⟨20⟩ | `accept` ‡ ⟨47⟩ | `accept` † ⟨48⟩ |
| **L03** relative invariant, alternating roles | `CL` † ⟨49⟩ | `CL` † ⟨50⟩ | `underivable` † ⟨51⟩ |
| **L04** release on a break edge | `refuse+repair` ⟨52⟩ | `accept` ‡ ⟨53⟩ | `accept` ‡ ⟨54⟩ |
| **L05** checked Boolean guards later iterations | `CL` † ⟨55⟩ | `accept+cost` † ⟨56⟩ | `CL` † ⟨57⟩ |
| **L06** a hole may leave through break | `accept` ‡ ⟨58⟩ | `accept` ‡ ⟨21⟩ | `accept` ⟨59⟩ |

### 1.3 Counts

| | `accept` | `accept+cost` | **accepted total** | `refuse+repair` | `CL` | `underivable` |
|---|---|---|---|---|---|---|
| **VS**, as derived | 2 | 10 | **12** | 15 | 3 | 1 |
| **VS**, corrected | 2 | 8 | **10** | 13 | 7 | 1 |
| **WF**, as derived | 24 | 6 | **30** | 1 | 0 | 0 |
| **WF**, corrected | 18 | 5 | **23** | 1 | 1 | 6 |
| **BCB**, as derived | 24 | 5 | **29** | 1 | 0 | 1 |
| **BCB**, corrected | 24 | 4 | **28** | 1 | 1 | 1 |

Corrected cells (`†`): **VS 16**, **WF 14**, **BCB 11** of 31.

Movement under the critiques: VS **−2** (P8 and L03 fall from accepted to `CL`);
WF **−7** (P8(a), B05, B06, B10, B11, B12 fall to `underivable` for one missing
rule — the join of `Own` — and L03 to `CL`); BCB **−1** (L03 falls to
`underivable`). B11 is counted under `CL` for VS; its binding rendering is a
`refuse+repair`.

### 1.4 Cell notes

| # | Note |
|---|---|
| ⟨1⟩ | DV-1/DV-2: a projection is never stored (§1.1, II-19) and `Handle<P,T>` names a pool row only (I-18); the named repair (thread `v.data`, `v.meta` through every intermediate signature) is non-local. The index-only rendering delivers the other two conjuncts exactly. Not overturned by any critique. |
| ⟨2⟩ | As spelled the derivation does not hold: the **first** read is not derivable (`c.i` is a field load, not §1.1's index term — soundness S2), the **second** read still rejects after the named `reserve` repair (II12's reserve row is two-armed — soundness S1, confirmed by BCB-cross O1). Both repairable inside the candidate (formation or written-invariant capacity fact). Cost understated by one identity parameter + one `where` (cross X8); no numbered rule for a nominal's store parameters. ⚠ soundness F1: the accepting route lets a declared invariant conclude `St` from a measure — the `set_len` hazard, R1(a)(i). |
| ⟨3⟩ | ⚠ soundness F2: `I-14` closes a contract-asserted state downward only for the constant `Gone`, so the pre-push fact `st('b[0]) = Init` survives the reallocating leaf and `c.read()` is **accepted** — use-after-free, R1(ii). Cross F6: no rule rebuilds a stored nominal at the new backing, so only the non-reallocating arm has a continuation. F5/S13: `I-17`'s example rejects and `III-3`'s example accepts the *same line*. Cost: 2 identity params + 1 `where` + 2 `FREEZE` lets + 1 writer branch. |
| ⟨4⟩ | 3 of 4 required lines exact in the pool rendering; line 1 (`read(q)` after `take`) accepts returning `None` (DV-4). † soundness S7: the derivation's own correction to DV-3 makes R3(e)'s acceptance test **writable**, and the candidate then fails R3(a)'s hole clause on it — R3 is partial with a live, failing test, not "no test exists". |
| ⟨5⟩ | † soundness F3: `I6`'s worked example ("discharged by I7's closure") and `I7`'s premise ("every obligation inside ρ is discharged") contradict, and §1.2 has no place-keyed obligation structure. Outcome is either an R2 leak or a **compiler-inserted recursive release** = ⚠ M2(ii), undeclared. Two writer lines charged; the value claim `read(q) // reads new` is not derivable (M3, G4). |
| ⟨6⟩ | † soundness S5: the access accept stands; the *contents* claim ("reads `new`") is overturned — `I-9` kills `val` and **no rule installs `val`** (S17). Two writer lines with an affine content, zero with a copy content. |
| ⟨7⟩ | † soundness S1 + F5. P8's own required conjunct 2 ("the writer's own branches carry the state") is exactly what II-2 refuses; conjunct 1 ("no drop flag") is delivered only by relocating the flag into source — the mandated `var slot: Option<T>` + `replace` + trailing `match` is ⚠ a runtime release-selecting branch on a predicate the checker did not discharge (M2(ii) in letter only, declared satisfied). P8 is scoped by `PROGRAMS.md` to B10–B13 and L04–L06, five of which reject and repair with that flag. |
| ⟨8⟩ | † cross X3 + soundness S4: `Own`, the obligation state, **has no join rule** (II3 merges `St` and `Facts` only) and §1.2 contains no binding→identity map, so P8(a)'s `free(s)` on the bound survivor is not derivable. P8(b) accepts. |
| ⟨9⟩ | † soundness S1: the join term must be `ite(c, Gone, Init)`, not `ite(c, Gone, Live)`; as written the derivation accepts `use(pa)` at `Live`, which R1(i) refuses. The "no drop flag" conjunct genuinely holds — II-3's terms are erased checker state and every branch is the writer's. |
| ⟨10⟩ | † cross F1: ⚠ **R8a hazard admitted and undeclared**. I-9/I-10/I-11/I-13/I-21 carry no "no projection is open" premise, so II-17 *permits* `reserve(inout v.meta,128)` and a reallocating `push` inside `with inout v.data[i] as pi { … }` (sibling planes, PATH refutes overlap). P1's required ordering property ("push allowed only after both parts are back") is enforced by no rule. |
| ⟨11⟩ | † cost F5 + cross X4/X16: P1's own second route ("else the writer must branch") is **underivable** — II1 says an expression discriminant captures nothing and II5 leaves the atom free of its defining predicate, so no source branch can discharge an index fact. The sole admitting route is `use k != i && k != j`, a `use` over a runtime-scalar disequality with no written invariant behind it. M7(a) violated undeclared. Cost: 4 annotations + 1 `use`. |
| ⟨12⟩ | † cross F8: the cost is **overstated** — `JF-ENT`'s own stated inputs include `requires`, so the five index facts can be written as a callee `requires` with no runtime branch (VS's rendering), not only as the derivation's branch + outcome arm. ‡ two gaps: no distinctness effect for an `I-2` frame mint; `II-3` has no row for a bridge fact or an `RSt` residual list at an `if`-join. |
| ⟨13⟩ | † cross F2: three load-bearing rules missing — `I-20`'s two-part footprint cannot cover a linked-list `remove`'s neighbour writes (and III-3 cannot resolve their load-derived index terms); `INV` has no witness rule to re-derive `LIVE` from `Links`; `Links` admits no terminator. **R6(e)'s only named acceptance test is P3**, so R6 is undelivered and undeclared. Cost critique S6 adds that `I-20`'s own `use Links(other)` example must reject, so DV-6's only named repair does not exist. |
| ⟨14⟩ | ⚠ cross X6: after `remove(P,m); insert(P,x)` reusing `m`, WF **accepts** the stale read and returns the new occupant with no diagnostic (III11 declares a generation absent; OD1). R1(e) names P3's removal clause as a test of R1; both rivals refuse it by rule (VS III-7, BCB III-6). Declared, hence not fatal. Cost: ≥5 annotations, ≥1 `INV` step per traversal read site. |
| ⟨15⟩ | ‡ wholly conditional on owner decision **O3** (D5). ⚠ soundness F5: `JF-LIN` schema 9 makes ghost-index inequality a *disjointness* proof feeding R4/R5, contradicting III-6's "never emitted as a `#` fact" — reused arena bytes and one pool slot can be proved disjoint, exposing P3's parallel map independently of O3. Cost: 1 pool invariant + 1 per traversal loop head + 1 `use` per symbolic-write group per iteration + 1 branch per traversal read. |
| ⟨16⟩ | † cross S1/S2: the program accepts, but **the property the case is written to exhibit is not derivable**. MR-1: no use-direction path equality, so a fact at `g.slots[p.idx]` cannot discharge a goal at `g.slots[q.idx]` given `p == q`. MR-2/MR-3: `I-19` (pool insert) states no contents `ensures` and `I-7` (`replace`) states none at all, so no printed value (10, 11, 20, 9) is in `Γ` in the pool rendering. |
| ⟨17⟩ | † cross X2: rests on an **unwritten `DIS` base case** for two identities at distinct `⊑`-roots; `DIS` is declared total, and R4(b)/I11/III6 forbid the only available ground (name inequality). Load-bearing: `contents(ρa)` must survive a write to `ρb`. |
| ⟨18⟩ | ‡ soundness F8: `'A # 'B` has no writable source — `I-1` requires the allocator's written `ensures` to range over the *caller's* live identity set, which the vocabulary cannot express, and `I-2` mints no distinctness; both derivations take the name-inequality route R4(b) forbids. Also ‡ G1: **no numbered rule for `read`**, the operation every R1 verdict rests on. |
| ⟨19⟩ | ‡ no `ensures` carries a value out of storage (WF G4; BCB G2/S17: `JF-KILL` kills `val`, nothing installs it). Acceptance unaffected; the printed values are not derivable. |
| ⟨20⟩ | DV-4: the case's expected REJECT has no instance — `replace(inout s, None)` is total (I-7) and the read of an `Option` is total (III-11). Repair named: declare the slot `Option` and write the `match` arm. M2(ii)(i) intact; the writer who omits the arm gets a silent `None` instead of a diagnostic. |
| ⟨21⟩ | ‡ cross X11 / gap G1: `I4`'s premise requires `type(π)`'s class to be **affine or linear**, while the whole `CASES` fragment — and `I4`'s, `I3`'s and `II7`'s own worked examples — `take` copy integers. The `take` line of fourteen items has no rule under the premise as written. |
| ⟨22⟩ | ‡ G1: no numbered rule for `read`; ‡ G3: no numbered rule forms or copies a pointer (`p = ref(a)`, `q = p`). |
| ⟨23⟩ | ‡ G4: no rule fixes intra-statement operand evaluation order, so S06's stated principle is not derivable in general (this instance derives because the take is its own statement). |
| ⟨24⟩ | ‡ G3: release through a locator has no rule — `I7`'s premise is written `Own(x) = ρ` and no clause says whether `x` is the syntactic argument or the binding owning the resolved `ρ`. §3.21's S07 row requires the second reading. |
| ⟨25⟩ | † cross S10 / MR-4: accepted only in a rendering the case does not write; the natural `let p = if c { … } else { … }` is uncovered — **no rule states which `Γ`/`G` facts a name bound at a join from an arm-yielded value carries**, and that form is also II-2's own headline repair for B10/B12/B13/L04. |
| ⟨26⟩ | ‡ G3: no rule forms or copies a pointer; `II-14`'s origin split states only the arm case plus its copy clause. |
| ⟨27⟩ | The swapped variant that should REJECT also accepts: the correlation `cond ⇒ is_none` is exactly the condition-indexed fact II-3/II-5 refuse (O11(a)). The sharpened variant is **unrepairable** under O11(a). |
| ⟨28⟩ | † soundness S11: two classes, not one — `DR` for the intermediate reads, **`CC` for the case's conclusion "afterward both original slots are initialized"**, to which MR-3 leaves no route at all (no contents `ensures` on pool insert). |
| ⟨29⟩ | † soundness: stands **conditional on F8** — the `'A # 'B` every line cites has no writable source. |
| ⟨30⟩ | † cross S3 + soundness S4: cause re-attributed. Appendix B files B05 under DV-6 with the repair "a written distinctness premise on the pool interface"; the line-by-line derivation shows the straight-line form **accepts**, and what removes the input is **II-3's syntactic join** discarding `σ(p) != σ(q)` — O11(a). No pool-interface premise can repair a fact about two caller-local variables, so M7(a)'s "a repair that exists" fails. |
| ⟨31⟩ | † soundness S4: both `release` lines need a **guarded `Own`**, which no rule provides (§1.2's `Own` is unguarded and II3 does not merge it); † cross X2: the surviving read also rests on the unwritten `DIS` base case. |
| ⟨32⟩ | † soundness S4: the `st` half of every line stands; the **`own`/`obl` half is unruled** — `II-15`'s table has columns for `read` and `write/take/put/free` but its cells describe only the effect on `st`, and no rule says what a release through a non-singleton origin does to `own` or `obl`. Plus F8. |
| ⟨33⟩ | † cross S3: cause is **not** B05's — `II-12` kills `saved == p` at the rebinding `p = q` and no rule substitutes `saved == ha` first (the MR-1 family), so B06 rejects **even with the branch deleted**. |
| ⟨34⟩ | † cross S9: recorded as a *correction of the case* (`CC`); overturned to a **precision loss under O11(a)** — the refinement `is_some(a)` is genuinely not recovered, and both rivals derive the case's ACCEPT *with* the refinement, which is evidence the case has an instance. The derivation and Appendix B also disagree on the class. |
| ⟨35⟩ | † soundness S13: stands **only under the SSA reading** of a guard's frozen value; under `III-1`'s `FREEZE` reading (a fresh frozen value per application, defined for measure heads) the second `if cond` is a new atom and B08 rejects. Both derivations take the SSA reading without saying so. |
| ⟨36⟩ | † cross S9: recorded `CC`; overturned to `DR` under DV-4 — both rivals reproduce the case's REJECT by rule. III-10, the rule the case is about, never fires, because II-5 discards the premise at the join before `cond = !cond`. |
| ⟨37⟩ | † cost F4 / cross X5: the accepting `if !cond { … }` variant is **underivable** — `II1`'s premise admits only a boolean *binding* and states that an expression discriminant "captures nothing", while `II5`, `II6`, `I8`'s example and §3.21 all check the arm under `¬atom(cond)`. The derivations follow the worked examples against the rule. |
| ⟨38⟩ | REJECT at the join by II-2's definiteness clause; repair named (`let keep = if c { sink a; b } else { sink b; a }; sink keep`) — but the repair's own rule is missing (MR-4, note ⟨25⟩). ‡ Also: no rule states whether a live pool **row** carries an R2 obligation, so under the pool rendering II-2's definiteness clause is bypassed entirely and B10's hazard is unchecked and unstated. |
| ⟨39⟩ | † cross X3: `Own` has **no join rule**; §3.21's "obligation retained, guarded" is asserted, not derived. B11 additionally needs `I7` to end an obligation at a guarded identity `ite(c,ρb,ρa)`, which G3 records as unstated. |
| ⟨40⟩ | ‡ cross F2: **`obl` has no join rule and no term form** (`II-3`'s table covers `st`/`own`/`origin`; `TERM` admits state/own/origin terms), so after one conditional `take` on an obligation-carrying content `I-5`'s guard is bypassed and the obligation is dropped on the `¬c` leaf. F9: `II-3` is not total over the candidate's own fact components. |
| ⟨41⟩ | **The O7/O12 boundary appearing inside a case.** Binding rendering: REJECT at `remaining = ref(b)` (II-19, a projection cannot be the value of a binding) and again at the join (II-2); repair = `let remaining = if c { sink a; b } else { sink b; a }`, after which both lines accept. Pool rendering — the rendering that preserves the case's storable-locator shape — `read(remaining)` rejects for missing `Live(remaining)` (I-20/III-8 kill liveness for every handle not EXT-proved distinct) and **no local repair exists**. |
| ⟨42⟩ | REJECT at the **first** join, before `d` is reached; the case's own *accepted* `if c { … }; if !c { … }` variant also rejects. VS is the only candidate that loses the case's accepted variant. The `Option` repair then **accepts** the `c ∧ d` row the case wants rejected, moving the discrimination from the checker to a writer-declared discriminant. |
| ⟨43⟩ | REJECT at the join (II-2), so the case's policy fork has no instance. Repair: `Option` slot, after which I-3's unconditional scope-exit release implements the case's `if !original_cond { release(a) }` on writer-declared data with no drop flag. |
| ⟨44⟩ | † cost F4 / cross X5: a deliberate refusal whose **named repair is itself underivable** — `if !cond { release(a) }` is an expression discriminant, which II1 says captures nothing. M7(a) then fails undeclared. ‡ G5: II6(b)'s prose ("no release may be selected by an atom") and its formal criterion ("the operation set is a function of the source's own control flow") disagree about a cleanup on the source's own else edge, and B13's refusal rests on the prose. |
| ⟨45⟩ | † cost F4(b): the repair "write the guarded release" is checkable for the straight-line shape; **for the loop shape the repair is L05, which this candidate cannot derive** (note ⟨57⟩). |
| ⟨46⟩ | Zero annotations at the loop head. ‡ R3: no rule states whether a fact whose *term* (not support path) mentions a body-bound name survives the head intersection, so the case's value annotation (10) is not a checker fact after the loop — the back-edge fact `s == Some(v)` mentions `v` and II-7's intersection keeps only `is_some(s)`. |
| ⟨47⟩ | ‡ G2: no rule introduces a trip-count fact (`repeat n` contributes nothing, `LOOPCHK` consults no count), so the case's "`n` known at most one" variant has **no vocabulary**. |
| ⟨48⟩ | † cost overturn 10: the case's `n ≤ 1` sub-case is **inexpressible** — no rule binds `repeat n`'s iteration counter as a head φ-value — so the candidate rejects where the case says there is no repeated-take error. An unlisted capability loss inside an item §7 records as "same". |
| ⟨49⟩ | † soundness S2: `III-3` does not resolve `g.slots[σ(p)].value` in a span where `p` is assigned, so **the relative invariant the case is about is unwritable**, not merely unneeded, and `read(p) // ACCEPT: 10` is underivable in either rendering. The derivation ("zero annotations") and Appendix B ("one written loop invariant") also contradict each other. Sub-claim: `release(p); release(q)` rejects on the second release (I-20's kill clause, DV-6). |
| ⟨50⟩ | † cost S5 / cross X10: `release(p); release(q)` — an **ACCEPT in `CASES.md`** — is underivable: §1.1's identity grammar has no existential-binder form and `I7`'s premise is `Own(x) = ρ`, an owner binding at a coarse name. §3.21's "yes, with a stated cost" row is false, as is the blanket "nothing below is a capability loss". † soundness F2 adds that under II7's stated identity-default head frame L03 *rejects*, and accepts only under an unstated havoc reading. BCB derives both release lines (II-7 + I-4). |
| ⟨51⟩ | † soundness S2/S3/S4: on the plain reading the body take/put is refused by `II-15` row 2 (a write through an origin set never sets `Init`), the post-loop `read(p)` is refused because `II-9` **recomputes** the exit join from the edges rather than carrying the head invariant, and the two releases have no effect rule for `own`/`obl`. `II-7` asserts the opposite by worked example with no precedence clause. |
| ⟨52⟩ | REJECT at the loop exit join: `a` is consumed on the break edge and bound on the normal and zero-iteration edges (II-9 + II-2's definiteness clause); the rejection fires at the join itself even with nothing after the loop. Repair = `Option` slot + `replace` + `match`. |
| ⟨53⟩ | ‡ G5 / cross X12: the **atom-free loop-exit merge is unstated** — II9 delegates to II3, whose merge is indexed by a guard atom, and II8 has retired the body's atoms at the back edge. The verdict rests on II9's worked examples. BCB introduced a `Live` lattice element for exactly this case; VS intersects over all exit edges. |
| ⟨54⟩ | ‡ cross F10: `II-3` row 5 says an `own` join with no atom available is **itself rejected** (R2 definiteness), while `II-9`'s example produces `own('A) = ⊤` and defers to `I-3` at scope exit; `⊤` is not a value of `own` in §1.3. §7 endorses II-9. The enclosing scope exit rejects under I-3, whose named repair is L05. |
| ⟨55⟩ | † soundness F5: the *rejection* is a deliberate refusal, but **the repair is not a repair** — the trailing `match` on the `Option` discriminant selects the release at run time on a predicate the checker did not discharge, so the flag is relocated into source rather than deleted. Measured against P8's own "no runtime drop flag" requirement this is a capability loss. `CASES.md` L05 forbids requiring every locator to carry such a flag. |
| ⟨56⟩ | **Contested inside the candidate's own critiques.** Cross §3: "stands, and WF is the only candidate that derives L05" (II8's written head row + II5's literal-assignment relation), cost = one written head row. Soundness S5/S6: **rejects** under the literal reading — II5 records `active@1 = const true`, so II6(a) makes the `¬active` leaf infeasible and consecution against the stop path fails; and II8 does not state whether consecution is checked per back edge or on the joined value, and join-then-retire destroys the correlation the head row exists to carry. |
| ⟨57⟩ | † soundness F6 / cross F3: the derivation books L05 "underivable — rule missing (atom re-keying)"; it is stronger — `TERM`/`II-2` **explicitly refuse** the shape, since the exit relation `a_exit = ite(a_h, ¬stop, false)` needs two atoms, so no re-keying rule can exist. §7's "same" row, the "0 capability losses" tally, the O11(b)-with-a-bound claim, and `II-8`'s own worked example marking this program accept are all contradicted. L05 is also the named repair for the `I-3` refusal class (B13, L04). |
| ⟨58⟩ | ‡ R1: no rule states which names bound inside a loop body run their affine release on a **break edge** (I-3 releases "every name still bound at the closing brace"; II-9 states only the join over exit edges). Costless for L06's Copy payload, load-bearing for an affine one — which is the case's own disclaimer. |
| ⟨59⟩ | Depends on §1.2's `Live` lattice element, introduced for this case. `I-5` refuses the same release for an obligation-carrying content, which is the case's own caveat. |

---

## 2. Rule-set completeness

### 2.1 Rules present

| | rule set I (create/destroy) | rule set II (split/rejoin) | rule set III (bridge) | auxiliary | families claimed |
|---|---|---|---|---|---|
| **VS** | I-1 … I-23 (23) | II-1 … II-23 (23) | III-1 … III-13 (13) | plane map (§1.2); `Π` premises | **12** (corrects the selection's 6) |
| **WF** | I1 … I23 (23) | II1 … II20 (20) | III1 … III13 (13) | `Auto` automata; carve forest; `Gmax` | **17** |
| **BCB** | I-1 … I-24 (24) | II-1 … II-16 (16) | III-1 … III-12 (12) | `ATOM`, `TERM`, `KEY` (3 bounding rules) | **9** (withdraws the base's "four and no fifth") |

Totals: VS **59** numbered rules, WF **56**, BCB **52 + 3**.

Where each candidate places the one rule the round asked for at each of
`VERDICT-CORE` §6's three convergence places:

| Convergence place | VS | WF | BCB |
|---|---|---|---|
| R2 definiteness at a join | **II-2, explicit reject** (marked "[stated here]") | **absent** — II3 merges `St`/`Facts` only | **II-3 row 5** (guarded if the atom is available, else reject) — but no `obl` row |
| storage created at ended bytes | I-16 fresh extent identity, no ghost (arena); III-7 ghost γ (pools only) | I23 fresh coarse name, **no ghost at all** | I-23 + III-6 ghost index on **every** reusable extent (arena *and* pool) |
| identity ↔ runtime value | III-1…III-13 + `OLDCHAIN` (one rewrite per killed measure fact per call) | III1…III13 + `COVER` residue + a row's wholesale `ensures` | III-1…III-12 + `RSt` residual exception list (`III-12`, `JF-QI`) |

### 2.2 Rules missing

Collected from the derivations' own "rule missing" lines and the critiques'
missing-rule findings, grouped by convergence place. `[D]` = the candidate's own
derivation recorded it; `[C]` = a critique found it.

**VS — 16 missing rules**

| Place | Missing rule | Bites |
|---|---|---|
| create/destroy | no rule assigns a **value class** (Copy/affine/Nominal) `[C S8a]` | everywhere |
| create/destroy | **R3's linear class has no rules at all**; I-3's unconditional release makes every non-Copy value affine `[C S8b]` | R3 |
| create/destroy | what "x's affine release" *is*, and who writes it — a pool binding's release at scope exit would walk runtime occupancy `[C S8c]` | ⚠ M2(ii) |
| create/destroy | minting **freshness ⇒ slot distinctness** is used by I-19's example and stated nowhere — and that ground is the alternative O8(a) refuses `[C S3]` | B04, B05, B06, L03, P3 |
| create/destroy | `I-20`'s footprint cannot cover a linked-list `remove`'s **neighbour writes**; III-3 cannot resolve their load-derived index paths `[D][C S10]` | P3 |
| create/destroy | no rule **re-derives `LIVE` after a removal**: `INV` has no witness rule, so `use Links(other)` must reject `[C S6]` | P3, DV-6's only repair |
| create/destroy | `Links` admits **no list terminator** `[D]` | P3 |
| create/destroy | which names bound in a loop body run their affine release on a **break edge** `[D R1]` | L06, L04 |
| split/rejoin | **MR-1 use-direction path equality** — a fact at `g.slots[p.idx]` cannot discharge a goal at `g.slots[q.idx]` given `p == q` `[D][C S1]` | S01, S02, S04, B03, B04 |
| split/rejoin | **MR-4** — which `Γ`/`G` facts a name bound *at* a join from an arm-yielded value carries `[D][C S7]` | B01, and II-2's own repair for B10/B12/B13/L04 |
| split/rejoin | whether `Γ` is **closed under REF before intersection** at II-3 `[C S4]` | decides B05 and B06 either way |
| split/rejoin | no **normal form** for facts, so II-3's syntactic equality is spelling-dependent `[C S8]` | every join |
| split/rejoin | II-3's rule (syntactic intersection, "no fact is weakened") **contradicts its own example** (accepts `cap≥64` out of an arm proving `cap≥128`) `[C S7]` | a 13th family or a wrong example |
| split/rejoin | whether a live pool **row** carries an R2 obligation `[D]` | B10, B12, B13 under the pool rendering |
| bridge | **MR-2/MR-3** — pool `insert` states no contents `ensures`, `replace` states none at all, `old()` binds measure symbols only `[D][C S2]` | every printed value in S01–S06, B01–B04 |
| bridge | **GHOST propagation on a handle copy or assignment** (`let q = p`, `p = q`, the L03 swap) `[D R2][C S5]` | P7.b, B04, B05, B06, L03 |
| bridge | whether a fact whose *term* mentions a body-bound name survives the head intersection `[D R3]` | L01, L03 |
| bridge | support-set closure: II-14's example gives `c.i < len(v)` support `{v.meta}`, contradicting §1.3 `[C F4]` | ⚠ R11 |

**WF — 14 missing rules**

| Place | Missing rule | Bites |
|---|---|---|
| create/destroy | **no numbered rule for `read`** — the operation R1(a) exists for `[C S11]` | every R1 verdict |
| create/destroy | `I4`'s premise ("affine or linear") excludes the copy-scalar `take` its own examples perform `[D G1][C X11]` | 14 items |
| create/destroy | release **through a locator** — is `x` the syntactic argument or the binding owning the resolved `ρ`? `[D G3]` | S07, B05, B06, B11 |
| create/destroy | I9 sets the whole moved name `Init` instead of re-keying the source's per-place state tree `[C F4]` | ⚠ R1(a)(i) on a partial struct move |
| create/destroy | absorption (I12) leaves a live first-class pointer at a name with no state `[C F5]` | ⚠ R1(a)(ii) |
| create/destroy | no numbered rule for a **nominal's store parameters** (a §1.3 table row only), so R7(a) has nothing to apply `[C X8]` | P4 |
| create/destroy | no **adjacency-and-cover** output from `EXT`, so I12's merge premise is undecidable by any named family `[C X7]` | P2, `split_at_mut` + `merge`, the O7 evidence |
| split/rejoin | **`Own` has no join rule**; §1.2 has no binding→identity map `[D G2][C X3]` | B10, B11, B12, B13, P8(a), L05, I8's scope exit |
| split/rejoin | II7 states **no frame at the loop head** — the identity-default reading leaves stale facts live in the body `[C F2]` | ⚠ L03, L04, L06 |
| split/rejoin | the **atom-free loop-exit merge** `[D G5][C X12]` | L04, L06 |
| split/rejoin | II1 captures nothing from `if !c`, yet II5/II6/I8/§3.21 all check the arm under `¬atom(c)` `[D G4][C X5/F4]` | B09 variant, B12 variant, B13's repair |
| split/rejoin | II7(d)'s **positional binder matching** is stated only for binders attached to a binding `[C S4]` | L03, P3's head row |
| split/rejoin | II5 (keep a version while any value mentions it) vs II8 (retire every body atom at the back edge) — no rule selects `[C S3]` | L04, L05, I17-in-a-loop |
| split/rejoin | **the default (total) carve has no defining rule**, although M10(ii), R15 and OD6 all reason about it `[C F3]` | every call, every in-window access |
| split/rejoin | an **origin-set access rule** — `ι ::= {ι,…}` is in the grammar and priced in `DIS`, and no rule states the effect of a write, take, put or end through it `[C X1]` | ⚠ P9 (a required program, derived nowhere) |
| bridge | `DIS` has **no base case for two distinct `⊑`-roots**, and R4(b) forbids the only available ground `[D G2/G4/G8][C X2]` | S01, B05, B06, B10, B11, P1, P3 |
| bridge | no rule relates a **source guard atom to its defining expression** `[D G6][C X4]` | P1, every bounds/index/capacity fact a branch would discharge |
| bridge | an index term that is a **field load** (`c.i`) `[C S2]` | P4 |
| bridge | `I3`/`I6` KILL contents facts and establish none, so the cases' value claims are underivable `[D G7]` | B07, B08, S02, S04 |

**BCB — 13 missing rules**

| Place | Missing rule | Bites |
|---|---|---|
| create/destroy | **no numbered rule for `read`** `[D G1][C S14]` | every R1 verdict |
| create/destroy | no rule **forms or copies a pointer** (`p = ref(a)`, `q = p`) `[D G3]` | S01–S07, B03, B04, B06 |
| create/destroy | nothing installs **`val`**, although `JF-KILL` kills it and R6's whole frame answer is stated over it `[D G2][C S17]` | P7's contents claim, every printed value |
| create/destroy | no **index-domain premise** on `I-6`/`I-7`/`I-8`, and the default state of an unestablished path is unspecified `[C F4]` | ⚠ R11, undeclared (R11/R12 absent from D10) |
| create/destroy | `I-5` quantifies over every path under `'a` and **no family discharges a universal** `[C F7]` | ⚠ R2 for any container |
| create/destroy | `I-13`'s premise list cites a nonexistent `I-25` and omits `I-21`/`I-24` `[C S7]` | ⚠ R2 for arena blocks and pool slots |
| create/destroy | distinctness of two **`I-2` frame mints** `[D][C F14]` | P1's parallel writes |
| create/destroy | `'A # 'B` has **no writable source** — `I-1`'s allocator `ensures` cannot quantify over the caller's live set `[C F8/S3]` | 13 items |
| create/destroy | no rule rebuilds a **stored nominal at a new backing**, and no rule says whether `where backing('v)='b` is a construction premise or a live obligation `[D][C F6]` | P4's reallocating arm |
| create/destroy | how an identity **minted on the back edge** (carve, insert) joins at a loop head `[C S12]` | the arena and pool loop shapes, derived nowhere |
| split/rejoin | **`II-3` is not total**: rows for `st`, `own`, `origin` only — no row for `obl`, `val`, `layout`, bridge facts, `RSt` residuals; no **constant-vs-term** row `[D][C F2/F9]` | B10–B13, P1's join, L05's exit |
| split/rejoin | **atom re-keying** along an entailed equivalence (`a1 = ¬stop`) `[D][C F3]` | L05 |
| split/rejoin | `II-8`'s back-edge check for **two terms over different atoms** `[D]` | L05 |
| split/rejoin | how a statement **updates a relational fact keyed on `origin(x)`** `[D]` | L03 |
| split/rejoin | the **loop head is an unstated fixpoint** — no initialization, re-check order, termination measure, family or degree `[C F2]` | every loop |
| split/rejoin | `II-15` row 1's leafwise write produces a **depth-2 term** and no rule covers it (`TERM` bounds joins only) `[C F4]` | reachable in three statements; M1 |
| split/rejoin | no rule binds a **loop-counter φ-value** `[D]` | L02's `n ≤ 1` |
| bridge | `use R(args)` **premise checking** has no family, no procedure, no degree — and one premise form ("`h` came from `insert` and was not passed to `remove`") is a whole-declaration history query `[C F3/F9]` | ⚠ M2(ii), M3, P3, P13 |
| bridge | no family substitutes a callee's frozen **scalars** for the caller's at `II-10` `[C S8]` | P4, P1 |
| bridge | bridge facts may not appear in an atom while `JF-ENT` takes only atoms; measure-head **functionality** is stated nowhere `[C S9]` | P4, P1 |
| bridge | no **atom normal form** `[C S11]` | every join, M1 |

### 2.3 Declared violations, and what the critiques do to each declaration

| Row | VS declares | corrected | WF declares | corrected | BCB declares | corrected |
|---|---|---|---|---|---|---|
| **M1** | satisfied, with a correction (12 families) | **violated** — II-14's `old()` binding and step-4 substitution are not functions of the source | satisfied | **violated** — 2 invoked judgments unnamed, 1 has no rule, 1 quantification is 2^\|Γg\| | met | **violated** — unstated loop-head fixpoint over a partial join; `use`-premise checking unspecified; no atom normal form; I-17 vs III-3 give one line two verdicts |
| **M2(i)** | satisfied | stands | satisfied | stands | met | stands |
| **M2(ii)** | satisfied under O3(a), violated under O3(b) | **satisfied in letter only** — the mandated `Option`-slot repair relocates a drop flag into source (F5) | satisfied, "and it is II6(b) that makes it so" | stands structurally; **no O3 dependence at all** (its strongest claim) | met, *conditional on O3* | **a second, undeclared exposure**: `III-5`'s `slot_live` and `III-10`'s `live(hid)` are the pool's runtime occupancy word (F9) |
| **M3** | satisfied, one enumerated entry (`split`) | stands | — (not declared) | `use k != i && k != j` is a `use` over a runtime-scalar disequality with no invariant behind it (X16) | met; the `use i != j` route **deleted** | the deletion is a *reading* of O15(b); WF writes the same step and calls it a named INV application (F7) |
| **M4** | satisfied | stands | **violated** (3 identity kinds, 2 pointer type forms) | stands | — | — |
| **M7** | *not declared* | **violated** — 5 unrepairable rejection classes, one emitting no payload at all | *not declared* | **violated** — 4 classes; 3 printed repairs re-reject under II1 | met | **violated** — 4 classes, incl. a repair (L05) the rules cannot derive |
| **M8** | undecidable pending O6; **17** taught items | **≥ 24** | **violated**; **≥34 cards** | **≈69 items + ≈73 addressable IDs**, and still *low* (3 omissions) | **RED**; **≈46 constructs** | **≥ 57** |
| **M10** | violated in edit stability; at the ceiling elsewhere | **also violated in the degree clause** — 6 families with no degree, 4 degrees omitting the closure they call, measured exponent 3.4, `F`/`C` not in program size | violated on two grounds (`AFF` exponent; edit reach) | **plus** M1(b) for `AFF`'s missing bound and M10(a)'s per-declaration clause | first clause met; second clause **violated** with the degree; edit stability met | **first clause violated** (`n = Θ(S)`, so `W = Θ(S⁴·I·d + S⁵)`); **edit stability violated** for nominal-row edits (I-19's closed row reaches transitively) |
| **R rows** | *no R row at all* | R6 has 3 exceptions and no declaration (S6) | R1(a)(iii) not owned; R5(ii)(iii), R13, R14(i)(ii)(v)(vi), R8b not supplied | R8a not carried either | D10: R5(ii)(iii), R8b, R13, R14 unsupplied | **R11 and R12 have no rule and no declaration** (F4) |

### 2.4 Open defects declared

| | count | the ones that bear on this round |
|---|---|---|
| **VS** | 12 (DV-1 … DV-12) | DV-1/DV-2 stored projection + signature cascade (O7/O12); DV-4 the hole read is total; DV-6 `remove` kills every handle's liveness; DV-9 no split-and-rejoin |
| **WF** | 12 (OD1 … OD12) | OD1 wrong occupant in a reusing pool (R1(e)); OD2 `Gmax` is an unjustified cliff; OD6 declared-type carve loses facts under abstraction; OD7 element-level projection across a cut unwritable; OD12 `PART` is O(E²) |
| **BCB** | 10 (D1 … D10) | D3 `JF-LIN`'s 9 schemas have no adequacy argument; D4 the origin set stays read-only; D5 M2(ii) rests on O3; D6 P13's compaction; D9 M8 red |

Cross-examination adds, per candidate, defects the file does not carry: VS — R8a
(F1) and P3/R6(e) (F2); WF — eight of the ten missing rules in §2.2 are absent
from §6, and the `Span(ρ)=∅` base suspension is priced at M4/M8 only (X13); BCB —
D4 is a capability boundary filed as an open defect (F13), and L05 is a
capability loss recorded as "same" (F3).

---

## 3. Cost

### 3.1 Judgment families and degrees

| | families claimed | invoked but unnamed | degrees the critiques correct | total work per declaration |
|---|---|---|---|---|
| **VS** | 12 | **≥ 6**: `PREMISE`, `FOREIGNKILL` (I-22, at *every* statement, `O(F·V)`), `NAMEKILL` (III-3, `O(F·d)` per assignment), `REKEY` (II-18), the `Γ`/`G` join (II-3/II-4), `DECL` (II-10) | `FRAME` `O(F·k·d)` → `O(F·k·d·C³)`; `CONFLICT` "`O(1)` after PATH/EXT" hides the same factor; `GHOST` `O(H)` → `O(H·C³)`; `INV` `O(S·|Inv|)` omits `REF` | **not stated**; `F` and `C` are never given as functions of program size |
| **WF** | 17 | **3**: the feasible-assignment enumeration II6(a) demands (**2^\|Γg\|**), II15's within-one-entry test (`O(E·AFF)` per access), the default-carve derivation (**no rule at all**) | `STATE` omits per-statement `KILL`/`COVER`/`MEAS`/`INST`; `COVER` omits the key meet (`O(K)`); `PART` omits `AFF`; `INST` omits the within-one-entry test; `AFF` has **no degree**, only a measured 2.5–3.4 exponent | **not computed**; M10(a)'s per-declaration clause never evaluated |
| **BCB** | 9 | **≥ 4**: the loop-head fixpoint, `use R(args)` premise checking, atom installation into `Γ`, the II-1/II-2 per-arm scans (`O(S·I·n³)` each) | `JF-KILL` `O(F_base·d)` → `O(F_base·I·d·n³)` (it calls `JF-OVL` per fact); `JF-ORG` omits per-leaf `KEY`; `JF-ENT` is charged once per context in one row and per query in another | `W = O(S·n³ + S·I·d·n³ + S²·n³ + S·F·d)`, **stated**; corrected to `Θ(S⁴·I·d + S⁵)` because `ATOM` makes `n = Θ(S)` |

Measurement status of the three M10 declarations: VS **measured** (29.9 / 164.5 /
1691.0 ms at N=16/32/64, exponent ≈2.6–3.4; 82.4 s edit-reach on a 375-line
generic fixture, 678 of 995 closures repeating); WF **measured** (30.0 / 165.4 /
1690.7 ms, exponent 2.5–3.4); BCB **asymptotic only, unmeasured** (cross F15).

### 3.2 Constructs introduced (M8)

| | declared | recounted by the cost critique | what the recount adds |
|---|---|---|---|
| **VS** | 17 | **≥ 24** | the ghost vocabulary the writer writes in signatures and invariants (`σ(h)`, `Live`, `gen`, `extent`, `disjoint`, `injective`), `Foreign<T>` + access class, `par`/`par for`, `split`'s `extent ++`, the `len m` measure binder, `NoReach` |
| **WF** | ≥34 cards | **≈69 writer-visible items + ≈73 addressable IDs** | ≈14 measure names, 4 automata, 11 row-clause kinds (not 7), 56 rule IDs + 17 family names under M8(a)'s one-lookup clause; and *still low* by 3 (store parameters, `merge`/`absorb`, origin sets) |
| **BCB** | ≈46 | **≥ 57** | `par`/`par forall`, `move`, `own`'s `held`/`discharged`, `obl`'s `none`/`one`, the `size` measure head, the origin former `'x\|'y`, `RSt`'s exception list, `subrange`/`open`/`close`, `==>` |

All three are counted against an **unset `K`** (owner decision O6), in three
different units, so the three figures are not comparable to each other (BCB
cross F16).

### 3.3 Writer annotations per item

From the three cost critiques' per-item tabulations. `A` = written annotation,
declaration, identity parameter, `where` clause or `FREEZE` `let`; `I` = written
invariant; `U` = written `use`/`INV` step; `B` = a branch written only to carry a
proof fact; `R` = a restructuring (a change to the data model or control flow).

| Item | VS | WF | BCB |
|---|---|---|---|
| S01, S02 | 0 | 0 | 0 |
| S03 | **2A** (`Option` + arm) | 0 | 0 |
| S04 | **1A** | 0 | 0 |
| S05 | 0 (+1 `sink` if affine) | 0 | 0 |
| S06 | **2A** | 0 | 0 |
| S07 | 0 | 0 | 0 |
| B01 | **1R** (bind before the branch) | 0 | 0 |
| B02 | **2A** (sharpened variant unrepairable) | 0 | 0 |
| B03, B04 | **2A** each | 0 | 0 |
| B05, B06 | — **no repair exists** | 0 | 0 |
| B07, B09 | **1A** each | 0 | 0 |
| B08 | 0 | 0 | 0 |
| B09 variant | — | **1R**, and it **re-rejects** under II1 | 0 |
| B10 | **1R** (yield the survivor; the repair's own rule is missing) | 0 | 0 |
| B11 | **1R** (binding rendering only; pool rendering unrepairable) | 0 | 0 |
| B12 | **2A + 1R** | **1R**, **re-rejects** | 0 |
| B13 | **2A + 1R** | **1R**, **re-rejects** | **1B** (the guarded release) |
| L01, L06 | 0 | 0 | 0 |
| L02 | **1A** | — (rejection; the `n ≤ 1` variant has no vocabulary) | — (`n ≤ 1` inexpressible) |
| L03 | 0 or 1I (the two files disagree) | **1A** (existential head row: 2 binders, 3 state facts, 1 `dis`) + 1 unstated `INV` | **1I** (3 conjuncts) |
| L04 | **1A + 1R** | 0 | — |
| L05 | **1A + 1R** (the source flag disappears into a tag word) | **1A** (head row) | **1I** (2 implications) — underivable |
| P1 | **1A** (`requires i != j`) | **4A + 1U** (`focus` head + 3 `where` + 2 captured index bindings) | **1A + 2B + 1R** (outcome arm) + **1 re-`FREEZE` per join** |
| P3 | **1A + 2I + O(H) `U` per removal** (and the `use` does not discharge) | **≥5A + ≥1U per traversal read site** | **1I + 1I per traversal loop head + 1U per symbolic-write group per iteration + 1B per traversal read** |
| P4 | **2 params × call-chain depth + 1I**, **non-local cascade** | **2A + 1U** (understated by 1 param + 1 `where`) | **5A + 1B**; the reallocating arm **unrepairable** |
| P7 | **2A** | **2R** (`drop(old)`; empty the slot before `free`) | **2A** (affine content), 0 (copy content) |
| P8 | **1R** | 0 | 0 |
| **Totals** | **19A, 3I, O(H) U per removal, 8R, 1 non-local cascade, 3 items with no repair** | **13A, ≥3U (P3's grows per read site), 5R of which 3 do not derive** | **8A, 3I + 1 per traversal loop head, O(1) U per symbolic-write group per iteration, 4B, 1 outcome arm, 1 re-`FREEZE` per join, 1 item inexpressible, 1 underivable, 1 unrepairable class** |

Each candidate's own §1.4 cost model against the measurement:

| | model says | measurement says |
|---|---|---|
| **VS** | statement 0 in the common case; loop head 0 or 1; **join 0** | the join row is contradicted by 8 restructurings; the statement row by P3's `O(H)` `use` steps per removal |
| **WF** | join nothing; loop head per such loop; `focus` per split | 26 of 31 items cost nothing and **four items (L03, P1, P3, P4) carry all of it** — a concentration §3.21 does not state |
| **BCB** | **join: nothing, always**; statement zero for "P3's writes"; loop head zero for P5, P6 | the join row is contradicted by P1 (one re-`FREEZE` per join); the statement row by P3 (one `use` per symbolic-write group per iteration); the loop-head row cites two programs derived nowhere |

### 3.4 Restructurings required

| Candidate | count | standard form |
|---|---|---|
| **VS** | **8** (B01, B10, B11, B12, B13, L04, L05, P8) plus 5 more items whose `Option` repair changes the data model (S03, S06, B03, B04, B07/B09/L02) | `var slot: Option<T>` + `replace(inout slot, None)` + a trailing `match` — one discriminant word per resource and one branch per release site |
| **WF** | **5** (B09 variant, B12 variant, B13, P7×2), of which **3 re-reject** under II1 as written | the complementary `if !c { … }` arm |
| **BCB** | **1** (B13's guarded release — a line the case itself offers) | the same complementary arm |

---

## 4. Capability differences, item by item

### 4.1 The three-way ledger

| Item / line | VS | WF | BCB |
|---|---|---|---|
| `let c = Cursor{at: &v, i: 0}` — a stored pointer into a container (P4, B11, P3's links) | **CL** — §1.1/II-19: a projection is never stored; the repair is a signature cascade (DV-1/DV-2) | **accept** — `ptr<ρ↓w.k,T>` + `ESC`, second class by a syntactic judgment; **no numbered declaration rule** (X8) | **accept** — II-13 (a projection is an ordinary sub-identity name, nothing is consumed) + I-19 (the nominal's identity row is closed) |
| two long-lived writable names for one slot (P7) | **unwritable in the projection rendering** (II-17's conflict table); writable in the pool rendering | **accept** (II15) | **accept** (`ptr<'a,T>` copyable, no mode, no duration; KEY keys state on `'A`) |
| `read(q)` after `take(p)` — the hole (S03, S06, B03, B04, B07, B09, L02, P7.1) | **accept, returns `None`** — the ladder's Step-0 rung has **no instance** (DV-4) | **reject** — I4 + `St` on the place | **reject** — I-7 + `KEY`'s ⊓ over the may-equal class |
| `release(p)` then `read(q)` with `p ≠ q` correlated (B05, B06) | **CL** — II-3's syntactic join drops `σ(p) != σ(q)` | accept (II2's guarded identity) — **but `Own` has no join rule** | accept (II-5 leafwise correlation + II-14 origin split) |
| an obligation that differs on two arms (B10, B12, B13, L04) | **REJECT by rule** (II-2 definiteness, marked "[stated here]") | **no join rule for `Own`** — §3.21 asserts "retained, guarded" | **II-3 rows 4–5**: guarded when the atom is available, **reject** otherwise |
| `if active { if stop { release(p); active = false } }` (L05) | **REJECT** (O11(a): the correlation is a condition term over state) | **accept**, one written head row — *the only candidate that derives it* (cross), *rejects under the literal reading of II5+II6(a)* (soundness) | **REFUSED by `TERM`/II-2** — the exit relation needs two atoms, so no repair can exist |
| `if !c { release(a) }` — the complementary arm (B09v, B12v, B13) | n/a (the mixed case is not representable) | **underivable** — II1: an expression discriminant captures nothing | **accept** — `¬b` is a first-class atom form; `JF-ENT`(4) unit-propagates `c1 = ¬c0` |
| `if i != j { … }` — a source branch discharging an index fact (P1, P17) | **accept** — the enclosing branch condition is a premise in `Π`, read by EXT/REF | **underivable** (X4) — the only route is a written `use` step | **accept** — `ATOM` freezes every operand, II-1 pushes α onto `Γ`, `JF-ENT` reads it |
| `use k != i && k != j` (P1) | accept (INV over a written invariant) | accept — "a named INV application, not a bare assume" | **deleted by M3** — "a disequality on two runtime scalars has no such rule"; the only route is a written branch |
| `write(r, 5)` through `pick`'s two-storage result (P9) | **accept**, footprint = the origin union (II-20), paying a span-wide kill | **no rule** — origin sets are in the grammar and priced in `DIS`, and no rule states the effect of a write through one | **refuse** with a named repair (II-15 row 2); D4 records the repair is unwritable when the callee hides the discriminant |
| read of a **refilled** pool slot (P3, R1(e)) | **reject** statically (III-7 ghost γ, no runtime compare) | **accept, reads the new occupant, no diagnostic** (III11, OD1) | **reject** (III-6 ghost index + KEY) — conditional on O3 |
| `release(p); release(q)` through a loop-head existential (L03) | reject the second (DV-6) | **underivable** — no binder form in `ι`, `I7`'s premise is `Own(x)=ρ` | **accept** (II-7 + I-4) — on the plain reading of II-15 row 2, contested |
| arena block reuse at coinciding bytes (P2) | fresh extent identity, **no ghost** | fresh coarse name, **no ghost** | **needs the ghost index** (KEY's may-equal class is keyed on the extent) → the broadest O3 exposure |
| a coalescing arena / re-joinable `split_at_mut` (P2) | **declared absent by rule** (II-23, DV-9) | claims I12 `merge` — **no family can decide "adjacent and cover"** (X7) | not supplied (D3) |
| per-column distinctness under a generic column type (P5) | accept, **0 written** — the plane map is a declaration, `PLANE` is an `O(1)` table | needs a **written `carve` clause** (OD6's "forced trade" against R15 check-once) | accept, **0 written** — `JF-LIN` schema 7 (sibling fields) |
| P6's partition rejoin | accept, 0 written (`ProvedRangePartition`) | **1 written prefix-run head row** | accept, 0 written (`JF-LIN` schemas 4 and 5) |
| `compact()` then `read(pool[h])` (P13) | "the handle IS the fixup form" — **no map plane declared**, and I-21's own example rejects `compact_renumber` (BCB O2) | `ensures forall h. Occupied(ρP, slot_of(h))` — `slot_of` has **no declared carrier** | **a written `map : array<HandleId,Slot>`** that `compact` rewrites, charged at one word per outstanding handle |
| storing a projection **without suspending the base** | n/a (nothing to suspend) | **refused** — `Carve(ρ)=∅` / `Span(ρ)=∅` are premises of `end`, relocation and reallocation (a scope-shaped restriction) | **accept** — D4: the base's writability is never suspended |

### 4.2 The two-sided delta, in the ladder's own style

```text
// P4 — the stored cursor
let c = Cursor{ at: &v, i: 0 }
  // VS  reject: a projection can never be the value of a binding (II-19, DV-1)
  // WF  accept: ptr<ρ↓w.k,T>, second class by ESC (II19)
  // BCB accept: Cursor<'v,'b> where backing('v)='b, closed row (I-19)
push(v, 5)
b = c.read()
  // VS  accept (index rendering): fact used — c.i < len(v) re-derived by OLDCHAIN
  // WF  reject as spelled: missing fact — cap_of(v) > len_of(v) (I17 arm B); the named `reserve` repair does not close it (II12 is itself two-armed)
  // BCB accept (soundness F2: st('b[0])=Init survives the reallocating leaf — use-after-free admitted)
```

```text
// S03 — the hole, the ladder's Step 0 rung
v = take(p); r = read(q)        // q and p name one slot
  // VS  accept: returns None — I-7 replace is total, III-11 vacancy is the writer's discriminant
  // WF  reject: missing fact — St(place) = Init (I4)
  // BCB reject: missing fact — st('A) = Init after KEY's ⊓ over mayEq (I-7)
```

```text
// B12 — two conditional releases
if c { release(a) }
if d { release(a) }
  // VS  reject at the FIRST join: missing fact — BIND(a) definite (II-2); the case's own accepted `if c/if !c` variant also rejects
  // WF  reject at the second release: c ∧ d is feasible (II6(a)); `if c/if !c` accepts — but by a worked example its own II1 refuses
  // BCB reject at the second release (I-4's constancy premise); `if c/if !c` accepts by JF-ENT(4)
```

```text
// L05 — a checked Boolean guarding later iterations
loop { if active { if stop { release(p); active = false } } }
  // VS  reject at the inner join: missing fact — the correlation active ⇒ BIND(p) is a condition term over state (O11(a))
  // WF  accept: fact used — head row { ρa : ite(active, Init, Gone) } (II8)   [soundness: rejects under the literal II5+II6(a)]
  // BCB reject at the arm exit: missing fact — the exit term carries `stop`, not the enclosing Γ's atom (TERM/II-2); no repair exists
```

### 4.3 Counted

| | items the other two accept that this one refuses or cannot derive | items this one accepts that the other two refuse |
|---|---|---|
| **VS** | **8** (B05, B06, B10, B11, B12 incl. the case's accepted variant, L04, L05, P4's stored-cursor conjunct) | **7** (S03, S06, B03's `read(a)`, B04's intermediate reads, B07, B09, L02's second take — every one the ladder's Step-0 hole read), **plus 1 safety win**: the refilled-slot read, refused statically where WF admits it |
| **WF** | **14**, of which **10 are lines its rules do not reach at all** and 8 of those 10 are absent from §6 | **8**, of which **5 stand**, **1 is unique** (L05), **2 are over-claimed** (P8(a)/B10–B12 rest on a join rule that does not exist; the `merge` rests on a family that cannot decide its premise) |
| **BCB** | **8** (L05; P4's `reserve` route and its reallocating arm; P1/P17's `use` route; P2's ghost-free arena reuse; a two-atom join; P9's write through an origin set; two frame-local mints' distinctness) | **7** (P4's stored pointer, P7's two aliases, the hole refusals, B05/B06, B10–B13/L04, P3 as a whole, P13's declared handle→slot map) |

Fifteen of the twenty-six cases diverge from both rivals under VS, in both
directions.

---

## 5. What each candidate asks, what it spares, and the three questions

### 5.1 `value-semantics`

**Asks.** A permanent capability boundary at O7(a)/O12(a) — a projection is never
stored, returned, captured or bound — whose price is DV-1 (no cursor, iterator,
callback or closure over a non-pooled container) and DV-2 (the repair is a
non-local signature cascade, so M7(a)'s local-fix payload cannot be honoured);
one name kind in a type after all, the pool brand `P` on `Pool<P,T>`/`Handle<P,T>`,
which `VERDICT-CORE`'s Disputed D-6 already asks whether it reopens D1; a ruling
of O3(a), since under O3(b) §5.1 says M2(ii) is violated and the model reverts to
the base's stale-slot residue; a written invariant per structure rather than per
program (one `plane` invariant per backing, one pool invariant per pool type with
3–8 `use` steps, one arena invariant, one liveness clause at the head of any loop
that frees into a pool) plus `O(H)` written `use` steps per pool removal, which
the cost critique finds do not currently discharge; O11(a), no condition term
anywhere, paid for in eight case items; a restructuring per case whose standard
form is an `Option` slot costing one discriminant word per resource and one
branch per release site, in exactly the cases where both rivals materialize
nothing; and M10's edit-stability clause declared violated with the
callee-direction reach measured at the transitive instantiation closure.
**Spares.** One distinctness relation and one only — the containment tree
`root → plane → extent → element` with `PATH`/`EXT` over it, C2's single relation
achieved, where WF carries `⊑`, `≡` and a total `DIS` over five identity forms
and BCB carries `#`, `≤` and `mayEq`; **no state at all** — `Σ` is a two-point
lattice, `Γ` is a set of `(fact, support)` pairs, `Π` is discarded at every join,
and there is no `ite` anywhere in the checker's state; nothing written at a loop
head for memory safety except one pool liveness clause; M4 satisfied, one
spelling per construct; the smallest taught inventory of the three (17 declared,
≥24 recounted, against WF's ≥34/≈69 and BCB's ≈46/≥57); no drop flag
*representable*, structurally rather than by a rule; and the safest rung on the
ladder's pool case — the refilled-slot read is refused statically with no runtime
compare, where WF's OD1 admits it with no diagnostic.

### 5.2 `window-focus`

**Asks.** A second pointer type form with a fine name in it, `ptr<ρ↓w.k,T>`,
second class by a five-clause `ESC` judgment and refused by the *parser* in every
cut position — O12(b) taken explicitly, and the only candidate asking the owner
to hold "a carve entry may appear in a local binding's type and nowhere else"; a
third identity kind (coarse name + place + window entry, plus guarded values over
all three), for which it declares M4 violated and asks the owner to ratify the
declaration; a bounded condition term with a magic constant, `Gmax` provisionally
2, whose value is set by nothing (OD2) and whose overflow collapses silently at
the join and surfaces as a rejection at a later access; a lexical exclusivity
window that **suspends operations on the base** — `Carve(ρ)=∅` and `Span(ρ)=∅`
are premises of `end`, relocation and reallocation, a scope-shaped restriction
the ladder's own closing row says no hazard needs, and which BCB shows is not
forced; a restructuring per case twice over (B13's automatic cleanup refused,
P1's read admissible by exactly one route, a written `use` step, because no
source branch can discharge an index fact); R1(e)'s P3 clause as a *permanent
boundary* rather than as a question — a stale handle into a type-stable reusing
pool reads the new occupant with no diagnostic, the one requirement test where
both alternatives pass and this one does not; and a default carve that is a
function of a **declared type**, hence OD6's abstraction loss and an edit reach
looser than the interface. **Spares.** No ghost entity and therefore **no
dependence on O3** — the cleanest structural claim in the round, since VS's
M2(ii) is violated under O3(b) and BCB's is conditional on O3 for both P2 and P3;
no quadratic written-step tax on a pool interface (VS's III-8 kills `LIVE(k)` for
every handle after any `remove`, costing it B05, B06 and part of L03); no
`use IA(b,b')` per block pair, which VS's arena distinctness needs; no closed
identity row on every pointer-carrying nominal, so no `O(f²)` `#`-clause count at
an interface producing an aggregate of separately named storages; no
whole-program instantiation closure — `LOOPCHK` and `ROWSUB` are per-declaration;
one join rule instead of two shapes; and a loop rule with **no fixpoint and no
widening at all** — `LOOPCHK` checks initiation and consecution, two entailments,
where VS runs a height-2 greatest fixpoint for `Σ` and BCB joins at a lattice.

### 5.3 `brand-context-bounded`

**Asks.** A name kind inside the identity *path* — the erased ghost index `@g` on
**every** reusable extent, arena bytes as well as pool slots, which makes M2(ii)
conditional on owner decision O3 for both P2 and P3 and is the broadest O3
exposure of the three (and, the cross-examination finds, self-inflicted on the
arena half, since both rivals reuse arena bytes with a fresh name and no ghost);
a row kind in a type — `Ρ`, an ordered identity row, `N<Ρ>` well-kinded at every
arity, with every pointer-carrying nominal's row **closed** by I-19, which costs
`Cursor<'v,'b>` plus a `where` and an `O(f²)` `#`-clause count at any interface
producing an aggregate of separately named storages, and whose edit reach the
cost critique shows is the transitive closure of declarations mentioning the
nominal rather than "callers and no further"; a bounded condition term whose
overflow is a **refusal** rather than a collapse — one atom per identity per
point, shrinking on arm entry, with `II-2` rejecting a divergence that crosses two
independent guards, a class the file declares never exercised and which the
critiques show is exercised exactly once, at L05, where it costs the candidate
the case and the general no-drop-flag loop idiom; a residual exception list on
every range invariant (`RSt(π[lo..hi), s, X)`, reset at loop heads and calls),
which is what delivers P3's fourth property and is unique to this candidate; a
reading of O15(b) that **deletes** the `use i != j` route, so P1 and P17 pay a
runtime branch and an outcome arm where WF pays a written step and VS pays a
`requires`; and an M10 declaration that is asymptotic only, unmeasured, against
two measured rivals. **Spares.** No second-class discipline — no `ESC` judgment,
no second pointer type form, no parser-position restriction, no span table — and
no suspension of the base while a projection is held; no window machinery — no
`focus`/`carve`, no carve forest, no cross-level `dis` walk, no `PART` at `O(E²)`
with `k(k−1)/2` writer-supplied inequalities, no declared-type default carve
losing per-column facts under abstraction; no plane-map declaration per backing
and no four-cell conflict table, since sibling-field distinctness comes from
`JF-LIN` schema 7 and unrestricted sequential aliasing is admitted (D4); **zero
restructurings across S01–S07, B01–B12, L01–L06** and 19 of 26 cases at zero
writer cost, the strongest measured writer-cost result in the round; the only
declared and charged handle→slot map of the three, where both rivals assert
handle survival across compaction with no carrier; and no unjustified precision
constant, since `TERM`'s bound of one atom is structural where `Gmax = 2` is set
by nothing.

### 5.4 The three questions, with the evidence rows that bear on each

#### Q1 — O7/O12: is "projections are second class, references are never stored" acceptable as a permanent capability boundary?

| Evidence row | VS (boundary taken) | WF (O12(b): one extra type form + `ESC`) | BCB (boundary refused) |
|---|---|---|---|
| **P4** first conjunct (a stored cursor over a growing, non-pooled container) | **CL**, with a non-local repair (DV-1/DV-2) | accept at 2A+1U — **understated** by 1 identity parameter + 1 `where`; no declaration-side rule (X8) | accept at 5A+1B; the reallocating arm has no expressible continuation (F6); ⚠ F2 admits use-after-free as written |
| **B11** `remaining = ref(b)` — the boundary inside a *case* | **CL** (pool rendering, no local repair) / `refuse+repair` (binding rendering, yield the survivor as a value) | accept — but not derivable as stated (no `Own` join, X3) | accept (II-13 + II-14) |
| **P7** two long-lived writable names | unwritable in the projection rendering; writable in the pool rendering (DV-3 is stale in the candidate's favour) | accept, 2 writer lines — ⚠ F3 leaves the M2(ii) question open | accept; the contents claim is not derivable (no `val` rule) |
| **P3** links through stored pointers | **underivable** (F2) | accept + `⚠` OD1 (the refilled-slot read) | accept, conditional on O3 |
| **L03** relative invariant over two rebindable names | **CL** — the invariant is unwritable under III-3 (S2) | **CL** — `release(p); release(q)` underivable (S5/X10) | **underivable** on the plain reading (S2/S3/S4) |
| The *price of refusing* the boundary, stated by the refuser | — | +1 type form, +1 judgment, +2 M8 cards; **and** a base-suspension restriction it does not price (X13) | exactly three items: I-19's closed row + `O(f²)` `#`-clauses; III-8's conditional `backing`, which forces `TERM`/`II-15`/`JF-ORG` into the model; `JF-OVL`'s `O(I·d·n³)` per symbolic access |
| The *price of taking* it, stated by the taker | DV-1 + DV-2 (a signature cascade; M7(a) cannot be honoured), and P4's first conjunct lost permanently | — | — |
| Sharpener from the critiques | F2: the pool rendering — the **only** rendering restoring a storable rebindable name — has a bridge (III-6) that contradicts its own relocation rule (I-21), so the boundary buys P4 back **only if compaction is declared out of the language** | X13: WF is the only candidate in which holding a projection blocks `end`, relocation and reallocation on the base — a borrow-scope-shaped restriction BCB's II-13/D4 shows is not forced | F13: D4 (a write through an undiscriminated origin set) is a **capability boundary filed as an open defect**, O7/O12-adjacent |

#### Q2 — O11: are condition terms admitted at joins?

| Evidence row | VS — O11(a), no terms | WF — O11(b), `Gmax = 2`, collapse on overflow | BCB — O11(b), one atom, **refusal** on overflow |
|---|---|---|---|
| **B02** sharpened variant | accepts where it should reject; **unrepairable** under (a) | accept (one atom carries both) | accept |
| **B05, B06** correlated targets | **CL, no local repair** — II-3's syntactic join drops `σ(p) != σ(q)` | accept (guarded identity) — resting on the missing `Own` join and the missing `DIS` base case | accept (II-5 leafwise correlation) |
| **B08** refinement recovery | precision loss (reclassified from `CC`) | accept, same atom version | accept — **only under the SSA reading** (S13) |
| **B09** / **B12** accepting variants | reject | **underivable** (II1 captures nothing from `if !c`) | accept (`¬b` is an atom form) |
| **B10, B11, B12, B13, L04** obligation at a join | **REJECT by rule**, repaired by an `Option` slot (5 restructurings) | asserted "retained, guarded" — **no `Own` join rule** | accept (II-3 rows 4–5) — but **`obl` has no join row** (F2) |
| **L05** the sharpest discriminator | **CL** — the repair relocates a drop flag into source (F5) | **accept**, one written head row — the only candidate deriving it (cross); **rejects** under the literal II5+II6(a) (soundness) | **CL** — `TERM`/II-2 refuse the shape and no repair can exist (F6/F3) |
| **P8** "no drop flag; the writer's branches carry the state" | conjunct 2 refused by II-2; conjunct 1 delivered only by relocating the flag | delivered by II6(b) as a *rule* — but P8(a) is not derivable as stated | delivered (II-3's terms are erased) — but the join term as written accepts a read at `Live` (S1) |
| The price each pays for its answer | 8 case items refused; 8 restructurings; one discriminant word per resource and one branch per release site — the exact thing `CASES.md` L05 forbids | an unevidenced constant (OD2), a silent collapse diagnosed at a downstream access (X14), and an exponential feasible-assignment enumeration no family names (F1) | one refusal class, whose single exercised instance (L05) the candidate cannot derive, and a cubic-in-program-size entailment closure (`n = Θ(S)`) |
| What each *buys*, by its own count | — | "12 of 26 cases decided by guarded states or guarded identities" (§3.21) | "19 of 26 cases at zero writer cost"; B02, B03, B05, B08, B09, B12 and P8's naive shape accepted |

#### Q3 — is a pool (or per-structure) invariant an acceptable writer cost?

| Evidence row | VS | WF | BCB |
|---|---|---|---|
| What is written per structure | one `plane` invariant per backing (III-2), one pool invariant per pool type (`PBounds`, `Links`), one arena invariant (`IA`), one liveness clause at the head of any loop freeing into a pool | one written `Links` invariant (4 conjuncts), a loop-head row, a parametric-entry `focus`, an occupancy `where` per parallel element, an `ensures` per pool op | the pool's ∀-invariant plus the free list as written data, **plus one per traversal loop head** (`X` resets at every loop head and call) |
| What is written per *use* | **`O(H)` `use` steps per removal** — one per surviving handle name — quadratic over a removal loop | **≥1 `INV` step per traversal read site** — grows with read sites, not declarations | **1 `use` per symbolic-write group per iteration** + 1 branch per traversal read |
| Does the written step discharge? | **No** (cost S6): `Links` quantifies over `σ ∈ Live(P)`, and instantiating it at the target needs the fact being re-derived; `INV` has no witness rule. DV-6's only named repair therefore does not exist | Yes, but `COVER` through a stored coarse pointer has **no stated diagnostic and no repair** short of redesigning links into handles (S8) | The `use` premise-checking procedure is **named by no family**, and one premise form is a whole-declaration history query (F3) |
| Cost when it fails | B05, B06 become capability losses; P3 becomes underivable; R6(e)'s only acceptance test is not delivered | P3 accepted, but `⚠` a refilled slot reads the new occupant with no diagnostic (OD1) | P3 accepted, conditional on O3; `⚠` schema 9 makes ghost inequality a disjointness proof feeding R4/R5 (F5) |
| Alternative to the invariant, per candidate | none — "ghost slot equality is not decidable, and deriving distinctness from two handle values being different is what R4(b) forbids" (DV-6) | keys state on **places**, so it pays no per-handle tax at all — bought at OD1 | the pool's **written contract** discharges slot disjointness; `RSt` residual framing *weakens* the invariant per symbolic write instead of killing it |
| Interface-level cost | a written distinctness premise on the pool interface cannot repair a fact about two caller-local variables (cross S3) | one parameter per stored pointer, no pairwise clauses | `O(f²)` `#`-clauses at any interface producing an aggregate of separately named storages (I-19) |
| Comparable: arena blocks (P2) | `use IA(b,b')` **per block pair** — quadratic | `EXT` over promotion extents, nothing written | `JF-LIN` over frozen bounds, nothing written — **but not derivable for a symbolic block size**, and P2 is derived nowhere (S8) |

---

## 6. Three standing cross-candidate facts the owner will need

1. **The three convergence places reproduced themselves.** Of WF's five fatal
   cross findings, four are rules both other candidates wrote and it did not, and
   all five land in `VERDICT-CORE` §6's second and third places. BCB's own
   closing paragraph names the same four holes (a total join rule, a positive
   premise on ending and writing, a term-overflow rule for its leafwise write, an
   accepted spelling for L05). VS's two fatal cross findings are at relocation
   (R8a) and at removal (P3). No candidate closed all three places.

2. **Every candidate's own "verdicts overturned" list is non-empty, and each
   file contradicts itself at least once on a load-bearing line**: VS's
   derivation and Appendix B disagree on L03's annotation count; WF's §3.21 says
   "nothing below is a capability loss" while its boundary derivation records
   L03's release line underivable; BCB's §7 records L05 "same" while its boundary
   derivation records it underivable and its own II-8 example marks it accept.

3. **`read` has no numbered rule in two of the three candidates** (WF S11, BCB
   S14/G1), and the third has no rule assigning a value class. The operation every
   R1(i)/R1(ii) verdict in this round depends on is assembled from examples in
   both cases.


# Completeness review of this comparison


Round: core2. Date 2026-09-16. Lens: **completeness**. Read in full:
`COMPARISON.md`, `rules-{value-semantics,window-focus,brand-context-bounded}.md`,
the nine `derive-*.md` files, the nine `critique-*.md` files, against
`PROGRAMS.md` (P1–P19), `CASES.md` (S01–S07, B01–B13, L01–L06),
`MECHANISM-MAP.md` §3 (the hazard ladder), `VERDICT-D0.md` §2/§7 and
`VERDICT-CORE.md` §6 (the nineteen owner decisions and the meta-finding).

**This file selects nothing and recommends nothing.** It reports gaps only:
items no candidate derived, verdict cells with no derivation behind them,
critique findings the comparison drops, rule sets still incomplete for P4, P7
and P8, costs asserted without a count, places where the comparison ranks
despite its own rule, and owner decisions this round produced evidence for and
did not route. Short keys: **VS** = `value-semantics`, **WF** = `window-focus`,
**BCB** = `brand-context-bounded`.

**32 gaps.** Six are corrections of fact inside `COMPARISON.md` (G07, G08, G11,
G16, G18, G21); the rest are missing work.

---

## 1. The gap table

`Where` cites the file and section that should carry the item. `What would
close it` is the smallest artifact that answers the gap, not a repair of any
candidate.

### A. Items no candidate derived

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G01** | **Thirteen of the nineteen fixed programs have no line-by-line derivation under any candidate**: P2, P5, P6, P9, P11–P19 (P10 is trigger-gated and correctly absent). The derivations cover S01–S07, B01–B13, L01–L06, P1, P3, P4, P7, P8 and nothing else. Six of the thirteen nevertheless carry verdicts in the comparison (P2, P5, P6, P9, P13, P17) | derivations; `COMPARISON.md` §4.1, §5.4 | Derive at least P2, P9, P13 and P17 line by line under all three — P2 for the ending rules, P9 for the origin-set write, P13 for relocation, P17 for R11 — or mark every row that rests on a candidate's own asserted verdict |
| **G02** | **The corpus-preservation requirement is untested and unmentioned.** `PROGRAMS.md` requires that whatever the current design lets overlap under PAR-1/PAR-2 in wfgrep, zlib-core-kernels, compute-bench and the accumulator snapshots stays permitted "or the loss must be stated". No candidate states it and the comparison has no row | `PROGRAMS.md` "Existing corpus programs to preserve"; `COMPARISON.md` §4 | One PAR-overlap row per candidate — permitted, lost, or not evaluated — or a stated deferral with the reason |
| **G03** | **Two rungs of the hazard ladder have no three-way delta.** Step 1 (`t = move s; read(px)` — a relocating move under an outstanding interior pointer — and its box counterexample `b2 = move b; read(pc)`) is derived by nobody, although WF-soundness F4 is exactly that hazard in WF's I9. Step 5's grounds (ii) (recombinable accumulator under a law) and (iii) (named weaker level) are exercised by no item; their program, P15, is underived | `MECHANISM-MAP.md` §3; `COMPARISON.md` §4.2 | Add the two Step-1 lines and one R5(a)(ii)/(iii) line to the fixed item set and derive them in the ladder's style, as §4.2 already does for P4, S03, B12 and L05 |
| **G04** | **No requirement-coverage table.** §2.3 compresses R1–R15 into a single "R rows" line. R9, R10, R12, R13 and R14 have no derived item under any candidate, and VS carries no R row at all, so a reader cannot see which rows are delivered, which are declared unsupplied, and which are simply untested | `COMPARISON.md` §2.3 | An R1–R15 × candidate table with three values — delivered (with the rule), declared unsupplied, untested — keyed to each row's own acceptance test |
| **G05** | **Four meta-requirements are compared nowhere.** §2.3 has rows for M1, M2(i), M2(ii), M3, M4, M7, M8, M10 and no row for **M5, M6, M9, M11**. M6 (soundness evidence) is the row the nine critiques are evidence for; M9 (measured performance floor) is the row no candidate can meet without an emitter; neither absence is stated | `COMPARISON.md` §2.3 | Four rows, or one line recording that the round produced no evidence for M5, M6, M9 and M11 and why |

### B. Verdict cells the comparison filled without a derivation behind them

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G06** | **Six of §4.1's eighteen ledger rows rest on assertions, not derivations**: the two P2 rows (arena reuse; coalescing arena), P5, P6, P9, P13. Only two carry a "derived nowhere" note (P9 in §2.2, P2 in §5.4 Q3). WF-cost S10 and BCB-cost m5/S8 and BCB-cross F17 all make this finding against their own candidate; the comparison reproduces the asserted rows in a table that otherwise contains derived ones | `COMPARISON.md` §4.1 | Mark each asserted row, or split §4.1 into "derived" and "asserted, not derived", which is BCB-cross F17's own fix applied to all three |
| **G07** | **L02's cells contradict their own notes.** The matrix shows WF `accept ‡ ⟨47⟩` and BCB `accept † ⟨48⟩`, while ⟨47⟩ says the case's `n ≤ 1` variant "has **no vocabulary**" and ⟨48⟩ says it is "**inexpressible** … an unlisted capability loss". Both cells are counted as accepts in §1.3 | `COMPARISON.md` §1.2 rows L02, §1.4 ⟨47⟩⟨48⟩, §1.3 | Split L02 into its two sub-cases or relabel both cells, then recompute §1.3's WF and BCB rows |
| **G08** | **§2.2's headline counts disagree with its own rows and invert their order.** The headings read "VS — 16 missing rules", "WF — 14", "BCB — 13"; the tables beneath them hold **18, 19 and 21** rows. A reader taking the headings reads the ordering backwards | `COMPARISON.md` §2.2 | Recount, or state the counting unit (per convergence place? per bitten item?) and make the headings agree with the rows |
| **G09** | **`underivable` is a fourth classification the round's rule does not admit.** The brief requires each difference from a `CASES.md` verdict to be a capability loss, a deliberate refusal with a named repair, or a correction of the case. Eight cells end as `underivable` (VS 1, WF 6, BCB 1) and are never classified. WF-soundness S8 raises exactly this against its own candidate and the comparison adopts the label instead of resolving it | `COMPARISON.md` §1.1 legend, §1.2 | Classify all eight; where the classification depends on a missing rule, say which of the three it becomes under each reading |
| **G10** | **Case variants are folded into single cells**, so §1.3 counts items of unequal content: B07's two narrower operations, B09's variant, B12's variant, B13's two policies, L02's `n ≤ 1` sub-case, L05's post-loop use, P8's two shapes, P4's four lines and P7's four lines each carry a separate verdict in the derivations and none has a row | `COMPARISON.md` §1.2 | One row per required line or variant — the shape §4.2 already uses — or a stated rule for what a cell aggregates |

### C. Critique findings the comparison ignored

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G11** | **Five *fatal* critique findings appear nowhere in the comparison** (detail in §2.1): VS-soundness **F1** (I-7 bundles plain assignment-over with `replace`, so assigning over an affine destination drops its release obligation on every path — R2(a), no diagnostic); VS-soundness **F3** (`par { reserve(inout v.meta,128) ∥ fill(inout v.data[0..4]) }` is *permitted* by II-22 ground (i) because PATH refutes overlap between sibling planes — R5(a), and distinct from the sequential R8a case the comparison does carry at ⟨10⟩); WF-soundness **F6** (`@E ⇒ noalias` with a live `⊑`-relative in the same row emits a **false** fact — R4(a), a miscompile, not a lost fact); BCB-soundness **F1** = BCB-cross **F1** (`I-4`/`I-6`'s premise `st ≠ Gone` is satisfied by `⊤`, so `free` and `write` are accepted at a state whose Gone leaf they do not exclude — R1(ii), found independently by two critiques); BCB-soundness **F3** (`X` discarded at a call or loop head lets one symbolic element be taken twice — R1(i) + R3) | `COMPARISON.md` §1.4, §2.2, §2.3 | Add the five to the cell notes or to a hazard ledger (G12); three of them (R2 leak, R5 race, R4 false fact) are the only findings in the round outside R1, and the comparison's hazard picture is R1-shaped without them |
| **G12** | **No hazard ledger.** Each soundness critique carries a "hazards admitted, by requirement" table; the comparison carries seven `⚠` cells (VS 2, WF 3, BCB 2). A hazard that is not attached to one of the 31 items has nowhere to land, which is why all five findings in G11 vanished | `COMPARISON.md` §2 | One merged table: requirement row × candidate × hazard × declared/undeclared, built from the three critiques' §2 tables |
| **G13** | **A derivation on a refused owner interim is not recorded.** WF-soundness S10: I13 derives arena exhaustion as "an ordinary library outcome" under O5(c), while the settled interim is O5(b) (`VERDICT-D0` §7 decision 5, R12's clause red; `VERDICT-CORE` O5). The brief fixes every §7 interim as settled, so this is a round-rule departure, not a preference | `COMPARISON.md` §2.3, §5.2 | Record the assumption in WF's declaration row and state which lines of P2/P18 change under (b) |
| **G14** | **Rule-versus-example conflicts are reported one at a time and never counted**, although `VERDICT-CORE` §6's meta-finding is precisely that "a tuple's stated verdicts are not a reliable guide to what its rules actually admit". At least twelve are on record: VS II-3's join rule vs its own weakening example, VS I-20's `use Links(other)` accept, VS I-19's freshness example; WF III1's example, WF I10 vs I17 arm B, WF II1 vs the II5/II6/I8/§3.21 repairs, WF I6's example vs I7's premise, WF I4's class premise vs its own examples; BCB I-17 vs III-3 on one line, BCB II-8's example vs §7 and its own derivation, BCB II-3's `Live` example, BCB TERM vs II-15 row 1 | `COMPARISON.md` §6 | One table of every line where a rule and its own worked example give different verdicts, with the reading each derivation silently took — the direct evidence for O19 |

### D. Rule sets still incomplete for the boundary programs P4, P7, P8

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G15** | **No per-required-property ledger for P4, P7, P8.** `PROGRAMS.md` fixes P4 at four lines plus two properties, P7 at four lines plus three properties, P8 at two shapes plus three requirements. The comparison gives one matrix cell each and examines exactly one conjunct in depth (P4's first, in §5.4 Q1) | `COMPARISON.md` §1.2, §5.4 | A property × candidate table for the three programs: the rule that delivers each property, or the named missing rule that does not |
| **G16** | **P7's third required line is derivable under no candidate, and the comparison credits one.** "`old = replace(p,new); read(q)` … reads new" needs a contents fact that no rule installs: VS's I-7 states no `ensures` at all (cross S2, MR-2/MR-3), WF's I5/I6 establish none (G4/M3), BCB's `I-9` kills `val` and nothing installs it (S17). Cell note ⟨4⟩ nonetheless records "the other three required lines are delivered exactly" for VS, which its own §2.2 row refutes | `COMPARISON.md` §1.4 ⟨4⟩, ⟨6⟩, ⟨19⟩, §2.2 | Correct ⟨4⟩, and add a short list — "required properties no candidate delivers" — which currently holds P7's contents claim and, in two of three, the premises of `read` |
| **G17** | **P8's third conjunct is evaluated for one candidate.** "The rejection names the path whose state differs" is assessed only for VS (soundness S1, delivered). WF's P8(a) is underivable so the conjunct is untested; BCB's is never examined | `COMPARISON.md` §1.4 ⟨7⟩⟨8⟩⟨9⟩, §5.4 Q2 | Derive conjunct 3 under WF and BCB, or record it untested |
| **G18** | **The O7/O12 evidence conflates WF's two pointer forms.** §4.1's closing row ("storing a projection without suspending the base — WF: refused, `Carve(ρ)=∅`/`Span(ρ)=∅`") is true of a carve-entry `ptr<ρ↓w.k,T>`; WF's own P4 derivation stores a **coarse** `ptr<β,u64>` and states "`push` was never blocked by the stored pointer … no `Span` entry is created". Q1's table asks the boundary question about one construct and answers it with the other | `COMPARISON.md` §4.1 last row, §5.2, §5.4 Q1 | Split every O7/O12 row into coarse-name pointer and carve-entry projection, and say which of the two O12 is asking about |
| **G19** | **P4's reallocating arm has no derived continuation under any candidate.** BCB's derivation records "no rule rebuilds a stored nominal at the new backing"; WF's asserts in prose that "the writer re-forms the pointer from the container" with no line-by-line derivation; VS has no instance. This is the line P4 exists to test | derivations, P4; `COMPARISON.md` ⟨1⟩⟨2⟩⟨3⟩ | Derive the else-arm continuation under each candidate, including the re-formed pointer and what happens to a `Cursor` value whose `where` clause lost its support |

### E. Costs asserted without a count

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G20** | **Two of three have no total checking cost at all** (§3.1: VS "not stated", WF "not computed"; BCB stated and corrected to `Θ(S⁴·I·d + S⁵)`), so only the candidate that computed a total has a number a critic could attack. §3.2 says the three M8 figures are incomparable; §3.1 does not say the same of M10 | `COMPARISON.md` §3.1 | State the three M10 totals in one unit, or mark the row incomparable as §3.2 does, so an absent number is not read as a small one |
| **G21** | **The measurement row misattributes a shared baseline.** VS's 29.9/164.5/1691.0 ms and WF's 30.0/165.4/1690.7 ms are the *same inherited* figures — `proof-use-cost/baseline-2026-09-14.tsv`, `growing`, N = 16/32/64 — for the existing affine/entailment family, not a measurement of either rule set (no checker for any candidate exists). §3.1's closing row reads as two independent measurements against BCB's "asymptotic only, unmeasured" | `COMPARISON.md` §3.1 | Name the fixture and what it measures, or drop the measured/unmeasured contrast; the honest statement is that no candidate's own rules have been measured |
| **G22** | **No runtime-cost axis.** The comparison counts writer annotations (§3.3) and checker work (§3.1) and never counts what the *program* pays: VS's `Option` standard form (one discriminant word per resource and one branch per release site, over 13 items), BCB's P13 handle→slot map (one word per outstanding handle) and P1's outcome arm plus four proof-only branches, WF's nothing. These are named in prose and summed nowhere | `COMPARISON.md` §3 | A third table — runtime artifacts per item, summed per candidate — since this is the axis on which the `Option` repair and the declared map differ from zero |
| **G23** | **Unbounded per-use costs are carried as symbols and never instantiated.** VS "`O(H)` `use` steps per removal", WF "≥1 `INV` step per traversal read site", BCB "1 `use` per symbolic-write group per iteration". None is evaluated on a program, so the three are not comparable | `COMPARISON.md` §3.3, §5.4 Q3 | Instantiate all three on P3 as written — state `H`, the read sites and the iteration count — and give three numbers |
| **G24** | **The R4 fact yield is compared nowhere.** What each candidate licenses the backend to assume (VS §1.5's lowering, WF §1.4's erasure table, BCB's erasure) has no row, although the round contains a finding that one candidate emits a **false** fact (G11, WF-soundness F6) and R4 is the row the whole erasure story serves | `COMPARISON.md` §4 | One erasure/fact-yield row per candidate: what is emitted, from which rule, and what the critiques say about its truth |

### F. Places the comparison ranks despite "selects nothing"

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G25** | **§1.3's "accepted total" is a scoreboard.** It orders the candidates (VS 10 / WF 23 / BCB 28) over items of unequal content (G10), scored against expected verdicts `CASES.md` wrote under an earlier candidate (DESIGN.md Candidate A), with an accept that admits a hazard weighing exactly as much as a clean one (VS's refilled-slot refusal and WF's OD1 acceptance are both one cell). The "movement under the critiques" line (−2 / −7 / −1) reads as a ranking of robustness | `COMPARISON.md` §1.3 | Either drop the totals, or carry with them: the provenance of the expected verdicts, a statement that hazard-admitting accepts are counted as accepts, and the item-weight caveat |
| **G26** | **Three comparative superlatives in the comparison's own voice**: "the smallest taught inventory of the three" and "the safest rung on the ladder's pool case" (§5.1), "the cleanest structural claim in the round" and "(its strongest claim)" (§5.2, §2.3), "the strongest measured writer-cost result in the round" (§5.3). Each originates in a critique's §5 "what it spares", where it is that refuter's judgment; restated unattributed they are the comparison's | `COMPARISON.md` §5.1–§5.3, §2.3 | Attribute each to the critique that made it, or restate as a fact with its measure ("17 declared / ≥24 recounted constructs, against ≥34/≈69 and ≈46/≥57") |
| **G27** | **One superlative is also unsupported as stated.** "The strongest **measured** writer-cost result in the round" is said of the candidate whose M10 declaration is the only unmeasured one (cross F15) and whose own cost table contains one inexpressible item (L02), one underivable item (L05) and one unrepairable class (P4's reallocating arm) | `COMPARISON.md` §5.3 | State the measure in the sentence — annotations counted on the items that derive — and name the three items excluded from it |

### G. Round-constraint compliance the comparison does not close

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G28** | **M1 is corrected to *violated* for all three and the consequence is never drawn.** The round's hard rule is that M1 is a constraint a candidate satisfies or explicitly declares violated; §2.3 shows all three declared it met and all three overturned. While M1 is open, none of the 31-item matrix is yet established as a function of the source and the specification | `COMPARISON.md` §2.3, §6 | One paragraph stating what an owner may and may not conclude from the matrix while every candidate's acceptance relation is unfixed — and which cells would change under each unresolved reading (the three known ones: VS's `Γ`-closure at a join, WF's II5-vs-II8 retirement order, BCB's `FREEZE`-vs-SSA guard atom) |
| **G29** | **M2(ii) has a declaration row and no artifact ledger.** What each candidate's rules actually require the program to carry is scattered: VS's writer-bound discriminant and trailing `match`, BCB's `slot_live`/`live(hid)` occupancy word and its ghost index under O3, WF's nothing. §2.3 compares the *declarations*; the artifacts are the evidence | `COMPARISON.md` §2.3 M2(ii) row | Enumerate every runtime artifact each candidate's rules require, per item, beside the declaration |
| **G30** | **The round's own authoring rule is not audited.** "Every rule carries a pseudocode example" is checked nowhere, and where a rule and its example disagree no file records which one the derivation followed (G14). The rules files hold ≈119 / ≈60 / ≈59 example blocks against 59 / 56 / 55 rules, so the audit is cheap | `COMPARISON.md` §2.1 | Two columns in §2.1: rules with no example, and rules whose example contradicts the rule text |

### H. Owner decisions with evidence in this round and no routing

| # | Gap | Where | What would close it |
|---|---|---|---|
| **G31** | **Evidence is routed to 7 of the 19 owner decisions** (O3, O6, O7, O8, O11, O12, O15). Five more have direct evidence here and no row. **O16** (may a callee's exit state be two-armed): this round's P4 is three different answers — VS's single unconditional `push` rule, WF's I17 two-armed row with a minted `g#`, BCB's `ensures st('b) = ite(n0<c0, Live, Gone)` — and O16 decides R8a. **O17** (is `Pool`/`Arena` a library or an enumerated M3 entry — `VERDICT-CORE` calls it "the single most-repeated defect"): VS-soundness S8(c) (a pool binding's release would walk runtime occupancy), BCB-soundness F9 (the occupancy word), and all three pool renderings bear on it. **O1** (a stale handle in a reusing pool: R1(a)(ii) or M3(d)): WF's OD1 position against VS's III-7 and BCB's III-6, which cross X6 asks to be put as an O1 ratification. **O2** (does M2(ii) refuse a compare wrapped in a total `Option` read): VS's DV-4 family, seven items. **O5**: G13 | `COMPARISON.md` §5.4 | One evidence row per decision in §5.4's shape; O16 and O17 are the two where this round produced more evidence than the decision currently has |
| **G32** | **Q3 is not an owner question.** "Is a pool (or per-structure) invariant an acceptable writer cost" has no decision number, while O17 — which the same evidence bears on — is unnamed anywhere in the file. The owner is given a third question that no ruling consumes | `COMPARISON.md` §5.4 Q3 | Re-key Q3 to O17 (or to O15, which its `use`-step rows are really about), or state which ruling Q3's answer would inform |

---

## 2. Supporting detail

### 2.1 The five ignored fatals, with the line each lands on

```text
// VS — soundness F1 (R2(a) leak), absent from COMPARISON
g.slots[σ(q)].value = v          // accept: I-7 — total; no "empty" fact needed
                                 // reject: missing fact — the value AT the destination is affine
                                 //   and its release obligation is dropped, on every path
// VS — soundness F3 (R5(a) race), absent
par { reserve(inout v.meta,128) ∥ fill(inout v.data[0..4]) }
                                 // accept: II-22(i) — PATH refutes overlap, sibling planes
                                 // reject: missing fact — reserve frees and moves the storage
                                 //   the sibling branch is writing
// WF — soundness F6 (R4(a) false fact), absent
row { ρ1 @E ; ρS @W }  with ρ1 ⊑ ρS
                                 // accept: §1.4 — @E lowers to parameter noalias
                                 // the emitted fact is FALSE in every execution: a miscompile
// BCB — soundness F1 = cross F1 (R1(ii)), absent
write(p, 20)  at st('A) = ⊤      // accept: I-6 — premise is st ≠ Gone, and ⊤ ≠ Gone
                                 // reject: missing fact — ⊤'s Gone leaf is not excluded
// BCB — soundness F3 (R1(i) + R3), absent
v1 = take(v,i); f(); v2 = take(v,i)
                                 // accept: JF-QI re-instantiates RSt(…, Init, {}) after the call
                                 // reject: missing fact — the hole recorded at 'b[i] was discarded
```

### 2.2 What is derived, and what is not

| | S01–S07, B01–B13, L01–L06 | P1, P3, P4, P7, P8 | P2, P5, P6, P9 | P11–P19 | corpus programs |
|---|---|---|---|---|---|
| **VS** | derived | derived | asserted (P2, P5, P6, P9) | asserted (P13) | none |
| **WF** | derived | derived | asserted (P2, P5, P6); P9 not even asserted — no rule | asserted (P13) | none |
| **BCB** | derived | derived | asserted (P2, P5, P6, P9) | asserted (P13, P16, P11) | none |

31 derived items; 13 programs underived; 6 of the 13 carry verdicts in
`COMPARISON.md` §4.1 or §5.4.

### 2.3 Three counts that do not agree with their own tables

| Claim | Where | The table says |
|---|---|---|
| "VS — 16 missing rules", "WF — 14", "BCB — 13" | §2.2 headings | 18, 19, 21 rows — and the ordering reverses |
| L02: WF `accept`, BCB `accept` | §1.2 | ⟨47⟩ "no vocabulary"; ⟨48⟩ "inexpressible … an unlisted capability loss" |
| P7: VS "the other three required lines are delivered exactly" | ⟨4⟩ | §2.2's VS row (MR-2/MR-3): `replace` states no `ensures`, so "reads new" is not in `Γ` |

Verified as consistent, for the record: the §1.2 matrix tallies exactly to
§1.3's corrected rows (VS 2/8/13/7/1, WF 18/5/1/1/6, BCB 24/4/1/1/1, 31 each),
and the `†` counts (16/14/11) are right.

---

## 3. What the gaps do and do not affect

Stated without selecting. The three questions the round exists to serve are
affected unevenly:

- **Q1 (O7/O12)** is the most affected: its central row conflates two pointer
  forms (G18), the boundary program's reallocating arm is underived in all three
  (G19), and the three programs that would test a stored pointer outside a
  container — P11, P13, P16 — are underived (G01).
- **Q2 (O11)** is the least affected: L05, B05/B06, B08, B10–B13 and P8 are all
  derived line by line under all three, and the discriminator (L05) is derived
  three ways. Its exposure is G09 — six of the eight unclassified cells are WF's
  and five of those six are O11 items — and G28, since two of the three L05
  verdicts depend on an unresolved reading of the candidate's own rules.
- **A third question the file asks (Q3) is not an owner decision** (G32), while
  two decisions with fresh evidence (O16, O17) are unasked (G31).
