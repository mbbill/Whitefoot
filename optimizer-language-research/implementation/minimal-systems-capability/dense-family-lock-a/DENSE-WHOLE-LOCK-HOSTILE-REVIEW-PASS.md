# Dense Unique-Owner Family Lock A: Whole-Lock Hostile Review

Status: PASS_EXACT_BYTES_WHOLE_LOCK_RESEARCH_ONLY

Date: 2026-07-15.
Freeze timestamp: 2026-07-15T01:11:35-07:00 (America/Los_Angeles).

Reviewer task identity: /root/whole_lock_hostile
Reviewer role: independent final exact-byte whole-lock reviewer

The reviewer edited no reviewed lock byte. The only repository artifact written
by this reviewer is this report, which the manifest deliberately excludes to
avoid a hash cycle. The reviewed integrating-author identity is `/root`.

This pass closes exact-byte research review only. It does not approve an owner
decision, choose a candidate mechanism, authorize an observation, or select a
language or production design.

## 1. Exact-byte review boundary

The following table pins the closure manifest, its summary, the owner dossier,
both whole-lock tools, and all three lower hostile-review authorities. Each
digest was recomputed from the final immutable bytes.

| Artifact | SHA-256 |
| --- | --- |
| `DENSE-LOCK-ARTIFACT-MANIFEST.json` | `0bff36e75a41575ae16bd51fc12ef5c0fcdb819288aa7755eeea320741a5ad97` |
| `DENSE-LOCK-BUILD-SUMMARY.json` | `a9584bc10ba6414a94d10cce2ba95ff066ef9e0632f8d2167b57b17bd31b9ea2` |
| `DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md` | `2a7114b82a6cf97d81a6bcf4695cfcd50b28a3b15aa5e2048dbfa039ad5a1f13` |
| `build_dense_lock.py` | `02290bc605aeb6c956c114cf2a9fcd6f3d0c434c27ceb2e732adb06d31f75afb` |
| `verify_dense_lock.py` | `e610888c724753e83d995b150238d8605cfaf723a3db1fd6054bf9dd7fddf282` |
| `DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md` | `d8ee4c161e84a3996c0167b54576893074a16775b30994ab8236e79fa63d4798` |
| `DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md` | `20b6325366c961a5d608066da8acd9a9c19352290fdaa44e3666f2e14430c7c7` |
| `DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md` | `e42823c8ecf94b2ac5c898c3215c511e9881fd082b7b77a112e98ff3b3b7bfe1` |

The manifest and summary are canonical sorted JSON. Independent in-memory
regeneration reproduced both byte for byte. The manifest's directory set is
exactly 74 artifacts; its current-control set is exactly eight files. Every
recorded byte count and SHA-256 matches the current path. The independently
recomputed artifact-set, artifact-map, and control-map digests are:

- artifact-set SHA-256:
  `e762ef954f162781fb902fd31428d8655131bfb0c6480fea4aa666de46a61a38`;
- artifact-map SHA-256:
  `7b7cb991d02bbb9da62be20a4ca0c950f2adf20d289032c8890151d0ad601bea`;
- control-map SHA-256:
  `af3b91b7fe61054dd3554ab87e2fca2afccd59df245ab9a71cd73d72a5c4e47f`.

All seven Section 19.1 artifact classes have a sorted exact path set, a current
path-to-hash map and map digest, a producer/tool identity, a reviewer identity,
and an honest status. The instantiated dossier and fail-closed validator await
this external review in the self-hashed manifest. Contract, soundness, and
performance classes bind their lower review authorities. The held-out custody
class is explicitly `BLOCKED_PENDING_EXTERNAL_H_FLATSET_CUSTODY`, names
`PENDING_EXTERNAL_H_FLATSET_CUSTODY`, and claims no hidden source or hash.

## 2. Authorization, identity, and predecessor audit

The D13 directive is byte-identical to the directive at commit
`c4ca5437fc90f3ce833fb026f2e794f4f758d011`, with directive SHA-256
`3a209c195f575408a65d1a81b9e3e01b4c95dd406b589fab801c64b3ed29c64c`.
That commit, the G0 closing commit, and the coverage-independence commit are
ancestors of the reviewed repository baseline.

The required predecessor-lock set is empty. This is the first owning-storage
Family Lock; G0-Core is a frozen research input rather than a completed family
predecessor. Its closing commit is
`a4de0eb70c345dcd198b11f435a5538ccc863113`, its unique decision-gate heading
is `G0-Core capability accounting is complete; mechanisms remain unselected
(2026-07-14)`, and its 110-artifact manifest SHA-256 is
`f0eced756688affef1732a133c43fb39ab6fc672334dca27b26129ddb5123719`.
Cross-family state or fact exposure is exactly `NONE`; candidate partial states
and fact channels remain sealed and unselected.

Coverage review is pinned to the durable D13-R3 evidence at commit
`32c01e188ba55f652700cf8547187fe462302f0b`. Contract/soundness review pins 27
current artifacts. Performance review retains reviewer task identity
`/root/repair_soundness_protocol`, pins 39 current artifacts, and attests that
the reviewer edited no reviewed byte. The three independently recomputed
current-artifact map digests match the manifest exactly.

The six unresolved decisions are exactly OD-0 through OD-5. Approving this
dossier resolves only those research-protocol choices. Candidate Freeze B,
scored and held-out work, family closure, production adoption, specification
or compiler work, xlc migration, teaching, and broader claims retain their
separate owner gates. The 13-row completion record honestly separates frozen
research evidence from operationally blocked work; owner authorization remains
`BLOCKED_SIX_OWNER_DECISIONS_UNRESOLVED`, and durability remains
`EXTERNAL_FINAL_COMMIT_AND_DECISION_GATE_REQUIRED` at manifest construction
time.

## 3. Independent evidence reconstruction

Independent parsers that import neither the generators nor their validators
reconstructed the following relations from the frozen TSV and JSONL bytes:

- 65 audit clusters, 426 selector children, 1,400 evidence-to-target terminals,
  780 evidence/member bindings, 101 overlay bindings, 85 role bindings, 1,267
  capability bindings, and 456 uniquely anchored direct evidence identities;
- 303 unique exact contracts over 93 unique members, a bijective 303-row owner
  role domain, five lifecycle candidates, and the complete 5 by 303 product of
  1,515 candidate/contract bindings;
- eight protocol-only fact channels and 2,002 deterministic ownership traces,
  including exactly 262 expected-rejection hostile traces; and
- no candidate-construction or candidate-execution authority in any applicable
  registry or trace.

The historical E0.1 join contains exactly 13 unique input rows and exactly 93
unique classified members, equal to the contract registry's member set. The
partition is exact: 84 `NEW_MANDATORY_EXACT_CONTRACT`, six
`RAW_OR_INITIALIZATION_AUTHORITY_EVIDENCE`, and three
`LAZY_LIFECYCLE_EVIDENCE_NOT_PROMOTED`. Every row pins the current authority
bytes, the E0.1 traceability SHA-256, and the Family Lock template SHA-256. It
authorizes no E0.1 restart.

The performance protocol independently reconstructs 303 dispositions, 97
same-shape Rust operation gates, 520 matrix cells, 502 timed-primary cells,
eight active owner branches, and 27 unique blocking rows. The earliest-stage
partition is exact: nine `REFERENCE_PILOT`, 13 `CANDIDATE_CONSTRUCTION`, three
`CANDIDATE_FREEZE_B`, and two nonselection
`DESCRIPTIVE_COUNTER_REPORT` rows. Four Mac-local branches have eight direct
pilot, 21 cumulative construction, and 24 cumulative Freeze B prerequisites;
four dual-native branches have nine, 22, and 25 respectively.

All 520 matrix cells name only known branch-applicable blockers and include the
common repository baseline. The four OD-4 scoped cells contain both the
reference-side scoped-contract blocker and the later candidate-artifact
blocker. `PENDING_EXTERNAL_REFERENCE_PILOT` blocks candidate construction, so
the first candidate prompt cannot occur until every applicable pilot
prerequisite and the feasible reference-only pilot have closed. The dossier,
stage manifest, completion record, and `THE-PLAN.md` state the same ordering.

## 4. Hostile review

The existing executable layers passed all 22 coverage mutations, all 32
contract/soundness mutations, and all 48 performance mutations. The independent
performance reviewer additionally rejected 14 coherent attacks after exact
29-file regeneration and 39-file authentication.

This reviewer then applied 65 independent whole-lock attacks:

- 51 manifest, summary, authorization, identity, artifact-class, custody,
  completion, frozen-git-authority, and lower-review attacks; and
- 14 dossier, staging, pilot-order, cumulative-blocker, and completion-closure
  attacks.

The attacks included setting every one of the 15 manifest authorization fields
to true, changing the family/revision/timestamp/timezone, replacing current
reviewer identities, inventing a predecessor or cross-family fact, changing the
G0 commit or manifest, escalating the owner approval scope, claiming held-out
custody, converting blocked completion rows to pass, removing construction,
Freeze B, or witness blockers, supplying noncanonical manifest/summary JSON,
and omitting or staling required lower-review hash rows. Every attack failed
closed.

The external-review parser was separately attacked with altered or duplicated
verdict lines, reviewer identity or role, negative-authority lines, and required
exact-byte hash rows. Every such attack was rejected.

## 5. Repository verification

The reviewer ran the following against the final reviewed-byte set both before
and after adding this external report:

```text
python3 -B verify_dense_lock.py
make check
make -C compiler check
git diff --check
cmp -s AGENTS.md CLAUDE.md
```

The layer checks, whole-dossier verifier, root verification stack, and
self-hosting compiler checks passed. The two repository instruction files are
byte-identical. Python 3.9 compilation of the lock tools passed without writing
repository bytecode, and the changed/new repository paths and contents contain
no non-English prose.

## 6. Exact negative authorities

Candidate construction authorized: NO
Reference pilot execution authorized: NO
Candidate Freeze B authorized: NO
Candidate-primary execution authorized: NO
Scored or held-out execution authorized: NO
Held-out access authorized: NO
Candidate selection or scoring authorized: NO
Language or specification change authorized: NO
Language or specification decision authorized: NO
Compiler implementation authorized: NO
Production implementation authorized: NO
Production adoption authorized: NO
Compiler or production implementation authorized: NO
E0.1 restart authorized: NO
xlc migration authorized: NO
Default teaching authorized: NO

No P-level finding remains on the exact bytes pinned above. This pass makes the
research dossier ready for owner review of OD-0 through OD-5 only. It does not
convert any blocked completion item into an operational permission.

Result: PASS
