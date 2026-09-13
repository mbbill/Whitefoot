# Ripgrep Flagship Project Frame

Status: OWNER-SELECTED UMBRELLA TARGET.

Ripgrep was selected on 2026-08-04 as Whitefoot's primary external validation
project. The target is deliberately the product result, **2x ripgrep**, rather
than one opportunity guessed before measurement. This note owns the durable
upstream pin, product claim, architecture findings, comparison rules, and
compiler questions that should not be recopied into each step.
[AGENTS.md](../../AGENTS.md) owns the workflow.

## Pinned upstream and authorities

- **Version:** [ripgrep 15.2.0](https://github.com/BurntSushi/ripgrep/releases/tag/15.2.0),
  released 2026-07-15.
- **Source commit:**
  [`e89fff89ac9af12e8d4ce9d5fd07beb408ca730f`](https://github.com/BurntSushi/ripgrep/commit/e89fff89ac9af12e8d4ce9d5fd07beb408ca730f).
- **License:** Unlicense or MIT, at the user's choice.
- **Behavioral authority:** the pinned executable and source tests for the
  exact command surface named by a benchmark or milestone. Rust regex syntax,
  gitignore behavior, output ordering, errors, and exit status are part of the
  compared work when that surface exercises them.
- **Performance authority:** end-to-end measurements against the faster of a
  pinned official release executable and a reproducible native LTO build on the
  same target. Beating a generic distributed binary alone is not a language or
  compiler result.

A newer ripgrep release does not silently move the pin. Re-pinning requires
rerunning the frozen comparator suite before claiming the same ratio.

## The claim

Build a Whitefoot-written command-line search tool that is credible as a
replacement for ripgrep's primary line-oriented recursive regex-search use and
reaches at least **2.00x** end-to-end speedup over pinned ripgrep on a
preregistered representative suite.

The suite must cover the product rather than one favorable kernel. It will span
real source trees and large text inputs; one and many files; literal, regex,
case-insensitive, Unicode, and no-required-literal matching; low and material
match counts; ignore and file-type filtering; and normal result production.
The exact cases, weights, cache state, target, build, thread and memory envelope,
and statistic are frozen before any Whitefoot performance result is observed.

The public claim must name its exact target and compatibility envelope. It may
not be restated as universal 2x speed on every CPU, input, regex, output mode,
or optional ripgrep feature. Conversely, a milestone may implement a smaller
vertical slice without shrinking the umbrella objective to that slice. No win
on a single file, sort mode, fixed string, microbenchmark, or private corpus
closes the flagship claim.

Correctness is not traded for speed. Every timed case compares the selected
files, match records, relevant ordering, rendered bytes or canonical structured
equivalent, diagnostics, and exit status required by its frozen surface.
Unsupported behavior must fail explicitly instead of silently changing
semantics. The benchmark consumes output and may not time a search whose result
is optimized away or discarded differently from upstream.

## What ripgrep 15.2.0 actually does

The pinned source has four major cooperating paths:

```text
CLI flags and paths
  -> ignore/glob directory walker
  -> regex matcher + byte searcher
  -> match/context printer
  -> stdout and exit status
```

The details matter because a product ratio can come from any layer:

- `HiArgs` parses patterns, paths, encoding, binary, output, sorting, and
  traversal policy. An explicit single file or any requested sort forces one
  thread. Otherwise the default thread count is available parallelism capped at
  12.
- The parallel top-level path obtains its concurrency from the recursive
  directory walker. The walker uses work stealing; each worker clones a
  matcher, searcher, and printer and searches one discovered file at a time.
- The ignore path combines `.gitignore`, `.ignore`, `.rgignore`, hidden-file,
  glob, file-type, symlink, depth, and filesystem rules while walking.
- The default matcher parses patterns to HIR and builds a `regex-automata` meta
  regex. It detects literal cases, performs inner-literal extraction when that
  can cheaply find candidate lines, and otherwise relies on the regex engine's
  automata and SIMD-aware search paths. Optional PCRE2 is a separate engine.
- The searcher chooses an incremental line buffer, a whole-input path required
  by multiline or transcoding cases, or a memory map where enabled and judged
  useful. On macOS 15.2.0 declines the mmap path. Binary detection, BOM and
  encoding handling, line numbers, context, inversion, and match limits remain
  in this layer.
- Parallel searches render each file into a private buffer and serialize the
  completed buffer to the shared output. Single-file searches write directly.
  Sorting disables the parallel top-level path.

The normal source is already highly optimized. It uses `regex-automata`,
`aho-corasick`, `memchr`, `crossbeam-deque`, buffered search, literal
extraction, SIMD-capable dependencies, and target-conditioned I/O choices.
The current release also includes a directory-traversal performance improvement.
The research may not treat an ordinary parallel loop or vector byte scan as an
unclaimed 2x opportunity.

The source contains writer-visible `unsafe` at the memory-map and hostname
system-call boundaries. File-backed mmap explicitly accepts a possible process
fault if another process truncates the file. This is relevant to Whitefoot's
boundary and proof-gated authority work, but safety alone is not the public
adoption claim and cannot substitute for the 2x result.

## Open performance opportunity map

These are hypotheses to investigate, not a selected implementation sequence or
a narrowed claim:

1. **Search strategy and regex execution.** Pattern analysis, automaton choice,
   literal candidate search, byte-class representation, Unicode handling, and
   match verification may have target- and workload-dependent alternatives.
2. **Cross-layer specialization.** CLI facts about the regex, line mode,
   output, encoding, binary handling, and corpus may justify a verified
   specialized pipeline rather than paying for dormant generality in each
   layer.
3. **Parallel work decomposition.** File-level work stealing, large-file
   partitioning, adaptive chunking, deterministic merge, match-density skew,
   cancellation, and bounded fan-out are one joint problem. Profitability must
   be measured; the compiler does not discover unrequested parallelism.
4. **I/O and traversal.** Directory enumeration, ignore matching, metadata,
   open/read strategy, buffering, page-cache behavior, and overlap between I/O
   and matching may dominate source-tree cases.
5. **Result construction.** Line discovery, numbering, context, JSON or text
   formatting, per-worker buffers, ordered or unordered publication, stdout,
   and high match counts may dominate after scanning becomes fast.
6. **Checked machine shape.** Bounds, alias, effects, loop structure, layout,
   vectorization, register pressure, instruction selection, and target cost
   modeling may leave differences between Whitefoot source and final code.
7. **Algorithm and representation.** Whitefoot need not translate ripgrep line
   by line. A different safe regex, walker, queue, buffer, or output algorithm
   is valid when it preserves the frozen contract and resource envelope.

The list stays open. Evidence may add, merge, reject, or reprioritize an
opportunity. An early loss does not lower the 2x target or turn one convenient
case into the flagship.

## Compiler and language questions

The product target supplies concrete questions for the current language and
compiler:

- A real executable receives `argv`, patterns, paths, stdin and filesystem
  data and produces stdout, stderr and exit status. Its checks cover malformed
  patterns, arbitrary file bytes, concurrent file changes, resource failure,
  cleanup, and race freedom.
- Parallel search must account for tasks, determinism, failure, cancellation,
  work distribution, bounded fan-out, and profitability. A
  [wider proof domain](../../docs/ideas.md#wider-parallel-proof-domains) matters
  only when measured work needs an unsupported partition or regrouping; not
  every merge is an algebraic reduction.
- [Optimizer facts](../../docs/ideas.md#proof-derived-optimizer-facts) or
  verified strategy selection need a profile showing the exact lost fact,
  alongside the zero-change baseline, comparable work, and final machine
  shape. A source proof alone does not establish a performance advantage.
- Compare the first accepted AI-written search path with the best measured
  shape and investigate accepted slow forms.
- [Storage transitions](../../docs/ideas.md#storage-transitions-and-representation-privileges)
  become concrete when a matcher, traversal, queue, result, or dynamic buffer
  exposes an ownership or growth blocker under the current language.

The first valid project attempt is allowed to stop on the first current-language
or compiler blocker. That is a delivery boundary, not a reduction of the
flagship. Later investigations resolve one measured blocker or opportunity at
a time and return to the same frozen product comparison.

## Comparison and attribution rules

The first approved performance frame freezes:

- upstream source and executable identities, dependency lock, toolchains,
  target features, link mode, and build flags;
- Whitefoot compiler and source identities, runtime and backend dependencies;
- corpus origin, revision, exact included bytes or digest, and eligibility;
- pattern, path, flag, environment, terminal/output destination, thread and
  memory settings;
- cold-, warm-, or hot-cache preparation, with unsupported cache controls
  reported rather than simulated asymmetrically;
- correctness normalization for inherently unordered output;
- repetitions, interleaving, statistic, uncertainty, timeout, and outlier rule;
  and
- the aggregate 2x rule and per-case regression guard.

The stronger upstream comparator is selected per frozen case before Whitefoot
timing. The official executable represents what users receive; a native LTO
build prevents a generic distribution target from being mistaken for a
Whitefoot advantage. Default-engine cases do not gain or lose work merely from
whether optional PCRE2 is linked.

The upstream ratio is a product result: it can combine algorithm, regex engine,
data structure, runtime, compiler, and code-generation differences. A claim
about one Whitefoot mechanism requires a separate same-source ablation with the
expected IR and final-binary consequence preregistered. Required safety checks
remain unless proof discharges them. Parity, regression, and inconclusive
experiments remain evidence and redirect the next search; they do not justify
changing the benchmark after seeing the result.

## Project discipline

The target is persistent until the owner reselects it. It is not abandoned
because the first implementation is slow or the first suspected optimization
fails. It is also not declared successful by accumulating unrelated favorable
microbenchmarks. Each selected investigation advances one independently
reviewable product path, blocker, or attributed optimization and states how it returns to
the 2x comparison.

Reevaluate the flagship only if the owner changes the product objective, the
comparison cannot be made correct and reproducible, or evidence shows that the
claimed product is not meaningfully substitutable for ripgrep. Implementation
difficulty and sunk cost alone neither lower the goal nor prove progress.
