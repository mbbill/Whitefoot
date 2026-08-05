- Assign every substantial task one immutable monotonically increasing number shared by planned, live, and terminal records.
- Optionally stage a task decomposed from an active plan as a claimable planned record; claiming moves the same numbered record into live coordination and binds its owner, workspace, and base revision, with the first claim to land on the integration branch winning.
- Admit a claim only when the planned record's listed dependencies are terminal or its cross-linked integration order explicitly permits the overlap.
- Keep a task record in live coordination while work remains and move the same numbered record to terminal history at disposition.
- Keep terminal records concise and frozen, with outcome, landed commits, canonical evidence, validation, and remaining dependency links.
- Keep task records non-authorizing and keep durable facts, measurements, decisions, approvals, and project status in their canonical owners.
- Delete unclaimed planned records when the plan they cite is replaced, unless the new active plan explicitly carries them; a deleted number never returns. Terminal history records only executed work.
- Resolve concurrent number collisions by renumbering the later integration before it lands.

## Facts

- 2026-08-05 owner rationale: retained numbered closure records make completed progress directly trackable by task, while concise contents and canonical-owner links prevent the history from becoming a second roadmap or design record. (sourced)
- 2026-08-05 owner rationale: a claimable planned stage lets the lead decompose one approved plan for executor fan-out with the git move as the atomic claim arbiter, reusing the existing number sequence and conflict resolution instead of a separate allocator or lock. (sourced)

## Moves

- 2026-08-05 (5ce43178) replaced [[delete-on-closure]]: deleting terminal coordination records made completed work hard to track; immutable task numbers plus concise frozen closure records preserve progress history without creating a second planning or design authority (sourced)
