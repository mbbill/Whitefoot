---
name: design-tree
description: Maintain a project's live design tree and change log, gate implementation on approved tree diffs, and check code against the tree at pull-request time. Use when planning a change, recording a design decision, reviewing a tree diff, or checking that code and design still correspond.
---

# Design tree

A design tree records the decisions a project is built on: what was chosen,
because of what, instead of what. It is organized by concept, not by code
structure, so it survives refactors. It holds only live decisions; git holds
history and the change log holds the reasons for each change.

Three parts:

- `design/tree.md` and `design/tree/`: the live tree. One file per node; a
  node's children live in the directory with the node's name.
- `design/log.md`: one entry per approved tree change, newest first. It
  carries the discussion summary and the reasons. Nobody reads it routinely;
  it answers "why" when a node is questioned.
- `design/skill/`: this procedure, its templates, its check prompts, and the
  structural lint. It is project-independent and moves out of the repository
  once stable.

## Node format

See `templates/node.md`. A node has a title, one `Scope:` line directly
under it, one or more `Decision:` lines, and an optional `Rejected:` list.
Nothing else: no dates, no facts, no measurements, no task progress. A
measurement belongs in its results record; a node may cite it in its reason.
Every field is one line, every field is separated from the next by a blank
line so it renders as its own paragraph, and a list follows its header line
directly. The lint rejects anything outside the template.

`Scope:` comes first because it says what the node governs: which code and
which concepts a reviewer must look at beyond the diff when the node
changes.

A decision line states the choice, its reason after `because`, and the
alternative after `instead of`. At least one of the two must be present. A
line with neither is a description, not a decision, and does not belong in
the tree.

A decision is written for a reader who has not seen the record it came
from. Terms of art compressed from a memory node, a specification section,
or a compiler internal are expanded into plain words or replaced. A reader
who has to open the source to understand the reason has found a defect in
the node, not in their reading.

`Rejected:` lists alternatives that were considered and refused, one per
line: `- <alternative>: rejected because <reason>`, optionally ending with
`; lapses when <condition>`. A refusal without a reason is not recorded. A
later session must not re-propose a rejected alternative without naming a
lapsed reason.

## Keeping the tree lean

The tree is read in full by the reviewer and carried in full into the
writer's context. It must stay small. Apply three filters at every change:

1. Decision, not description. A node without `because` or `instead of` is
   documentation and is removed.
2. Not derivable from code. A node that says what a signature, effect row,
   or type already says is removed.
3. Normalize upward. A rule true for a whole concept is stated once at that
   concept's node and never repeated in children. Siblings that repeat each
   other move to the parent.

Every tree diff review reports node count, depth, and net change.

## Log format

See `templates/log-entry.md`. Each entry has a date and title, a `Nodes:`
line listing every added, changed, or removed node path, and a `Summary:`
paragraph with the discussion's conclusion and reasons. The commits that
implement an entry are found through git; the log does not repeat them.

## Workflow

1. Discuss. The owner and the agent discuss the change.
2. Plan. Before any code or specification text, the agent writes the tree
   diff and its log entry. Every decision the agent adds on its own, without
   the owner having said it, is listed to the owner as such.
3. Design gate. The lint runs. The agent runs the design-gate checks in
   `checks.md`: the decision test and the consistency scan. The owner reads
   the tree diff, translated on request, and the scan report, and approves
   or edits it. The approved tree diff and log entry are committed before
   implementation starts.
4. Implement. The agent keeps the subtree it is working in, plus the chain
   of ancestors to the root, in its context. A decision it has to make that
   the tree does not cover is written as a new node in the same branch; it
   is unapproved until the pull request is reviewed, like everything else
   in the pull request.
5. Pull request. The lint and the gate run. The agent runs the consistency
   scan on every node the branch added or changed since the approved plan
   and the correspondence checks in `checks.md` over the pair of diffs, the
   tree diff and the code diff. The owner reads first the decisions the
   agent added on its own, each with the code that embodies it, then the
   findings, and approves or rejects each node; the agent changes code for
   every rejected one. Nothing merges with an unapproved node in the tree
   diff.

The owner reads tree diffs, log entries, and check reports. The owner does
not read code.

## Checks

`checks.md` holds the prompts: the design-gate checks run on a tree diff
before implementation, the correspondence checks run on a diff pair at pull
request time, the leanness filters, and the translation prompt. They are
written for a mid-sized model reading a bounded input and answering one
question. Their findings are review input, never acceptance authority.

## Lint

`lint.py` checks form, not meaning, and is the one check that cannot be
skipped: run it from the gate. It verifies node structure and field order,
blank-line separation, decision and rejection markers, name uniqueness,
ASCII-only text, absence of history sections, log-entry structure, and,
with `--base <ref>`, that every tree change since `<ref>` is listed in a
new log entry. It reports node count, depth, and per-subtree counts.

    python3 -B design/skill/lint.py
    python3 -B design/skill/lint.py --base origin/main

## Bootstrapping from existing material

To build a tree from an earlier decision record or from code:

1. Take each existing node's live summary as candidate decisions. Apply the
   three filters. Rewrite every reason in plain words; do not copy a
   record's terms of art. Drop dated statements unless one is the only
   source of a live decision.
2. Turn each recorded rejected alternative into a `Rejected:` line with its
   reason.
3. Migrate one subtree first, review its size, adjust the filters, then
   migrate the rest.
4. Run the correspondence checks at module level over one code area to find
   decisions the code embodies that no node states. Each becomes a node for
   the owner to approve, or nothing.
5. Write one log entry naming the source and its revision.
6. Run the consistency scan over the whole tree once, to establish the base
   that later local scans assume.
