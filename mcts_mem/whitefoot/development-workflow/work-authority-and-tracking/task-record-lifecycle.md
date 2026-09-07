- The numbered planned/live/terminal task-record lifecycle is retired together with [[work-authority-and-tracking]]. Its dated evidence below describes the replaced protocol.
- Current work requires no task number, claimed record, rolling plan, or terminal archive. AGENTS.md owns the approval boundary; investigations hold designs and measurements, and mcts_mem holds settled decisions.

## Facts

- 2026-08-05 owner rationale: retained numbered closure records make completed progress directly trackable by task, while concise contents and canonical-owner links prevent the history from becoming a second roadmap or design record. (sourced)
- 2026-08-05 owner rationale: a claimable planned stage lets the lead decompose one approved plan for executor fan-out with the git move as the atomic claim arbiter, reusing the existing number sequence and conflict resolution instead of a separate allocator or lock. (sourced)

## Moves

- 2026-08-05 (5ce43178) replaced [[delete-on-closure]]: deleting terminal coordination records made completed work hard to track; immutable task numbers plus concise frozen closure records preserve progress history without creating a second planning or design authority (sourced)
