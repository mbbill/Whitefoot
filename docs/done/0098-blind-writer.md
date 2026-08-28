# Batch 0098 — what an unguided writer actually writes

Branch: `batch/0098-blind-writer`, from `main` at `b2e2e267`.
Deliverables: the blind-writer corpus at
`research/experiments/blind-writer/2026-08-28/` — seven writer programs, three
judge probes, the writer's report, and the `--par-ledger` output for all ten;
this record and its defect table.

This batch changed no compiler code and no pattern. It is a measurement of the
language as it stands.

## Charter

The owner's rule of 2026-08-27: for every default an unguided writer writes
badly, the project must either change the compiler so the natural form is the
fast and correct form, or emit a warning and teach the form in
`docs/patterns.md`. One of the two is mandatory; silence is not allowed.

That rule needs a subject. So: hand a senior systems programmer with no prior
Whitefoot exposure the spec, `docs/patterns.md`, the `whitefootc` gate binary,
and `tests/programs/` — and nothing else — and ask for five ordinary I/O
utilities. Then judge what came back.

The writer produced `p1_tree_wc.wf` (line and byte counts over a tree),
`p2_tree_grep.wf` (literal grep over a tree), `p3_checksum.wf` (FNV-1a over
files named on argv), `p4_copy_count.wf` (copy a file to stdout, count
newlines), and `p5_two_outputs.wf` (interleaved stdout report and stderr log),
plus two `--par-ledger` probes. All five compile and all five are correct
against their Unix references; the judge re-ran each one:

```text
p1  0 0 c/empty.txt / 0 19 a/b/f3.txt / 1 2 a/f2.txt / 2 8 f1.txt / 3 29 total
    identical to per-file `wc -l` / `wc -c`
p2  sub/hit.txt                     identical to `grep -rl NEEDLE`
p3  f1.txt 8907130258221289207 8    digest stable, byte counts correct
p4  lines =2, and `cmp` reports the copy identical to the source
p5  report #N on stdout, log #N on stderr, correctly separated
```

Zero `claim` statements in 1,694 lines. Every subscript, every `%` and `/`,
every system range call was discharged by ordinary `if` branches and `len()`
rebinding. The proof obligations — the part of this language everyone expects
to be the wall — were not the wall.

## What was measured

The compiler was built at the branch base and every program compiled twice:
the way it ships, and with `--no-overlap`, which emits the module a compiler
with no overlap lowering at all emits. If the two modules are identical, the
program got none of the completion model's headline capability.

```text
p1_tree_wc                default == --no-overlap   ZERO overlap emitted
p2_tree_grep              default == --no-overlap   ZERO overlap emitted
p3_checksum               default == --no-overlap   ZERO overlap emitted
p4_copy_count             default == --no-overlap   ZERO overlap emitted
p5_two_outputs            default == --no-overlap   ZERO overlap emitted
probe_a_staged_permitted  default == --no-overlap   ZERO overlap emitted

tests/programs/dir_walk.wf      ZERO overlap        (the writer's teacher)
tests/programs/wfgrep.wf        ZERO overlap        (the flagship program)
tests/programs/byte_string.wf   ZERO overlap
many_files_narrow.wf            ZERO overlap
many_files_loop.wf              ZERO overlap
many_files_wide8.wf             1534 differing IR lines — overlap emitted
```

Every program in this repository that a writer can read as an example of doing
I/O compiles to the same code a compiler with no overlap lowering would emit.
The only programs that overlap are the hand-widened benchmark programs, which
exist to be benchmarked.

What that costs, against the best hand-written shape over identical work, on
this host today — medians and minima of nine interleaved rounds after two
warm-ups, 8,192 files, warm page cache:

```text
many_files_narrow    median 1.760 s   min 1.470     the natural loop
many_files_loop      median 1.860 s   min 1.680     the P15 loop
many_files_wide8     median 0.990 s   min 0.910     hand-widened
```

1.78x on medians, 1.62x on minima. This host was not quiet — a later
fifteen-round pass on the same binaries produced a 60-second outlier — so the
committed quiet-host medians in `research/investigations/io-model/RESULTS.md`
are the better numbers and they are worse: `C.narrow` against `C.wide8` is
1183.83 against 545.50 ms on macOS (2.17x) and 346.65 against 119.47 ms on
Linux (2.90x), and on the read-heavy workload 3058.12 against 1463.43 ms
(2.09x).

So the headline finding is not that the writer wrote something slow. It is
that the writer wrote the only shape the corpus taught, that shape overlaps
nothing, and nothing told them.

## Defect table

| # | Program | Shape | Consequence | Disposition |
|---|---|---|---|---|
| D1 | all five, probe B | one `write_once`/`emit_all` per iteration | staged permission denied on the output loan; zero overlap in every program | compiler-change |
| D2 | p1, p2, probe C | `reserve_file` + `open_*` behind one helper | flips `PAR stage permitted` to `denied … at &uniq 'f files`; [OWN-6] forces the helper | compiler-change |
| D3 | p1, probe D | `&uniq 'source deref(factory)` in a two-statement region | `InvalidChildReborrow`, byte offset only; the one wall of the session | compiler-change |
| D4 | p1 | `let skip = bor(dotted, bnot(addressable));` | `TerminalSet(38424498140022966840644862354)`; one of six attempts, no fix taught | compiler-change |
| D5 | p1 | one double space in an effect row | `CanonicalIssue` with no `kind` and no expected bytes; one compile round | compiler-change |
| D6 | all | any absolute source path | every diagnostic and every ledger line names `input0.wf` | compiler-change |
| D7 | p4 | `command.stdin as inp` | rejected; the whole Unix filter genre is unwritable | compiler-change |
| D8 | all | any I/O loop | the staged denial is silent without `--par-ledger`, and every example teaches the denied shape | warning+patterns |
| D9 | p1 | `set totals = walk(…, running: move totals, …);` | STOR-1's `mechanical_fix` leads to a form OWN-1 rejects; two of six attempts | compiler-change |
| D10 | p1 | `struct Counts { lines: u64; bytes: u64; status: u8; }` | affine; `BareAffineUse` twice, then the `replace` ceremony | warning+patterns |
| D11 | all five | `const log_label: array<u8, 5> =[108_u8, 111_u8, 103_u8, 32_u8, 35_u8];` | 12 such constants; a wrong codepoint is silent and undocumentable | compiler-change |
| D12 | all five | `let room = len(b);` after every callee write | 34 of 41 `len` bindings written defensively; not required | warning+patterns |
| D13 | all five | 120 `region` blocks in 1,694 lines | 7.1% density against the expert's 5.2% | no-action |
| D14 | probe A | the granted verdict lowers to nothing today | P15's form costs a per-iteration allocation and buys no speed | no-action |

## D1 — a report line per iteration denies the staged permission

Shape, `programs/p5_two_outputs.wf:131`:

```whitefoot
    let report_room = len(report);
    let report_fits = ile(report_end, report_room);
    if report_fits {
      region 'report_emit {
        match emit_all<'report_emit, 'report_emit>(output: &uniq 'report_emit out, source: &'report_emit report, length: report_end) {
```

Consequence. The writer's own probe pair isolates it: `probe_a` and `probe_b`
differ by one `write_once` inside the loop, and only that.

```text
probe_a  PAR stage  for  permitted  staged at open_file<'f, 'n>(…); 5 places classified
probe_b  PAR stage  for  denied     condition 3: a may-suspend call retains a borrow past its own
         submission on storage the body writes and the iteration does not introduce … at &uniq 'say out
probe_b  PAR place  denied  &uniq 'say out  the body writes it through a retained borrow and its
         type carries one position, so no iteration can be given its own
```

The same denial appears in every real program, naming the writer's own output
borrow: `&uniq 'recurse deref(output)` in p1 and p2, `&uniq 'flush
deref(output)` in p4, `&uniq 'attempt deref(output)` in the shared `emit_all`
of all five.

Disposition: compiler-change. Rationale: this is the shape of `wc`, `grep`,
`ls`, `find`, `md5sum`, and of every program in this trial — read many things,
report each one. No writer discipline reaches it, because `Output` carries one
position and a `may-suspend` `write_once` retains its loan past the
submission. The two exits are a per-iteration output resource that merges in
index order, or the buffered-output type `[SYS-12]` already names as future
work. Telling the writer to collect and print after the loop is not a
disposition: it costs the streaming property, and it is what the language would
be asking every utility to give up.

## D2 — the helper [OWN-6] forces is the helper [PAR-3] denies

Shape, `programs/p1_tree_wc.wf:242`:

```whitefoot
fn open_source_from['f, 'd](factory: &uniq 'f FileFactory, directory: &'d DirectoryRead) -> result: own Result<DirectorySource, IoError> reads(factory, directory), writes(factory) {
  doc "Reserves one permit and opens one enumeration over the named directory.";
  let permit = reserve_file<'f>(factory: move factory);
  return open_directory_source<'d>(permit: move permit, directory: directory);
}
```

Consequence, isolated by `probes/probe_c_helper_denied.wf`, which is
`probe_a_staged_permitted.wf` with exactly this factoring and nothing else:

```text
probe_a  PAR stage  for  permitted  staged at open_file<'f, 'n>(…); 5 places classified
         PAR place  serialized-P  &uniq 'f files   … prologues run in index order without overlapping

probe_c  PAR stage  for  denied     condition 3: a may-suspend call retains a borrow past its own
         submission … at &uniq 'f files
         PAR place  denied  &uniq 'f files  the body writes it through a retained borrow and its
         type carries one position, so no iteration can be given its own
```

Every other `PAR place` classification is identical between the two.

The writer did not choose the helper. `[OWN-6]` admits a child reborrow only
when its region "is a locally-introduced region whose block does not extend
beyond the enclosing statement", so `region 'source { let permit = …; match
open_… }` is rejected, and a one-statement region cannot carry the `permit` it
binds out to the open. Whenever the factory is itself a borrow — which it is in
any recursive walker — the only routes are a helper or a `let stale = replace`
ceremony. `tests/programs/dir_walk.wf` takes the helper route at three sites,
which is where the writer learned it, and p1 and p2 copy it.

Disposition: compiler-change. Rationale: two rules of the language push in
opposite directions and no writer discipline satisfies both. Either a
statement-scoped child reborrow must be able to bind a value that outlives its
region, or the staged judgment must see through a callee whose only retained
loan is the reserve it immediately consumes. A pattern entry cannot fix this,
because the form the pattern would teach is the one `[OWN-6]` rejects.

## D3 — the [OWN-6] rejection tells the writer nothing

Shape, `probes/probe_d_reborrow_two_statements.wf`, the smallest reproduction:

```whitefoot
  region 'source {
    let permit = reserve_file<'source>(factory: &uniq 'source deref(factory));
    match open_file<'source, 'c>(permit: move permit, root: root, name: name, start: 0_u64, end: 1_u64) {
```

Consequence, verbatim:

```text
whitefootc: Semantics/Source [OWN-6]: SemanticIssue { rule: Own6, location: SourceNode(NodePath
{ components: [0, 0, 5, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0] }, SyntaxCoordinate { source: SourceId(0),
start: ByteOffset(306), end: ByteOffset(334) }), kind: InvalidChildReborrow }
```

No message, no source line, no fix. The writer records this as the only rule
they could not apply from the spec text and the only one that would have
stopped them without the example corpus; they recovered the form by reading
`dir_walk.wf`.

Disposition: compiler-change. Rationale: the machinery already exists and is
already used well three rules away — `[OWN-1]` ships `mechanical_fix: "write
\`move p\` for the affine place"`, `[STOR-1]` ships `"use replace: let old =
replace p = e; binds the previous owner"`, `[GRAM-6]` ships `"give the
condition exact value mode and type \`own Bool\`"`. `InvalidChildReborrow`
carrying a `kind` with no payload is an omission, not a design.

## D4 — syntax rejections print a bitset

Shape, `programs/p1_tree_wc.wf`, the writer's first attempt:

```whitefoot
let skip = bor(dotted, bnot(addressable));
```

Consequence, reproduced on a five-line program:

```text
whitefootc: Parsing/Source [GRAM-9]: SyntaxIssue { rule: Gram9, coordinate: SyntaxCoordinate
{ source: SourceId(0), start: ByteOffset(178), end: ByteOffset(183) },
expected: ExpectedTerminals { terminals: TerminalSet(38424498140022966840644862354), source_end: false } }
```

Byte 178 is ` bnot(`. The writer had to `head -c` the file to find it. The judge
hit the same class twice more while writing minimal probes — `[GRAM-5]` on a
nested expression in a constructor argument, `[GRAM-3]` on a one-byte
coordinate pointing at `{` — and bisected by byte offset both times.

Disposition: compiler-change. Rationale: the compiler holds the expected
terminal set and the source bytes. Rendering the set as spellings and quoting
the offending line is the whole fix, and this is the rule an unguided writer
hits first, because flat three-address form is the single largest departure
from every other systems language.

## D5 — [FORM-2] does not say what it expected

Consequence, reproduced with one double space after a comma in an effect row:

```text
whitefootc: CanonicalSource/Source [FORM-2]: CanonicalIssue { location: SourceNode(NodePath
{ components: [0, 0, 3] }, SyntaxCoordinate { source: SourceId(0), start: ByteOffset(88), end: ByteOffset(90) }) }
```

`CanonicalIssue` carries no `kind` at all — it is the only diagnostic in the
session with no payload whatsoever. It cost the writer one compile round and
cost the judge one while hoisting a binding in p5.

Disposition: compiler-change. Rationale: canonical form is machine-decided, so
the compiler knows the exact expected bytes at the point it rejects. Printing
them is free and turns a bisect into a diff.

## D6 — an absolute path is renamed to a file that does not exist

`compiler/src/bin/whitefootc.rs`:

```rust
fn logical_path(path: &Path, index: usize) -> String {
    let candidate = path.to_string_lossy();
    if !path.is_absolute() && portable_logical_path(&candidate) {
        candidate.into_owned()
    } else {
        format!("input{index}.wf")
    }
}
```

Consequence, the same program compiled two ways:

```text
$ whitefootc --par-ledger -o /dev/null /abs/path/probe_c_helper_denied.wf
PAR stage  input0.wf:26  for  denied  condition 3: …

$ whitefootc --par-ledger -o /dev/null probe_c_helper_denied.wf
PAR stage  probe_c_helper_denied.wf:26  for  denied  condition 3: …
```

Every byte offset and every `PAR` line then refers to a file name that exists
nowhere on disk. The writer's report quietly rewrote `input0.wf` to the real
program names before quoting the ledger, which is the tell: the output as
emitted was not usable as written.

Disposition: compiler-change. Rationale: an absolute path is how a script, a
Makefile, and an agent all invoke the compiler. Whatever portability property
`logical_path` protects belongs to the module identity the backend emits, not
to the text a human reads.

## D7 — there is no standard input

Shape, `programs/p4_copy_count.wf` as originally specified:

```whitefoot
command fn main(command.stdin as inp: own ReadFile, …)
```

Consequence:

```text
whitefootc: Semantics/Source [FN-7]: SemanticIssue { rule: Fn7, …, kind: InvalidStandardInputLabel
{ label: "command.stdin", declared_labels: ["command.args", "command.cwd", "command.stdout",
"command.stderr", "command.files"] } }
```

`[SYS-2]`'s operations contain no way to obtain a `ReadFile` for an already-open
descriptor, and `[PATH-1]` closes the escape: `p3_checksum /etc/hosts` exits 4,
`PathInvalid`, so `/dev/stdin` is unreachable too. `cat`, `wc`, `sort`, `grep -`
and every other filter is unwritable. The writer wrote a file-named substitute
instead.

Disposition: compiler-change. Rationale: this is a capability gap, not a
teaching gap — the diagnostic is the best in the toolchain and prints the entire
closed set. Nothing a warning or a pattern could say would make the program
writable.

## D8 — the denial is silent, and every example teaches the denied shape

Consequence. A writer compiles p1 and hears nothing. The staged verdict exists
and is excellent — it names the loop, the numbered condition, the offending
place rendered as its own source text, and usually the restructuring — but only
behind `--par-ledger`, which the writer would not have run had the trial not
asked for it. Meanwhile the three worked I/O programs a writer can read
(`dir_walk.wf`, `wfgrep.wf`, `byte_string.wf`) all compile to code identical to
`--no-overlap`, so the corpus teaches the denied shape by example while the
patterns file teaches the granted one by prose.

The ledger's rendering is also uneven where it matters least and most. It
renders a whole statement for some places — `at set cursor = cursor +wrap
record_size;` — and a bare binder for others, `at clean` in p3 and `at
log_fits` in p5, which name a `let` but not what is wrong with it, and give no
line number for the place as they do for the loop.

Disposition: warning+patterns. Rationale: the judgment is landed and correct;
what is missing is that it reaches the writer by default. An I/O loop the
compiler could not stage should say so without a flag, the way any other missed
optimization would. And `docs/patterns.md` P15 should say plainly that no
worked example in the tree currently holds the permission, so a writer copying
`dir_walk.wf` knows what they are copying.

## D9 — STOR-1's mechanical fix leads to a form OWN-1 rejects

Shape, the writer's third through fifth attempts on p1:

```whitefoot
set totals = walk<…>(…, running: move totals, …);
```

Consequence, both halves reproduced:

```text
[STOR-1] … kind: AffineSetTarget { target_type: "Counts",
         mechanical_fix: "use replace: let old = replace p = e; binds the previous owner" }

… applying exactly that fix:

[OWN-1] … kind: UseAfterMove { mechanical_fix: "introduce a new `let` binding before reuse" }
```

The right-hand side consumes the target root, so `replace` cannot help. The
real fix — have `walk` return its own subtotal and add fields at the caller — is
named by neither diagnostic; the writer found it by restructuring and calls it
three minutes of confusion.

Disposition: compiler-change. Rationale: a mechanical fix that the next rule
rejects is worse than no fix, because it spends an attempt. `[STOR-1]` already
knows the right-hand side; when the right-hand side consumes the target root it
should offer the fresh-`let` fix instead, which is the one `[OWN-1]` will
accept.

## D10 — an all-scalar struct is affine

Shape, `programs/p1_tree_wc.wf`:

```whitefoot
struct Counts {
  lines: u64;
  bytes: u64;
  status: u8;
}
```

`[OWN-1]`: "primitives, shared borrows, and tag-only enums copy on use; all
other values (owned composites, `box`, `arena`, `slice` as `&uniq`, uniq
borrows) are affine." So three `u64`s in a record need `move` at every use. The
writer hit `BareAffineUse` twice — at `let totals = running;` and again at
`return totals;` — because they did not generalise after the first, and the
interaction with D9 then produced the `let stale = replace …` idiom that
appears twice in p1:

```whitefoot
let stale = replace sub = walk<'recurse, …>(…);
```

Disposition: warning+patterns. Rationale: the affinity rule is the one-owner
law and is not up for change, and its diagnostic already carries the exact
fix — the per-site cost is one word. What is missing is the shape: an
accumulator threaded through a recursive walk should return its subtotal and
have the caller add fields, and nothing in `docs/patterns.md` says so. A warning
on a `let stale = replace …` whose binding is never read is the cheap half.

## D11 — hand-encoded byte lists cannot be documented or checked

Shape, `programs/p5_two_outputs.wf:1`:

```whitefoot
const report_label: array<u8, 8> =[114_u8, 101_u8, 112_u8, 111_u8, 114_u8, 116_u8, 32_u8, 35_u8];

const log_label: array<u8, 5> =[108_u8, 111_u8, 103_u8, 32_u8, 35_u8];
```

Consequence. Twelve such constants across the five programs. `[FORM-4]`: "There
are no comments. Documentation is the `doc` field of declarations", and
`const_decl := "const" IDENT ":" type "=" cvalue ";"` has no `doc` field. So
there is no place in the language to record that those bytes spell `report #`,
and the compiler cannot check that they do. The writer got all twelve right;
nothing would have told them if they had not.

Disposition: compiler-change. Rationale: this language's entire claim is that
the dangerous mistakes are unrepresentable. Here is the one construct where a
mistake is representable, silent, and permanent — and the writer must reach for
it for every byte of output any program produces. A `doc` on `const_decl` is
the minimum; a byte-string literal form that lowers to the same array is the
real answer.

## D12 — a re-bind the checker never asked for

Shape, repeated 34 times across the five programs:

```whitefoot
region 'x { set end = put_text<…>(destination: &uniq 'x line, …); }
let room = len(line);
let fits = ile(end, room);
```

The writer's report states this as the largest tax in the trial, on the reading
that `[ENT-5]` kill (b) applies whenever a callee's projected `writes` reaches
the buffer's root, and proposes a compiler change to remove it.

Consequence: the belief is wrong, and the change is unnecessary.
`probes/probe_e_hoisted_length.wf` is `p5_two_outputs.wf` with both `len`
bindings moved above the loop and above every `put_text` and `put_decimal`
call, and it compiles clean. `[ENT-5]` already says so — "for each length term
`len(P)`, the root binding of `P` but not `P`'s element storage — an element
write never kills a length fact" — and the compiler honours it across a callee
boundary. The judge also confirmed the guard is load-bearing: removing it
produces `[FN-8] UndischargedCallRequirement`, so the proof is real and the
stale fact discharges it.

Disposition: warning+patterns. Rationale: the compiler is right and needs no
change. What went wrong is that `[ENT-5]`'s element-storage exception is a
subclause inside a 300-word sentence that goes on to state kill (b) in terms
that read as overriding it, and no worked example shows a length fact
surviving a write. A patterns entry with the hoist, and a warning on a `len`
binding that re-establishes a fact that never died, both close it. Left alone,
this is a wrong mental model an unguided writer forms in twenty minutes and
carries into every program.

## D13 — region ceremony, and why it is not charged

The writer's five programs contain 120 `region` blocks in 1,694 lines, 7.1%,
and the report calls most of them punctuation for the borrow checker. Measured
against the hand-written comparators: `many_files_loop.wf` is 9 in 172 lines
(5.2%), `many_files_wide8.wf` 23 in 420 (5.5%).

Disposition: no-action, with reason. The writer's density is within the
expert's. A named region block is the only spelling the language has for a
borrow, so this is not a default written badly — it is what everybody writes,
including the people who know the answers. Charging it would be charging the
language's borrow form under the heading of writer defaults, which is a
different argument and belongs to a different batch.

## D14 — the granted verdict lowers to nothing, and P15 says so

`probe_a_staged_permitted.wf` holds the staged permission and still compiles to
code byte-identical to `--no-overlap`. On this host `many_files_loop` — the
P15 form, per-iteration scratch — measured 1.860 s against `many_files_narrow`'s
1.760 s, the per-iteration allocation with nothing yet to repay it, though the
two are within this host's noise.

Disposition: no-action, with reason. Nothing is silent here. `docs/patterns.md`
P15 states it in its own words: "the lowering that turns a granted verdict into
overlapped execution is not [landed], so today this form costs a per-iteration
`malloc` and fill and buys a granted verdict rather than speed", and explains
why writing it now is what makes the program fast later with no source change.
The owner's rule is satisfied by the existing teaching; landing the staged
lowering is the project's declared open work, not a defect this trial found.

## What this batch did not do

No compiler change, no pattern change, no spec change. The dispositions above
are findings; each names what would have to change and why, and none of them
was made here. Seven of the fourteen are diagnostics or ledger surfacing, which
are the cheapest, and two — D1 and D2 — are the ones that decide whether the
completion model reaches the shape of an ordinary utility.

## Evidence

- `research/experiments/blind-writer/2026-08-28/` — programs, probes, ledgers,
  and the writer's report. Its `README.md` states why the directory exists and
  when it is removed.
- `research/investigations/io-model/RESULTS.md` — the committed quiet-host
  narrow-against-wide medians this record cites.
- `research/experiments/io-completion-bench/programs/` — `many_files_narrow.wf`,
  `many_files_loop.wf`, and `many_files_wide8.wf`, the comparators.
