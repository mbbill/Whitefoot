# Batch 0102 — claim authority selects definitions, not positions

Branch: `batch/0102-clm1-narrow`, from `integration/2026-08-28c` at `100b37cd`
(main `10b76c66` plus batches 0093–0100, 0096 and 0103). Deliverables: kernel
specification v0.39 with its activation, the narrowed [CLM-1] claim-locality
judgment, seven conformance cases, the library tests, and this record.

Approval classes: **specification bytes change — yes**; **conformance content
change — yes**; no new root entry.

## The decision

Batch 0097's differential-fuzz campaign discarded 63 generated programs over
2004, and all 63 carried one diagnostic kind: `NonLocalClaim`. Its minimized
pair, recorded in `docs/done/0097-differential-fuzz.md` under "The rejection
tally", differs by one line — an early `return` in the `Err` arm of a `match`
on a system-call result — and receives opposite verdicts. After that return the
definitions `let seed = 3209_u64;` and `let offset = seed % 64_u64;` are
post-join state reached through the `Ok` edge, so `claim guard: ilt(offset,
64_u64)` was refused as `[CLM-1]` non-local with carrier `offset`, although the
claim's truth reads nothing the system returned.

The 0097 record was right that this was not a compiler defect: v0.38 named
"post-join state" among the things a selector's witness joins, and the compiler
followed the rule. The owner ruled that the *rule* was too wide, and ruled
NARROW rather than repeal: control dependence still counts, but only where the
selector actually chooses among reaching definitions.

## The specification delta (v0.38 → v0.39)

Exactly one paragraph of [CLM-1]'s claim-authority state changes. Nothing else
in the specification does: 138 rules, 75 grammar productions, 203 system
operations, and every other counted surface are unchanged.

**Old (v0.38), three sentences:**

```text
Claim authority deliberately includes control dependence although [PRV-1] provenance does not.
When a `BoundaryResult` condition, match scrutinee or tag, counted endpoint, or other selector chooses an edge, its witness joins every binder, delivered value, or storage write whose reaching definition is selected by that edge, including `value_if`, `value_match`, ordinary match, `give`, loop-carried updates, and post-join state.
Thus selecting constants on the two arms, selecting the same local value on both arms, or writing a local constant only under boundary-selected control does not declassify the resulting value or storage.
```

**New (v0.39), seven sentences:**

```text
Claim authority deliberately includes control dependence although [PRV-1] provenance does not, and it includes exactly the control dependence a selection carries.
A `BoundaryResult` condition, match scrutinee or tag, counted endpoint, or other selector chooses an edge; its witness joins each matching binder that edge's arm introduces, each value `value_if` or `value_match` delivers along it, and, at each ordinary match reconvergence, loop head, and loop exit the selector reaches, exactly those components whose reaching definition on one incoming edge is a different definition occurrence from their reaching definition on another.
Two reaching definitions are the same occurrence when they are one definition of that component, not when two separate definitions compute equal values; `value_if` and `value_match` deliver a selected value in every case, so selecting constants on the two arms or selecting the same local value on both arms does not declassify the delivered value.
Standing on a boundary-selected edge is not itself a selection.
A binder, delivered value, or storage write whose own operands are every one Local — a literal, a named const, an ordinary parameter, or another Local value — is Local, and stays Local across a reconvergence, loop head, or loop exit whose every incoming edge reaches it through that one definition, whether it stands inside the selected arm or in post-join state.
Thus writing a local constant on one arm and joining it with the other arm's older definition, and updating loop-carried state under a boundary-selected iteration, each retain the selector's witness at the join, while a definition formed after the join from literals, named consts, parameters, and other Local values is Local although a boundary result selected the edge that reaches it.
So a `match` on a system-call result whose `Err` arm returns leaves a following `let seed = 3209_u64;` and `let offset = seed % 64_u64;` Local, and `claim guard: ilt(offset, 64_u64)` is admitted; the same claim over a value that reads the delivered payload, a binder joined from two arms, storage the selected edge wrote and the other edge did not, or state a boundary-selected loop updates remains non-local and is refused.
```

The last sentence is the worked example the fuzzer minimized, stated in prose.
The specification has one fenced worked example, §19 [EX-1], and it is about
canonical bytes; a judgment rule here states its examples in a `Thus` sentence,
which is the form used.

[PRV-1] provenance is untouched — it never included control dependence. Every
other [CLM-1] clause and every [CLM-2] and [CLM-3] clause is untouched,
including the component tree, the unconditional call-result seed, the witness
identity and its tie-break, the protected families, and the constrained
subjects.

### Activation and the digest chain

- v0.38 archived byte-exact as `spec/kernel-spec-v0.38.md`, verified at its
  recorded digest `5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`.
- v0.39 active at `4be4830fa87a534879de17524599b0919aef4dfab072dad823bf2f9b54d32d58`.
- `governance/APPROVALS.md` carries the merge-time record and the chain line
  `ACTIVE-SPEC: v0.39 4be4830f… 5a43c763…`, taking the chain to 31 activations.
- `compiler/src/spec.rs` carries the two transcribed literals (the byte array
  and the `shasum` string); `compiler/src/spec_identity.rs` was regenerated by
  `cargo run --bin whitefoot-spec -- --emit-identity src/spec_identity.rs`.
- The six digest anchors `make spec-digest-sync` names — `README.md`,
  `compiler/README.md`, `docs/roadmap.md`, `docs/current-plan.md`,
  `docs/patterns.md`, and `spec/derivation/derivation-ledger.md` — name v0.39;
  the ledger gains its v0.39 amendment section.
- `compiler/src/backend/qualification.rs` `REVIEWED_FOR` is `v0.39` with a
  dated note: the narrowing is a front-end source-acceptance judgment over one
  function's own definitions and names no target facility, so every v0.38
  mapping stands.
- `make spec-append-only spec-archive-integrity spec-digest-sync
  approval-history-integrity` are green, as is `cargo run --bin whitefoot-spec`.

If another branch activates v0.39 before this merges, the integrator re-chains:
the outgoing digest, the chain line, the two transcribed literals, the
generated identity, and the six anchors are the complete set of places a
re-chain touches.

## The judgment

`compiler/src/semantic/claim_locality.rs`. The pass already computed a forward
authority state with a `ControlAuthority` of lexical frames. v0.38 spent that
authority twice — it unioned the live control witness into every value a `let`,
`set`, `replace`, `give`, or match binder defined, and it seeded every claim
query with the control witness live at the claim. Both are the position clause,
and both are gone.

What replaces them is the rule's own vocabulary. Every value component now
carries a `DefinitionId`, the identity of the reaching definition that produced
it, derived from the address of the checked statement that wrote it plus a kind
discriminator. Addresses make the identity deterministic across the loop
fixed point's repeated walks of one body, which is what keeps the fixed point
converging; the identity is scratch, compared only with another identity of the
same component of the same binding, and never rendered, published, or ordered.
`materialize` gives new children their parent's identity, so a component one
arm never touched still compares equal after either side has been split.

`AuthorityValue::merge` is the new control-flow join. Where the two incoming
identities differ, the merge unions the selection witness and stamps the merge's
own identity; where they agree, it recurses and takes only the ordinary
authority union. `ControlAuthority::acquired(base, edges)` supplies the
selection: the earliest witness among the frames the incoming edges hold and
the merge's entry state does not. That is the boundary decisions taken between
the merge's dominator and the merge, and nothing else.

Three sites keep an explicit unconditional selector, because the rule states
them as selections in every case: the matching binder (its own arm's tag), the
`value_if`/`value_match` delivered value (its own selector), and the counted
binder (its endpoint). Each was narrowed from the whole live control to that
one selector, so an outer boundary arm no longer taints them.

Frame *removal* is deleted along with the exhaustiveness bookkeeping that drove
it — `unrepresented_exit`, the exhaustive-break count, the
exhaustively-delivering-arm count, and the three discharge loops. A frame the
merge's entry state already carries is excluded by `acquired` on its own, so a
discharge that only affected reachability had nothing left to affect. This is
strictly fewer special cases on one path, not a second path.

`entailment/flow.rs`'s `claim_locality_failure` loses its control-witness
fallback and the `control_fallback` tie-break bit that existed only to let a
value support outrank a control frame at an equal node path. A component now
carries a boundary result only through a support it reads.

Fail-closed remains the disposition everywhere it was: an unclassifiable value
is `InvalidResolution`, a merge of values whose types or shapes disagree is
`InvalidResolution`, a partial or possible-overlap write joins and stamps rather
than replacing, and a claim reached by two states merges them under the whole
control either state stands under.

## Conformance

Seven cases added, none modified, deleted, or renamed; the manifest gains seven
records and loses none; coverage stays 138/138. The exact boundary is in
`governance/APPROVALS.md`.

Accept, the verdicts the amended sentence moves:

- `accept-clm1-local-claim-after-boundary-exit` — the fuzzer's minimized member
  with the early `return`. Refused by v0.38 with carrier `offset`; admitted now.
- `accept-clm1-local-claim-after-boundary-join` — the other member, differing by
  one line. Accepted under v0.38 as well; the pair is kept together so the
  corpus states that the line no longer separates the verdicts.
- `accept-clm1-local-claim-inside-selected-arm` — a claim over a
  parameter-derived value inside the arm a call-result condition selected.

Reject, the selections the amended sentence retains:

- `reject-clm1-claim-on-selected-payload` — the matching binder is the payload
  the tag delivered.
- `reject-clm1-claim-on-delivered-selection` — a `value_if` on a call-result
  condition whose two arms both give a literal below the bound.
- `reject-clm1-claim-on-storage-written-under-selection` — one arm writes the
  place and the other does not, so the reconvergence chooses between two
  different reaching definitions.
- `reject-clm1-claim-on-loop-carried-update` — a counted loop whose endpoint is
  a call result updates loop-carried storage.

No pre-existing verdict changes. Every verdict this batch states is stated by
the amended sentence quoted above, and no verdict moved for any other reason.

## Tests

`compiler/src/semantic/tests/claim_locality.rs`. Ten tests asserted the
repealed clause. None is deleted or disabled; each is rewritten to the verdict
v0.39 gives, with the reason in a doc comment:

- nine become acceptance tests — a claim inside a selected arm, a partial arm
  continuation, a nested partial return, a delivery with a returning arm, a
  claim in a counted body, three post-loop shapes (returning body, partial
  return, propagation), and an ordinary loop with a boundary-selected break;
- one, `control_authority_rejects_a_component_without_binding_supports`, becomes
  `a_local_named_const_component_reaches_the_redundancy_judgment`. Its claim is
  `ieq(four, 4_u64)`: with the position clause gone the component is local, so
  the occurrence reaches the next judgment and [CLM-2] refuses it as redundant.
  That is the honest verdict for a component the checker decides itself, and it
  is the direct evidence that a control-only [CLM-1] rejection no longer exists.

Six tests are added for the retained selections and the narrowed edges: a write
on one arm only is selected at the join; a definition formed after that same
join from literals is local; a counted endpoint selects loop-carried state; an
ordinary loop selects state its iterations wrote; a matching binder is selected
by its own tag; and a `value_if` whose own condition is local delivers a local
value even inside a boundary-selected arm.

`cargo test --profile gate --lib`: 1425 pass. `cargo clippy --all-targets` and
`cargo fmt --check` are clean.

## Evidence that nothing else moved

- **Conformance IR and diagnostics.** All 510 pre-existing conformance cases
  compiled with the base compiler at `100b37cd` and with this revision:
  510 identical, 0 IR differences, 0 status differences, 0 diagnostic
  differences. The only byte that differs anywhere is the QUAL-1 banner's
  version token, which an activation is expected to move.
- **Program corpus IR.** Every source under `tests/programs` and
  `tests/codegen` (26 files, 20 emitting a module): byte-identical LLVM IR
  apart from the same banner token.
- **Native conformance adapter.** `Pass=516 Skip=1` over the 517 declared
  cases, the skip being the one declared pending case.

## The fuzzer re-run

CAMPAIGN-NUMBERS-PLACEHOLDER

## Judgment calls

JUDGMENT-CALLS-PLACEHOLDER

## Not done

NOT-DONE-PLACEHOLDER
