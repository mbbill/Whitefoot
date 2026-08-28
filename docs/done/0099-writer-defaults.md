# Batch 0099 — the defaults an unguided writer met, answered

Branch: `batch/0099-writer-defaults`, from `main` at `b2e2e267`.
Deliverables: the two-name source input and the diagnostics that use it, four
diagnostic payloads, the default-channel report of a denied I/O loop, three
forms in `docs/patterns.md`, and this record.

## Charter

Batch 0098 handed a senior systems programmer the specification,
`docs/patterns.md`, `whitefootc`, and `tests/programs/`, asked for five
ordinary I/O utilities, and recorded what came back as a fourteen-row defect
table. The owner's rule of 2026-08-27 is that for every default an unguided
writer writes badly the project must either change the compiler so the natural
form is the fast and correct form, or emit a warning and teach the form in
`docs/patterns.md`; silence is not allowed.

This batch takes the rows that need no specification change. D1 (a report line
per iteration denying the staged permission), the `const` documentation form
(D11), and standard input (D7) are owner decisions about the language and are
not touched here.

## 1. A source is named by the path the caller typed (D6)

`compiler/src/bin/whitefootc.rs` renamed any absolute source to `input0.wf`
and every diagnostic and every `PAR` line then cited a file that exists nowhere
on disk. One source now carries two names, and they answer two different
questions.

`SourceInput::from_host_path(logical, display, bytes)` supplies both.
`logical_path` stays the closed portable spelling, because it is the bundle's
own key: it orders the bundle and detects a duplicate. `display_path` is the
argument the caller typed, unchanged, and it is what `SourceFile::display_path`
returns, what `TreeView::source_identity` resolves a node to, and what the
permission ledger and every source rejection print.

The task expected to find the portability property still needed where the
backend emits module identity. It is not needed there: `compiler/src/backend/
emitter.rs` writes a fixed `source_filename = "whitefoot"`, so no source name
has ever reached an emitted module. What consumes `logical_path` is the source
bundle itself, and that is where it stayed.

Before and after, the same program compiled by absolute path, with the host
prefix written here as `/abs/path`:

```text
before  PAR stage       input0.wf:26  for  denied  condition 3: …
after   PAR stage       /abs/path/probe_c_helper_denied.wf:26  for  denied  condition 3: …
```

Tests: `a_source_is_shown_by_the_path_the_caller_wrote` in
`compiler/src/bin/whitefootc.rs` pins the two answers apart, and
`a_ledger_names_the_host_path_the_source_was_read_from` in
`compiler/src/driver.rs` compiles a staged fixture by a host path the logical
domain cannot spell and asserts every ledger line names it and none names the
bundle key.

## 2. Four diagnostic payloads

Every source rejection is also now wrapped with the file, line, column, and the
whole offending source line, in both stages that reject after parsing. That is
`compiler/src/driver/rejection.rs`, and it is presentation only: the detail
text is still the stage value's `Debug`, which this batch deliberately did not
redesign.

### GRAM-9 prints the expected spellings and quotes the line

```text
before  whitefootc: Parsing/Source [GRAM-9]: SyntaxIssue { rule: Gram9, coordinate: …
        ByteOffset(187) … expected: ExpectedTerminals { terminals:
        TerminalSet(38424498140022966840644862354), source_end: false } }

after   whitefootc: Parsing/Source [GRAM-9]: SyntaxIssue { rule: Gram9, coordinate: …
        ByteOffset(187) … expected: ["{", ";", ")", ",", "[", "]", ".", "..", "because",
        "+", "+wrap", "+defined", "+checked", "+sat", "-", "-wrap", "-defined", "-checked",
        "-sat", "*", "*wrap", "*defined", "*checked", "*sat", "/", "/defined", "/checked",
        "%", "%defined", "%checked"] } at /abs/path/wc.wf:5:26 in line
        "  let skip = bor(dotted, bnot(addressable));"
```

`FixedTerminal::spelling` became `&'static str` and `spelling_bytes` serves the
byte comparisons; `Terminal::spelling` answers `IDENT`, `TYPEID`, `REGIONID`,
`LABEL`, `OPNAME`, `literal`, `STRING`, or `digits` for the pattern predicates,
which are the names the grammar productions use. The order is DIAG-1's.

### OWN-6 says what the rule means and names both routes

```text
before  kind: InvalidChildReborrow

after   kind: InvalidChildReborrow { mechanical_fix: "a child reborrow's region admits
        exactly one statement, and a value that statement binds dies at the region's end;
        either move the borrow holder into a helper that takes it as `&uniq`, `move`s it
        there, and returns the derived state (P4 linear threading), or bind the reborrowed
        result with `replace`: `let stale = replace target = call(...);`" }
```

Three conditions, three sentences: the statement-scope one above, an
argument-position one, and a holder one. Each is the exact restructuring for
the condition that rejected, in `docs/patterns.md` P4's vocabulary.

### FORM-2 prints the bytes it expected beside the bytes it found

```text
before  whitefootc: CanonicalSource/Source [FORM-2]: CanonicalIssue { location:
        SourceNode(NodePath { … }, SyntaxCoordinate { … ByteOffset(142) … }) }

after   whitefootc: CanonicalSource/Source [FORM-2]: CanonicalIssue { location:
        SourceNode(NodePath { … }, SyntaxCoordinate { … ByteOffset(142) … }),
        expected: " ", found: "  " } at /abs/path/report.wf:3:27 in line
        "  return exit_status(code:  0_u8);"
```

The auditor decided the expected gap and read the found bytes at the point it
stops, so both were already in hand.

### STOR-1 offers the fix the next rule accepts

D9 is the pair that spent two of the writer's six attempts: `[STOR-1]` offered
`replace`, and `replace` produced `[OWN-1] UseAfterMove`, because the
right-hand side had moved the target's root away to compute the value. The
discriminator is exactly a written `move` of the target's root inside the value
expression, and `Checker::set_affine_restructuring` reads it from the syntax
before the target is formed. Nothing is accepted or rejected by it; only which
of two sentences a rejection prints.

```text
before  set totals = walk(running: move totals);
        kind: AffineSetTarget { target_type: "Counts", mechanical_fix:
        "use replace: let old = replace p = e; binds the previous owner" }

after   kind: AffineSetTarget { target_type: "Counts", mechanical_fix:
        "the right-hand side consumes the target root, so replace cannot commit into it:
        bind the result under a new let, and combine it with the old value field by field" }
```

The ordinary sentence still stands where the root survives the value:
`set left = move right;` and `set totals = fresh(lines: 3_u64);` both keep
`"use replace: let old = replace p = e; binds the previous owner"`, and
`let stale = replace totals = fresh(lines: 3_u64);` compiles.
`an_affine_set_offers_the_restructuring_its_right_hand_side_admits` pins all
three, the third by checking that the offered restructuring is accepted, so the
diagnostic and the form it names cannot drift apart.

## 3. A denied I/O loop reports itself without a flag (D8)

`compile_with_io_notices` returns the module beside the ledger lines an
ordinary compile reports, and `whitefootc` prints them to stderr prefixed
`whitefootc: note:`, followed by one line saying the compilation succeeded.
`--par-ledger` is unchanged and remains the full report; the notices are a
marked subset of exactly the same rendered lines, so a notice can never say
something the report does not.

What is marked: every denied `[PAR-3]` staged verdict, and the denied `[PAR-2]`
counted verdict of a loop whose staged verdict was also denied. A granted loop
is silent.

The writer's `p1_tree_wc.wf`, compiled with no flag at all, now prints:

```text
whitefootc: note: PAR stage  p1_tree_wc.wf:126  loop  denied  condition 2: a return leaves
  the loop from the remainder; instead, take every early return, break, or propagate in the
  prologue, before the body's first I/O submission, at return Err<unit, IoError>(error: move problem);
whitefootc: note: PAR stage  p1_tree_wc.wf:205  loop  denied  condition 2: a break naming this
  loop leaves the loop from the remainder; …
whitefootc: note: PAR stage  p1_tree_wc.wf:315  loop  denied  condition 7: the body contains a
  statement that forms a borrow of storage the iteration does not introduce; …
whitefootc: note: PAR loop   p1_tree_wc.wf:335  loop  denied  condition 2: the body contains a
  statement that forms a borrow of storage the iteration does not introduce
whitefootc: note: PAR stage  p1_tree_wc.wf:335  for   denied  condition 1: a statement of the
  body neither executes before the submission on every path nor is reached only through it; …
whitefootc: note: the compilation succeeded; run --par-ledger for the complete permission report
```

`probe_a_staged_permitted.wf` prints nothing.

## 4. Three forms in `docs/patterns.md`

- **P15, amended.** Reserve the file factory inline in the loop body, with the
  two ledger lines from the judge's `probe_c` pair that differ by the factoring
  and nothing else. It also records honestly that when the factory is itself a
  borrow the two rules conflict and no writer discipline satisfies both, and
  that no worked example in `tests/programs/` currently holds the permission,
  so a writer copying `dir_walk.wf` is copying the denied shape. It states the
  new default-channel note.
- **P16, one length fact above the writes.** [ENT-5] kills the root binding of
  `len(P)` but not `P`'s element storage, so a length bound once above the loop
  and above every write still discharges a later `requires`. Two lines, the
  evidence that a whole program in that shape compiles
  (`probe_e_hoisted_length.wf`), and the [FN-8] rejection the fact is holding
  off.
- **P17, subtotal return instead of a threaded accumulator.** The walk returns
  its own subtotal and the caller combines field by field; `replace` is the
  commit only where the value leaves the target's root alive. It ends with the
  two rejections a writer meets without it, [OWN-1] then [STOR-1], in that
  order.

## 5. The corpus re-run

Every program of the 0098 corpus, compiled by the branch compiler and by the
compiler at the branch base, emitting LLVM to a file and comparing bytes:

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
```

Standard error is where they differ: zero bytes for every one of them before,
and now the denied verdicts of the I/O loops in `p1`, `p2`, `p3`, `p4`, `p5`,
`probe_b`, `probe_c_helper_denied`, and `probe_e`. The three that stay silent
are exactly the three whose staged verdict is permitted:
`probe_a_staged_permitted`, `probe_b1_write_after_loop`, and
`probe_c_inline_same_regions` — including `probe_a`, whose counted `[PAR-2]`
verdict is denied and stays in the full report.

All six distinct rejections of the trial were reproduced and now carry a file,
a line, and the offending source line: `FN-7` and `TYPE-6` keep the payloads
that were already good, `OWN-1` keeps its `mechanical_fix`, and `GRAM-9`,
`OWN-6`, and `FORM-2` gained the payloads above.

## Judgment calls

- **A denied counted verdict on a granted staged loop stays in the full
  report.** `probe_a_staged_permitted.wf` holds the staged permission and its
  `[PAR-2]` counted verdict is denied, because the counted rule refuses the
  short factory loan the staged rule exists to admit. Reporting every denial by
  default would tell that writer their granted loop was denied, so the notice
  channel marks a counted denial only when the same loop's staged verdict was
  also denied. The full report is unchanged.
- **The rendering was not redesigned.** Every diagnostic still prints Rust
  `Debug` output. The location and the source line are appended to it rather
  than replacing it, which is the smallest change that turns a byte offset into
  something a writer can act on, and it leaves the machine-parseable form
  intact.
- **`P16` ends with `[FN-8]` rather than a ledger line.** Nothing rejects the
  defensive re-bind — the compiler accepts it — which is exactly why the wrong
  mental model survived a whole program. The concrete text a writer can be
  shown is the rejection they get with no live length fact at all, so that is
  what the entry prints.
- **`P17` states the `replace` case narrowly.** `replace` is right only where
  the value being committed leaves the target's root alive; where the call
  consumed the root, `replace` produces `[OWN-1]` and the fresh `let` is the
  answer. Both cases were compiled before the entry was written.
- **The claim ledger's source name is now the host path.**
  `TreeView::source_identity` feeds both the permission ledger and
  `ClaimSourceIdentity`, and the field was renamed to `display_path` to say so.
  No mandatory record and no emitted module reads it: the only readers outside
  the checker are tests, and `compiler/src/backend/emitter.rs` writes a fixed
  `source_filename`.

## Approval classes

No specification change: `spec/kernel-spec.md` is untouched. No conformance
evidence change: no conformance case, manifest, adapter, runner, or collection
pins any diagnostic text — `grep -rl` over `tests/` for `TerminalSet`,
`CanonicalIssue`, `InvalidChildReborrow`, `mechanical_fix`, `ExpectedTerminals`,
and `logical_path` returns nothing. Rule 4 therefore records nothing for this
merge.

`research/investigations/proof-derived-parallelism/gap-hunt-findings.md` F7,
open since 2026-08-22, is closed in place by item 1.

## What this batch did not do

- D1, D7, and D11 are owner decisions and were not touched.
- D10 and D12 are `warning+patterns` in the 0098 table and this batch shipped
  only the patterns half of each. The warning half — a note on a `let stale =
  replace …` whose binding is never read, and a note on a `len` binding that
  re-establishes a fact that never died — needs a semantic warning channel that
  does not exist. The only default-channel output added here is the permission
  note of item 3, which reports a rendered ledger line the checker already
  produces. A semantic warning channel is a design decision, not a payload, and
  it is the open follow-up this record leaves.
- The `[PAR-3]` lowering that turns a granted verdict into overlapped execution
  is still not landed; P15 continues to say so.
