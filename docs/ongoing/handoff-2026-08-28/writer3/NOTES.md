# Blind writer #3 — NOTES

Export: 917f79ee (batch/0100-writer-defaults-2) -> $SCRATCH/wf-0100-verify/writer/
Target dir: $SCRATCH/wf-0100-verify/writer-target
Reading ONLY spec/kernel-spec.md and docs/patterns.md.

## Step log
- [s0] exported, build started, read AGENTS.md.
- [s1] built whitefootc OK (gate profile, 15.6s). Read spec sections 2-12,15-18 and docs/patterns.md in full.
- [s2] design decisions before first compile:
  * main has no region params (FN-7) -> every borrow needs a local `region` block; deep nesting expected.
  * P15: factory is `own FileFactory` in main, so reserve+open inline in one region block is the admitted form.
  * P15/OWN-6: a `&uniq` reborrow child needs a region block holding exactly ONE statement -> helper factoring
    for read_at through a `&uniq buffer` parameter (read_chunk).
  * P16: hoist `len(...)` above the loop and above every write; do NOT re-bind. Testing that claim.
  * decimal rendering: const DIGIT array + branch guards (no loop induction available, ENT-5).

## Program 1 (work/sizes.wf) — list of names -> byte count or '?'
Five diagnostics, then it compiled and ran correctly. Verbatim, with what I did next:

D1. whitefootc: Parsing/Source [FORM-3]: SyntaxIssue { rule: Form3, coordinate: SyntaxCoordinate { source: SourceId(0), start: ByteOffset(6), end: ByteOffset(11) }, expected: ["IDENT"] } at sizes.wf:1:7 in line "const DIGIT: array<u8, 10> = [48_u8, ...];"
  -> renamed the const `DIGIT` to `digit`. The message names only "IDENT"; I had to know IDENT is
     lowercase from FORM-3. It does point at the exact token. Cost: ~1 min, no doc lookup needed
     because I had just read FORM-3.

D2. whitefootc: CanonicalSource/Source [FORM-2]: CanonicalIssue { location: SourceNode(NodePath { components: [0, 0] }, SyntaxCoordinate { source: SourceId(0), start: ByteOffset(28), end: ByteOffset(29) }), expected: "", found: " " } at sizes.wf:1:29 in line "const digit: array<u8, 10> = [48_u8, ...];"
  -> deleted the space in `= [` giving `=[`. expected/found plus the exact byte span is enough to fix
     mechanically without consulting the attachment sets.

D3. whitefootc: Resolution/Source [TYPE-6]: ... kind: DeclarationCollision { spelling: "permit", conflicts: [DeclarationConflict { domain: LexicalIdentifier, class: Value, origin: Source(... ByteOffset(2284) ...) }] } at sizes.wf:115:35 in line "                              let permit = reserve_file<'g2>(factory: &uniq 'g2 files);"
  -> renamed the inner binding to `permit2`. Both the offending and the conflicting declaration are
     located. Note the outer `permit` was already MOVED; liveness of the *declaration*, not the value,
     is what collides.

D4. whitefootc: Semantics/Source [OWN-1]: SemanticIssue { rule: Own1, ... kind: BareAffineUse { mechanical_fix: "write `move p` for the affine place" } } at sizes.wf:6:10 in line "  return taken;"
  -> wrote `return move taken;`. mechanical_fix is exact.

D5. whitefootc: Semantics/Source [EFF-2]: SemanticIssue { rule: Eff2, ... kind: EffectMismatch { expected_row: "reads(digits), writes(digits)", found_row: "writes(digits)", missing: ["reads(digits)"], extra: [], mechanical_fix: "declare exactly the row the body exhibits: add every missing category and path and remove every extra one; EFF-2 admits no wider and no narrower declaration than the union of the body-syntactic and release contributions" } } at sizes.wf:29:82
  -> pasted expected_row into the header. This is the single best diagnostic I met: it prints the row
     to write. I had guessed `writes(digits)` alone because I could not tell from OP-1 whether `len`
     counts as a read.

Then: compiled. `whitefootc: note: PAR stage ... denied ...` notes printed (5 loops), and
"whitefootc: note: the compilation succeeded; run --par-ledger for the complete permission report".

RESULTS (cd work/sample && ../sizes ../flat.txt):
  a.txt 6 / b.txt 10 / empty.txt 0 / big.bin 5000 / missing.txt ?   exit=0
  no argument -> exit 2; unopenable list file -> exit 3.

Things that worked FIRST TRY and were non-obvious:
  * P16 hoisted length fact: `let capacity = len(content);` above the slurp loop, then
    read_at writes `content` every iteration, and `end <= len(content)` still discharges on
    iteration 2+. Also `let limit = len(deref(scratch));` above a `move scratch` into read_at.
    P16's claim held exactly.
  * P15 inline-owned-factory: `region 'g2 { let permit2 = reserve_file<'g2>(factory: &uniq 'g2 files);
    region 'n2 { match open_file<'g2, 'n2>(...) { ... } } }` was accepted with a two-statement region,
    because `files` is an OWN entry parameter, not a borrow.
  * P15 helper factoring for the &uniq-buffer reborrow: `read_chunk` exists ONLY so that
    `region 'c { match read_chunk<'f, 'c>(scratch: &uniq 'c deref(scratch), ...) { ... } }` is a
    one-statement region. I wrote it that way from P15 without ever seeing the OWN-6 rejection.

## Program 2 (work/largest.wf) — walk a tree, print the ten largest files
Only THREE diagnostics total, all mechanical:

D6. whitefootc: Resolution/Source [TYPE-6]: ... kind: DeclarationCollision { spelling: "src", conflicts: [...] } at largest.wf:179:41 in line "                                    let src = p3 +wrap k;"
  -> renamed the DirectorySource binder `src` to `lister`.

D7. whitefootc: Semantics/Source [EFF-2]: SemanticIssue { rule: Eff2, ... kind: EffectMismatch { expected_row: "reads(out, files), writes(out, files), allocates(heap)", found_row: "reads(cwd, out, files), writes(cwd, out, files), allocates(heap)", missing: [], extra: ["reads(cwd)", "writes(cwd)"], mechanical_fix: "declare exactly the row the body exhibits: ..." } } at largest.wf:54:158
  -> deleted cwd from the row. This answered a question I could not answer from the spec: I `move cwd`
     into a `buffer<Option<DirectoryRead>>` slot, and the release then frames OUT of main's row.
     expected_row made it a copy-paste fix.

D8. (none — it compiled after D7.)

DESIGN WALL I HIT AND ROUTED AROUND BEFORE COMPILING (no diagnostic was involved):
  The natural shape is a recursive `walk(factory: &uniq FileFactory, ...)`. P15 gives the helper
  factoring for the reserve+open pair, but P15 does NOT cover what happens next: with
  `region 'sub { match open_sub<'sub,...>(factory: &uniq 'sub deref(factory), ...) { Ok(value: child) => { ... } } }`
  the `factory` holder is SUSPENDED for the whole match statement (OWN-6), so the recursive call inside
  the Ok arm cannot make a second `&uniq` child of the same factory — OWN-6 says the only admitted
  operation through a suspended holder is a sibling child, and "any overlapping pair containing a
  `uniq` child [is] an error". So a recursive directory walker CANNOT open a child directory and then
  recurse into it with the same factory. I did not get a diagnostic for this; I predicted it from
  OWN-6 and rewrote the program as an explicit worklist in `main`, where `files` is `own` and the
  multi-statement `region 'g { let permit = reserve_file(...); region 'n { match open_...} }` form is
  legal. P15 asserts the recursive-walker form works ("write the helper factoring, and the pipeline is
  the price"); for a walker that must recurse THROUGH the opened value it does not.

  The worklist needed `buffer_vacant<DirectoryRead>` + `replace stack[k] = Some<DirectoryRead>(...)`.
  That worked first try. `None<DirectoryRead>()` / `Some<DirectoryRead>(value: move child)` need the
  written type argument (TYPE-5).

RESULTS: on work/tree it printed the 6 files largest-first; on the compiler source tree its ten rows
are byte-identical to `find . -type f -exec stat -f '%z %N' {} \; | sort -rn | head -10`.

## CORRECTION to the program-2 design note above
I predicted from OWN-6's text that a recursive walker could not recurse through an opened value,
because the `match` on the helper call keeps the factory holder suspended for the whole statement.
That prediction is WRONG. `work/probes/w2walk.wf` writes exactly the three-part idiom the OWN-6
`mechanical_fix` prescribes — helper `open_source_from_factory['f, 'd](factory: &uniq 'f FileFactory,
directory: &'d DirectoryRead) -> result: own Result<DirectorySource, IoError>`, the region's single
statement is the `match` on its call, and the recursive `walk<'w, 'w, 'w>(factory: &uniq 'w
deref(factory), directory: &'w child, batch: &uniq 'w deref(batch))` sits inside the `Ok` arm with a
SECOND `&uniq` child of the same factory — and it COMPILES (exit 0). The holder resumes when the
call returns, not at the region's end. So W2's and W9's claim holds; my iterative worklist rewrite
was unnecessary (it is still a correct program). I record this because it is the one place where my
own reading of the spec, not a diagnostic, cost me time.

## Probes for the eight W items I did not meet naturally (work/probes/)
All ran against the same gate binary. Verbatim payloads:

W1 contract block (probes/w1a.wf:2:21):
  mechanical_fix: "a `call` or `construct` in an atom position does not derive [GRAM-9]: a `contract_block` has no `let`, so bind the inner call with a preceding `define` in this same block and write that binder in the atom position — `define inner = f(x: 0_u64); requires g(y: inner);`"
W1 body (probes/w1b.wf:6:25):
  mechanical_fix: "a `call` or `construct` in an atom position does not derive [GRAM-9]: bind the inner call with its own preceding `let` in this body and write that binder in the atom position — `let inner = f(x: 0_u64); let outer = g(y: inner);`"
  -> the `define` repair parses (probes/w1a2.wf).

W2 (probes/w2.wf:3:44): InvalidChildReborrow { mechanical_fix: "a child reborrow's region admits exactly one statement, ... The whole idiom is three parts: ... The other route, `let stale = replace target = call(...);`, applies only where the call leaves the target's root alive: a call that consumes the target root — one taking `move permit` — rejects OWN-1 instead." }
  -> followed literally in probes/w2walk.wf; compiles, recursion included.

W3: default channel for probes/w5.wf prints PAR loop + PAR stage + PAR place(denied), and each of the
  three lines is byte-identical to a line of --par-ledger (checked with grep -qxF). Every denied place
  row is a notice.

W4 both branches reproduced:
  selected_by_submission=false (sizes.wf:13, sizes.wf:78): "condition 2: a break naming this loop leaves the loop from the remainder; instead, take every early return, break, or propagate in the prologue, before the body's first I/O submission. Where the exit is selected by the may-suspend call's own outcome — a read-to-EOF loop's `ReadEnd` break is — it cannot be taken before the submission and PAR-3 cannot stage that loop as written: the shapes staged today are a fixed-trip bounded loop and a per-file loop over names, and one file's chunk loop stays sequential"
  selected_by_submission=true (probes/w4.wf:5): "condition 2: a propagate leaves the loop from the remainder; instead, PAR-3 cannot stage this loop as written: the submission's own outcome selects this edge, so no rewrite takes it before the submission. The shapes staged today are a fixed-trip bounded loop and a per-file loop over names; one file's chunk loop stays sequential, at let handle = propagate open_file<'c, 'n>(...);"

W5 (probes/w5.wf:2): "condition 3: a may-suspend call retains a borrow past its own submission on storage the body writes and the iteration does not introduce; instead, give each iteration its own resource; or, where the body only publishes to that storage — an output stream is the pointed case — hoist the per-iteration write out of the loop, folding a total in the body and writing it once after the loop; or leave this loop sequential, because storage that carries one position cannot be held by two iterations at once, at &uniq 'w out"
  plus PAR place denied "&uniq 'w out  the body writes it through a retained borrow and its type carries one position, so no iteration can be given its own".
  Removing the write flips the same loop to "PAR stage probes/w5b.wf:2 for permitted staged at open_file<'f, 'n>(...); 3 places classified" and the default channel goes SILENT even though its PAR loop verdict is still denied — exactly P15's withholding claim.

W6 EffectMismatch: met twice for real (D5, D7) and once more in probes; InvalidEffectRow (probes/w6b.wf:1:117): reason "a category appears at most once in one row, and the row is written in the canonical order reads, writes, allocates, traps", mechanical_fix "merge the repeated category's paths into one occurrence — `writes(cwd), writes(out)` is `writes(cwd, out)` — and order the categories reads, writes, allocates, traps".

W7 TypeMismatch { expected: "own u64", found: "own u32" } (probes/w7a.wf:2:18);
   TypeMismatch { expected: "Result with both type arguments written: as a type `Result<u64, IoError>`, and as a variant constructor `Ok<u64, IoError>(value: v)`", found: "Result with no written type-argument list" } (probes/w7b.wf:2:10);
   InvalidBorrowLifetime { region: "'r", binder: "local", mechanical_fix: "a borrow of local storage names a region introduced inside that binding's own scope: write `region 'r { ... }` after the binding and take the borrow inside it. A caller-supplied region parameter is never admitted here, because it outlives the storage." } (probes/w7c.wf:8:26).

W8 (probes/w8.wf:10:11): instantiated_goal: "band(igt(n, 0_u64), ilt(n, 10_u64))" — source terms, caller's binding name.

W9: see the CORRECTION above; also probes/w5b.wf shows the owned-factory inline form is `permitted`.

W10 (probes/w10.wf): "expected: \"\\n  \", found: \"\\n    \" } at probes/w10.wf:3:1 in line \"    let b = a +wrap 2_u64;\"" — the line the writer must edit, matching the record's "after".

W11: not writer-observable. It needs one generic loop monomorphized twice whose staged verdicts
  differ between instances; system operations are nongeneric, so I could not construct a
  discriminating source program. Nothing to confirm or refute from practice.

## Remaining bad defaults I met (verbatim)
1. whitefootc: Parsing/Source [FORM-3]: SyntaxIssue { rule: Form3, coordinate: ..., expected: ["IDENT"] } at sizes.wf:1:7 in line "const digit: array<u8, 10> =[...]"
   — no mechanical_fix, and "IDENT" does not say "lowercase" or "a const name is not a TYPEID".
2. whitefootc: Parsing/Source [GRAM-2]: SyntaxIssue { rule: Gram2, ..., expected: ["}", "requires", "ensures"] } at probes/w2walk.wf:9:3 in line "  define room = len(deref(name));"
   — no mechanical_fix stating GRAM-2's fixed order (all definitions, then all requirements, then all postconditions).
3. whitefootc: Resolution/Source [TYPE-6]: ... kind: DeclarationCollision { spelling: "permit", conflicts: [...] }
   — no mechanical_fix. Met twice. It locates both declarations, but never says the outer binding
   being already MOVED does not end its declaration scope, which is the actual surprise.
4. whitefootc: Semantics/Source [SYS-8]: ... UndischargedSystemRangeObligation { residual: "wide <= len(buffer)", ... }
   — `buffer` is the system operation's declared parameter name, not the caller's place. With two
   buffers in scope (probes/sys8.wf: `header` and `payload`) the residual does not say which one.
   OP-4 gets this right in the same program shape: residual "pick < len(table)".
5. Nine lines of `whitefootc: note: PAR ...` on every successful build of work/sizes.wf, and neither
   `--par` nor `--no-overlap` changes the count (9 / 9 / 9). A correct program that a writer
   recompiles all day has no quiet build.
