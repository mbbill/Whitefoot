Decision: Acquisition and close borrow an explicit HandleFactory exclusively for the whole ordinary call, spending one credit on successful acquisition, restoring it on refusal and returning it on the explicit close that releases the allocation, because this exposes accounting without implicit creator links, cancellation effects or early loan release, instead of hidden global capacity or one-shot permits with implicit cancellation.

Rejected:
- A reservation whose drop silently returns factory credit: rejected because the selected ordinary interface has empty opaque drop and explicit host-state writes; a later reservation API would need explicit consuming return and cancellation outcomes.
- Available factory capacity as a promise that host acquisition succeeds: rejected because other host activity and limits can still refuse an otherwise funded attempt, so refusal remains an ordinary outcome.
