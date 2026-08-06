# 0017 — Conformance harness v0.18 lane

Live coordination record. It reports how authorized work is being carried
out; it is not authority and cannot expand or resequence work.

- **Status:** IN PROGRESS
- **Owner / workspace:** executor agent / isolated worktree
  `worktree-agent-a9359311cd5a0fb00`, lead-reviewed
- **Base revision:** `8ecb736`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2
  ("first-slice conformance execution" bullet — the harness and compile-time
  lane; task 0014 retains runtime execution). Split from task 0014's scope so
  the compiler-independent harness work runs ahead of the compiler chain.
  This record authorizes nothing beyond Work item 2 itself.

## Goal

Make `tests/conformance` able to express and check the v0.18 first slice
without waiting for the compiler chain: extend the manifest schema and
`runner.py` for the case shapes the catalog needs, admit the `unsupported`
verdict, re-pin the corpus to the active `spec/kernel-spec-v0.18.md` with
coverage annotations for the 25 new rules, and land the compile-time lane of
the reviewed 40-case catalog.

## Direction and invariants

- Protected-surface law is absolute: no existing case source, verdict,
  expectation kind, or runnable status changes. New cases are additive. If
  any existing case's verdict would differ under v0.18, STOP and report (the
  activation evidence says none does).
- Schema extension: manifest gains a fixture-file list, an argv byte list, an
  optional stdin body, and a redirection description; `validate_manifest`
  accepts them and the `unsupported` verdict its own docstring already names.
  Existing manifest entries remain valid unmodified.
- The corpus pin moves to `spec/kernel-spec-v0.18.md` SHA-256
  `307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28`;
  coverage must report 119/119 rules covered once the new annotations and
  cases land.
- Compile-time lane only: cases expressible as source + expected
  reject/accept verdicts without executing I/O — the catalog's groups A
  (entry/kind/name-visibility), G (effects and release attribution, including
  the `return unit;` canonical case), the reserved-spelling and
  primitive-lookalike cases, and the kindless-unit cases. Runtime groups
  (B-F, I) stay with 0014.
- `expect` carries the real v0.18 spec verdict; toolchain readiness lives in
  `status`, with a `pending` reason naming the gating task and the exact
  current compiler stop. The `unsupported` expectation kind lands as a
  first-class schema feature reserved for the spec-level non-rejections —
  `QUAL-1` target-qualification failure and `QUAL-2` startup refusal — which
  are permanent language content. When the compiler tasks land, only
  `pending` flips to `runnable`; no expectation ever changes.
- Python here is legitimate (the corpus is deliberately compiler-independent
  tooling); keep `runner.py` self-contained.

## Method

Read `tests/conformance/runner.py` and the manifest format; design the schema
extension first (it gates every runtime case in 0014); then the `unsupported`
verdict; then the v0.18 pin plus per-rule coverage annotations; then
transcribe the compile-time lane from the reviewed catalog at
`/Users/bytedance/do_not_scan/wf-v018/conformance.md`, checking each case's
expectation against the actual `spec/kernel-spec-v0.18.md` rule text (the
catalog predates integration fixes).

## Progress

- Direction corrected by lead ruling 2026-08-06 — expectation kind is
  spec-verdict only; toolchain readiness lives in status (the as-registered
  instruction would have scheduled protected-surface edits).
- **Completed:** claimed at base `8ecb736`. Schema extension (`arrange` with
  hex-encoded argv, stdin, file/directory fixtures, and named redirection
  sinks; closed key set; only a `run`/`trap` case may carry one) and the
  `unsupported` expectation kind, both validated and unit-tested. Corpus
  re-pinned to `spec/kernel-spec-v0.18.md`. Compile-time lane landed: 24
  additive cases (8 group A, 3 reserved/unadmitted kind, 6 entry-form label
  near-misses, 7 group G) plus 13 coverage annotations for the rules with no
  compile-time source-to-verdict pair. Coverage reports v0.18 119/119
  (90 by case, 30 by annotation); `make check` green.
- Every case source was run through `whitefootc`: the four system-unadmitted
  cases produce their exact expected verdict today (accept, accept, OP-1,
  TYPE-5), and all 20 kind-declaring cases pass lexical formation, terminal
  membership, grammar derivation, and FORM-2 canonical rendering before
  stopping at the system-declaration-domain gate.
- **Current:** awaiting lead review.
- **Next (not this task):** 0014 builds its runtime lane on the `arrange`
  schema; 0007/0008/0009 flip the `pending` cases to `runnable`.

## Scope and expected touch set

- `tests/conformance/runner.py`, `tests/conformance/manifest.jsonl` (additive
  entries + schema), `tests/conformance/cases/` (new case files), the corpus
  identity/coverage metadata.
- Read-only: `spec/kernel-spec-v0.18.md`, the case catalog, `compiler/` (to
  confirm current unsupported surface only).

## Dependencies and integration order

- **Prerequisites:** none (v0.18 is active). Runs concurrently with 0006 and
  0007; no touched file overlaps them.
- Task 0014 depends on this task (schema + pin + compile-time lane) and on
  0012 (runtime execution); its record carries the mirror link.

## Validation

`make check` green with coverage reporting v0.18 119/119; `runner.py`
self-validation passes on the extended schema; every new case's expected
verdict cross-checked against the spec rule it cites. A claimed task lands
only through lead review per the executor lane.

Catalog-vs-spec divergences found while transcribing (spec wins in each):

- Catalog flag 3 calls Route C's same-spelling collision policy unsettled and
  says its group-A case cannot get a verdict. `SYS-1` settles it — the unit is
  rejected and neither declaration resolves — and `DIAG-1` places it at
  declaration-inventory rank 5, which cites `TYPE-6`, not `DIAG-1`. Drafted as
  `reject-sysname-collision-in-kind-unit`.
- The catalog derives the `ExitStatus`-from-`u8` rejection from the dossier's
  no-implicit-conversion prose. The activated spec routes the diagnostic
  through `FN-1` (return type differing from the written `rtype`), so the case
  cites `FN-1` with `SYS-13`/`TYPE-4` as exercised rules.
- The catalog's ID `accept-sysexit-implicit-conversion-rejected` names a
  reject case; renamed to `reject-sysexit-return-u8-no-conversion` to match
  the corpus's `<verdict>-<family>-<slug>` shape.
- Catalog flag 2 justifies the `unsupported` expectation kind by `QUAL-1`
  qualification failure and `QUAL-2` startup refusal. That reading is correct
  and is the one implemented; both rules are annotated, since neither is
  reachable from source until a case can pin an unqualified target.

## Done-when

The harness expresses fixtures/argv/stdin/redirection and `unsupported`; the
corpus pins v0.18 with full rule coverage; the compile-time catalog lane is
landed additively; `make check` is green.
