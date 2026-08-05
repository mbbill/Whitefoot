- Assign every substantial task one immutable monotonically increasing number shared by live and terminal records.
- Keep a task record in live coordination while work remains and move the same numbered record to terminal history at disposition.
- Keep terminal records concise and frozen, with outcome, landed commits, canonical evidence, validation, and remaining dependency links.
- Keep task records non-authorizing and keep durable facts, measurements, decisions, approvals, and project status in their canonical owners.
- Resolve concurrent number collisions by renumbering the later integration before it lands.

## Facts

- 2026-08-05 owner rationale: retained numbered closure records make completed progress directly trackable by task, while concise contents and canonical-owner links prevent the history from becoming a second roadmap or design record. (sourced)

## Moves

- 2026-08-05 (5ce43178) replaced [[delete-on-closure]]: deleting terminal coordination records made completed work hard to track; immutable task numbers plus concise frozen closure records preserve progress history without creating a second planning or design authority (sourced)
