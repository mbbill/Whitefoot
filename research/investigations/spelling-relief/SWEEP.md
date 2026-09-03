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
| Bool-scrutinee `match` with `True()/False()` arms | GRAM-6/GRAM-7/PRE-1 | replace by `if expr { } else { }`: the two arm labels are reconstructible (always exactly these two, fixed order), so deleting them *is* the if form — a redundancy deletion, not a new alternative. Class rule is type-driven and global: Bool scrutinee → `if` is the only form; enum scrutinee → `match` is the only form. Empty `else` is forbidden, non-empty mandatory (content-driven, single spelling each way). **Chains: an else-block whose content is exactly one if-statement must flatten to `else if` (mandatory, content-driven — kills the corpus's deep Bool-ladder nesting); any other else content must use a block. Canonical layout is single: multi-line, two-space indent, `} else {` on the join line, following the existing match-arm brace convention; no one-line form exists (a layout option would be a second spelling, T3).** |

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

   **What actually shipped, and why the row's own example overstates it
   (lead, 2026-08-08).** v0.23 respells `== != <= >=` and leaves `ilt` and
   `igt` as named calls, so `a < b` — this row's own example — is not legal.
   The asymmetry is not arbitrary and not implementation convenience; it is
   one irreducible collision, and locating it exactly changes which fixes
   are available.

   `<` in **type** position is not the problem: a comparison cannot occur
   there, and the bulk of the corpus's angle brackets are type constructors
   (`buffer<u8>`, `slice<'r, T>`). The collision is confined to expression
   position, where an atom may be followed by `<`, and it decomposes into
   three cases — two of which need only one token of lookahead beyond the
   atom already consumed:

   - **TYPEID `<`** — a generic variant constructor, `Some<T>(x)`. [FORM-3]
     makes TYPEID `[A-Z][A-Za-z0-9]*` a lexical class distinct from IDENT,
     and an ANF comparison's operands are atoms whose roots are
     lowercase-or-numeric, so no comparison begins with a TYPEID. (Worth
     confirming against the `atom` production before relying on it.)
   - **a reserved lowercase operation name `<`** — the retained-argument
     class (`cvt`, `reinterpret`, `array_new`, `arena_new`, `finf`, `fnan`)
     and the SYS operations (`arg_get`, `read_once`, `open_read`, …). These
     spellings can never be declared as anything, by [FORM-3]'s
     `ReservedLowerNames` rule, which is precisely what that rule says it
     buys: operation-versus-function resolution stays context-free. So set
     membership decides, with no resolution and no inference.
   - **a plain IDENT `<`** — a user-generic call, `f<i32>(x)`, against a
     comparison, `a < b`. **This one is irreducible**, and no bounded
     lookahead resolves it, because types nest: the closing `>` of
     `f<Result<buffer<u8>, E>>(x)` sits at unbounded distance, so no fixed
     *k* suffices and the "consume `<`, then decide on the next two tokens"
     mechanism does not generalize. This is a property of the grammar, true
     whether or not the current corpus happens to write a nested
     instantiation.

   So the real options are: leave the asymmetry; **give user-generic
   instantiation a distinguishing marker**, which is the only one that
   dissolves the collision rather than working around it; or drop infix
   comparisons altogether for symmetry, forfeiting the ergonomic win.

   Measured cost of the marker, migrated conformance corpus:

   ```
   git grep -ohP '(?<![A-Za-z0-9_.])[a-z][a-z0-9_]*(?=<)' \
     <branch> -- 'tests/conformance/cases' | sort | uniq -c | sort -rn
   ```

   About 34 sites across about 25 distinct function names, against 106
   type-position occurrences and about 86 reserved-operation occurrences.
   The user-generic classification is by inspection of that tally, not by
   resolution, so treat the figure as an order of magnitude rather than an
   exact count. Note also that `-E` silently treats `\b` as a literal `b`
   here — every result came back beginning with `b` on the first attempt —
   so this measurement requires `-P`.

   **The observation underneath.** `<>` is carrying two concerns at once,
   comparison and type application, and that duplication is what forces the
   choice. Rust pays for the same overload with turbofish. A marker on
   generic instantiation is therefore not added ceremony; it is naming which
   of two mechanisms is meant, which is the direction the residue-hunt axis
   points.
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

## Comparison respelling: rulings of 2026-09-03

Status: owner rulings, recorded here because this file is the C1 row's home.
The candidate that carries them into the specification is
`governance/spec-evolution/comparison-symbols-v041-candidate.md`; this
section is superseded when that candidate activates or is rejected.

C1 was re-examined after v0.40. The six integer comparisons stayed named in
v0.23 on the owner's whole-class ruling, not on evidence against symbols, and
the four-of-six asymmetry that ruling rejected was forced by the `<`
collision alone. Two statements in the record above were imprecise:

- Only `<` collides. `IDENT >` begins no other arm at the `expr` decision, so
  `>` never needed a marker.
- The collision is bounded. The token after `<` separates a type-argument
  list from a comparison operand in every case but a const-generic IDENT
  argument (`preserve<n>`, two sites in `tests/`), and one further token
  separates that; strong-LL(4) decides it. The "closing `>` at unbounded
  distance" framing looked for the wrong signal.

What changed since v0.23 and bears on the row:

- v0.40 gave `ile`, `ilt`, `ige`, and `igt` a second grammar role: the
  relation of a `header_invariant`, an `invariant_stmt`, and a relation-form
  `proof_use`, over `affine_expr` operands that are themselves infix with
  `+`, `-`, `*`, and parentheses. `ile(sum, 255_u32 * i)` is a prefix
  relation over infix arithmetic; `sum <= 255_u32 * i` is one grammar.
- Integer comparison is the most frequent table operation in the gate corpus
  (`ieq` 1276, `ile` 960, `ilt` 805, `ige` 245, `igt` 97, `ine` 61 sites
  under `tests/`), ahead of every arithmetic operator.
- A positional `ile(end, room)` was the one remaining direction-sensitive
  positional form; GRAM-8/10/11 keep names for exactly that transposition
  class (R4). `end <= room` puts the direction in the byte order.

Rulings (owner, 2026-09-03):

1. The six integer comparisons are respelled `==`, `!=`, `<`, `<=`, `>`,
   `>=` as one `compare_op` class of `infix_tail`. The symbols are
   integer-only table rows exactly as `+` is; `feq`..`fge` and `eeq`/`ene`
   keep their prefixed names (OP-7). In proof position the four ordered
   symbols replace `ile`/`ilt`/`ige`/`igt`; `==` and `!=` stay outside the
   invariant surface (INV-1).
2. The `<` collision is dissolved by a delimiter on call-site type
   application: `call := callee ("::" targs)? "(" ...`, spelled
   `cvt::<u8, u32>(w)` and `open_file::<'f, 'n>(...)`. `::` is one compound
   token in both attachment sets. It delimits the expression-position
   argument list and is admitted on the ground the `for (` parentheses are:
   T1 governs elements, not delimiters. Constructors (`Some<T>(x)`) and type
   position stay unmarked — neither collides, and `construct` is its own
   production (T4). The `expr` decision stays two-token. Strong-LL(4) was
   weighed and rejected for the diagnostics it costs: a comparison whose
   right operand is a nested expression (`a < b + 1_u64`, `i < len(p)`) —
   the first errors an ANF-unaware writer makes — is either committed to
   the call arm or reported as the union of two constructs' continuations,
   and DIAG-1's two-token GRAM-9 attribution becomes a four-token case
   analysis. Rust met the identical collision and keeps the turbofish for
   the same two reasons. Parsing measures 0.13–2.5% of compile time across
   `tests/programs`, so lookahead cost decided nothing.
3. Inequality is `!=`; the byte `!` enters the token alphabet only inside
   that compound. `<>` was rejected as a third meaning for the angle pair
   and a minority spelling; `/=` diverges from C's compound assignment
   (LEX-1).
4. The `proof_use` multiplier is retained. A multiplied relation-form source
   is parenthesized — `use 3 * (a <= b);` — and a bare relation source is
   not; `use 3 * name;` is unchanged. The parentheses delimit the multiplied
   premise so `*` does not carry two precedences on one line. Deleting the
   multiplied relation form was rejected: its rewrite through a named
   invariant changes proof structure (a factor-2 block becomes
   AUTO-redundant and must be dropped), which makes it a semantic change,
   not a spelling one.
5. Everything else stays named. Bool logic: `&&` and `||` short-circuit in
   every source language and Whitefoot's `band`/`bor` are eager, the LEX-1
   divergence `docs/bargain.md` records as a live W1 pitfall, and `&` is the
   borrow sigil. The bit family: `<<`/`>>` collide with a nested `>>` and
   GRAM-1 forbids a grammar-consulting lexer; shifts carry modes and
   obligations. Float and enum comparison keep their domain prefix. Every
   unary operation (`ineg`, `iabs`, `inot`, `bnot`) stays a call: the
   language has no prefix-operator position, and `-x` lexes as an operator
   form with an illegal suffix, so unary minus could never follow.
6. `infix_op` remains the arithmetic list and `const` keeps reusing it;
   `compare_op` is a separate production so `f::<n > 1>` cannot derive.

Migration cost was not a criterion (owner instruction). The tiebreaks were
the rule's own: token counts, rule-count delta, LL(2) preservation, and the
simplicity of the uniqueness argument.
