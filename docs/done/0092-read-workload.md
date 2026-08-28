# Batch 0092 — a read-dominated workload, and reads that really wait

Branch: `batch/0092-read-workload`, from `main` at `79b29665`, with
`batch/0090-ci-real-hosts` merged in at its tip `7c644216` for the workflow the
tables are taken through.
Deliverables: the `WF_IO_NOCACHE` target policy in `compiler/`; the read-heavy
workload, its page-cache verification, and the host-portable `read-bench.sh`
in `research/experiments/io-completion-bench/`; the `bench-linux-read` and
`bench-macos-read` jobs in `.github/workflows/io-hosts.yml`; the
read-dominated section of `research/investigations/io-model/RESULTS.md`; this
record.

## Charter

Batches 0084 and 0086 measured whole Whitefoot programs against hand-written
native shapes, and both measured the same workload: open a small file, read
it, close it, thousands of times. On the macOS host one `openat` costs 116 us
against a 1.9 us `pread`, so that table is very largely a measurement of the
host's endpoint-security stack, and every conclusion it reaches about the
completion framework is reached through a 116 us constant the framework does
not control. Worse, every file number in both records has a warm page cache,
where a read is a memory copy: a model whose whole purpose is to overlap waits
was measured on a workload with almost no waits in it.

So: build a workload whose time is made of reads, on files opened once,
outside any loop, with reads that genuinely reach the device, and measure the
framework against it.

## The mistake this batch made first, and the check that replaces it

The first pass at this table asked for uncached reads and believed the answer.
It set `WF_IO_NOCACHE=1`, generated the tree, and ran the uncached tables
immediately — which is exactly the order that guarantees a warm cache, because
the pages were still resident from the writes that had just made them.
`F_NOCACHE` stops a read *populating* the cache; it does not evict a page that
is already there.

The run said so plainly in its own output. The `N.direct` uncached line was
294 ms for 32,768 reads of 64 KiB, which is 9 us a read, and the prose written
beside it stated that "the host's SSD answers an uncached 64 KiB read in about
14 us". Nine microseconds for 64 KiB is 7 GB/s and 14 us is 4.5 GB/s; both are
memory bandwidth, not an NVMe round trip. The number that should have been the
alarm was instead offered as the evidence that the reads were reaching the
device.

An independent re-run hours later, after ordinary builds had evicted the tree,
put the same line at 4378 ms — 134 us a read, fifteen times slower. Every
ratio in the first table was a ratio between cache hits.

Three things now stand between that mistake and a table.

**The tree is generated uncached.** `make read-uncache` rewrites it through a
descriptor that does not populate the cache, flushes each file, and on Linux
drops its pages. That the descriptor really keeps its traffic out of the cache
is measured, not assumed: three passes over the same eight blocks of a freshly
generated file cost 248, 230 and 315 us, with no drift towards the 7 us a
resident page costs. The same property is what lets the probe read the tree
without warming it.

**The warm tables are warmed on purpose.** `make read-warm` reads every block
of every file back in through plain descriptors, rather than relying on
whatever the previous table left behind. It is a full sequential pass and not
a rerun of the workload, because 32,768 pseudo-random reads leave about two
per cent of the blocks untouched.

**A fresh tree is left alone before it is measured.** `make read-settle` runs
the same probe the table is gated on, quietly, until it passes. Writing half a
gigabyte makes this host's malware scanner walk the new files: a probe every
fifteen seconds on an otherwise idle machine watches residency move from the
first file towards the last over a minute or two and then vanish as the pages
age out, with no benchmark line running. Waiting that out is a wait for an
outside reader to finish, not a softer threshold; the gate probe below is
unchanged and still refuses the table. It applies to the local target only --
the runners have no such reader.

**The label is measured, not asserted.** `read_baseline probe-uncached` times
sixteen positioned reads in each of the eight files, through descriptors that
do not populate the cache and at offsets that differ on every invocation, and
refuses the label unless all but ten per cent of those reads cost more than
40 us. The threshold sits in the gap between the two populations this host
keeps far apart: 6 to 20 us for a read served from the unified buffer cache,
about 134 us for one the device answers. It runs immediately before *and*
immediately after every table, which is what catches the two distinct
mistakes — a tree that was resident when the table started, and something
making it resident while the table ran. `probe-warm` is the same check
inverted, so the warm tables are labelled by measurement too.

The rule bounds a share of sampled reads rather than demanding that every file
pass, and that is a measured decision, not a softened one. With no benchmark
process running at all — only the probe and a CPU loop between passes — one of
the eight files on this host intermittently answers at least half of the
blocks the probe samples in it from memory, for a minute or so, and is then
back to device speed. Something outside the benchmark reads the tree; the
host's endpoint-security stack is the obvious candidate, and `XprotectService`
ran at about 90 per cent of a core throughout these runs. Half of one file in
eight is about six per cent of the tree, which moves every line of a table by
the same six per cent and so cannot move a ratio. A whole file resident is
twelve and a half per cent and is refused. The per-file medians are printed
either way, so a reader sees the tree rather than a verdict.

## What this workload isolates, and what it cannot

It isolates the per-read cost of the completion path. Eight 64 MiB files are
opened once, before any read; then the program performs 32,768 positioned
reads of 64 KiB, or the same number of 4 KiB, at offsets a deterministic mix
decides from the read's own position. Nothing but reads scales with the work.

Three things it deliberately does not answer.

**It is not an open benchmark.** Eight opens remain inside the timed region,
because the runner times whole processes. At 116 us each that is 0.93 ms, in
every line — N, S and C alike — against a table whose fastest uncached line is
over a second. A constant every line pays identically cannot move a ratio, and
that is the whole point of opening once: the open cost stops scaling with the
work. This is the honest account of "the timed region excludes the opens";
there is no marker and no separate clock, because at eight opens there is
nothing worth building one for.

**It does not check every byte it transfers.** Each line folds the first
sixty-fourth of every window into the position-weighted checksum and publishes
the full transferred byte count beside it. The reason is measured: the digest
is a serial multiply-add chain running at about 800 MB/s in both C and
Whitefoot, so folding a whole 64 KiB window costs about 80 us of CPU. Against
the 134 us uncached read that is not a rounding error — it is more than half
the read again, on every line, and it is CPU that the eight-wide program can
spread over helpers while the sequential one cannot, so it would flatter C for
a reason that has nothing to do with I/O. Against a 7 us warm read it would
swamp the table outright and every line would converge whatever the I/O did.
So the fold stays at one sixty-fourth. What the checksum still pins is the
file, the offset, the size and the position of every single read; what it does
not pin is the tail of a window whose first sixty-fourth was correct.

**It says nothing about a slower device.** The host's SSD answers an uncached
64 KiB read in about 134 us. On a device an order of magnitude slower every
ratio here would move in C's favour, because the wait C overlaps would be
larger relative to the handoff that overlaps it. The numbers below are a
statement about this machine.

## The knob

`WF_IO_NOCACHE=1` is a target policy of the same class as `WF_IO_HELPERS` and
`WF_WORKERS`. It is not a language surface: no Whitefoot source names it, no
accepted program changes meaning under it, and no byte any program publishes
differs with it set. The bundle's `read-verify` target checks exactly that —
every line's bytes, with the setting off and on, before any line reports a
time. Absent, and for any value other than the exact text `1`, the runtime
makes no host call at all and the open path is byte-for-byte what it was.

It lives in `compiler/src/backend/completion/file_adapter.h`, in the same
header that already holds the one rule deciding an open's typed outcome, and
both adapters apply it: the bounded POSIX adapter where `openat` returns, and
the Linux io_uring adapter where the ring's open is reaped. It is applied once
to each descriptor an open hands back, and never to one the kind check
refused.

- **Darwin:** `fcntl(fd, F_NOCACHE, 1)`. This is a mode of the descriptor:
  every read through it bypasses the unified buffer cache for the life of the
  open, so a loop of tens of thousands of reads stays uncached throughout.
- **Linux:** `posix_fadvise(fd, 0, 0, POSIX_FADV_DONTNEED)`. Linux has no
  per-descriptor equivalent, so this evicts what is cached at the moment of
  the open and nothing later. See "Not done".
- **`O_DIRECT` is deliberately not used.** It constrains buffer address,
  offset and length alignment, which would change the program's own buffers,
  and a benchmark whose buffers changed with the knob would not be measuring
  the same program.

`research/experiments/io-completion-bench/workload.h` mirrors the same two
host calls for the native baselines, so N and C wait on the same device rather
than on two different cache states.

A test pins the policy's shape. `compiler/src/backend/completion/harness.c`
runs `test_uncached_reads_are_target_policy_only` four times from
`compiler/Makefile`: three with the setting absent and one with it written.
The harness build names the application through `WF_FILE_UNCACHED_APPLY` so it
can count it while still making the host call the shipped policy makes. With
the setting absent the count does not move at all; with it written it moves
once for each descriptor an open hands back and never for one a kind check
refused. Both runs assert the identical open flags — `O_RDONLY | O_NONBLOCK`
for a kind-checked open, bare `O_RDONLY` for an unchecked one — the identical
typed outcomes, and the identical bytes, so what separates them is a cache
hint and nothing else.

## The programs

`programs/read_heavy_narrow.wf` is the natural loop: one read per iteration
into one destination buffer. `programs/read_heavy_wide8.wf` states eight reads
consecutively per round into eight buffers, which is the shape the lowering
can overlap. `_4k` variants of each carry the smaller window. Everything above
`command fn main` is byte-identical to `many_files_narrow.wf`, so `name_at`,
`render_u64` and `fold_bytes` are the same functions the existing programs
use; the one added helper, `block_at`, computes the schedule.

Read *k* of a run reads file `k mod 8` at window `mix(k * golden) mod blocks`.
The narrow program spends eight loops, one per file, stepping *k* by eight;
the eight-wide one spends `reads / 8` rounds of eight consecutive reads. Both
therefore fold exactly the same set of (file, offset, position) triples as the
native baselines do, and the checksum is position-weighted so the order they
are folded in cannot change it.

Both windows do the same number of reads. An uncached read on this host costs
about the same at either size, because it is a device round trip rather than a
bandwidth question, so equal read counts keep the two tables comparable and
keep the 4 KiB uncached table from running for an hour.


## Where the tables were taken, and why not here

The tables this batch records come from GitHub-hosted runners. The local
machine produced the first ones and they are kept, labelled provisional, but
they are not the evidence.

Two reasons, and the first is the batch's own subject. This machine's
endpoint-security stack charges 116 us for an `openat`, and it reads the
benchmark tree while the benchmark is running: with no benchmark process alive
at all, one of the eight files intermittently becomes half resident and is
then reclaimed. That is a measurement of the host, and the whole point of the
read-heavy workload was to stop measuring the host. Second, the machine is
shared and loaded; the local uncached table was taken at a one-minute load
average of 2.5 and its lines were run in groups rather than interleaved, so
drift across the minutes the table took is inside the numbers.

`research/experiments/io-completion-bench/read-bench.sh` is one protocol for
every host that can run it -- the `linux-read` container, the Linux runner,
the macOS runner -- with `ROOT`, `OUT` and `CLANG` naming the paths and
`uname -s` deciding the io_uring lines. This follows batch 0090's decision to
parameterize `linux-bench.sh` rather than copy it; a second script would have
been a second protocol.

It differs from `make bench-read` in one deliberate way. `bench-read` refuses
to print a table whose cache-state label the probe did not confirm, which is
right on a machine whose two populations -- 6 to 20 us from the cache, about
134 us from the device -- are known and far apart. A hosted runner's storage
is not known in advance and its device may be a host-cached network disk that
answers faster than the threshold. There the honest outcome is a table
labelled by what was measured, not the absence of a table, so the script
always runs the probe, always prints its per-file medians, and prints the
verdict on the table's own label line.

## What the runners measured

Run `https://github.com/mbbill/Whitefoot/actions/runs/33130875022`, jobs
`bench-linux-read` (ubuntu-24.04, 4 x Xeon 8370C, ext4) and `bench-macos-read`
(macos-14, 3 x Apple M1 virtual, APFS). Seven recorded interleaved passes
after two warm-ups, medians across passes with the minimum and maximum beside
them. The full tables are in
`research/investigations/io-model/RESULTS.md`; what follows is what they say.

**The model works where there are waits.** The Linux 4 KiB table is uncached
at both ends -- the residency probe confirmed the label immediately before and
immediately after it -- and in it the eight-wide Whitefoot program costs
1463.43 ms against a hand-written 32-deep io_uring pipeline's 1459.84 ms. A
quarter of a per cent. The same source built `--no-overlap` costs 3071.27 ms,
so overlap is worth 2.10 times, and the whole of that distance is device wait:
one read outstanding costs 2993.34 ms and thirty-two cost 1459.84. Eight reads
stated consecutively in Whitefoot source recover all of it. At 64 KiB on the
same runner C is 1.43 times faster than S, 1.04 times faster than an
eight-thread pool and an eight-deep ring, and 1.10 times slower than the best
native line, which is a two-thread pool on a four-core host.

**The model costs where there are none.** Every warm table has C at best level
with S. On Linux that is level: 1.02 and 1.06 times faster, with C's system
time at 0.87 of S's. On macOS it is a loss: 1.27 times slower at 64 KiB and
2.88 times slower at 4 KiB, with five times S's system time. The same program
with `WF_IO_HELPERS=0` costs 41.84 ms where the default costs 94.72, so what
is being paid for is the helper handoff and not the completion state machine.

**The system-time ratio is the sharpest statement.** C at eight wide against S
at eight wide: 0.58 on the Linux uncached 4 KiB table, 0.98 on the Linux
64 KiB one, 1.55 and 1.60 on the macOS cold tables, 5.00 on the macOS warm
4 KiB one. Overlap does not cost kernel work on Linux; it saves it. The model
does not change between those hosts. The adapter does.

**The many-files result from batch 0084 is retired.** The macOS runner ran
that workload too, and it is the first macOS reading of it not taken through
an endpoint-security stack: one open, read, close and fold of a small file
costs 17.2 us here against 116 us on the maintainer's machine. With that
constant gone, C is 1.20 times *slower* than S at eight wide and 1.51 times
slower at four, where batch 0084 recorded C 2.05 times *faster*. The 2.05x was
the 116 us `openat` -- a wait that large is worth overlapping whatever the
handoff costs, and when it drops to 17 us the handoff is all that remains.
Batch 0090 reached this conclusion on Linux hardware; it now holds on macOS,
and no table in the document still credits the overlap lowering with a win on
the many-files workload.

**Every reading was taken three times.** The workflow ran again at commits
`e2e4535d` and `031df30e` (runs
[33131934257](https://github.com/mbbill/Whitefoot/actions/runs/33131934257) and
[33133182075](https://github.com/mbbill/Whitefoot/actions/runs/33133182075)),
each on separately provisioned hosts -- the second Linux job on an AMD EPYC
7763 rather than an Intel Xeon 8370C. Every ordering holds on all three.

The three runs also say something one could not: **the completion build's cost
is pinned to the native floor and the sequential build's is not.** Across the
three Linux runners `C.wide8.default` uncached varies by 9.0 per cent at
64 KiB and 1.3 at 4 KiB, and `N.pool8` by 1.0 and 0.6 -- while the same source
built `--no-overlap` varies by 44.8 and 58.8. A program with one read
outstanding pays 32,768 times whatever that host's per-read latency happens to
be, and these hosts differ by more than half; a program with eight pays what
the device delivers, which varies far less. That is the design's own claim,
and it is the first time this repository has measured it.

The later runs also refused the macOS uncached label *before* their tables ran
rather than only after, printed that on the label line, and went on. That is
the labelling behaviour these jobs were built for, and it is the reason to
read the macOS cold rows as an ordering rather than as a device measurement.

## Judgment calls

1. **`linux-read-bench.sh` became `read-bench.sh`, host-portable, rather than
   being copied for macOS.** `ROOT`, `OUT` and `CLANG` name the paths and
   `uname -s` decides the io_uring lines, so the container target and both
   runner jobs execute one protocol. This follows batch 0090's decision for
   `linux-bench.sh`; a second script would have been a second protocol.
2. **The script labels; `make bench-read` refuses.** On the maintainer's
   machine the two populations the 40 us threshold separates are known and far
   apart, so refusing to print an unconfirmed table is right there. A hosted
   runner's storage is not known in advance, and the honest outcome of a
   surprising device is a table labelled by what was measured rather than no
   table. Both probes always run and always print; only the consequence
   differs, and the script's header says so.
3. **The 64 KiB Linux table is published as "cold start", not "uncached".**
   Its after-probe refused the label and the record says why: 2 GiB of reads
   over a 512 MiB tree on a 16 GB host ends with the tree resident, and Linux
   has no per-descriptor mode that would stop it. Every line still starts cold,
   because the eviction runs on each open, and every line covers the same
   schedule, so the self-warming is a constant common to all of them. The
   alternative -- moving the threshold until the label passed -- would have
   been the batch's original mistake with a different number.
4. **The macOS cold tables are published with their drift stated.** Their
   after-probes found 37 to 55 us reads where the before-probes found 76 to
   316 and the warm probes find 4 to 6. That is a cache below the guest, which
   `F_NOCACHE` cannot reach: a virtualized host has three populations, not
   two, and a single threshold chosen between a physical machine's 6 us and
   134 us falls inside the middle one. It is reported rather than worked
   around, and the interleaved schedule is what keeps it from becoming a
   ranking.
5. **The runner became a pass-interleaving runner in this batch.** It runs the
   whole plan, then runs it again reversed, and takes each line's median across
   passes. A grouped schedule measures the first line against a different
   machine from the last one whenever the host drifts, and both of these hosts
   drift. The local provisional table predates the change and is labelled with
   that among its reasons.
6. **The gate workflow is red on this branch and was red before it.** On the
   tip of `batch/0090-ci-real-hosts` that this branch merged, run
   [33128887524](https://github.com/mbbill/Whitefoot/actions/runs/33128887524),
   `gate-macos` fails at
   `backend::tests::parallel::an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not`
   and `gate-linux` at two `backend::stack_ledger::tests` rows and
   `backend::tests::parallel::the_runtime_replaces_the_modules_weak_refusal`.
   These are host-timing and host-layout observations, and `gate-macos` fails
   the same way on this branch. Nothing in this batch touches them. Canonical
   `make check` passes on the maintainer's machine, which is the merge
   requirement.

## Approval classes

- **No specification change.** `spec/kernel-spec.md` is untouched.
- **No conformance change.** No case, manifest line, adapter, runner, or
  collection wiring is added, modified, deleted, or renamed.
- **No new root entry.** `.github/workflows/io-hosts.yml` gains two jobs;
  `research/experiments/io-completion-bench/` gains one renamed script.
- Ordinary compiler, research, and documentation changes otherwise.

## Not done

- **Linux cannot hold a descriptor uncached.** `F_NOCACHE` is a mode of the
  descriptor and holds for every read through it; `POSIX_FADV_DONTNEED` is an
  action, and evicts only what is resident when the open runs. So on Linux
  every line starts from a cold tree and then warms it as it reads, and the
  probe that follows a Linux uncached table measures how far that went rather
  than confirming a label. `O_DIRECT` would hold, but it constrains buffer
  address, offset and length alignment, which would change the program's own
  buffers and so change what is being measured. The Linux uncached tables are
  therefore *cold-start* tables, and the record says so where they appear.
- **The many-files workload was not re-measured on the local machine.** The
  macOS runner's reading of it is new evidence about the open path; the local
  reading it should be compared against is the one batches 0084 and 0086
  already recorded.
- **No slower device was measured.** Every ratio here is a statement about
  storage that answers a 64 KiB read in roughly a hundred microseconds. On a
  device an order of magnitude slower every ratio would move in C's favour,
  because the wait C overlaps would be larger relative to the handoff that
  overlaps it.
- **The pipe workload was not re-run.** It discriminated nothing in batch
  0084 and nothing in this batch touches it.
- **The threshold was not recalibrated per host.** `READ_THRESHOLD_US` is one
  number, chosen from the two populations a physical machine keeps far apart.
  The macOS runner has three, and the middle one straddles the line. A probe
  that measured each host's populations first and placed its own threshold
  between them would label a virtualized host correctly instead of reporting
  a refusal for a reader to interpret. This batch reports the refusal.
- **Why macOS needs a helper to overlap at all was not attacked.** The macOS
  numbers say the bounded POSIX adapter buys overlap with system time, and
  that a warm workload therefore pays for a pool it cannot use. Whether a
  Darwin target should reach for a real completion path, or whether the pool
  should shrink when the operations it runs stop waiting, is a design question
  this batch only measured the cost of.
