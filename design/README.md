# Design tree

This directory holds Whitefoot's live design decisions and the procedure
that maintains them. It replaces `mcts_mem/`, which recorded the same
decisions with their full history and had no consumer that could fail.

- `tree.md`, `tree/`: the decisions, organized by concept. Only live
  decisions; a retired decision is deleted and git keeps its history.
- `log.md`: one entry per approved tree change with the discussion summary
  and reasons.
- `skill/`: the procedure (`SKILL.md`), templates, check prompts, and the
  structural lint. Project-independent; it moves out of the repository once
  stable.

Status: migration in progress. Migrated subtrees are listed in the first
`log.md` entry. Until every subtree is migrated, `mcts_mem/` remains the
record for the rest. When the migration completes, `mcts_mem/` is deleted
and the references in `AGENTS.md`, `CLAUDE.md`, `README.md`,
`docs/practice.md`, `docs/review-checklist.md`, `docs/roadmap.md`, and
`spec/derivation/derivation-ledger.md` move here.

Run the lint with `make design-lint`. The `--base` comparison needs the
base branch fetched.
