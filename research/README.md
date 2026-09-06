# Project log and design records

This directory started as the research corpus and is now the project's
living record. The research-era material (debates, sources, matrices,
synthesis) moved to `../archive/research/`.

Research supplies facts to the living [`Direction Outline`](../docs/roadmap.md).
[`docs/roadmap.md`](../docs/roadmap.md) records current high-level
sequencing. Neither it nor a research note or experiment grants or withholds
permission to work on a branch.

- `archive-promotion-audit.md` — live, non-authoritative crosswalk of useful
  archived conclusions, their current successors, and unresolved promotion
  questions. The archived originals remain provenance.
- `../archive/governance/decision-log.md` — archived transition log and index
  of the versioned decision record. Historical entries may cite pre-rewrite
  commit hashes and pre-archive paths; those are labels, not links.
- `../mcts_mem/` — durable design choices, rejected alternatives, and evidence.
- `../archive/governance/directives.md` — original mixed owner-ruling record;
  historical only and not a source of current workflow.
- `notes/` — memos that pose or record design decisions (e.g. the STOR-1 pool
  question).
- The design drafts and hostile reviews that remain useful live at this
  directory's root or under `investigations/`. `experiments/` also retains
  reproducibility bundles; an old script inside such a bundle may name a retired
  compiler and is historical evidence, not an active repository tool. Completed
  or superseded design corpora live under `../archive/research/`.

Retained records redact machine-specific locations. `<repository-root>`,
`<historical-repository-root>`, `<scratch-root>`, `<local-home>`, and
`<local-workdir>` preserve the role of a path without publishing a developer's
absolute directory. Where a reproducibility bundle hashes a redacted record,
the bundle records the digest of the redacted bytes.
