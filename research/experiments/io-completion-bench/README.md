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

## Reproducing

    make -C research/experiments/io-completion-bench verify      # bytes only
    make -C research/experiments/io-completion-bench bench       # macOS table
    make -C research/experiments/io-completion-bench bench-pipe
    make -C research/experiments/io-completion-bench linux       # Linux table

`linux` builds `linux.Dockerfile` and runs the whole pipeline inside one
container, because the generated tree must sit on a container-local
filesystem: measuring a bind mount would measure the host's file sharing
rather than the kernel's I/O path. It passes
`--security-opt seccomp=unconfined`, without which `io_uring_setup` is refused
and both the native baseline and Whitefoot's own Linux adapter silently fall
back.

`FILES`, `MAX_KIB`, `ROUNDS`, `WARMUP`, `PIPE_ROUNDS`, and `PIPE_DELAY_US`
override the shape. `FILES` or `MAX_KIB` change the checksum; `make expected`
prints the new one for `EXPECTED`.

This bundle is deliberately not reachable from the repository's `make check`.
It generates a large tree, runs for minutes, and reports timings no build
should depend on. Generated trees, binaries, and raw output stay under
`$(WHITEFOOT_SCRATCH_ROOT)`; only the summarized table is committed, in
`research/investigations/io-model/RESULTS.md`.
