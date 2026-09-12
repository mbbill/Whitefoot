# How a module audit is run

Every file in this directory is one module family of the compiler read
against the design tree. The direction is code to tree: find the choices the
code embodies that `design/` does not record, not the reverse. An audit is
reading and judgment only; nothing is compiled, deleted, or re-run for it, and
the auditor changes no file but its own audit.

## Read first

`design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`, every
node the module's own comments or subject touch, `design/recall-tmp/README.md`
and `sources.md`, and the finished audits in this directory as the format.

## Three classes, nothing else

1. **Covered by the tree.** The code does what a node's decision says. Cite
   the node and the decision.
2. **No decision needed.** Implementation any competent engineer would write
   the same way, or a mechanical consequence of the specification. One line
   each. Every such item is listed; the owner reads this list and may
   disagree, so nothing is silently dropped.
3. **Choices without a node.** A choice the owner would have to make for this
   project: a language rule, a compiler policy, a measured parameter, an
   architectural commitment with real alternatives. For each: what the code
   does with `file:line` anchors; the alternative it implicitly refuses; the
   reason, recovered from the sources where one exists; whether it reads as a
   deliberate choice, a likely defect, or a discrepancy with
   `spec/kernel-spec.md`; and a draft `Decision:` line in the tree's form
   (what, `because` reason, `instead of` alternative) for the owner to accept,
   edit, or strike.

A finding that contradicts a node or the specification is drift and is named
as such, whichever class it falls in.

## Sources for reasons

`git log -S` and `git blame` over the full history (the clone is complete back
to the 2026-07-07 root), the commit and pull-request indexes under
`design/recall-tmp/sources/`, the frozen `mcts_mem/` tree, the research records
under `research/investigations/` and `research/experiments/`, and `archive/`
as historical evidence. Quote a reason with its hash and date. "No reason
recorded" means the introducing commit and its successors say nothing.
Sources only help the owner remember; nothing enters a tree from an audit.

## Boundaries

The specification defines the language; the tree records decisions; code
defines neither. A historical plan or research proposal is not an implied
requirement. Tests are read as evidence of intended behavior, not audited, and
no audit proposes deleting one. The file is English only, its items are
numbered in bold as in the finished audits, and it ends with the three counts.
