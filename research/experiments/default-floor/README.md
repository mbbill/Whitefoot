# Default-floor runner

`generate.py` runs and freezes one benchmark-blind model trajectory. It has no
target-specific source, compiler command, evaluator, or benchmark knowledge.
See `PROTOCOL.md` for the normative boundaries and artifact layout.

The two-target D9a series is complete. See `RESULTS.md` for the cross-target
claim and its limits, then `percent-decode/RESULTS.md` and
`utf8parse/RESULTS.md` for the authoritative target-specific measurements and
artifacts.

The model-runner, evaluator harnesses and pinned reference-crate self-tests
belong to this completed experiment. They remain available through
`make historical-tool-tests` at the repository root for reproduction, outside
the current compiler gate. Current program tests and independent oracles keep
their own checks; this retirement changes no Whitefoot case or verdict.

The reproduction target uses Cargo's locked offline mode. Before its first run
on a fresh host, populate each listed crate's registry with
`cargo fetch --locked --manifest-path <crate>/Cargo.toml`; the root Makefile
names those manifests.

Example:

```sh
python3 experiments/default-floor/generate.py \
  --run-dir /tmp/my-default-floor-run \
  --prompt-file /absolute/path/to/prompt.txt \
  --model-argv-json '["/absolute/path/to/model-command", "--fixed-model"]' \
  --evaluator-argv-json '["/absolute/path/to/evaluator-command", "--machine-json"]' \
  --public-model-metadata-json '{"provider":"example","model":"fixed-model-id"}' \
  --repair-budget 1 \
  --source-name source.xl
```

The model reads the prompt from stdin and writes source to stdout. The evaluator
receives the absolute source path as its final argv item and writes the allowed
JSON channels to stdout. Commands are never parsed by a shell, and relative
script paths should not be used because the model runs from an empty temporary
working directory.

The argv arrays themselves are not archived because they may contain secrets.
The run records their canonical SHA-256 values and explicitly public metadata.
Do not place credentials in metadata.

For Codex CLI, use the included adapter rather than raw `codex exec` stdout.
The adapter consumes `--json` events, accepts exactly one completed agent
message, and rejects every tool/non-message event; only that message becomes
candidate stdout. CLI JSONL is retained in model stderr for audit:

```sh
--model-argv-json '["python3","/absolute/path/to/codex_model_adapter.py","--codex","/absolute/path/to/codex","--model","FIXED_MODEL_ID","--reasoning","medium","--service-tier","default"]'
```

Run the self-contained mock tests with:

```sh
python3 -m unittest discover -s experiments/default-floor/tests -v
```
