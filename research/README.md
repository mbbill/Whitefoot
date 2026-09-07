# Research and design evidence

This directory holds investigations and experiments that expose language or
compiler needs and test possible solutions. The active
[specification](../spec/kernel-spec.md) defines the language, the
[compiler README](../compiler/README.md) describes the implementation, and
[AGENTS.md](../AGENTS.md) defines the work-branch and merge boundary.

- `investigations/`: a selected question's design, measurements, and rejected
  alternatives. Keep useful evidence here after implementation; an ended task
  does not require relocating it.
- `experiments/`: reproducible measurements and their inputs, comparisons,
  limitations, and maintained harnesses.
- `notes/` and existing documents at this directory's root: bounded design
  questions and supporting analysis.
- [Decision memory](../mcts_mem/): what was settled, why, and which attempts
  were replaced. Update the affected standing guidance when recording a change.
- [Archive promotion audit](archive-promotion-audit.md): a non-authoritative
  map from historical findings to useful successors and remaining questions.

[Compute runtime without I/O scheduling](investigations/compute-runtime/README.md)
recovers the original join/help/steal path and defines fresh native reference
comparisons. Its initial audit distinguishes the historical compute runtime
from the shared completion scheduler; implementation and timing status are
recorded there.

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
