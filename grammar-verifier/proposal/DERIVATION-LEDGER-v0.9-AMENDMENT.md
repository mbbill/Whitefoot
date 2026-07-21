# Proposed v0.9 derivation-ledger amendment

Status: non-authoritative owner-review material. This changes neither the live
ledger nor v0.8. Installation requires advance approval of the exact v0.9
candidate, protected migration, and evidence package.

The candidate binding is frozen at 98,044 bytes, SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
Every candidate-bound receipt must match that identity before installation.

The boundary comments enclose the exact text to append to
`spec/derivation-ledger.md`. Earlier text remains historical provenance; these
entries supersede it only for their stated v0.9 deltas.

<!-- BEGIN EXACT V0.9 DERIVATION-LEDGER APPEND -->

## v0.9 amendment — canonical frontend entrance closure (2026-07-21)

Specification binding:
`spec/kernel-spec-v0.9.md`, SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.

The candidate's conditional authority sentence is part of those exact bytes:
the reviewed document becomes authoritative only when installed through the
guarded procedure. No post-approval status-header rewrite is permitted.

Earlier entries remain dated history. For v0.9, this amendment supersedes an
earlier entry only for the changed fact it states; all other status and debt
continue. The original header was a historical v0.3 accounting statement.
Immediately before this amendment the ledger contains 91 unique rule rows: 50 derived, 41
derived-existence-only, and zero underived. This amendment changes 21 existing
rules and adds PROG-2. The v0.9 total is therefore **50 derived · 42
derived-existence-only · 0 underived** across 92 rules.

`derived` means existence and form have a constitutional or evidence-selected
chain. `derived_existence_only` means at least one surviving form choice remains
minimality-selected and R3-provisional. A selected correctness repair does not
erase an independent older form debt.

### Rule amendments

The entries cover all changed rules. This row exposes new PROG-2 to META-6:

| Rule | v0.9 status |
|---|---|
| PROG-2 | derived_existence_only |

#### CONST-1 — derived

- **Delta and selection ground:** the unchanged `const := "[0-9]+" | IDENT`
  production moves from GRAM-3 into CONST-1, its unique semantic owner. This is
  a correctness-selected META-4 repair; it changes no accepted source shape.
- **Continuing chain and debt:** the existing closed-const-language chain from
  R2, R4, TYPE-2, and FN-2 continues. Const arithmetic remains a deferred
  specification addition, and the earlier weak-writer/external-generation debt
  remains open. No v0.8 discrepancy record is claimed closed by this move.

#### CONST-2 — derived

- **Delta and selection ground:** the unchanged `cvalue` production moves from
  GRAM-3 into CONST-2, its unique semantic owner. This is the same
  correctness-selected META-4 repair and changes no value semantics.
- **Continuing chain and debt:** the T1 total-value, immutable-rodata, and P0
  read-only-fact chains continue. Struct- and enum-typed consts remain deferred.
  No v0.8 discrepancy record is claimed closed by this move.

#### DIAG-1 — derived

- **Delta and selection ground:** R4 requires a rule-citing rejection, W1
  requires a stable repair location, and W3 requires deterministic bytes. The
  closed `SourceBytes` / `SourceNode` / `BundleRoot` sum, frontend stage order,
  quote-aware raw spans, source-EBNF failure machine, expected-terminal order,
  and tree-owned FORM-2 gap location are correctness-selected ways to make
  those requirements truthful before and after a tree exists.
- **Evidence and debt:** independent candidate-bound frontend models must agree
  in `grammar-verifier/evidence/frontend-boundary-evidence.json`. Ordering for
  later semantic and target failures remains unselected. This amendment
  addresses `discrepancy:v0.8/diag1-pre-tree-node-path` only after v0.9 is
  installed; the v0.8 record remains immutable history.

#### EX-1 — derived_existence_only

- **Delta and selection ground:** only the enum declaration is re-rendered into
  the executable FORM-2 block shape. Program meaning and the selected match/give
  idiom do not change. This is a formatting-only correctness repair.
- **Continuing chain and debt:** EX-1 still inherits the R3-provisional surface
  debts of the constructs it demonstrates. It is part of the FORM-2 migration,
  not a new semantic claim or an expected-verdict change.

#### FN-1 — derived

- **Delta and selection ground:** the complete-signature chain from W1, P0,
  D1a, and the explicit-fact rulings continues. D24/A-01 additionally selects
  every top-level function signature as visible throughout the completed closed
  unit, so forward calls and mutual recursion do not depend on traversal order.
- **Continuing debt:** this does not make locals, regions, labels, generic
  parameters, or named constants globally visible. It does not resolve contract
  member semantics. A-01 is an owner ruling, not a v0.8 discrepancy ID.

#### FN-4 — derived

- **Delta and selection ground:** W3 requires every writer-stated law to be
  checked rather than trusted; R4 requires an unavailable discharge to fail
  closed; P0 and R0 are supported by the measured checked-law reassociation
  channel. Source acceptance therefore uses one mandatory, compiler-independent
  calculus: exact contract/member/conformance binding, an exact nongeneric
  two-`own D` pure signature, and a bound body containing only the direct
  `return iadd.sat<D>(p0, p1);` shape. The closed integer table defines totality,
  domain equality, the unsigned holds cells, the signed holds/refuted cells,
  and exact zero identity. No optional prover can accept another source shape.
- **Authority boundary:** a successful source discharge emits one canonical
  base derivation record but grants no optimization consequence. A law can
  affect optimization only through a separately approved optional proposition
  family whose independent verifier rederives the exact relation from the
  accepted artifact and binds the artifact, target, backend, proposition, and
  consequence. Absence or failure of that optional path leaves acceptance,
  semantic identity, checks, and empty-overlay lowering unchanged. The gated
  ledger remains a separate source of candidate propositions, not a source
  `conform` discharge. Static-proof menus, general source proof artifacts,
  runtime enumeration, and sampling are not v0.9 admission routes.
- **Evidence and debt:** the exact initial slice is grounded by
  `experiments/checked-law-channel/RESULTS.md` and the protected discharged,
  refuted-signedness, and undischarged FN-4 cases; the fact channel still
  requires hostile review before shipment. Additional operations and complete
  proof calculi are deferred specification additions. Most importantly,
  FN-4's local obligation relation does **not** define whole-conformance member
  completeness, extra/missing binding behavior, law-free conformance behavior,
  generic contract substitution, or behavior-parameterized calls.
  `discrepancy:v0.8/fn3-contract-member-semantics` remains open. This amendment
  addresses `discrepancy:v0.8/fn4-law-admission` only after installation.

#### FN-8 — derived_existence_only

- **Delta and selection ground:** grammar accepts ordinary typed `doc | stmt`
  children, then one early FN-8 pass requires ordinary lets followed by exactly
  one final check before recursively checking children. This correctness-
  selected boundary gives every excluded shape one deterministic owner without
  creating a requires-only parser. FORM-3 owns misuse of fixed `requires` as an
  identifier.
- **Continuing chain and debt:** the measured callee-entry semantics, retained
  check, and downstream proof chain continue. The block spelling remains
  R3-provisional, and contract/refinement use remains deferred. This contributes
  to resolving `discrepancy:v0.8/fn8-reserved-rule-attribution` after install.

#### FORM-2 — derived_existence_only

- **Delta and selection ground:** FORM-1 and W3 require a total byte format.
  The proposal makes it executable by rendering each source's ordered forest of
  top-level item subtrees from the one compilation-unit tree, with closed line,
  indentation, attachment, block, and source-boundary rules. The one-tree and
  per-source ownership mechanics are correctness- and evidence-selected.
- **Evidence and debt:** the primary and independent structural reports and the
  exact protected migration must bind the final candidate. The specific visual
  formatting conventions remain R3-provisional because no writer-tier format
  comparison selected them. This addresses
  `discrepancy:v0.8/form2-protected-conformance-spacing` only together with the
  separately approved protected migration; no verdict change is implied.

#### FORM-3 — derived_existence_only

- **Delta and selection ground:** deterministic grammar terminals and META-2
  require a context-free partition. IDENT now excludes the complete
  mechanically extracted set of exact fixed lowercase grammar words, removing
  fixed-word call/binding derivations without ordered-choice priority. The
  OPNAME explanation is aligned with that exclusion. The exclusion is evidence-
  selected by the two grammar engines and protected FORM-3 attribution.
- **Continuing debt:** casing and sigil choices retain their earlier lexicon and
  writer-tier debt. This contributes to resolving
  `discrepancy:v0.8/gram-terminal-ident-partition` and
  `discrepancy:v0.8/fn8-reserved-rule-attribution` after install.

#### FORM-4 — derived_existence_only

- **Delta and selection ground:** the `doc` production owner reference changes
  from GRAM-3 to its actual unique owner, GRAM-2. This is a minimal correctness
  erratum under META-4 and changes no construct.
- **Continuing debt:** the no-comments and doc-field choices remain
  R3-provisional. This addresses
  `discrepancy:v0.8/form4-doc-cross-reference` after install.

#### FORM-5 — derived_existence_only

- **Delta and selection ground:** one-spelling and no-undefined-value rules
  require a total host-independent finite-float contract. Exact rational input,
  signed zero, IEEE round-to-nearest-ties-to-even, shortest byte length, and
  unsigned-ASCII tie-breaking are correctness- and evidence-selected.
- **Evidence and debt:** exact-rational and independent Rust checks in
  `grammar-verifier/evidence/float-canonicality.json` must bind the final
  candidate. Decimal-only literals, no boolean literals, and other inherited
  surface choices remain R3-provisional. Together with FORM-7 this addresses
  `discrepancy:v0.8/form5-form7-float-canonical-spelling` after install.

#### FORM-7 — derived

- **Delta and selection ground:** T2, R4, W3, and FORM-1 already require finite,
  non-silent, canonical numeric values. Requiring the unique FORM-5 spelling
  closes the former contradiction and is correctness- and evidence-selected.
- **Continuing debt:** no host parser or formatter is language authority. Future
  literal-family additions remain separate specification deltas. This shares
  the FORM-5 discrepancy disposition above.

#### GIVE-1 — derived_existence_only

- **Delta and selection ground:** hostile review found that the prior recursion
  could treat an inner value match, a may-trap operation, or an unproved loop as
  outward delivery. Correctness selects the exact repair: an inner value match
  delivers only to its own let; only a final statement match whose arms deliver
  relative to the same outer value match recurses outward; a may-trap check or
  call retains a continuing edge; no loop is assumed to diverge; and a break
  counts only when its resolved loop lexically encloses that same value match.
- **Continuing debt:** the explicit `give` surface remains R3-provisional. This
  is a semantic contradiction repair found during successor review, not a claim
  to close a separately registered v0.8 discrepancy ID.

#### GRAM-1 — derived

- **Delta and selection ground:** W3, FORM-1, DIAG-1, and META-2 require one
  context-free derivation. The exact maximal raw forms, predicate-valued terminal
  membership, full matching-predicate retention, pairwise-disjoint strong-LL(2)
  `SELECT_2` languages, and one-production/one-node rule are correctness-
  selected and independently executable. Quoted fixed atoms expand inside the
  specification to their unique raw-token sequence: `"&uniq"` counts as `&`
  then `uniq`, while `"->"` and `"=>"` each count as one compound token. This
  leaves exactly one pattern atom: quoted `"[0-9]+"` in `const` denotes one
  complete numeric-form token matching that pattern and is not a fixed atom.
  These self-contained rules make the two-token bound a statement about formed
  tokens and prevent an unreviewed tool-local atom table. Predicate priority
  and parser-local keyword lists remain forbidden.
- **Evidence and debt:** both grammar engines must rerun against the final
  candidate and agree on complete static and bounded generalized-parser
  evidence, including identical fixed-atom expansion and sole-pattern-atom
  classification; the generalized parser remains an evidence tool, never
  production authority. This contributes to resolving
  `discrepancy:v0.8/gram-terminal-ident-partition` and
  `discrepancy:v0.8/gram1-gram7-match-node-bijection` after install.

#### GRAM-2 — derived_existence_only

- **Delta and selection ground:** law arguments and requires entries parse as
  ordinary syntax before their semantic owners check them; `law_arg` and
  `requires_entry` make that boundary explicit. CONST-1 and CONST-2 become the
  unique owners of their unchanged grammar definitions. This factoring is
  correctness-selected where required for strong-LL(2), and otherwise
  minimality-selected to preserve one general parser.
- **Continuing debt:** contract/conform syntax and doc fields inherit their
  existing R3 debts. In particular, broad parsing plus FN-4 does not resolve
  `discrepancy:v0.8/fn3-contract-member-semantics`.

#### GRAM-3 — derived

- **Delta and selection ground:** the type, mode, targs, and targ productions
  keep their source shapes; duplicate `const` and `cvalue` definitions move to
  CONST-1 and CONST-2. Unique semantic ownership is correctness-selected under
  META-4.
- **Continuing debt:** region vocabulary and composite-type debts are unchanged.
  Fixed-terminal partition evidence is owned by GRAM-1/FORM-3, not by an
  invented GRAM-3 priority.

#### GRAM-4 — derived_existence_only

- **Delta and selection ground:** the complete let prefix precedes one explicit
  choice among `ordinary_let_rhs`, `try_let_rhs`, and `value_match`; statement
  and value matches are distinct productions. This factoring is evidence-
  selected by the strong-LL(2) and one-derivation requirements.
- **Continuing debt:** loop and match surface choices remain R3-provisional.
  Broad `requires_entry := doc | stmt` is checked by FN-8 and does not give
  excluded statements requires semantics. This contributes to the GRAM-7 node
  discrepancy disposition.

#### GRAM-7 — derived_existence_only

- **Delta and selection ground:** one-production/one-node requires distinct
  `match_stmt` and `value_match` node kinds in their disjoint source positions.
  This is correctness- and evidence-selected; a shared node kind is not an
  allowed normalization.
- **Continuing debt:** the contained let-initializer match and explicit `give`
  spelling remain R3-provisional. This addresses
  `discrepancy:v0.8/gram1-gram7-match-node-bijection` after install.

#### PRE-1 — derived_existence_only

- **Delta and selection ground:** only the byte layout of the existing prelude
  declarations changes so the normative fence is an exact FORM-2 rendering.
  Types, variants, fields, contracts, and conformer sets do not change.
- **Continuing debt:** Bool-as-prelude-enum and the other earlier prelude form
  debts remain open. This is formatting-only and proposes no semantic or verdict
  change.

#### PROG-1 — derived

- **Delta and selection ground:** the closed-world P0/W3 chain continues while
  PROG-2 becomes the unique unit-formation owner. The rule expressly preserves
  every ban: include, import, module, separate compilation, incremental semantic
  cache, internal ABI, dynamic loading, reflection, and source-path lookup.
- **Continuing debt:** a logical path contributes identity only and cannot
  become a namespace or lookup key. The whole-program check-loop latency debt
  remains. A-10 is an architecture question, not a v0.8 discrepancy ID.

#### PROG-2 — derived_existence_only

- **Delta and selection ground:** GRAM-1, PROG-1, DIAG-1, and A-01 require one
  exact answer for how several transported sources become one program. The rule
  therefore defines an ordered nonempty sequence of exact logical source
  records, portable paths, envelope failures, per-source derivation and FORM-2
  audit, no cross-record syntax, one flattened program root with
  `BundleRootExtent`, whole-unit declaration order, and identity-preserving
  empty records. Existence is architecture- and correctness-required; the
  precise path grammar and nonempty ordered-record form are minimality-selected.
- **Evidence and debt:** independent frontend-boundary evidence must cover
  invalid/duplicate paths, zero records, zero-byte versus one-LF sources,
  reorder/repartition distinctions, root extent, and cross-source isolation.
  Future modules or separate compilation would be new specification decisions.
  This resolves A-10 only after exact approval and installation; A-10 is not a
  v0.8 discrepancy ID.

#### TYPE-6 — derived_existence_only

- **Delta and selection ground:** D24/A-01 selects a total visibility table:
  every top-level function signature is visible throughout the completed closed
  unit, while every other declaration remains visible only after its lexical
  declaration. This removes traversal-order semantics from forward calls and
  mutual recursion without broadening other namespaces.
- **Continuing debt:** no-shadowing remains R3-provisional, and constructor/type
  collision questions remain separate. This does not authorize or define
  general contract-member resolution. A-01 has no v0.8 discrepancy ID.

### Discrepancy and evidence boundary

After, and only after, exact v0.9 installation plus its separately approved
protected migration, these eight immutable v0.8 records have v0.9 dispositions:

- `discrepancy:v0.8/diag1-pre-tree-node-path`;
- `discrepancy:v0.8/fn4-law-admission`;
- `discrepancy:v0.8/fn8-reserved-rule-attribution`;
- `discrepancy:v0.8/form2-protected-conformance-spacing`;
- `discrepancy:v0.8/form4-doc-cross-reference`;
- `discrepancy:v0.8/form5-form7-float-canonical-spelling`;
- `discrepancy:v0.8/gram-terminal-ident-partition`; and
- `discrepancy:v0.8/gram1-gram7-match-node-bijection`.

Their v0.8 records are never rewritten. A versioned v0.9 discrepancy set must
record the installed dispositions. The seven other registered v0.8 gaps remain
unresolved: affine-deref storage lifecycle, retained-check `proof_ref`, EFF-1
row canonicality, body-local region effects, FN-3 contract-member semantics,
`main` return spelling, and dotless-operation reservation. This amendment also
does not settle A-02 through A-09 or A-11 through A-18.

Before installation, all of the following must bind the exact final candidate
hash: the two grammar-engine report, float-canonicality report, primary and
independent FORM-2 reports, frontend-boundary report, protected-surface census,
and exact protected migration. A green development gate is evidence, not owner
approval, and no production parser receives authority from this ledger text.

<!-- END EXACT V0.9 DERIVATION-LEDGER APPEND -->
