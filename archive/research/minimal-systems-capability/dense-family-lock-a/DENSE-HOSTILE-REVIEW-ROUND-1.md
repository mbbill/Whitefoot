# Dense Unique-Owner Family Lock A: Hostile Review Round 1

Date: 2026-07-14. Review status: `FAILED_AND_REPAIR_REQUIRED`.

This review covered the first generated draft only. It authorizes no candidate
construction, Candidate Freeze B, timing, held-out access, language selection,
or production change. The reviewed artifact manifest had SHA-256
`41c76b2ae8711a791992e19428129d846947b4664a431b9cfe7cf01432ce0565`.
The report had SHA-256
`fb2389bf51fbdbdd83097f9864ce9ff1ad2f58796a34d1534ff24834d18c3460`.

## Findings that survived independent recomputation

The coverage reviewer independently recovered the immutable G0 snapshot and
confirmed these exact facts:

- 65 routed F-DENSE audit clusters: 54 primary and 11 implicated or reopening;
- 651 parent evidence identities: 117 stable-safe, nine D10, 18 stable-unsafe,
  312 concrete implementations, and 195 selector parents;
- a complete concrete-topology partition of 119 dense and 193 non-dense
  implementation identities;
- 29 dense concrete identities with an additional operation-gate target,
  partitioned as three bulk, six index, and 20 conversion targets;
- 25 required role identities and 49 capability identities; and
- the exact payload-source partition of 39 deferred, 17 no-complement, four
  active BR-STORED, and five boundary clusters, including all 75 overlay keys
  and the single delegated allocation-error row.

These confirmations are evidence-domain facts only. They do not validate the
first draft's generated dispositions or contracts.

## Blocking coverage findings

1. The generator read mutable working-tree G0 files while the report claimed an
   immutable historical snapshot. Every consumed G0 byte must instead come from
   `git show a4de0eb70c345dcd198b11f435a5538ccc863113:<path>` and be checked
   against the historical manifest.
2. Rust-surface selector expansion substituted unrelated evidence rows and
   omitted required helper identities. Concrete examples were `Vec::drain`,
   `Vec::extract_if`, `Vec::splice`, `mem::swap`, and `TryReserveError`.
3. Non-concrete applicability used substring routing with an F-DENSE default.
   The replacement must be an enumerated, source-audited, fail-closed authority.
4. Evidence-to-member routing used substring matches and a cluster-wide
   fallback. It conflated reserve/reserve-exact, push/push-mut, sort/cached-sort,
   reverse/rotate, swap/swap-with-view, pop/pop-if, and dedup variants.
5. The 29 additional operation-gate targets had a pending label rather than one
   legal terminal. Until their owning locks exist they must explicitly block
   the named gate, cluster, family, and whole-floor claims.
6. Payload overlays were attached by a cluster-wide Cartesian product. Each
   branch instead needs an exact evidence/member/outcome binding. The four
   active BR-STORED routes must require BR-STORED; only stronger complement
   branches may be deferred.
7. Role and capability ledgers were identity-complete but lacked exact
   member/outcome and frozen-fixture bindings.

## Blocking soundness findings

1. The 183 generated rows were not an outcome ledger. They used four generic
   outcome names, contained duplicate keys, omitted checked traps,
   abandonment, divergent OOM, and several real operation branches, and often
   gave success and failure identical semantics.
2. The 22,692 generated soundness rows were scenario descriptors rather than
   executable oracles. They lacked candidate, member, outcome, transition,
   failure-point, concrete initial-state, concrete expected-state, and
   diagnostic identities. Their claimed payload, exit, capacity, and failure
   axes did not match the generated rows.
3. `C-PROOF-CARRYING-STATE` did not identify the unique authority that releases
   an allocation shared by base and range owners. It must freeze one master
   allocation rule that prevents escape, reallocation, or release while a
   range owner remains live.
4. No optimizer fact channel was frozen despite FT-STATE being mandatory. Each
   fact needs an exact proposition, owner/root/version, producer, scope,
   consumers, invalidators, transfers, speculation boundary, facts-off rule,
   artifact evidence, and negative traces.
5. The five candidates lacked a non-collapse matrix and exact per-candidate
   partial-state and normal-exit rules.
6. ZST growth and destruction were inconsistent. A Vec-like branch requires
   logical capacity `usize::MAX`, no payload allocation, index-based logical
   identity, and exactly `len` logical destructions.
7. Borrow, behavior, failure, abandonment, wrong-fact, and metadata/payload
   attacks were materially incomplete.

## Blocking performance and statistics findings

1. The 2,052 rows were a blind target-by-payload-by-trace-by-scale product, not
   executable contract cells. The replacement needs one explicit disposition
   for every member/outcome and sparse operation-specific primary cells.
2. The matrix omitted independent reserved-append, swap, eager-drain,
   shared/unique/owning traversal, H-FLATSET, W-SMALL, W-GAP, B-FIX, and B-P2
   cells. Compound traces cannot hide a regressing atomic operation.
3. Reference algorithms, exact Rust source routes, capacity policy, allocator
   accounting, byte boundaries, event streams, target identities, and
   structural thresholds were not frozen. The ZST route contradicted pinned
   Rust 1.97 `RawVec`.
4. The statistical protocol left `k`, the Williams rows, facts-off placement,
   bootstrap statistic, multiplicity family, memory zero rule, and raw schema
   ambiguous. Power cannot be 90% when the simulated true ratio equals the
   decision boundary.
5. A post-lock crossover would change the candidate universe and multiplicity.
   The repaired draft must freeze `NO_CROSSOVER`, or enumerate the complete
   crossover as a sixth candidate and rerun every design calculation.
6. B-FIX and B-P2 were prose rather than exact source, artifact, layout,
   code-shape, structural, and equality gates.
7. META-5 omitted the template's cumulative syntax, grammar, normative,
   ownership, exit/drop, proof-state, artifact, test, derivation, and protected
   baseline columns.

## Required disposition

The reviewed bytes fail owner review and fail candidate-construction readiness.
All findings above must be repaired in generated artifacts, validated by
independent fail-closed checks, and re-reviewed on new exact hashes. A repaired
dossier may still be conditional on explicit owner policy choices and external
runner or held-out custody identities; those conditions must be enumerated and
must continue to block construction.
