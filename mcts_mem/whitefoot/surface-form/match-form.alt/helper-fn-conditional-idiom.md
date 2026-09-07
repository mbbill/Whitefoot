- Match was statement-only: no arm delivered a value, and a conditionally initialized value required extracting a helper function that returned from each arm.
- The worked example (EX-1) normatively baked this idiom in as the way to write conditional initialization.

## Facts

- 2026-07-07 statement: the form's recorded provenance was "preserve one arm shape" / "cheapest to specify", while the same critique record's weak-writer signal already favored value delivery — the disqualifier and its replacement's rationale were on file together before the re-decision. (sourced)

## Moves

- 2026-07-08 (e687100a) replaced by [[match-only-conditional]]: conditional initialization is the most common pattern an AI writer needs; the helper-function idiom's recorded provenance was the literal R3 disqualifier ("cheapest to specify") and value delivery via give removes a mechanical helper function per conditional value (sourced)
