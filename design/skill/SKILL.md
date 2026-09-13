---
name: design-tree
description: Discuss and revise designs, keep the live design tree to owner-approved decisions, and run Design Correspondence Review (DCR). Use when designing a change, recording a decision, proposing a tree revision, or asked to run dcr.
---

# Design tree

A design tree records the decisions a project is built on: what was chosen,
because of what, instead of what. It is organized by concept, not by code
structure. It holds only owner-approved decisions; unapproved choices remain
amendments beside it. Git holds history; the log provides concise traceability.

Use project-appropriate locations and concept names for three roles:

- Live tree: one file per node, with children in a directory of the same name.
- Amendments: proposed additions, replacements, or retirements of decisions.
- Change log: one short entry per approved tree change, newest first.

## Node format

A node is a file named for the decision it owns, holding one or more
`Decision:` lines and an optional `Rejected:` list, without dates, standalone
facts, measurements, or progress. Cite evidence in a reason instead. Name
events by what happened, not by date. Each field occupies one line, with a
blank line between fields; list items directly follow their header.

A `Decision:` line states the choice, its reason after `because`, and the
alternative after `instead of`. At least one must be present; a line with
neither is a description, not a decision. Write for a reader who has not
seen the source record, expanding compressed terminology.

`Rejected:` lists refused alternatives as `- <alternative>: rejected because
<reason>`, one per line. Give a discriminating reason, and do not re-propose
an alternative without explaining what changed.

## What is a decision

A choice between viable alternatives is a decision, even when the selection
seems obvious. An implementation step with only one viable way needs no
record. Start coarse; the owner tunes the threshold when the tree grows too
fine or too thin.

## Keeping the tree lean

Apply three filters to every proposed tree revision:

1. Decision, not description. Remove `Decision:` lines without `because` or
   `instead of`.
2. Not derivable from code. Remove nodes that only restate an interface or
   implementation.
3. Normalize upward. State a shared rule once at its common ancestor instead
   of repeating it in children.

Every tree diff review reports node count, depth, and net change; the lint
prints them against the review base.

## Amendments

Only the owner's approval of a proposed revision permits changing the live
tree, including on a work branch. Approval may occur at any point in the work.
Until then, keep the proposal beside the tree and record autonomous choices
as amendments while continuing authorized implementation.

An amendment file starts with `Node: <tree path>`, then a blank line and the
node form. Identify any decision replaced or retired and explain why. Keep
amendments current so the owner can review the complete outstanding revision.

When approved, apply that revision, add its log entry, and remove the accepted
amendment. When rejected, add a log entry naming the node, what was proposed,
and why it was refused, remove the amendment, and adjust the design and
implementation to the ruling. Later revisions need their own approval.

## Log format

The log records every ruling on the tree, each approved revision and each
refused amendment, and anything else a later reader must be able to find.
Each entry has a `## <date> <title>` heading, a `Nodes:` line listing every
node touched or ruled on, and a concise `Summary:` paragraph with the
conclusion, its reasons, and the ruling it records, naming the discussion or
review that approved or refused it. Cite data, measurements, and evidence at
their source under the project's research record instead of reproducing
them. Git supplies the detailed history. When parallel branches add entries,
retain both, newest first.

## Workflow

Discuss the design with the owner and develop the implementation in one
continuous workflow, without required phases, separate design submissions,
or separate design commits. A PR may open before coding and carry the whole
discussion. Use documents, experiments, and changes to code, specifications,
or tests as needed, following project conventions.

Present a complete design for the agreed scope with its proposed tree
revision: mechanism, requirements, alternatives, evidence, uncertainty, and
exactly which decisions change and why. Scale the explanation to the work;
the comparison with the current tree is central to the owner's review.

Keep affected nodes and ancestors in context, extending as needed. Discuss
material discoveries with the owner when available. Otherwise choose a
reasonable solution, record its grounds as an amendment, and continue;
present outstanding revisions when the owner returns.

## Design Correspondence Review (DCR)

Requests to run `dcr` invoke the bidirectional tree/code review below. Also
run it before declaring a goal or agreed work complete, moving a draft PR to
ready, or presenting finished work as ready to merge. Opening a PR or
publishing progress does not trigger it; goal completion does, even on a
draft. These triggers are defined here; a project's completion review refers
to them rather than restating them. Reuse the project's completion review
when it covers these checks.

Use a separate, read-only reviewer that did not implement the change,
normally a small or mid-sized model with bounded inputs. It reads actual
artifacts and reports scope, revision, findings, evidence, and uncertainty.
The primary agent sends those results to the owner with its own assessment
and recommended next steps, keeping findings and commentary distinct. Await
the owner's direction before acting on the findings, including during
unattended work; DCR does not authorize fixes or tree changes. Recheck affected
items after directed changes, reusing unaffected review. Tests and merge
rules belong to the project.

## Lint

Use project structural validation. The bundled `lint.py` checks form, not
design quality. Its layout has one root node file and optional child
directory per concept, with `amendments/` and `log.md` beside the roots.
Supply the project's paths, live concept names, and review base; amendments
may propose new concepts. Tree changes since the base must appear in a new
log entry; pending amendments need none.

    python3 -B <skill-directory>/lint.py --root <design-directory> --trees <concept-name> --base <review-base>

## Design checks

Use these during discussion and DCR; discussion needs neither finished
implementation nor an independent gate at every exchange.

G1. Decision test. Check each added or changed node or amendment against
the node format and leanness filters. Report descriptions without decisions,
circular refusal reasons, and choices or grounds that require the source
record to understand.

G2. Consistency scan. Check changed nodes and amendments against ancestors
and siblings, extending to related decisions as needed. A change governing a
whole concept requires reading its subtree. Report nodes read and conflicts,
narrowings, or broken dependencies, naming both sides.

## Correspondence: design and implementation

Inputs: the agreed delivery scope, its design commitments including relevant
existing nodes and ancestors, proposed revisions, the complete work diff,
resulting artifacts, and validation. Do not limit review to the tree diff.
Here, code means whichever artifact implements a decision, including a
specification or configuration. Extend into affected consumers as needed.

C1. Decisions in code. For each changed region embodying a design choice,
name its node or amendment. Report unrecorded choices as missing amendments;
ordinary implementation steps need no record.

C2. Contradiction. Report code that contradicts a decision or implements a
refused alternative, naming any amendment proposing that change. Without an
amendment it is drift.

C3. Orphaned support. For deleted code, identify decisions that lose their
implementation. Report a retired approach missing its rejection rationale,
and rejected approaches still implemented.

C4. Missing or partial implementation. For each design commitment in scope,
including existing nodes and pending revisions, identify support for its
required behavior and conditions. Report missing or partial paths,
placeholders, and insufficient evidence; a related function alone is not
proof of completion. Exclude unrelated or explicitly deferred designs unless
the deferral contradicts the agreed scope or completion claim.

## Translation: run on request

Render the requested tree diff or subtree in the requested language, keeping
node names, paths, and code identifiers untranslated. Do not store the
translation in the repository; the tree is English only.
