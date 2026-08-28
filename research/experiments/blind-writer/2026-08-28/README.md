# Blind-writer corpus, 2026-08-28

Seven Whitefoot programs written by a senior systems programmer with no prior
Whitefoot exposure, given only `spec/kernel-spec.md` (v0.38), `docs/patterns.md`,
the `whitefootc` gate binary, and `tests/programs/`. Compiler source,
`docs/done/`, and `research/` were withheld. `REPORT.md` is that writer's own
account, verbatim.

## Why this directory exists

The owner's rule of 2026-08-27: for every default an unguided writer writes
badly, the project must either change the compiler so the natural form is the
fast and correct form, or emit a warning and teach the form in
`docs/patterns.md`. Silence is not allowed. That rule needs a standing record
of what unguided writers actually write, kept next to the compiler that judges
it, so a later change can be checked against the same programs rather than
against a memory of them.

No existing home fits. `compiler/tests/programs/` is the curated corpus the
compiler is tested against and is written by people who know the answers;
`research/experiments/io-completion-bench/programs/` holds hand-shaped
benchmark programs whose whole point is that an expert widened them. This
bundle is the opposite of both: programs whose value is that nobody corrected
them.

It is removed when the language stops changing. Until then each blind-writer
trial adds one dated directory beside this one, and the batch record in
`docs/done/` carries the defect table that trial produced.

## Contents

- `REPORT.md` — the writer's report. Verbatim except for three lines naming
  the writer's scratch tree outside the repository, which the `make check`
  repository invariant forbids in tracked content; those are rewritten and the
  rewrite is marked in place. Its §9 reproduction block is the writer's own and
  assumes their tree; use the one below instead.
- `programs/` — the five utilities (`p1`–`p5`) and the writer's two
  `--par-ledger` probes, byte-identical to what the writer submitted.
- `probes/` — three programs the judge wrote to isolate a claim the report
  makes. `probe_c_helper_denied.wf` is `probe_a_staged_permitted.wf` with
  `reserve_file` and `open_file` factored into one helper and nothing else
  changed. `probe_d_reborrow_two_statements.wf` is the smallest [OWN-6]
  rejection. `probe_e_hoisted_length.wf` is `p5_two_outputs.wf` with both
  `len` bindings hoisted above the loop and above every write.
- `ledger/` — `whitefootc --par-ledger` output for every program above, taken
  with the gate build of the compiler at this branch's base.

## Reproducing

    cd research/experiments/blind-writer/2026-08-28
    WFC=../../../../compiler/target/gate/whitefootc
    for f in programs/*.wf probes/*.wf; do "$WFC" --par-ledger -o /dev/null "$f"; done

Every program compiles except `probes/probe_d_reborrow_two_statements.wf`,
which exists to be rejected: it is the smallest `[OWN-6]` failure and exits 1.

Like `io-completion-bench/`, this bundle is deliberately not reachable from
`make check`: its value is the writer's uncorrected source and the verdicts a
given compiler gives it, and a gate that had to keep those verdicts green would
be a gate against changing the compiler. The batch record is its reader.

Name the source by a relative path. A source given by an absolute path is
labelled `input0.wf` in every diagnostic and every ledger line
(`compiler/src/bin/whitefootc.rs`, `logical_path`), which is defect D6 in the
batch record.
