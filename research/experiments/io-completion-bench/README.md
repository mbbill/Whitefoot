# io-completion-bench

Program-level measurement of the unified-state completion I/O model against
the best hand-written native shape and against Whitefoot's own sequential
build.

## What it serves

`research/investigations/io-model/RESULTS.md` held only C-level
microbenchmarks of the completion core — round-trip cost, cached `pread`
delta, park/wake parity. Those numbers cannot answer the question the design
stands or falls on: whether a whole Whitefoot program that does real I/O
reaches the best native performance. This bundle answers that question and
supplies the evidence section that RESULTS.md now carries.

It is removed when the completion model stops being an open performance
question — when the numbers are stable, the bar is settled, and no further
runtime or lowering change is being measured against them.

## The three lines

Every line publishes the same bytes; a line that publishes anything else
cannot report a time.

- **N** — the best hand-written native C shape. `baseline.c`: a single-threaded
  blocking loop, a pthread pool over a striped index range, and on Linux a raw
  `io_uring` read pipeline (`uring_baseline.h`, kernel ABI directly, no
  liburing). Compiled `-O2` with no handicap.
- **S** — the Whitefoot program built with `whitefootc --no-overlap`, which
  emits the module a compiler with no overlap lowering at all emits. Every I/O
  call is an ordinary direct call.
- **C** — the same Whitefoot source built the way it ships. The C and S lines
  are one source compiled two ways, so the pair is a statement about the
  lowering rather than about two programs.

## Workloads

### The many-files workload

`programs/many_files_wide.wf` opens and reads four independent generated files
per round, four opens and then four positioned reads written consecutively so
the lowering can overlap them. `programs/many_files_wide8.wf` is the same
shape hand-widened to eight, so the comparison against an eight-thread pool
and a deep io_uring baseline is made at a matched width.
`programs/many_files_narrow.wf` is the same work written as the natural
one-file-at-a-time loop; it exists to measure what a writer gets who does not
hand-widen, and the answer is no overlap at all.
`programs/pipe_relay.wf` pushes two independent byte streams at two
independent consumers through `command.stdout` and `command.stderr`.

The generated tree and the checksum are defined once, in `workload.h`, and
shared by the generator, the native baselines, and the Whitefoot programs. The
checksum is position-weighted so a four-wide lane split and a one-at-a-time
loop fold to the same value.

### The read-heavy workload

`programs/read_heavy_narrow.wf` and `programs/read_heavy_wide8.wf`, with their
`_4k` variants, answer the question the many-files workload cannot. That
workload opens a file per unit of work, and on the macOS host one `openat`
costs 116 us against a 1.9 us `pread`, so its table is mostly a measurement of
the host's endpoint-security stack. These four programs open eight 64 MiB
files once, before any read, and then perform tens of thousands of positioned
reads into them: 32,768 reads of 64 KiB, or the same number of 4 KiB, so
what the time is made of is reads. Both windows do the same number of reads,
because an uncached read on this host costs about the same at either size --
it is a device round trip, not a bandwidth question -- so equal read counts
keep the two tables comparable and roughly equal in wall time.

`read_heavy_narrow.wf` is the natural loop: one read per iteration into one
destination buffer, eight such loops so that each stays on one file.
`read_heavy_wide8.wf` states eight reads consecutively per round into eight
buffers, which is the shape the lowering can overlap. Read *k* of a run takes
its file and its window-aligned offset from *k* alone, so the narrow program,
the eight-wide one, and every native baseline traverse exactly the same list
and fold exactly the same value.

The opens are inside the timed region — the runner times whole processes — but
there are exactly eight of them in every line, N, S, and C alike. At 116 us
each that is 0.93 ms against a table whose fastest line is over a hundred
milliseconds, and a constant every line pays identically cannot move a ratio.
This is what "open-once" buys: the open cost stops scaling with the work.

Each line folds the first sixty-fourth of every window rather than all of it,
and publishes the full transferred byte count beside the checksum. The reason
is in `workload.h`: the digest is a serial multiply-add chain running at about
800 MB/s, so folding a whole 64 KiB window costs about 80 us of CPU against a
134 us uncached read and a 7 us warm one. Folding everything would make the
warm table pure compute, and would add to the uncached table CPU that the
eight-wide program can spread across helpers and the sequential one cannot.

### WF_IO_NOCACHE

`WF_IO_NOCACHE=1` is a target-policy knob of the same class as `WF_IO_HELPERS`
and `WF_WORKERS`: it is never a language surface, no Whitefoot source names
it, and it changes no byte any program publishes — the `read-verify` target
checks every line's bytes with it off and on. What it changes is where a
read's bytes come from. On Darwin the runtime applies `fcntl(fd, F_NOCACHE, 1)`
to each descriptor an open hands back, which is a mode of the descriptor: every
read through it bypasses the unified buffer cache for the life of the open. On
Linux it applies one `posix_fadvise(fd, 0, 0, POSIX_FADV_DONTNEED)`, which
evicts what is cached at the moment of the open. `O_DIRECT` is deliberately not
used, because its alignment constraints would change the program's own buffers
and so change what is being measured.

The knob lives in `compiler/src/backend/completion/file_adapter.h` and is
applied by both the bounded POSIX adapter and the Linux io_uring adapter, once
per descriptor an open hands back and never to one a kind check refused.
`workload.h` mirrors it for the native baselines, so N and C wait on the same
device rather than on two different cache states.

### Proving a table is uncached

The knob alone does not make a table uncached. `F_NOCACHE` stops a read
populating the page cache; it does not evict a page that is already resident,
so a table run over a tree that was just written, or just read by a warm
table, is served from memory however loudly it asks for the device. Batch
0092 published a table that way once; `docs/done/0092-read-workload.md`
records it.

Three things now stand between that mistake and the table.

`make read-uncache` regenerates the tree so nothing is resident: the generator
writes each file through a descriptor that does not populate the cache,
flushes it, and on Linux drops its pages. That such a descriptor really keeps
its traffic out of the cache is measured rather than assumed — three passes
over the same eight blocks of a freshly generated file cost 248, 230 and
315 us on this host, with no drift towards the 7 us a resident page would
cost — which is what lets the probe below observe residency without creating
it.

`make read-warm` is the other half: it reads every block of every file back in
through plain descriptors, so a warm table has the state it claims. A full
sequential pass rather than a rerun of the workload, because 32,768
pseudo-random reads would leave about two per cent of the blocks untouched.

`read_baseline probe-uncached` then checks the claim rather than trusting it.
It times sixteen positioned reads in each of the eight files, through
descriptors that do not populate the cache and at offsets that differ on every
invocation, and refuses the label unless all but ten per cent of those reads
cost more than 40 us — the gap between a 6-to-20 us cache hit and a 134 us
device read on this host. It runs immediately before and immediately after
every table, which is what catches both the tree being resident when a table
starts and something making it resident while the table runs. `probe-warm` is
the same check in the other direction, so the warm tables are labelled by
measurement too.

## Reproducing

    make -C research/experiments/io-completion-bench verify       # bytes only
    make -C research/experiments/io-completion-bench bench        # macOS table
    make -C research/experiments/io-completion-bench bench-pipe
    make -C research/experiments/io-completion-bench linux        # Linux table

    make -C research/experiments/io-completion-bench read-verify  # bytes only
    make -C research/experiments/io-completion-bench bench-read   # macOS tables
    make -C research/experiments/io-completion-bench linux-read   # Linux tables

`linux` builds `linux.Dockerfile` and runs the whole pipeline inside one
container, because the generated tree must sit on a container-local
filesystem: measuring a bind mount would measure the host's file sharing
rather than the kernel's I/O path. It passes
`--security-opt seccomp=unconfined`, without which `io_uring_setup` is refused
and both the native baseline and Whitefoot's own Linux adapter silently fall
back.

`linux-read` runs the read-heavy tables in the same container for the same
reasons, and adds the `io_uring` baselines at depths 4, 8, and 32.

It does that by running `read-bench.sh`, which is one protocol for every host
that can run it: the container, the project's Linux runner, and its macOS
runner. Only paths and the host's own capabilities differ -- `ROOT`, `OUT`,
`CLANG` and `CARGO_TARGET_DIR` name the paths, and `uname -s` decides whether
the io_uring lines are in the plan. The `io-hosts` workflow's
`bench-linux-read` and `bench-macos-read` jobs run exactly those bytes.

The script differs from `bench-read` in one deliberate way. `bench-read`
refuses to print a table whose cache-state label the probe did not confirm,
which is the right rule on a machine whose two populations are known and far
apart. A hosted runner's storage is not known in advance, and its "device" may
be a host-cached network disk that answers faster than the threshold; there
the honest result is a table labelled by what was measured rather than no
table at all. So the script always runs the probe, always prints its per-file
medians and its verdict, and prints that verdict on the table's own label
line.

The two probes around a table are not worth the same on both hosts, and the
script says so where it runs them. Before the table each host is making the
same claim and the probe checks it. After it, only Darwin is: `F_NOCACHE` is a
mode of the descriptor, so a Darwin table that asked for uncached reads cannot
have populated the cache itself, and a refusal there means something outside
the benchmark made the tree resident. Linux has no per-descriptor mode; its
one lever evicts at open and nothing later, so every Linux line starts from a
cold tree and warms it as it reads. The Linux after-probe measures how far
that went and is reported as that, not as a verdict.

`FILES`, `MAX_KIB`, `ROUNDS`, `WARMUP`, `PIPE_ROUNDS`, and `PIPE_DELAY_US`
override the many-files shape. `FILES` or `MAX_KIB` change the checksum;
`make expected` prints the new one for `EXPECTED`. `READ_FILES`, `READ_KIB`,
`READS_64K`, `READS_4K`, and `READ_ROUNDS` do the same for the read-heavy
shape, and `make expected-read` prints the two checksums they imply.

`bench-read` prints four tables: 64 KiB and 4 KiB, each with the page cache
warm and with `WF_IO_NOCACHE=1`. Both cache states publish the same bytes,
which `read-verify` checks before any line reports a time, and each table's
cache state is checked by the probe above before and after it runs.
`READ_PROBES`, `READ_THRESHOLD_US`, and `READ_TOLERANCE_PERCENT` set that
check.

This bundle is deliberately not reachable from the repository's `make check`.
It generates a large tree, runs for minutes, and reports timings no build
should depend on. Generated trees, binaries, and raw output stay under
`$(WHITEFOOT_SCRATCH_ROOT)`; only the summarized table is committed, in
`research/investigations/io-model/RESULTS.md`.
