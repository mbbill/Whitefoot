# Test-system simplification

This investigation records the selected changes and remaining questions for
replacing redundant verification machinery. The [inventory](test-inventory.md)
and its [Chinese translation](test-inventory.zh-CN.md) describe the existing
system; this document describes subsequent work, not implemented behavior.
Keep it current during that work and retire it when the replacement system's
guidance and design decisions cover these choices and no questions remain.

## Selected direction

The discussion on 2026-09-15 selected these changes for later implementation.
First remove redundant assertions, derived copies and unnecessary test/tool
boundaries while retaining useful correctness and performance-regression
checks. Defer timing optimization and new timing campaigns until those
responsibilities are clear. Required correctness validation still applies;
existing execution guards remain in force.

1. **Remove the Rust specification tool.** Delete `whitefoot-spec`, its
   Cargo binary/test target and its gate invocation. Fold useful document
   checks into the existing compiler-independent Python conformance tooling:
   duplicate rule definitions and unresolved rule references, with the
   necessary malformed-input tests. That tooling already reads the active
   Markdown and extracts rule IDs. Remove redundant same-source identity
   comparisons rather than translating them. Keep the build-derived identity
   and specification embedding used by the compiler. Do not automatically
   reproduce `--index`/`--counts`; no automatic caller was found, and a
   retained query needs a concrete consumer.
2. **Remove the specification's Status line.** The active path already names
   the active specification and the title already names its version.
   Delete `Status: ACTIVE vN` and the checks for that duplicate field.
   `build.rs` already reads the version from the title. Update the
   `AGENTS.md`/`CLAUDE.md` wording about changing two version lines and
   affected fixtures/comments. This changes specification bytes, so use the
   normal version/archive procedure against the then-current main; preserve
   existing archives. No WF syntax or semantic change is intended.
3. **Generate parser tables during the build.** Retain the Rust EBNF/table
   derivation logic, invoke it from `build.rs`, write the generated table to
   Cargo's `OUT_DIR`, and include that output in the compiler. Remove the
   independent `whitefoot-grammar-tables` binary/test target, committed
   `compiler/src/syntax/grammar/generated.rs` copy and the test that merely
   regenerates and compares that copy. Preserve tests of actual parsing and
   diagnostics. Track generation inputs and keep generation independent of
   the compiler library it produces data for. Automatic derivation removes
   stale-copy maintenance; it does not prove the generator algorithm correct.

The [proposed build-input decisions](../../../design/amendments/compiler-build-inputs.md)
record the corresponding tree amendment. The live tree has not changed and
the exact tree revision has not received an owner ruling. The earlier
verification-cost amendment remains separate and unchanged.
The proposed `design/compiler/build-inputs.md` is a new child of the
compiler root and replaces no existing live node. Corpus case retirement
and the replacement integration-target layout are outside that amendment.

## Selected corpus responsibilities

The four current integration executables do not establish four necessary
physical test boundaries. The owner also selected the following four changes
on 2026-09-15 and explicitly deferred execution while discussion continues:

- `conformance`: concentrate specification-derived acceptance, rejection
  and runtime requirements here, with the specification determining the
  expected result.
- `programs`: keep whole-program functionality, host interaction and
  parallel behavior here; remove checks demonstrated to be fully covered
  elsewhere.
- `canonical_corpus`: retain batch canonical rendering/idempotence and exact
  normative-example checks without a dedicated test executable; rendering
  checks still need compiler functionality.
- `snapshot`: review each historical verdict against the active spec.
  Migrate unique useful cases with justified expectations; retire proven
  duplicates or obsolete historical expectations. Do not assume an old
  verdict is correct or delete all 484 cases as one unexamined group.

Compare input, property, failure mode and oracle before declaring coverage
redundant. Consolidating Rust targets is packaging; it does not itself remove
duplicate assertions or WF compilations. The selected direction is not
blanket case-retirement approval: individual migrations and retirements still
need that examination, and the final target layout remains to be worked out.

## Affected material and evidence

- Specification identity: `compiler/build.rs`, `compiler/src/spec.rs`,
  `compiler/src/bin/spec.rs`, and the title/header of
  `spec/kernel-spec.md`. Current hash generation and recomputation share
  the same SHA-256 implementation; comments claiming independent recorded
  `shasum` evidence are stale.
- Parser construction: `compiler/src/bin/grammar_tables/`,
  `compiler/src/syntax/grammar.rs`, `compiler/src/syntax/grammar/generated.rs`
  and its parser/finalizer/diagnostic consumers.
- Invocation and coverage: Cargo/Make/CI callers,
  `tests/conformance/runner.py`, its tests, and the manifest's
  `whitefoot-spec` coverage annotations. Reassign honest coverage claims;
  do not leave a deleted checker named as evidence or invent a check of
  semantic properties/PR content the replacement does not perform.
- Update current guidance and both inventory languages to the resulting
  system; retain dated measurements with their original revision boundaries.

No implementation, test retirement, specification edit or new performance
measurement accompanies this record.
