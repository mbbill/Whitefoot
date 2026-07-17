# Dense Family Lock A Contract and Soundness Hostile Review: Pass

Date: 2026-07-14. Status: `PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY`.

An independent reviewer examined the frozen contract and mathematical
soundness layer without editing its sources or generated registries. The
review independently checked the coverage-to-contract relation, owner-role and
candidate-binding products, candidate isolation, lifecycle rules, soundness
traces, optimizer-fact boundaries, and hostile mutations. No P1 or P2 finding
remains on the exact bytes below.

This is not a Family Lock A approval. It authorizes no candidate construction,
Candidate Freeze B, execution, scoring, held-out work, selection, language or
specification decision, compiler or production implementation, production fact
channel, E0.1 restart, xlc migration, or default teaching.

## 1. Reviewed local bytes

| Artifact | SHA-256 |
| --- | --- |
| `dense_owner_decisions.py` | `93f3d22ce02ea3b654177a80d1a46a50af4ed6e38bfee6f45dd4af23c72ed4bc` |
| `dense_contract_registry.py` | `31dda5ccfd33202860022946fdf456404b104a14118ffcd286406595e2da2d06` |
| `dense_soundness_oracle.py` | `9a750e9a01a3dc7329ed26393df1fbcad233509dc8cbc05079c1d317b81e68c2` |
| `dense_meta5.py` | `40c684507e1a5c3592da2ede792be2fb1bcc8679eb659de97d72ead92e6ae376` |
| `dense_literal_registry.py` | `a8eb255184ebf560f2fcd5eab659405b08185431a224cee69bfca9e32233cdc2` |
| `dense_coverage_closed_registry.py` | `84bc687641746607ba3798b8cf419f427ef4a4fe7b3a402e377287804f1024a3` |
| `DENSE-EVIDENCE-TARGET-AUTHORITY.tsv` | `687aecfaa3ef122ee1a9270a97a4ab614e4693d62a32eb69e93a9a33ad56866e` |
| `DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv` | `34b1e14f611b7eee1f069bf9dbf31b0beab706fcb1dfa70b807e692b9dd53e2d` |
| `DENSE-OVERLAY-BRANCH-AUTHORITY.tsv` | `2d1d737501c75903fe564e46af334700703a61a00c1e7f1fac8f0058c4638738` |
| `DENSE-CAPABILITY-UNIT-AUTHORITY.tsv` | `e58a922be6b9c917c69f813b29362d0cfed8e94b9a4709ae33a7d3ecaa774dd0` |
| `DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv` | `3879e7bf3896c466b38a46c719ae6acf0f6113aab3e4606cb4593bc0baace9a9` |
| `DENSE-ROLE-UNIT-AUTHORITY.tsv` | `f949d1ca7f05d742eccb3264702591e1cc5ba559c4867c3b175796afe633f824` |
| `DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv` | `3016206708a63b858b655e81de0e5c08e21055b0c4aa4c1ce37c2561c73e3418` |
| `DENSE-OWNER-ROLE-REGISTRY.tsv` | `c35f15e6f63b5e1067246aaa555329294b753a294009dc5c17154b54eadef971` |
| `DENSE-COMMON-SUBSTRATE-REGISTRY.tsv` | `99fa5fc0c0ad44033c360027a0b2d5caf2bdb65253013776995f21e145e28e3f` |
| `DENSE-STORED-BORROW-ROUTE-REGISTRY.tsv` | `6af02ae6c410d9ae08e802eb2733eeab80d94e905bf5593ab6ce29327fa9ddf8` |
| `DENSE-OD4-POLICY-REGISTRY.tsv` | `073c206be16fb9d85cfd7d90bd3743c21633b30ef1a93fd21828d3a1a5938bdd` |
| `DENSE-OD1-POLICY-REGISTRY.tsv` | `558e4d11dc9ba6d97f9f1de537e4f9f528167836f9e5a7b4036380dd7aad9379` |
| `DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv` | `63f6c1e0cad521ea718b40b9570857e2401a2e83e4d36d1d58785b7276a041df` |
| `DENSE-CANDIDATE-OPERATION-REGISTRY.tsv` | `e6309a06014fbc573974786e91597344d29c665b463bd4589553d1e8a55812bd` |
| `DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv` | `efb9336256340fd4b49177ba738fa3de1099f3e9f8157e77b6232979afee01d3` |
| `DENSE-CANDIDATE-DISTINCTION-REGISTRY.tsv` | `5b1a4fd41658cfd21f9330c79c8914ed9107aede3c5ffe619d5fa5216580c018` |
| `DENSE-ZST-POLICY-REGISTRY.tsv` | `7f8d3d4cd12e5e35fc773388458d028f306aba3f1a2445a0b08378737d765059` |
| `DENSE-FACT-CHANNEL-REGISTRY.tsv` | `8f6cb60bfb657a695168992e825cd75b45f2702b278d4755afa8b54b1843868f` |
| `DENSE-SYNTHETIC-UNIT-REGISTRY.tsv` | `d03491046d15c96f42e04c80a27c98b4547895fc687d3b929bcc53183d3a87a3` |
| `DENSE-MATHEMATICAL-SOUNDNESS-TRACES.jsonl` | `be4b755f44b244ba12338d96d483e3c7de5b282858b23beefc02ec32e95d1052` |
| `DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json` | `692f235334ef16798bc3722c67d3205364ab7033a0b5f8a110abc286d6f0a991` |

The manifest independently pins all 25 semantic dependencies and the trace
artifact, including the exact reviewed coverage registry and authorities. Any
byte change invalidates this pass.

## 2. Independent relation audit

The reviewer recomputed the joins rather than accepting generated row counts:

- 780 coverage evidence/member bindings resolve bidirectionally through 739
  unique real evidence triples plus exactly two declared protocol-synthetic
  members, across 97 cluster/member units;
- 303 exact member/outcome contracts and 303 owner-role rows form a bijection;
- 1,515 candidate bindings are exactly the five-candidate by 303-contract
  Cartesian product, with no missing, duplicate, or extra pair;
- all 97 candidate adapter groups preserve identical owner, allocation, borrow,
  fact, and observable result contracts across all five arms; and
- all ten candidate pairs retain an explicit distinction whose collapse rule
  identifies when two mechanisms would cease to be distinct.

The two synthetic members are the declared eager extract and eager splice
protocol units. No real excluded Rust evidence is laundered through synthetic
authority.

## 3. Lifecycle and ownership audit

The five candidates differ only in operation-local partial-transition handling.
Every arm uses the same conditional OD-0 common substrate and the same affine
single-live-interval carrier for owning traversal. The reviewer confirmed that
no proof, runtime-topology, repair, linear, or atomic arm substitutes a private
cursor, allocator, sealing route, provenance route, or ordinary-library
privilege.

The common interval carrier owns one master allocation and exactly one live
range `[front, back)`. An endpoint dies before its value is yielded.
Abandonment destroys exactly the remaining interval and releases the allocation
once. Zero-sized values use logical indices and exact logical drop counts, never
addresses, as identity. The carrier exposes no hole, mutation,
repair-to-Dense, second range, or arbitrary-liveness authority.

The candidate-specific states remain distinct and structurally droppable:

- atomic transitions expose no persistent partial owner;
- linear rebuild requires exact flow use and rejects an open normal exit;
- derived repair inserts compiler-derived repair on every normal-abandonment
  edge;
- proof-carrying state owns zero to two statically proved live ranges without a
  runtime topology tag; and
- runtime topology contains only sealed `Dense` or one-`Hole` state, never a
  bitmap, interval cursor state, third range, or hidden repair-to-Dense path.

All normal exits leave one complete valid owner. Recoverable failures precede
destructive commitment and return the exact required owners. Traps do not
unwind and therefore cannot serve as cleanup. A conventional `finish` call or
writer discipline cannot satisfy an abandonment obligation.

## 4. Policy, stored-state, and fact-channel audit

Both OD-1 failure policies are represented without selecting one. The OD-4
registry contains the exact three-way choice: eager-only, eager plus scoped
consume, or promoted persistent lazy cursors. The recommended scoped form is
nonescaping and nonreentrant, calls direct monomorphized behavior in exact
source order, uses O(1) auxiliary container state, allocates no removed-result
sequence unless the caller collects, and restores a complete dense owner before
every normal return, including early stop.

All four active `BR-STORED` routes retain exact root/leaf provenance through
move, call, result, destruction, and failure. Region-free instantiations add no
provenance fields or branches. Generic retained state cannot inherit this
zero-tax route.

The eight optimizer-fact channels each record an exact proposition, owner/root
and version, producer, scope, consumers, invalidators, transfer and join rules,
speculation boundary, facts-off behavior, surfaced artifact, and hostile
attacks. `LIVE-PREFIX` cannot be produced by a Hole or multi-range state. None
of these protocol rows grants production fact authority.

The checked allocation facade transfers exactly one allocation-owner token on
success and consumes it once on release. It grants no raw bytes, writer-set
liveness, unchecked capacity mutation, manual deallocation, or forged
root/version authority.

## 5. Executable oracle and hostile attacks

The executable oracle reproduced 2,002 deterministic traces:

- 1,515 primary candidate-contract traces: 750 normal, 555 pre-abort, 165
  recoverable-failure, and 45 static-rejection cases;
- 25 capacity-boundary and 25 ZST cases;
- 10 growth/root cases and 50 common-cursor cases;
- 20 stored-borrow positive cases and 80 stored-borrow attacks;
- 100 scoped-consumer cases and 35 scoped-consumer attacks;
- 120 fact-channel attacks; and
- 22 candidate-lifecycle attacks.

All 32 registered hostile artifact mutations failed closed. They cover contract,
owner-role, candidate-binding, owner-mint/loss, returned-owner, allocation-root,
post-state, proof/runtime/repair lifecycle, private-cursor, stale-fact,
fact-invalidator, live-prefix Hole, ZST allocation/drop, authorization,
sort/call order, executable-registry, and pinned-coverage attacks. The reviewer
also rejected three independently constructed coherent mutations: a forged
evidence identity, a candidate-private cursor, and a `LIVE-PREFIX` producer in
Hole state. The reviewer independently checked the owner equations, OD
branches, cursor abandonment, fact invalidation, and exact trace hashes rather
than treating generated TSVs as their own oracle.

The following commands passed on the listed bytes:

```text
python3 -B dense_contract_registry.py --check
python3 -B dense_soundness_oracle.py --check
make check
make -C compiler check
```

The root gate ended with `== ALL VERIFICATION LAYERS GREEN ==`.

## 6. Conclusion

Result: `PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY`. The exact contract and
mathematical soundness layer may enter the remaining performance and whole-lock
reviews. OD-0, OD-1, OD-3, and OD-4 remain unresolved owner decisions; no
recommended option is selected by this pass. Candidate construction and every
production-relevant action remain unauthorized.
