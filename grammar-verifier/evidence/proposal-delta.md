# Exact v0.9 successor delta

Status: non-authoritative owner-review material. This document changes no
numbered specification, protected source, expected verdict, oracle, or active
compiler target. Exact v0.8 remains authoritative unless and until the owner
separately approves the frozen candidate and every protected installation item
below, followed by the guarded version-bump procedure.

## Frozen inputs

Current specification:

- path: `spec/kernel-spec-v0.8.md`
- byte length: 73,769
- SHA-256: `d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`

Proposed successor:

- review path:
  `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- proposed installed path: `spec/kernel-spec-v0.9.md`
- byte length: 98,044
- SHA-256: `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`

The frozen candidate already has the installed-version header:

```text
# Kernel Specification v0.9

Status: DRAFT v0.9 (2026-07-21; canonical-frontend entrance closure).
```

Its status paragraph says that these exact bytes become authoritative only
after complete evidence, protected-surface review, owner approval, and guarded
installation. That conditional sentence is part of the installable bytes. No
post-approval removal of `PROPOSAL`, status rewrite, or other cleanup is
planned. If approved, `spec/kernel-spec-v0.9.md` must be byte-for-byte identical
to the reviewed candidate. Any edit creates a new candidate and invalidates all
candidate-bound evidence and hostile reviews.

## Exact numbered-rule delta

The current specification has 91 numbered rules. The candidate has 92.

- added: `PROG-2`
- removed: none
- changed existing rules, exactly 21: `CONST-1`, `CONST-2`, `DIAG-1`, `EX-1`,
  `FN-1`, `FN-4`, `FN-8`, `FORM-2`, `FORM-3`, `FORM-4`, `FORM-5`, `FORM-7`,
  `GIVE-1`, `GRAM-1`, `GRAM-2`, `GRAM-3`, `GRAM-4`, `GRAM-7`, `PRE-1`,
  `PROG-1`, and `TYPE-6`
- unchanged existing rules: the other 70 v0.8 rules

`EX-1` and `PRE-1` change only because their normative byte fences are rendered
in the new exact FORM-2 layout. `GIVE-1` is a semantic repair found during
hostile review. The remaining changes close the canonical frontend entrance or
move unchanged grammar productions to their unique semantic owners.

## Exact grammar delta

The executable grammar changes from 59 to 62 productions:

- added: `law_arg`, `ordinary_let_rhs`, `requires_entry`, `try_let_rhs`, and
  `value_match`
- removed: `match_block` and `try_stmt`
- changed in shape: `law`, `let_stmt`, `match_stmt`, `requires_block`, and
  `stmt`
- moved without shape change: `const` from `GRAM-3` to `CONST-1`, and `cvalue`
  from `GRAM-3` to `CONST-2`

The current and candidate grammars each contain 169 quoted fixed-terminal
occurrences. External lexical definitions fall from eight to seven because
`LAWNAME` is removed. Grammar-reference occurrences rise from 176 to 178. The
expanded exact fixed-lowerword domain remains 48 spellings in each document.

### GRAM-1 fixed-atom and terminal contract

The candidate makes the grammar-verifier's atom interpretation part of the
specification rather than an unreviewed tool convention:

- Raw lexical formation scans one source independently and chooses the exact
  maximal form at each cursor. It does not consult grammar position, name
  lookup, the operation table, or another source.
- A quoted fixed atom expands to the unique sequence of formed raw tokens whose
  concatenated bytes equal the quoted bytes. Thus `"&uniq"` contributes two
  tokens, `&` then `uniq`; `"->"` and `"=>"` each contribute one compound
  token.
- The sole pattern atom is quoted `"[0-9]+"` in `const`. It denotes one formed
  numeric token that fully matches the pattern and is not a fixed atom.
- `SELECT_2` counts the expanded formed-token sequence, not characters or
  quoted atoms.
- Terminal membership evaluates every formed token against every approved
  fixed and external predicate, retains all matching predicates, and rejects
  only when the set is empty. There is no preferred predicate, keyword
  priority, or context-sensitive retagging.
- Every choice, optional, and repetition decision must have pairwise-disjoint
  strong-LL(2) `SELECT_2` languages. Predicate overlap that never competes at a
  decision remains a reported census fact, not an ambiguity.
- Every production has one exact core-tree node kind. A shared semantic node
  cannot erase a production distinction.

The grammar then factors the full let prefix before selecting ordinary, try,
or value-match right-hand sides; gives statement and value matches distinct
productions; and parses broad law and requires syntax before their semantic
owners check it. The generalized parser remains independent grammar evidence,
not a production parser or language authority.

## Normative change families

### FORM-2, EX-1, and PRE-1: one executable tree-derived source format

After the complete compilation unit has one derivation, each source owns the
ordered forest of top-level `item` subtrees whose terminals belong to that
source. The forest is a view under the single compilation-unit `program` root,
not a second program. Each source is rendered independently with closed line,
indentation, attachment, block, and source-boundary rules. Lexical or grammar
failures occur before this comparison and never receive a fabricated tree.

The normative prelude and example fences are rewritten into that exact layout,
which accounts for the `PRE-1` and `EX-1` rule-byte changes without changing
their declarations or program meaning. For a FORM-2 mismatch, the location is
the first unequal byte boundary and its actual-or-required trivia gap. An
intra-item gap belongs to the deepest common production-node ancestor of the
adjacent terminal leaves. Leading, final, inter-item, and zero-item gaps use
`SourceBytes`. Renderer emission order does not select the owner.

Selection ground: correctness- and evidence-selected mechanics; the particular
visual conventions remain R3-provisional. This addresses
`discrepancy:v0.8/form2-protected-conformance-spacing` only after approval of
the separate protected migration.

### FORM-3 and FORM-4: terminal partition and erratum

`IDENT` excludes every lowercase spelling produced by expansion of exact fixed
grammar atoms. The exclusion is mechanically derived from the complete
grammar, not maintained as a second keyword list. This removes fixed words such
as `requires` and fixed `deref` from every IDENT position while preserving
ordinary non-fixed lowerwords. Name-shape errors retain FORM-3 attribution.

`FORM-4` now points to `GRAM-2`, the actual owner of `doc`, rather than
`GRAM-3`. This one-reference erratum changes no source construct.

Selection ground: terminal partition is correctness- and evidence-selected;
the cross-reference is a minimality-selected erratum. Relevant v0.8 records are
`discrepancy:v0.8/gram-terminal-ident-partition`,
`discrepancy:v0.8/fn8-reserved-rule-attribution`, and
`discrepancy:v0.8/form4-doc-cross-reference`.

### FORM-5 and FORM-7: total finite-float spelling

`FORM-5` defines the exact rational value represented by coefficient `C`,
fraction length `F`, and exponent `E`, preserves signed zero, and fixes IEEE
round-to-nearest-ties-to-even. Canonical spelling first minimizes ASCII bytes
before the suffix and then uses unsigned-ASCII lexical order. `FORM-7` accepts
only that unique finite spelling. NaN and infinity remain operations, not
literals.

The executable Python evidence is an exact-rational canonical search and
membership model. The Rust `float_contract.rs` path independently extracts and
pins the exact FORM-5 and FORM-7 clauses and lexical grammar, and rejects
hostile clause mutations. It is deliberately **not** represented as a second
exact-rational canonical search or canonicality implementation. The
mathematical totality and uniqueness argument comes from the finite IEEE target
set, the nonempty set of decimal spellings for every finite value, well-ordering
by prefix byte length, and unique minimum under the ASCII tie-break.

Selection ground: correctness- and evidence-selected. This addresses
`discrepancy:v0.8/form5-form7-float-canonical-spelling` after installation.

### GRAM-2, GRAM-3, GRAM-4, and GRAM-7: factored typed nodes

The central changed productions are:

```text
requires_block := "requires" "{" requires_entry* "}"
requires_entry := doc | stmt

law     := "law" IDENT "(" (law_arg ("," law_arg)*)? ")" ";"
law_arg := IDENT | literal

let_stmt := "let" IDENT ":" mode type "="
            (ordinary_let_rhs | try_let_rhs | value_match)
ordinary_let_rhs := expr ";"
try_let_rhs      := "try" expr ";"
match_stmt       := "match" expr "{" arm+ "}"
value_match      := "match" expr "{" arm+ "}"
```

`match_stmt` and `value_match` preserve one arm surface but are different tree
nodes in disjoint grammar positions. `try_stmt` and shared `match_block`
disappear. `const` and `cvalue` keep their source shapes but move to their
unique owners, `CONST-1` and `CONST-2`. Broad law and requires children are
ordinary syntax; FN-4 and FN-8, not parser variants, enforce their semantic
subsets.

Selection ground: deterministic factoring and node identity are correctness-
and evidence-selected; use of one broad parser before semantic checks is
minimality-selected. This addresses
`discrepancy:v0.8/gram1-gram7-match-node-bijection` after installation.

### GIVE-1: exact value-match delivery

`give` targets the nearest enclosing value match. An arm delivers to a given
value match only through its final `give`, a `return`, a `break` whose resolved
loop lexically encloses that same value match, or a final statement match whose
every arm delivers relative to that same value match. A nested value match
delivers only to its own inner let. A check or call that may trap retains a
normal continuing edge and is not delivery. No loop is assumed to diverge.

This is a semantic contradiction repair discovered during hostile review, not
a registered v0.8 discrepancy closure. Existing protected value-match/give
outcomes and their exact reasons are enumerated in the protected-surface census.
The nested-value-match, may-trap, loop, and break clarifications require
additive successor semantic cases where existing executable evidence does not
already cover them. Grammar derivation evidence proves parse shape only; it
does not prove this control-flow predicate or broad verdict preservation.

### FN-4: closed mandatory source discharge, separate optional facts

FN-4 source acceptance is a mandatory closed compiler-independent calculus. It
is not selected by optional prover availability:

1. The conformance is nongeneric; its concrete `conform D : C` has no contract
   arguments; `D` is a concrete integer type; and `C` resolves exactly.
2. A law's function role resolves to exactly one `fn_sig` and exactly one
   `fn_bind`; the binding's right side resolves to one top-level function.
3. Contract and bound-function signatures are exactly two `own D` parameters
   and an `own D` result, with `pure`, no regions, no generic parameters, and
   no `requires` block.
4. Apart from an optional leading `doc`, the bound body is exactly one
   statement: `return iadd.sat<D>(p0, p1);`, using the two parameters in order.
   Semantic equivalents, wrappers, helper calls, or a two-step body do not
   qualify.
5. The closed table is:

   | Domain | Total | Associative | Commutative | Identity |
   |---|---|---|---|---|
   | `u8`, `u16`, `u32`, `u64` | yes | holds | holds | exact zero |
   | `i8`, `i16`, `i32`, `i64` | yes | refuted | holds | exact zero |

Every absent, refuted, or unavailable cell rejects with FN-4. Each successful
law/conformance pair emits exactly one canonical base derivation record, and
same-kernel replay recomputes it. The record is source-acceptance evidence only:
it grants no lowering or optimizer authority.

Optimization is a separate optional boundary. A future fact family would need
advance approval and an independent verifier that rederives its exact relation
from the accepted artifact and binds artifact, target, backend, proposition,
and consequence. Absence or failure leaves source acceptance, semantic
identity, safety checks, and empty-overlay lowering unchanged. A gated law
candidate is likewise only an optional proposition source. General proof
artifacts, general static proof menus, runtime enumeration, bounded testing,
and sampling are not v0.9 source-admission routes.

This local relation does not decide whole-conformance member completeness,
extra or missing bindings, law-free conformance behavior, generic contract
substitution, or behavior-parameterized calls. The separate
`discrepancy:v0.8/fn3-contract-member-semantics` remains open.

#### Exact five protected FN-4 outcomes

1. `fn4-pos-law-in-contract.wf` remains accepted. It declares a contract law
   but no concrete conformance, so it states an obligation and emits no
   accepted-law record.
2. `fn4-neg-bad-lawname.wf` remains rejected with FN-4. `distributive` is not a
   member of the closed law table and also has the wrong closed role/arity.
3. `fn4-pos-law-discharged.wf` remains accepted. Its `u64` saturating-addition
   body has the exact direct form, and associative, commutative, and zero
   identity each occupy a holding table cell, producing three base records.
4. `fn4-neg-law-refuted-signedness.wf` remains rejected with FN-4. Signed
   `i64` saturating addition is total but its associativity cell is refuted.
5. `fn4-neg-law-undischarged.wf` remains rejected with FN-4. Its two-step body
   is not the exact direct discharge form, so evidence is unavailable.

The expected-verdict change count for these five cases is zero. The dedicated
hostile review confirms source acceptance and optimizer-authority separation;
it does not authorize an optional fact schema.

### FN-8: broad parse, one early structural semantic pass

The grammar admits each direct requires child as `doc | stmt`. Before recursive
child semantics, FN-8 requires zero or more direct ordinary-expression
`let_stmt` nodes followed by exactly one final `check_stmt`. The first direct
shape violation wins; an empty or all-let block reports the block's missing
final check. Only after this early pass succeeds are the admitted children
checked for FN-8's closed purity, totality, operation, typing, ownership, and
scope restrictions. Fixed `requires` in an IDENT position remains FORM-3.

This keeps production grammar general while making the semantic boundary and
diagnostic precedence exact. The candidate intends to preserve the accepted
requires subset and existing protected outcomes, which are enumerated in the
census. Grammar evidence establishes that broad entries receive typed
derivations; it does not execute the FN-8 semantic pass. Until an additive
semantic model/checker covers those cases, preservation is reasoned from the
closed rule and protected sources, not claimed as compiler-validated.

Selection ground: correctness- and evidence-selected boundary repair, with the
block spelling still R3-provisional. This contributes to resolving
`discrepancy:v0.8/fn8-reserved-rule-attribution` after installation.

### FN-1 and TYPE-6: closed-unit function-signature visibility

All top-level function signatures are visible throughout the completed closed
compilation unit. Locals, regions, labels, generic parameters, and explicitly
earlier constants remain lexical declaration-before-use. PROG-2 supplies the
unit and whole-unit declaration order.

Selection ground: owner-selected resolution A-01, authorized 2026-07-21. The
existing mutual-recursion case already expects this behavior, so no verdict
change is proposed. A-01 is not a v0.8 discrepancy ID.

### PROG-1 and PROG-2: exact compilation-unit formation

One unit is an ordered nonempty sequence of logical source records. Every
record binds a portable relative path and exact source bytes. A path has one or
more nonempty components separated by exactly one `/`, no leading, trailing,
or repeated separator, only ASCII letters, digits, `.`, `_`, and `-`, and no
`.` or `..` component. Invalid or duplicate paths and zero records are envelope
failures. Invocation order, never filesystem or sorted path order, is semantic
transport order.

Each record derives its own `item*`; no token, trivia, production, or span may
cross a record boundary. One root flattens items by source ordinal and local
order and carries the exact bundle extent. Paths and boundaries contribute
identity only, never namespaces or lookup. Empty records remain identity-
bearing; a zero-byte source is valid input then fails FORM-2, while one LF is
the sole canonical zero-item source.

PROG-1 retains every closed-world ban: include, import, module, separate
compilation, incremental semantic cache, internal ABI, dynamic loading,
reflection, and source-path lookup. The gated FFI wall remains the only
external boundary. This resolves architecture question A-10 only after exact
approval and installation; it is not a v0.8 discrepancy record.

### DIAG-1: truthful pre-tree location and deterministic attribution

Source rejection locations are the closed `SourceBytes`, `SourceNode`, or
`BundleRoot` sum. Envelope, resource, compiler-invariant, artifact, backend,
and external-tool failures are non-language outcomes and cite no language rule.

The raw scanner is quote-aware and gives exact first-defect spans for malformed
UTF-8, prohibited controls, comment prefixes, malformed region/label sigils,
illegal token starts, invalid STRING escapes, non-ASCII scalars, and
unterminated strings. Comment-looking bytes inside STRING remain string bytes.

Grammar attribution follows approved source EBNF and strong-LL(2) decision
rows, not parser call stacks. It fixes the failure boundary, grammar-order
expected-terminal set, closed overrides, transparent mandatory-name traversal,
structural-choice stops, fallback, and leftover-input rule. It consumes no
input, performs no recovery or name lookup, and fabricates no node. FORM-2 gap
ownership uses the deepest common production-node ancestor rule described
above.

Selection ground: correctness-selected and independently modeled. This
addresses `discrepancy:v0.8/diag1-pre-tree-node-path` after installation.

## Evidence bindings and measured results

Every candidate-bound artifact below names the same 98,044-byte candidate and
SHA-256 `bdfb461d...b82b68` unless explicitly described as a patch or sidecar.

| Evidence | SHA-256 | Exact claim |
|---|---|---|
| `evidence/float-canonicality.json` | `4ee9b329a4fd72d0cd9ed33af94b019b7b7fe68116181f280113f9b9a744062e` | exact-rational Python model; 20 canonical, 9 classification, and 9 rounding vectors; independent Rust clause/grammar extraction and hostile mutation checks |
| `evidence/frontend-boundary-evidence.json` | `03eddf37794a2397815998768d0cd07558e3c519974bf7f5f8d628d0a9ced208` | two independent models agree on 134 cases: 100 structured and 34 raw, all B01-B10 projections, case projection `b92636cce14d172007b60ad6ba5e6334e6b0f8681c0e2e0ae3b380a114ab00b6` |
| `evidence/frontend-boundary-evidence.sha256` | `49b73463efd482a470522b5f7c9e4630cfbf9530ba7def7dc8719567713d9ecf` | sidecar binding for the frontend report |
| `evidence/form2-structural-layout-evidence.json` | `7bab5d114dc1b4d0818232c88c580b1247e139e911eee6501c116bd6422fdf80` | primary structural evidence over all 293 protected and pinned sources |
| `evidence/form2-evidence.sha256` | `b907fc38adbcf9174a832d66e36917f4df8c1c434d26651d65e39d9a0ec72a68` | primary FORM-2 evidence sidecar |
| `evidence/form2-independent-report.json` | `142a34c3b9e9fd1f3c20da9848bda3984092a88b7c995c63a5c2dcf22333b404` | independent comparison agrees byte-for-byte on repair and migration patches |
| `evidence/form2-independent-evidence.sha256` | `d4dd3bd42759e9138a93514020ec2ca39adc2139651e43ee7a15ce21cf6f1fdc` | independent FORM-2 evidence sidecar |
| `evidence/form2-structural-migration.json` | `775d54381999b670619e240426de285b28bb6483647d697d67db483e68c5f099` | structural migration inventory |
| `proposal/SUCCESSOR-HOSTILE-REVIEW.md` | `e6e52759b74c43d82863b6ae2860605648a3cdead172585b3e33fc5acd7879c0` | GO for owner-approval presentation of the reviewed successor/frontend delta |
| `proposal/FN4-HOSTILE-REVIEW.md` | `658f5a8ba03fcd1f6ce999a261a0fd57c0234d36be365df45b1727de61c87cc7` | GO for the mandatory FN-4 source calculus and closed optimizer-authority boundary |
| `proposal/FORM2-FROZEN-HOSTILE-REVIEW.md` | `3998150ddd5aae1b5a44b9447de160bfc719502f467eca05b378af288f938fff` | GO for presentation of the exact FORM-2 migration artifacts |
| `proposal/REMAINING-SEMANTIC-HOSTILE-REVIEW.md` | `2b9a86c1105dfec4691dff16b9c6a16d5085fe71b679e591b07ab1456f146abf` | GO for the reviewed FORM-5/FORM-7, FN-1/TYPE-6, PRE-1, and EX-1 deltas |
| `proposal/CASE-INTENT-HOSTILE-REVIEW.md` | `9a28e0b935547b46764be398608959e5fcf6c4392615c78e3d6fdd481feb469b` | GO for exact A-to-B-to-C composition, final protected postimages, and all five FN-4 outcomes |
| `proposal/DERIVATION-LEDGER-v0.9-AMENDMENT.md` | `f29b326f446aa9e5f512d079f1dbd14e641e6d840f18b69faab0ea39950e52a0` | exact append proposal for the live derivation ledger after approval |

FORM-2 measured 293 sources: 270 already had complete derivations; three need
the separately reviewed repairs and then derive; one isolated tab fixture is
recovered only to exercise FORM-2; 19 remain intentional earlier-stage
negatives. The two implementations render 274 sources and produce identical
patch bytes. Of those, 272 are canonical migrations and two are intentional
FORM-2 negatives. This is structural evidence, not production compiler
validation or proof of semantic verdict preservation; no production parser
exists.

The standalone grammar engines report a 135,581-byte common extraction block,
SHA-256 `e725430cc6a4bb87c5d1aa4673576efc77018e61d302850d142246872e285d30`.
Static comparison removes all 34 current strong-LL(2) conflicts, introduces
zero, removes 51 terminal intersections, retains 74 noncompeting intersections,
and introduces zero. Both `deref(p)` and `deref(x)` move from two complete
derivations to one. Both engines agree on the 48-stream fixed-lowerword domain,
SHA-256 `f3e54408ce7c4234bb3b61e27f2decd6c84ffcc4d7fb1b201c9583dd0190480c`.
The committed package report is regenerated only after DELTA, the census,
README, source manifests, and hostile reviews reach final bytes. Its checked
sidecar binds those exact inputs; no earlier package hash is part of this
approval request.

## Exact protected migration proposed separately

Specification approval alone does not approve protected edits.

### Protected source layer

Three syntax repairs are separately reviewable in
`grammar-verifier/evidence/form2-protected-syntax-repairs.patch`, SHA-256
`724dbb970c8ce7ede7a52daf3ad2c9286b7872137e83f495fbf845df75252479`:

- `NAME` becomes `name` in `const1-neg-noninteger.wf`;
- `LIMIT` becomes `limit` in the unmanifested `pending-const2-item.wf`; and
- one `doc` declaration is removed from a stmt-only match arm in
  `type7-neg-match-borrow-expression.wf`.

The intended CONST-1 and TYPE-7 rejection targets remain; the legacy source has
no manifested verdict. FORM-2 rendering after those repairs changes 274 paths.
The formatting layer has SHA-256
`1296a396b3297f2da64a8bfef2eb6fbf9b0d57bb6bfc622b20b4bfda7f74ae1e`.
The only repository-applicable patch is the combined
`grammar-verifier/evidence/form2-structural-migration.patch`, SHA-256
`4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2`.
It contains both layers and must be applied exactly once from the reviewed
repository state. The 274-path inventory SHA-256 is
`72894313ab0e7f7db4fbdddf4bde13eb973df1a298abfa15ec473e448f9dba68`.

The manifest expected-verdict/runnable-status projection is unchanged at
SHA-256 `5fb0e54ec006c3fea82d5fc0d8c454e5e9f022ba472cdcc6a90c44a31ade2132`.
The requested expected-verdict change count is exactly zero. This projection
does not claim a production compiler has established semantic preservation.

### Post-FORM2 case-intent layer

After the combined FORM-2 patch, the separately approved
`grammar-verifier/evidence/v0.9-post-form2-case-intent.patch`, SHA-256
`62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a`,
must be applied second. It changes five protected source postimages:

- the two FN-4 positive cases replace obsolete LAWNAME/OP-8 descriptions with
  the candidate's semantic law-name and closed FN-4-calculus wording;
- the FN-8 negative becomes `let`, `return`, `check`, so `return` is the first
  invalid direct requires entry while the required final check remains;
- the GRAM-1 positive names the actual IDENT-headed place/call fork and the
  distinct noncompeting match positions; and
- the GRAM-7 positive names the two distinct core-tree node kinds.

Their exact final source SHA-256 values are, in that order by path,
`9cd070cd331b163f0f230c8c57ee7c38f0d7aa23a6807987981bc29ee13c0418`,
`66f30c62380f95a332a00bd468ae9505307c87ca77db3c62dcb13f1e767b7d0d`,
`00a2b65bbfd272897a2b0596123c32e0069306c680cb77c4f7f229337c25202f`,
`3b146c7ac6185b12e5e703a4643cf0afd3c8b4f05ccc56fdd6ef5d6a07b71b18`,
and `a1c1986fedbbc00c0756986582dccebe07b7aad013258ccb4936a40dc5d6e43e`.

The same patch repairs only `doc` or `reason` prose in nine manifest records:
FORM-2 noncanonical whitespace; GRAM-1, GRAM-2, GRAM-3, GRAM-4, and GRAM-7;
and the two original FN-4 rows plus the discharged-law row. Every `id`,
`rules`, `expect`, `status`, and `covered_by` field is unchanged. The manifest
intermediate postimage is 99,869 bytes, SHA-256
`e0e3138869c337c47f2c527bda359fef1108ca1483b8a3e3f22cb86140581c3f`.
The expected-verdict/runnable-status projection remains
`5fb0e54ec006c3fea82d5fc0d8c454e5e9f022ba472cdcc6a90c44a31ade2132`.

### Manifest metadata layer

After the case-intent layer, exactly three manifest records need an additional
metadata-only edit:

- `META-1`: `kernel-spec-v0.8.md` to `kernel-spec-v0.9.md`, and `91 rules` to
  `92 rules`;
- `META-3`: `kernel-spec-v0.8.md` to `kernel-spec-v0.9.md`; and
- `META-4`: `kernel-spec-v0.8.md` to `kernel-spec-v0.9.md`.

Within this third layer no other manifest byte changes. The 99,869-byte
case-intent postimage moves from SHA-256
`e0e3138869c337c47f2c527bda359fef1108ca1483b8a3e3f22cb86140581c3f`
to final SHA-256
`0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c`.
The exact non-applying proposal artifact is
`grammar-verifier/evidence/v0.9-manifest-metadata.patch`, SHA-256
`ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6`.
It uses ordinary contextual unified-diff hunks, is bound by the runner source
manifest, and is applied third only after separate owner approval. Exact line
bindings are in the census. All three records remain
`covered_by: spec_ci`; expected verdicts and runnable statuses do not change.
Frozen oracle digests and existing reference-semantics tests remain unchanged.

## Derivation and discrepancy accounting

The exact ledger-amendment proposal covers all 21 changed rules and new
PROG-2. The live v0.8 ledger has 50 derived, 41
derived-existence-only, and zero underived rules. The proposed v0.9 state is 50
derived, 42 derived-existence-only, and zero underived across 92 rules. The
amendment is a separately approved append; the live ledger is not edited now.

After installation, the candidate supplies v0.9 dispositions for these eight
immutable v0.8 discrepancy records:

- `discrepancy:v0.8/diag1-pre-tree-node-path`
- `discrepancy:v0.8/fn4-law-admission`
- `discrepancy:v0.8/fn8-reserved-rule-attribution`
- `discrepancy:v0.8/form2-protected-conformance-spacing`
- `discrepancy:v0.8/form4-doc-cross-reference`
- `discrepancy:v0.8/form5-form7-float-canonical-spelling`
- `discrepancy:v0.8/gram-terminal-ident-partition`
- `discrepancy:v0.8/gram1-gram7-match-node-bijection`

Their v0.8 records remain unchanged. A versioned v0.9 registry must be
generated. The other seven registered v0.8 discrepancies remain open: affine
deref storage lifecycle, retained-check proof references, EFF-1 row
canonicality, body-local region effects, FN-3 contract-member semantics,
`main` return spelling, and dotless operation reservation.

## Honest residual debts

- FORM-2 evidence proves exact structural parsing/rendering and patch equality,
  not compiler acceptance or semantic verdict preservation.
- GIVE-1 and FN-8 need additive executable semantic cases for the clarified
  control-flow and early-pass boundaries; grammar evidence alone is not that
  checker.
- The candidate closes the canonical frontend entrance only. Semantic-kernel,
  artifact-schema, target, backend, and release gates remain separate owner
  decisions in `THE-PLAN.md`.
- FN-4 authorizes no optional optimizer fact schema or consumer consequence.
  Every authority-increasing fact family still needs independent verification,
  hostile review, and owner approval.
- A production implementation must preserve source-EBNF provenance and exact
  child-index identity for DIAG-1. Parser-stack approximations and a second
  diagnostic grammar do not satisfy the candidate.
- The candidate does not resolve FN-3 whole-conformance behavior or the seven
  other v0.8 discrepancies listed above.
- R3-provisional surface debts recorded by the derivation ledger remain debts;
  closing a contradiction does not pretend those forms were writer-selected.

## Separate exact owner approval items

The approval request must ask separately for:

1. installation of the exact 98,044-byte candidate, SHA-256
   `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`,
   as immutable `spec/kernel-spec-v0.9.md`, including its current installed-
   version header and conditional authority sentence;
2. the exact three-path protected syntax-repair layer, SHA-256
   `724dbb970c8ce7ede7a52daf3ad2c9286b7872137e83f495fbf845df75252479`;
3. one application of the exact combined 274-path protected source patch,
   SHA-256
   `4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2`,
   which already contains the repair and FORM-2 rendering layers;
4. application second of the exact five-source and nine-manifest-description
   post-FORM2 case-intent patch, SHA-256
   `62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a`;
5. application third of the exact three-record metadata patch, SHA-256
   `ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6`,
   and acceptance of the complete resulting 99,869-byte manifest SHA-256
   `0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c`;
6. the exact derivation-ledger append in
   `DERIVATION-LEDGER-v0.9-AMENDMENT.md`, SHA-256
   `f29b326f446aa9e5f512d079f1dbd14e641e6d840f18b69faab0ea39950e52a0`;
7. creation/regeneration of v0.9-bound assets and exact live-reference updates,
   preserving all v0.8 history; and
8. the active compiler and roadmap target switch to the exact installed v0.9
   bytes only after items 1 through 7 are complete; and
9. guarded baseline regeneration and approval-ledger append only through
   `make approve-spec REASON="..."` after the exact approved edits.

The request asks for zero expected-verdict changes, zero runnable-status
changes, zero frozen-oracle changes, and zero existing reference-test changes.
Approval does not authorize editing v0.8, weakening evidence, inventing an
optional FN-4 fact path, starting a production parser before the Phase 3
installation gate exits, switching targets before the exact approved sequence
is complete, or making a release claim.
