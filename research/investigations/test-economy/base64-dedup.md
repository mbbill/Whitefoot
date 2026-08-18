# Duplicated conformance-case execution: what it costs and what removing it would cost

Status: ANALYSIS ONLY, 2026-08-17, batch 0070 (W5). No change is proposed for
autonomous landing. Every candidate action below is a protected-compliance
decision requiring the owner's exact before/after approval, because each one
changes what the canonical `make check` gate exercises.

Purpose: the `x-base64-rfc-vectors-run` case is compiled and executed twice per
full verification, and it is the single most expensive test in the compiler
suite. This note establishes the exact scope of that duplication, attributes its
cost, and states what the owner would have to approve for each way of removing
it — including the finding that the obvious removal is not currently available.

Delete this file when the owner has ruled on the decision it frames, or when
`conformance-run` joins `make check` and makes the question moot.

## 1. The exact duplication

Two independent surfaces compile the same corpus sources:

- `cargo test --profile gate --lib` (inside `make check`, via
  `make -C compiler check`). 101 conformance cases are embedded into library
  tests with `include_bytes!("../../../tests/conformance/cases/<id>.wf")`.
- `make conformance-run` (`cargo test --test conformance -- --ignored`), the
  compiler-independent corpus walk over all 446 manifest cases.

Cross-referencing the embedded paths against `tests/conformance/manifest.jsonl`:

| | count |
|---|---|
| corpus cases embedded in library tests | 101 |
| of those, declared `expect.kind == "run"` | 56 |
| of those, declared `reject` | 42 |
| of those, declared `accept` | 3 |
| embedded paths that are not manifest cases | 0 |

The 56 `run` cases are the duplication proper: both surfaces compile them from
source, link them, execute them, and check the exit status. Their combined
source is 61,491 bytes, of which `x-base64-rfc-vectors-run` is 14,111 (23.0%).
The largest six:

| case | source bytes |
|---|---|
| `x-base64-rfc-vectors-run` | 14,111 |
| `x-wc-chunk-summary-run` | 4,992 |
| `x-borrowed-pool-tree-run` | 3,632 |
| `x-result-buffer-transform-run` | 2,784 |
| `x-enum-twostate-result-payload` | 2,573 |
| `x-buffer-borrowed-columns-run` | 2,536 |

The 42 `reject` cases are compiled twice but never executed; the library side
generally asserts the cited rule and the corpus side asserts the verdict.

## 2. Where the cost actually is

Measured on the base64 case with the gate-profile `whitefootc` and the exact
link command the test harness uses (`/usr/bin/clang -x ir … -O2`):

| stage | command | wall |
|---|---|---|
| Whitefoot front end to LLVM text | `whitefootc --emit-llvm -o b64.ll x-base64-rfc-vectors-run.wf` | 34.7 s (24.7 s user) |
| host link at `-O2` | `/usr/bin/clang -x ir b64.ll -O2 -o b64` | 0.37 s (0.06 s user) |
| whole library test | `cargo test --lib compiler_independent_base64` | 32.3–52.1 s across runs |

The front end is ~94x the link. **Deduplicating the execution therefore saves
essentially nothing; the duplicated cost is the semantic analysis, and both
surfaces need it for their own assertion.** Any proposal framed as "stop running
it twice" is measuring the wrong term.

The base64 case is expensive because it is 14 KB of source carrying a const
alphabet, nine claims, and an opaque output-capacity DAG — entailment closure
and provenance dominate. It is not expensive because it executes.

Measurement caveat, stated rather than hidden: these numbers were taken on a
machine running many concurrent agent workspaces. The same single test measured
55–92 s across six runs at two revisions, a 1.67x spread. The stage *ratio*
above is robust to a uniform slowdown; the absolute seconds are not, and no
conclusion here rests on them.

## 3. What each removal would cost, and what the owner must approve

### (a) Delete the 56 duplicated executions from the library suite

Before: `make check` compiles and executes 56 corpus programs through
`cargo test --lib`, and additionally asserts emitted-IR shape for many of them
(for base64: the `encode` signature, exactly one `free` in `encode`, three
`wf_encode` calls and three `free`s in `main`).

After: `make check` executes none of them. `make conformance-run` still does —
but `conformance-run` is **not part of `make check`** (Makefile: `check:
repository-invariants spec-append-only spec-archive-integrity spec-digest-sync
conformance compiler`), and its corpus walk is `#[ignore]`d with a live blocker
(`own3-pos-outlives-store` does not reach its declared verdict).

Net effect: the green gate would stop executing any corpus program at all. That
is a coverage reduction of the canonical gate, not a deduplication.

Owner must approve: removal of 56 executions from the canonical gate, with the
exact list; and the accompanying statement that corpus execution evidence moves
to a target that is not green and not in `check`. **Not recommended.**

### (b) Keep the library tests but drop only their `compile_and_run`

Before/after as above, except the IR-shape assertions and the front-end compile
stay; only the link and execution go.

Net effect: saves ~0.37 s per case (§2). It also silently converts an
end-to-end test into a shape test, which is the weaker of the two claims.

Owner must approve: still a gate-behaviour change (56 tests stop establishing
execution). The saving does not justify the packet. **Not recommended.**

### (c) Remove cases from the manifest so the corpus walk stops covering them

Deletes protected conformance evidence to make a gate cheaper. This is exactly
the class `CLAUDE.md` names as a governance breach when done to go green, and
here it would be done to go fast. **Rejected; recorded only so the option is
visibly closed.**

### (d) Share one compilation between the two surfaces

The library suite and the corpus walk are separate test binaries; sharing one
compiled artifact would mean a build-time cache or an artifact handoff between
targets — new machinery serving no compiler capability, which project law
forbids absent a current need.

Additionally, sharing would destroy the property that makes the corpus walk
worth running: it reaches each verdict through the *ordinary* compiler path
independently of any library test's setup. **Rejected.**

### (e) The one action that makes the question answerable

Fix `own3-pos-outlives-store` (W1's runnable failure, already in the ACTIVE
plan), make `conformance-run` green, and add it to `make check`. Then and only
then does deleting the 56 duplicated library executions reduce work without
reducing coverage — because the corpus walk would be carrying them inside the
gate.

Owner must approve, at that point: `conformance-run` joining `make check`
(canonical gate wiring), and the deletion of the 56 duplicated executions with
their exact list. Both belong in one packet, after the blocker is closed, not
before.

## 4. Conclusion

The duplication is real (56 cases, 61,491 bytes, compiled and executed twice)
but it is not the cost centre, and it is not removable today without shrinking
what the green gate exercises. The recommendation is to **change nothing now**
and to revisit only as a consequence of `conformance-run` becoming a gate
member — at which point the removal is a bookkeeping follow-up to that decision
rather than a decision of its own.

The separable win that needs no approval at all is in the other direction: the
base64 case's 34.7 s is front-end analysis time on one 14 KB program, and
reducing it is an ordinary compiler-performance question about entailment
closure, not a test-economy question. If the suite's wall clock is the actual
complaint, that is where to look.

## Provenance

- Case/manifest cross-reference: `include_bytes!` paths under `compiler/src`
  intersected with `tests/conformance/manifest.jsonl` (446 case objects; 433
  runnable, 13 pending; 139 `run`, 227 `reject`, 65 `accept`, 15 `trap`).
- Duplicated executions: `compiler/src/backend/tests/base64.rs`,
  `compiler/src/backend/tests.rs`, and the sibling modules under
  `compiler/src/backend/tests/`; corpus side
  `compiler/tests/conformance/adapter.rs::the_corpus_reaches_its_declared_verdict_through_the_ordinary_compiler_path`.
- Gate membership: `Makefile` targets `check` and `conformance-run`.
- Stage timings: `compiler/target/gate/whitefootc` and `/usr/bin/clang`, the
  same binary and link command `compiler/src/backend/tests.rs::build_linked_executable`
  invokes.
