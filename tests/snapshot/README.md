# The snapshot corpus

This corpus records what the current compiler does with 484 programs — a
snapshot of today's verdicts, nothing more. It is **not specification
evidence** and it lives **outside the conformance boundary**: no row here
states what the language requires, no row may be cited as a rule, and the
integrity obligations that govern `tests/conformance/` do not reach it. The
active specification at `spec/kernel-spec.md` decides what is correct; this
directory only remembers what happened.

Its job is catching regressions cheaply. The programs came from a 665-program
adversarial and writer-authored sweep across twelve language areas, which found
two real compiler defects. The cheapest way to keep that reach is to compile
the survivors on every gate run and notice when one of them changes verdict.

Only sweep programs whose author stated an expectation are here; a program with
no stated expectation records a verdict nobody predicted, which is not worth a
gate row. That excludes the sweep's known unsound accept, which is tracked
where a language claim belongs — the conformance case
`ent5-neg-callee-uniq-buffer-replace-kills-length`, which v0.45's [CALL-5]
closed and which now runs.

## What is here

- `index.tsv` — the single source of truth: one row per case, plus header
  comments describing the columns.
- `cases/<family>/<id>.wf` — the programs, one file per row. Every case is
  canonical Whitefoot source that reaches a *semantic* verdict: an accept, or a
  rejection from resolution or semantic checking. A program that cannot get
  past lexing, parsing, or the [FORM-2] canonical-source audit records nothing
  about the checker and does not belong here.

## The columns

| column | meaning |
| --- | --- |
| `id` | case name; the file is `cases/<family>/<id>.wf` |
| `family` | the sweep area the program came from |
| `verdict` | `accept` or `reject` — the only thing the gate compares |
| `rule` | the numbered rule the rejection cites; empty for an accept |
| `finder_expectation` | `accept` or `reject`, what the program's author expected |
| `agreement` | `yes` or `no`, whether the two above agree |
| `doc` | one line on what the case is; for a disagreement, what the author expected, what the compiler does, and why |

## The gate compares ACCEPT/REJECT only

`make snapshot-run` compiles each case in index order through the ordinary
compiler path — no linking, no execution — and compares `verdict` alone. The
`rule` column is reference only and is **never checked**. Which numbered rule a
rejection cites is a diagnostic choice, and pinning it here would turn every
diagnostic improvement into a corpus failure. If you want a cited rule pinned,
that is a conformance case, not a snapshot row.

Twenty-one rows carry `agreement = no`: the author expected an accept and this
compiler rejects. Each row's `doc` gives the rule that decides it and the
mechanism behind it — twelve are expectation errors; one
(`kills__writer-r2__06_chain_middle_replace`) is a program whose stated
expectation the compiler now meets in the part it was written to exercise, yet
which still rejects for an unrelated unproved product; and eight are programs
whose expectation the language itself moved under. Those eight hand a
`&uniq buffer<u8>` to a helper, and v0.45's [CALL-5] selects no transport for
such a parameter, so the call kills the caller's length whatever the callee
does — the shape that keeps a caller's measures is the exclusive view
[CALL-3]. They are kept because a wrong or superseded expectation is still a
fixed verdict worth watching.

## When a verdict flips

A flip is information, not a failure to route around. The driver names the
case, the recorded verdict, the reached verdict, and the first diagnostic line.

- If the flip is a compiler defect you just introduced, fix the compiler.
- If the flip is a **known defect being fixed**, it is welcome: update the row
  and say so. That is the corpus doing its job.
- Either way, the commit that changes a `verdict` cell must explain the flip in
  its message. A silently updated row is the one thing this corpus cannot
  survive.

Never delete a case or edit a verdict merely to get a green run.

## What B7c4b-1 moved

The retiring `buffer<T>` surface left this corpus in three ways.

**Fifty-two rows kept their source's meaning and their verdict.** A read-only
`buffer<T>` parameter became a `Slice<T>`, a written one became a
`&uniq MutSlice<T>`, and a `buffer_new` at a literal count became a run taken
from one bump extent reserved in the entry's own frame and viewed where the
program wrote through it.

**Seven rows moved from `reject` to `accept`, and the move is [CALL-3]
working.** `contracts__writer-r1__10_append_within_capacity`,
`indexing__writer-r1__subslice_copy`,
`indexing__writer-r2__01_offset_length_window_copy`,
`indexing__writer-r2__07_bucket_index_from_hash`,
`indexing__writer-r2__11_ring_buffer_wrap_read`,
`joins__writer-r1__r12_writes_join_common_bound_accept` and
`kills__writer-r1__kill03_buffer_write_loop_invariant` each recorded a
rejection whose cause was a caller's measure dying at a call through a
`&uniq buffer<T>`. A write through a view reaches the viewed range's element
storage and no measure of the origin place, so the measure now stands and the
program is accepted. Each row's `doc` says so. One further row,
`contracts__writer-r1__02_buffer_capacity_copy`, keeps its `reject` and moves
the rule it cites from `OP-4` to `FN-8` for the same reason: the missing
premise is now the caller's own capacity bound at the call rather than the
subscript.

**Seven rows were retired**, because their program cannot be written on the run
surface without changing what the row records:
`indexing__writer-r1__chunk_reader`,
`indexing__writer-r2__02_ring_buffer_slot_write`,
`kills__adversary-r1__k10_loop_buffer_write_kills_length_fact`,
`kills__writer-r1__kill12_replace_buffer_field_kill`,
`kills__writer-r2__08_callee_write_kills_caller_fact`,
`diagnostics__writer-r1__r03_allocation_fit_unproved` and
`diagnostics__writer-r2__r05_world_value_allocation_no_branch`. The first five
lend a run, or a struct holding one, through a `&uniq` parameter, which
[BLK-4] refuses; the hand-back or view restructure changes the kill each row
was written to record. The last two allocate at a runtime count in an entry
with no store, and their successor's entry row carries `command.heap`, which is
a different program. Their sources are deleted with their rows.

## The case sources

Case files are the sweep programs unchanged, except that each carries a leading
`doc` statement that is true of the file. Thirteen of them — the disagreeing
rows — had a `doc` asserting an expectation this compiler does not meet; those
lines now state the snapshot verdict and what the author expected instead. The
programs themselves were not touched.

## When to remove this directory

Delete it, wiring and all, when either becomes true:

- `tests/conformance/` covers these shapes, in which case the specification
  says what this corpus only remembers; or
- the corpus stops paying for itself — flips that are noise rather than
  signal, or a maintenance cost larger than the regressions it catches.

It is a snapshot, not an asset. Removing it needs no ceremony beyond the
commit message that explains why.
