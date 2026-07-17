# Dense Family Lock A Coverage Hostile Review: Round 2 Failure

Date: 2026-07-14. Status: `FAILED_AND_REPAIR_REQUIRED`.

This review examined only the repaired coverage/provenance layer. It did not
review the contract resolver because the coverage authority failed first. It
authorizes no candidate construction, execution, scoring, language decision,
or production change.

## Reviewed bytes

| Artifact | SHA-256 |
| --- | --- |
| `dense_lock_model.py` | `3b9067c54d049f9841074788ecfd788cc73174d322f7b3f37036947f1d7753f0` |
| `dense_coverage_authority.py` | `556c2b8ef5d75a51d1ed57ec34ec3c5fbac61abee0827ad6391d008e4e4d200f` |
| `DENSE-FROZEN-G0-INPUT-AUTHORITY.tsv` | `95aaa6196c5e78506c7a9f4755c04bf9a8eea02aa669d30f3f6e9d1d672f3330` |
| `DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv` | `a5cdd456fa76e61e880c8e04cd8ef66a5091aedf3389e9b1e9e996814ed822d9` |
| `DENSE-EVIDENCE-TARGET-AUTHORITY.tsv` | `7661751b739b5b4060eff8ffc0af49f8068f9b2afcc44c01917ba6aa17b4acb9` |
| `DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv` | `9ad05359418d3280a4ba78873163f223eab2f3764c4aa7f3df20f86dfcbdcf22` |
| `DENSE-OVERLAY-BRANCH-AUTHORITY.tsv` | `2d1d737501c75903fe564e46af334700703a61a00c1e7f1fac8f0058c4638738` |
| `DENSE-ROLE-UNIT-AUTHORITY.tsv` | `eaf6f6bdd5536b48ae9da187667d0b56d763394da01d7f6fd9a75e7660dc14a0` |
| `DENSE-CAPABILITY-UNIT-AUTHORITY.tsv` | `f7af4823c1c52bb898b412d548a8a5c8f2a15a1a82e621d23bbfb7d8556500a3` |

## P1 findings

1. Unanchored selector children inherited cluster-wide target and member
   unions. This leaked unrelated targets into BuildHasher source clauses and
   unrelated members into clone, fill, and initialization helpers.
2. The four frozen `ACTIVE_BR_STORED` clusters had no required BR-STORED
   binding. BR-STORED appeared only as a deferred overlay capability.
3. The role authority contained 24 identities and silently omitted the frozen
   `O-ROPE-UNIQUE` boundary identity. A non-applicable role still requires an
   explicit no-member terminal.
4. Coherent mutations of selector, target, member, role, capability, and input
   rows were accepted. The validator checked mutual consistency instead of
   independently deriving complete ordered truth from frozen inputs and closed
   declarations.
5. The coverage source imported mutable member and exclusion declarations from
   an unhashed local module, so the reviewed semantic dependency set was open.
6. Excluded targets named exact real evidence members but the member authority
   omitted their outcome bindings. Those evidence-backed exclusions must not
   be represented as synthetic protocol units.

Mutation canaries accepted by the failed validator included a coherent direct
and selector reroute from F-DENSE to F-RECURSIVE, member narrowing and
substitution, role-member removal and substitution, capability-member removal
and substitution, and a zeroed frozen-input SHA-256.

## Independently confirmed facts

The review independently recovered 65 clusters, 651 parent evidence
identities, a 119/193 dense/non-dense concrete topology partition, 29
additional operation gates partitioned 3/6/20, the 39/17/4/5 payload
partition, and 75 overlay keys. It recovered 426 grammar children: 382 clauses,
35 canaries, and nine helper types. All 456 direct identities were anchored
exactly once. The 12 historical G0 inputs matched their commit, blob, byte, and
SHA claims. Direct collision-sensitive member mappings and all 29 additional
gate terminals were correct.

## Required disposition

Replace cluster-wide routing with a complete closed per-child authority;
preserve all 25 roles; bind active BR-STORED and real excluded evidence units;
close every local semantic dependency; and compare every full ordered output
against an independent derivation. Durable mutation tests must reject every
attack above. The repaired bytes require a new exact-hash hostile review.
