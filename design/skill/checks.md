# Check prompts

Each prompt reads a bounded input and answers one question. Run them with a
mid-sized model. Their output is review input for the owner; it never
accepts or rejects a program.

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
its ancestor chain and the nodes in the same scope, then extend to whatever
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
