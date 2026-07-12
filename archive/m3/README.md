# M3 AI-Codegen Harness

This directory is the executable scaffold for `DECISION_SPRINT.md`.

The harness does not call model APIs. It runs submitted source files through the
same task manifest and records comparable results. Model-tier execution is a
separate producer step: put generated files under `submissions/<suite>/<language>/`
and run this harness.

## Layout

- `tasks.jsonl` — task manifest, one JSON object per task.
- `prompts/` — model-facing task prompts.
- `submissions/reference/` — deterministic reference submissions.
- `harness/run.py` — local compiler/test runner.
- `RESULTS.md` — current local result and decision-readiness blockers.
- `IMPLEMENTATION_GATES.md` — audit of the xlang-pending task blockers and the
  next continue/stop gate.

## Run

```
python3 m3/harness/run.py --suite reference
```

To save JSONL:

```
python3 m3/harness/run.py --suite reference --out /private/tmp/xlang-m3-reference.jsonl
```

The output records compile status, run status, timing samples, source hash, and a
pass/fail verdict per task/language pair. Tasks may be pending for one language
while runnable for the other; pending entries are emitted as decision blockers
without failing the smoke run.

To summarize a saved result file:

```
python3 m3/harness/score.py /private/tmp/xlang-m3-reference.jsonl
```

To ask whether the evidence is decision-ready for named model suites:

```
python3 m3/harness/score.py weak.jsonl middle.jsonl strong.jsonl \
  --required-suite weak --required-suite middle --required-suite strong \
  --require-decision-ready
```

## Adding Model Outputs

For a model suite named `weak-001`, either write one generated file per task:

```
m3/submissions/weak-001/rust/checked_loop_sum.rs
m3/submissions/weak-001/xlang/checked_loop_sum.xl
```

or write multiple fixed-budget trials per task:

```
m3/submissions/weak-001/rust/checked_loop_sum/001.rs
m3/submissions/weak-001/rust/checked_loop_sum/002.rs
m3/submissions/weak-001/xlang/checked_loop_sum/001.xl
m3/submissions/weak-001/xlang/checked_loop_sum/002.xl
```

Optional sidecars named `001.rs.meta.json`, `001.xl.meta.json`, or
`001.meta.json` are copied into the result record. Use them for model name,
prompt hash, repair turns, prompt tokens, or other producer metadata.

Then run:

```
python3 m3/harness/run.py --suite weak-001 --out /private/tmp/weak-001.jsonl
```

The same task IDs and filenames are used for every suite so results are directly
comparable.

To require multiple model trials during readiness scoring:

```
python3 m3/harness/score.py /private/tmp/weak-001.jsonl \
  --required-suite weak-001 --min-trials-per-task 3 --require-decision-ready
```
