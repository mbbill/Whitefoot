# Batch 0085 — I/O correctness: hunting the unified-state completion model

Branch: `batch/0085-io-correct`, from `main` at `0295399d` (spec v0.37 active).

The job was to find real defects in the v0.37 implementation, fix each one
inside the existing model, and leave a regression test and a probe behind for
every finding. The direction that mattered most was widening: any case where
the compiler accepts what the rules forbid, or permits an overlap whose
observable result can differ from sequential execution.

No reachable widening was found. The permission judgment answered correctly on
every line this batch drew at it, and the one place where the *code* was
correct only by accident of what the operation catalog happens to contain
(F-04) has no row that reaches it. What was found instead is that the
*lowering* could not compile most of the programs that judgment permits: two
independent defects each made a permitted overlap fail to build, one on every
host and one on Linux only.

## Findings

| id | angle | classification | probe | status |
| --- | --- | --- | --- | --- |
| F-01 | differential execution | backend lowering: invalid module | `OVERLAP_BEFORE_A_BLOCK_JOIN`, `compiler/src/backend/tests/completion.rs` | fixed with test |
| F-02 | permission attacks | over-rejection: unimplemented capability | both programs quoted below | flagged |
| F-03 | runtime protocol, Linux | driver defect: cannot link on Linux | `the_compiler_owned_c_units_compile_in_the_default_dialect`, `compiler/src/backend/tests/completion.rs` | fixed with test |
| F-04 | permission attacks | latent widening in the system-call footprint | reasoning below; no catalog row reaches it today | fixed, no observable change |

### F-01 — a completion window before a block join named the wrong phi predecessor

A permitted overlap of two `may-suspend` system calls lowers to a direct
completion hand-out, which opens `completion.submit`, `completion.inline`,
`completion.offered`, `completion.wait`, `completion.join.inline`, and
`par.done` blocks inside the checked-IR block holding the window. The emitter
writes a block's phis before it writes the blocks that reach it, so an incoming
edge has to be named by predicting the label its predecessor will end at.
`block_exit_label` is that prediction, and it modelled the eight other
block-opening constructs but not the completion hand-out. Any accepted program
whose window sits in a block that jumps to a block with parameters emitted

```llvm
bb1:
  %v15 = phi i32 [ %v0, %bb2 ], [ %v0, %bb3 ]
```

while `bb2` in fact ended at `par.done.v26`. `clang` rejects that module —
`PHI node entries do not match predecessors!` — so the program did not compile
at all.

The reach is what makes this first-rank rather than a corner. Rebuilt against
the pre-fix compiler, four of the five shapes probed failed to compile:

| probe | pre-fix | post-fix |
| --- | --- | --- |
| `d2-read-distinct-destinations.wf` — two positioned reads in a `match` arm | invalid module | accepted |
| `f1-window-in-loop.wf` — window in a `loop` body | invalid module | accepted |
| `f2-window-before-nested-join.wf` — window in an `if` | invalid module | accepted |
| `f4-window-in-both-arms.wf` — a window at the end of every arm | invalid module | accepted |
| `f3-window-then-value-if.wf` — window followed by a `value_if` | accepted | accepted |

Every existing test put its window in a straight-line entry body whose
successor carried no block parameters, which is why the gate was green. A
window inside any ordinary control flow did not build.

The fix extends `block_exit_label` in `compiler/src/backend/emitter.rs` to
replay the completion queue the way emission drains it: `wait_for`
dependencies are joined at the step naming them, `submit` leaves the block at
`completion.offered`, a `finish` step drains everything outstanding, and
`emit_terminator`'s final drain settles the label before the terminator. The
match over the other block-opening operations moved into
`definition_exit_label` so the two hand-out mechanisms and the ordinary
constructs read as one list.

The regression test `a_completion_window_before_a_block_join_names_its_join_block`
asserts the join block's phis name `par.done`, then builds and runs the
executable under `WF_IO_HELPERS` 0, 1, and 4 in both lowerings and checks the
bytes on each stream. Building is half the assertion: an invalid module never
links. Verified to fail without the fix (`%v11 = phi i32 [ %v0, %bb2 ], …`).

### F-02 — disjoint parts of one resource have no expressible overlap route (flagged)

FIRST-PRINCIPLES §5 says field-level parallelism is spelled by borrowing the
field, and §11 builds the whole structural-independence story on it:

```whitefoot
refresh_checksum(value: &uniq record.checksum);
scan_payload(value: &record.payload);

receive_once(input: &uniq connection.receive, ...);
send_once(output: &uniq connection.send, source: &payload, ...);
```

Neither spelling compiles today, and the two routes to it are closed
independently.

A borrow of a field projection of an owned binding is an unimplemented
capability. In `compiler/src/semantic/check/borrows.rs`, only `buffer<T>`
carries `fields` through to a checked borrow; the `Box`, `SystemResource`,
`Slice`, and addressed-storage arms all require `fields.is_empty()`, and
anything else reaches `unsupported(RegionsAndBorrows)`. So `&uniq 'r
record.left` on a local `struct Record { left: u64; right: u64; }` is refused,
and so is `&uniq 'r sinks.left` on a `struct Sinks { left: Output; right:
Output; }` — the struct itself constructs fine, only the field borrow fails.

The child-reborrow route is closed by a different rule. `&uniq 'c deref(h).left`
is admitted, but [OWN-6] requires `'c` to be a locally-introduced region whose
block does not extend beyond the enclosing statement, so each child reborrow
needs its own `region` statement around it. [PAR-1] requires its two members to
be adjacent `let_stmt`s of one block. A `region_stmt` between them is neither,
so two child reborrows can never be members of one window — confirmed by
`j1-child-reborrow-two-adjacent.wf`, where a `region 'c` spanning the two
statements is rejected `OWN-6 InvalidChildReborrow`.

Both failing programs, so the next batch starts from a reproduction rather
than from this prose. The first is refused
`SemanticUnsupported { feature: RegionsAndBorrows }`:

```whitefoot
struct Record {
  left: u64;
  right: u64;
}

fn bump['r](value: &uniq 'r u64) -> result: own unit reads(value), writes(value) {
  let current = deref(value);
  set deref(value) = current +wrap 1_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  doc "FIRST-PRINCIPLES section 5 spells field-level parallelism as a borrow of the field itself.";
  let record = Record(left: 0_u64, right: 0_u64);
  region 'r {
    let a = bump<'r>(value: &uniq 'r record.left);
    let b = bump<'r>(value: &uniq 'r record.right);
  }
  return exit_status(code: 0_u8);
}
```

Borrowing the whole binding — `&uniq 'r record` — is accepted, so the refusal
is exactly the field projection. The same program with `Output` fields instead
of `u64` fields fails the same way, at the same place.

The second is refused `OWN-6 InvalidChildReborrow`, because the `region 'c`
that would have to cover both statements extends beyond either of them:

```whitefoot
fn touch['h](record: &uniq 'h Record) -> result: own unit reads(record.left, record.right), writes(record.left, record.right) {
  region 'c {
    let a = bump<'c>(value: &uniq 'c deref(record).left);
    let b = bump<'c>(value: &uniq 'c deref(record).right);
  }
  return unit;
}
```

Giving each statement its own `region` block is admitted, but then the two
statements are no longer adjacent `let_stmt`s of one block, which is what
[PAR-1] requires of its members.

This is under-approximation, not unsoundness, and it is reported honestly as
`SemanticUnsupported` rather than as a source rejection, so it breaks no
compiler rule. It is flagged rather than fixed because closing it is a language
capability — place resolution, lowering, cleanup attribution, and derived
release all have to carry a resource-typed field projection — and not a bug fix
inside this batch's boundary. Until it exists, every resource is atomic to the
overlap judgment, and the only disjointness a program can express is between
two separately owned values.

### F-03 — `whitefootc` could not link a completion program on Linux

`bridge.c` declared a union member named `linux`. GNU C predefines `linux` as
an object-like macro on a Linux host outside a strict `-std=cNN` dialect. The
repository gate compiles the completion runtime with `-std=c11`, where the
macro is absent and the member is an ordinary C11 identifier; the shipped
driver named no dialect at all, so `clang` used its `gnu17` default and the
member expanded to `1`:

```text
/tmp/whitefootc-completion-7/bridge.c:1034:36: error: expected identifier
 1034 |         file_result->kind = result.linux.kind == WF_LINUX_FILE_READ_AT
<built-in>:390:15: note: expanded from macro 'linux'
  390 | #define linux 1
whitefootc: clang exited with exit status: 1
```

Every Whitefoot program that needs the completion runtime — which is every
program that does any I/O — failed to link on Linux. The gate could not see it
for two reasons at once: it only ever compiled these units in the one dialect
the driver does not use, and its host is macOS, where `linux` is not
predefined.

Two changes, because there are two defects. The member is renamed to `ring`,
which removes the collision; and the driver and the backend test link both name
`-std=c11`, which is what makes the gate's green a statement about the shipped
link rather than about a dialect nothing selects. The sources already define
their own `_GNU_SOURCE`, so the strict dialect costs them nothing.

The regression test `the_compiler_owned_c_units_compile_in_the_default_dialect`
syntax-checks every compiler-owned C unit with no `-std` at all, which asserts
the complementary property: the units must not *depend* on the pin. It is
host-limited by construction — a macOS run leaves the `__linux__` bodies
unpreprocessed — and that limit is stated in the test. Verified in Docker that
it fails on Linux with the old member name and passes with the new one.

### F-04 — a consumed `own` system argument could lose its written footprint (latent)

[PAR-1] forms a call's written footprint from its row's `writes` paths
*together with* the places its consumed `own` arguments name. `user_call_footprint`
pushes that consumed-own write unconditionally. `system_call_footprint` pushed it
in an `else` branch of the row test, so a parameter the row mentions at all took
the row's path instead:

```rust
if written || read {
    …push the row's read and write accesses…
} else if matches!(parameter.mode, SystemParameterMode::Own) …
```

For a parameter in `writes` this is the same place either way, and for a
parameter in neither list the `else` fires, so every row in today's catalog
comes out right. The uncovered shape is an `own` parameter listed under `reads`
and not `writes`: it would contribute a read footprint element and no written
one, and a read element does not deny against another statement's read
footprint or shared loans. Another statement could then read the place
concurrently with the consume.

No catalog row has that shape, so there is nothing to reproduce and no
behavioural regression test to write — running the ledger over this batch's
probes and the whole corpus before and after the change gives identical
verdicts. The change is structural: `system_call_footprint` now pushes the
consumed-own write unconditionally, exactly as its user-call sibling does, so
the footprint is correct by the rule rather than by what the catalog happens to
contain.

## Differential execution matrix

Every program below was run across `WF_WORKERS` in {0, 1, 2, 4} crossed with
`WF_IO_HELPERS` in {0, 1, 4}, in both the default lowering and `--par`,
repeated. Each stream's byte sequence and the exit status had to match the
independent expectation on every run.

| program | what it forces | macOS runs | Linux runs | mismatches |
| --- | --- | --- | --- | --- |
| `read_overlap.wf` | two permitted positioned reads on one shared file into disjoint destinations, reassembled in fixed order | 240 | 240 | 0 |
| `write_order.wf` | eight rounds of one ordered stdout write beside one independent stderr write | 360 | 360 | 0 |
| `blocked_order.wf` | three 96 KB writes of one `Output`, each overlapped with an independent small write, stdout on a FIFO whose reader sleeps 250 ms so every bulk write really blocks | 96 | 96 | 0 |
| `abc_order.wf` | FIRST-PRINCIPLES §10 exactly — `A(out)`, `B(err)`, `C(out)`, with A a 96 KB write into a FIFO nobody is draining yet | 96 | 96 | 0 |

1,584 runs, 0 mismatches. The blocking cases are the ones that matter: with the
reader delayed, the pipe buffer fills and the target genuinely suspends, so the
surviving guarantee — successive unique loans of one `Output` run in that
order — is exercised against a suspending target rather than against a target
that finishes inline.

`abc_order.wf` is the schedule §10 spells out as required, and the emitted code
is that schedule literally: A submits, B submits, `par.done.v13` joins A alone,
C runs as a direct call on the loan A just returned, and only then does
`par.done.v16` join B. C therefore becomes ready when A completes rather than
when the whole group does, and B is still in flight while C uses `out`. The IR
shape is already pinned by `a_reused_unique_output_waits_only_for_its_own_prior_operation`;
what these runs add is that the ordering survives a target that actually
suspends, which a shape assertion cannot show.

Beside the purpose-written programs, every runnable `run`-kind conformance case
was compiled and executed across the same matrix, with its `arrange` argv,
files, directories, and stream redirects reproduced and its exit status
compared to the manifest's:

| lowering | host | cases built | runs | mismatches |
| --- | --- | --- | --- | --- |
| default | macOS | 173 of 173 | 2,076 | 0 |
| `--par` | macOS | 173 of 173 | 2,076 | 0 |
| default | Linux | 169 of 173 | 2,028 | 0 |
| `--par` | Linux | 169 of 173 | 2,028 | 0 |

8,208 further runs, 0 mismatches. This is measured evidence for this batch, not
a gate addition: `make check` already runs each case once, and multiplying the
corpus by twelve is runtime no current question needs.

The four cases Linux does not build are `sys14-list-outcome-exhaustive`,
`sys14-list-zero-range`, `sys14-directory-release`, and
`sys14-entry-kind-closed`, all directory enumeration. Linux has no approved
enumeration row in the qualification table, which
`compiler/src/backend/qualification.rs` states deliberately: "Linux supplies
`getdents64`, but this compiler intentionally has no approved ABI/record
mapping for it yet. Qualification must therefore report MissingMapping rather
than pretending the target lacks the semantic facility." The refusal arrives as
`TargetQualification(MissingMapping(Operation(12)))` — a target-qualification
failure, not a source rejection, which is what the compiler rules require of an
unimplemented capability. The consequence worth stating is that `wfgrep` and
`dir_walk` cannot be compiled for Linux at all today.

An honest note on the harness. The conformance driver above reported 180
mismatches on its first run and 36 on its second; all of them were the driver's
own bugs — zsh's reserved `argv` array, `arrange.argv[0]` being the program
name rather than an argument, and tab-delimited fields collapsing when a
`bytes` field was empty. Each was identifiable as a harness fault rather than
schedule dependence because the wrong answer was *constant* across all twelve
worker/helper combinations. A compiler defect of the kind this batch hunts
would have varied with the schedule.

macOS host: Darwin 25.5.0, Apple clang. Linux host: Docker, Ubuntu 24.04
aarch64, kernel 6.8, 2 CPUs, `--security-opt seccomp=unconfined` so
`io_uring_setup` is permitted. Without that flag Docker's default seccomp
profile blocks the syscall and the adapter reports `io_uring qualification
unavailable: Operation not permitted`, which is the fallback path, not the
native one.

## Sanitizers

| target | macOS (Apple clang) | Linux (Docker) |
| --- | --- | --- |
| `completion-test`, helpers 0/1/4 | PASS | PASS (clang 18 and gcc 14) |
| native adapter probe | n/a | `target=linux-io-uring status=pass`, then the harness re-run under `WF_REQUIRE_LINUX_IO_URING=1` PASS |
| `completion-sanitize` (ASan + UBSan) | PASS | PASS (gcc) |
| `completion-core-read-stress` | PASS, 200 of 200 | PASS, 200 of 200 (gcc) |
| `completion-core-read-tsan` | PASS | PASS (gcc, ASLR disabled) |
| full harness under TSan, helpers 0/1/4 | PASS | PASS, and again with io_uring required |

The full-harness TSan runs are not a `compiler/Makefile` target — the shipped
`completion-core-read-tsan` covers only the isolated core/read probe. They were
built by hand from the same source list `completion-test` uses, on both hosts.

Two environment notes, neither a program defect. The `wf-io-bench:linux` image
carries clang 18 without `libclang_rt`, and the container has no working
package mirror, so the Linux sanitizer runs used `gcc` from `rust:latest`
instead. And TSan on aarch64 Linux aborts with `unexpected memory mapping`
under the container's ASLR; running it under `setarch -R` fixes it and the
probe passes.

## Refuted angles

Each of these was attacked with both sides constructed and behaved exactly as
the rules require. The probe names below are the batch's scratch files; the two
findings whose probes had to survive are the ones now carried as compiler tests
(F-01, F-03) and the one quoted in full above (F-02).

| angle | probe pair | result |
| --- | --- | --- |
| same-`Output` vs distinct-`Output` window | `a1.wf` | denied / permitted |
| field effect path vs whole-aggregate loan (§5) | `a3.wf` | denied — the field path does not shrink the loan |
| pass-through release attribution (§9) | `b1.wf`, `b2-passthrough-declared.wf` | `pure` rejected, `writes(file)` accepted |
| aggregate leaf identity | `b3`…`b6` | the released leaf is the right formal; flattening rejected |
| declared-but-unexhibited padding | `b7-padded-row.wf` | rejected [EFF-2] |
| enum payload origin | `c1`, `c2` | payload carries the formal origin |
| two reads, same vs distinct destination | `d1`, `d2` | denied / permitted |
| read destination as a concurrent write's source | `d3` | denied — exclusive vs shared loan |
| slice view vs exclusive write of its backing | `g1`, `g2` | rejected at [OWN-5] before permission is reached |
| user wrapper omitting or padding a system call's row | `h3`, `h4` | both rejected [EFF-2] |
| permit forged, duplicated, reused after a failed open | `p1`, `p2`, `p3` | [TYPE-6], [OWN-1], [OWN-1] |
| two `DirectorySource` over one directory | `p5` | permitted — two permits, two fresh owners, one shared selector loan |
| host exhaustion is a typed outcome | `exh/exhaust.wf` | under `ulimit -n 4` the open returns `ResourceExhausted` and the program exits normally |
| intervening `set` on shared vs unrelated storage | `r1`, `r2` | denied / permitted |
| a propagating member inside a window | `r3` | `(write, propagate)` permitted, `(propagate, write)` denied by condition 4 |
| drop ordering around a completion join | `q1` | every derived release is emitted after `par.done` |
| PAR-2 loop overlap with I/O in the body | `s1-loop-reads-shared-file.wf` | permitted, not actualized — see below |
| T3: a claim must not tax the correct path | `t3-no-claim.wf`, `t3-with-claim.wf` | see below |

### PAR-2 with I/O

A counted loop whose body reads one shared `ReadFile` at an iteration-derived
offset into an iteration-local buffer, folding into one `+wrap` accumulator, is
*permitted*: the shared file loan needs no condition of its own, every written
place is either the accumulator or iteration-introduced, and `read_at`'s
contract is a positioned read that does not advance the resource's logical
state, so two concurrent reads at different offsets are exactly what the
contract admits.

The lowering then declines to actualize it, and emits no split thunk at all.
The mechanism is the same `overlap_is_actualizable` guard that keeps a
may-suspend call off a compute lane: a loop split lowers to a recursive
splitter whose left and right halves are one `IrOverlap`, and that overlap is
refused because the splitter may suspend. That is [PAR-2]'s own escape — "an
implementation without that lowering executes the loop sequentially" — and
permission is never an obligation, so nothing is wrong. It is worth writing
down because it means the loop half of the I/O concurrency story is currently
judgment only, with no lowering behind it.

### T3

The two programs differ only in that one discharges a loop-induction bound with
a branch and the other with a `claim`. The permission ledger is identical — the
same window, permitted, with the same chain. The emitted window is identical
modulo SSA numbering across all six blocks of the hand-out. The claim's only
cost is its own conditional branch to its trap block; nothing on the submit,
offered, wait, join, or drain path reads a trap latch, and the latched trap
writer is selected only when compute thunks are actually used, which a
completion-only overlap never triggers.

Two attempts at a claim were refused before one was admitted, and both refusals
were correct: a claim about `args_count` was rejected [CLM-1] as an unstated
system-result property, and a claim the checker already derives was rejected
[CLM-2] as redundant.

## Gate

Canonical `make check` at the repository root, on the final revision of this
branch, ends `== WHITEFOOT ALL TESTS GREEN ==`. Its own numbers:

```text
fmt and clippy                   clean
conformance structure            29 of 29
gate all-target Rust tests       1323 passed, 0 failed, 1 costly adapter ignored
maintained programs              54 passed, 0 failed
spec activation chain            29 unbroken activations
conformance coverage             137 of 137 rules, 0 uncovered
separate conformance adapter     502 Pass, 1 Skip
```

The library test count moves 1321 to 1323: this batch's two regression tests.
Nothing else in the corpus changed, and no verdict, case, or manifest record
was touched.

The sanitizer targets, the hostile stress, the differential matrices, and the
Linux runs above are not part of that entry point. They are separate
`compiler/Makefile` targets and scratch harnesses invoked independently, and
their results are this record's own measurements.

## Judgment calls

- **F-02 is flagged, not fixed.** It is under-approximation reported as an
  explicit capability gap, and closing it is a language capability rather than
  a bug fix. Recorded with both failing programs so the next batch starts from
  a reproduction.
- **F-04 was fixed without a regression test.** No catalog row reaches the
  uncovered shape, so any test would assert a property of the catalog rather
  than of the checker, and would become rot the moment the catalog changed. The
  guard is that the code now matches its user-call sibling and the rule text;
  the existing ledger tests prove nothing regressed.
- **F-03 got two changes rather than one.** The rename alone would have fixed
  today's failure; the dialect pin is what stops the next GNU predefine from
  doing the same thing, and it removes a real divergence between what the gate
  compiles and what the driver compiles.
- **The F-03 regression test asserts the opposite of the fix's pin.** The
  driver names `-std=c11`; the test compiles with no `-std`. That is
  deliberate: pinning the dialect makes the shipped link defined, and the test
  keeps the units from depending on the pin, so a consumer compiling them any
  other way is not broken.
- **`docker run --security-opt seccomp=unconfined`** is a per-run flag, not a
  VM change. Without it no io_uring evidence is possible at all.
- **Linux sanitizers used gcc.** The image's clang has no sanitizer runtime and
  the container has no package mirror. gcc's ASan/UBSan/TSan check the same
  sources.
- **The clock, keyed directory entries, whole-directory `DirectorySource`
  loans, and create/namespace APIs were left alone**, as deferred design items.
- **No `spec/kernel-spec.md` change.** Nothing found required one.

## Not done

- **Windows was not executed.** It has no runner here and stays cross-link
  only, exactly as batch 0082 left it.
- **Directory enumeration was not exercised on Linux**, because the
  qualification table has no approved Linux enumeration row. Everything said
  above about `open_directory_source` and `directory_next` is macOS evidence.
- **The two overlap mechanisms do not currently mix, so their interaction was
  not stressed.** `overlap_is_actualizable` refuses to hand any may-suspend
  call to a compute lane, and it also refuses the recursive splitter a `for`
  loop with I/O in its body would need. Every run above therefore exercises
  completion hand-out beside compute hand-out in one process, never one inside
  the other. Whether they should mix is a design question this batch did not
  open.
- **No new conformance case was added.** Every finding is a compiler or driver
  defect rather than a language rule, so the regression tests are ordinary
  compiler tests and the conformance corpus is untouched.
