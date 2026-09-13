---
name: design-tree
description: Maintain a project's live design tree, its amendments, and its change log; keep the tree to what the owner has ruled; and check code against the tree at pull-request time. Use when planning a change, recording a design decision, proposing an amendment, reviewing a tree diff, or checking that code and design still correspond.
---

# Design tree

A design tree records the decisions a project is built on: what was chosen,
because of what, instead of what. It is organized by concept, not by code
structure, so it survives refactors. It holds only live decisions; git holds
history and the change log holds the reasons for each change. The tree is
the owner's: every line in it is something the owner has ruled, and a choice
an agent makes on its own stands beside the tree as an amendment until the
owner rules on it.

Four parts:

- `design/language.md` with `design/language/`, and `design/compiler.md`
  with `design/compiler/`: the two live trees, one for language decisions
  and one for compiler decisions. One file per node; a node's children live
  in the directory with the node's name. A language decision is checked
  against the specification, a compiler decision against the code.
- `design/amendments/`: one file per decision an agent made on its own, in
  the node form with a first line naming the node it amends or adds. An
  amendment is a proposal, not a decision, until the owner accepts it; the
  directory is empty when nothing is pending.
- `design/log.md`: one entry per approved tree change, newest first. It
  carries the discussion summary and the reasons. Nobody reads it routinely;
  it answers "why" when a node is questioned.
- `design/skill/`: this procedure, with its check prompts, and the
  structural lint.

## Node format

A node is a file named for the decision it owns, holding one or more
`Decision:` lines and an optional `Rejected:` list.
Nothing else: no dates, no facts, no measurements, no task progress. A
measurement belongs in its results record; a node may cite it in its reason.
An event is named by what it was, never by its date.
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

## What is a decision

A choice is a decision when more than one viable way exists, however plain
the pick: a balanced tree chosen over another tree with different tradeoffs
is a decision and is written down even when the choice is obvious. A step
with one viable way is not a decision and is written nowhere: a hash map
where nothing else would serve is just the code. The threshold starts coarse,
and the owner tunes it when the record grows too fine or too thin.

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

## Amendments

An agent never edits a node the owner has settled. When implementation needs
a decision the tree does not cover, or finds a decision of the tree that
cannot be followed, the agent chooses the reasonable option itself, writes it
as an amendment, and continues; it does not wait for the owner.

An amendment is a file `design/amendments/<name>.md` whose first line is
`Node: <tree path>`, naming the existing node it amends or the new node it
would add, followed by a blank line and the node form: `Decision:` lines and
an optional `Rejected:` list. A decision that replaces one the tree holds
says so in its reason. The lint checks the form and reports the count.

The owner rules on every amendment at the next review. Accepted: the agent
moves its lines into the node, writes the log entry, and deletes the file.
Rejected: the agent changes the code to the owner's ruling, or the work
stops there. Nothing merges with an open amendment.

## Log format

Each entry is a `## <date> <title>` heading, a `Nodes:` line listing the
path of every node the change touched, such as `language/checks-and-proofs`,
and a `Summary:` paragraph with the
discussion's conclusion and reasons. Whether a node was added, changed, or
removed, and which commits implemented the entry, are found through git.
Parallel branches both insert at the top and conflict there; keep both
entries, newest first.

## Workflow

1. Discuss. The owner and the agent discuss the change. The agent writes
   the conclusion as a tree diff and its log entry, the owner confirms it,
   and it is committed. This is the only way a line enters a tree. Work that
   starts without such a discussion starts with no tree diff, and every
   decision it needs is an amendment.
2. Implement. The agent keeps the subtree it is working in, plus the chain
   of ancestors to the root, in its context. It edits no node. A decision the
   tree does not cover, or one the tree gets wrong, becomes an amendment, and
   the work goes on. Before the pull request the agent runs the lint and the
   design-gate checks below over its amendments.
3. Pull request. The lint and the gate run. The agent runs the
   correspondence checks below over the tree, the amendments, and the code
   diff, and writes the report. The owner reads two things and nothing else:
   the tree revision, which is the tree diff since the last review together
   with every open amendment, and the correspondence report. The owner
   accepts or rejects each amendment and each finding; the agent moves an
   accepted amendment into the tree with a log entry and changes the code for
   a rejected one. Nothing merges with an open amendment or an unresolved
   finding.

The owner reads tree revisions, amendments, and the correspondence report.
The owner does not read code, and does not read a pull request beyond those.

## Lint

`lint.py` checks form, not meaning; its messages say what it checks. It
covers the nodes, the amendments, and the log. Run it from the gate with
`--base <ref>` so that every tree change since `<ref>` must be named in a new
log entry; an amendment needs no entry until it is accepted.

    python3 -B design/skill/lint.py --base origin/main

## Checks

The prompts below are written for a mid-sized model reading a bounded input
and answering one question. Their findings are review input, never
acceptance authority.

## Design gate: run on a tree diff or an amendment before the pull request

G1. Decision test. For each added or changed node or amendment: does every
`Decision:` line name a choice and either a reason or a refused alternative,
and does every `Rejected:` line give a reason that would actually rule the
alternative out? Can a reader who has not seen the source record understand
the choice and the reason from the line alone? Report lines that describe
without deciding, refusals whose reason is a restatement, and lines that
need the record to be understood.

G2. Consistency scan. The tree is assumed consistent before the change;
only the change is checked against it. For each added or changed node or
amendment, read its ancestor chain and its neighbors under the same parent,
then extend to whatever else looks relevant. When the changed node is high
in the tree or governs a whole concept, read that whole subtree. Report the
nodes read and every conflict, narrowing, or broken dependency found, naming
both nodes.

## Correspondence: run at pull request time

Inputs: the tree diff since the last review, the open amendments, the code
diff of the pull request, and the existing tree nodes in the concept areas
the code diff touches. For a language change the code is the specification.
Every check is reading and judgment: nothing is compiled, deleted, or re-run
for it, and CI owns the rest.

C1. Decisions in code. For each changed region of code (a new function, a
changed hunk inside a function, a moved or split function) that embodies a
decision under "What is a decision", name the node or amendment that
records it. A decision recorded nowhere is a missing amendment, and the
agent writes it before the review. A region with one viable way is not a
decision and is not listed.

C2. Contradiction. For each changed region: does it do what a node's
decision refuses, or what a `Rejected:` line names? Report each, with the
amendment that proposes the change when the agent meant it; without one it
is drift.

C3. Orphaned support. For each deleted region: is a node that it supported
still present and still claiming to be implemented? Report each such node.

C4. Unsupported node. For each node added or changed in the tree diff, and
each amendment: which code implements it? Report those with no supporting
code.

Also report: any deleted function whose disappearance retires an approach,
when the tree gained no `Rejected:` line for it; and any new `Rejected:`
line whose approach still has code.

## Leanness filters: run on every tree diff

Apply the three filters of "Keeping the tree lean" and report node count,
maximum depth, and net change.

## Translation: run on request

Render the tree diff, or a named subtree, in the requested language for
review. Keep node names, paths, and code identifiers untranslated. Do not
store the rendering in the repository; the tree is English only.
