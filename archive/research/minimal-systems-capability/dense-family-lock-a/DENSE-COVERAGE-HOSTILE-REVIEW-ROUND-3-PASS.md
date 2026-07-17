# Dense Family Lock A Coverage Hostile Review: Round 3 Pass

Date: 2026-07-14. Status: `PASS_EXACT_BYTES_COVERAGE_ONLY`.

This review passes only the coverage and provenance layer on the exact bytes
listed below. It closes the six Round 2 coverage findings. It does not review
or approve candidate construction, candidate selection, execution, scoring,
held-out work, a language or specification decision, compiler or production
implementation, E0.1 restart, xlc migration, or default teaching. It is not a
Family Lock A approval and does not substitute for the remaining contract,
soundness, performance, and whole-lock reviews.

## 1. Reviewed local bytes

### 1.1 Sources and hostile tests

| Artifact | SHA-256 |
| --- | --- |
| `dense_literal_registry.py` | `a8eb255184ebf560f2fcd5eab659405b08185431a224cee69bfca9e32233cdc2` |
| `dense_coverage_closed_registry.py` | `84bc687641746607ba3798b8cf419f427ef4a4fe7b3a402e377287804f1024a3` |
| `dense_coverage_authority.py` | `f353f65815de96d0e0d3c198b22bb1707e3ee64a418e046ddf65f966d1e58d31` |
| `test_dense_coverage_authority.py` | `554613c73347dedc6dced8e850cf79881d3a7981828810ab45eabfef6ecb3c68` |

### 1.2 Generated authorities

| Artifact | SHA-256 |
| --- | --- |
| `DENSE-LOCAL-DECLARATIVE-INPUT-AUTHORITY.tsv` | `e7c0835510f220e86e9ea6b23bf52dde4d55cff142cd5ebd8ae586d76b4162bd` |
| `DENSE-FROZEN-G0-INPUT-AUTHORITY.tsv` | `95aaa6196c5e78506c7a9f4755c04bf9a8eea02aa669d30f3f6e9d1d672f3330` |
| `DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv` | `3879e7bf3896c466b38a46c719ae6acf0f6113aab3e4606cb4593bc0baace9a9` |
| `DENSE-EVIDENCE-TARGET-AUTHORITY.tsv` | `687aecfaa3ef122ee1a9270a97a4ab614e4693d62a32eb69e93a9a33ad56866e` |
| `DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv` | `34b1e14f611b7eee1f069bf9dbf31b0beab706fcb1dfa70b807e692b9dd53e2d` |
| `DENSE-OVERLAY-BRANCH-AUTHORITY.tsv` | `2d1d737501c75903fe564e46af334700703a61a00c1e7f1fac8f0058c4638738` |
| `DENSE-ROLE-UNIT-AUTHORITY.tsv` | `f949d1ca7f05d742eccb3264702591e1cc5ba559c4867c3b175796afe633f824` |
| `DENSE-CAPABILITY-UNIT-AUTHORITY.tsv` | `e58a922be6b9c917c69f813b29362d0cfed8e94b9a4709ae33a7d3ecaa774dd0` |

Any byte change to a source, test, or authority in these tables invalidates
this pass and requires regeneration, focused verification, and another
exact-hash hostile review.

## 2. Frozen G0 input bytes

The review independently resolved every row below through `git show` at
`a4de0eb70c345dcd198b11f435a5538ccc863113`, then compared the Git blob ID,
byte count, and SHA-256 with the frozen authority. All 12 matched.

| Frozen input | SHA-256 |
| --- | --- |
| `G0-CORE-ARTIFACT-MANIFEST.json` | `f0eced756688affef1732a133c43fb39ab6fc672334dca27b26129ddb5123719` |
| `G0-COVERAGE-EVIDENCE-UNIVERSE.tsv` | `521fbbe8d49bbe95f2b4d6d7c46122d74443291e6546db48e16569f506e72eff` |
| `G0-CLUSTER-FAMILY-ROUTING.tsv` | `f56f589580f1a8c25f718fc2efc1cd968f1dac8fb5a4fac51484736060a1308d` |
| `G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv` | `162e4af0fb8da0e3be306250e84727f949a28e5956435d61df57633825ab45d0` |
| `RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv` | `3e6a60450b62fdf0bc0794d10318566ca151f9e09a21d027fc016976d1acde50` |
| `RUST-DATA-SURFACE-MAP.tsv` | `5b20b78531332e79645a752235cc82f0dd600785807422c990ebf8cebecb90e4` |
| `RUST-D10-SURFACE-MAP.tsv` | `c486e491c159e48f7249c364f2b8c8a3bd96452c04fa7deebbd00433d131696d` |
| `RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv` | `c7e43770df4bb534ad02f9b2829b75a4f4c75a771135f427cf4269e30ee7d058` |
| `PAYLOAD-SCOPE-CLASSIFICATION.tsv` | `c54d6274f603057feca7bdb629fa8d1b4c701a39a91204d2622a8be22a3984b8` |
| `PAYLOAD-SCOPE-OVERLAY.tsv` | `d441b95ae44eb7386ce46b59284906450661ea0ef9f98e9af175cc2e7b5ca38e` |
| `G0-FAMILY-REQUIREMENT-REGISTRY.tsv` | `6e53a8dbbe24045c22448141022459b6bcaa200fa181018baf15c824437bc1a6` |
| `CAPABILITY-OBLIGATION-REGISTRY.tsv` | `e002960d0e315cf5f1f04e42c54decb4fc2b04ecb5c53d2911d7cf768fdfd541` |

## 3. Independent reconstruction

The hostile oracle did not import `dense_coverage_authority.py`,
`dense_coverage_closed_registry.py`, or any generated authority. It read only
the immutable G0 bytes through `git show`, independently implemented the
selector grammar and exact child applicability audit, and then compared its
canonical result with the generated authority.

The independent result was:

- 65 audit-domain clusters and 195 selector parents;
- 426 grammar children: 382 selector clauses, 35 helper canaries, and nine
  helper types;
- selector-expansion digest
  `f6f7ad98a163b5fc148caa0140414df0d6c6abe7ad63f32fb82f585212855c7f`;
- selector target/member-assignment digest
  `a7ae172df71295eb56faf67fea2bd64a130a3ca9624ea9b6a449b066cf0fb073`;
- canonical full selector oracle digest, including sorted exact anchors,
  `3c6631625da0e5ccae58a3dc5673976912ba6b82bc2425ab5fa41ddcee227d4e`;
- 130 multi-target selector children;
- 386 children with both F-DENSE applicability and at least one exact dense
  member, and 40 with neither; no child occupied either inconsistent quadrant;
- 456 direct G0 identities, comprising 117 stable-safe declarations, nine D10
  routes, 18 stable-unsafe evidence identities, and 312 concrete trait
  implementations; every identity was anchored exactly once, with no omission
  or duplicate anchor;
- the exact 30-identity `IntoIterator` direct set digest
  `c8f3b690219eb2355f17a1d04fac9390308e243141d004202750fecd0b65bde6`;
- the exact 25-role set digest
  `89a31fbed13799af4b55d4c3488018109a8dc0b7cc7ef964f2a5d8360cee4eed`;
  and
- the four-contract `ACTIVE_BR_STORED` set and binding digest
  `ceeb1aadae423b544d826c4dc91d32219ef6a310e01bc688124011868d02d270`.

The generated authority matched the canonical full selector oracle exactly.
The only ordering normalization was sorting anchor sets: 11 broad trait
selector rows preserve frozen G0 relation order in the generated TSV, while
the oracle deliberately sorts set members before hashing. This is a
presentation-order difference, not a semantic difference.

Additional hostile spot checks confirmed that BuildHasher clauses and canaries
route only to F-SPARSE plus their exact operation gate and carry no dense
member; Clone helper canaries retain their intrinsic family targets and bind
only exact clone-consuming dense members; `repeat_n` binds the repeated-Clone
post-source/final-move witness rather than a non-Clone sibling; and all nine
owning helper types retain their topology-specific routes.

## 4. Cross-topology raw-evidence adjudication

The final review separately adjudicated five stable-unsafe direct identities
whose underlying checked topology is not dense. The immutable G0 route makes
`GATE-RAW-SPELLING-REJECTION` applicable to each exact raw declaration. The
Box, Rc, or HashMap topology controls the independently expanded selector
child's family target; it does not erase the direct raw gate evidence contract.
Conversely, the raw route does not make F-DENSE applicable to these five
direct identities.

| Direct evidence identity | Rust surface | Exact direct target | Exact evidence member |
| --- | --- | --- | --- |
| `38b5efec11f68412855ba517c859b120afa2e1781334cb7aaa647904d277696a` | `Box::assume_init` scalar | `GATE-RAW-SPELLING-REJECTION` | `DENSE-INIT-AUTHORITY-EVIDENCE` |
| `ca9acc6bc5de258ca241de47a64ac7223cbe8dac9de59eadbc3b0769192efe9b` | `Box::assume_init` slice | `GATE-RAW-SPELLING-REJECTION` | `DENSE-INIT-AUTHORITY-EVIDENCE` |
| `0d4e2844a1ef5d364611baeae48ac2c044c2cd1f3903ba1e5d1289657eec6cdb` | `Rc::assume_init` scalar | `GATE-RAW-SPELLING-REJECTION` | `DENSE-INIT-AUTHORITY-EVIDENCE` |
| `91e4f0e7df38bfa017d6ec6d2d8b73ef7c3d4eb368ae38be178de136b105d405` | `Rc::assume_init` slice | `GATE-RAW-SPELLING-REJECTION` | `DENSE-INIT-AUTHORITY-EVIDENCE` |
| `7ca65e779a3a028234fe1fea8859af0abe861b1fedb47744719e1d922d44339d` | `HashMap::get_disjoint_unchecked_mut` | `GATE-RAW-SPELLING-REJECTION` | `DENSE-UNCHECKED-ACCESS-EVIDENCE` |

This disposition follows both sides of the G0 rule. Cluster unions never
create F-DENSE applicability for a different topology, while every applicable
real excluded declaration still binds an exact member and declarative excluded
outcome. A target-only synthetic terminal would repeat Round 2 finding 6.

## 5. Closed dependency and mutation review

The local semantic dependency set is closed by two exact-hash rows. The shared
loader accepts only a module docstring plus the exact required top-level
assignments and evaluates their values with `ast.literal_eval`; it does not
execute the registry. The hostile suite proved that merely updating the
approved registry hash cannot admit an expression.

All 22 hostile tests passed. The attacks covered:

- byte mutations of both the literal loader and the closed registry;
- an executable expression in a coherently hash-approved registry;
- omission of a direct identity;
- coherent direct target rerouting and direct member substitution;
- coherent selector target rerouting, member narrowing, and member
  substitution;
- mutation of every field in every frozen-input and local-input authority row;
- removal of real excluded-evidence outcomes;
- removal of active stored-borrow members and their excluded terminals;
- omission of the `O-ROPE-UNIQUE` terminal;
- role removal and substitution;
- removal and substitution of all six closure-sensitive capabilities;
- incomplete H-FLATSET A/B sealing; and
- collapse of distinct multi-target evidence outcomes.

The unmodified authorities validated. The read-only reconstruction command
also passed with the following exact totals:

```text
local_inputs=2
frozen_inputs=12
selector_children=426
target_terminals=1400
evidence_member_bindings=780
overlay_bindings=101
role_bindings=85
capability_bindings=1267
audit_clusters=65
```

## 6. Round 2 disposition and conclusion

Round 2's six coverage failures are closed on the reviewed bytes:

1. exhaustive per-child routing replaces cluster-wide selector inheritance;
2. all four active stored-borrow contracts have exact member bindings;
3. all 25 roles, including `O-ROPE-UNIQUE`, have explicit terminals;
4. generated authorities are compared with closed truth and an independent
   G0-only oracle rather than only with one another;
5. every local semantic dependency is hash-locked, and registry data is loaded
   without execution; and
6. every real evidence-backed exclusion has its exact declarative outcome
   binding, including the five cross-topology raw identities above.

Result: `PASS_EXACT_BYTES_COVERAGE_ONLY`. This pass permits the listed coverage
and provenance bytes to enter the remaining Family Lock A review. It grants no
construction, selection, execution, scoring, adoption, language-design, or
production authority.
