# 0017 — Conformance harness v0.18 lane

**Planned task.** Split from task 0014's scope (same `docs/current-plan.md`
Work item 2 bullet) so the compiler-independent harness work runs ahead of the
compiler chain. Not yet claimed — claiming fills in `Status`, `Owner`,
workspace, and `Base revision` and moves this file unchanged in number to
`docs/ongoing/` per `docs/WORKFLOW.md`. This record authorizes nothing beyond
Work item 2 itself; if `docs/current-plan.md` is replaced before this task is
claimed, delete this file unless the new plan explicitly retains it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2
  ("first-slice conformance execution" bullet — the harness and compile-time
  lane; task 0014 retains runtime execution). Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

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
  reject/unsupported/accept verdicts without executing I/O — the catalog's
  groups A (entry/kind/name-visibility), G (effects and release attribution,
  including the `return unit;` canonical case as `unsupported` until task
  0009 lands its real judgment), the reserved-spelling and
  primitive-lookalike cases, and the kindless-unit cases. Runtime groups
  (B-F, I) stay with 0014.
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

## Done-when

The harness expresses fixtures/argv/stdin/redirection and `unsupported`; the
corpus pins v0.18 with full rule coverage; the compile-time catalog lane is
landed additively; `make check` is green.
