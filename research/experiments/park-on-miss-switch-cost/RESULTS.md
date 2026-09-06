# Park on miss: the cost of one stack switch

Design `research/investigations/io-model/PARK-ON-MISS.md` §12, first item:
"the context switch itself: cost of one save/restore of callee-saved
registers and the stack pointer, against the 2.2 microsecond park-and-wake
figure the tree measured. A switch that is not well under that number removes
the design's reason to exist."

`switch.c` is the switch the design names for POSIX — a hand-written spill of
the callee-saved registers and the stack pointer, per architecture, with no
signal-mask syscall — timed as a ping-pong between two stacks on one thread.
Beside it on the same host: `swapcontext`, which carries the `sigprocmask`
syscall the design refuses to pay, and a condition-variable park-and-wake
between two threads, which is the figure the design is measured against.

## Measured 2026-09-04

Host: Darwin 25.5.0 arm64 (Apple silicon laptop, the owner's machine). Four
runs of `make run`; the first row is the first run, the range is over all four.

| operation | per operation | rounds | range over four runs |
|---|---|---|---|
| stack switch (hand-written, arm64) | 10.4 ns | 20,000,000 round trips | 9.8–10.4 ns |
| `swapcontext` | 347 ns | 1,000,000 round trips | 345–355 ns |
| condvar park-and-wake, two threads | 872 ns | 100,000 round trips | 872–934 ns |
| park-and-wake ÷ switch | 84× | | 84–95× |

The design's bar is met by a wide margin: one switch costs about one percent
of one park-and-wake on this host. `swapcontext` is thirty-five times the
hand-written switch, which is the syscall the design refuses to pay, measured.

## Caveats, stated rather than implied

- One host, one architecture. The x86-64 arm of `switch.c` compiles but has
  not been run here; the Linux runner gives that number later in the batch.
- The park-and-wake here is 0.87–0.93 µs, not the 2.2 µs the
  `par_runtime.c` comment reports. That comment measured a different machine
  and a different primitive (the runtime's own park); the number that matters
  for the bar is the ratio on one host, and it is the same conclusion at
  either figure.
- The ping-pong is cache-hot and switches between two stacks only. A switch
  in the scheduler loop lands on a colder stack and pays the misses of the
  frame it resumes; that cost belongs to the resumed work, not to the
  switch, and is what the four-stage chain measurement (§12, fourth item)
  will show.
- No guard-page fault, no signal delivery, and no floor bounds update are on
  the timed path. The design's switch also writes the floor's three bounds
  per switch (§5); that is three stores.

Removal condition: this bundle is superseded when the scheduler core lands
and `compiler/src/backend/completion/harness.c` or the core's own test measures
the switch inside the runtime; until then it is the number the design cites.
