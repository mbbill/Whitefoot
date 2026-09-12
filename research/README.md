# Research and design evidence

This directory holds investigations and experiments that expose language or
compiler needs and test possible solutions. Treat agent authorship as a
changed design condition: identify what a candidate restriction or capability
should buy, compare plausible alternatives, and retain the observed limits.
The [evidence method](../docs/practice.md#evidence-guidance) separates design
objectives, assumptions, mechanism choices, and experimental results. The active
[specification](../spec/kernel-spec.md) defines the language, the
[design trees](../design/) record why the language and the compiler are the way
they are, and [AGENTS.md](../AGENTS.md) defines the work-branch and merge boundary.

- `investigations/`: a selected question's design, measurements, and rejected
  alternatives. Keep useful evidence here after implementation; an ended task
  does not require relocating it.
- `experiments/`: reproducible measurements and their inputs, comparisons,
  limitations, and maintained harnesses.
- `notes/` and existing documents at this directory's root: bounded design
  questions and supporting analysis.
- [Design trees](../design/): what was settled, why, and which alternatives
  were refused. A tree change is planned and approved before the code it governs.
- [Archive promotion audit](archive-promotion-audit.md): a non-authoritative
  map from historical findings to useful successors and remaining questions.
- [Decision workflow investigation](investigations/decision-workflow/DESIGN.md):
  external practices, the constitutional reassessment and index migration,
  and the evidence still needed to assess the selected workflow.
- [Prior compute-runtime bundle](investigations/compute-runtime/PRIOR-BUNDLE.md):
  the two-runtime comparison behind the 2026-09-09 current-stack selection, the
  grain panel and recursion-frontier evidence, and what the compute scoreboard
  replaced.

The [roadmap](../docs/roadmap.md) is reference material outside the working
loop. Research does not update its status or wait for it. Dated results state
what their recorded program, toolchain, and environment established; they are
not descriptions of current compiler capabilities. Historical approval or
phase language in evidence does not add current workflow requirements.

The root `make check` owns the maintained research test inventory. A deferred
prototype that depends on a retired compiler is evidence, not an executable
test target; its local README identifies that boundary. Active tools do not
import archived code. Keep experiments useful to an identifiable compiler
question and remove superseded material when it no longer carries evidence.

Retained records use role placeholders such as `<repository-root>` and
`<scratch-root>` instead of personal filesystem paths. When a reproducibility
bundle hashes a redacted record, its digest describes the redacted bytes.
