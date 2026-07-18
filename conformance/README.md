# Whitefoot conformance suite

A **spec-anchored, rule-keyed, toolchain-agnostic** test system. It tests the
*language* — `source → verdict` — not any prototype's internals, so the same suite
validates every implementation: today's demo compiler, the real compiler, and the
self-hosted compiler. It is the correctness oracle the M3 AI-codegen harness scores
against, and the safety net for the self-hosting bootstrap.

It is the production artifact of the toolchain. The checker and democ are disposable;
the guarantees this suite pins are not.

## Layout
- `cases/<id>.wf` — one canonical Whitefoot program per case (also a FORM-1/2 byte-exact fixture).
- `manifest.jsonl` — one JSON object per case: the rule id(s) it exercises + the expected verdict + status.
- `runner.py` — the runner (a toolchain **adapter**) + the coverage tracker.

## A case
```json
{"id": "reject-own10-dangle", "rules": ["OWN-10"],
 "expect": {"kind": "reject", "rule": "OWN-10"}, "status": "runnable",
 "doc": "Returning a borrow of an own param into a caller region dangles; rejected."}
```
`expect` is one of: `{"kind":"accept"}`, `{"kind":"reject","rule":R}`, `{"kind":"run","exit":N}`,
`{"kind":"trap"}`. For a rejection the runner asserts the **exact cited rule id** (DIAG-1).

`status`:
- **runnable** — the toolchain must produce `expect`; a mismatch is a `FAIL`.
- **pending** — the toolchain can't process the case yet (a construct it doesn't support, or
  it rejects without citing a rule id); skipped, but the case still counts for coverage and
  becomes runnable for free once the toolchain grows.
- **xfail** — `expect` is the *correct spec behavior*, but the current toolchain does **not**
  produce it (a tracked gap). Reported, non-failing. If it starts matching, it flags `XPASS`
  ("fix landed — drop the xfail"). This is how known gaps stay visible instead of forgotten.

## Run
```
python3 conformance/runner.py run        # cases; nonzero exit on any FAIL or XPASS
python3 conformance/runner.py coverage   # which of the spec's rules have a case
python3 conformance/runner.py all -v     # both, verbose
make conformance                         # the same, as make-check layer 5
```

## Adding a case
Write `cases/<id>.wf` in canonical form, add a manifest line tagging the rule(s) and the
expected verdict. Prefer one rule per negative case (so the coverage map is precise). To
close a coverage gap, target a rule from the `untested` list the coverage report prints.

## Plugging in a new compiler
The only coupling to the current toolchain is `ADAPTER` in `runner.py` — a function
`(source, want_run) -> verdict`. Point it at the real (then self-hosted) compiler and the
entire corpus runs against it unchanged. That is the reuse: the corpus is the contract; the
compiler behind the adapter is swappable.
