# Batch 0103 — quiet under `--no-overlap`, and the explicit buffer form

Branch: `batch/0103-quiet-no-overlap`, from `integration/2026-08-28b` at
`16228216`. Deliverables: the narrowed default note channel with its test, the
`docs/patterns.md` P18 entry, and this record.

Two owner decisions, taken on the two observations batch 0100 recorded and did
not answer: B5 (a successful build has no quiet form) and the writer-facing
half of D1 (one output write per iteration denies the staged permission).

## 1. `--no-overlap` prints no denied-I/O-loop note (B5)

Since batch 0099 every ordinary compile prints each denied I/O-loop verdict to
stderr as `whitefootc: note: PAR …`, plus one closing line. The third blind
writer's program printed nine of them on every rebuild, including the rebuilds
that had already said they wanted no overlap at all.

The decision is by the flag, not by the text of any line: a build that wrote
`--no-overlap` has stated that this build takes no overlap lowering, so a loop
without a pipeline is the build it asked for rather than news about the
program. `--par-ledger` is untouched and prints the whole report under every
lowering, `--no-overlap` included.

`compiler/src/bin/whitefootc.rs` grows one function, `io_notice_report`, which
returns the lines to print and returns none of them under the flag; `run` now
prints what it returns. `Options::overlap` was lifted out of `run` in the same
edit so a test can select a lowering the way the binary does.

Before and after, on a source whose loop writes to `command.stdout` per
iteration:

```text
before  default      3 notes + 1 closing line
        --par        3 notes + 1 closing line
        --no-overlap 3 notes + 1 closing line
after   default      3 notes + 1 closing line
        --par        3 notes + 1 closing line
        --no-overlap nothing
        --no-overlap --par-ledger  the complete report, unchanged
```

The judgment is pure and none of it moved: the three ways reach the same
verdicts and, because a denied loop reaches the host through ordinary direct
calls under every lowering, they emit one module, byte for byte. Verified on
the branch compiler — the same source compiled the three ways before the change
and the same three plus `--no-overlap --par-ledger` after it, and all seven
`.ll` files hash `9ac52751`.

`tests::a_no_overlap_build_reports_no_denied_io_loop` in
`compiler/src/bin/whitefootc.rs` compiles one denied-output-loop source the
three ways through `Options::overlap`, asserts the notes are present by default
and under `--par` and absent under `--no-overlap`, asserts the three modules
and the three verdict lists are identical, and asserts the full report still
carries every line the quiet build withheld. Removing the flag from the guard
fails it.

## 2. P18, the loop's own buffer, published once

D1's runtime half — a final-stage `write_once` under in-order commit, so the
write can stay where the line is produced — belongs to batch 0095's Stage B and
is not this batch. The writer-facing half does not wait for it:
`docs/patterns.md` P18 teaches the explicit form. Whenever a writer would hold
a `&uniq` resource inside a loop body, the loop holds a buffer instead, each
iteration folds its line into it with an element `set`, and one write after the
loop publishes it.

The entry carries the denied form with its byte-exact condition-3 ledger line —
the compiler's own remedy text already names this rewrite — and the buffered
form with its `permitted` line, both compiled with the branch compiler before
being written. It also records what the measurement found on the way: writing
into that buffer through a helper that takes `&uniq` of it is denied by
condition 4 instead, so the page is written by element. No probe file:line is
cited, because the probes are not in the repository; `‹loop›` stands for the
writer's own file and line, as P15 already does.

P15's channel paragraph, `compiler/README.md`'s `--no-overlap` paragraph,
`docs/done/0099-writer-defaults.md` item 3, and `docs/done/0100-writer-defaults-2.md`'s
B5 and D1 bullets are updated in place to state the narrowed channel and to
point at this record.

## Approval classes

No specification change: `spec/kernel-spec.md` is untouched. No conformance
evidence change: no case, manifest, adapter, runner, or collection wiring is
added, modified, deleted, or renamed. The added test is an ordinary compiler
test.
