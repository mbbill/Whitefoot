# Differential fuzzing of the overlap lowerings

## What this serves

Three times in one day an implementer's "no observable change" claim was
refuted only because a reviewer hand-wrote a program the test corpus did not
contain (`docs/done/0087-*`, `0091-*`, `0085-*`). A passing suite is not
coverage: it is blind to every program nobody thought to write. This experiment
is the mechanical source of programs nobody thought of.

The property under test is the one [PAR-1], [PAR-2], and [PAR-3] all state in
the same words: *bindings and every Whitefoot state place equal the source-order
result*, and *whether an overlap was performed at all is not observable*. That
makes a differential oracle exact rather than statistical. One program compiled
with `--no-overlap` is the source-order reference; the same program compiled the
way it ships, and again with `--par`, must publish the same stdout bytes, the
same stderr bytes, and the same exit status under every worker and helper
setting. Any difference is a defect in exactly one of four places: the
permission judgment widened past what the rule admits, the runtime raced, the
lowering mistranslated, or the harness lied.

## What it does

`src/generator.rs` writes accepted Whitefoot command programs from the grammar
fence [GRAM-4, GRAM-5] under a typing and ownership environment, so the emitted
forms are grammar-derived rather than string-spliced: reads and writes on
stdout and stderr, opens and positioned reads over a fixture tree, directory
enumeration, counted and unbounded loops, matches with typed exits, user
functions carrying real effect rows, always-true claims, and buffers with
guarded and structurally-proved subscripts. It biases toward the shapes the
three permissions can admit — independent calls on disjoint places, iteration-own
scratch, accumulators written in source order — and toward their exact
boundaries: a shared destination buffer, a second write through one `Output`, a
`break` after the submission, scratch hoisted above the loop.

`src/oracle.rs` compiles each program three ways, runs the sequential build
twice to establish that the program is its own stable oracle, then runs the
overlapping builds across `WF_WORKERS` × `WF_IO_HELPERS` with repetitions, plus
sampled runs whose stdout is a FIFO with a delayed reader so a write genuinely
waits. `src/minimize.rs` delta-debugs any differing program down to the smallest
source that still compiles and still differs.

A generated program the compiler rejects is discarded and counted by the rule
its diagnostic cites, so generator bias stays visible instead of silent.

## Running it

    make -C research/experiments/differential-fuzz smoke      # 20 programs
    make -C research/experiments/differential-fuzz campaign   # the full run
    make -C research/experiments/differential-fuzz probes     # recorded findings
    make -C research/experiments/differential-fuzz lint

It is not part of `make check` and must not become part of it: it compiles and
executes thousands of programs, and it reports rather than gates.

## probes/

The directory appears with the first finding a campaign keeps: one minimized
`.wf` per finding, and a `probes/README.md` naming, for each, the seed, the
classification, the defect, and the outcome the probe produces today. `make
probes` prints the current outcome of each probe and the reader compares it
against that index -- the expected outcome is written down rather than encoded,
because a probe may record a rejection as readily as a divergence.

A probe whose outcome changes has had its defect fixed; it is then either
promoted into `compiler/src/backend/tests/` as an ordinary compiler test or
deleted in the same change. Probes are maintained, not archived. `make probes`
with no directory recorded says so and exits clean.

## Removal condition

This directory is removed when the campaign stops paying: when a full run over
current shapes finds nothing across two consecutive language or runtime changes
to the overlap path, and `probes/` is empty. Until then it is the only source of
programs the corpus does not contain, and the campaign report in
`docs/done/0097-differential-fuzz.md` records what each run covered.
