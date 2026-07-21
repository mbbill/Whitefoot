# Grammar-change verifier

This directory contains the standalone grammar-change evidence tool, the
immutable Phase 2 owner-review packet, and the separate installed-v0.9
reproduction. It is outside the production compiler workspace, and normal
compilation neither links nor invokes it.

The tool compares exact v0.8 bytes with the exact successor bytes that became
v0.9. Two engines independently extract the specification grammar:

- `static-auditor/` is safe Rust. It computes terminal intersections,
  nullable, `FIRST_2`, `FOLLOW_2`, exact strong-LL(2) decision collisions, and
  concrete witnesses.
- `oracle/` is Python standard-library code. It uses its own extractor and a
  bounded generalized parser to count complete source-level derivations.

The engines share only the exact framed inputs and resource limits. They do
not share extraction code, grammar objects, tokenization, lookahead tables,
generated streams, expected results, or implementation modules. Their common
source-coverage ledgers are compared byte for byte by a small runner that does
not parse EBNF or compute language facts.

## Historical review result

The committed evidence binds exact v0.8 SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`
and candidate SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
Both engines emit the same 135,581-byte common extraction ledger, SHA-256
`e725430cc6a4bb87c5d1aa4673576efc77018e61d302850d142246872e285d30`.
The final canonical report has SHA-256
`1e26d171b58504e10a9a2d4510f1e5f6fd6c7190a70ad4cccacd558db0905789`;
the package manifest that binds every published component has SHA-256
`39436bbabaf194be43251a6afa028ff2b95c53c309ae906020672e2be959d03d`.

For both `deref(p)` and the exact transition witness `deref(x)`, v0.8 has two
complete derivations and the candidate has one: one fixed-place derivation is
retained, one call-through-IDENT derivation is removed, and none is introduced.
The static transition count is exactly one to zero for the complete `deref(x)`
source. Both engines independently generate the same 48-stream fixed-lowerword
domain, SHA-256
`f3e54408ce7c4234bb3b61e27f2decd6c84ffcc4d7fb1b201c9583dd0190480c`,
with no introduced terminal intersection or predictive conflict.

The authored ordinary-let, try-let, statement-match, value-match, and
requires-value-match probes each retain exactly one complete source derivation
in both documents. Their proposal trees select the new typed productions;
`match_stmt` and `value_match` are distinct, while a value match inside
`requires` still parses so FN-8 remains the rejection owner. The static engine
reports 34 current strong-LL(2) conflicts and zero proposal conflicts: all 34
are removed, with none retained or introduced. Retained terminal-predicate
intersections remain legal census facts when the predicates never compete at a
grammar decision.

The owner approved this exact packet, and its candidate bytes are now installed
unchanged as `spec/kernel-spec-v0.9.md`. The evidence remains historical: it is
not rewritten to make its pre-installation statements sound current.

## Owner-review migration material

The historical packet proposed three protected patches in one exact order and
applied none at review time: the combined 274-path FORM-2 patch
`4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2`,
the post-FORM2 case-intent patch
`62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a`,
and the manifest metadata patch
`ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6`.
Ordinary application in that order yields a 99,869-byte manifest with SHA-256
`0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c`
and changes no expected verdict or runnable status. `proposal/DELTA.md` and
`proposal/protected-surface-census.json` contain the exact approval items,
postimage hashes, reviews, and explicit non-authorizations as they stood when
approved. The repository installation applies those exact postimages. The
current gate proves this by reversing C, B, A to their reviewed preimages and
then applying A, B, C back to byte-identical installed contents.
The primary and independent FORM-2 suites also reconstruct that approved
pre-migration corpus in temporary trees and rerun their parsing, rendering,
repair, topology, and hostile-mutation checks. Installation does not disable
the historical algorithm tests.

## Installed-v0.9 reproduction

`evidence/` is the immutable owner-review packet. `run.py` validates every
approved artifact digest and never writes there. It requires
`spec/kernel-spec-v0.9.md` to be byte-for-byte identical to the reviewed
candidate, supplies the installed bytes as the successor engine input, and
writes only `installed-v0.9-evidence/`. The installed report records both paths,
their byte lengths and hashes, and their exact equality. This reproduces the
grammar observations after installation; it grants no compiler authority.
The same packet gate also keeps the complete v0.8 lexical model, observer,
terminal audit, tests, and frontend-corpus snapshot hash-bound after active
development switches to v0.9.
It pins the approved derivation-ledger amendment and requires the live ledger
to equal the pinned pre-v0.9 prefix plus exactly the text between the amendment's
delimiters, with one separating LF and no other change.

## Frontend-boundary proposal evidence

The separate frontend-boundary gate checks the proposed source-bundle,
tokenization, tree, visibility, and diagnostic contracts before any production
parser exists. A primary model and an independently implemented model evaluate
the same closed descriptor of 100 structured cases and 34 exact-byte scanner
cases. The committed canonical report records their complete agreement, all ten
boundary-requirement projections, the descriptor and source revisions, and the
full current candidate bytes by length and SHA-256.

The candidate digest is generated from
`proposal/kernel-spec-successor-candidate.md` on every check and compared with
the immutable `evidence/frontend-boundary-evidence.json` and its `.sha256`
sidecar. These are historical approval evidence and are not regenerated by the
normal gate. They do not authorize a production frontend.

## Trust boundary

This is a development evidence tool, not production compiler authority. Its
claim is deliberately limited:

> On a clean, non-adversarial checkout, the declared OS, filesystem, process
> controls, Python 3 runtime and standard library, pinned Rust/Cargo toolchain,
> and trusted host linker/system tools are trusted to run the bound source
> revisions over the bound input bytes under the reported limits.

Source manifests are stable-tree observations. They are not attestation of
already-executed Python bytes or proof that source hashes equal mapped machine
code. Unexpected OOM, signal, timeout, process failure, or malformed output is
tool-inconclusive and can never prove a grammar conflict absent or a case
unambiguous.

Run the verifier gate with:

```text
make -C grammar-verifier check
```

The root `make check` includes this gate. `FORMAT.md` is the complete wire,
ledger, ordering, binding, and failure contract.

The checked-in review packet records why the exact v0.9 installation was
approved. Neither the historical packet nor its installed reproduction is a
production-parser or compiler-capability claim.
