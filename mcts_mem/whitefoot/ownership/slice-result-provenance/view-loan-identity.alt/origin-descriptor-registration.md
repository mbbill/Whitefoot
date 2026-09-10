- A view descriptor's checker metadata carries its data region and possible storage origins (SliceInfo).
- Loan records are separate entries keyed by data region, protected place and strength, with descriptor associations selected by matching storage places.
- Every view binding attaches to every existing loan whose protected place occurs in its origin set; no exact continuing claim or checked parent holder travels with the descriptor.

## Facts

- 2026-09-10 (59d21f83) pitfall: Descriptor registration compares only protected places, even though loan records distinguish region and strength. A new exclusive descriptor can consequently be attached to an older shared claim on the same storage and extend that claim's apparent liveness. (code)

## Moves

- 2026-09-10 (be12d98d) replaced by [[view-loan-identity]]: matching descriptor holders only by storage origin conflated distinct continuing loans and could not carry an ordinary borrowed holder's delegation through copies and returned views (sourced)
