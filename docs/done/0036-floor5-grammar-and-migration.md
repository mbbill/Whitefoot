# 0036 — FLOOR-5 grammar path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `DONE` — closed by the v0.23 activation on 2026-08-09. The
  approved bytes are installed at `spec/kernel-spec-v0.23.md`, SHA-256
  `e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5`, and
  every derived artifact was brought to them in the same commit. Two gate
  failures survive the activation and neither is this record's: the
  `RegionsAndBorrows` capability gap in `general_borrows_…`, and
  `own3-pos-outlives-store`, the A3 counterexample the approved bytes name as
  a removed expressible form. The rounds below are frozen coordination
  history, not authority, and the live status they describe is superseded by
  this line.
- **Authority:** owner approval 2026-08-07 (`governance/APPROVALS.md`); the
  candidate `governance/spec-evolution/spelling-relief-candidate.md`; the lead's
  2026-08-07 rulings on this task's round-1 blocker report, which re-key FN-4's
  discharge premise, expand this task to full atomic activation, and sequence
  re-assembly after the fixed delta; and the lead's 2026-08-07 ruling that the
  three spec-identity checks are activation-gated by design
- **Owner / workspace (rounds 5–6):** exec-0036d /
  `<scratch-root>/wf-0036e` on branch
  `task/0036-floor5-grammar-and-migration`, rebased onto main at f80840d
- **Base revision:** b345e2c
- **Dependency:** none

## Goal

0031-style **full atomic activation** for the FLOOR-5 spelling batch on one
task branch. The original card scoped this task to the grammar path only while
still demanding a green compiler gate and adapter parity; the lead's 2026-08-07
ruling accepts that the card was under-scoped and expands the task to the whole
activation. Scope:

1. Re-assemble `kernel-spec-v0.23-candidate.md` from active v0.22 plus the
   **fixed** FLOOR-5 delta (FN-4's discharge premise re-keyed to the
   operand-derived selected type, the ten uncovered prose sites, GRAM-1's
   shape-kind enumeration, and the drafter's re-sweep for the same miss class).
2. Extend the grammar path — the `if` keyword, twenty operator spellings, the
   left-factored `expr`/`infix_tail`, `if_stmt`/`value_if`, the targ and
   let-annotation deletions, FORM-2 layout. Expected verifier total 69
   productions. Reviewer carry-forward as an explicit success criterion:
   production `infix_tail` maps to node kind `infix`, and every other
   production/node pair in this grammar shares a name, so this is where a
   generator deriving node kinds from production names breaks.
3. Extend the **semantic path**: TYPE-5 statement-local derivation replacing
   written annotations, OP-2 operand-derived row selection, GIVE-1 derived
   delivery with the empty-delivery-set rejection, GRAM-6 type-driven
   conditional forms, and `if_stmt`/`value_if` through resolution, checking,
   ENT-3 S1 branch facts, ENT-5 joins, lowering, and backend. FN-4's discharge
   shape in `calls.rs` and `catalog.rs` follows the re-keyed premise.
4. Repoint identity pins (0029/0030 style).
5. Migrate the corpus by scripted transform in scratch: deleted-class type
   arguments, body-let annotations, Bool matches to if/else with mandatory
   else-if flattening, infix respells. Every migrated file passes the branch
   compiler's parse and FORM-2 canonical audit; conformance cases migrate in
   the same change and no verdict changes meaning.
6. Evidence: verifier green on the branch with the expected production total;
   `make -C compiler check` and repo `make check` exit 0, exit codes read
   directly and never through a pipe; conformance adapter comparison against
   main's 386/1/14 lane. Plus the REVIEW PACKET — per-file diffstat, ten
   representative before/after excerpts covering every transform class, and the
   verdict-meaning statement.
7. STOP before merge, reporting the candidate SHA-256.

Discoveries outside the candidate stop the task with evidence. If the expanded
scope proves too large to finish reliably, hand back at a clean boundary as
0031's first executor did.

## Progress (round 1 — SUPERSEDED by round 5, retained for history)

Its `9ca13585…` bytes and its site count are both superseded; see "Round 5".

Step 1 only. `governance/spec-evolution/kernel-spec-v0.23-candidate.md` is
assembled from `spec/kernel-spec-v0.22.md` by a scratch script (deleted after
use) applying 57 verbatim anchors, each asserted to occur exactly once. Rule
count 128 unchanged, sections 20 unchanged, no let annotation survives in a
normative program, and the frontend contract changed only in the
`[FORM-1]`..`## 4. Types` section (22819B -> 26914B); the `[CONST-1]` and
`[EFF-1]` sections are byte-identical, as the delta predicts. Candidate
SHA-256 `9ca13585dca6668bec9c1b4c219cbb5fc94559bcc29f67d04d52bbb61af5eef4`.
These bytes are NOT ready for step-4 approval; see the blocker.

Steps 2–5 not started. They are deliberately not started rather than partly
done: the blocker's likely repair moves TYPE-5's retained-argument class, which
would change the terminal inventory, the grammar tables, the migration counts,
and the candidate hash.

## Read-only preparation (round 2, at 46c1a0c)

Semantic-path site map, measured not estimated. The load-bearing finding is
that `if`/`value_if` is far cheaper than the card implies: `CheckedEnumType`
already has a `Bool` variant and `check/control/matches.rs:193` already builds
a Bool `MatchDescriptor`, so both forms check into the *existing* checked
Bool-match statements. Entailment (`entailment/flow.rs:208,219,958,978,1155,
1163`), lowering (`lowering/builder.rs:506,519`), cleanup, drops, and backend
all consume `CheckedStatement::Match`/`ValueMatchLet` and need no change — ENT-3
S1 and the ENT-5 join come out isomorphic for free, which is what the candidate
asserts. GRAM-1's 1:1 production-to-node law governs the source tree; the
checked IR is compiler-internal, so this is not desugaring in the spec's sense.

- `if`/`value_if` new work: syntax productions and nodes, the FORM-2 printer,
  and GRAM-6's three new rejections in `check/control.rs` and
  `check/control/matches.rs` (Bool-scrutinee `match`, empty `else`, else-if
  flattening).
- Let-annotation deletion: `check/control.rs:443 check_let` reads the `Mode`
  and `Type` children and threads the declared type into
  `check_match(.., Some(expected))`. The new GIVE-1 inverts that direction —
  the delivery set produces the type by agreement, so the descriptor returns
  the derived mode and type from the first delivering `give`, later `give`s are
  checked for exact agreement, and the empty delivery set rejects at the
  `let_stmt` node. The OWN-5 `SliceValueMatch` guard and the propagate path
  re-key onto the derived type.
- Type-argument deletion: 31 `Production::Targs`/`Targ` read sites across
  semantic and resolution; the table-op ones concentrate in
  `check/expressions/calls.rs` (4). Retained-class readers stay. FN-4's
  discharge shape sits at `check/expressions/calls.rs:150` and
  `resolution/catalog.rs:108` and follows the re-keyed premise.
- FORM-2 printer: `finalize/canonical/format.rs` is table-driven
  (`is_line_bearing:18`, `is_block_bearing:58`). Adding the two productions is
  one line each; the new mechanism is the join line (`} else {`,
  `} else if … {`), since no production renders a close-and-open brace line
  today. The `fn … requires` `} {` line is the precedent to follow.
- Grammar tables are the real risk: `syntax/grammar/generated.rs` is 4131 lines
  of committed LL(2) data (65 productions, 75 decisions, 530 nodes, 364 select
  atoms, 2003 select rows) with no in-repo generator; 0030 and 0031 both used
  one-shot scratch generators. Going to 69 productions and +21 fixed terminals
  (76 -> 97 predicates, still inside `TerminalSet`'s u128) moves FOLLOW sets
  broadly, because operator tokens newly follow every atom — `psuffix*`,
  `place`, `atom_list`, and `expr` rows all shift. Plan: build the generator so
  it reproduces v0.22's existing tables byte-for-byte first, then extend. The
  reviewer's `infix_tail`/`infix` name-mismatch carry-forward is a property of
  that generator, so it is a first-class test of it.

## Generator reproduction gate (round 3, 2026-08-07)

The generator is built (scratch, `<scratch-root>/wf-gen`): it
reads a numbered specification, parses the normative EBNF — including the four
inline prose productions the hand-back brief warned about — and emits the whole
of `generated.rs`. Two inputs are historical rather than derived and are carried
as explicit tables: the `Production` enum order, and the decision-slot order
(`psuffix`'s choice occupies slot 74 because v0.22 appended it).

**Reproduced byte-for-byte:** the header, the `Production` enum, `PRODUCTIONS`
(65), `PRODUCTION_ROOTS`, `PRODUCTION_OWNERS`, `GRAMMAR_CHILDREN` (465),
`GRAMMAR_TERMINALS` (232), `GRAMMAR_NODES` (530 — every kind, arena range,
decision slot and atom-only flag), and `DIAGNOSTIC_ORDER` (76). `DECISIONS`
matches on all 75 nodes, kinds, contexts and arm counts.

**Not reproducible, and the reason is a defect in the committed data.**
`SELECT_ROWS` is 2003 committed against 1893 derived. Read as what
`select_arm` actually consumes — the set of predicate pairs per arm — the
derived table is a strict subset of the committed one:

- 70 predicate pairs across 27 decisions appear only in the committed table;
- 0 appear only in the derived table;
- all 70 involve `]` (`RightBracket`) at a position the grammar cannot produce.

Witness: decision 8 is `variant := TYPEID "(" vfield_list? ")" ";"` at the
`vfield_list?` optional. The committed table carries `(arm 1, RightBracket,
Semicolon)` — on lookahead `] ;`, skip the field list. Nothing in the grammar
puts `]` there; the parser's very next action after that arm is
`Match(RightParen)`, which `]` cannot satisfy. The same shape recurs after
`type`, `targs?`, `param_list?`, `effect` and the rest of the 27.

A further 57 rows differ only in provenance: the committed row names the first
node in node order bearing that predicate (192 = `region_params`' `]`, 41 =
`variant`'s `)`, 125 = `law`'s `,`, 170 = `generics`' `>`) where the derivation
names the real site (478 = `psuffix`'s `]`, 421/432/470 = call/construct/deref
`)`). Provenance feeds diagnostics only.

84 committed rows carry provenance node 192 — exactly the "84 discovered-missing
`]`-closing SELECT_2 follow rows" recorded in `docs/done/0031-v022-grammar-and-
respell.md`. That patch added `]` far more broadly than the grammar warrants and
attributed it to the wrong node.

**The difference is inert.** With the derived table swapped in,
`cargo test --manifest-path compiler/Cargo.toml --lib` reports 517 passed,
1 failed. The single failure is `assert_eq!(SELECT_ROWS.len(), 2_003)` at
`compiler/src/syntax/grammar/tests.rs:126` (`left: 1893, right: 2003`) — a
literal count pin that every grammar change already updates (0030 moved it
1925 -> 1959). No parser, diagnostic, finalize or conformance test changed
verdict. The committed file has been restored; this is a measurement, not a
change.

**Ruled (A), canonical generation, and implemented in 7f68c71.** The generator
is now `compiler/src/bin/grammar_tables/` (bin `whitefoot-grammar-tables`).
`--check` regenerates from the active specification and compares against the
committed tables; the same comparison runs as a test inside
`make -C compiler check`, so "the committed tables are the tables the
specification's grammar implies" is machine-checked from now on. The two
historical inputs — `Production` enum order and decision-slot order — are
explicit tables carrying comments that say they are historical, not derived.

The canonicalization was landed as its own commit so the v0.23 delta lands
against a clean base. Its complete diff is the first two of the three
components the lead asked to see enumerated:

| component | effect |
|---|---|
| 70 grammar-underivable predicate pairs dropped, 27 decisions | landed 7f68c71 |
| 57 provenance corrections | landed 7f68c71 |
| rows the v0.23 grammar adds | not started |

`SELECT_ROWS` 2003 -> 1893 and the count pin moved with it, as it does for
every grammar change. The third component will move the count again; report it
as its own number rather than netting it against these two.

## Definition of done for this branch (lead ruling, 2026-08-07)

**The three spec-identity checks are activation-gated by design.** A branch
carrying a specification the owner has not approved is legitimately red on
them, and that is the checks working: the identity of the active specification
cannot be true of bytes no one has approved. They are a defect neither in task
0039 nor in this task's candidate-stage pinning, and they are **not** to be
silenced, weakened, or made green by writing an `ACTIVE-SPEC:` line.

The branch's finish line is therefore **green except exactly these three, each
failing for the reason named**:

| check | fails because |
|---|---|
| `spec::tests::path_and_version_label_agree` | the pin names the candidate path, not `spec/kernel-spec-v0.23.md` |
| `spec::tests::computed_identity_is_the_approved_digest` | the embedded bytes hash to a digest the owner has not approved |
| `whitefoot-spec` `tests::recorded_chain_ends_at_the_embedded_specification` | the `ACTIVE-SPEC:` chain ends at v0.22 |

The **activation commit closes all three at once**: install the approved bytes
at `spec/kernel-spec-v0.23.md`, repoint the pins there, and append the
`ACTIVE-SPEC:` line. Anyone reading these three later should read them as the
gate that is still waiting for owner approval, never as breakage to fix.

## Round 6 (2026-08-07) — re-assembled from the corrected delta

The lead repaired [OP-1] (iii) in the delta (main, 32e2af4): the anchor now
takes the whole `ModeWords` sentence including its trailing
`{wrap, trap, checked, sat, strict}`, which the prefix anchor had left stranded
as a second "in this version" clause. Round 6 re-assembled from that delta
rather than editing the round-5 bytes.

**Candidate SHA-256
`a92b45138c82c3d19dc2f0bfdfe2d04b5571ccc898d6427c9661bf0903b2918e`.**
Same shape as round 5 and re-verified rather than assumed: 74 verbatim anchors,
**64 sites across 34 rules**, each asserted exactly once and all matching first
run; **128 rules**, **20 sections**.

The assembled [OP-1] (iii) sentence now reads once:

> Let `ModeWords` be exactly the suffix alternatives in FORM-3's active OPNAME
> formation rule together with the operator-form suffixes of [GRAM-1]; in this
> version the two carriers share one closed set, `{wrap, trap, checked, sat,
> strict}`.

**The whole-file difference from the round-5 candidate is exactly one line** —
the `ModeWords`/`DotlessOperationNames` paragraph. That is also the round's
control on the assembler itself: the round-5 script had been deleted, so round 6
rebuilt it, and a faithful rebuild landing on a one-line diff is what says the
rebuild is faithful.

**The derived tables are byte-identical to round 5's**, re-derived and compared
rather than assumed — the delta's change is normative prose in [OP-1] and moves
no EBNF. So the 69/84/97 triple, the +1371 SELECT rows, the 76 -> 97 terminals,
and the whole table decomposition stand exactly as round 5 reported them. Only
the three digest pins moved: `compiler/src/spec.rs`,
`tests/conformance/runner.py`, and the ledger's v0.23 binding.

**Gate states, exit codes read directly from `$?` without a pipe.**

- `cargo test --bin whitefoot-grammar-tables` (the derivation check against
  v0.23): **exit 0**.
- `whitefoot-grammar CANDIDATE CANDIDATE`: **exit 0** — **69 productions, 84
  decisions, 97 terminal predicates**.
- `cargo fmt --all -- --check`: **exit 0**. `cargo clippy --all-targets -D
  warnings`: **exit 0**.
- `make -C compiler check`: **exit 2**, lib tests **253 passed, 270 failed** —
  the same count and the same classification as round 5.
- `make check`: **exit 2**, failing at the compiler stage after passing
  repository invariants, spec append-only, spec archive integrity (23
  specifications), and the 18 conformance plumbing tests.

The failure classification is unchanged: 266 v0.22-spelled sources under the
v0.23 grammar (0038), 1 stale operation catalog, 1 lexer gap, and the three
activation-gated spec-identity checks above. None is a table, terminal, or
derivation failure.

**Workspace note.** Round 5's worktree `<scratch-root>/wf-0036c`
was removed between rounds and this branch was re-homed to
`<scratch-root>/wf-0038-exec` while task 0038 sits unclaimed in
`docs/planned/`. Round 6 therefore worked in a separate detached worktree,
`<scratch-root>/wf-0036e`, rather than committing inside another
task's workspace. The branch ref needs one fast-forward to pick this round up.

## Round 4 (2026-08-07) — superseded bytes, retained findings

Round 4 assembled a candidate from the delta at 1a41eed (31 rules, 61 sites)
at `9135ac6c…`, extended the tables, and repointed the pins. Those bytes are
superseded: the delta has since moved to 34 rules at 64 sites. Round 5
regenerated rather than patched, and the round-4 commits are not in this
branch's history (old tip `6bd069d`, reachable if the reasoning is wanted).

Three round-4 findings survive and were not re-derived.

- **The `infix_tail`/`infix` carry-forward does not fire in the generator.**
  `GrammarNodeKind` is EBNF structure (`Production`, `Terminal`, `Sequence`,
  `Choice`, `Group`, `Optional`, `Repeat*`), not a core-tree node kind, and
  `Production` carries no name string anywhere in the compiler — DIAG-1 node
  identity is by ordinal. The name mismatch lands in the parser's tree
  construction, where an `infix` node must span the complete `expr`. That is
  0038's, not a generator risk.
- **The [OP-2] (g) defect round 4 reported is now carried by the delta**, with
  its repair and the seventh sweep pattern. Round 5 applies the repaired text.
- **The lexer gap is not the mechanical add it looks like.** The candidate
  makes a bad operator suffix a terminal-membership rejection while a lone `!`
  stays a raw lexical defect, so it needs its own negative cases and DIAG-1
  attribution review.

## Round 5 (2026-08-07) — re-assembled, tables extended, one blocker

**Candidate.** `governance/spec-evolution/kernel-spec-v0.23-candidate.md` is
regenerated from `spec/kernel-spec-v0.22.md` plus the current delta by a
scratch script (do_not_scan, deleted after use). SHA-256
`935b9538df69f6f6289e8a6c99004db45a1f5e1865929c4b7cc1ced861bec9d2`.

74 verbatim anchors implement all **64 sites across all 34 rules**, each
asserted to occur exactly once and all matching on the first run; 74 rather
than 64 because ten sites are one contiguous delta site spanning several
sentences or table rows. The per-rule site counts reproduce the delta's header
claim exactly. Where the delta states a replacement as a blockquote or fenced
block, the script read those bytes out of the delta rather than transcribing
them, so the largest replacements cannot carry a transcription error.

Self-verified on the output: **128 rules** unchanged and distinct, **20
sections** unchanged, and of the verifier's three frontend-contract ranges only
`[FORM-1]`..`## 4. Types` changes, **22819B -> 26904B**, with
`[CONST-1]`..`## 5. Ownership` (1966B) and `[EFF-1]`..`[EFF-2]` (1789B)
**byte-identical** — exactly what the delta predicts. No normative text retains
a deleted-class type argument or a `let` annotation; the three surviving
`ineg.wrap<T>` spellings sit in the frozen v0.14 `Prior:` paragraph the delta
clears as history. (Round 4 reported the baseline range as 22808B; 22819B is
what the verifier's own line-start rule measures on the unchanged v0.22 file,
and round 1 measured the same.)

**Cross-checked against round 4's superseded bytes.** Exactly nine lines
differ, and each is an accounted delta change: the status header, the three
new seventh-pattern sites [OWN-13] [OP-4] [SYS-13], the repaired [OP-2] (g),
the [DIAG-1] closing citation re-keyed to the callee's class, the extended
[DIAG-3] and [ENT-3] S4 anchors, and [OP-1] (iii). Nothing else moved.

**One assembled sentence reads badly and was left literal.** [OP-1] (iii)'s
anchor is a prefix of the `ModeWords` sentence, so applying it literally leaves
the original tail standing and the result says "in this version" twice: "…
together with the operator-form suffixes of [GRAM-1]; in this version the two
carriers share one closed set; in this version it equals `{wrap, trap, checked,
sat, strict}`." Round 4 silently smoothed this to ", `{wrap, trap, checked,
sat, strict}`." and did not report it — the same unrecorded-repair shape round 4
itself caught round 1 committing at [OP-2] (g). Round 5 applied the delta
literally and reports it instead. The delta's anchor or its replacement needs
one more byte of drafting before these bytes go to approval; smoothing it here
would be an unrecorded editorial change to normative text.

**Tables.** 65 -> **69 productions**, regenerated from the candidate and
installed. The derivation gate inside `make -C compiler check` passes against
v0.23 (exit 0), so "the committed tables are the tables the specification's
grammar implies" holds for the new grammar.

| table | v0.22 | v0.23 | delta |
|---|---|---|---|
| PRODUCTIONS | 65 | 69 | +4 |
| GRAMMAR_CHILDREN | 465 | 522 | +57 |
| GRAMMAR_TERMINALS | 232 | 263 | +31 |
| GRAMMAR_NODES | 530 | 591 | +61 |
| DECISIONS | 75 | 84 | +9 |
| SELECT_ATOMS | 361 | 415 | +54 |
| SELECT_ROWS | 1893 | 3264 | **+1371** |
| DIAGNOSTIC_ORDER | 76 | 97 | +21 |

**The third diff component, as its own number.** The v0.23 grammar **adds 1371
SELECT rows**, 1893 -> 3264. Not netted against the two components 7f68c71
landed:

| component | effect |
|---|---|
| 70 grammar-underivable predicate pairs dropped, 27 decisions | landed 7f68c71 |
| 57 provenance corrections | landed 7f68c71 |
| rows the v0.23 grammar adds | **+1371** (this round) |

Independently reproduced: these table numbers match round 4's exactly, derived
from a candidate that differs from round 4's in nine lines of normative prose.
That is the expected result — none of the nine touches the EBNF — and it is a
real cross-check on the generator rather than a coincidence.

**Terminals.** 76 -> **97 predicates**, 68 -> **89 fixed spellings**: `if` plus
the twenty `infix_op` operator spellings, and no `&&`, `||`, bare `<` or bare
`>`. Verified against the candidate's own `infix_op` block, not transcribed on
trust. `ALL_FIXED_TERMINALS` order is the fixed subsequence of the derived
`DIAGNOSTIC_ORDER`; that rule was confirmed by **reproducing v0.22's committed
68-entry array from v0.22's committed `DIAGNOSTIC_ORDER`** before being used
for v0.23. `TerminalSet`'s u128 still holds 97.

**One real defect found and fixed, not classified away.** `TerminalPredicate::
index()` hard-coded the eight external predicates at 68..75, the old fixed
count. Growing the fixed inventory to 89 made `Fixed(Minus)` (declaration index
73) collide with `Literal` (73), so one bit decoded as two predicates and
`membership_set_retains_noncompeting_overlap` failed with
`[Fixed(Unit), Fixed(Minus), Literal]` for a two-element set. The indices now
run 89..96. This was mine, it was a genuine terminal-inventory defect, and it
is repaired rather than reported as expected breakage.

**Pins name the candidate, never `spec/`.** Installing into `spec/` is the
activation step and needs the owner's step-4 exact-byte approval, which has not
been given. `compiler/src/spec.rs` (version, path, `include_str!`, digest),
`compiler/src/bin/spec.rs`'s `include_bytes!`, `compiler/src/bin/grammar.rs`'s
triple 65/75/76 -> **69/84/97**, `compiler/src/syntax/grammar/tests.rs`'s count
pins, `tests/conformance/runner.py` (path and digest), and
`spec/derivation/derivation-ledger.md` (a v0.23 candidate-stage entry following
the v0.21 entry's wording). `docs/roadmap.md` is deliberately not repointed:
the active-authority line moves at activation, which 1e23d03 shows the lead
doing in the same commit as the `spec/` repoint.

Two pin classes moved that no earlier brief listed, both ordinary version
maintenance. `qualification.rs` carries three `ACTIVE_KERNEL_SPEC_VERSION !=`
rows that have moved at every bump since v0.19 (`git log -L` confirms
v0.19 -> v0.20 -> v0.21 -> v0.22); left at v0.22 they fail command-entry
qualification for ten backend and driver tests. And the conformance runner's
three sandbox builders created `spec/` and wrote the active specification into
it by basename, an assumption the candidate path falsifies; they now create the
active specification's own parent. The lookalike-authority test keeps its decoy
in `spec/`, which makes it a slightly stronger control, not a weaker one.

## Round-5 blocker — candidate-stage pinning and task 0039 are incompatible

**This needs a ruling and is the reason the round hands back here.** Task 0039
landed spec-identity machinery that assumes the compiler's active specification
is an installed, owner-approved file under `spec/`. This task's ruled shape —
pin at the candidate, do not install — cannot satisfy it. Three checks fail,
and none is a migration failure that 0038 will clear:

| check | what it asserts | why the candidate stage cannot satisfy it |
|---|---|---|
| `spec::tests::path_and_version_label_agree` | `ACTIVE_KERNEL_SPEC_PATH == format!("spec/kernel-spec-{VERSION}.md")` | the candidate path is not under `spec/` |
| `spec::tests::computed_identity_is_the_approved_digest` | the embedded bytes hash to the digest the owner approved | the owner has not approved `935b9538…` |
| `whitefoot-spec` `recorded_chain_ends_at_the_embedded_specification` | `governance/APPROVALS.md`'s `ACTIVE-SPEC:` chain ends at the embedded specification | the chain ends at v0.22; adding a v0.23 link would record an activation the owner has not granted, and `spec-archive-integrity` would then demand `spec/kernel-spec-v0.23.md` exist |

Exact failure text for the third: `the chain ends at v0.22 but the active
version is v0.23`, `the chain ends at v0.22 but the specification is titled
v0.23`, `the chain records b133b793… for v0.22, but its bytes hash to
935b9538…`.

None was touched. Weakening any of them, or writing an `ACTIVE-SPEC: v0.23`
line into the approval record, would be a governance breach: the first two are
0039's deliberate controls and the third would fabricate an owner approval.
`spec-archive-integrity` itself still passes — 23 recorded specifications hash
as recorded — precisely because the chain was left alone.

The choice is the owner's and the lead's, not an executor's. The shapes
available are: (a) approve these bytes and activate, which makes all three
green by installing into `spec/` and adding the chain link; (b) rule that
candidate-stage pinning is a recognized state and adapt 0039's three checks to
admit exactly the two exact forms, which keeps their strength but is a change
to governance controls another task just landed; or (c) rule that the compiler
never pins a candidate, which unwinds this task's step-4 shape and means the
tables cannot be committed until activation. This branch implements the ruled
shape and stops.

## Round-5 gate states, exit codes read directly

Read from `$?` without a pipe.

- `make -C compiler check` **before** any round-5 compiler change: **exit 0**
  (with the re-assembled candidate already committed).
- `make -C compiler check` **after**: **exit 2**. `cargo fmt --all -- --check`
  and `cargo clippy --all-targets -D warnings` both exit **0**; the lib tests
  are **253 passed, 270 failed**.
- `make check` (repository): **exit 2**. Its earlier stages pass —
  repository invariants, spec append-only, spec archive integrity (23
  specifications), and the conformance plumbing tests (18 OK) — and it fails at
  the compiler stage on the same 270.
- `cargo test --bin whitefoot-grammar-tables`: **exit 0**, the derivation check
  against v0.23. This is the round's load-bearing verification.
- `cargo test --bin whitefoot-grammar`: **exit 0** (8 tests).
- `whitefoot-grammar CANDIDATE CANDIDATE`: **exit 0** —
  **69 productions, 84 decisions, 97 terminal predicates**. Run in the
  two-argument form task 0039 introduced. Against the v0.22 baseline the
  verifier correctly refuses (`candidate changes the lexer or source grammar of
  the baseline`), which is the fail-closed result the delta's §2 predicts for a
  grammar-extending batch.

**All 270 lib failures classified. None is a table, terminal, or derivation
failure.**

| class | count | owner |
|---|---|---|
| v0.22-spelled test sources under the v0.23 grammar | 266 | 0038 |
| operation catalog still carries v0.22 spellings | 1 | next unit |
| lexer cannot form the operator tokens | 1 | next unit |
| spec identity (the blocker above) | 2 | owner/lead ruling |

Plus one `whitefoot-spec` bin failure, the third spec-identity check, for three
spec-identity failures in total.

The 266 are one failure mode with a direct reproduction. `let a: own i32 =
40_i32;` is rejected at the `:` citing GRAM-4 with `ExpectedTerminals(
TerminalSet(65536))` — bit 16 is `FixedTerminal::Equal`, the `=` that A3's
annotation deletion now puts there. The positive control matters as much:
`let a = 40_i32;` and `if ilt(a, 50_i32) { … }` both **parse** under the new
tables and fail later at unimplemented semantics (`InvalidCanonicalTree`,
`InvalidFinalizedTree`), so the extended grammar accepts the new forms rather
than merely rejecting the old ones.

`resolution::catalog::tests::catalogs_match_independent_extraction_from_exact`
is evidence, not breakage. Its extraction side reads the active specification's
op column and returns `["+wrap", "-wrap", "*wrap", "+", "-", "*", "+checked",
…, "==", "!=", "ilt", "<=", "igt", ">=", …]` — exactly the respelling [OP-1]
site (i) specifies, O1 asymmetry included. The compiler's hand-written catalog
is the stale side, so this failure independently confirms the op-column
respell landed correctly in these candidate bytes.

## Round-5 successor brief

Rounds 1–5 are discharged; do not re-derive them. Base yourself on this branch.

1. **Get the round-5 blocker ruled.** Nothing else on this branch can reach a
   green gate until the candidate-stage pinning question is settled, and the
   answer decides whether the next step is activation or an adaptation of
   0039's checks.
2. **The lexer's operator form** — the one non-corpus gap. `TokenKind` gains
   `OperatorForm` plus the four compound comparisons;
   `compiler/src/lexer/scanner.rs:51`'s dispatch gains `==`, `!=`, `<=`, `>=`
   ahead of the single-byte `=`/`<`/`>` arms, the `+ * / %` starts, and the
   `-`-not-followed-by-digit-or-`>` case (the existing `b'-'` guards at lines
   59 and 62 are the precedent); `compiler/src/syntax/classifier.rs:106` gains
   the matching arms, where `FixedTerminal::from_spelling` already does the
   work. Not a mechanical add: see the round-4 finding above.
3. **The operation catalog respell** in `compiler/src/resolution/catalog.rs` —
   the independent extraction already returns the right answer.
4. **The corpus migration (0038)** — 266 failures, one failure mode.
5. **Fix [OP-1] (iii)'s anchor in the delta** before these bytes go to
   approval, and decide whether the doubled clause is the drafter's or the
   assembler's to resolve.

**Two consequences for the lead.** Two other candidates are drafted against
v0.22 and sit on a superseded base:
`governance/spec-evolution/provenance-gate-candidate.md` and
`governance/spec-evolution/ent5-loop-fix-v024-candidate.md`. And the generator
derives tables for exactly one grammar at a time, so pointing it at a future
candidate means moving the `ebnf.rs` fenced count first, which transiently reds
the `--check` comparison against the active specification.

**One trap that survives.** Byte-for-byte agreement between the tables and the
EBNF proves they agree; it never proves the extended tables are *complete*. The
migrated corpus parse remains the completeness oracle, and it has not run.

## 0031's defect, recorded where it will be found

Task 0031 hand-added 84 `]`-closing SELECT_2 follow rows (its record says so).
The rows are wrong in two independent ways:

- **Over-broad.** 70 of the resulting predicate pairs, across 27 decisions,
  place `]` at a position the grammar cannot derive. Decision 8 is
  `variant := TYPEID "(" vfield_list? ")" ";"` and carried
  `(arm 1, RightBracket, Semicolon)`; the parser's next action after that arm
  is `Match(RightParen)`, which `]` cannot satisfy.
- **Mis-attributed.** All 84 name provenance node 192, `region_params`' `]`,
  rather than node 478, `psuffix`' `]` — the site that actually made `]` a
  follower in v0.22. The same first-occurrence mistake affects 57 rows for
  `)` (41 instead of 421/432/470), `,` (125 instead of 438/458), and `>`
  (170 instead of 241).

**Nothing caught it for a whole version, and the verifier could not have.**
`whitefoot-grammar`'s triple counts productions, decisions and terminal
predicates; none of those three moves when rows are added, and its internal
consistency check only asks that every arm has a row and that arms stay
disjoint — adding rows to one arm violates neither. The corpus could only ever
catch *missing* rows, by failing to parse; a spurious row is invisible to it
because no valid program reaches that lookahead. That asymmetry is why the
derivation check is the right home for this invariant.

Harm was bounded: `select_arm` requires both positions to match, and the
terminal `Match` tasks still enforce the real tokens, so a spurious row could
degrade a diagnostic but not admit an invalid program. Wrong provenance
mis-attributes DIAG-1 name slots and expected-set reporting.

## Round-3 successor brief

Start here. Everything the round-2 brief says about building a generator
(its items 3, 4 and 5) is discharged — the generator exists, is in-repo, and
is gated. Its items 1 and 2 are semantic-path findings that belong to 0038,
not to this task's remaining scope. Base yourself on branch tip 7f68c71,
rebased onto main at 6c0333e.

**What is left, in order.**

1. **Re-assemble the candidate** from `spelling-relief-candidate.md` at
   1a41eed (31 rules, 61 verbatim-anchored sites; the round-1 assembly at
   `9ca13585…` is superseded and should be regenerated, not patched). Be aware
   the delta states its sites as *prose*, not as a machine-applicable patch:
   §3 runs about 780 lines and each rule's anchors have to be read out of the
   paragraph describing them. This is the single largest remaining unit and is
   why round 3 stopped here rather than starting it with the budget left.
   Assert each anchor occurs exactly once; report the SHA-256.
2. **Extend the tables.** Point the generator at the candidate
   (`cargo run --bin whitefoot-grammar-tables -- PATH` prints the derived
   file) and install its output. The 21 new fixed spellings are already in
   `fixed_terminal` in `model.rs`, transcribed from the delta's GRAM-5 block
   rather than guessed — note `+checked`, `/checked`, `%checked`, and that
   there is no `&&`, `||`, `<` or `>`. You must still add the matching
   `FixedTerminal` variants and spellings in `compiler/src/syntax/terminal.rs`
   and grow `ALL_TERMINAL_PREDICATES` 76 -> 97 (`TerminalSet`'s u128 still
   fits). `if_stmt`, `value_if`, `infix_tail` and `infix_op` append to
   `ENUM_ORDER`; their decisions append to `DECISION_ORDER` automatically.
   Expect the verifier at 69 productions.
3. **Repoint identity pins** (0029/0030/0031 style):
   `compiler/src/spec.rs` (path, `include_str!`, and the test), the
   `include_bytes!` in `compiler/src/bin/spec.rs`, the 65/75/76 assertions in
   `compiler/src/bin/grammar.rs`, `tests/conformance/runner.py`,
   `docs/roadmap.md`, and `spec/derivation/derivation-ledger.md`.
4. **Report the third diff component as its own number** — rows the v0.23
   grammar adds — rather than netting it against the 70 dropped pairs and 57
   provenance fixes already landed in 7f68c71. That decomposition is a
   standing lead instruction.

**Two traps that survive.** The verifier's `frontend_contract` compares the
candidate against the *compiled-in* active spec, so it only goes green once the
pins move (0030's round-1 blocker; do not re-derive it). And byte-for-byte
agreement on a grammar proves the tables and the EBNF agree, never that the
extended tables are complete — 0038's corpus parse stays the completeness
oracle.

## Hand-back brief (round 2 — superseded in part, retained for history)

Items 3, 4 and 5 are discharged by round 3. Items 1 and 2 belong to 0038.

**Why the hand-back is here rather than at "grammar path + pins green".** The
generator that ruling 2 makes a hard gate is an iterative build — reproducing
530 nodes, 364 select atoms, and 2003 select rows byte-for-byte means many
diff-and-adjust cycles against the committed table, each one cheap on its own
and expensive in aggregate. Starting it with the budget left would have
produced a half-built generator that a successor must first understand and then
probably discard, which is worse than a clean hand-off. The fixed delta had
also not landed, so the extension target was still unknown. Everything below is
work a successor would otherwise have to redo.

**1. The single most valuable finding: `if`/`value_if` is cheap.**
`CheckedEnumType` already has a `Bool` variant and
`check/control/matches.rs:193` already builds a Bool `MatchDescriptor`, so both
new source forms check into the *existing* checked Bool-match statements.
Entailment, lowering, cleanup, drops, and backend need no change, and ENT-3 S1
plus the ENT-5 join come out isomorphic for free. The lead approved this on
2026-08-07: GRAM-1's 1:1 production-to-node law governs the source tree, while
DIAG-2 makes the checked program private compiler state, so two source
constructs sharing one checked form is an implementation choice, not
source-visible desugaring.

**2. Its two approval conditions.** (a) Every diagnostic and DIAG-3 path must
resolve to the `if` source node — **verified, not assumed**: `issue_node`
(`check/support.rs:121`) takes a source `NodeId` and resolves it through
`self.tree.path(node)`; every `TrapSite` is built in the checker the same way
(`control.rs:268`); and `grep -c NodePath` over `compiler/src/lowering` and
`compiler/src/backend` returns **0**, so nothing downstream can override the
location. Pass the `if_stmt` node for the empty-else and flattening
rejections, the condition `expr` node for a condition failure, and the
scrutinee `expr` node for a Bool-scrutinee `match`. (b) Pin it with negative
conformance cases asserting both the cited rule and that the citation lands on
the `if` construct: `gram6-neg-bool-scrutinee-match`, `gram6-neg-empty-else`,
`gram6-neg-unflattened-else-if` (citation at the *nested* `if_stmt`, per the
candidate), plus `give1-neg-empty-delivery-set` at the `let_stmt` node.

**3. Grammar source map — includes a non-obvious trap.** The 65 productions do
not all live in fenced EBNF blocks. 61 do: GRAM-2 has 26, GRAM-3 has 5, GRAM-4
has 18, GRAM-5 has 12. The remaining **four are written inline in prose** with
backticks — `const` in [CONST-1], `cvalue` in [CONST-2], and `effects` and
`effect` in [EFF-1]. That is why the verifier's frontend contract hashes the
prose ranges `[CONST-1]`..`## 5. Ownership` and `[EFF-1]`..`[EFF-2]` rather
than code blocks, and it maps exactly onto the `RuleOwner` enum. A generator
that scrapes only fenced blocks silently produces 61 productions.

**4. Generator plan (ruling 2).** Build in scratch, delete after use, per 0030
and 0031 precedent. Milestones: parse the EBNF above into node trees; reproduce
`PRODUCTIONS`, `PRODUCTION_ROOTS`, `PRODUCTION_OWNERS`; then `GRAMMAR_NODES`
(530) and the children/terminal arenas; then `DECISIONS` (75), `SELECT_ATOMS`
(364), and `SELECT_ROWS` (2003) with provenance. Gate: byte-identical to the
committed `generated.rs` before extending. Then extend to 69 productions and
+21 fixed terminals (76 -> 97 predicates; `TerminalSet`'s u128 still fits).
Treat the `infix_tail` production / `infix` node-kind name mismatch as a
first-class test of the generator — every other pair shares a name. Note that
byte-for-byte reproduction proves agreement on the v0.22 grammar, not
completeness of the extended tables; the migrated corpus parse is the
completeness oracle, which is the control that was missing when 84 follow rows
went missing in 0031's predecessor.

**5. Candidate assembly is a solved problem.** The round-1 assembler applied 57
verbatim anchors, each asserted to occur exactly once, and its output verified
at 128 rules and 20 sections with the frontend contract changing only in the
`[FORM-1]` section. Re-assembling from the fixed delta is the same mechanical
exercise; the round-1 candidate at
`9ca13585dca6668bec9c1b4c219cbb5fc94559bcc29f67d04d52bbb61af5eef4` is
superseded by the fixed delta and should be regenerated, not patched.

**6. Do not redo.** The blocker below is ruled (FN-4's premise is re-keyed to
the operand-derived selected type; TYPE-5's retained-argument class is not
touched, so review finding F3 stays closed). The GRAM-1 shape-kind deviation
below is accepted and is being folded into the delta as a proper site.

## Blocker (round 1 — RULED, retained for history)

The approved delta enumerates 24 rules at 46 anchored sites and its §4 claims
the acceptance-set change is "one canonical respelling plus two deliberate
narrowings". Both statements are incomplete: the sweep covers the 24 rules it
modifies but not the rules whose normative prose *uses* the respelled
operations. Ten normative sites in five unlisted rules still write a deleted
type argument or a respelled spelling, measured on the assembled candidate:

| rule | line | dead spellings |
|---|---|---|
| STOR-2 | 299 | `box_new<T>` |
| STOR-5 | 313 | `box_new<T>` |
| OP-2 | 384 | `iadd.wrap<T>` `isub.wrap<T>` `imul.wrap<T>` |
| OP-2 | 386 | `iadd.trap<T>` `isub.trap<T>` `imul.trap<T>` |
| OP-2 | 388 | `ieq<T>` `ine<T>` `ilt<T>` `ile<T>` `igt<T>` `ige<T>` |
| OP-2 | 392 | `ineg.wrap<T>` `ineg.trap<T>` `ineg.checked<T>` |
| OP-9 | 410 | `buffer_new<T>` |
| FN-4 | 446 | `iadd.sat<D>` |
| FN-4 | 452 | `iadd.sat<T>` |
| FN-4 | 453 | `iadd.sat<T>` |

Plus FORM-3 line 68, whose OPNAME example is `iadd.checked` — a spelling the
batch deletes.

Nine of the eleven are mechanically forced respellings with no design choice,
though OP-2's four make the rule self-contradictory as assembled: the
candidate's own rewritten judgment paragraph opens "Each operation in the
preceding paragraphs carries no written type argument", and those preceding
paragraphs write the type argument.

**FN-4 line 446 is the blocker and is not mechanical.** It reads: "the bound
function's body must contain exactly one statement, `return iadd.sat<D>(p0,
p1);` … and the explicit type argument is D." Under the batch `iadd.sat`
respells to `+sat` and its type argument is deleted, so the one mandated body
shape has no legal spelling and every source conformance-law discharge becomes
impossible. Reproduction, `tests/conformance/cases/fn4-pos-law-discharged.wf`:

```
fn satadd(x: own u64, y: own u64) -> own u64 pure {
  return iadd.sat<u64>(x, y);
}
```

The old bytes reject under OP-1/TYPE-5; the migrated bytes `return x +sat y;`
reject under FN-4's literal mandate. Live impact: 6 corpus sites, 4 conformance
cases (`fn4-pos-law-discharged`, `fn4-pos-law-in-contract`,
`fn4-neg-law-undischarged`, `fn4-neg-law-refuted-signedness`), and the
compiler's own discharge shape.

The repair is a decision, not a transform, because "the explicit type argument
is D" is a *premise of the discharge relation*: either FN-4 re-derives D from
operand agreement under the new OP-2, or `iadd.sat` joins TYPE-5's
retained-argument class — which the review certified "total against the
complete operation table", so moving it reopens that finding. Owner or lead
ruling needed.

## Deviation applied and reported

One 47th anchored site was applied beyond the approved 46, in GRAM-1. Adding an
operator form to GRAM-1's maximal-form list leaves the shape-kind enumeration
("Raw formation gives every token exactly one context-free shape kind: …")
false for operator tokens, and that enumeration is the interface terminal
membership and the compiler's `TokenKind` are written against
(`compiler/src/lexer/token.rs:63`). The candidate's header accounting was
updated to "forty-seven" and GRAM-1 relabelled "four sites" in the same change.
This is a byte the owner did not review and it returns to review with the rest.
