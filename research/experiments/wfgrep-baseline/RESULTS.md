# WFGREP-BASELINE results

Status: COMPLETE — all four classified cases material; both divergence
mechanisms attributed with profile and control evidence

The protocol, corpus digests, output-identity pins, statistics, bands, and
attribution plan were frozen in the commit carrying `PROTOCOL.md` before any
comparative number was observed. The single run `wfgrep-baseline-1` executed
on 2026-08-06 in protocol order (verify, null-before, bench, null-after,
counters, profile). Raw evidence: `raw/wfgrep-baseline-1.jsonl`, 985 lines,
SHA-256 `d1e66e61b9a807a150f0cf1d366670f16354f61b8c0cb71df114f6531c651a43`.
No sample was deleted, extended, or rerun. Subject binary SHA-256
`d910bc2606162af69925b6ea0925071fb7e29510ea7e7c3815e665c181f40927` built by
the ordinary `whitefootc` path from the frozen program bytes; comparator
`/usr/bin/grep` (BSD grep 2.6.0-FreeBSD) SHA-256
`569588bf23c56895f046b63b029285217e442d46bec1b18498b58fefb50d8f1f`.
Host: Apple M4, 16 GiB, macOS 26.5.2 (25F84), on battery power throughout
with Low Power Mode off and no mid-phase source change (disclosed below).

## Replay status (2026-09-03)

The run above is complete and closed; this bundle is now its record, not a
current driver.

- The committed `raw/wfgrep-baseline-1.jsonl` (985 lines) hashes to
  `1ee5c5a73c3454166bdff746a89faa41bc8bc1132406e3cd6834291636d4fb49`.
  The digest pinned above names the bytes before commit `c4e82fba`
  (2026-08-25) rewrote the
  personal path strings inside four `profile` records; no sample or summary
  value changed.
- The subject was the `tests/programs/wfgrep.wf` of 2026-08-06 (digest in
  `PROTOCOL.md`). Since commit `238ba7ce` (2026-08-18) that file is a
  recursive search printing `PATH:LINE:TEXT` lines, so `make verify` no
  longer matches the pinned outputs from HEAD and the timed phases would
  measure a different program. The Makefile and `runner.rs` stay as the
  frozen protocol's driver; a replay writes its JSON lines to the scratch
  root and never appends to the committed raw file. `MANIFEST.txt` remains
  the pinned corpus and output identity that `../wfgrep-double-walk/`
  verifies its shape sources against.

## Result

Ratio = comparator elapsed / wfgrep elapsed; below 1.0 means wfgrep is
slower. Medians of 30 paired warm rounds; 95% bootstrap intervals.

| Case | wfgrep median | grep median | Ratio | 95% interval | Classification |
|---|---:|---:|---:|---:|---|
| `large` | 292.4 ms | 190.0 ms | 0.647 | [0.643, 0.653] | material wfgrep loss |
| `nomatch` | 292.1 ms | 192.5 ms | 0.656 | [0.649, 0.661] | material wfgrep loss |
| `dense` | 180.6 ms | 200.3 ms | 1.105 | [1.101, 1.130] | material wfgrep win (fragile margin, see below) |
| `many` | 482.6 ms | 291.7 ms | 0.605 | [0.599, 0.610] | material wfgrep loss |
| `floor` | 1.43 ms | 1.68 ms | 1.150 | [1.110, 1.208] | diagnostic, not classified |

The frozen sequential wfgrep is 1.5x slower than the system grep on the
scan-dominated large-file cases, 1.65x slower on the many-small-files case,
and 1.10x faster on the match-dense output-heavy case. Its whole-process
floor is smaller than grep's (1.43 ms vs 1.68 ms). Floor shares are 0.3–0.9%
of every classified case; floor-adjusted ratios (descriptive) change nothing:
large 0.647, nomatch 0.656, dense 1.108, many 0.603.

## Demonstrated precision

Null comparisons (wfgrep against itself, full harness) ran before and after
the bench phase. Relative half-widths: before 0.89% / 0.64% / 0.21% / 0.74%
(large/nomatch/dense/many), after 0.42% / 0.28% / 0.25% / 0.98% — all far
inside the 2% gate. Two null-before intervals, however, excluded 1.0:
`dense` [0.9954, 0.9996] and `many` [0.9819, 0.9965] — a resolved
position bias of 0.26% and 0.89% (the second process in a round runs
slightly faster), the RG-BASE effect now measured at sub-percent scale. Both
null-after intervals contain 1.0. Handling, disclosed rather than
reinterpreted: folding the observed bias into the demonstrated precision
gives effective `w` of 0.9% (large), 0.6% (nomatch), 0.5% (dense), 1.9%
(many) — still within the 2% gate, and every loss classification clears its
band by more than an order of magnitude beyond `w`.

The one margin that does not: `dense`'s material-win label. Its interval
clears 1.10 by only 0.13%, and the per-order sensitivity split (wf-first
median 1.100, grep-first 1.115) straddles the threshold. By the frozen rule
the classification is material win; the robust reading is a ~10% win whose
"material" label sits inside the order wobble. The loss cases are
order-insensitive (large 0.645/0.653, many 0.609/0.602).

## Counters (medians of five, `/usr/bin/time -l`)

| Case | Binary | real | user | sys | instructions | instr/byte |
|---|---|---:|---:|---:|---:|---:|
| large | wfgrep | 0.29 | 0.25 | 0.03 | 5.990e9 | 22.3 |
| large | grep | 0.19 | 0.17 | 0.01 | 4.132e9 | 15.4 |
| nomatch | wfgrep | 0.29 | 0.25 | 0.03 | 5.989e9 | 22.3 |
| nomatch | grep | 0.20 | 0.18 | 0.01 | 4.140e9 | 15.4 |
| dense | wfgrep | 0.18 | 0.12 | 0.05 | 3.221e9 | 24.0 |
| dense | grep | 0.20 | 0.18 | 0.01 | 4.286e9 | 31.9 |
| many | wfgrep | 0.47 | 0.07 | 0.12 | 2.582e9 | 38.5 |
| many | grep | 0.29 | 0.05 | 0.20 | 3.001e9 | 44.7 |

Three shapes stand out before any profile: the large/nomatch divergence is
user-time compute (wfgrep +45% instructions, match density irrelevant);
grep's per-byte cost doubles on dense while wfgrep's stays flat (the sign
flip); and on many wfgrep executes *fewer* instructions and fewer cycles
than grep yet loses 1.65x on wall time — 0.28 s of its 0.47 s is off-CPU.

## Attribution

### `many` (dominant divergence, 0.605): host per-open identity cost — not a Whitefoot layer

`sample` on the enlarged workload puts 80% of wfgrep's wall samples inside
`__openat` (2049/2558; compute 14%, read 3%, close 3%); grep spends 70%
in `__open`. A full `time -l` run showed wfgrep making ~8,475 voluntary
context switches (~2 per file) where grep makes 0. The cause-distinguishing
control (`open-control.c`, same unsigned local clang provenance as the
wfgrep binary) reproduces each side's exact syscall shape over the same
4,096-file list, warm, medians of seven:

| Program | Syscall shape | Wall median | CPU accounted |
|---|---|---:|---|
| `open-control openat` | wfgrep's (dirfd + openat + 4096-byte reads) | 401.5 ms | ~0.16 s |
| `open-control open` | grep's (plain open) | 413.5 ms | ~0.16 s |
| wfgrep | — | 467.7 ms | ~0.23 s |
| grep | — | 294.3 ms | ~0.26 s |

The two control shapes are equivalent (openat is not slower than open), the
control pays the same large off-CPU wall cost as wfgrep despite doing almost
no user work, and wfgrep's total is the control plus its own ~66 ms compute
almost exactly. A platform-signed binary doing the same work is fully
CPU-accounted. The first material divergence of the dominant case is
therefore attributed to the host environment: a per-open wall cost
(~25–30 µs/file) charged to locally built unsigned binaries but not to the
platform-signed comparator — plausibly the endpoint-security layer present
on this host, though the exact OS mechanism is not identified within this
protocol. No Whitefoot layer (language, checks, lowering, runtime shape)
owns it: an identical-syscall C program pays it identically. The remainder
of the gap is the same compute mechanism as `large`.

### `large`/`nomatch` (0.647/0.656): algorithm and source shape; the preregistered suspect refuted as primary

`sample` on `large`: 87% of wfgrep samples are in-`main` compute, 13% in
`read` (65,537 read calls at the frozen 4096-byte chunk ≈ 38 ms). Mapping
the reported return-address offsets onto the measured binary's disassembly:
the openat/read/write/close call sites match offsets +592/+1336/+1712/+2268
exactly, and the hot compute offsets (+1584 = 0xB84, +1632 = 0xBB4) fall in
the inlined `line_matches` literal-match inner loop — the candidate-restart
instruction (which reloads the pattern pointer from the stack each
candidate) and the mismatch-advance path — not in the newline scan
(0xB30–0xB50). Both hot loops are scalar with a live per-byte trap edge:

- newline scan: 9 instructions/byte, including a 2-instruction bounds check
  branching to a trap block (`0x10a4`);
- literal matcher fast path: 13 instructions/byte, including a
  2-instruction bounds check against 4096 branching to a trap block
  (`0x10c8`).

The static model 9 + 13 = 22 instructions/byte matches the measured 22.3
exactly. The compute divergence is attributed to the algorithm/source-shape
layer: every input byte is walked twice by scalar per-byte loops (LLVM
vectorized neither — unlike the guard-clean WF-SCAN-FLOOR kernels, these
loops' data-dependent exits and carry logic kept both per-byte checks), and
the literal matcher is the larger share (~13/22 of compute instructions)
over the newline scan (~9/22). The preregistered suspect — the scalar
newline scan with its retained trap — is thereby **refuted as the primary
cause and confirmed as a secondary contributor of the same shape**. The
required-check layer's ceiling is bounded: the trap edges are 4 of 22
compute instructions (~18%); even free elision of both would move ~292 ms
to ~247 ms and the ratio to ~0.77 — still a material loss. PROOF-1 pressure
from this baseline is real but bounded and secondary; the primary term is
the scalar double-walk shape itself.

### `dense` (1.105 win): the comparator's per-match machinery

grep's own profile shows its architecture: 66% of its `large` samples sit
in `tre_match` with per-call malloc/free churn (`tre_tnfa_run_parallel`
allocating and freeing inside the hot path), and its per-byte instruction
cost doubles from 15.4 to 31.9 when half the lines match. wfgrep's cost is
density-independent (flat 22–24 instr/byte, batched 4096-byte output
writes), so it wins the match-dense case. This is a real product win on
this host against this comparator's fixed-string-through-regex-machinery
architecture; it predicts nothing about better-engineered comparators.

## Deviations and threats, disclosed

1. Two null-before intervals excluded 1.0 (position bias 0.26%/0.89%);
   handled by the conservative fold above, not by rerunning.
2. The battery power source (house default is AC) held stable across all
   phases with Low Power Mode off; the null gates demonstrate the achieved
   precision under it.
3. The profile phase's repetition formula hit the macOS argv limit on
   `many` (pointer bytes count toward `ARG_MAX`); 8 repetitions were used
   instead of 15, with sample windows 3 s/2 s (wfgrep/grep). `large` used
   the formula (52 and 79 repetitions, 10 s windows).
4. Beyond the four preregistered instruments, two labeled post-hoc
   diagnostics were added for attribution only: the `large` profile pair
   (the dominant case's divergence is off-CPU, so ruling on the
   preregistered compute suspect required profiling the compute-bound
   material case) and `open-control.c` with warm median-of-seven timing
   (the cause-distinguishing control for the per-open cost). Neither
   classifies anything.
5. The per-open identity cost makes `many` substantially a host-security
   artifact; on a host that does not charge unsigned binaries per open, the
   expectation is a ratio near the compute-only gap. The absolute per-open
   costs here (~50–95 µs) are far above bare-kernel costs for warm opens.
6. One further Whitefoot-side threat: the read share of `large` (13% of
   wall at 65k syscalls) may also carry the identity cost; this would make
   the compute attribution conservative (the algorithm share can only be
   larger than stated).

## What this baseline does and does not mean

It means: on this host, the frozen first-slice sequential wfgrep — with its
required checks intact and no optimizer facts — completes equivalent
`grep -h -F` work byte-identically at 0.60–0.66x the system grep on scan-
and open-dominated workloads, at 1.10x on the match-dense workload, and
with a smaller process floor; the losses are attributed, with instrument
agreement, to the host's per-open treatment of unsigned binaries (dominant
case, not a Whitefoot layer) and to the program's scalar double-walk
compute shape (compute cases, algorithm/source-shape layer, with the
retained per-byte traps a bounded ~18%-ceiling secondary term).

It does not mean: any language-versus-C machine-quality claim (different
programs, different algorithms — the scoped product comparison is all an
upstream ratio supports); any ripgrep, GNU grep, regex, traversal,
cold-cache, or cross-host claim; and no fraction of any flagship claim.
The `dense` win is against this comparator's architecture, not against the
state of the art. No compiler, language, proof, or specification change is
authorized by these numbers; the scalar-shape and trap findings feed the
PERF-1/PROOF-1 direction decision as knowledge, per the plan's deliverable.
