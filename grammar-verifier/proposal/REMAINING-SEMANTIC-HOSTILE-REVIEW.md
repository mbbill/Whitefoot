# Remaining semantic-delta hostile review

Date: 2026-07-21

Verdict: **GO for owner-approval presentation**

Authority: proposal-only review evidence. This is not specification approval,
protected-surface approval, compiler authorization, optimizer authority, or a
release claim.

Reviewed candidate:

- path: `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- byte count: `98044`
- SHA-256: `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`

This verdict is valid only for those exact bytes.

## FORM-5 and FORM-7 finite floats

The canonical selection is total. Every finite binary32 or binary64 nonzero
value is a dyadic rational. A dyadic rational has a finite exact decimal
expansion because its denominator is a power of two and therefore divides a
power of ten. A value at least one can be written with a nonzero first integer
digit and at least the fraction `.0`; a value below one can be written as
`0.` followed by finitely many fraction digits. Applying the value sign gives a
matching spelling for either sign. That exact decimal rounds to its originating
finite bit pattern, so every finite nonzero bit pattern has at least one
candidate. Positive and negative zero have `0.0` and `-0.0`, respectively.

The selection is unique. Candidate prefix lengths are natural numbers, so the
nonempty candidate set has a least length. Only finitely many ASCII strings
have that length, and unsigned-ASCII lexicographic order is a total order on
that finite set. It therefore has exactly one least member. The fixed `_f32`
or `_f64` suffix is excluded from the length comparison but cannot affect an
order among candidates for one stated type.

Signed zero remains separated. FORM-5 assigns the sign of a zero coefficient
directly from the leading sign, and IEEE conversion preserves the sign of an
underflowed nonzero input. The shortest positive and negative candidates are
therefore `0.0` and `-0.0`; no same-length nonzero decimal is small enough to
round to zero. The exact-rational evidence pins both signs for both formats,
minimum subnormal, maximum subnormal, minimum normal, maximum finite, ordinary
values, fixed-versus-exponent ties, underflow ties, and overflow ties. The
maximum-finite value itself supplies the nonempty boundary candidate, while an
input at the upper RNE midpoint rounds to infinity and is rejected by FORM-7.
Sign symmetry covers the corresponding negative finite boundaries.

The evidence roles are deliberately narrower than one another:

- `float_exact.py` and `float_canonical_evidence.py` use integers and
  `fractions.Fraction`, retain the sign of zero separately, implement exact
  RNE, and search grammar structures in ascending prefix length with the ASCII
  tie-break. Their 64-byte search bound covers the committed vectors; it is not
  an exhaustive run over every finite bit pattern and is not the proof of
  specification totality above.
- The Rust static auditor exact-shape-checks the complete FORM-5 and FORM-7
  clauses and recognizes the successor lexical grammar. It does not convert
  decimals to IEEE bits, search canonical spellings, or independently prove
  rounding semantics. No claim in this review treats it as a second float
  semantics engine.

The committed exact-rational evidence is SHA-256
`4ee9b329a4fd72d0cd9ed33af94b019b7b7fe68116181f280113f9b9a744062e`.
Its 13 focused tests pass. The Rust clause and lexical-shape tests pass,
including the exact-clause hostile mutations. No host float parser, formatter,
or arithmetic operation is in the Python evidence model.

## FN-1 and TYPE-6 visibility

The table is total because it partitions every declaration governed by name
lookup into two disjoint rows: a top-level `fn_decl` signature, and every other
declaration. Unit formation can first inventory and reject duplicate top-level
function signatures, then expose that completed signature table before any
body is checked. Forward calls and mutual recursion therefore do not depend on
source traversal order. Function bodies, local bindings, regions, labels,
generic parameters, type and contract declarations, and named constants gain
no forward-visibility exception; they remain visible only after their lexical
declaration through the end of their lexical scope. FN-1 delegates visibility
to this one table and introduces no second rule.

The protected mutual-recursion case
`conformance/cases/x-fn-mutual-recursion-runs.wf`, source SHA-256
`1355415dc4667780eba95cb7ecf61858575be761148d9de68e0d046c0bbe85b4`,
retains its `run`/exit-0 expectation. Its forward `isodd` call resolves from the
completed signature table. The separately proposed FORM-2 postimage is
`b96da2f8452934a7d21e48af2f7b93c3558b5e5b808ce9da902d701fbb0dc076`;
both FORM-2 engines report the unchanged terminal projection
`d9738992c98090fb07da896806035b1304aa04cdaa9490e0a71a9be4f2c6e454`.

There is no authored protected conformance case whose intent is a forward
constant. The additive proposal evidence supplies that missing direct probe:
both boundary models reject a constant reference in source 0 followed by its
declaration in source 1 with TYPE-6, and accept the reversed order. They also
reject forward local, region, and label references while accepting a forward
top-level function. The exact evidence JSON is SHA-256
`03eddf37794a2397815998768d0cd07558e3c519974bf7f5f8d628d0a9ced208`;
its checksum index binds that value, and all 49 focused boundary tests pass.
This is proposal-only evidence, not a protected-outcome claim.

## PRE-1 and EX-1 formatting only

Both the generalized candidate parser and the separately implemented FORM-2
parser produce the same terminal and source-forest projections for the v0.8
and candidate fenced programs. Each renderer maps both old and new input bytes
to the candidate bytes:

| rule | v0.8 bytes / SHA-256 | candidate bytes / SHA-256 | common terminal projection | common source-forest projection |
|---|---|---|---|---|
| `PRE-1` | `281` / `9be4318017d24afdfc6dadcd72d12fc7871131059c732a6cfc6955822b2514d8` | `303` / `547eedebc7d9f262580c824045acf6b4643b10e42e388ce399479f901240c469` | `d602f6beee8105f2cbec6cb73e05ac2e2415f4c0727c0c6c8f649e4643371dce` | `e3d6da9447c541cc9e5868408ca483d4513589158c0966da8fd004d08c2b74a4` |
| `EX-1` | `857` / `8e707165c64ec442b42c52d80cefbea7395ac685d7fc30320cc33f43347c8f42` | `863` / `490b202c156669e29030a4e6c2b2a86434da0aa7d33005f3db5079d830cbec71` | `d6b72e69e4e112b5b7de80564855fe45c41eacb867b811a0d483db3213bb2c3a` | `f20b222a0b7052db53e902637518b774386209480cc992c6a0df35a19834714f` |

No identifier, literal, operation, declaration, statement, production, or
ordering changes. PRE-1 expands compact declaration blocks to FORM-2's
one-child-per-line block form. EX-1 expands only the compact `Sign` enum in the
same way. The rule deltas are therefore formatting-only consequences of the
new FORM-2 renderer.

## Scope conclusion

No totality failure, second canonical spelling, signed-zero collapse,
finite-boundary hole, visibility escape, protected mutual-recursion regression,
or PRE-1/EX-1 semantic change remains in these reviewed deltas. The frozen
candidate is a **GO for owner review**. Installing it, applying any protected
source migration, or starting parser implementation still requires the
separate approvals and gates in `THE-PLAN.md`.
