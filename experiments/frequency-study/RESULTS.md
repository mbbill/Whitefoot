# One-time frequency-pilot results

Status: **provisional GO; directional evidence only; study stopped**

The pilot did what it was meant to do: cheaply estimate whether Whitefoot still has
plausible leverage in real Rust code. It did not measure a whole-program
speedup, establish population prevalence, or validate every raw source signal.

## Source panel

The download-ranked prefix produced 30 eligible projects after 36 archives were
examined. Across the complete examined prefix, the scanner read 1,959 production
Rust files and 1,284,764 nonblank production lines.

Approximate source signals in the 30 eligible projects were:

| Signal | Records | Projects |
| --- | ---: | ---: |
| multi-slice/scoped-alias candidate | 25 | 10 |
| indexed-loop bounds candidate | 6 | 4 |
| strict unsigned saturating reassociation | 0 | 0 |

The expert-shape controls found 8 `chunks_exact`, 23 `get_unchecked`, and 59
`zip` records. They are context for authorship style, not missed Whitefoot wins.

The manual current-whitefoot bounds/alias audit classified all 31 high-signal
records as follows:

- 0 plausible current-whitefoot advantages;
- 20 likely to provide no advantage;
- 9 false positives or non-production sites; and
- 2 unresolved.

Therefore the raw source counts are an upper bound on interesting shapes, not
an opportunity-frequency estimate. In this conservative popular-library panel,
the current alias and checked-law mechanisms did not emerge as common wins.

## Application panel

The command-line-utilities ranking produced 12 eligible binary crates after 24
archives were examined. The complete examined prefix contained 662 production
Rust files and 463,712 nonblank production lines.

The approximate scanner found:

| Signal | Records | Projects |
| --- | ---: | ---: |
| indexed-loop bounds candidate | 18 | 5 |
| multi-slice/scoped-alias candidate | 28 | 4 |
| lexical serial saturating recurrence | 18 | 1 |
| strict reassociation-miner candidate | 0 | 0 |

Manual inspection accepted no current scoped-alias win and no checked-law win.
It did identify approximately six bounds/precondition sites across three
projects as worth examining in optimized IR. This is the pilot's useful
positive signal: bounds-proof quality, not law or alias frequency, is the best
next bet.

## Optimized-IR follow-up

Three selected, locally buildable crates were sampled: `comrak`, `inferno`, and
`crc`. This is a feasibility follow-up, not a probability sample.

Bounds results:

- `comrak`: 101 direct bounds-panic calls, grouped into 40 first-party candidate
  functions in the library capture;
- `inferno`: 15 direct calls, with two first-party candidate functions; and
- `crc`: zero.

Those are surviving-check candidates, not 116 proven removable checks. They
still need workload hotness and proof-shape inspection.

Alias-versioning results:

- `inferno` contained three first-party versioned instances over two source
  sites: two monomorphizations at `src/collapse/common.rs:459` and one instance
  at `src/collapse/vsprof.rs:172`;
- `comrak` had zero first-party instances; five detected loops belonged to
  dependencies in the binary capture; and
- `crc` had zero.

The `inferno` cases are real local-allocation disjointness facts, but they are
outside current Whitefoot's emitter/expressibility and are likely small. They do not
count as current-whitefoot advantages.

The effect-attribute classifier could not resolve cross-module targets from
these single-module captures. No effect-frequency claim is made.

## Hotness and one intervention

A 20 MB repeated-Markdown workload ran `comrak --gfm --syntax-highlighting
none` 50 times under a 1 kHz sampler, producing 8,929 samples. Several
functions with surviving bounds checks were genuinely hot:

| Function | Self CPU | Inclusive CPU |
| --- | ---: | ---: |
| `Parser::parse` | 18.19% | 68.69% |
| `html::escape` | 5.30% | 7.12% |
| `Parser::process_line` | 3.33% | 16.58% |
| `Parser::open_new_blocks` | 2.07% | 4.73% |

The focused audit examined 39 bounds edges in core parser/formatter code.
About four are current-PROOF-1-shaped after canonical transcription, roughly
30--33 need generalized relational or callee-postcondition proofs, and two to
five remain data-dependent or unclear. Twenty-three come from
`advance_offset` in parser paths; the byte-mode cases need a relation like
`offset + count <= line.len()` rather than a per-access check.

One controlled intervention replaced `bytes[offset + i]` in hot
`html::escape` with its equivalent `get_unchecked`: `i` was returned by a
search over `bytes[offset..]`, so the access is valid. Baseline and intervention
produced byte-identical 22,100,000-byte HTML output. Same-path sequential
release builds used one codegen unit; 35 alternating measurements gave medians
of 172.971 ms checked and 173.909 ms unchecked. The unchecked variant was 0.54%
slower, i.e. **no measurable whole-workload win from removing that one check**.

This is useful negative evidence: surviving checks occur in hot real code, but
one eliminated check need not matter. The remaining opportunity is clustered
relational proof across many checks, which belongs in the next real-workload
compiler experiment rather than another frequency scan.

## Decision

**Provisional GO: stop analyzing and move on.** The evidence supports the next
bounded work being improved bounds/precondition proof plus real workload ports.
It does not support prioritizing checked-law or current scoped-alias frequency,
and it does not establish that Whitefoot is faster than Rust on whole projects.

The reason to proceed is narrower: ordinary popular code contains surviving
bounds checks and source shapes worth testing, the current proof-elision channel
already has a controlled mechanism win, and a few real ports will answer the
performance question more directly than a larger analyzer. No further
frequency-tool investment is planned.

Principal caveats:

- source heuristics have false positives and unknown false negatives;
- the application panel is popular CLI crates, not all Rust applications;
- only three library-oriented IR captures were completed;
- only one synthetic-but-representative large Markdown workload was profiled;
  and
- the single-check intervention does not estimate the effect of removing a
  whole family of related checks.
