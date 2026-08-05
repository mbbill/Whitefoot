- Every unproved D1-critical checkable fact carries a runtime check in every build mode; there is no debug/release semantic divergence.
- A check is removed only by a deterministic machine-verified proof; a solver may promote performance facts but never licenses elision; nothing writer-stated is trusted unchecked.
- No writer-accessible syntax removes, weakens, or silences a check; explicit `check` statements are never elided, even when tautological.
- The active safe-Rust compiler now reaches semantic and ownership checking,
  exact memory-effect checking, a private checked program, target-independent
  typed control-flow IR, target qualification, conservative LLVM, and host
  execution. It retains required runtime checks and implements no proof-driven
  check elision or effect-derived LLVM attributes.
- The archived democ PROOF-1 implementation and accounting reports are historical evidence for a later optimizer experiment, not live compiler capability or acceptance authority.

## Facts

- 2026-08-04 code correction: the former live summary saying the compiler ended
  at name resolution had become stale. `compiler/README.md` and the active
  safe-Rust implementation now establish the semantic-checking, checked-IR,
  LLVM, and executable path summarized above; proof-driven check elision remains
  absent. This correction changes compiler status, not the proof doctrine.
  (code)
- 2026-07-10 measurement: the elision ceiling on base64 encode — removing every bounds check took the kernel from 2.44 to 4.2 GB/s (1.7x), branches 41 to 9, byte-identical outputs, and still zero SIMD (the shuffle algorithm is not vectorizer-discoverable); the ceiling justified building the proof tier, and its value is scalar. (sourced)
- 2026-07-10 statement: provably-in-range trapping reductions stay scalar and the base64 hot loop retained ~18 bounds branches blocking vectorization — recorded as the evidence that earned the tier before any code was written; the doctrine answer is that writers keep `.trap` and the compiler earns the speed via proof, never by pushing writers to `.wrap`. (sourced)
- 2026-07-10 statement: the `rem = len - i` derived-range proof is sound only because the induction variable provably starts at zero and its sole mutation is the exact stride increment; the guard alone is unsound under unsigned wrap. (code)
- 2026-07-11 statement: the accounting review's governing principle — at the performance gate, unresolved accounting resolves to failure, never to credit (reject-when-unsure applied to accounting); and an unused explicit check must never be a hard failure, because that would incentivize deleting defensive checks, the one incentive this language exists to make impossible. (sourced)
- 2026-07-11 statement: guard versioning was kept out of the accounting slice as the cheapest-route-to-credit hazard, with the measured 17x code-size delta of Rust's versioned loops cited as the warning precedent. (sourced)
- 2026-07-15 proposed research architecture pending owner review: ordinary opaque modules may submit producer-generated resource-protocol proofs to one fixed reference verifier; proof search, AI generation, solvers, optimizers, and derived proof rules remain untrusted, and an accepted derived rule must prove implication to the fixed policy rather than install an axiom. Only an irreducible new machine, helper, lowering, ABI, OS, device, foreign, proof-axiom, or fact-schema edge crosses the separately authenticated privilege gate. A `CERTIFIED` result requires refinement or checked validation through optimization, code generation, linking, relocation, provider resolution, loading, and the exact immutable loaded image; determinism or a pre-link proof is insufficient. This is a paper candidate with exact D-2 and P-1 still pending, not a production fact channel or checker implementation. (sourced)
- 2026-07-20 specification gap: DIAG-3 makes every check-report field mandatory and defines `proof_ref` as a checker-derivation identifier for eliminated checks, but gives no required value for a retained check. The report codec and artifact schema cannot close that field by convention. (code)
