# Frontend boundary rationale

Status: non-authoritative design evidence. The exact proposed rule bytes live
only in `kernel-spec-successor-candidate.md`. This file explains those bytes;
it does not define a second copy of DIAG-1 or PROG-2, change v0.8, or authorize
implementation.

## PROG-2 decisions

The candidate binds one compilation unit to an ordered, nonempty sequence of
logical source records. Each record has an exact portable logical path and an
exact byte sequence. The order comes from the bound invocation and is never
derived from path spelling or host filesystem order.

The distinction between envelope and language failures is intentional:

- zero records, an invalid path, and a duplicate path are envelope failures;
- a zero-byte source is a valid record, then fails canonical FORM-2;
- one LF byte is the sole canonical source containing no items; and
- invalid UTF-8 is a source-language rejection after the record is bound.

Each record derives its own `item*` sequence. The compilation root flattens
those items by source ordinal and local item order. No production, token,
trivia, or span crosses a record boundary, and record containers are not
grammar nodes. `BundleRootExtent` is an ordered sequence of per-record byte
extents, not a fabricated span across sources.

Logical paths preserve transport identity but create no namespace, scope, or
lookup behavior. Declaration order follows record order, subject to the exact
FN-1 and TYPE-6 visibility rules.

## DIAG-1 decisions

The candidate uses a closed location sum because no one location shape is
truthful at every stage:

- `SourceBytes` is available before a tree or for a source boundary;
- `SourceNode` binds an existing canonical node to its source coordinate; and
- `BundleRoot` identifies a whole-unit failure without inventing a source.

Frontend processing is stage-first. A stage scans sources in ordinal and byte
order; the next stage starts only after the current stage succeeds everywhere.
This makes a lexical defect in a later source win over a grammar defect in an
earlier source, matching the production pipeline and avoiding implementation
order as accidental policy.

The candidate closes attribution for raw lexical failures, requires a
grammar-ordered nonempty expected-terminal set on every grammar rejection, and
defines the exact owner of a FORM-2 separator mismatch. Envelope, resource,
invariant, artifact, backend, and external-tool failures never cite a language
rule. Ordering for later semantic and target stages remains separately gated;
the frontend proposal does not invent it.

## Alternatives rejected

- Byte concatenation permits productions and spans to cross source boundaries.
- Path sorting makes metadata control language order.
- A source-less empty program has no FORM-2 source and no truthful source
  location.
- Synthetic pre-tree nodes claim a derivation that does not exist.
- One umbrella parse rule discards the FORM or GRAM rule that rejected input.
- A global semantic ordering in this frontend change would silently settle
  later architecture questions.

## Evidence required before approval

The final owner packet must independently bind all of the following to the
candidate hash:

1. invalid paths, duplicate paths, zero records, and resource failures remain
   non-language envelope outcomes;
2. zero-byte and one-LF sources have distinct required results;
3. source order, path order, record repartitioning, and empty-record retention
   cannot alias;
4. functions may refer across the complete unit while named constants and
   lexical declarations retain their approved order rules;
5. a production, token, trivia item, or span cannot cross a record boundary;
6. the compilation root has exact child order and byte extents;
7. every raw lexical class has the proposed FORM attribution and coordinate;
8. grammar failures have the exact GRAM attribution and expected terminals;
9. FORM-2 missing, excess, and wrong separators have exact locations; and
10. missing `main`, duplicate declarations, resource failures, and invariant
    failures use distinct location and authority families.

Additive multi-source evidence needs one canonical descriptor whose schema,
guard, runner, and independent model bind the same ordered logical paths and
bytes. Existing protected single-source cases and verdicts must remain
byte-identical unless the owner separately approves an exact migration.

Any change to the candidate rule bytes invalidates evidence generated against
an earlier candidate hash.
