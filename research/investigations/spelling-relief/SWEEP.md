# Spelling rule and the v0.20 surface sweep

Status: investigation record, 2026-08-06. The rule below was agreed with the
owner in discussion (obligation-discharge thread) and governs this sweep; it
is not yet spec law — adoption runs through the language-change loop. This
file is the work list for the spelling-relief batch and the companion card
to `research/investigations/obligation-discharge/DOSSIER.md` §8. Removal
condition: superseded by the spec batch that lands or rejects these verdicts.

## The rule

A surface element earns its bytes by four tests plus one corollary:

- **T1 (decision):** an element may exist only if it carries a decision the
  checker cannot uniquely reconstruct from the remaining bytes of the same
  declaration. Uniquely reconstructible = ceremony = deletion candidate.
- **T2 (boundary):** redundancy that restates derivable facts is
  load-bearing exactly at trust boundaries — signatures, requires/ensures,
  effect rows, conformances, cross-declaration names — because it is drift
  detection. Inside a body it is rot. Boundaries stay fully explicit.
- **T3 (uniqueness):** any relief must preserve one-program-one-spelling:
  parse∘print identity, no second spelling of any checked program,
  verified mechanically against the corpus (FORM-1/FORM-2 machinery).
- **T4 (globality):** legality of a spelling may depend only on grammar
  class (construct kind, operation identity, declaration vs body), never on
  use-site context or on whether inference succeeds at that site. Relief is
  all-or-nothing per class; a class that cannot be relieved totally stays
  uniformly mandatory.
- **Corollary (no optionality):** nothing is ever "may write" — every
  position is mandatory or forbidden, decided by the class rules above.

Explicitly inadmissible bases: aesthetics; what AI models happen to emit
(motivation at most, never a criterion). Tiebreaks among isomorphic
surviving candidates use measurable quantities only: token counts, grammar
rule-count delta, LL(2) preservation, simplicity of the T3 uniqueness
argument.

## Sweep of v0.20 surface rules

### A. Whole-class deletions (T1 ceremony; all auto-migratable — the
canonical printer computes the new spelling from the old tree, so corpus
migration is mechanical and semantics-free)

| element | v0.20 rule | verdict |
|---|---|---|
| `targs` on value-typed table-op calls (`ieq<u64>(a,b)`) | GRAM-5 `call := callee targs?` | delete: operands are typed atoms under GRAM-9, so the type argument is reconstructible at 100% of sites. Per-op table column decides uniformly: type-choosing ops (`cvt`, `reinterpret`, `array_new`) keep their targs everywhere — a real decision lives there. |
| `index "<" type ">"` | GRAM-5 place grammar | delete: element type derivable from the place. Composes with the `p[atom]` respelling (C2). |
| `mode type` on body `let` | GRAM-4 `let_stmt` | delete: every RHS is typed (ops typed, calls typed by FN-1, literals suffixed). Note: this narrows the TYPE-5 redundant-explicit-facts class to boundary positions, per T2 — TYPE-5's rationale predates this rule and survives at boundaries only. |
| Bool-scrutinee `match` with `True()/False()` arms | GRAM-6/GRAM-7/PRE-1 | replace by `if expr { } else { }`: the two arm labels are reconstructible (always exactly these two, fixed order), so deleting them *is* the if form — a redundancy deletion, not a new alternative. Class rule is type-driven and global: Bool scrutinee → `if` is the only form; enum scrutinee → `match` is the only form. Empty `else` is forbidden, non-empty mandatory (content-driven, single spelling each way). |

### B. Whole-class keeps

| element | v0.20 rule | why kept |
|---|---|---|
| literal suffixes `1_u64` | FORM-5/FORM-7 | owner ruling: partial relief would be positional (violates T4); whole-class redesign (untyped literals + mandatory anchors) is a separate future investigation with its own T3/T4 proof. |
| loop/break labels | GRAM-4 | bare-`break`-when-single-loop is positional (T4); bare+labeled coexistence is two spellings (T3). Stays uniformly labeled. |
| named construction fields, match binders, call arguments | GRAM-8/10/11 | T2: cross-declaration drift detectors (rename/transposition), exactly as their existing R4 rationale states. |
| full signature surface: modes, types, effect rows, regions, requires | FN-1, EFF-1 | T2: the interface is the trust boundary; redundancy here is the review story. |
| `set` vs `let`, `move`, borrow spellings, `region` | GRAM-4/5 | genuine decisions (mutation, transfer, mode, lifetime). |
| FORM-5 float/STRING canonicalization, FORM-4 no-comments | — | out of scope; untouched. |

### C. Per-operation respellings (uniformity untouched: each operation keeps
exactly one constant spelling, as today; only the constants shorten. R3
selection by the objective tiebreaks)

1. Infix symbols for the hottest table ops (`a + b`, `a +wrap b`,
   `a == b`, `a < b`, …). **Key fact: with GRAM-9 (ANF) retained, an
   expression contains exactly one operation, so no precedence table
   exists and the T3 uniqueness argument is trivial.** This respelling is
   cheap *because* ANF stays.
2. `index<T>(p, i)` → `p[i]` (the sole place form respelled; unique
   trivially).
3. `check e else trap "msg"` → subsumed by the claim construct
   (obligation-discharge §8 item 1; named, `because`-carrying). Cross-track.
4. `.trap`/`.checked` OPNAME suffixes dissolve under obligation-discharge
   §2.9 (bare op = goal-carrying form); `.wrap/.sat/.strict` remain
   distinct operations with their own spellings. Cross-track; listed for
   composition.

### D. Deferred, own batches

1. **GRAM-9 relaxation (nesting single-use pure intermediates)** — deferred
   indefinitely by default: it is the one relief whose class rule keys on
   writer-visible but non-grammatical structure (use counts), it forfeits
   the precedence-free property that makes C1 cheap, and status quo wins on
   T4. Revisit only with new evidence.
2. **Literal-class redesign** (B1's future path).
3. **Counted range loop `for i in a..b`** — semantic addition (structural
   discharge), owned by obligation-discharge §8 item 6; its spelling lands
   with that batch.

## Batch and migration notes

- A + C form one spelling batch: one spec version, one mechanical corpus
  migration (printer-driven, zero semantic judgment), conformance verdicts
  respelled in the same change per the derived-material consistency rule.
- Ordering vs obligation-discharge slice 1: independent — discharge
  semantics works unchanged on today's match-based branches. Default
  recommendation: discharge slice first (deep semantics shouldn't rebase
  onto concurrent syntax churn; the spelling migration rebases mechanically
  over anything).
- Spec size: net shrink — A deletes productions and rule text; C changes
  spelling constants; the only addition is the `if` production, offset by
  deleting Bool-match legality text.
- Grammar verification: every change through the native grammar verifier
  before proposal, per WORKFLOW.
