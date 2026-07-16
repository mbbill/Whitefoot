# M1 rule-text / machine-spec amendments for human review

The repair round changed checker semantics in ways that require frozen rule text
and `machine_spec.md` to be amended. These are NOT applied to the repo: spec /
fact-channel text is a review-gated artifact and this round produced no commits.
The checker in the work dir already implements each amendment; a human must
ratify the text before it lands.

Each item: rule number -> one-line change.

1. R14 (PAR-1) + machine_spec [par calls] and [Mint legality]
   ADD a statement-local mint-disjointness clause: every mint of ONE statement
   is accumulated as a pseudo-entry (split / &uniq -> uniq; replicate / & ->
   shr); any two mints on overlapping places where at least one is unique are
   rejected; replicate+replicate (shr+shr) on the same place stays legal. This
   is the sole authority for par cross-slot disjointness AND closes same-
   statement aliased &uniq mints in ordinary calls (E2d / S1a). Previously R14
   and [par calls] checked each slot only against the pre-existing loan table,
   recording no per-mint entry, so two slots never saw each other.

2. R6 (LOAN-3) + machine_spec [Mint legality]
   ADD a mode-capability clause: a &uniq mint (or own-pass) of a place whose
   root binding was borrowed &-shared is rejected; only an owned local or a
   &uniq-borrowed source can be uniquely accessed. This removes M1's silent
   dependency on the base ownership (OWN) layer -- previously mint legality
   consulted only the loan table, so a `&`-shared parameter could be
   &uniq-minted inside a body (BREAK-1).

3. R9 (SIG-1) + machine_spec pre-pass (b)
   REPLACE the undocumented `is_form => SELF` carve-out with an explicit
   form-op receiver-holder tie: a form-table op's confined receiver region with
   no distinct borrow candidate ties to the receiver holder itself; an
   `issues K on source(receiver)` clause issues the result on that receiver
   holder's recorded source. The receiver-holder tie skips the distinct-place
   R10a comparison but STILL applies an identity brand -- the token must hold a
   live entry at its recorded source (a genuine issued token), a local confined
   value with no recorded source reaching a form op is rejected as forged, and
   a self-tied parameter is trusted from the signature. This makes R9's
   sentence "form ops are subject to the same call-site checks as derived
   signatures" true, and restores the R10a coverage the carve-out silently
   dropped.

4. R12 (JOIN-1) text vs machine_spec [match]
   RECONCILE the comparison domain: R12 rule text says "holder liveness" while
   the machine spec (and the checker) compare identical liveness of ALL
   bindings declared outside the construct, plus loan-table entries INCLUDING
   holder names. Amend the R12 rule text to state the machine-spec domain.

5. (RECOMMENDED, not applied) R15 (FORM-1) / machine_spec form-table pre-pass
   Fail closed on provably-dead issues-on-source declarations: reject an
   `issues K on source(receiver)` op whose result kind is uniq or whose
   receiver kind is uniq at declaration (it can never legally fire -- the
   receiver's own live entry always blocks a sibling uniq issue). NOT applied:
   the expressiveness probes E5c / E7b currently document these rejecting at
   the call site under R5, and leaving that intact preserves their frozen
   reason. Apply only if the shr-from-shr-only restriction is written into the
   spec at the same time.

Implementation-only fixes (no rule-text change; the spec already mandates them,
the checker was diverging in the accept direction and now conforms):
  R7  confined bindings are dropped-and-overlap-checked at scope end (CE1).
  (iii) an own-mode confined arg to a clause-none form op is consumed (CE2).
  R11 the destructure gate keys on the binding's confined type, not the AST
      annotation (CE3).
  [par calls] slot arguments are liveness-checked (CE4).
  R1  the loanable >=1-region check runs at declaration for issuing form ops,
      not only at a call site (CE5).
  R13 (ESC-1) is now implemented in v_SReturn (was absent; A1 also tests it via
      the A1b-body variant returning a local-sourced token).
