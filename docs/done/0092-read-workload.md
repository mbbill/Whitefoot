# Batch 0092 — a read-dominated workload, and reads that really wait

Branch: `batch/0092-read-workload`, from `main` at `79b29665`.
Deliverables: the `WF_IO_NOCACHE` target policy in `compiler/`, the read-heavy
workload and its page-cache verification in
`research/experiments/io-completion-bench/`, the read-dominated section of
`research/investigations/io-model/RESULTS.md`, this record.

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

## What the tables say

The tables are in `research/investigations/io-model/RESULTS.md`; run
[33130875022](https://github.com/mbbill/Whitefoot/actions/runs/33130875022) at
commit `6ac36126`, `bench-linux-read` on `ubuntu-24.04` (Xeon Platinum 8370C,
4 CPUs, 16 GiB, ext4 on a local disk) and `bench-macos-read` on `macos-14`
(virtual Apple M1, 3 CPUs, 7 GiB, APFS). Seven recorded passes after two
warm-ups, the plan reversed on alternate passes.

**The uncached Linux tables are the first result that meets both halves of the
standing bar.** At width eight the shipped completion build finishes 1228.53 ms
against the eight-thread native pool's 1278.13 and a raw io_uring pipeline's
1274.99 at depth 8; at 4 KiB it finishes 1463.43 against `N.uring32`'s 1459.84,
a fifth of a per cent apart. Against the same source compiled with no overlap
lowering it is 1.43x faster at 64 KiB and 2.10x at 4 KiB. The bar asked for
within ten per cent of the best native shape at matched width; this is faster
than it. The honest qualifier is that the fastest native line in the 64 KiB
table is `N.pool2` at two threads, not eight, and against that C is 10.5 per
cent behind.

**Warm, the same lowering costs what it has always cost**, because there is no
wait to overlap and what remains is the submission, the token and the join.
Linux: within two per cent either way. macOS: 1.27x slower at 64 KiB and 2.88x
at 4 KiB — and `WF_IO_HELPERS=0` on the same program brings that to five per
cent, so the distance is the helper pool spun up for operations that never
wait, not the lowering.

**The system-time column is the mechanism.** On the Linux 4 KiB uncached table
the completion build spends 202 ms of system CPU where the sequential build
spends 349 and finishes 2.1 times faster: one ring submission carries eight
reads that the sequential build enters the kernel eight times for. On macOS the
ratio inverts — 756 against 474 uncached — because the bounded POSIX adapter
has no ring and buys its overlap with threads, spending system time rather than
saving it. Uncached that trade returns 1.73x; warm it costs 1.27x.

**The macOS many-files run retires a claim this project has carried since batch
0084.** "Overlap is worth about two times on a program that exposes width" was
measured on a machine whose endpoint-security stack charges 116 us for an
`openat`. The same workload on a macOS runner without that stack costs 17.2 us
per open-read-close against 139 us on the maintainer's M4 — eight times faster
on a virtual M1 with three cores — and there the completion build is **1.20x
slower** than its own sequential build, the same sign as batch 0090 found on
Linux hardware. The two-times figure was the hook, not the lowering.

## What the macOS runner taught about the threshold

The residency probe confirmed both macOS uncached tables before they ran and
refused both after, and the refusal is worth more than the confirmation.

`F_NOCACHE` is a mode of the descriptor, so no line in a macOS uncached table
can have populated the page cache, and the after-probe's numbers agree that
nothing was resident: a resident 64 KiB read on that host costs 5.5 to 6.0 us
and the after-probe saw 37 to 55 us. But it saw about half what the same files
cost before the table, which ran 75 to 315 us. There are three populations on
a virtualized host, not two — the guest's page cache, the hypervisor's disk
cache, and the device — and a threshold calibrated on a physical machine
between 6 us and 134 us cannot separate three. The 40 us line falls inside the
middle one.

The probe was right to refuse and the script was right not to stop: the
finding is that the macOS runner's storage got faster as the table ran, which
is a fact about the table that a reader needs and that a refusal would have
hidden. Both directions of that warming land on every line equally, because
the passes alternate.

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
