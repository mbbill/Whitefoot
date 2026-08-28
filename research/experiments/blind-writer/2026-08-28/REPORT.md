# Writer trial: five ordinary I/O utilities in Whitefoot v0.38

Author: a senior systems programmer with no prior Whitefoot exposure.
Materials: `spec/kernel-spec.md` (v0.38), `docs/patterns.md` (D6), the
`whitefootc` gate binary, and the example programs in `tests/programs/`.
Compiler source, `docs/done/`, and `research/` were not read.

Programs: `programs/` beside this file. [Path rewritten by the judge; the
writer named a scratch tree outside the repository.]
Scratch, binaries and test trees: the writer's own scratch tree, not retained.

---

## 0. Headline results

| # | Program | File | Lines | Compile attempts | Correct against reference |
|---|---|---|---|---|---|
| 1 | line/byte count over a tree | `p1_tree_wc.wf` | 497 | **6** | yes, matches `wc -l`/`wc -c` |
| 2 | literal grep over a tree | `p2_tree_grep.wf` | 455 | **1** | yes, matches `grep -rl` |
| 3 | checksum of files named on argv, 64 KiB chunks | `p3_checksum.wf` | 305 | **1** | yes, matches a Python FNV-1a |
| 4 | stdin → stdout, count newlines | `p4_copy_count.wf` | 258 | **1** | **not writable as specified** — see §5 |
| 5 | interleaved stdout report + stderr log | `p5_two_outputs.wf` | 179 | **1** | yes |

Total wall clock for the five programs: **23:50 → 00:13, about 23 minutes** —
roughly 8 minutes reading the spec, patterns and two example programs, and
15 minutes writing, compiling and testing. This report took a further 4 minutes. That is model time,
not human time; treat the *ratios* (reading vs. writing, attempts per program)
as the signal, not the absolute number.

Zero `claim` statements were needed in any of the five programs. Every partial
operation — 60-odd subscripts, every `%`/`/`, every system range call — was
discharged by ordinary `if` branches and by `len()` rebinding. That is the
single most encouraging result in this trial.

Two extra probe programs isolate the `--par-ledger` behaviour:
`probe_a_staged_permitted.wf` and `probe_b_staged_denied.wf` (§7).

---

## 1. Program 5 — two outputs, interleaved

`programs/p5_two_outputs.wf`, 179 lines, **1 compile attempt**, ~4 minutes.

Writes `report #N\n` to `command.stdout` and `log #N\n` to `command.stderr` for
N in 0..8. Verified: separated when redirected to two files, correctly
interleaved when both land on one fd.

I wrote this first to establish the toolchain loop. It compiled on the first
try, which surprised me — I expected effect-row trouble. The row
`reads(out, err), writes(out, err), allocates(heap)` was a guess from EFF-2
("Rows are checked both ways… undeclared-but-exhibited and
declared-but-unexhibited are both errors") and it happened to be right.

Nothing confused me here except one thing I only understood later: I already
had to write `let report_room = len(report);` *after* the last write into the
buffer and before the `emit_all` call, because `emit_all`'s
`requires ile(length, capacity)` needs a live `len` fact and my own `put_text`
call had just killed it. See §6.2.

---

## 2. Program 4 — copy stdin to stdout, counting newlines

`programs/p4_copy_count.wf`, 258 lines, **1 compile attempt**, ~5 minutes.

### The program as specified cannot be written

There is no standard input. `[FN-7]`'s closed entry table is exactly:

```
| 0 | command.args   | own Args          |
| 1 | command.cwd    | own DirectoryRead |
| 2 | command.stdout | own Output        |
| 3 | command.stderr | own Output        |
| 4 | command.files  | own FileFactory   |
```

and `[SYS-2]`'s sixteen operations contain no way to obtain a `ReadFile` for an
already-open descriptor. I probed it directly:

```
$ whitefootc work/stdin_probe.wf
Semantics/Source [FN-7]: … InvalidStandardInputLabel {
  label: "command.stdin",
  declared_labels: ["command.args", "command.cwd", "command.stdout",
                    "command.stderr", "command.files"] }
```

The diagnostic is excellent — it prints the whole closed set. But the gap is
real and it is not small: `cat`, `wc`, `sort`, `grep -` and every other filter
in the Unix idiom is unwritable. Worse, the usual escape hatch is also closed:
`[PATH-1]` makes `relative_path` reject any target-root prefix, so a program
cannot reach `/dev/stdin` either (confirmed: `./p3 /etc/hosts` exits 4 =
`PathInvalid`).

### What I wrote instead

The nearest writable program: copy the file named in `argv[1]` to stdout in
64 KiB positioned reads, count `\n`, report the count on stderr.

```
$ ./p4 multi.txt > multi.copy 2> multi.err
$ cat multi.err ; wc -lc multi.txt ; cmp multi.txt multi.copy && echo IDENTICAL
lines =20000
   20000  548894 multi.txt
IDENTICAL
```

Missing file exits 3; no argument exits 2.

### What confused me

`read_at` returns an **absolute endpoint**, not a count (`ReadBytes(next)` with
`start <= next <= end`). That is stated clearly in `[SYS-8]` and it is the right
design — it makes the `[ENT-3.S10]` bound facts usable — but it inverts the
muscle memory of every `read(2)` I have written. I got it right only because I
had just read S10.

---

## 3. Program 3 — checksum files named on the command line

`programs/p3_checksum.wf`, 305 lines, **1 compile attempt**, ~6 minutes.

FNV-1a-64 over each file, 64 KiB chunks, one output line per file:
`<name>  <digest>  <bytes>`.

```
$ ./p3 sample.txt multi.txt big.txt nosuch.txt
sample.txt  8531520280712633980  17
multi.txt  12599103310628858505  548894
big.txt  1918533108972712538  266669
nosuch.txt  14695981039346656037  0
exit=3
```

All three digests match a Python reference byte for byte.

This is the one program where I deliberately followed **P15** from
`docs/patterns.md`: the per-iteration `data`, `name` and `line` buffers are
constructed *inside* the loop body rather than hoisted. Doing that cost me
nothing in writing effort and is what the pattern doctrine asks for. It did not
earn the staged permission — see §7 for why, which turns out to be nothing to
do with the scratch buffers.

### What confused me

Nothing blocked me, but two things cost thinking time:

* A `HostString` is consumed by `relative_path` on success *and* on failure
  (`[PATH-1]`). To both print the name and open it, I call `arg_get` twice.
  `[SYS-9]` explicitly blesses this ("several leases may refer to the same
  immutable bytes"), so it is correct, but it reads like a bug the first time
  you write it.
* `host_copy_bytes` and `open_read` are on opposite sides of a design seam:
  `open_read` takes a `RelativePath` (multi-component paths allowed), while
  `open_file`/`open_directory` take a raw name range that must be exactly one
  component with no separator. I used `open_read` here and `open_file` in the
  tree walkers. Both choices are forced; the spec says so in `[SYS-14]`, but you
  have to notice that two different open operations exist for two different
  name sources.

---

## 4. Program 1 — count lines and bytes of every file in a tree

`programs/p1_tree_wc.wf`, 497 lines, **6 compile attempts**, ~11 minutes.

Recursive `DirectorySource` walk, one line per regular file, then a total.

```
$ cd work/tree && ../p1 | sort
0 0 c/empty.txt
0 17 a/b/f3.txt
1 2 a/f2.txt
2 8 f1.txt
3 27 total
```

matching `wc -l` / `wc -c` per file. On the 25-file `tests/programs/` tree it
printed `6656 245576 total`, exactly `cat *.wf | wc -lc`.

### The six attempts

**Attempt 1 — `[GRAM-9]`** at `let skip = bor(dotted, bnot(addressable));`

> `Parsing/Source [GRAM-9]: SyntaxIssue { … ByteOffset(11951) … }`

Flat three-address form: a `call` may not sit in an `atom` position. I knew the
rule and still wrote a nested call, because `bnot(x)` reads like an operator.
The diagnostic gives a byte offset but no message text; I had to `head -c` the
file to see what it was pointing at. **This is the one diagnostic in the whole
session that did not tell me what to do.**

**Attempt 2 — `[TYPE-6]` `DeclarationCollision { spelling: "permit" }`**

I reused the binder name `permit` in a nested block while an outer `permit` was
still in lexical scope — even though the outer one had already been *moved* and
was dead. `[TYPE-6]` shadowing is about declarations, not liveness. The
diagnostic named the spelling and both byte offsets: perfect, fixed in seconds.

**Attempt 3 — `[OWN-1] BareAffineUse`** at `let totals = running;`
**Attempt 4 — same** at `return totals;`

`struct Counts { lines: u64; bytes: u64; status: u8; }` is **affine**, because
`[OWN-1]` makes every owned composite affine regardless of its field types. So
an all-scalar three-field struct needs `move` on every use. The mechanical fix
string ("write `move p` for the affine place") is in the diagnostic, so each fix
took one edit — but I hit it twice because I did not generalise after the first.

That led to the real problem: I had written

```whitefoot
set totals = walk<…>(…, running: move totals, …);
```

which `[SET-1]` rejects — the right-hand side moves the target root, so the
commit writes a dead root — *and* `set` cannot target an affine place at all
(`[STOR-1]`). I restructured the accumulator: `walk` no longer takes a running
total, it returns its own subtotal and the caller adds fields. That is a better
design, and the language pushed me to it, but it was three minutes of confusion.

**Attempt 5 — `[OWN-6] InvalidChildReborrow`** at `&uniq 'source deref(factory)`

This was the only genuine wall. I had written

```whitefoot
region 'source {
  let permit = reserve_file<'source>(factory: &uniq 'source deref(factory));
  match open_directory_source<'c>(permit: move permit, directory: dir) { … }
}
```

`[OWN-6]` requires a child reborrow's region to be "a locally-introduced region
whose block does not extend beyond the enclosing statement". My `region 'source`
block holds two statements, so it extends beyond the `let`. The rule as written
is opaque; what it means operationally is:

> **Every `&uniq 'r deref(h)` must sit inside a `region 'r { … }` block that
> contains exactly one statement.**

And that immediately creates a second problem: anything the statement *binds*
dies at the end of that one-statement region. `let permit = …` in its own region
is useless.

The diagnostic (`InvalidChildReborrow`, no message) did not tell me any of this.
I recovered it from `tests/programs/dir_walk.wf`, which solves it with a
one-line helper:

```whitefoot
fn open_source_from['f, 'd](factory: &uniq 'f FileFactory, …) -> … {
  let permit = reserve_file<'f>(factory: move factory);
  return open_directory_source<'d>(permit: move permit, directory: directory);
}
```

`move factory` moves the borrow *holder* (uniq borrows are affine) instead of
reborrowing, so no region constraint applies. I added three such helpers and
attempt 6 compiled. **Without the example programs I do not believe I would have
found this in reasonable time from the spec alone.**

A related consequence I had to work around: when a one-statement region must
also produce a value that outlives it, the only route is `[SET-2]`:

```whitefoot
let sub = Counts(lines: 0_u64, bytes: 0_u64, status: 0_u8);
region 'recurse {
  let stale = replace sub = walk<…>(…);
}
set totals.lines = totals.lines +wrap sub.lines;
```

That `let stale = replace …` line, whose binding is immediately discarded, is
pure ceremony forced by the interaction of two otherwise good rules.

---

## 5. Program 2 — grep a literal byte pattern across a tree

`programs/p2_tree_grep.wf`, 455 lines, **1 compile attempt**, ~7 minutes.

Same walk as program 1; per file, a chunked scan with a carried overlap of
`patlen-1` bytes so a match straddling a 64 KiB boundary is still found.

```
$ cd work/gtree && ../p2 NEEDLE | sort     # needle straddles offset 65536
straddle.bin
sub/hit.txt
$ grep -rl NEEDLE . | sed 's|^\./||' | sort
straddle.bin
sub/hit.txt
```

Exit 0 on match, 1 on no match, 2 on usage error — `grep` conventions.

One attempt, because by this point I had internalised the five rules that cost
me program 1: flat calls, `move` on composites, unique binder spellings,
one-statement reborrow regions, and rebinding `len()` after every write.

The in-place overlap shift is the one place the borrow rules made me write
something odd. Reading and writing the same buffer through a `&uniq` parameter
inside a loop needs two separate one-statement regions:

```whitefoot
for @slide step in from..to {
  let byte = 0_u8;
  region 'peek {
    set byte = byte_at<'peek>(source: &'peek deref(data), index: step);
  }
  region 'poke {
    set cursor = put_byte<'poke>(destination: &uniq 'poke deref(data), at: cursor, value: byte);
  }
}
```

That is correct and safe. It is also four lines of scaffolding around `a[i]=a[j]`.

---

## 6. What the language made easy, and what it taxed

### 6.1 Easy, and better than I expected

* **Proof obligations were not the problem.** I braced for a fight with
  `[OP-4]`, `[ENT-6]` and `[SYS-8]` and never had one. Zero claims. The pattern
  that carries everything is the *safe accessor*:

  ```whitefoot
  fn byte_at['s](source: &'s buffer<u8>, index: own u64) -> result: own u8 reads(source) {
    let capacity = len(deref(source));
    let inside = ilt(index, capacity);
    if inside {
      return deref(source)[index];
    }
    return 0_u8;
  }
  ```

  One `if`, one `len`, and `[ENT-3.S1]` + `[ENT-3.S6]` discharge the subscript.
  I wrote this once per program and every bounds problem downstream vanished.
* **`[ENT-3.S10]` pays for itself.** `start <= next <= end` on the `ReadBytes`
  and `ListBytes` arms is exactly the fact you need and never have in C.
* **`for @l i in lo..hi` (P11) is the right primitive.** Every index walk in
  these programs is one, and `binder < upper_capture` discharged every derived
  subscript with no writer input.
* **Effect rows were cheap.** I guessed every row from EFF-2's rules and got all
  of them right on the first try across five programs. That is a strong result
  for a redundancy the language enforces both ways.
* **`--par-ledger` is genuinely excellent.** See §7.

### 6.2 The taxes, in order of how much they cost me

1. **`len()` must be re-bound after every write.** `[ENT-5]` kills a length fact
   when a call's projected `writes` reaches the buffer's root, even though the
   call only touched elements. So the sequence

   ```whitefoot
   region 'x { set end = put_text<…>(destination: &uniq 'x line, …); }
   let room = len(line);          ← mandatory, and easy to forget
   let fits = ile(end, room);
   if fits { …emit… }
   ```

   appears **34 times** across the five programs — 34 of the 41 `let … = len(…)`
   bindings are immediately followed by a comparison guard, i.e. they exist only
   to re-establish a fact a preceding call killed. The rule's own prose says
   "an element write never kills a length fact" — that is true for direct `set
   p[i]`, but false the moment the element write is behind a function whose
   effect path can only name the whole parameter. This is the biggest gap
   between what `[ENT-5]` promises and what a writer experiences.

2. **The one-statement reborrow region.** Discussed in §4. It is the reason the
   five programs contain **120 `region` blocks** (39/28/22/16/15). Most carry no
   design intent at all; they are punctuation for the borrow checker.

3. **No modules, so every program re-declares the prelude.** `digit_byte`,
   `put_byte`, `put_text`, `put_range`, `put_decimal`, `emit_all`, `byte_at`
   come to roughly 110 lines that are byte-identical in four of the five
   programs — 30–60% of each file before `main`. `[PROG-1]` is explicit that
   this is by design (one closed unit, no import), and multi-source units exist,
   but as a single-file writer I paid it five times.

4. **No string literals in expressions.** Every piece of output text is a
   `const … : array<u8, N> =[114_u8, 101_u8, …];`. I had to hand-encode
   `report #`, `log #`, `lines =`, `total`, `/` and `\n` as decimal byte lists.
   This is error-prone in a way nothing else in the language is: the compiler
   cannot tell me I typed the wrong codepoint, and `[FORM-4]` means I cannot
   even leave a comment saying what the bytes spell. The `doc` string on the
   *declaration* is the only place to say it, and `const_decl` has no `doc`.

5. **All-scalar structs are affine.** `move` on a three-`u64` struct is a
   surprise every time, and it interacts badly with `set` (`[STOR-1]` rejects
   affine targets, `[SET-1]` rejects a moved root), forcing the
   `let stale = replace x = …;` ceremony.

6. **`FORM-2` byte-exact formatting.** One double space cost one compile round
   (`CanonicalIssue` at a byte offset, no message). I support the rule; the
   diagnostic should print the expected line.

### 6.3 Diagnostics scorecard

Of 6 distinct rejections across the whole session:

| Rule | Told me the offending construct | Told me the fix |
|---|---|---|
| `FN-7` unknown entry label | yes, plus the whole legal set | yes |
| `TYPE-6` collision | yes, both declaration sites | yes |
| `OWN-1` bare affine | yes | yes, exact `mechanical_fix` string |
| `GRAM-9` nested call | byte offset only | no |
| `OWN-6` invalid child reborrow | byte offset only | **no** |
| `FORM-2` non-canonical bytes | byte offset only | no |

The three that carry a `kind:` payload are excellent. The three that carry only
a coordinate are the ones that cost time, and `OWN-6` is the one that would have
stopped me without the example corpus. All six are machine-parseable Rust
`Debug` output rather than rendered text; a writer sees
`SemanticIssue { rule: Own6, location: SourceNode(NodePath { components: […] }, …) }`
and has to reach for `head -c` to find the source line.

---

## 7. `--par-ledger`: what was permitted, what was denied, and whether the reasons helped

The ledger is the best part of this toolchain. Every line names the loop, the
numbered condition, the offending place *rendered as its own source text*, and
usually a restructuring. Example:

```
PAR stage  p3_checksum.wf:189  for  denied  condition 1: a statement of the body
  neither executes before the submission on every path nor is reached only
  through it; instead, write the body so its first I/O submission is reached on
  every path through it and everything else is reached only through it, at clean
```

### 7.1 Verdicts

| Program | Loop | Verdict | Reason given |
|---|---|---|---|
| p1/p2/p3/p4/p5 | `@copy` in `put_text`/`put_range` | PAR-1 denied | "the body contains a statement that forms a borrow of storage the iteration does not introduce" |
| all | `@publish` in `emit_all` | PAR-3 denied | "a return leaves the loop from the remainder; instead, take every early return, break, or propagate in the prologue…" |
| p1/p3/p4 | `@chunks`/`@pump` (read loop) | PAR-3 denied | "a break naming this loop leaves the loop from the remainder…" |
| p1/p3/p4 | `@scan`/`@fold` (byte fold) | **PAR-2 permitted** | "eligible; one accumulator under `+wrap`" |
| p3 | `@fold` with FNV mix | PAR-2 denied | "the loop writes storage outliving the iteration that no exactly associative operation reduces, at `set digest = mixed *wrap 1099511628211_u64;`" |
| p1/p2 | `@records` (per directory entry) | PAR-3 denied | "condition 1 … at `set cursor = cursor +wrap record_size;`" |
| p1/p2 | `@batches` (directory batches) | PAR-3 denied | "condition 7: … forms a borrow of storage the iteration does not introduce; instead, write the borrow as an argument of the call that uses it" |
| p3 | `@each` (per argv file) | PAR-3 denied | "condition 1 … at `clean`" |

Interesting non-denials inside the denied loops:

```
PAR place  p1_tree_wc.wf:315  replicated    let child = buffer_new(4400_u64, 0_u8);
  iteration-own storage with copy elements, which an implementation may give
  each in-flight iteration its own of
PAR place  p1_tree_wc.wf:315  serialized-E  set totals.lines = totals.lines +wrap sub.lines;
  every footprint element and loan touching it belongs to the remainder, whose
  accesses to storage rooted outside the loop are taken in index order
```

So P15 works exactly as advertised at the *place* level: the per-iteration
buffers were classified `replicated`, and the plain source-order `set`
accumulator was classified `serialized-E` with no associativity requirement.
The loop still failed on other grounds.

### 7.2 Did the denial reasons tell me how to change the program? Yes — twice, decisively.

I wrote two probes to test it.

**`probe_a_staged_permitted.wf`** — a P15-shaped loop: per-iteration `name` and
`data` buffers, one `open_file` per iteration, an ordinary `set digest = digest
+wrap widened;` accumulator, no output inside the loop. My *first* version wrapped
reserve+open in a helper `open_regular_from(…)`, exactly as programs 1 and 2 do.
The ledger said:

```
PAR stage  denied  condition 3: a may-suspend call retains a borrow past its own
  submission on storage the body writes and the iteration does not introduce;
  instead, give each iteration its own resource, or leave this loop sequential:
  storage that carries one position cannot be held by two iterations at once,
  at &uniq 'open files
```

That named `&uniq 'open files` — the file factory. I moved `reserve_file` inline,
exactly as P15 spells it, so the factory loan becomes a call-scoped temporary
that ends before the submission:

```whitefoot
region 'f {
  let permit = reserve_file<'f>(factory: &uniq 'f files);
  region 'n {
    match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 2_u64) {
```

and the ledger flipped:

```
PAR stage  probe_a…:17  for  permitted  staged at open_file<'f, 'n>(…); 5 places classified
PAR place  serialized-P  &uniq 'f files   … prologues run in index order without overlapping
PAR place  read-only     &'f cwd
PAR place  serialized-E  set digest = digest +wrap widened;
PAR place  replicated    let name = buffer_new(16_u64, 0_u8);
PAR place  replicated    let data = buffer_new(65536_u64, 0_u8);
```

**One denial message, one edit, permission granted.** That is the ledger working
as designed.

It also isolates a trap: **the natural refactor destroys the permission.**
Factoring `reserve_file` + `open_*` into one helper — which `dir_walk.wf` does,
which I did in programs 1 and 2, and which is unavoidable given the `OWN-6`
one-statement-region rule of §4 — extends the factory loan across the
submission and denies the whole loop. Two good rules (P15's inline reserve,
OWN-6's reborrow restriction) push in opposite directions.

**`probe_b_staged_denied.wf`** — probe A plus one `write_once` to `command.stdout`
inside the remainder. Same loop, one added statement:

```
PAR stage  denied  condition 3: … at &uniq 'say out
PAR place  denied  &uniq 'say out  the body writes it through a retained borrow
  and its type carries one position, so no iteration can be given its own
```

### 7.3 The structural finding

**Any loop that writes to `stdout` or `stderr` per iteration can never hold the
staged permission.** `Output` is a single-position resource; a `may-suspend`
`write_once` retains its `&uniq` loan past submission; `[PAR-3]` requires every
such retained loan to be on iteration-introduced or replicable storage; `Output`
is neither.

That is exactly the shape of all five programs in this trial, and of `wc`,
`grep`, `ls`, `find`, `md5sum` and every other tree utility: *read many things
concurrently, report each one*. Programs 1, 2 and 3 could each be staged on the
read side and are blocked only by the report line.

This is not a spec bug — the rule is doing precisely what it says, and
interleaving two overlapped iterations' output would be observable. But it means
the headline I/O-overlap capability does not reach the canonical utility shape
without one of: a per-iteration output resource that merges in index order; a
buffered-output type (already named as future work in `[SYS-12]`); or a writer
discipline of collecting results and printing after the loop, which costs the
streaming property. Worth carding.

---

## 8. Summary of findings

1. **No stdin.** `[FN-7]`'s entry table has no standard input and `[SYS-2]` has
   no operation to reach one; `[PATH-1]` closes the `/dev/stdin` workaround. The
   entire Unix filter genre is unwritable. Highest-value single addition.
2. **`OWN-6`'s one-statement reborrow region** is the only rule I could not
   apply from the spec text. It needs either a worked example in
   `docs/patterns.md` or a diagnostic that states the restructuring.
3. **`len()` dies on every write through a function boundary**, so the
   `let room = len(b);` re-bind is mandatory boilerplate — 34 of 41 `len`
   bindings in these five programs are that guard. A `writes` path that could name element storage, or an S6-style
   "length survives a callee write" rule, would remove it.
4. **Three of six diagnostics carry no message**, only a `NodePath` and a byte
   offset. `GRAM-9`, `OWN-6` and `FORM-2` should print the offending source line
   and, for `OWN-6`, the mechanical fix — the machinery is already there
   (`OWN-1` does it).
5. **No string literals** means hand-encoded `array<u8, N>` byte lists, with no
   comment form to say what they spell. `doc` on `const_decl` would help.
6. **All-scalar structs are affine**, which forces `move` plus the
   `let stale = replace x = call();` idiom whenever a value must escape a
   one-statement region.
7. **`--par-ledger` is the best diagnostic surface in the toolchain.** It named
   the offending place in source text, cited the numbered condition, and its
   fix advice converted a denial into a grant in one edit.
8. **The staged permission cannot reach the per-iteration-output shape.** §7.3.
9. **The proof obligations are not the barrier.** Zero claims, zero unproved
   subscripts, five working programs. Whatever else is hard about this language,
   proving array bounds and integer domains in real I/O code was not.

## 9. Reproduction

```
cd research/experiments/blind-writer/2026-08-28
for f in programs/p*.wf; do ./compiler/target/gate/whitefootc -o work/$(basename $f .wf) $f; done
./compiler/target/gate/whitefootc --par-ledger -o /dev/null programs/p3_checksum.wf
cd work/tree && ../p1                       # tree wc
cd work/gtree && ../p2 NEEDLE               # tree grep
cd work && ./p3 sample.txt multi.txt        # checksum
cd work && ./p4 multi.txt > /dev/null       # copy + line count
cd work && ./p5                             # two outputs
```
