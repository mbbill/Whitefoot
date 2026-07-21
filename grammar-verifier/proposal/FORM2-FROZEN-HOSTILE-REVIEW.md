# FORM-2 frozen-evidence hostile review

Date: 2026-07-21

Verdict: **GO for owner-approval presentation**

Authority: proposal-only review evidence. This review does not approve a
successor specification, apply either protected-source patch, change a
protected expectation or status, authorize compiler work, or make a release
claim.

Reviewed candidate:

- path: `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- byte count: `98044`
- SHA-256: `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`

This verdict is valid only for those exact bytes.

## Evidence identity

| artifact | SHA-256 |
|---|---|
| `form2-evidence.sha256` | `b907fc38adbcf9174a832d66e36917f4df8c1c434d26651d65e39d9a0ec72a68` |
| `form2-independent-evidence.sha256` | `d4dd3bd42759e9138a93514020ec2ca39adc2139651e43ee7a15ce21cf6f1fdc` |
| `form2-structural-layout-evidence.json` | `7bab5d114dc1b4d0818232c88c580b1247e139e911eee6501c116bd6422fdf80` |
| `form2-independent-report.json` | `142a34c3b9e9fd1f3c20da9848bda3984092a88b7c995c63a5c2dcf22333b404` |
| `form2-protected-syntax-repairs.patch` | `724dbb970c8ce7ede7a52daf3ad2c9286b7872137e83f495fbf845df75252479` |
| `form2-structural-migration.json` | `775d54381999b670619e240426de285b28bb6483647d697d67db483e68c5f099` |
| `form2-structural-migration.patch` | `4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2` |

Both reports bind the frozen candidate hash. `make -C grammar-verifier
form2-evidence` reproduced the checked-in evidence. All 72 focused FORM-2
tests passed. A separate cross-audit matched all 293 source records between
the primary and independent implementations.

## Structural result

The two implementations agree on all 274 rendered sources: canonical bytes,
ordered source-local `item` forests, terminal projections, separator owners,
and bundle topology. A source forest contains no `program` node. A bundle has
exactly one global `program` root, including a one-source bundle. An empty
source contributes no tree node but keeps its source identity and byte extent.
Separator ownership is derived from the deepest common production ancestor of
the adjacent terminal leaves; source-leading, source-final, inter-item, and
zero-item bytes remain `SourceBytes`. The independent hostile controls reject
a fabricated source-local program root and a forged authored owner digest.

The remaining 19 sources receive no invented derivation and remain unchanged:
17 stop at grammar formation and two stop at lexical formation. The rendered
set consists of 270 direct complete derivations, three exact syntax-repair
controls followed by complete derivations, and one isolated tab recovery.

## Exact protected-source proposals

The syntax-repair patch has exactly these three paths:

- `conformance/cases/const1-neg-noninteger.wf`: three `NAME` spellings become
  `name`.
- `conformance/cases/pending-const2-item.wf`: `LIMIT` becomes `limit`.
- `conformance/cases/type7-neg-match-borrow-expression.wf`: one `doc` line is
  removed from a statement-only arm.

The combined structural migration patch has exactly 274 unique paths, all
under `conformance/cases/`. Both patches pass `git apply --check`. The primary
and independent implementations produced byte-identical copies of both
patches. Neither patch contains the numbered specification, manifest,
expectations, statuses, or governance files.

The expectation-and-status projection is unchanged before and after the
proposal, with SHA-256
`5fb0e54ec006c3fea82d5fc0d8c454e5e9f022ba472cdcc6a90c44a31ade2132`.
This is structural evidence, not a claim that a production compiler has
validated every semantic verdict.

## Protected FORM-2 negative controls

The combined patch does not accidentally canonicalize either protected
FORM-2 negative. Each proposed postimage still has exactly one
`indentation-two-spaces-per-brace-level` violation and retains the protected
`reject FORM-2`, `runnable` expectation:

| case | proposed postimage SHA-256 | retained authored bytes at defect |
|---|---|---|
| `form2-neg-noncanonical-ws` | `d50093fefabbda75b84d5516335dc97a56bd98feaa0f003fe3c79e4916f57cfe` | four spaces where canonical rendering has two |
| `x-form-form2-tab-indent` | `6f34923fc07b693faf080ac52f20ed431e0d5b8edd076d9e71fdea91b355dc37` | one tab where canonical rendering has two spaces |

Both controls independently derive the same canonical reference bytes,
SHA-256 `790672a14dac7af911e0bcb3ab6830bedce03f706b15e88d29c5eca0274db733`;
the migration deliberately preserves the one authored defect instead of
installing those reference bytes.

## Guard and scope conclusion

No guarded edit was applied during this review. The active numbered v0.8
specification remains SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`,
and `conformance/manifest.jsonl` remains SHA-256
`20bb50032c112150c3d9a7387a17bde708922e426550b47b64f2214cd7341d69`.
The primary report intentionally records independent comparison as pending;
the separately generated independent report and this exact-hash hostile review
close that proposal-review blocker without rewriting the primary artifact.

**GO for owner review of the frozen proposal; NO-GO for applying protected
patches or installing a successor specification without separate exact owner
approval.**
