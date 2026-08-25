# RG-BASE Protocol

Status: FROZEN BEFORE UPSTREAM COMPARATOR SELECTION

This bundle implements the upstream-only RG-BASE step authorized by the active
Current Plan. It freezes the target, inputs, command bytes, output oracle,
statistics, comparator-selection rule, and future 2x acceptance rule before
reading official-versus-native timing. It does not authorize a Whitefoot
search implementation, compiler change, specification change, or benchmark
revision after results are visible.

The machine-readable authority for exact values is manifest.json. This file
explains why they were selected and how the phases fit together. RESULTS.md
records what has actually run.

## Question being answered

RG-BASE asks two bounded questions:

1. What is the strongest fair ripgrep 15.2.0 comparator for each frozen
   workload on the available Apple M4 target?
2. Where does that comparator spend end-to-end time, so the next Whitefoot
   plan is derived from evidence rather than a guessed optimization?

It does not ask whether one regex kernel can beat ripgrep, and it cannot
complete the flagship. The later product claim remains a Whitefoot-written
replacement reaching 2.00x on this whole suite.

## Freeze boundary and phase order

The experiment is deliberately split so a later result cannot influence an
earlier choice.

1. Preparation may download, build, and inspect result counts, but may not
   compare elapsed time between binaries.
2. The first experiment commit records this protocol, manifest, runner,
   tests, all corpus and executable identities, path manifests, output
   oracles, and the frozen-before-selection phase. This is the preregistration
   boundary.
3. Comparator selection runs three warmups and 31 official/native pairs for
   every case. The command order is deterministic, cases rotate each round,
   and official/native first position alternates per case.
4. The lower-median executable is selected per case. The raw selection
   evidence digest, freeze commit, input-manifest digest, mechanically
   recomputed winner map, and selected executable hashes are committed, and
   the manifest moves to selected-before-baseline. Baseline refuses a hand
   edited winner that does not follow the raw evidence.
5. A separate three-warmup, 31-repetition baseline is then run. Separating it
   from selection prevents reporting the same noise that picked the winner.
6. Resource sidecars and sampling traces attribute the selected baseline.
   Only then are RESULTS.md, the Direction Outline, and the next proposed
   Current Plan updated.

If the fixed 31 samples do not satisfy the preregistered precision gate, the
case is inconclusive. No samples are appended until the result looks good, and
no pattern, corpus, weight, or timeout is changed after seeing timing.
The first run ids are frozen as rg-base-selection-1 and rg-base-baseline-1.
Timeout, signal, status, digest, schedule, power, or precision failure preserves
the partial attempt and returns to owner review. The runner will not silently
retry under a convenient new id. An owner-approved retry requires a prior
protocol commit naming the failed attempt and one new fixed id.

The create-once directory prevents accidental reuse inside the documented work
root; it is not tamper-proof storage. Deleting that scratch evidence or moving
the frozen inputs to a second work root is an operator protocol violation and
invalidates the run. This trust boundary is recorded rather than presented as
a machine-enforced guarantee.

## Pinned upstream

Behavioral authority is ripgrep 15.2.0 at source commit
e89fff89ac9af12e8d4ce9d5fd07beb408ca730f. The committed Cargo.lock digest is
recorded in the manifest.

The two performance contenders are:

- the official aarch64 Apple release executable, representing what users
  receive; and
- a source build using the release-lto profile and explicit
  target-cpu=apple-m4, representing the strongest pinned native build.

A generic source LTO build is retained only as diagnostic provenance. It is
not a third contender because the flagship frame defines the stronger upstream
comparator as the faster official or native-LTO executable.

Every workload forces engine=default. The official executable links PCRE2 and
the source build does not, but the optional engine is therefore dormant in
both. A default-engine case may not silently become a PCRE2 comparison.

The native build command, from the pinned clean source checkout, is:

    CARGO_TARGET_DIR=<scratch-root>/whitefoot-rg-base/upstream/build-native-apple-m4 \
    RUSTFLAGS=-Ctarget-cpu=apple-m4 \
    cargo build --frozen --offline --profile release-lto \
      --target aarch64-apple-darwin --no-default-features

The exact rustc, Cargo, LLVM, profile, target-feature list, executable sizes,
SHA-256 digests, and Mach-O UUIDs are in manifest.json. The spelling apple-m4
is fixed rather than the mutable native alias.
The executable digest fixes what is measured; this bundle does not claim that
an independently rebuilt Mach-O is bit-for-bit reproducible without also
freezing the Apple SDK, linker, and Command Line Tools.

The process-spawning runner is also part of the measured apparatus. The gate
fixes CPython 3.14.4, the resolved interpreter and framework-library digests,
and a minimal workload environment containing only the frozen locale,
timezone, PATH, and do_not_scan TMPDIR. Supplying a different Makefile PYTHON
does not bypass this identity check.

## Target envelope

The first claim target is a Mac16,12 Apple M4 host with four performance cores,
six efficiency cores, 16 GiB RAM, macOS 26.5.2 build 25F84, and a fixed thread
cap of 10 for recursive searches. Direct single-file searches use one thread.

Runs are sequential. Both contenders use the same host, corpus, output pipe,
thread cap, and available 16 GiB memory; no artificial memory limit is imposed.
Timing requires AC power and Low Power Mode off. Power, thermal text, load
average, manifest digest, and repository commit are captured around each
block. Serial numbers, hostnames, and other unique device identifiers are
never recorded.

The gate also fixes the work root to the host's internal solid-state APFS
storage class (Apple Fabric, 4 KiB device/allocation blocks) and verifies that
the Linux corpus mount is the named case-sensitive APFS sparse image stored on
that work root. It ignores and never records device identifiers, UUIDs, SMART
counters, and volume-specific capacity. The result is still target/storage
specific rather than a claim about every SSD or filesystem.

There is no ordinary-user, verifiable page-cache purge on this macOS target.
The primary suite is therefore explicitly warm-conditioned:

- identity and correctness preflight touches both executables and all inputs;
- each arm receives three additional untimed warmups;
- every measured invocation is a fresh process; and
- official/native order is interleaved.

Fresh copies, first runs, and ad hoc eviction files are not called cold cache.
The protocol does not prove physical page residency; resource sidecars report
fault and I/O evidence where available. No cold-cache number, guaranteed-hot
number, or universal cache-state claim will be published from this target.

## Corpora

Bulk data stays under <scratch-root>/whitefoot-rg-base and is not
committed.

### Linux source tree

The Linux origin is the fork used by ripgrep's own benchmark suite. RG-BASE
pins a clean source tree from that origin:

- origin: https://github.com/BurntSushi/linux.git
- commit: 84e57d292203a45c96dbcb2e6be9dd80961d981a
- tree: ceaf69a4e25f3fce6e3682275aff8711a584c640

Linux contains 13 case-colliding filename pairs. A normal case-insensitive
macOS checkout warns that it cannot materialize both members and leaves an
invalid working tree despite the right commit/tree object ids. The usable
checkout lives on a case-sensitive APFS sparse image. Image creation is a
one-time preparation step:

    hdiutil create -size 8g -type SPARSE -fs "Case-sensitive APFS" \
      -volname WF_RG_LINUX -nospotlight \
      <scratch-root>/whitefoot-rg-base/linux-case-sensitive.sparseimage

The image is not mounted automatically after reboot. Each measurement session
creates the mountpoint if needed and attaches the existing image:

    mkdir -p <scratch-root>/whitefoot-rg-base/corpora-case-sensitive

    hdiutil attach \
      <scratch-root>/whitefoot-rg-base/linux-case-sensitive.sparseimage \
      -mountpoint <scratch-root>/whitefoot-rg-base/corpora-case-sensitive \
      -nobrowse

The runner rejects a missing mount, dirty checkout, wrong commit/tree/content
digest, ignored or untracked material, changed .git/info/exclude, a filesystem
that aliases the frozen xt_CONNMARK.h and xt_connmark.h probe paths, a
non-APFS/case-insensitive volume, a wrong mountpoint, or a mount not backed by
the frozen sparse-image path. It parses but never records volume/device UUIDs.
The original case-insensitive checkout is not referenced by the manifest.

The ripgrep benchmark suite builds its Linux checkout after cloning; RG-BASE
does not run that kernel build and does not claim to reproduce its dirty,
generated benchmark tree. This suite instead freezes the clean source-tree
search named in manifest.json.

### llama.cpp source tree

The independently sourced code tree is llama.cpp tag b10012:

- origin: https://github.com/ggml-org/llama.cpp.git
- commit: c71854292f7c367cc3b35939f88121d81945472f
- tree: be8dba6f315d2c3eefb18b680e3df3b348832442

The selection rule was the newest llama.cpp release available by the end of
the UTC day before ripgrep 15.2.0's release. It provides a modern C/C++ and AI
workload independent of ripgrep's own benchmark corpus.

### OpenSubtitles Russian text

The large single file is the complete OPUS OpenSubtitles v2016 Russian mono
text, not a sample:

- compressed origin:
  https://object.pouta.csc.fi/OPUS-OpenSubtitles/v2016/mono/ru.txt.gz
- compressed size: 482,716,779 bytes
- compressed MD5/ETag: 85af1038045e7d8a81b084a66c709d12
- compressed and decompressed SHA-256: manifest.json
- decompressed size: 1,714,880,274 bytes

The archive is retained beside ru.txt so its origin can be independently
checked. The gate verifies both compressed and decompressed size/SHA-256. The
corpus is not redistributed in this repository.

For each Git corpus, the runner verifies a clean checkout and hashes an ordered
manifest of every tracked path, mode, byte length, and content SHA-256, rejects
ignored/untracked material, and hashes .git/info/exclude because it can change
ignore cost without changing tracked bytes. The Git tree id alone is not
treated as proof that macOS materialized the intended bytes. Paths containing
colon or newline are rejected because the frozen text record format would be
ambiguous.

## Workload suite

All cases set LC_ALL=C, LANG=C, and TZ=UTC, disable user and parent config
effects, preserve repository-local ignore files, force non-color text output,
print filename, line and column, and send ordinary output through a fully
consumed pipe.

| Weight | Case | Frozen work |
| --- | --- | --- |
| 1/9 | linux_literal | Recursive PM_RESUME literal over Linux |
| 1/9 | linux_required_regex | C/H type filter and word-regex [A-Z]+_SUSPEND |
| 1/9 | linux_unicode_class | Recursive no-literal Unicode Greek category |
| 1/9 | llama_literal | C++ GGML_ASSERT with material result production |
| 1/9 | llama_case_insensitive | token family, case-insensitive, all files |
| 1/9 | llama_literal_set | C++ TODO, FIXME, or XXX alternation |
| 1/9 | subtitles_unicode_literal | Russian Sherlock Holmes literal |
| 1/9 | subtitles_unicode_case_insensitive | Same Unicode literal, folded |
| 1/9 | subtitles_no_literal | Seven five-letter Unicode words regex |

The exact argv arrays are in manifest.json; the prose table is not a second
command authority. Result-count inspection before timing established that no
case is empty: outputs range from 39 to 15,402 matched-line records. Those
counts are now frozen and may not be used to replace a case after timing.

The suite covers two independent real code trees and one large text file; one
and many files; literal, required-literal, no-required-literal,
case-insensitive, Unicode, type-filter, repository-ignore, low-result, and
material-result work. Each corpus owns one third of the future aggregate and
each case one ninth, so the 1.7 GB no-literal scan cannot dominate merely by
wall-time volume.

Known limits remain part of any claim:

- Linux and several patterns originate in ripgrep's own suite and may favor a
  mature ripgrep path. This makes the comparator strong but does not prove
  workload universality.
- The Linux tree is a clean 2022 snapshot, not a dirty modern monorepo.
- It is not the generated post-build tree used by upstream's historical
  benchmark runs.
- llama.cpp patterns are frozen real developer searches but were selected by
  judgment.
- Russian subtitles are one old language/domain and do not represent logs,
  DNA, JSON, or English prose.
- no-ignore-global and no-ignore-parent isolate personal machine state; the
  comparison covers repository-local defaults, not every user's configuration.
- the explicit 10-thread cap is part of the claim.

## Correctness oracle

Correctness is a gate before every accepted timing.

For recursive work, the runner first compares official and native selected
paths under the exact ignore and type-filter flags. The NUL-delimited path
multiset, file count, and selected byte count are frozen.

Single-file output is compared byte for byte. Recursive parallel output is
allowed to reorder completed file blocks, but not lines within one file. Its
oracle:

1. splits only at newline bytes and preserves the delimiter;
2. parses the unambiguous filename prefix and rejects a file that reappears in
   a non-contiguous block;
3. concatenates each file's original records in their observed order;
4. hashes each filename and ordered block, then sorts only the file blocks; and
5. hashes the file count, record count, and sorted block sequence.

It never decodes, trims, normalizes, or asks ripgrep to sort its timed output.
Bytes, file-block count, record count, stderr bytes, and exit status must also
agree. Each timed process drains stdout and stderr to EOF inside the measured
spawn-to-exit interval, then validates its digest before the elapsed value is
accepted.

Correctness-only controls freeze exit 1 with empty output for no match and exit
2 plus exact diagnostics for an invalid regex. Signals, timeout, missing
inputs, digest mismatch, or unexpected status are failures, not source
rejections or removable outliers.

## Measurement and statistics

The primary clock is Python's monotonic high-resolution perf_counter_ns around
process spawn, complete pipe consumption, and exit. Every run includes pattern
construction, traversal, open/read, matching, line accounting, formatting,
output write, and process startup.

Comparator selection uses 31 paired samples after three warmups. For each
case, the point statistic is:

    median(official elapsed) / median(native elapsed)

Values above one select native; values below or equal to one select official.
A deterministic 10,000-resample paired bootstrap with seed 20260804 reports a
central percentile interval using the 2.5th and 97.5th percentiles. Its
relative half-width must be at most 3%. All successful samples are retained.
If the interval crosses one, the lower-median arm remains the preregistered
operational comparator, but the result is described only as the
point-estimate-faster arm, not as a statistically resolved win.

A preflight refusal happens before a run header or timing sample and therefore
does not consume the fixed id. Once the header is created, a timeout, signal,
wrong status, output mismatch, schedule mismatch, AC/Low Power postcondition
failure, or precision failure invalidates the whole first attempt rather than
only its slow samples. The create-once run directory retains every completed
record.

The selected comparator is committed before a separate baseline phase. The
baseline uses three warmups, 31 fresh-process repetitions, median elapsed, and
a deterministic 10,000-resample 95% bootstrap interval with the same 3%
precision gate.

The gate rejects unknown phase spellings. After selection, it reconstructs the
frozen manifest and apparatus bytes from the recorded commit, verifies the raw
evidence digest and fixed run id, replays the exact schedule and correctness
fingerprints, recomputes every statistic and winner, and permits only the
declared phase fields to change. Baseline evidence is checked the same way
against the committed selected phase.

## Future Whitefoot acceptance rule

When a later plan supplies a correctness-green Whitefoot executable, each case
uses the already selected upstream executable and the same envelope. The
historical RG-BASE elapsed values are opportunity evidence, not a paired
denominator months later. The selected upstream binary is rerun in the same
future block as Whitefoot: three warmups followed by 31 paired rounds, rotating
cases and alternating upstream/Whitefoot first position by the same fixed
schedule.

For case i:

    speedup_i = median(upstream_i) / median(whitefoot_i)

The product statistic is the equal-weight geometric mean:

    G = exp(sum((1/9) * ln(speedup_i)))

The future bootstrap uses 10,000 resamples and seed 20263804. Each resample
draws 31 round indices with replacement and applies that same index vector to
all nine cases, preserving within-round pairing and cross-case environmental
correlation. It computes each case ratio and G for that resample. The central
95% percentile endpoints are the 2.5th and 97.5th percentiles.

The flagship passes only when the resulting 95% lower bound for G is at least
2.00. The per-case guard uses the same paired round resamples to compute
Whitefoot elapsed divided by selected-upstream elapsed; every case's 95% upper
bound must be at most 1.10. Correctness failure overrides every speed number.

The selected-path command is an independent oracle, so identical match output
alone is not enough to show that an implementation searched all no-match
files. A future Whitefoot acceptance plan must bind its timed search to the
same checked candidate producer as its path-oracle mode and supply an untimed
same-command stats or trace check for files and bytes searched. Upstream's
files mode and search mode share the pinned ignore walker; that source fact,
the path manifest, and the same-search stats sidecar form the present evidence
boundary. If a future implementation cannot establish this binding, the case
is not correctness-green.

The future candidate must be one normally installed, freshly invoked
Whitefoot executable. It may perform general runtime pattern specialization,
but it may not invoke or link ripgrep as the search implementation, rely on a
prebuilt corpus index or resident daemon, or specialize on frozen benchmark,
corpus, path, case, or pattern identity. Those are delegation or data leakage,
not language/compiler performance.

## Profiling and opportunity map

Profiling occurs only after comparator selection and cannot supply baseline
elapsed values. Every one of the nine frozen cases receives one selected-binary
resource/stats sidecar and one sampling trace; the opportunity map cannot pick
only favorable profiles after seeing baseline results.

- /usr/bin/time -lp records coarse real/user/system time, maximum RSS, faults,
  I/O, context switches, retired instructions, and cycles. Its 0.01-second
  clock is a sidecar, not the primary timer.
- An untimed diagnostic invocation may use ripgrep stats to record files and
  bytes searched, matches, matched lines, and bytes printed.
- `xcrun xctrace record --template 'Time Profiler' --launch -- ...` attributes
  one selected-binary run of every frozen case without sudo. The launch target
  and argv are the selected executable and exact frozen case command. There is
  no after-the-fact profiler fallback: if Time Profiler cannot capture a case,
  that profile is recorded as a blocker. Raw traces stay under do_not_scan;
  committed evidence records trace digests and compact summaries.

Attribution is limited to what the target exposes: startup/pattern compile,
traversal/ignore, file I/O, matching, line/output work, scheduling/wait,
allocation, kernel, or unknown. No cache-miss, branch, energy, P/E-core pinning,
or function-level PMU precision is invented. Timed binaries are stripped; a
symbolized diagnostic build, if needed, is never used for comparator timing
and must be tied back to the timed machine code.

## Commands

The bundle-local gate is the only correctness gate:

    make -C research/experiments/ripgrep gate \
      WORK_ROOT=<scratch-root>/whitefoot-rg-base

It runs the focused unit tests, revalidates host, executable and corpus
identity, reruns every selected-path and output oracle with both contenders,
and fails closed. It is deliberately not part of top-level make check because
it needs target-specific multi-gigabyte external data.

After the preregistration commit and a clean worktree, comparator selection is:

    make -C research/experiments/ripgrep select \
      WORK_ROOT=<scratch-root>/whitefoot-rg-base \
      RUN_ID=rg-base-selection-1

After selection evidence and manifest changes are committed, the independent
baseline is:

    make -C research/experiments/ripgrep baseline \
      WORK_ROOT=<scratch-root>/whitefoot-rg-base \
      RUN_ID=rg-base-baseline-1

Run ids are create-once scratch directories. The two ids above are the only
first attempts. A failure is recorded and returned for owner review rather
than rerun under another id.

## Stop boundary

RG-BASE stops after upstream selection, baseline, profiling, hostile review,
and the evidence-backed opportunity map. It does not write Whitefoot search
code, select a regex implementation, add threads or parallel language forms,
change the compiler or specification, or lower the 2x target. If the gate,
precision rule, or fair-cache boundary cannot be satisfied, RESULTS.md records
the exact blocker and returns to owner review.
