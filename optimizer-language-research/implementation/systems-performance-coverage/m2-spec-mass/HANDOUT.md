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

## How to use it

Copy spellings from cards.md's worked examples first; when a needed construct is
not in a card, look it up by the rule ID above in `kernel-spec-v0.6.md`. The op
rows in optables.md are meta-notation — translate them to program text through
[CAT-1a] (call-site grammar) and [CAT-5a] (effect rows), never by transcribing
the signature column literally.
