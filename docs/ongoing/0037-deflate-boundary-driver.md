# 0037 — deflate driver through a real system boundary

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** owner approval 2026-08-07 (provenance gate advanced in
  priority); the held candidate
  `governance/spec-evolution/provenance-gate-candidate.md`
- **Owner / workspace:** `exec-deflate-driver` /
  `/Users/bytedance/do_not_scan/wf0037`, branch
  `task/0037-deflate-boundary-driver`
- **Base revision:** `407abde51707d903fe9f3ea1bf45ab6775ac6018`
- **Dependency:** none for the driver itself; the provenance gate's
  activation depends on this task's measurement

## Goal

Make the provenance gate measurable on the sites that motivated it. Today
the deflate programs build their compressed input synthetically
(`make_dynamic_input()`, unlabelled `main`), so no value in them has a
boundary origin and the gate has zero live instances — it cannot fire on
the three canonical-Huffman sites the 0035 acceptance run found migrated
as aborting claims.

Wire a deflate driver through a real §14 boundary: a `command fn main`
that reads compressed bytes with the SYS read path and decodes them, so
the decoder's table indices carry external provenance. Then measure, with
the held candidate's rule applied by hand or by a scratch prototype:
which existing claims become illegal, whether the three canonical-Huffman
sites are among them, and what the honest repair costs (branch with an
`InvalidHuffmanCode` value path). Record the measurement beside the
acceptance evidence in `research/investigations/obligation-discharge/`.

Deliverable is evidence, not a language change. A finding that the gate
does NOT fire on those sites is a successful outcome and must be reported
as plainly as a finding that it does.

## Ordering constraint (drafter's finding, 2026-08-07)

Do NOT measure the gate before the ENT-5 loop fix is active. The deflate
path's discharge is currently dominated by the loop-rule defect, so a
measurement taken now would attribute to provenance what the loop rule
caused. Sequence: ENT-5 fix active → re-run the acceptance measurement →
then wire the boundary and measure the gate.

## Progress

Driver landed; the measurement is untouched and still waits on ENT-5. No
gate rule was applied, no claim legality was assessed, and
`governance/spec-evolution/provenance-gate-candidate.md` was not read or
changed.

Completed:

- `tests/programs/raw_deflate_boundary.wf` — a `command fn main` that
  names one compressed file on the command line, resolves it with
  `relative_path`, opens it against `command.cwd` with `open_read`, and
  accumulates the bytes with `read_once` until `ReadEnd`. The bytes are
  copied into an exact-length buffer and decoded, and the decoded bytes
  are published to `command.stdout`. Every table index the decoder
  computes therefore derives from a value with a §14 boundary origin.
- Each outcome reaches its own status, none traps and none is absorbed:
  1 no file named, 2 unreadable (bad relative path, failed open, failed
  read), 3 empty input, 4 input longer than the accepted 4096 bytes,
  5 stream ends early (`Truncated`), 6 malformed stream (the other five
  tree/code/distance/block-type failures), 7 decoded output longer than
  the 65536-byte output buffer (`OutputFull`), 8 the decoded bytes could
  not be published.
- `compiler/tests/programs/raw_deflate.rs` — two added cases build the
  driver and run it over fixture files, so `make check` exercises it.

Next (not this task): the gate measurement, once ENT-5 is active.

## Structural choice

The driver is a new file compiled with `raw_deflate.wf`,
`raw_deflate_dynamic.wf`, and `raw_deflate_dynamic_decode.wf` — not a
`main` added to `raw_deflate_dynamic_decode.wf`. That file is compiled
together with `raw_deflate_vectors.wf`, which already declares an
argument-free `fn main`, so adding a second entry there would have broken
the existing case in `compiler/tests/programs/raw_deflate.rs`. The
boundary set substitutes the driver for the vectors file and leaves that
case untouched.

The driver calls `inflate` (`tests/programs/raw_deflate.wf`), which
dispatches to `decode_dynamic` on a dynamic block. Calling
`decode_dynamic` directly would have required the driver to build its own
`InflateState` and read the three block-header bits itself, which is
`inflate`'s block loop rewritten. The corpus stream used as evidence is
one final dynamic block, so `decode_dynamic` is on the executed path.

## Scope and expected touch set

`tests/programs/raw_deflate_boundary.wf` (new),
`compiler/tests/programs/raw_deflate.rs`. No compiler, spec, conformance,
governance, or research file was touched. v0.22 source throughout, which
is what `main`'s active specification defines; it migrates with the rest
of the corpus when v0.23 activates.

## Validation

`make -C compiler check` exit 0 before (lib 523 passed / 0 failed;
programs 28 passed / 0 failed) and exit 0 after (lib 523 passed / 0
failed; programs 30 passed / 0 failed — the two added cases).
`make check` exit 0.

Run evidence, driver built by
`cargo run --bin whitefootc --locked --offline -- -o driver
../tests/programs/raw_deflate.wf ../tests/programs/raw_deflate_dynamic.wf
../tests/programs/raw_deflate_dynamic_decode.wf
../tests/programs/raw_deflate_boundary.wf`:

- `./driver dynamic_text.deflate` — exit 0, 5036 published bytes,
  `cmp` identical to the corpus oracle output. Input and expected output
  are fixture `stock-zlib-l6-default-strategy-text` of
  `research/experiments/raw-deflate-default-shape/correctness-corpus.json`
  (150 compressed bytes, one final dynamic block).
- exits 1, 2, 3, 4, 5, 6, 7 observed on the invocations listed above,
  each with its own diagnostic; exit 8 observed against a standard-output
  pipe whose read end the parent closed before the driver ran.
- A FIFO delivering the same 150 bytes in three chunks of 37, 54, and 59
  produced exit 0 and the identical 5036 bytes, so the read loop
  accumulates across short reads rather than assuming one full read.

## Corrections to the brief

- The brief named `reference.py` as the reference tooling. It needs a
  zlib-ng checkout at a path absent from this workspace. The usable
  artifact in that directory is the recorded `correctness-corpus.json`
  — 351 fixtures, each with `input_hex` and `oracle_output_hex` — which
  is where both the compressed input and the expected output came from.
  One input is not from the corpus: no recorded fixture decodes to more
  than 5036 bytes, so none can overflow the driver's output buffer, and
  the exit-7 stream was produced with the host zlib at level 6 raw. It
  is scratch only and is not in the tree.
- `wfgrep.wf` line 325 is the right template for the entry parameters,
  effect row, and read path, but its diagnostic shape does not transfer
  verbatim. Reborrowing `deref(report)` from an incoming `&uniq 'r`
  parameter inside a `match` arm needs a second nested `region`;
  a single region there is rejected as `[OWN-6] InvalidChildReborrow`.
  `wfgrep.wf` lines 280-283 already use the nested form inside an arm,
  and lines 209-212 use the single-region form at function top level.
  This is a language shape, not a defect.
- No compiler defect surfaced. The two rejections encountered
  (`[OWN-6]` above, and `[OWN-1] MoveOfCopy` for `move` on an
  `InflateError`) are both correct behavior.

## Stop condition

The driver is complete and validated. The measurement half of this card
does not start until the ENT-5 loop fix is active.
