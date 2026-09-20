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
