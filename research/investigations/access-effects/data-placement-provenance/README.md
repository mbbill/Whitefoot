# Records data-placement provenance (disposable)

This manual-only research branch observes the result pointers returned by the
exact baseline and candidate `records` images archived by compute run
35527433610. It does not rebuild or relink them, and it does not perform a
performance experiment. Runner timing rows are an unavoidable by-product of
reaching the six performance-shaped allocations and are discarded.

The debugger stops at `wf_bench_records_release`, which is called by
`wf_oracle_check` after both clocks have stopped and tail-jumps directly to
`free` in both optimized images. On x86-64 the incoming `rdi` is the baseline
payload or the candidate retained cell; the candidate payload begins eight
bytes after that cell.
The first word is also read without mutation; every candidate observation must
hold the 131,072-element length there.

The fixed observation is one fresh process for each combination of exact image
and W1/W2/W4, six processes total. Each must make six release calls. ASLR is
left enabled with `set disable-randomization off`. No inferior memory or
register is written and no inferior function is called.

All six observations are archived. Ordinal 0 is the runner's discarded warmup
and is reported separately because the first large allocation/free can change
glibc's later mmap policy. The result supports a later controlled experiment
only when the five retained-sample payloads across all three widths have one
baseline residue, have one candidate residue, and the candidate residue is
exactly baseline plus eight modulo 64. Otherwise it stops as inconclusive. A
zero-aligned residue is never substituted or selected from a mixed sample.

This branch and its workflow are deleted after the one-shot investigation.

Run 35532641001 stopped before downloading or executing either image because
the Ubuntu image did not contain gdb. The corrected one-shot workflow records
the exact images' mapped libc package, path, SHA-256 and ELF Build ID, installs
only GNU gdb from the image's official Ubuntu source with
`--no-install-recommends`, then records and compares the same identity. Any
libc/allocator change stops before observation. This is a documented departure
from the first run's no-install infrastructure gate; it does not relax any
address or interpretation criterion.

Run 35533030765 established that installation left allocator identity
byte-equal, then stopped after baseline W1 because its breakpoint named the
source-level `wf_record_result_release`; optimization retained that symbol but
made the called host release wrapper jump straight to `free`. It produced no
address observations and is an instrumentation failure, not data or timing
evidence. Disassembly fixes the wrapper above as the actual post-clock seam.

## Executed runs and result

| Run | Branch revision | Result |
| --- | --- | --- |
| [35532641001](https://github.com/mbbill/Whitefoot/actions/runs/35532641001) | `66c0ce3a14da1ffa352c2ad391b74ced4055f3f3` | Infrastructure failure: gdb was absent; no artifact image was downloaded or executed. |
| [35533030765](https://github.com/mbbill/Whitefoot/actions/runs/35533030765) | `a78ce5364a204f8a9c452d84bcc570479c09a9c9` | Instrumentation failure: allocator identity was preserved, but the source-level breakpoint was bypassed by optimization. Baseline W1 made six checked calls and exited normally, with zero address observations. No other arm ran. |
| [35533234207](https://github.com/mbbill/Whitefoot/actions/runs/35533234207) | `6d3baa4862a470b27ded2c8404c82cfc7ec2ba91` | Observation completed and the preregistered gate returned `INCONCLUSIVE` because the kept-sample residue was not unique across fresh processes. |

Run 35533234207 used an AMD EPYC 7763 host with two cores, four hardware
threads and a 32 MiB L3. GNU gdb 15.1 observed the post-clock release wrapper
with ASLR enabled. Each of the six processes exited normally, emitted six
ordinary runner rows whose timing values are discarded, completed six oracle
comparisons, and reached six release observations. The archive therefore
contains 36 observations and 36 successful oracle checks. All 18 candidate
observations held the required length header `131072`.

| Workers | Baseline warmup | Candidate warmup | Baseline kept samples 1--5 | Candidate kept samples 1--5 |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 16 | 24 | 0 | 8 |
| 2 | 16 | 24 | 48 | 56 |
| 4 | 16 | 24 | 16 | 24 |

Residues are payload addresses modulo 64. The five kept observations were
stable within each process, and each same-width candidate residue was eight
bytes after its baseline counterpart. They were not stable across the three
fresh processes for either image: the baseline set was `{0, 16, 48}` and the
candidate set was `{8, 24, 56}`. Width and process identity are confounded by
the fixed design, so this table does not establish that worker count selected
an allocator residue.

The exact images were unchanged before and after observation:

- baseline SHA-256
  `904028379508662fc988cb256896f90dbb4f24a82746c771b0e1cc6024412ce5`;
- candidate SHA-256
  `7d0f38f2f61e109cd2260c8ba335beaff4eae44a47a0d7ec8e1b06bc954fbf68`.

Installing gdb with `--no-install-recommends` did not change the mapped
allocator or loader. Before and after installation, `libc6` and `libc-bin`
were `2.39-0ubuntu8.8`; mapped libc had SHA-256
`8db37cf3f2169f59a0f07ef1fea308c35656668c64c8ff294e1860f4121eb161`
and Build ID `328820b908de8ea1ef79afa8995e302e819163d7`; the loader had
SHA-256
`cd4df4f3c7b83673d61189bf2eaebd33ca4f2853ab9772b8a25e025ef99b1e81`
and Build ID `f58808c9c8a388055b126492a1706d732761f86e`.

The final result supplies pointer provenance only. It contains no timing
evidence, does not recover the addresses used by original compute run
35527433610, and does not establish or disprove a causal effect from the
within-line eight-byte shifts. Because the preregistered unique-residue gate
failed, it authorizes no controlled pair, Stage 2 run, or production alignment
policy.

## New preregistered controlled-residue follow-up

The earlier Stage 1 unique-residue gate failed, so it authorized no Stage 2.
This is a new preregistered follow-up with a different controlled construction,
not a prior-criterion success or an outcome selected from those observations.
The old observations did motivate the fixed `16/24` and `48/56` pairs, but
every width/process observation, including those W2/W4 pairs, was confounded by
its fresh process. Their timing direction remains unknown until this new test.
The next bounded experiment tests the remaining data-address premise before any
new worker-placement or production-layout change. It uses the current exact
candidate records LLVM module, oracle objects, and ordinary runtime from one
compute artifact. A research-only LLVM rewrite replaces exactly two calls: the
`malloc` in the unique `wf_box_array_filled$instance$34` called by
`wf_summarize_records`, and its matching `free` in
`wf_record_result_release`. C-oracle buffers and all other allocations retain
the artifact allocator. The replacement overallocates once, returns the same
cell layout at a selected address, and remembers the real allocation immediately
before that cell for release. Static construction auditing requires the two
shared-row callers to be exactly `wf_summarize_records` and
`wf__par_seq_summarize_records`, and the sole release caller to be
`wf_bench_records_release`.

One controlled ELF image accepts only four predeclared payload residues modulo
64: **16, 24, 48, and 56**. Wrapper names select the residue through an
environment value; all four wrappers execute the byte-identical ELF. Thus every
controlled timing arm has identical `.text`, including the parallel worker and
all relative placements and PC-relative encodings. Fresh PIE processes may
receive different ASLR bases, so absolute instruction addresses are neither
fixed nor compared and ASLR remains enabled. Each process reports one helper
module data address only after timing so the raw evidence retains that nuisance
variable. This experiment does not relink the worker at alternate addresses
and calls no placement “good” or “bad”. The two registered contrasts
are 16 versus 24 and 48 versus 56, the two observed baseline/candidate `+8`
pairs that do not use the process-confounded W1 residues 0/8.
With a 64-byte-aligned raw allocation, the selected payload residues map to
cell offsets `16→72`, `24→80`, `48→104`, and `56→112` bytes. Each offset is
eight-byte aligned, leaves room for the saved raw pointer immediately before
the cell, and leaves the requested result bytes inside the 128-byte overage.

The controlled link may place code at a different address from the downloaded
candidate, so no absolute-address comparison crosses those two images. The
construction instead extracts the complete 440-byte parallel worker from both
ELFs and requires the bytes to be identical. This connects the controlled
image to the original candidate's hot computation while keeping causal claims
about residues within the one controlled image.

Before either controlled contrast, the exact unmodified baseline/candidate pair
from the same artifact must reproduce the records failure on the experiment
host under the unchanged formal `tests/performance/compare.sh`. The live runner,
checked before any construction, must identify exactly as an AMD EPYC 7763
64-Core Processor or AMD EPYC 9V45 96-Core Processor;
the workflow archives its `lscpu`, kernel, Clang, and linker identity. Records must be
below 0.97 with at least four of five adverse pairs at both W2 and W4, while the
identical-image null has no `FAIL` or `suspect`, and every unchanged kernel has
neither result in reproduction or controlled comparisons. Failure to
reproduce stops the experiment as inconclusive. Each controlled image must then
pass the existing complete-result `verify` mode at W1/W2/W4, report its assigned
payload residue with balanced allocation/release counts at process shutdown,
and pass an identical-residue null before its contrast is read. Every allocation
checks the selected residue immediately without printing; one destructor summary
is emitted after the runner returns, so even the sensitivity control's deliberate
intermediate check adds no output inside its measured interval.

The exact runner source and downloaded runner object are SHA-gated. In that
runner, only `wf_oracle_call` lies between the wall and CPU clock reads;
`wf_oracle_check` invokes the result release after both clocks stop. Residue
reporting occurs once at process shutdown. Every ordinary process must report
exactly six allocations and six releases; a sensitivity-control candidate must
report exactly twelve of each. The same line records the helper's ASLR-selected
module data address outside the measured interval.

The timing protocol, fixture (`records=131072 max_length=255 shape=unicode
seed=812381`), oracle, runner, warmup, five paired passes, calls per sample,
worker widths, slowdown control, and verdict thresholds remain unchanged. No
sample, residue, or direction is selected after timing begins.

Prewritten interpretation:

- Support for the eight-byte payload-residue mechanism requires **both** fixed
  contrasts, 16→24 and 48→56, to put the `+8` arm below 0.97 with at least four
  of five adverse pairs at both W2 and W4. W1 must remain within `[0.97, 1.03]`,
  and all null, slowdown, oracle, and unchanged-kernel controls must pass.
- If both contrasts are within `[0.97, 1.03]` at W2 and W4, this experiment
  finds no sensitivity to these forced residues under the custom overallocated
  path on that host. It does not rule out natural `malloc` placement/alignment
  effects in the original images and does not establish instruction placement
  as the cause.
- A direction that changes between the two residue pairs, an effect at only one
  width, W1 outside its band, a missed assigned residue, failure to reproduce,
  or any control failure is inconclusive. There is no retry with different
  residues or threshold.

A supporting result establishes only forced-residue sensitivity on this custom
overallocated path on the one measured, qualified host. A neutral result establishes
only its absence on that path. Same-host reproduction and the earlier pointer
observations motivate the test but do not isolate causality in the original
images. Neither outcome selects an alignment policy, Box ABI, header order, or
a fat descriptor, nor establishes or rules out the original glibc allocation
or header change as the CI cause. A serious fat-descriptor alternative needs a
later direct representation comparison after causal measurement; it is not a
presumed winner here. Positive sensitivity would justify a subsequent
comparison that preserves the original allocator route.

`prepare-controlled.sh` constructs the single controlled image and four thin
wrappers. `run-controlled.sh` is the sole timing caller, and the disposable
manual-only workflow invokes both through the repository's guarded check
wrapper. Neither script is part of correctness CI or a maintained performance
gate.

Run [35538904894](https://github.com/mbbill/Whitefoot/actions/runs/35538904894)
at `a8de35b0` qualified an EPYC 7763 host and constructed the controlled image,
but its worker audit stopped before any program execution or timing. GNU
objdump included eight inter-function alignment bytes after the 440-byte
worker, while the original extraction stopped only at the next symbol. The
corrected extraction uses the ELF symbol's exact start and size as disassembly
bounds and still requires all 440 bytes to match. Applying it to both retained
images produces identical 440-byte sequences. This is an instrumentation
failure, not a performance result; the preregistered criteria are unchanged.

Run [35539134459](https://github.com/mbbill/Whitefoot/actions/runs/35539134459)
at `754eb772` stopped at the required EPYC 7763 host qualification, before
construction or execution. That workflow checked the model before saving host
identity, so the observed mismatch has no recorded model string. The workflow
now records identity before testing the same requirement. No timing result or
experimental criterion changed; retries of these pre-measurement infrastructure
stops do not select from timing outcomes.

Run [35539370298](https://github.com/mbbill/Whitefoot/actions/runs/35539370298)
at `383577a1` recorded an EPYC 9V74 and stopped at the original 7763-only
qualification. No controlled diagnostic has collected timing data.

Before any diagnostic timing, eligibility was revised to the two model strings
above. Independent formal run
[35538118987](https://github.com/mbbill/Whitefoot/actions/runs/35538118987)
had already observed the same pinned baseline/candidate records images failing
at W2/W4 on an EPYC 9V45 (ratios 0.959639/0.942219, five adverse pairs each).
This supplies a ground to test that model as well, not a pass under the old
protocol or a forced-residue result. The formal run also had quadrature/stencil
suspects, so it does not satisfy this diagnostic's stricter qualification:
either admitted host still must pass fresh same-host reproduction and all
strict controls before residue comparisons have meaning. The comparison stays
within one host, does not pool models, and transfers neither the old 7763
address observations nor a result between models. Outcomes are not resampled;
the fixed residues, oracle, counts, and thresholds are unchanged.

Run [35539634206](https://github.com/mbbill/Whitefoot/actions/runs/35539634206)
at `1a8b207f` received an Intel Xeon Platinum 8573C and stopped before
construction at the revised hardware gate. The identity was retained. These
attempts have produced no forced-residue timing or new performance conclusion;
the formal PR regression remains unresolved.
