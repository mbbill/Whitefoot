# Batch 0100 — the defaults the verification writer met, answered

Branch: `batch/0100-writer-defaults-2`, from `integration/2026-08-28b` at
`44c7a513`. Deliverables: eleven diagnostic and reporting changes W1–W11, the
resolution of the recursive-walker conflict in `docs/patterns.md` P15, the
library tests that pin every new payload by its exact rendered text, and this
record.

## Charter

Batch 0099 answered the defects the 0098 blind writer met. Its verification
re-writer — a fresh writer given only `spec/kernel-spec.md` and
`docs/patterns.md` — wrote a recursive tree line/byte counter and a per-file
FNV-1a checksum utility, both correct, and reported eleven more places where a
default was met badly. The owner's standing rule of 2026-08-27 applies to each:
change the compiler, or emit a warning or diagnostic that teaches the fix; one
of the two is mandatory, teaching is preferred over silent transformation, and
a hidden trick is never allowed.

Every item below is a diagnostic or a report. No acceptance changed: the
corpus re-run of section 12 shows every program that compiled still compiles to
byte-identical LLVM.

Host paths are written here as `/abs/path`; the fixtures were compiled by
relative name, so none appears.

## 1. GRAM-9 names the binding form its grammar position admits (W1)

The rule's own specification text gives the repair — "[GRAM-9] admits a
computed value only through a preceding `let`" — and inside a `contract_block`
that repair is wrong, because the block has no `let_stmt` at all. Its binding
form is `contract_define`, `define IDENT = expr;`. This is the very first
rejection the re-writer met, on their own `a1.wf`, at
`requires ile(end, len(deref(data)));`.

`SyntaxIssue` gained `mechanical_fix: Option<&'static str>`, and its `Debug` is
now written by hand so a rule with no restructuring prints exactly what it
printed before. The position is decided by the grammar: `ProbeContext` and the
parser's production `Frame` carry `in_contract`, set when the frame is
`Production::ContractBlock` or when any enclosing frame is, and read from the
open frames at the point of failure. No text of the offending line is
consulted.

```text
before  [GRAM-9]: SyntaxIssue { rule: Gram9, coordinate: … ByteOffset(126) …,
        expected: ["{", ";", ")", ",", "[", "]", ".", "..", "because", "+", …] }
        at /abs/path/a1.wf:2:21 in line "  requires ile(end, len(deref(data)));"

after   … expected: [ … ], mechanical_fix: "a `call` or `construct` in an atom position
        does not derive [GRAM-9]: a `contract_block` has no `let`, so bind the inner call
        with a preceding `define` in this same block and write that binder in the atom
        position — `define inner = f(x: 0_u64); requires g(y: inner);`" }
        at /abs/path/a1.wf:2:21 in line "  requires ile(end, len(deref(data)));"
```

In a body the same rejection reads:

```text
after   … mechanical_fix: "a `call` or `construct` in an atom position does not derive
        [GRAM-9]: bind the inner call with its own preceding `let` in this body and write
        that binder in the atom position — `let inner = f(x: 0_u64); let outer =
        g(y: inner);`" } at /abs/path/body.wf:2:25 in line
        "  let a = double(value: double(value: value));"
```

Tests: `a_forbidden_atom_names_the_binding_form_its_grammar_position_admits`
pins both sentences, and `the_contract_block_repair_gram9_names_is_accepted`
compiles the `define` form, so the rejection and the repair cannot drift apart.

## 2. OWN-6 states the whole idiom, not a third of it (W2)

Batch 0099's `mechanical_fix` offered two routes and neither reached a working
recursive walker. The `replace` route cannot commit where the right-hand side
consumed the target's root, which is exactly what `open_file(permit: move
permit, …)` does; the helper route is one part of a three-part idiom.
`tests/programs/dir_walk.wf:227` pairs the helper with two more parts, and the
sentence now names all three and states the `replace` route's exact limit.

```text
before  mechanical_fix: "a child reborrow's region admits exactly one statement, and a
        value that statement binds dies at the region's end; either move the borrow holder
        into a helper that takes it as `&uniq`, `move`s it there, and returns the derived
        state (P4 linear threading), or bind the reborrowed result with `replace`:
        `let stale = replace target = call(...);`"

after   mechanical_fix: "a child reborrow's region admits exactly one statement, and a
        value that statement binds dies at the region's end, so `region 'r { let permit =
        reserve_file<'r>(factory: &uniq 'r holder); match open_...(permit: move permit,
        ...) { ... } }` is two statements and cannot be repaired by shortening the region.
        The whole idiom is three parts: move the reserve and the open into one helper that
        takes the holder as `&uniq 'f` and returns the opened value
        (`fn open_source_from_factory['f, 'd](factory: &uniq 'f FileFactory, directory:
        &'d DirectoryRead) -> result: own Result<DirectorySource, IoError>`); make the
        single statement of the region the `match` on that helper's call; and write every
        statement that uses the opened value inside that `match` arm, because the opened
        value dies with the region (P4 linear threading, P15 recursive walker). The other
        route, `let stale = replace target = call(...);`, applies only where the call
        leaves the target's root alive: a call that consumes the target root — one taking
        `move permit` — rejects OWN-1 instead."
```

The offered idiom was compiled before it was written.
`a_child_reborrow_rejection_states_the_scope_rule_and_the_whole_idiom` pins all
three parts and the limit, and `semantic::tests::borrows` keeps its deliberate
second copy of the text, so a change has to be made twice on purpose.

## 3. A denied loop reports every condition that denied it (W3)

`treelines.wf:99` printed one line — the break denial — and `--par-ledger`
showed two more denied places on the same loop. A writer who repairs the first
cause meets the second one it was hiding.

Every denied row of a denied staged loop's disposition table is now a notice.
The notice channel remains a marked subset of the same rendered lines, so it
can still never say something the report does not, and a row that is not denied
stays inside the full report.

```text
before  note: PAR stage  treelines.wf:99  loop  denied  condition 2: a break naming this
        loop leaves the loop from the remainder; …
        note: the compilation succeeded; run --par-ledger for the complete permission report

after   note: PAR stage  treelines.wf:99  loop  denied  condition 2: … (see item 4)
        note: PAR place  treelines.wf:99  denied  &uniq 'd data  the body writes it and a
              may-suspend call retains a borrow of it past its own submission
        note: PAR place  treelines.wf:99  denied  offset  the body reaches it on both sides
              of the cut, so no single segment serializes it
        note: the compilation succeeded; run --par-ledger for the complete permission report
```

## 4. The condition-2 remedy is honest about what it cannot repair (W4)

The remedy "take every early return, break, or propagate in the prologue,
before the body's first I/O submission" is unsatisfiable for a read-to-EOF loop
whose break is selected by the read's own `ReadEnd` outcome. No language change
was invented. The judgment already distinguishes one case — an edge the cut
statement itself takes on the submission's outcome, which is `let x = propagate
open_file(…)` — and `StagedDenial::ExitInRemainder` now carries that fact as
`selected_by_submission`. The two remedies are:

```text
selected_by_submission = true
        "PAR-3 cannot stage this loop as written: the submission's own outcome selects this
        edge, so no rewrite takes it before the submission. The shapes staged today are a
        fixed-trip bounded loop and a per-file loop over names; one file's chunk loop stays
        sequential"

selected_by_submission = false
        "take every early return, break, or propagate in the prologue, before the body's
        first I/O submission. Where the exit is selected by the may-suspend call's own
        outcome — a read-to-EOF loop's `ReadEnd` break is — it cannot be taken before the
        submission and PAR-3 cannot stage that loop as written: the shapes staged today are
        a fixed-trip bounded loop and a per-file loop over names, and one file's chunk loop
        stays sequential"
```

The second sentence is carried in both branches because the judgment does not
compute whether a break *statement* of the remainder is data-dependent on the
cut call's outcome; see the judgment calls. It never misleads: it states the
hoist where the hoist exists and states the limit where it does not.

## 5. An output stream is offered the remedy that works (W5)

For `&uniq 'p out` the remedies were "give each iteration its own resource, or
leave this loop sequential", and neither is available for stdout. The remedy
that works is to take the write out of the loop; the re-writer's own
`probe_nooutput.wf` is the same program with it hoisted and it flips to
permitted.

```text
before  condition 3: … instead, give each iteration its own resource, or leave this loop
        sequential: storage that carries one position cannot be held by two iterations at
        once, at &uniq 'p out

after   condition 3: … instead, give each iteration its own resource; or, where the body
        only publishes to that storage — an output stream is the pointed case — hoist the
        per-iteration write out of the loop, folding a total in the body and writing it
        once after the loop; or leave this loop sequential, because storage that carries
        one position cannot be held by two iterations at once, at &uniq 'p out
```

`a_one_position_resource_is_offered_the_hoist_that_works` pins the sentence and
then compiles the hoisted form, asserting it is granted. This is the writer-side
answer to the D1 language question until the owner decides D1.

## 6. EFF-2 publishes both rows and the difference; EFF-1 names its condition (W6)

`EffectMismatch` carried no payload through four blind rounds: a writer was told
two rows differ and left to derive both sides. Both are in hand at the judgment,
and so is the exact difference.

```text
before  kind: EffectMismatch

after   kind: EffectMismatch { expected_row: "pure", found_row: "reads(data)", missing: [],
        extra: ["reads(data)"], mechanical_fix: "declare exactly the row the body exhibits:
        add every missing category and path and remove every extra one; EFF-2 admits no
        wider and no narrower declaration than the union of the body-syntactic and release
        contributions" }
```

`expected_row` is the row the body exhibits, which is what the declaration must
say; `found_row` is what the declaration writes. Both render in EFF-1 canonical
spelling from the parameters' own names and the struct field names, so the
writer reads their own effect paths back.

`InvalidEffectRow` now names which of EFF-1's six conditions rejected and the
repair for that condition. The once-per-category rule the re-writer met is one
of them:

```text
before  kind: InvalidEffectRow

after   kind: InvalidEffectRow { reason: "a category appears at most once in one row, and
        the row is written in the canonical order reads, writes, allocates, traps",
        mechanical_fix: "merge the repeated category's paths into one occurrence —
        `writes(cwd), writes(out)` is `writes(cwd, out)` — and order the categories reads,
        writes, allocates, traps" }
```

## 7. TYPE-5 and OWN-10 publish what they compared (W7)

`TypeMismatch` gained `expected` and `found`, filled at all sixty-nine sites
that cite it (TYPE-5, and the TYPE-2, FN-2, FN-3, SYS-2, GIVE-1, and FORM-5
sites that share the kind). Each side is the type, mode, or written form that
position requires or found, in source spelling; `checked_value_name` renders a
mode and type together and spells the region the way the source does.

```text
before  kind: TypeMismatch                       at /abs/path/t5.wf:4:18 in line
                                                 "  let c = ile(a, b);"
after   kind: TypeMismatch { expected: "own u64", found: "own u32" }
```

Where the form is a generic one written with no type arguments — which is what
a variant constructor is — both spellings that carry them are named, because a
writer meeting this at `Ok(value: v)` sees a constructor name and no type
anywhere:

```text
before  kind: TypeMismatch                       at /abs/path/type5c.wf:2:10 in line
                                                 "  return Ok(value: value);"
after   kind: TypeMismatch { expected: "Result with both type arguments written: as a type
        `Result<u64, IoError>`, and as a variant constructor `Ok<u64, IoError>(value: v)`",
        found: "Result with no written type-argument list" }
```

`InvalidBorrowLifetime` names the region, the binder, and where a region it
admits must be introduced:

```text
before  kind: InvalidBorrowLifetime               at /abs/path/own10.wf:7:24 in line
                                                  "  return sum<'r>(data: &'r local);"
after   kind: InvalidBorrowLifetime { region: "'r", binder: "local", mechanical_fix: "a
        borrow of local storage names a region introduced inside that binding's own scope:
        write `region 'r { ... }` after the binding and take the borrow inside it. A
        caller-supplied region parameter is never admitted here, because it outlives the
        storage." }
```

The repair was compiled: the same program with `region 'l { return sum<'l>(data:
&'l local); }` is accepted.

## 8. FN-8 renders its goal in source terms (W8)

FN-8 published its goal as a structural dump while [OP-4] and [SYS-8] published
theirs as source terms from the same renderers.

```text
before  instantiated_goal: "Boolean(And)<types=[], consts=[]>(Integer { operation: Greater,
        operand_type: Integer(U64) }<types=[], consts=[]>(Place { root: BindingId(0),
        projections: [], ty: Integer(U64) }, Literal(Integer { ty: U64, bits: 0 })):Bool,
        Integer { operation: Less, operand_type: Integer(U64) }<types=[], consts=[]>(Place
        { root: BindingId(0), projections: [], ty: Integer(U64) }, Literal(Integer { ty:
        U64, bits: 10 })):Bool):Bool"

after   instantiated_goal: "band(igt(value, 0_u64), ilt(value, 10_u64))"
```

The rendering is `EntailmentFlow::render_concrete_goal`, beside the OP-4
residual renderers, because that is where the caller's binding names are in
scope; `CallGoalOutcome` and `CallGoalCounterfactual` carry the rendered string
the way they already carried `residual`. The operation spellings come from
`CheckedIntegerOperation::spelling`, `CheckedFloatOperation::spelling`, and
`CheckedBooleanOperation::spelling`, which are the exhaustive maps that used to
live in `semantic::tests::operation_table`; that test now calls them, so every
spelling a diagnostic prints is one the test has compared, cell for cell, with
the specification's own `wf-ops` table. `goal::render_goal` was deleted:
`render_concrete_goal` and `render_stable_schema_goal` are its two replacements
and nothing else called it. The generic-schema path is unchanged and still
publishes no scratch identity.

## 9. P15 states the walker resolution (W9)

`docs/patterns.md` P15 stated the D2 conflict and gave no resolution, and its
instruction "write the reserve and the open in the loop body itself" walks a
recursive-walker writer straight into the wall it names. The resolution is that
in a walker only one of the two forms is a program at all, and the three
measured outcomes are now in the entry:

```text
owned factory, inline    PAR stage  probes/inline_owned.wf:3   for  permitted  staged at
                         open_file<'f, 'n>(…); 4 places classified
borrowed factory, inline [OWN-6] InvalidChildReborrow — the program does not compile
borrowed factory, helper PAR stage  probes/walk_helper.wf:14   for  denied     condition 3:
                         a may-suspend call retains a borrow past its own submission … at
                         &uniq 'open deref(factory)
```

So: an owned entry factory takes the inline form and is granted; a `&uniq`
factory takes the helper form and pays the pipeline, because there is no third
form. The entry says the helper is only the first of three parts and names the
other two.

Line ~412's overclaim is corrected in the same entry. It said an ordinary
compile "prints the denied verdict of every I/O loop", which is not true and was
deliberately not true: a denied counted [PAR-2] verdict on a loop whose staged
verdict is permitted is withheld from the default channel, because reporting it
would tell that writer their granted loop was denied. The paragraph now says
what the default channel prints, what it withholds and why, and that every
notice is a line of the full report byte for byte. The "no worked example holds
this permission" paragraph is corrected too: `dir_walk.wf`'s walker loops carry
the helper factoring because nothing else compiles, and their denial is the
price of the only admitted form — what a writer must not copy from them is the
hoisted scratch buffer.

## 10. FORM-2 quotes the line its offending bytes are in (W10)

FORM-2's coordinate is the trivia gap between two terminals. A gap that carries
a line break begins at the end of the line *before* the one the writer must
edit, so anchoring the reader at its first byte quoted the enclosing item's
header while the byte offset in the payload was right.

```text
before  … expected: "\n  ", found: "\n    " } at /abs/path/form2b.wf:2:29 in line
        "  let a = value +wrap 1_u64;"
after   … expected: "\n  ", found: "\n    " } at /abs/path/form2b.wf:3:1 in line
        "    let b = a +wrap 2_u64;"
```

`Located::in_gap` is the narrow constructor for this one location class, and it
anchors at the first byte of the gap that lies on the line the gap ends in. A
gap inside one line is unchanged — `at /abs/path/form2.wf:3:12 in line "  let b
= a  +wrap 2_u64;"` before and after — and no other diagnostic's anchoring
moved, because a semantic rejection's coordinate is a node extent whose start is
the right place to stand.

## 11. The ledger's dedup key includes the notice flag (W11)

`render_ledger` deduplicated on ordinal and text, and the notice flag was
outside the identity that dedup asserts. For a `stage` or `place` line the flag
is a function of the text, so nothing could differ; for a `loop` line it is not.
Its flag depends on the *staged* verdict of the same function instance, so one
source loop of a generic monomorphized twice can render one identical `loop`
line that is a notice in one instance and not in the other, and the surviving
line would then depend on the order of the permission table.

The flag is now merged rather than dropped: the surviving line carries it when
any instance raised it. The six-tuple became `struct Entry` and the sort and
dedup became `fn collapse`, which three unit tests in `permission_ledger.rs`
pin directly — the merge, the two-distinct-lines case, and the one-row-per-
position rule the disposition table depends on.

## 12. The corpus re-run

Every program of the 0098 corpus and of the 0099 verification, compiled by the
branch compiler and by the compiler at the branch base, emitting LLVM to a file
and comparing bytes:

```text
p1_tree_wc                       identical
p2_tree_grep                     identical
p3_checksum                      identical
p4_copy_count                    identical
p5_two_outputs                   identical
probe_a_staged_permitted         identical
probe_b_staged_denied            identical
probe_b1_write_after_loop        identical
probe_c_helper_denied            identical
probe_c_inline_same_regions      identical
probe_d_reborrow_two_statements  rejected by both, [OWN-6]
probe_e_hoisted_length           identical
a1                               rejected by both, [GRAM-9]
a2                               rejected by both, [OWN-6]
b1                               identical
b2                               identical
checksum                         identical
probe_hoisted                    identical
probe_inline                     identical
probe_nooutput                   identical
treelines                        identical
```

Standard error is where they differ, and only there. Exactly four programs are
silent before and after — `probe_a_staged_permitted`,
`probe_b1_write_after_loop`, `probe_c_inline_same_regions`, and `probe_inline`,
which are the four whose staged verdicts are permitted. The other seventeen
print more than they did: the denied rows of item 3, and the longer remedies of
items 2, 4, and 5.

The two rejections in the corpus are the two the re-writer met. `a1.wf` is the
GRAM-9 contract-block case of item 1 and now carries the `define` fix; `a2.wf`
and `probe_d` are the OWN-6 case of item 2 and now carry the whole idiom.

## Judgment calls

- **The condition-2 remedy states the limit in both branches.** The judgment
  knows when the exit edge is the cut statement's own, and that case gets the
  short, unambiguous sentence. It does not know whether a `break` *statement* of
  the remainder is guarded by a binding the cut's own `match` arms wrote, which
  is the read-to-EOF shape — answering that is a dataflow question this batch
  did not add. Rather than invent an analysis whose correctness could not be
  established here, the general remedy carries the limit as a second sentence.
  It over-explains for a break that genuinely can be hoisted; it never tells a
  writer to do something they cannot do.
- **Sixty-seven pre-existing test assertions kept what they asserted rather
  than gaining the new payload text.** `TypeMismatch`, `EffectMismatch`,
  `InvalidEffectRow`, and `InvalidBorrowLifetime` were unit variants, so
  `assert_rule(source, rule, kind)` asserted exactly two things: which rule
  rejected and which kind it cited. Those call sites now use `assert_rule_kind`,
  which asserts the same two things and nothing less. The payloads themselves
  are pinned text-for-text by the nine new `driver::tests` of this batch, which
  is where a wording change has to be made deliberately. Pinning all
  sixty-seven would be stronger evidence and is the obvious follow-up; it needs
  the actual payload of each fixture, which is a mechanical but large pass.
- **`TypeMismatch` carries two fields and no third "note".** Sixty-nine sites
  would each print `note: None` for the sake of the handful that want one, so
  where the disagreement is about a written form rather than two types, each
  side states that form: `"Result with no written type-argument list"` is what
  is there, and the expected side names both spellings that carry the arguments.
  The doc comment says the fields are "the exact type, mode, or written form".
- **`InvalidBorrowLifetime`'s `mechanical_fix` is a `String`, not a
  `&'static str`.** Four of its six sites compare two concrete regions and the
  repair is only actionable if it names them, so the sentence is built per site.
  The one condition with a fixed repair — a borrow of local storage — keeps a
  constant and shares it across its three sites.
- **The one-position remedy names the hoist for a stream and not for a
  cursor.** The same condition denies an output stream and an enumeration
  cursor, and only the stream can have its use hoisted. The sentence therefore
  offers the hoist under an explicit condition — "where the body only publishes
  to that storage" — rather than as advice for the arm as a whole.
- **`goal::render_goal` was deleted rather than kept beside its replacement.**
  Both of its callers moved to `rendered_goal`, and the generic-schema path has
  its own renderer. A superseded renderer left in place is the parallel version
  the repository rules forbid.
- **The operation-spelling maps moved into `semantic::model` and the
  specification lock now calls them.** The maps existed only in the test that
  compares them with the `wf-ops` table. Copying them into the renderer would
  have created a second spelling table nothing checks; moving them makes the
  diagnostic's spellings the locked ones.

## Two language questions for the owner

These are recorded, not decided.

- **A single file's chunk loop (W4).** A read-to-EOF loop over one file cannot
  be staged as written: its only exit is selected by the read's own `ReadEnd`
  outcome, and moving the check to the top of the body does not help — the
  `stop` flag and the running `offset` are then storage the body reaches on both
  sides of the cut, which condition 5 denies for exactly the reason the flag
  exists. Pipelining one file's sequential chunk reads therefore needs something
  the current judgment has no form for: a way to carry the offset across the cut
  as a value the prologue may compute for iteration i+1 before iteration i's
  read completes, and a way for a short read to cancel the operations already in
  flight. Both are language questions. `treelines.wf:99`, `checksum.wf:107`,
  `dir_walk.wf:262`, and `wfgrep.wf:598` are all this shape.
- **Per-iteration output (W5, the standing D1).** The staged judgment denies a
  loop that writes to one output stream per iteration, and the writer-side
  answer above — fold a total and publish after the loop — changes the program's
  output ordering guarantees from "interleaved as produced" to "all at the end".
  Whether the language should instead admit a per-iteration publish to a stream
  whose order the runtime restores is D1, and this batch only makes the working
  rewrite discoverable.

## Gate results

Local canonical `make check` on macOS: see the run recorded with this branch's
final commit. Compiler library tests: 1417 passing, up from 1401, with the
fourteen new tests of this batch and the two extended staged-permission tests.

CI: `gate` and `io-hosts` on the branch; the URLs are in the report that
accompanies this record. No pre-existing documented red exists on this base.

## Approval classes

No specification change: `spec/kernel-spec.md` is untouched. No conformance
evidence change: no conformance case, manifest, adapter, runner, or collection
pins any of the diagnostic text this batch changed — `git grep -l` over
`tests/conformance/` for `TypeMismatch`, `EffectMismatch`,
`InvalidEffectRow`, `InvalidBorrowLifetime`, `InvalidChildReborrow`,
`instantiated_goal`, and `mechanical_fix` returns nothing. Rule 4 therefore
records nothing for this merge.

`docs/done/0098-blind-writer.md` D2, open since 2026-08-28, is closed by item 9.

## What this batch did not do

- D1, D7, and D11 remain owner decisions and were not touched. Item 5 is the
  writer-side answer to D1's symptom, not a decision about D1.
- The semantic warning channel that 0099 left open is still not built. The only
  default-channel output added here is more of the permission notice of 0099
  item 3, which is a rendered ledger line the checker already produces.
- The [PAR-3] lowering that turns a granted verdict into overlapped execution is
  still not landed; P15 continues to say so.
