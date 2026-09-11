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
- `design/skill/`: this procedure, with its check prompts, and the
  structural lint. It is project-independent and moves out of the repository
  once stable.

## Node format

A node is a file named for the decision it owns, holding one or more
`Decision:` lines and an optional `Rejected:` list.
Nothing else: no dates, no facts, no measurements, no task progress. A
measurement belongs in its results record; a node may cite it in its reason.
Every field is one line, every field is separated from the next by a blank
line so it renders as its own paragraph, and a list follows its header line
directly. The lint rejects anything outside the template.

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
line: `- <alternative>: rejected because <reason>`. A refusal without a
reason is not recorded. A later session must not re-propose a rejected
alternative without saying what changed.

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

Each entry is a `## <date> <title>` heading, a `Nodes:` line listing the
path of every node the change touched, and a `Summary:` paragraph with the
discussion's conclusion and reasons. Whether a node was added, changed, or
removed, and which commits implemented the entry, are found through git.

## Workflow

1. Discuss. The owner and the agent discuss the change.
2. Plan. Before any code or specification text, the agent writes the tree
   diff and its log entry. Every decision the agent adds on its own, without
   the owner having said it, is listed to the owner as such.
3. Design gate. The lint runs. The agent runs the design-gate checks
   below: the decision test and the consistency scan. The owner reads
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
   and the correspondence checks below over the pair of diffs, the tree
   diff and the code diff. The owner reads first the decisions the
   agent added on its own, each with the code that embodies it, then the
   findings, and approves or rejects each node; the agent changes code for
   every rejected one. Nothing merges with an unapproved node in the tree
   diff.

The owner reads tree diffs, log entries, and check reports. The owner does
not read code.

## Lint

`lint.py` checks form, not meaning; its messages say what it checks. Run
it from the gate with `--base <ref>` so that every tree change since
`<ref>` must be named in a new log entry.

    python3 -B design/skill/lint.py --base origin/main

## Checks

The prompts below are written for a mid-sized model reading a bounded input
and answering one question. Their findings are review input, never
acceptance authority.

## Design gate: run on a tree diff before implementation

G1. Decision test. For each added or changed node: does every `Decision:`
line name a choice and either a reason or a refused alternative, and does
every `Rejected:` line give a reason that would actually rule the
alternative out? Can a reader who has not seen the source record understand
the choice and the reason from the line alone? Report lines that describe
without deciding, refusals whose reason is a restatement, and lines that
need the record to be understood.

G2. Consistency scan. The tree is assumed consistent before the change;
only the change is checked against it. For each added or changed node, read
its ancestor chain and its neighbors under the same parent, then extend to whatever
else looks relevant. When the changed node is high in the tree or governs a
whole concept, read that whole subtree. Report the nodes read and every
conflict, narrowing, or broken dependency found, naming both nodes.

## Correspondence: run on a diff pair at pull request time

Inputs: the tree diff since the approved plan, the code diff of the pull
request, and the existing tree nodes in the concept areas the code diff
touches. For a language change the code is the specification.

C1. Justification. For each changed region of code (a new function, a
changed hunk inside a function, a moved or split function): which node
states the decision this change embodies? Report regions with no node.

C2. Delete test. For each region and the node it is matched to: if this
region were removed, would the node still be fully implemented? If yes, the
region is not justified by that node; report it as needing its own node or a
more specific parent.

C3. Orphaned support. For each deleted region: is the node it supported
still present and still claiming to be implemented? Report each such node.

C4. Unsupported node. For each node added or changed in the tree diff: which
code implements it? Report nodes with no supporting code.

Also report: any deleted function whose disappearance retires an approach,
when the tree gained no `Rejected:` line for it; and any new `Rejected:`
line whose approach still has code.

## Leanness filters: run on every tree diff

L1. Remove nodes whose decisions have neither `because` nor `instead of`.
L2. Remove nodes that restate what a signature, type, or effect row says.
L3. Move a rule repeated across siblings to their parent; state it once.
Report node count, maximum depth, and net change.

## Translation: run on request

Render the tree diff, or a named subtree, in the requested language for
review. Keep node names, paths, and code identifiers untranslated. Do not
store the rendering in the repository; the tree is English only.
