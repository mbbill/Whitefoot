# Round-3 writer handout manifest [M5R2-FIX-5]

What a blessed writer receives. Per the D2 spec-mass budget, the kernel spec and
the catalog are BOTH always in context; this handout does not copy the kernel
into the catalog — it bundles the documents and indexes the kernel sections a
writer must have surfaced. Nothing here is new normative content; it is a
reading guide that closes the "rule lives only in the kernel" defects the
writability rerun exposed.

## Documents in the bundle (all three, always in context)

1. `spec/kernel-spec-v0.6.md` — the 90-rule language definition (grammar,
   ownership, effects, prelude, operation tables). The source of truth for every
   spelling. Surface the sections in the index below.
2. `optables.md` (Appendix S) — the sealed-form op tables (`seq`, `table`,
   `conc_queue`) plus the S.0 conventions. Its signature columns are op-table
   meta-notation [CAT-1], NOT program text; the call-site grammar is [CAT-1a].
3. `cards.md` — the composition cards (C1 bounded cache, C2 FIFO/ring, C3
   iteration) with the worked, compilable examples a writer copies from.

## Kernel-spec sections to surface (rule ID -> why the writer needs it)

Conformer / iteration (the banned-loop replacement, cards.md C3):
- `[FN-3]` (§8) — contract/conformance: `conform T : C { member = fn; }`; one
  conformance per (type, contract), checked per member; effect rows are exact.
- `conform_decl` / `fn_bind` grammar (`[GRAM-2]`, §3) — the exact conform
  spelling: `conform CountEnv : SeqVisit<u32> { visit = count_visit; }`.
- `[FN-6]` (§8) — recursion is permitted (the second blessed iteration spelling).

Call convention:
- `[GRAM-11]` (§3) — a call to a user `fn` writes NAMED arguments in declared
  order (`f(line: l, from: i)`); a call to a table op writes POSITIONAL operands.
  This is the single rule behind the swp/seq "named vs positional" reds.

Function-signature and generics surface (the meta-notation-vs-real-syntax gap):
- `[GRAM-2]` `fn_decl`/`fn_sig`/`generics`/`gparam`/`region_params`/`param` (§3)
  — the real header grammar: generics bracket `<...>` before region bracket
  `[...]`, every `param` carries a `mode` (`own` / `&'r` / `&uniq 'r`), header
  runs through `{` on one line.
- `[GRAM-3]` `mode`/`targs`/`const` (§3) and `[CONST-1]` (§4) — const-generic
  params are lowercase idents (`fn f<const n: u64>['e](s: &uniq 'e seq<u8, n>)
  ...`); the `N`/`T`/`K`/`h` in op-table rows are [CAT-1] meta-notation, never
  written in a `fn_decl` or respelled at a call.
- `[FN-1]` (§8) — signatures state everything callers rely on; `[EFF-1]` (§9) —
  effect-row grammar (`reads, writes, allocates, traps` in that order; `pure` is
  the empty row).

Prelude (the `Ok`/`Err`/`Some` field-name reds):
- `[PRE-1]` (§15) — the prelude verbatim, including the field names writers got
  wrong: `Option<T> { None(); Some(value: T); }` and
  `Result<T, E> { Ok(value: T); Err(error: E); }`. A `match` binds
  `Err(error: e)`, `Some(value: v)` — the field name is `error`, not `value`.
- `[ERR-3]` (§10) — `let x: own T = try e;` Result propagation (same `E`).

Scalar operation tables (the invented-op reds):
- `[OP-1]`/`[OP-2]` operation tables and `[OP-7]`/`[OP-8]` (§7) — the closed
  scalar vocabulary and its edge semantics: compares `ieq ine ilt ile igt ige`
  (dotless, `(T,T) -> Bool`), moded arithmetic `iadd`/`isub`/`imul` +
  `.wrap`/`.trap`/`.checked`/`.sat`, `idiv`/`irem` + `.trap`/`.checked`, bitwise
  `iand ior ixor`, counts `ipopcount iclz ictz`. (Mirrored for the catalog in
  [CAT-7a].) There is no bare `iadd`/`isub`; strict-less-than is `ilt`.

Statement / place / borrow surface (the region-minting and `set` reds):
- `[GRAM-4]`/`[GRAM-5]` (§3) — statements and places: `set place = expr` where a
  `place` is a bare IDENT, `deref(p).field`, or `index<T>(p, i)`; `borrow_expr`
  (`&'r p` / `&uniq 'r p`) is an atom passed inline with no binding and no
  `move` (GRAM-9). Iteration statements `loop`/`break` exist here but are held
  out of the blessed surface (cards.md C3).
- `[GRAM-7]`/`[GIVE-1]` (§3) — the `let`-initializer `match` with `give e;` for
  conditional initialization (there is no `if`).

## Call convention: user fn vs table op [M5R3-FIX-1]

| callee | argument spelling |
|---|---|
| user `fn` | NAMED args in declared order [GRAM-11]; all type/region/const args explicit in one `<...>` list [TYPE-5] |
| table op | POSITIONAL operands, unnamed [OP-1]/[CAT-1]; region args only at a dedicated-result-region row [CAT-1a] |

Paired positive/negative:
- user fn `fn add3(a: own u64, b: own u64, c: own u64) -> own u64`: `add3(a: 1_u64, b: 2_u64, c: x)` OK; ~~`add3(1_u64, 2_u64, x)`~~ REJECTED (positional at a user fn, GRAM-11).
- table op: `seq_push(s, move v)` OK; ~~`seq_push(s: s, v: move v)`~~ REJECTED (named args at a table op); ~~`tbl_get['r](t, k)`~~ REJECTED (region list at a positional op).

## Derive your effect row (mechanical) [M5R3-FIX-3]

1. List the effects each op and fn your body calls exhibits, one by one, from the tables ([EFF-1] clauses `reads`/`writes`/`allocates`/`traps`).
2. Drop every effect confined to a region you minted inside the body with `region 'x { }` — it does not escape [CAT-5a](ii). (A queue brand `'q` is confined the same way and never appears in the row.)
3. A bounds-checked `index` without a discharging fact — and any `.trap` op, `check`, or `requires` block — exhibits `traps` [EFF-2].
4. Write the union in canonical order `reads, writes, allocates, traps` (`pure` if empty). Check BOTH directions: undeclared-but-exhibited and declared-but-unexhibited are each an error [EFF-2].
5. If the fn conforms to a contract member, its declared row must be contained in the member's ceiling (subsumption, [FN-7]/[CAT-5a](iii)) and equal what its body exhibits.

## Affine-spelling checklist [M5R3-FIX-6]

An affine value (owned composite, `box`, `arena`, `buffer`, uniq borrow, `slice` as `&uniq`, queue endpoint) is consumed exactly once by `move` [OWN-1]:
- Return an affine binding: `return move x;` — a bare affine place is a hard error.
- Pass an affine binding to a user-fn argument: `f(param: move x)` (named + `move`).
- Consume an affine operand of a table op: `move` it, e.g. `seq_push(s, move v)`.
- Bare-place exception: a place used as a NON-consuming operand of a table op is written bare — a borrow receiver, and (inside a `requires` block) a non-consuming place operand [FN-8].
- Copy values (primitives, `Bool`, tag-only enums, shared borrows, `hdl`) are ALWAYS bare; `move` on a copy value is a hard error [OWN-1].

## Diagnostic coverage — round-4 blind spot [M5R4-FIX-6]

The feedback/diagnostic cycle must surface two defect classes it missed in
round 4 (both slipped through as accepted), not only locally-fixable
spelling/derivation errors:
1. **Reborrow-out-of-borrowed-struct** (T-A): minting a new `&`/`&uniq` field
   borrow OUT of an already-borrowed wrapper — `&uniq 'e deref(h).items` where
   `h: &uniq 'e History`. v0 has no reborrowing; a helper must take the field
   PARTS from the owner (cards.md C1 no-reborrowing example), never a borrowed
   wrapper. This is a hard error the diagnostic must name at the borrow site.
2. **WRONG_CHOICE performance anti-patterns**: a legal-but-not-blessed shape a
   card supersedes — e.g. `seq_remove_at(s, 0_u64)` front-removal (O(len)/pop,
   SEQ-4) where C2's FIFO is the blessed route; using a plain SwissTable where
   C7 tiny-map wins; per-pop `conc_queue` where a single-threaded C2 ring
   suffices. These are not spelling errors, so a spelling-only checker passes
   them; the grader/feedback generator must flag the choice against the card set.

## How to use it

Copy spellings from cards.md's worked examples first; when a needed construct is
not in a card, look it up by the rule ID above in `kernel-spec-v0.6.md`. The op
rows in optables.md are meta-notation — translate them to program text through
[CAT-1a] (call-site grammar) and [CAT-5a] (effect rows), never by transcribing
the signature column literally.
