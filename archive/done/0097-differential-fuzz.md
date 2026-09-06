# Batch 0097 — a differential fuzzer for the overlap lowerings

Branch: `batch/0097-differential-fuzz`, from main at `b2e2e267`.
Deliverables: `research/experiments/differential-fuzz/` (a native Rust
generator, oracle, minimizer, and campaign driver behind its own Makefile,
deliberately outside `make check`), the campaign report, and this record.
Approval classes: no specification change, no conformance change, no new
root entry, no compiler change.

## Why this exists

Three times in one day an implementer's "no observable change" claim was
refuted only because a reviewer hand-wrote a program the corpus did not
contain (`docs/done/0085-io-correctness.md`, `0087-semantic-check-memory.md`,
`0091-par3-judgment.md`). The suite was green through all three. A passing test
may test nothing, and a failure-hunting process is blind to every problem that
manifests as a pass.

The gap that closes is not "more tests". It is a *mechanical source of programs
nobody thought of*, judged by a property that does not depend on anyone's
opinion of what the program should print.

## The property, and why it makes an exact oracle

All three permissions fix the same observable. [PAR-1] states it, and [PAR-3]
repeats it verbatim, in `spec/kernel-spec.md`:

```text
Under a permitted overlap, bindings and every Whitefoot state place equal the
source-order result.
```

[PAR-2] states the same guarantee in its own unit — "every state-place
observable is the one produced by executing L's iterations in index order" —
and [PAR-1] closes the loop on what the implementation may not leak:

```text
The number of workers, the identity of the host thread that executes a
statement, the schedule, and whether an overlap was performed at all are not
observable, and no rule of this specification is stated in terms of them.
```

Those two sentences fix the observables exactly, so the oracle needs no model of
the language. The reference is the same program compiled with `--no-overlap`,
whose execution *is* the source order. The shipped completion build and the `--par`
build must publish the same stdout bytes, the same stderr bytes, and the same
exit status, under every `WF_WORKERS` × `WF_IO_HELPERS` setting. A difference is
a defect in exactly one of four places, and the classification is mechanical:

| class | what it means |
| --- | --- |
| permission widening | the judgment granted an overlap the rule does not admit |
| runtime race | a permitted overlap was executed unsafely |
| lowering | the emitted code computes a different value than the sequential one |
| harness | the difference is the fuzzer's, not the compiler's |

The fourth row is not decoration. Both findings of the first smoke run were in
it, and the section below says exactly how.

## What the generator writes

`src/generator.rs` emits accepted Whitefoot command programs from the [GRAM-4]
statement fence and the [GRAM-5] expression fence under a typing and ownership
environment: which scalars and buffers are live and how long each buffer is,
which entry inputs exist, which regions are open, how deep the nesting goes.

Free derivation over the fence is not an option here and the file says so. A
language whose acceptance is a borrow, effect, and domain judgment rejects
essentially every randomly derived sentence, and a fuzzer whose programs are
rejected tests the parser and nothing else. So the environment carries the
facts that make a program *canonical* rather than merely parseable:

- a subscript is either a constant below a length `buffer_new` established, or
  the binder of a `for` whose upper endpoint is exactly that length (P11), or
  guarded by a written `ilt(position, len(store))`;
- every divisor is a nonzero literal;
- every borrow is taken inside a region the generator opened;
- every affine value — every `FilePermit` — is consumed exactly once;
- the entry's effect row is *computed from what the body exhibited*, never
  guessed: `command.cwd` carries `writes(cwd)` whenever it is declared, because
  the entry's normal return edge performs the directory state's compiler-derived
  close, while an unused `Output`, `Args`, or `FileFactory` contributes nothing.

Acceptance is still verified by the compiler on every program, never assumed.

The shape catalog leans deliberately toward what the three permissions can grant
*and* toward their exact boundaries, because a permission that is never granted
and a permission that is wrongly granted are both invisible to a generator that
writes only the easy middle:

| shape | what it puts under test |
| --- | --- |
| `independent-pair` | two `write_once` on the two distinct `Output` values — the pair [PAR-1] grants |
| `same-output-pair` | two `write_once` through one `Output` — one exclusive loan against the other; the published order is the source order |
| `shared-source-pair` | two publications reading one buffer — two shared loans meeting on one place |
| `pure-call-pair` | two adjacent pure user calls, the cheapest permitted pair |
| `accumulator-loop` | the one loop shape [PAR-2] grants: one outside place, one fixed associative-commutative operation with an identity, no other occurrence of the binding in the body |
| `file-loop-iteration-own` | P15's shape: per-iteration name and destination, factory reserved in the prologue, accumulator written as an ordinary source-order `set` |
| `file-loop-hoisted-scratch` | the destination hoisted above the loop — [PAR-3] condition 3 |
| `file-loop-break-after-submission` | a `break` in the remainder — [PAR-3] condition 2 |
| `file-loop-shared-scratch` | the name buffer written across iterations |
| `read-then-write-buffer` | a publication that reads the buffer a positioned read wrote |
| `directory-scan` | `open_directory_source` plus a `directory_next` batch loop |
| `claim` | an always-true residual claim in the shape [CLM-2] admits, so the trap path is never taken |
| `bulk-write` | more than one host pipe buffer of bytes, so a delayed reader makes the write genuinely wait |
| `slice-view` | a direct view (P10) over a live buffer, moved into a helper that reads through it — the descriptor's finite static origin set is what the permission judgment must form the footprint from |
| `typed-exit` | a failure arm that leaves through an exit status |
| `give-match`, `branch`, `counted-loop`, `unbounded-loop`, `nested-loop`, `arithmetic`, `argument-read` | ordinary control and value forms |

Every program ends by rendering the accumulated `total` as a fixed twenty-digit
line and leaving through an exit status derived from the same `total`. Both
observables therefore depend on every statement the program executed, which is
what makes byte equality a real oracle instead of a check that two runs both
printed nothing.

## What the oracle does with one program

`src/oracle.rs`:

1. compiles the source three ways — `--no-overlap`, the shipped default, and
   `--par` — and reads the permission ledger off the default build;
2. runs the sequential build twice, at `WF_WORKERS=0 WF_IO_HELPERS=0` and at
   `4`/`4`, and requires the two to agree. A program that is not its own stable
   oracle is *discarded and counted*, never reported. This is the guard that
   keeps host-ordered facts — directory batch boundaries, short reads — from
   manufacturing findings;
3. runs the completion build across the whole `WF_WORKERS {0,1,2,4}` ×
   `WF_IO_HELPERS {0,1,4}` matrix with repetitions, and the `--par` build across
   the worker axis at both ends of the helper axis, because `WF_WORKERS` is read
   only by `par_runtime.c` and `WF_IO_HELPERS` only by the completion bridge;
4. for a program that publishes more than one pipe buffer, runs it again with
   stdout on a FIFO whose reader sleeps before draining, so the publication
   genuinely suspends inside the runtime instead of completing inline;
5. on any disagreement, re-runs the reference three times (a reference that has
   stopped agreeing with itself reclassifies the program as unstable) and then
   the differing configuration three times. A difference that survives is a
   finding; one that does not is still reported, tagged `intermittent`.

`src/minimize.rs` then delta-debugs the source by chunk-wise line removal, with
the same judgment as the validity oracle: a candidate survives only if it still
compiles under all three lowerings, is still its own stable oracle, and still
diverges. Removing a line usually unbalances a brace or drops a binding a later
line reads, the compiler rejects that candidate, and the chunk is kept — which
is what makes brace-blind line removal safe without a parser of our own.

### The oracle is not vacuously green

"Green is not coverage" applies to the fuzzer itself. Two independent
demonstrations that this one detects what it claims to detect:

- **A real end-to-end catch.** The first smoke campaign reported two
  divergences, minimized both, and saved them. They were harness defects (below),
  but the detection path — compile, run the matrix, compare bytes, re-verify,
  delta-debug, save — ran end to end on real output.
- **An injected miscompilation.** A one-shot wrapper standing in for
  `whitefootc` compiled a one-byte-different source whenever the *completion*
  build was requested (`[20_u64] = 10_u8` became `11_u8`, the line terminator of
  the digest). The oracle reported it immediately and attributed it correctly:

```text
DIVERGED: completion under WF_WORKERS=0 WF_IO_HELPERS=0 (captured pipe):
  stdout differs -- reference 21 bytes: 11489073398875347405\x0a
                  / observed  21 bytes: 11489073398875347405\x0b
```

The wrapper was a scratch one-shot and is not in the repository.

## The two harness findings, and why they mattered

The first smoke campaign (20 programs) reported two divergences, both under
`--par` at `WF_WORKERS=0`. `par_runtime.c:633` reads that setting as *fewer than
two lanes*, so the parallel runtime started no worker at all — a deterministic
difference with no thread in sight, which is the signature of a lowering defect
rather than a race. Reproducing the minimized program by hand gave the real
answer:

```text
m13seq   exit=146
m13comp  exit=148
m13par   exit=146
```

The programs read `arg_get(position: 0_u64)` and folded `host_bytes_len` of the
result into the digest. Argument zero is the program's own path, and the oracle
runs the same program from three different files — one per lowering. The three
names were `seed-13-no-overlap`, `seed-13-completion`, and `seed-13-par`: the
first two are the same length and agreed, the third is shorter and did not. The
compiler was never involved.

Two changes close the class, both in the same commit
(`fuzz: keep argument zero out of the comparison`):

- generated programs read only argument positions one and two, the literal
  arguments the oracle itself passes, identical in every run it compares;
- the three build file names are now the same length (`-a`, `-b`, `-c`), so even
  the length of argument zero is out of the comparison.

The smoke campaign re-ran clean. The finding is worth recording rather than
quietly fixing, for two reasons. It is the empirical case for keeping *harness*
in the classification table instead of treating every divergence as a compiler
defect. And it is a real constraint on this oracle that a future shape must
respect: **any invocation datum the harness rather than the compiler chooses
cannot appear in a generated program's observables.**

## The rejection tally, and the one shape it exposed

A generated program the compiler refuses is discarded and counted by the rule
its diagnostic cites. That tally is the generator's own bias made visible: a
rule that dominates it names a shape the generator writes wrong, or a language
rule that is broader than a writer would guess.

One rule accounts for all 63 of this campaign's rejections, and it is the second
kind. Recompiling the first 400 programs of the corpus by hand reproduced 19
rejections, and all 19 carry the same diagnostic kind, `NonLocalClaim`. Reducing
one lands on this pair, whose two members differ by a single line and receive
opposite verdicts:

```whitefoot
command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap), traps {
  doc "A publication whose failure arm leaves early, and a later claim about purely local state.";
  let banner = buffer_new(4_u64, 65_u8);
  region 'publish {
    match write_once<'publish, 'publish>(output: &uniq 'publish out, source: &'publish banner, start: 0_u64, end: 4_u64) {
      Ok(value: reached) => {
      }
      Err(error: failure) => {
        return exit_status(code: 57_u8);      // <-- the only difference
      }
    }
  }
  let table = buffer_new(64_u64, 65_u8);
  let seed = 3209_u64;
  let offset = seed % 64_u64;
  claim guard: ilt(offset, 64_u64) because "premises: ...";
  set table[offset] = 90_u8;
  return exit_status(code: 0_u8);
}
```

With the early return, the compiler refuses:

```text
whitefootc: Semantics/Source [CLM-1]: NonLocalClaim { name: "guard", carrier: "offset",
  boundary: SystemCall { declaration_ordinal: 9, operation: "write_once" },
  mechanical_fix: "use the system operation's specified fact or typed outcome, or branch
  on the returned value; do not claim an unstated system-result property" }
```

Replace that one line with an empty arm and the identical claim is accepted.

**This is not a compiler defect.** `spec/kernel-spec.md` is explicit:

```text
Claim authority deliberately includes control dependence although [PRV-1]
provenance does not.
When a `BoundaryResult` condition, match scrutinee or tag, counted endpoint, or
other selector chooses an edge, its witness joins every binder, delivered value,
or storage write whose reaching definition is selected by that edge, including
`value_if`, `value_match`, ordinary match, `give`, loop-carried updates, and
post-join state.
```

The scrutinee is a system-call result, the `Err` arm leaves, so every reaching
definition after the match is selected by the `Ok` edge, and "post-join state"
is named. The compiler follows the rule.

What the fuzzer found is therefore a **language consequence**, mechanically and
reproducibly: *a function that takes a typed exit on any I/O failure can write
no further claim about its own local arithmetic.* The writer's repair is to
restructure so the claim precedes the publication, or to drop the claim and
branch. Whether that is the intended reach of control dependence in claim
authority is a question for the owner and a candidate for a follow-up batch; it
is recorded here, not acted on, because acting on it would be a specification
change and this batch has none.

The generator keeps writing the shape. The rejection costs a few percent of the
compile budget and buys a standing measurement of exactly this boundary.

*Follow-up, 2026-08-28.* The owner answered the question this section left open
and ruled NARROW. Kernel specification v0.39 amends the quoted [CLM-1]
paragraph: a selector's witness joins a matching binder its arm introduces, a
`value_if` or `value_match` delivery, and the components a reconvergence, loop
head, or loop exit chooses between different reaching definitions — and nothing
else. The refused member of the pair above is now accepted, and the sentence
that named "post-join state" is gone. Everything this record says about the
v0.38 rule remains an accurate account of the rule it was written against.
Batch record: `docs/done/0102-clm1-narrow.md`.

## The campaign

One run, `make campaign PROGRAMS=2000 BUDGET=6600 JOBS=5 SEED=1 REPS=2`, at
`nice 19` on a shared macOS host whose load average moved between 13 and 68
while it ran, which is why 40 minutes of wall clock bought 2004 programs rather
than the 12 minutes the unloaded smoke rate predicted.

```text
== differential-fuzz campaign ==
first seed 1, 5 jobs, 2 repetitions, 40.1 minutes
attempts 2067, accepted 2004 (97.0%), rejected 63
agreed 2004, diverged 0, unstable 0, reference timeouts 0, lowering refusals 0
executions 78156 captured, 573 through a delayed fifo reader

rejections by cited rule
  CLM-1                63  3.0% of attempts

permission ledger over accepted programs
  PAR-1 pairs        1255 permitted,    932 denied
  PAR-2 loops         678 permitted,   2336 denied
  PAR-3 stages        857 permitted,    799 denied
  programs holding at least one permitted PAR-3 stage: 647 (32.3% of accepted)
  programs holding at least one permitted PAR-1 pair:  920 (45.9% of accepted)
```

Read that as coverage rather than as a green light:

- **2004 accepted programs, 78 156 executions, 573 of them through a delayed
  FIFO reader.** Every one of the 2004 was compiled three ways, established as
  its own stable oracle, and then run across the whole worker × helper matrix
  twice per cell.
- **Every permission was exercised on both sides.** 1255 permitted [PAR-1] pairs
  against 932 denied ones; 678 permitted [PAR-2] loops against 2336 denied;
  857 permitted [PAR-3] stages against 799 denied. Nearly a third of the
  accepted programs carry a *granted* staged loop and nearly half carry a
  granted pair, so this is not a corpus that only ever asked the judgment to say
  no.
- **`unstable 0` and `reference timeouts 0`.** Not one of the 2004 programs
  failed to agree with itself across two sequential runs at opposite ends of the
  environment matrix. The stability guard cost nothing here and remains armed.
- **`lowering refusals 0`.** No overlapping lowering refused source the
  sequential lowering accepted; acceptance is not a property of the lowering, and
  nothing in this corpus says otherwise.
- **`diverged 0`.** No program published different bytes or a different exit
  status under any overlap setting.
- **`reference crashes`, a count added in review, after this campaign.** The
  oracle above took a sequential run that ended by a signal (no exit status,
  no timeout) as the reference, so three builds dying the same way would have
  counted as `agreed`. That is a defect of its own class rather than an
  overlap divergence: an accepted program has one writer-reachable trap, a
  written claim, and the generated claims are always true. The reference run
  is now judged before it judges anything (`Judgment::ReferenceCrash`), the
  program is saved as a finding, and the campaign line carries the count. The
  40-minute campaign predates that count; the evidence at hand is a re-run
  through the amended oracle on the same host, 4 jobs, 1 repetition, seed 1:

  ```text
  attempts 208, accepted 203 (97.6%), rejected 5
  agreed 203, diverged 0, unstable 0, reference timeouts 0, reference crashes 0, lowering refusals 0
  executions 4263 captured, 51 through a delayed fifo reader
  ```

The shape distribution over accepted programs, which is what "diverged 0" is
worth:

```text
  accumulator-loop                      594  29.6%    give-match             399  19.9%
  arithmetic                            624  31.1%    independent-pair       383  19.1%
  branch                                593  29.6%    nested-loop            148   7.4%
  bulk-write                            191   9.5%    pure-call-pair         496  24.8%
  claim                                 401  20.0%    read-then-write-buffer 217  10.8%
  counted-loop                          594  29.6%    same-output-pair       551  27.5%
  directory-scan                        172   8.6%    shared-source-pair     198   9.9%
  file-loop-break-after-submission      149   7.4%    slice-view             458  22.9%
  file-loop-hoisted-scratch             243  12.1%    stderr-write           258  12.9%
  file-loop-iteration-own               647  32.3%    stdout-write           581  29.0%
  file-loop-shared-scratch              167   8.3%    typed-exit             192   9.6%
  argument-read                          89   4.4%    unbounded-loop         455  22.7%
```

**No defect in the compiler was found.** That is a real result and a bounded
one. It says the overlap path is sound over 2004 programs of these shapes on
this host — not that it is sound. The corpus has no threads it did not create,
no `io_uring`, no `arg_get` of argument zero, no arena, no generic
instantiation, and no program larger than a few hundred statements. What the
campaign *did* find, it found in the harness and in the language, and both are
recorded above.

The two findings this batch produced are therefore classified as follows.

| finding | class | disposition |
| --- | --- | --- |
| argument zero reaching a program's digest | harness | fixed in this batch; the class is closed by two changes |
| a claim refused after a system-outcome early exit | neither — spec-conformant | recorded for the owner; no probe, because there is nothing to reproduce that the compiler gets wrong |

`probes/` is therefore not created. There is nothing to put in it.

## Where it lives, and when it goes

`research/experiments/differential-fuzz/` — the existing home for measurement
and evidence bundles that are not gates. Six source files, one Makefile, one
README, no dependencies, no new root entry, and no reachability from
`make check`: the campaign builds thousands of executables and runs for tens of
minutes, and its verdict is a report rather than a pass/fail a build should
depend on.

```text
make -C research/experiments/differential-fuzz smoke      # 20 programs
make -C research/experiments/differential-fuzz campaign   # the full run
make -C research/experiments/differential-fuzz probes     # recorded findings
make -C research/experiments/differential-fuzz lint       # fmt + clippy
```

Everything generated — the fixture tree, the compiled programs, the raw output,
the findings — stays under the scratch root. Only the generator, the oracle, any
recorded probe, and this record are tracked.

Removal condition, stated in the README so it survives this document: the
directory goes when the campaign stops paying — when a full run over current
shapes finds nothing across two consecutive language or runtime changes to the
overlap path, and `probes/` is empty.

## The gate

`make check` on the merge candidate: green, once.

```text
conformance adapter: Pass=509  Skip=1
== WHITEFOOT ALL TESTS GREEN ==
```

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean on
the new crate, run through its own `make lint` target. The crate has no
dependencies and builds `--locked --offline`.

## Judgment calls

**"Grammar-driven" means the fence plus an environment, not free derivation.**
A uniformly random derivation over [GRAM-4] and [GRAM-5] is well-formed text
that the borrow, effect, and domain judgments reject essentially always. Such a
fuzzer measures the parser. The generator therefore derives from the same fence
but under a typing and ownership environment, and its bias is made visible
rather than hidden: every rejection is counted by the rule its diagnostic cites,
and that tally is in the report.

**The reference is the program's own sequential build, not a model.** Writing an
interpreter would give a second implementation to disagree with, and a second
implementation is a second thing to be wrong. The three permissions state
source-order equality themselves, so `--no-overlap` is the reference the rules
name.

**A program that is not stable is discarded, not reported.** Directory batch
boundaries and short reads are the host's. Requiring the sequential build to
agree with itself before the program is used as an oracle is what keeps those
out of the findings, and the count of discarded programs is reported so the
guard cannot silently swallow everything.

**One disagreement is not a finding.** The reference and the differing
configuration are each re-run three times. A difference that stops reproducing
is still reported, tagged `intermittent`, because an intermittent divergence is
still a divergence — but it is labelled so nobody reads it as deterministic.

**The `--par` lowering is in the matrix even though it is not what ships.**
`WF_WORKERS` is read only by `par_runtime.c`, so without `--par` the worker axis
of the matrix is inert. Including it costs a third compilation per program and
is the only way that axis means anything.

**`probes/` is created by the first finding, not in advance.** An empty
directory with a README explaining what it would contain if anything were in it
is rot. The `probes` target exists, degrades cleanly when nothing is recorded,
and the README says when the directory appears.

**The CLM-1 rejection is recorded, not repaired.** It is spec-conformant
behaviour with a real writer consequence. Changing it is a specification
question and this batch changes no specification.

## What this batch did not do

- **No compiler change of any kind.** Not a line. The only repository code added
  is the experiment, and nothing in `compiler/` reaches it.
- **No specification and no conformance change.** Approval classes: neither.
- **No new root entry.** The experiment lives in the existing
  `research/experiments/` home.
- **`make check` is untouched.** The campaign is not reachable from it and must
  not become reachable: it compiles and executes thousands of programs, and it
  reports rather than gates.
- **No cross-platform run.** Everything here is one macOS host. The completion
  adapter's Linux `io_uring` path is a different implementation of the same
  contract and this campaign has said nothing about it; the `linux` shape of
  this campaign is the obvious next step and needs a Linux host, not more code.
- **No `--par` actualization coverage beyond what the judgment grants today.**
  The generator writes the shapes the rules can permit; how much the backend
  actually overlaps is the backend's choice, and the ledger reports the
  judgment, not the actualization.
