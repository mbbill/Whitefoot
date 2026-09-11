# Check prompts

Each prompt reads two short things and answers one bounded question. Run
them with a mid-sized model. Their output is review input for the owner; it
never accepts or rejects a program.

## Design gate: run on a tree diff before implementation

G1. Decision test. For each added or changed node: does every `Decision:`
line name a choice and either a reason or a refused alternative? Report
lines that describe without deciding.

G2. Scope test. For each node whose scope begins with `all `: is every
instance the concept currently has listed with a status? Report instances
the tree or the code knows about that the list omits.

G3. Neighborhood test. For each added or changed node: read its parent, its
`Applies-to:` targets, and every node whose scope contains it. Does the new
text contradict any of them, or silently narrow one? Report each tension
with both node names.

## Correspondence: run on a diff pair at every commit and at pull request

Inputs: the tree diff since the last approval, the code diff since the last
check, and the existing tree nodes in the concept areas the code diff
touches. For a node changed near the root, the code input is everything in
the node's scope, not only the diff.

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
code implements it? Report nodes with no supporting code. An instance marked
`pending` and a scope marked `(new code only)` are exempt.

C5. Universal coverage. For each node whose scope begins with `all `: is
every instance marked `applied` actually compliant in the code, and is every
`pending` instance still pending rather than silently changed? Report
mismatches.

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
