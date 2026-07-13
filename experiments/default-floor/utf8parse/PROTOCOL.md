# Default-floor utf8parse protocol

Status: preregistered.  No model output or timing result may be inspected before
this protocol, `task.md`, `teaching-pack.md`, and their assembled base prompt
are fixed.  The generic runner archives and hash-binds the exact prompt in
every round.  A later amendment is append-only and cannot change this primary
score.

## Question and primary comparison

The primary question is whether the first correctness-green xlang program from
one fixed low-tier-model trajectory is competitive with the ordinary released
implementation of a real Rust library.  The primary ratio is:

`throughput(xlang facts-on) / throughput(shipped Rust)`.

The same frozen xlang source compiled facts-off is an attribution control.  It
is not another generated candidate.  There is no hand-written or expert xlang
arm and no constructed expert-Rust arm.

This study is intentionally a **one-shot, single-buffer** comparison.  Each
variant starts in the UTF-8 ground state and parses the entire input in one
call.  It does not test persistence of parser state across separate caller
chunks, an EOF/finalize operation, allocation, or a text-decoding API.  Claims
from the result must remain within that scope.

## Locked shipped-Rust target

- crate: `utf8parse` 0.2.2;
- Cargo requirement: exact `=0.2.2`;
- crate features: ordinary Cargo defaults enabled; release 0.2.2 declares an
  empty default feature set, while its `nightly` test/benchmark feature remains
  opt-in and is not selected;
- registry artifact SHA-256 / crates.io checksum:
  `06abde3611657adf66d383f00b093d7faecc7fa57071cce2578660c9f1010821`;
- repository: `https://github.com/alacritty/vte`;
- published source anchor: `ebc4a4d7259678a8626f5c269ea9348dfc3e79b2`,
  path `utf8parse`;
- authoritative lock: `rust-baseline/Cargo.lock` plus the registry checksum.

The checksummed `.crate` artifact, rather than a reconstructed checkout,
defines the tested library.  Before generation, on every evaluator round, and
before and after scoring, the harness hashes every cached
`utf8parse-0.2.2.crate`, rejects non-regular or content-mismatching artifacts, and
byte-compares every packaged file and implied directory with every matching
local Cargo registry source tree.  Cargo's own `.cargo-ok` marker is the sole
allowed extra entry.  Symlinks, special files, duplicate archive paths, root
files, extra directories, and mismatching source trees are rejected.  The
verbose scoring build must reference exactly one verified source tree.  The
archive and per-file hashes are retained in generation and score metadata.

The crate source is unmodified.  The only Rust adapter is the committed safe
`parse_into(out, src)` function.  It checks `out.len() >= src.len()`, creates a
public `utf8parse::Parser`, calls its public `advance` method once for every
source byte, and observes events through a public `Receiver`.  The receiver
writes a valid `char` as its `u32` scalar value and writes `0x00110000` for
`invalid_sequence`.  It returns the written count.  The adapter does not copy,
reimplement, batch, specialize, or inspect the crate state machine.  It has no
manual inline or no-inline annotation.  Cargo's ordinary release optimizer is
free to optimize this normal generic public-API use.

## Frozen ABI and behavior

The xlang ABI is:

```text
fn parse ['r] (out: &uniq 'r buffer<u32>, src: own buffer<u8>) -> own u64 reads('r), writes('r), traps requires { ... } { ... }
```

At the machine boundary both buffers use `{ pointer, signed i64 element
count }`; the output pointer addresses `u32` elements and the source pointer
addresses bytes.

Every call begins in ground state.  Events are `u32` values: a completed valid
UTF-8 sequence emits its Unicode scalar, and an invalid sequence emits
`0x00110000`.  UTF-8 validity is exactly the following canonical grammar:

- `00..7F`;
- `C2..DF 80..BF`;
- `E0 A0..BF 80..BF`;
- `E1..EC 80..BF 80..BF`;
- `ED 80..9F 80..BF`;
- `EE..EF 80..BF 80..BF`;
- `F0 90..BF 80..BF 80..BF`;
- `F1..F3 80..BF 80..BF 80..BF`;
- `F4 80..8F 80..BF 80..BF`.

A byte invalid in ground emits one invalid event.  A byte which violates a
pending sequence emits one invalid event, resets to ground, and is consumed;
it is not processed again from ground.  A valid-looking prefix still pending
when the buffer ends emits nothing.  The public crate has no EOF/finalize call,
and this task adds none.

The entry contract is `out.len >= src.len`, even when this input will emit
fewer events.  Violation traps before the first output-element write.  On
success the return value is the number of events, the exact event stream is in
`out[0..return]`, the remaining visible suffix and guards are unchanged, and
the source is unchanged.  Source and output are distinct live buffers.

## Model trajectory and information boundary

The generator is exactly:

- `codex-cli` 0.144.0;
- model `gpt-5.6-terra`;
- reasoning effort `medium`;
- Codex service tier `default`;
- one initial response and at most three sequential repairs, for four
  candidate sources maximum;
- no parallel samples, restart, alternate seed, best-of-N selection, or
  stronger-model replacement.

Each round is a fresh `codex exec --ephemeral` invocation in an empty isolated
working directory, read-only sandbox, with user configuration and repository
rules ignored.  There is no resume identifier or hidden conversation state.
On a repair, the generic runner sends the same base prompt, the immediately
preceding candidate, and only the immediately preceding accepted machine
evaluator JSON.

The target base prompt is the literal bytes of `task.md`, followed by the UTF-8
separator `\n===== BEGIN COMPLETE XLANG WRITER'S PACK =====\n\n`, followed by
the literal bytes of `teaching-pack.md`.  Its locked identities are:

- `task.md` SHA-256:
  `9f301b9a0776b855439fb23d403e990ebc5ce8b2add9730c4040de99071732d9`;
- `teaching-pack.md` SHA-256:
  `88917635d551c9352fd788a0c339369e65ad54459ae16157b566fb0e05782672`;
- assembled prompt: 8,489 bytes, SHA-256
  `81f023e583987d4610f15faa529b6481805dc4094fda2168146cf9ea9e9c903a`.

The generic output contract requires exactly a complete candidate source file:
no Markdown fence, prose, result, or measurement.  Candidate bytes are the one
agent message exactly as emitted—no newline normalization, stripping, fence
extraction, or rewriting.

The JSONL adapter accepts exactly four events in order: `thread.started`,
`turn.started`, one `item.completed` whose item is an `agent_message`, and
`turn.completed`.  Missing, repeated, reordered, post-completion, tool,
non-message, or multiple-message events invalidate the trial and produce no
candidate.  Raw JSONL, stderr, prompts, public metadata, and hashes remain in
the audit trace; credential-bearing raw argv is not archived.

The model is forbidden from receiving or accessing:

- the Rust crate identity or source, adapter source, implementation details,
  LLVM IR, assembly, or another implementation of the task;
- this protocol, correctness cases, benchmark corpus, timings, profiles,
  ratios, proof reports, retained-check counts, or optimization advice;
- prior xlang UTF-8 parser sources, human edits to its candidate, or feedback
  from another model.

Repair feedback is limited to compiler/checker diagnostics or one correctness
failure with the complete input bytes, expected and actual event streams,
return values, termination, guards, and source-integrity status.  The longest
fuzz case is 2,048 bytes so one complete diagnostic fits the locked 65,536
character feedback ceiling.  Proof information and performance information
are never repair feedback.

The first candidate passing the complete correctness gate is frozen
immediately, byte-for-byte and by SHA-256, before proof reports, IR, assembly,
or timing are inspected.  Unused repairs cannot improve it.  Four failed
candidates are the preregistered generation-failure outcome and have no score.

The adapter timeout is 600 seconds, outer model timeout 660 seconds, evaluator
timeout 900 seconds, and every internal compiler/linker/checker timeout is
fail-closed.  `run_generation.py` is the sole no-argument launcher for
`runs/primary-terra-medium-preregistered`; the path can never be reused.  It
requires all prompt, model, evaluator, compiler, Rust adapter, and benchmark
inputs tracked and byte-clean against `HEAD`, and locks the selected Codex,
Python, Cargo, rustc, Clang, SDK, compiler, and checker identities before any
prompt is sent.

## Deterministic correctness gate

An independent specification oracle stores the literal pending UTF-8 bytes and
expected sequence length, validates the next positional byte, and decodes a
completed sequence arithmetically.  It does not copy the crate's state/action
table or accumulated-point representation.  The shipped Rust adapter must
first agree with this oracle over the entire corpus.  Any disagreement is a
harness failure and is never model feedback.  Only after that complete
preflight are facts-on and facts-off xlang checked.

The stable corpus has exactly 84,041 cases, in this order:

1. empty input (1 case);
2. every singleton byte `00` through `FF` (256 cases);
3. every ordered two-byte input, first byte `00..FF` outermost and second byte
   `00..FF` innermost (65,536 cases);
4. the following 32 valid pending prefixes in exact listed order:

   ```text
   C2
   DF
   E0
   E1
   EC
   ED
   EE
   EF
   F0
   F1
   F3
   F4
   E0 A0
   E0 BF
   E1 80
   EC BF
   ED 80
   ED 9F
   EE 80
   EF BF
   F0 90
   F0 BF
   F1 80
   F3 BF
   F4 80
   F4 8F
   F0 90 80
   F0 BF BF
   F1 80 80
   F3 BF BF
   F4 80 80
   F4 8F BF
   ```

   For each prefix, first visit the prefix alone, proving that EOF emits no
   event (32 cases).  Then, with `x` in numeric order `00..FF`, visit
   `prefix || x || 80 80 80 41` (8,192 cases).  The fixed literal observation
   suffix makes every accepted or rejected transition externally visible and
   ends with ASCII; it is not an EOF/finalize operation.
5. these 24 fixed streams, in order:

   ```text
   00 7F
   C2 80
   DF BF
   E0 A0 80
   E0 BF BF
   ED 9F BF
   EE 80 80
   EF BF BF
   F0 90 80 80
   F0 BF BF BF
   F4 80 80 80
   F4 8F BF BF
   41 C2 A2 E2 82 AC F0 9F 92 A9 5A
   C2 41 42
   E2 82 41 42
   F0 90 80 41 42
   E0 9F 41
   ED A0 41
   F0 8F 41
   F4 90 41
   C2 C2 80
   E1 80 E1 80 80
   80 BF C0 C1 F5 FF
   41 C2
   ```

6. 10,000 deterministic fuzz cases.

All generator arithmetic is modulo `2^64`.  Xorshift64* starts at
`0x5554463850415253`; each `next()` applies `state ^= state >> 12`, then
`state ^= state << 25`, then `state ^= state >> 27`, and returns
`state * 2685821657736338717`.  Each fuzz length is `next() % 2049`.  For each
byte take one `r = next()`, let `sample = (r >> 8) & 0xff`, and map by `r & 7`:

- 0: `sample & 0x7f`;
- 1: `0x80 | (sample & 0x3f)`;
- 2: `0xc2 + (sample % 30)`;
- 3: `0xe0 + (sample & 15)`;
- 4: `0xf0 + (sample % 5)`;
- 5: select `C0, C1, F5, FF` by `(r >> 16) & 3`;
- 6 or 7: `sample` unchanged.

The count identity is
`1 + 256 + 65,536 + 32 + (32 * 256) + 24 + 10,000 = 84,041`.

For every ordinary case, Rust, facts-on, and facts-off each run twice: first
with exact visible output capacity `src.len`, then with surplus capacity
`src.len + 32`.  Each run has 32-element guards on both sides and initializes
every output element to `0xA5A5A5A5`.  The verifier checks exact returned
length and event prefix, unchanged suffix and guards, and unchanged source in
both capacity modes.  Worker progress remains one case/variant record; the two
capacity checks occur inside that record, and a structured failure identifies
its capacity mode and visible length.

Capacity behavior runs in separate subprocesses.  The locked sources are
empty, ASCII `41`, valid `C2 A2`, valid `F0 9F 92 A9`, invalid `80`, broken
`C2 41`, boundary-invalid `E0 9F 41`, trailing-incomplete
`41 F0 90 80`, and mixed `41 C2 A2 FF 5A`.  For every nonempty source, every
visible capacity from zero through `src.len - 1` must trap by signal before any
output element changes, even when the actual event count would fit.  Exact
input-sized capacity must succeed, including the empty case.  A small
independent C pending-byte oracle checks return, events, suffix, guards, and
source.  Both facts builds must pass all gates from the exact frozen source;
only the compiler facts toggle differs.

## Builds and primary benchmark

All builds record repository revision and dirty state, source and tool hashes,
host/OS/CPU, power and thermal observations, versions, and full commands.  Rust
uses `cargo rustc --release --locked --offline` with its ordinary default
release profile and generic/default CPU target.  Xlang uses the same compiler
snapshot and Clang `-O3`, also at the generic/default CPU target.  No
`target-cpu=native`, `-march`/`-mcpu`, target feature search, LTO, PGO, source
patch, alternate feature set, custom wrapper, release-profile override, or
post-result flag search is allowed.  Cargo configuration search points and
environment override variables are rejected or neutralized and retained in
metadata.

The primary corpus is exactly 128 MiB (`134,217,728` bytes), divided into
4,096-byte blocks repeating classes A, B, C, D.  Each class contributes exactly
32 MiB.  A single xorshift64* stream uses seed `0x5554463842454e32` and the
same `next()` definition above; it is never reset at block boundaries.

- **A — ASCII:** every byte is `(next() >> 8) & 0x7f`.
- **B — valid ASCII-heavy:** take one `r = next()` per token.  If
  `(r & 3) != 0`, emit `(r >> 8) & 0x7f`.  Otherwise choose UTF-8 width
  `2 + ((r >> 8) % 3)` and encode the scalar selected from `r >> 16` by the
  width rule below.
- **C — valid multibyte-heavy:** take one `r = next()` per token, choose width
  `2 + ((r >> 8) % 3)`, and emit the corresponding scalar from `r >> 16`.
- **D — malformed/boundary mixed:** take one `r = next()` per token.  If
  `(r & 3) == 0`, emit a valid scalar as in C.  Otherwise select by
  `(r >> 8) % 12` from, in order: `80`, `BF`, `C0`, `C1`, `F5`, `FF`,
  `C2 41`, `E0 9F`, `ED A0`, `F0 8F`, `F4 90`, `E2 82 41`.

For width 2 the scalar is `0x80 + r % (0x800 - 0x80)`; for width 3 it is
`0x800 + r % (0x10000 - 0x800)`, with `0x800` added when that value lies in
the surrogate range; for width 4 it is
`0x10000 + r % (0x110000 - 0x10000)`.  Here `r` denotes the already selected
`random >> 16` scalar input.  Canonical UTF-8 encodes the scalar.  A token that
does not fit in the current block is not emitted; successive A-class bytes
fill the remainder.  Every token completes a scalar or returns the parser to
ground, and the harness asserts every block is ground-aligned.

The three timed variants, in identity order, are:

1. frozen xlang facts-on;
2. the byte-identical frozen source facts-off;
3. shipped Rust through the public `Parser`/`Receiver` adapter.

Each fresh process reads and hashes the immutable corpus, allocates and page
touches three separate input-sized `Vec<u32>` outputs, and then measures each
variant once in the scheduled order.  Every timed interval is one complete
one-shot parse of all 128 MiB, including the adapter's capacity check, parser
initialization, byte loop, event production, and the corresponding xlang
entry/body.  Corpus generation, file read, allocation, page touching, event
digesting, and correctness comparison are outside timing.  Throughput is input
bytes divided by elapsed monotonic time.  Returned counts and SHA-256 of the
little-endian `u32` event prefixes must match across all variants, suffixes must
remain sentinel-filled, and the corpus hash must remain unchanged.

Run 30 fresh-process blocks.  Let F, N, R mean facts-on, facts-off, and Rust.
Start with `FNR, FRN, NFR, NRF, RFN, RNF` repeated five times, then perform one
descending Fisher-Yates shuffle for `i = 29..1`, swapping `i` with
`next() % (i + 1)` from xorshift64* seed `0x50444f5244455233`.  Thus each
variant occupies every ordinal position ten times and every ordered adjacent
pair occurs five times at each adjacency position.  Retain every complete
sample; there is no warmup selection, outlier removal, fastest-of-N, or
per-block timeout in scoring.

The orchestrator records parsed power-source identity and `pmset -g therm`
before and after every block.  A changed available value, crash, missing or
malformed row, output mismatch, source/tool/artifact change, or external
interruption invalidates the whole campaign and preserves its logs.  A slow
sample alone never invalidates it.  A rerun is allowed only as a new directory
with explicit lineage to an invalid attempted score (`mode = score`,
`not_a_score = false`) which has the same protocol, frozen source, and
generation binding.

## Statistics and preregistered verdict

For every process block compute paired throughput ratios.  The primary point
estimate is the median of the 30 facts-on/Rust ratios.  Report raw samples,
per-variant median throughput, MAD/median, order-position and order-stratum
medians, the primary ratio and interval, and the facts-on/facts-off attribution
ratio and interval.

The descriptive 95% percentile interval uses 10,000 stratified bootstrap
resamples with xorshift64* seed `0x5044424f4f5432`.  Visit strata in
`FNR, FRN, NFR, NRF, RFN, RNF` order.  Within each stratum, order its five rows
by ascending campaign block index and draw five with replacement using
`next() % 5`.  Take the median of all 30 ratios.  The attribution estimate uses
the identical selected rows.  An even-sized median is the arithmetic mean of
the two central sorted values.  After sorting the 10,000 estimates, the
nearest-rank interval is zero-based elements 249 and 9,749.

For the primary facts-on/Rust interval:

- lower bound greater than `1.02`: meaningful xlang win;
- both bounds within `[0.98, 1.02]`: practical parity;
- upper bound less than `0.98`: meaningful Rust win;
- otherwise: inconclusive against the 2% band.

Apply the same four-way rule to facts-on/facts-off for attribution, but it
cannot change the primary verdict.  Correctness failure, generation failure,
or invalid measurement is reported directly and is not converted into a
performance claim.  Every label applies only to this frozen implementation,
one-shot corpus, machine, and campaign.
