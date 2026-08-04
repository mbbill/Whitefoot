# First External Validation Candidate Shortlist

Status: N1 RESULT — evidence review completed 2026-08-03; no project selected

Authorization: N1 was activated on 2026-08-03 in commit `9dfe193`. The rolling
[`docs/current-plan.md`](../../docs/current-plan.md) now contains the
non-authorizing N2 proposal produced at N1 closure.

This note replaces the 2026-07-10 brainstorm in place. Git retains the old
version. The authorized question was narrow: which three to five external
projects have a recognizable public result, an authentic bounded milestone, a
strong oracle, acceptable licensing, and a plausible first path through the
current compiler? No port, benchmark, compiler change, or specification change
was authorized or performed.

## What counts as a flagship

Recognition is an entry condition, not a tie-breaker. The first external
project should help Whitefoot attract attention as well as test the language.
A candidate therefore has to pass all of these tests:

1. **Recognizable result.** Its intended audience can identify the project or
   format and understand the result without first learning a private kernel.
   Upstream deployment or adopter evidence is stronger than star counts.
2. **Authentic boundary.** The milestone implements the component people care
   about. A scalar loop carrying a SIMD project's name, a header check carrying
   a model-format project's name, or a raw block carrying a streaming codec's
   name is brand borrowing, not validation.
3. **Independent oracle.** Correctness can be compared with pinned upstream
   behavior, official vectors, or a maintained corpus. Performance is compared
   only when the milestone claims it.
4. **Reachable first result.** The current compiler can attempt the real path
   before several unrelated runtimes, targets, or integrations have to be
   invented.
5. **Useful Whitefoot pressure.** Success or failure tests an outline direction
   through the normal compiler path rather than producing another repository
   micro-benchmark.
6. **Reproducible identity.** Upstream revision, date, license, included scope,
   exclusions, and stop condition are explicit.

The attention assessments below are qualitative. They use recognizable formats,
official ecosystem placement, and named upstream adopters; they are not a
made-up popularity score.

## Shortlist at a glance

| Candidate | Public-attention signal | Smallest flagship-worthy result | Current reach | N2 disposition |
|---|---|---|---|---|
| **yyjson 0.12.0 strict reader** | JSON is universal; upstream names DuckDB and other production users | Complete default strict reader returning a bounded immutable owned DOM | Plausible, but decimal conversion and variable recursive storage may expose a real blocker | **Advance** |
| **LZ4 1.10.0 frame decoder** | `.lz4` is well known in systems work and is the project's interoperable file/stream format | Stateful decoder for real default-option LZ4 frames under hostile chunking | Plausible through a closed oracle harness; external consumption remains later | **Advance** |
| **QOI full decoder** | Format support is listed in FFmpeg, ImageMagick, GIMP, KDE, and others, but the name has a smaller audience | Valid-stream decoding is easy to compare; strict malformed-input behavior has no independent upstream oracle | Technically reachable, but fails the attention-plus-oracle gate | **Reject as first flagship** |
| **BLAKE3 1.8.5 parallel tree hash** | Upstream lists Cargo, LLVM, Nix, OpenZFS, Solana, Wasmer, and others | Explicit parallel multi-chunk tree hash with official digest and absolute multicore benefit | No parallel source form or runtime exists | **Park** |
| **CMSIS-DSP 1.17.1 Q31 biquad** | Arm's official DSP library has a clear embedded audience | Requires a concrete Cortex target and oracle boundary that N1 has not established | Scalar arithmetic is reachable; target and boundary are not | **Park** |

`Advance` means only that the candidate survives into an N2 recommendation.
It is not project selection or implementation authority. N1 permits fewer than
three finalists; convenience does not lower an entry condition.

## 1. yyjson strict reader and immutable DOM — advance

### Upstream and public meaning

- **Pin:** [`ibireme/yyjson` 0.12.0](https://github.com/ibireme/yyjson/releases/tag/0.12.0),
  commit [`8b4a38dc994a110abaec8a400615567bd996105f`](https://github.com/ibireme/yyjson/commit/8b4a38dc994a110abaec8a400615567bd996105f),
  released 2025-08-18.
- **License:** [MIT](https://github.com/ibireme/yyjson/blob/0.12.0/LICENSE).
- **Included upstream surface:** the default `yyjson_read_opts` / `yyjson_read`
  behavior, immutable document/value access, and iteration. The pinned upstream
  README still calls ABI stability unfinished, so the milestone targets
  source-level reader behavior rather than binary compatibility.
- **Recognition evidence:** JSON makes the result immediately legible, while
  the [pinned upstream README](https://github.com/ibireme/yyjson/blob/0.12.0/README.md)
  describes a strict RFC 8259 high-performance C implementation and names
  DuckDB, an optional orjson backend, and other users. That is upstream adopter
  evidence, not an independent market census.

The claim to test is: **within a stated input and nesting domain, Whitefoot can
express a complete strict JSON reader with an owned immutable DOM and
deterministic malformed-input failures, and its result matches yyjson's default
reader semantics without a writer trust escape.** The first milestone makes no
speed claim and does not claim yyjson's memory layout or ABI.

### Authentic first milestone and oracle

For the proposed first domain, accept input buffers of at most 1 MiB and nesting
depth at most 128; report a distinct resource-limit result outside that domain.
Within it, implement every standard JSON value kind under
`YYJSON_READ_NOFLAG`, including strict UTF-8, escapes and embedded NUL bytes,
ordered duplicate object keys, and yyjson-compatible `i64` / `u64` / `f64`
classification.

Use one explicit zero-change representation: copied strings in owned byte
buffers and one self-recursive boxed `DomNode` enum. Besides the JSON value
variants, that enum carries `End`, `ArrayItem(value, next)`, and
`ObjectItem(key, value, next)` structural variants, avoiding a forward or
mutually recursive nominal dependency. The parser alone constructs the
sequence invariants. Return that owned DOM and walk it to a deterministic
structural digest containing node kind and order, exact string bytes and
lengths, integer tags and values, and floating-point bits. This representation
deliberately uses no arena, returned borrow, hash map, or yyjson layout promise.

Compare the digest and accept/reject result with the pinned C reader. Use the
bundled [JSONTestSuite and JSON_checker data](https://github.com/ibireme/yyjson/blob/0.12.0/test/data/json/README.md),
the pinned [reader tests](https://github.com/ibireme/yyjson/blob/0.12.0/test/test_json_reader.c),
and [fuzzer entry point](https://github.com/ibireme/yyjson/blob/0.12.0/fuzz/fuzzer.c)
as hostile-input sources. Treat JSONTestSuite `i_` cases as unspecified. Keep
allocation/resource failure separate from language-level parse failure.

Exclude writers, mutable DOM, JSON5 extensions, incremental input, JSON
Pointer/Patch, file I/O, in-situ strings, binary ABI compatibility, and timing.

### Whitefoot pressure and stop

- **Primary outline items:** `CAND-3` and `VERIFY-1`. `BOUND-1` becomes a strict
  dependency only for a later externally consumable library; `FLOOR-3` becomes
  relevant only if a later milestone measures the AI-written performance floor.
- **Current compiler fact:** checked buffers, fixed-width integers and floats,
  enums, boxes, cleanup, and a
  [recursive owned prefix AST](../../tests/programs/prefix_expression.wf) run
  through the normal compiler. This supports a zero-change attempt; it does not
  prove that a practical JSON DOM fits.
- **Open risk:** exact decimal-to-binary conversion is project code that has not
  been exercised. The boxed-list DOM can be attempted now, but its allocation
  volume or traversal shape may expose `STORE-2` pressure after correctness; no
  storage direction is a prerequisite to the first port.
- **Stop:** stop on the first reproducible compiler or language blocker, or
  after the complete strict reader passes the oracle. Do not shrink the result
  to tokenization or validation-only. Park it if more than one independent
  compiler direction must land before a document can be returned and traversed.

**Inference:** this is the broadest medium-scale language test among the two
survivors. It also carries the highest risk that storage and number parsing
turn the first result into several projects at once.

## 2. LZ4 frame decompression — advance

### Upstream and public meaning

- **Pin:** [`lz4/lz4` v1.10.0](https://github.com/lz4/lz4/releases/tag/v1.10.0),
  commit [`ebb370ca83af193212df4dcbadcc5d87bc0de2f0`](https://github.com/lz4/lz4/commit/ebb370ca83af193212df4dcbadcc5d87bc0de2f0),
  released 2024-07-22.
- **License:** `lib/` is BSD-2-Clause; tests and most other repository material
  are GPL-2.0-or-later, as the pinned [license file](https://github.com/lz4/lz4/blob/v1.10.0/LICENSE)
  states. Any upstream test program remains an external oracle rather than
  copied Whitefoot source.
- **Included upstream surface:** the public `LZ4F_createDecompressionContext`,
  repeated `LZ4F_decompress`, and `LZ4F_freeDecompressionContext` behavior in
  the [frame API](https://github.com/lz4/lz4/blob/v1.10.0/lib/lz4frame.h),
  implementing [frame specification 1.6.4](https://github.com/lz4/lz4/blob/v1.10.0/doc/lz4_Frame_format.md).
- **Recognition evidence:** the upstream project describes multi-gigabyte-per-
  second decoding, supplies the `lz4` CLI, identifies frames as the
  interoperable format for arbitrarily long files and streams, and notes broad
  package-manager distribution. This gives the result a clear systems audience
  without pretending that use of a raw LZ4 block proves adoption of `LZ4F`.

The claim to test is: **Whitefoot implements a stateful decoder for real `.lz4`
frames, agrees with the reference implementation under hostile chunking and
corruption in a closed differential harness, and retains required checked
memory behavior.** The first milestone does not claim external consumption,
drop-in ABI compatibility, or competitive speed.

### Authentic first milestone and oracle

Decode a pinned corpus through a source-level equivalent of the public
streaming API under multiple source and destination chunk schedules. Separate
frames cover linked and independent block modes, compressed and raw blocks,
declared and undeclared content size, and block and content checksums;
independently constructed fixtures cover skippable frames. Exclude compression,
external dictionaries, legacy frames, concatenated-file policy, CLI/file I/O,
and non-default `stableDst` or `skipChecksums` promises.

A host-side test harness generates or validates the finite corpus with pinned
liblz4, embeds each case into ordinary Whitefoot buffers, invokes the same
closed decoder source, and observes a compact result record. Production
compiler behavior does not dispatch on the corpus. Run identical schedules
through both decoders and compare final output bytes and completion versus
error. Check per-call safety and progress invariants independently: reported
consumption and production stay within the supplied spans, and a nonterminal
call cannot loop forever without progress. Do not require exact per-call
consumption or next-input-hint parity; upstream documents the latter as only a
hint, and conforming decoders may buffer differently. Exact error strings are
debug text, not a stable taxonomy. Use the upstream
[`frametest`](https://github.com/lz4/lz4/blob/v1.10.0/tests/frametest.c)
behavior as an external hostile oracle and the
[frame specification](https://github.com/lz4/lz4/blob/v1.10.0/doc/lz4_Frame_format.md)
as format authority.

### Whitefoot pressure and stop

- **Primary outline items:** `CAND-2` and `VERIFY-1`. `BOUND-1` becomes strict
  only when a later milestone lets an external caller supply frames or hold the
  context. `PERF-1` and `PROOF-7` remain out of scope until a correct whole-
  frame baseline exists.
- **Current compiler fact:** checked buffers, bit operations, wrapping
  arithmetic, nominal state, hashing, and all raw-DEFLATE block classes already
  run through the normal compiler path. The
  [raw-DEFLATE witness](../../compiler/tests/programs/raw_deflate.rs) is evidence
  of the implemented surface, not evidence that LZ4 works or is fast.
- **Historical evidence only:** the retired DEFLATE experiment found severe
  short-distance overlap and burst-decoding losses. It warns against promising
  performance before attribution; it predicts no current LZ4 ratio.
- **Open risk:** the closed harness establishes algorithm and source-level state
  behavior but not usefulness to an external caller. There is no inbound
  library ABI or foreign context ownership. XXH32 and frame bookkeeping are
  project code, not presumed compiler features.
- **Stop:** stop after a complete frame differential result or the first
  reproducible blocker. Do not retreat to `LZ4_decompress_safe` raw blocks:
  upstream notes that raw blocks lack self-contained metadata, so that would be
  an internal kernel witness rather than the public result.

**Inference:** LZ4 has the best balance of recognizable systems relevance and
current-language reach. Its language pressure is narrower than yyjson's, which
can be an advantage for the first end-to-end project but may reveal fewer
general storage needs.

## 3. QOI complete in-memory decoder — reject as first flagship

### Upstream and public meaning

- **Pin:** [`phoboslab/qoi`](https://github.com/phoboslab/qoi) commit
  [`97bacc86a9c4abf5a2d452102dc26546c4c670b9`](https://github.com/phoboslab/qoi/commit/97bacc86a9c4abf5a2d452102dc26546c4c670b9),
  dated 2026-05-29. The project has no formal release, so the commit is the pin.
- **License:** [MIT](https://github.com/phoboslab/qoi/blob/97bacc86a9c4abf5a2d452102dc26546c4c670b9/LICENSE).
- **Included upstream surface:** the full in-memory decoder described by pinned
  [`qoi.h`](https://github.com/phoboslab/qoi/blob/97bacc86a9c4abf5a2d452102dc26546c4c670b9/qoi.h),
  adapted to a caller-owned output buffer rather than C `malloc` ownership.
- **Recognition evidence:** the upstream
  [adopter list](https://github.com/phoboslab/qoi/blob/97bacc86a9c4abf5a2d452102dc26546c4c670b9/README.md)
  names ImageMagick, FFmpeg, GIMP, KDE, PowerToys, Google Earth Pro, barebox,
  and many language libraries. The format name itself is less broadly known
  than JSON, LZ4, BLAKE3, or Arm, so this is the lower-attention screened
  candidate.

For valid streams, a complete Whitefoot decoder could compare every output byte
with the reference implementation. That result would cover every opcode and be
easy to visualize. It is not enough for this shortlist's stronger claim about
malformed input.

### Authentic first milestone and oracle

Pinned `qoi.h` is not a strict negative oracle: it does not fully validate the
eight-byte end marker, does not define typed errors, and can consume padding or
continue output on some truncated chunks. The upstream
[fuzz entry point](https://github.com/phoboslab/qoi/blob/97bacc86a9c4abf5a2d452102dc26546c4c670b9/qoifuzz.c)
tests crashes rather than conformance. Whitefoot could define stricter
spec-derived errors, but those expectations would be written by this project
rather than independently confirmed by upstream.

### Whitefoot pressure and stop

- **Primary outline items if reopened:** `CAND-2` and `VERIFY-1`; an external
  library claim later depends on `BOUND-1`.
- **Current compiler fact:** primitive buffers, checked indexing and arithmetic,
  structs, borrows, and `Result` appear sufficient for the decoder core. The
  existing image witness is a small internal transform, not compressed-format
  validation.
- **Stop:** do not advance it as the first flagship without an independent
  strict conformance oracle and a stronger public-attention case.

**Inference:** QOI remains a plausible internal whole-component witness, but it
fails two hard N1 conditions: the attention signal is weaker than the surviving
projects, and upstream does not supply the negative oracle needed for the
safety headline. Ease of implementation does not rescue it.

## 4. BLAKE3 parallel tree hashing — park

- **Pin and license:** [`BLAKE3-team/BLAKE3` 1.8.5](https://github.com/BLAKE3-team/BLAKE3/releases/tag/1.8.5),
  commit [`93a431c78a52d7ccf0f366f106467f5070e6075e`](https://github.com/BLAKE3-team/BLAKE3/commit/93a431c78a52d7ccf0f366f106467f5070e6075e),
  released 2026-04-25; CC0-1.0, Apache-2.0, or Apache-2.0 with LLVM exception.
- **Recognition:** the [pinned upstream project](https://github.com/BLAKE3-team/BLAKE3/tree/1.8.5)
  names Cargo, Bazel, LLVM, Nix, ClickHouse, IPFS, OpenZFS, Solana, Wasmer, and
  others. This is the shortlist's strongest broad adoption signal.
- **Milestone required by Whitefoot's flagship criterion:** hash an in-memory
  input spanning multiple chunks with an explicitly requested parallel subtree
  split, deterministic parent combination, an
  [official vector](https://github.com/BLAKE3-team/BLAKE3/blob/1.8.5/test_vectors/test_vectors.json),
  and measured absolute multicore benefit. Upstream explicitly provides its
  scalar reference as a valid starting point for ports; parallel execution is
  required here to produce a distinctive Whitefoot result, not to make a valid
  BLAKE3 implementation. A `b3sum` claim additionally needs real file/stdin.
- **Outline mapping:** primary `CAND-6` and `PAR-1`; `PAR-4` and `PERF-1` are
  required by this runtime/performance claim. `PAR-2` and `PAR-3` depend on the
  selected task/result shape; `BOUND-1` is required only for a later CLI
  boundary.
- **Why parked:** the compiler has a scalar SHA-256 witness but no parallel
  source form, worker runtime, scheduling/cost boundary, or concurrent failure
  semantics. The official optimized implementations also use SIMD and runtime
  dispatch. A scalar one-shot port would be valid BLAKE3, but it would reproduce
  an internal hashing witness without supplying the attention-worthy Whitefoot
  difference used by this shortlist.
- **Reopen/stop:** reopen after an independently justified declared-parallelism
  construct and runtime exist. Stop if the proposed result omits parallel
  execution or cannot show an absolute benefit after scheduling cost.

## 5. Arm CMSIS-DSP Q31 biquad — park

- **Pin and license:** [Arm CMSIS-DSP v1.17.1](https://github.com/ARM-software/CMSIS-DSP/releases/tag/v1.17.1),
  commit [`4b4fa8ff218ca5ac20bad71b653a37d93815f24b`](https://github.com/ARM-software/CMSIS-DSP/commit/4b4fa8ff218ca5ac20bad71b653a37d93815f24b),
  released 2026-07-16; Apache-2.0.
- **Recognition:** this is Arm's official optimized compute library for
  Cortex-M and Cortex-A. Its audience is narrower than JSON or LZ4 but directly
  relevant to the embedded direction.
- **Milestone boundary not yet established:** a credible proposal would name a
  concrete non-MVE Cortex core, board or emulator, toolchain, and result-
  extraction path, then run the scalar
  [`arm_biquad_cascade_df1_q31`](https://github.com/ARM-software/CMSIS-DSP/blob/v1.17.1/Source/FilteringFunctions/arm_biquad_cascade_df1_q31.c)
  against the official two-call state test and its SNR/absolute-error
  thresholds. N1 did not establish those target facts, so the candidate cannot
  be reopened from this note.
- **Outline mapping:** primary `CAND-5` and `TARGET-2`. `BOUND-1` becomes strict
  only if the selected result crosses the CMSIS C boundary; `PROOF-5` matters
  only if a later claim adds WCET or totality.
- **Why parked:** the arithmetic likely fits the current scalar language, but
  there is no Cortex/bare-metal target, inbound CMSIS ABI, pointer-bearing
  instance representation, SIMD intrinsic path, or resource certificate. A
  host-only adapted kernel would use Arm's name without testing the deployment
  properties its audience cares about.
- **Reopen/stop:** first research and select the exact target and oracle
  boundary. Stop if the result is only a host scalar kernel.

## Screened out

- **simdutf 9.0.0:** upstream recognition is excellent, including Node.js,
  WebKit, Chromium, and Bun, but the authentic claim is SIMD and runtime
  dispatch. The current compiler exposes neither. A scalar UTF-8 validator
  repeats the current witness and historical utf8parse study.
- **zlib-ng / libdeflate:** a raw decoder repeats the current raw-DEFLATE
  witness, while an authentic optimized or streaming claim reopens boundary,
  SIMD, and strategy gaps already identified historically. Recognition alone
  does not make “another zlib” a distinctive first result.
- **llama2.c:** the official small-model oracle is attractive, but one real
  token already requires checkpoint I/O, math runtime functions, tokenizer
  storage, and parallel execution. Those prerequisites dominate the milestone.
- **Safetensors:** adoption and safety relevance are strong, but an authentic
  result needs complete JSON metadata plus usable zero-copy typed views and a
  file or ML boundary. Header or offset validation alone would borrow the name;
  the full boundary is currently several projects.
- **vLLM:** a parser or scheduler kernel cannot honestly carry the name. The
  authentic request-to-streamed-response path requires networking, async
  execution, dynamic data, model/runtime integration, cancellation, and an
  accelerator boundary.
- **SlotMap:** it isolates useful storage questions but lacks the recognition
  required for the first flagship. Retain such a specialization as an internal
  witness if a selected project exposes the same representation need.

## N1 conclusion

Only two candidates pass every entry condition, and neither dominates the
remaining tradeoff:

- yyjson provides the broadest medium-scale language pressure;
- LZ4 best balances systems recognition and current reach.

QOI would be easier, but its weaker attention signal and missing independent
negative oracle make it an internal witness rather than a flagship finalist.

BLAKE3 and CMSIS-DSP remain valuable future project anchors precisely because
their honest milestones name the parallel and target capabilities that are
missing. They should not be weakened into branded micro-kernels.

N1 is complete. N2 may recommend yyjson, LZ4, or neither after comparing breadth
of language pressure with first-result risk; both survivors already pass the
public-attention gate. This note does not select the project and authorizes no
port or implementation.
