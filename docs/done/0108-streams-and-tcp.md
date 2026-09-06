# Batch 0108 — streams and TCP (specification v0.46, the first API that waits)

Branch: `io/t4-resource-relations` (PR #13), continuing from batch 0107 on the
same branch. Deliverables: the v0.46 amendment (the standard streams as
`InputStream` and `OutputStream`, the descriptor factory as `HandleFactory`,
`SocketAddress`, `TcpListener`, `TcpConnection` and the seven TCP operations
under T4), the TCP runtime on POSIX and on Windows, the staged hand-out of a
may-suspend user call, the control benchmark against the io_uring and epoll
references, and this record. The plan is `docs/current-plan.md`, Batch 3; the
design is `research/investigations/io-model/NETWORK.md`, whose section numbers
this record uses.

## Why

Batch 0107 left one thing it could not settle its own numbers on: the surface
the language had, read-only files, directory enumeration and two standard
outputs, cannot exercise a real wait. A cached read does not wait and a cold
read waits briefly and uniformly, so the scheduler that parks a stack on a
miss had never faced a wait another program decides the length of. The owner's
revision of 2026-09-05 made the network that first waiting API rather than
standard input alone, because concurrency's real demand is the network and a
loopback is a controlled peer; standard input came along as the first instance
of the same stream design. Every socket resource passes the T4 question the
file permit passed: it is a value in the signature, and every dependency
between it and an existing resource is one the checker can see (§2).

## 1. The amendment (slice 1, `5c31b74`, `6884dd2`, `8cd4bae`)

The proposal was revised twice before it was written into the specification.
Full duplex is designed in rather than added later: a connection is a receive
half and a send half from accept or connect, the pair close returns the
credit, and the derived release of a half is the host's half-close. The
owner's decisions (§8): `InputStream` and `OutputStream`; `TcpConnection` as a
system-declared struct of `TcpReceive` and `TcpSend`, the first such struct;
the factory renamed `HandleFactory` with `HandlePermit`, `command.handles` and
`reserve_handle`, because a listener and a connection draw one credit each
from the same capacity a file open draws from; the backlog fixed by the
target; TCP only, address literals only; and the reference for the control
test the fastest existing solution regardless of language, because the target
is first place.

`8cd4bae` landed the amendment as one change: `spec/kernel-spec.md` declares
`Status: ACTIVE v0.46` over v0.45's archived bytes, the `ACTIVE-SPEC:` record
is appended in `governance/APPROVALS.md`, the identity module is regenerated,
the derivation ledger carries the v0.46 section. [SYS-15] adds `InputStream`
at entry ordinal 5 and `read_next`; [SYS-16] adds `SocketAddress` with two
total pure constructors; [SYS-17] adds `TcpListener` with `tcp_listen`,
`tcp_accept`, `tcp_connect` and `close_listener`; [SYS-18] adds
`TcpConnection`, whose `receive` and `send` fields are ordinary places, so two
`&uniq` loans on disjoint fields coexist under [OWN-5] and a partial move
kills the whole binding under [OWN-1]. Eleven conformance cases were added and
thirty-one respelled with their verdicts unchanged. `read_next` ran end to end
on POSIX in the same slice; the TCP operations were declared, checked, lowered
and emitted, and stopped at target qualification with a missing mapping,
never a source rejection, until their routes existed.

Two defects the amendment exposed in the compiler: lowering decoded a system
constructor ordinal against the active inventory rather than the unit's own,
which a second nominal-record block exposed and the prefix-differential
property caught; and no checked borrow form carried a field path for a
non-buffer type, which `&uniq connection.receive` needed.

## 2. The runtime on POSIX (slice 2, `debece8`, `1ad703b`)

Every TCP operation lowers and runs on POSIX. `qualification.rs` maps
ordinals 22 through 28 on the native column. The emitter carries the seven
wrappers in the one submit-then-join shape the opens and the reads have, with
the three outcome enums built as `FileOpenOutcome` is and the permit handed
back in every failed variant. `wf_file_request` gains six kinds inside the
record's 160 bytes: listen, accept, connect, receive, send and the half-close;
the accept's peer record lives in the arm the accept alone uses, because
twenty-four more bytes on the shared result head would put the record past the
block an emitted frame reserves. `file_posix.c` executes every kind;
`linux_io_uring.c` carries accept, connect, receive and send on the ring, with
a connect's socket and native address made in the submitting call; listen,
bind and the half-close stay on the adapter; `file_adapter.c` keeps the pair's
two-count so the second release of a connection closes the object and spends
the credit. `SO_REUSEADDR` is deliberately not set: two binds of one port are
the program's own source-order conflict and `AddressInUse` is the host's
answer to the second [SYS-17].

`tests/programs/` gains `tcp_echo.wf`, `tcp_client.wf`, `tcp_fanout.wf` and
`tcp_refused.wf`, each run on both routes against a `std::net` peer, and the
five `systcp-*` conformance cases that expected `unsupported` expect `accept`,
because their subject moved and no expectation about the language changed.

`1ad703b` fixed what the macOS gate found: the completion harness's accept
published the zero address because the family was read as a sixteen-bit
number at the record's first byte, which is `sa_family` on Linux and `sa_len`
beside a one-byte `sa_family` on Darwin. It now copies the host's own `struct
sockaddr` and reads the member, and `socket_address.h` carries that rule for
every platform.

## 3. The runtime on Windows (slice 3, `ae7c508`, `29f4e14`, `c966ee0`)

The six socket operations reach the same two Windows engines the file
operations reach, in shared code with no twin files. `file_windows.c` gains
listen, connect, accept, transfer and shutdown arms over Winsock;
`windows_iocp.c` carries the connect through `ConnectEx` (resolved once by
`WSAIoctl`), and the receive and the send through `WSARecv` and `WSASend` on
the record's own `OVERLAPPED`; `windows_runtime.c` learns a socket descriptor
class, the once-per-process `WSAStartup`, and the mapping that makes a
`WSAGetLastError` code and the port's own Win32 code for one condition answer
one [SYS-7] class. The address vocabulary moved out of `file_posix.h` into
`socket_address.h`, which every engine on every platform includes.

The accept stays on the shared file adapter on Windows, by measurement rather
than preference: an `AcceptEx` address pair is `2 * (sizeof(sockaddr_in6) +
16)` = 88 bytes that must live until the operation completes, and the record
is exactly 160 bytes with its arms at the ceiling `contract.h` asserts. The
record may not grow and the runtime allocates nothing at run time, so there is
nowhere for those bytes to go; `wf_windows_iocp_carries` says so.

The real host judged twice. `bridge_default_probe.c`'s loopback round trip
passed on both routes in the first run (`tcp-port=45231 tcp-ring=3`), while
the echo server never listened on the port the step passed it: the four
programs copied the argument with `host_copy_bytes`, the lossless route of
[HOST-2], which on a UTF-16 host family hands the program the native code
units, so "60527" arrived as `36 00 30 00 ...` and the decimal scan ended at
the first zero byte. `29f4e14` reads the port through the text route
`host_copy_utf8` in all four programs, and the io-hosts step lists the
process's own TCP endpoints when a server does not listen, so a wrong port
names itself. `c966ee0` put Winsock on the Windows bench's one hand-written
link, which slice 3 had missed. The `completion-windows` job then ran the echo
on the completion port, under `--par`, and on the adapter, and the refused
connect on both routes, in twenty seconds.

## 4. The staged hand-out of a may-suspend user call (`7dbbf2e`)

Slice 2 found where the widening to a real server loop starts: [PAR-3] staged
`tcp_fanout.wf`'s accept loop as written, but the backend handed out only a
system operation with a typed adapter at the staged point, so a permitted
user call there kept its wrapper and ran on the loop's own stack, and the four
peers were served in turn. `7dbbf2e` takes the grant. The bounded-batch
recognizer in `lowering/builder/loops.rs` admits a second staged tail, the
call bound by a `let` with the statements after it as the remainder, and
selects the lane form by the cut's kind alone, a may-suspend user call; a
system operation bound by a `let` is an explicit, tested decline, because a
submitted operation's drain publishes its outcome at the block boundary and a
remainder in the same block would read a value that does not exist yet.

The pipeline carries the form as one mechanism with the ring: the slot holds
a lane frame's address instead of a completion record, the iteration's own
bindings ride the same per-slot storage as captured scalars, and their
compiler-derived releases run in the drain on the value that iteration
allocated. The emitter acquires a frame, fills it inside the granted edge and
publishes it with the same thunk the [PAR-1] compute hand-out uses, or on a
refused acquisition runs the same call where it is written and leaves its
answer in the same ring element; the drain has one thing to do either way. The
window ceiling for the form is the lane's own slot count, `LANE_SLOTS` beside
`LANE_FRAME_BYTES`, pinned to `core.h` by a test. Four peers that connect
before any of them speaks are answered in the order they speak, on both routes
(`four_peers_are_served_at_once_under_par_on_both_routes`); only
`tcp_fanout.wf` emits the form and every other corpus program's `--par` bytes
are unchanged.

## 5. What the first real wait exposed in the runtime

- **A peer-bound request must be a helper's (`95e85a2`).** The three-core
  macOS gate runner served `tcp_fanout.wf`'s peers in turn. Read from a hung
  server with gdb on Linux at `WF_WORKERS=3`: every worker was inside
  `wf_bridge_run_own`, the joining thread's own claim, blocked in the host's
  `recv` on a silent peer, and the fourth connection had no thread left to
  accept it. The adapter's helper pool grew only on the measured wait
  verdict, which cannot see a wait that has not happened yet. A request whose
  kind waits on a peer (accept, receive, connect, send) is now peer-bound by
  kind: enqueueing one grows a helper whenever the cap allows; a scheduler
  thread's progress pass takes only requests no peer can hold up while a pool
  may exist, and anything under a pinned zero pool; a pool stack withholds its
  own peer-bound claim at the join once a helper exists, while a thread
  waiting in place keeps it; and peer-bound executions are not sampled into
  the wait mean. Adapter-route socket concurrency is thereby bounded by
  `WF_BRIDGE_MAX_HELPERS`, and the four-peers case pins `WF_WORKERS=3` so the
  proof does not depend on the host's core count: before the change it failed
  seven of twelve runs here, after it twelve of twelve.
- **A completion the kernel holds on its overflow list (slice 4).** The
  control test's server made no progress above exactly 128 connections
  while burning three cores. The ring's depth is 64, so its completion queue
  held 128 entries; under `IORING_FEAT_NODROP` the 129th completion went to
  the kernel's overflow list, which only an `io_uring_enter` moves into the
  queue, and no submission was coming because every callee was parked on one
  of those completions; the ring's descriptor stays readable for exactly that
  reason, so every park returned at once and the reaper, reading only the
  mapped queue, reaped nothing. `linux_io_uring.c` now maps the submission
  ring's flags word, treats `IORING_SQ_CQ_OVERFLOW` as a non-empty queue in
  the park, and enters the kernel for no minimum inside the non-waiting
  progress pass whenever the flag is raised; the completion queue is sized by
  the caller through `IORING_SETUP_CQSIZE`, 2048 for the bridge so 1024
  connections never reach that path, and 16 in the native adapter probe,
  whose new case stages twenty-four reads against it and reaps them without
  a wait.
- **The pool fiber's emergency stack (`4a21668`, `2d455e5`).** The io-hosts
  overflow proof passed on one commit and segfaulted on the next with the same
  runtime bytes: Windows keeps a stack guarantee for the calling thread or
  fiber, and a fiber takes it only when set from inside that fiber, so a pool
  fiber overflowed with nothing under the handler but what was left of the
  guard page. Every pool fiber's first frame now arms the floor's guarantee.
  The first fix hit LNK1227 on the real host, two weak defaults for one
  symbol, and moving the weak answer alone failed the mingw proxy the other
  way, since GNU ld satisfies a PE weak default only for references from its
  own unit; the floor's attach is now reached through the primitive layer,
  one weak answer per link. The io-hosts step runs each configuration five
  times, because one draw is not evidence against an intermittent defect.
- **A bounded spin before the idle park (`6311482`, `436222c`).** The Windows
  qualification bench missed its bar at 1.06 against 0.95 on a four-vCPU VM
  where a park is a completion-port sleep and both wakes on the loop's
  critical path are kernel round trips. `wf_sched_idle_step` now repeats its
  looks for 256 pause rounds and 16 yield rounds after the drain and the last
  look, inside the capture-to-park window, so §6's lost-wake argument is
  unchanged; the sweep, its placement result (a spin in front of the drain
  delays every completion by its own length) and the Windows judge's table
  (mixed-full 0.6920 against 0.95) are in
  `research/experiments/park-on-miss-measurements/README.md`. The Windows VM
  moves between runners by more than the bar's margin, and the record says so.
- **The core reports its counters (`1ad703b`, `493a5b1`).** `WF_SCHED_REPORT=1`
  makes the entry format the summed counters as one line, printed by the
  grant observer after the grant line. The macOS sampling case that read "the
  pool started" from a steal then said what the threads did:
  `threads=3 workers_started=2 parks=0 steals=0 inline_runs=63`, a program
  that ran to its end before either worker was scheduled on a saturated
  three-core runner. The case now reads the property it states from the
  started-worker count, and the existential observation that the default
  build is granted lanes stays in the `WF_WORKERS=4` case.

## 6. The control benchmark (slice 4)

Three servers on one contract, measured by one load generator over one
loopback, in `research/experiments/io-completion-bench/`: `uring_echo.c`, the
io_uring reference on the raw ABI with multishot accept, multishot receive
into a registered buffer ring, one ring and one `SO_REUSEPORT` listener per
thread, the echo sent out of the kernel-filled buffer with one `sendmsg` over
the queued buffers, and single-issuer deferred task running; `epoll_echo.c`,
one edge-triggered epoll and listener per thread; and
`programs/tcp_echo_server.wf`, the Whitefoot line, a fixed-trip accept loop
over the connection count the invocation names, one parked callee per
connection. `netload.c` opens every connection, then drives them all at once,
verifies every echoed byte, and reports round trips per second and the
latency distribution; `linux-net-bench.sh` runs warm-up and recorded passes
in alternating order and reports medians, and `io-bench.yml`'s Linux job runs
it after the file tables.

The table, on this host (Linux 6.18, four cores, `ROUNDS=3 WARMUP=1`), with
the Whitefoot line under `WF_STACKS=1100` and otherwise the shipped defaults,
at the batch's last revision, after the three runtime changes the two
isolation series below arrived at:

```text
line                conns   bytes    trips     rt_per_s     p50_us     p99_us   connect_us   vs_uring   vs_epoll
uring.k1                1      64    20000      28086.8       33.0       62.0         87.0       1.00       0.95
epoll.k1                1      64    20000      29613.8       32.0       57.0         79.0       1.05       1.00
wf.k1                   1      64    20000      35468.5        6.0      188.0        122.0       1.26       1.20
uring.k64              64      64     2000     314881.9       60.0     2204.0        497.0       1.00       1.02
epoll.k64              64      64     2000     307524.1       53.0     2834.0        457.0       0.98       1.00
wf.k64                 64      64     2000     232273.0      245.0      714.0        497.0       0.74       0.76
uring.k1024          1024      64      200     349084.5     2408.0     9383.0       5056.0       1.00       1.08
epoll.k1024          1024      64      200     323565.8     1403.0    12823.0       3341.0       0.93       1.00
wf.k1024             1024      64      200     201527.1     4870.0     7192.0       5445.0       0.58       0.62
uring.k64.64k          64   65536      200      60512.3      576.0     4476.0        348.0       1.00       0.80
epoll.k64.64k          64   65536      200      75730.4      305.0     5189.0        494.0       1.25       1.00
wf.k64.64k             64   65536      200      52176.5     1064.0     3572.0        436.0       0.86       0.69

line                  bytes_per_s
uring.k64.64k        3965736271.7
epoll.k64.64k        4963069082.4
wf.k64.64k           3419437103.7
```

What the table says. At one connection the Whitefoot line leads both
references by a quarter, because a receive whose bytes have already arrived
and a send the host accepts at once complete on the submitting thread without
an `io_uring_enter`, and on one connection driven by one client nearly every
operation is one of those (the median of 6 microseconds is that path, and the
99th percentile of 188 the park). At 64 connections it is three quarters of
the references, at 1024 six tenths, and on the 64 KiB payload 0.86 of io_uring
and 0.69 of epoll: no longer flat across the connection counts, but short of
the references by a margin that grows with the number of peers waiting at
once. What that margin is, measured where it could be and read from the
structure where it could not, is the second series below. The reading of the
same revision on the runner (`io-bench.yml`) is the runner's own line, and
it differs from this host's by about the factor the first reading did.

The first reading of the same protocol, at a6b31b5 before either runtime
change, is the point the isolation started from:

```text
line                conns   bytes    trips     rt_per_s     p50_us     p99_us   connect_us   vs_uring   vs_epoll
uring.k1                1      64    20000      28538.2       33.0       63.0         81.0       1.00       0.98
epoll.k1                1      64    20000      29235.4       32.0       62.0         26.0       1.02       1.00
wf.k1                   1      64    20000      15354.3       48.0      234.0        101.0       0.54       0.53
uring.k64              64      64     2000     329876.3       64.0     2070.0        440.0       1.00       1.12
epoll.k64              64      64     2000     294618.1       51.0     3532.0        520.0       0.89       1.00
wf.k64                 64      64     2000      35993.9     1862.0     2792.0        636.0       0.11       0.12
uring.k1024          1024      64      200     343760.3     1481.0    10234.0       5370.0       1.00       1.04
epoll.k1024          1024      64      200     330230.5     1634.0    10224.0       3305.0       0.96       1.00
wf.k1024             1024      64      200      26782.6    38105.0    43023.0       3771.0       0.08       0.08
uring.k64.64k          64   65536      200      59161.2      696.0     4131.0        423.0       1.00       0.79
epoll.k64.64k          64   65536      200      74564.0      324.0     5362.0        501.0       1.26       1.00
wf.k64.64k             64   65536      200      18242.5     3302.0     8699.0        539.0       0.31       0.24

line                  bytes_per_s
uring.k64.64k        3877187281.9
epoll.k64.64k        4886625315.3
wf.k64.64k           1195542174.9
```

What that table said. At one connection the Whitefoot server answered a round
trip in 48 microseconds against the references' 33: every receive and every
send is a submission, a park of the callee's stack, and a wake, and the
difference is about what one park-and-wake costs on this host. At 64 and 1024
connections the references reach 330 to 344 thousand round trips a second
and the Whitefoot line stays at 27 to 36 thousand whatever the connection
count, about 14 microseconds per operation, which is the mark of one serial
resource rather than of the number of peers. What is serial in the runtime,
read from the code and not yet measured apart: the whole pool's operations go
through one ring under one submission lock and one completion lock, and each
completion wakes a parked stack through the runtime's one eventfd. The
references have one ring or one epoll per thread and no lock anywhere. The
64 KiB line is a third of the io_uring reference and a quarter of epoll's,
which is the same structure plus a copy through the program's own scratch.
Two results about the references themselves: epoll leads io_uring by 1.26 on
the 64 KiB payload, because one large `recv` and one `send` in user space beat
a completion-per-arrival pipeline for bulk transfer, and the two are level at
a single connection. `--sqpoll` is admitted on this host and loses by a third
with a poll thread per ring on four cores, so it is off.

The runner's own reading, `io-bench.yml`'s Linux job on a6b31b5 (ubuntu-24.04,
four cores, kernel 6.8, `ROUNDS=3 WARMUP=1`), where both references are
slower than here and the Whitefoot line faster, so the ratios differ from this
host's by about a factor of two:

```text
line                conns   bytes    trips     rt_per_s     p50_us     p99_us   connect_us   vs_uring   vs_epoll
wf.k1                   1      64    20000      16179.3       60.0      116.0         89.0       0.80       0.71
uring.k64              64      64     2000     186447.5      135.0     2717.0        796.0       1.00       1.05
epoll.k64              64      64     2000     177846.4       89.0     4246.0       1021.0       0.95       1.00
wf.k64                 64      64     2000      50283.0     1280.0     2080.0       1198.0       0.27       0.28
uring.k1024          1024      64      200     193084.2     5090.0     9698.0       9489.0       1.00       1.03
epoll.k1024          1024      64      200     187009.6     5333.0     9886.0       8694.0       0.97       1.00
wf.k1024             1024      64      200      50056.8    20328.0    25674.0       9987.0       0.26       0.27
uring.k64.64k          64   65536      200      23527.3     2313.0     6528.0       1147.0       1.00       0.64
epoll.k64.64k          64   65536      200      36791.7     1032.0     5885.0        552.0       1.56       1.00
wf.k64.64k             64   65536      200      19670.4     2556.0     5420.0         34.0       0.84       0.53
```

The shape was the same on both hosts: the Whitefoot line flat across the
connection counts while the references scale, and the ratio at one connection
the cost of one park-and-wake per operation on that host. The owner asked for
the line to lead or for the reason and the data it cannot, and to find that
reason one variable at a time.

### Isolating the serial resource, one variable at a time

Every row below is the same server, hand-linked with the grant observer so
the core's counters print at exit, at 64 connections and 2000 round trips of
64 bytes, three trials per variable, on this host; the reference io_uring
line is 330 thousand round trips a second here. The counters for the shipped
runtime said what the threads did: `parks=256132 resumes=256132 steals=64
inline_runs=0` for 256 thousand operations, one park and one resume per
receive and per send, and `strace -c` counted 720 thousand `futex` calls and
178 thousand `io_uring_enter` calls per run, 5.6 and 1.4 per round trip.

| variable | rt/s (three trials) | what it says |
|---|---:|---|
| shipped runtime | 34.0k, 35.0k, 37.2k | the baseline |
| WF_WORKERS 2 / 4 / 8 | 35.0k / 36.0k / same | flat: a serial resource, not the core count |
| E1: the progress pass reaps 64 completions instead of 1 | 69.8k, 69.5k, 69.4k | the reap was the serial resource; futex calls fall to 19 thousand a run |
| E2: read the staged count without the submission lock before kicking | 69.1k, 69.6k, 70.5k | nothing; reverted |
| E3: reap in batches and publish after dropping the completion lock | 66.4k, 68.5k, 68.8k | nothing; reverted |
| E4: idle spin 0 rounds, and 4096 rounds with no yields | 68.3k, 69.8k, 67.7k and 69.9k, 68.5k, 68.2k | the spin is not in this path |
| E6: WF_WORKERS 2 / 8 on E1 | 67.7k to 70.7k / 68.6k to 71.3k | still flat: one more serial resource |
| E5a: a send the host accepts at once completes without the ring | 208.5k, 207.3k, 197.0k | half the parks gone (127 thousand); three times the rate |
| E5b: the same attempt for a receive whose bytes have arrived | 215.5k, 208.1k, 204.8k | parks 107 to 115 thousand; kept, it is the same rule |
| E7: reap budget 1024 | 214.6k, 202.4k, 217.1k | the same as 64 |

What E1 and E5 are. The progress pass a scheduler thread makes on every idle
turn reaped exactly one completion, taking the submission lock to kick and
the completion lock to read it; with four threads doing that for 64
connections the two mutexes were the convoy the futex count showed. The
budget is `WF_BRIDGE_REAP_BUDGET`, 64, in `bridge.c`. E5 is the rule the
bounded adapter already applies to a positioned read the submitting thread
would run itself, applied to a socket transfer: `wf_file_transfer_now` in
the platform leaf asks the host once with `MSG_DONTWAIT`, and an answer that
is the operation's own outcome, bytes moved, the peer's end, or a definite
refusal, completes the record where it was submitted, with no ring, no park
and no wake; only the answer that the host would have to wait leaves the
record for the engine. A send on a loopback whose window has room is always
that first case, so the send half of every round trip stopped parking. The
Windows leaf answers that the host would wait for every transfer, so nothing
moves there.

### The second series: where the remaining margin is

The same method on the runtime after E1 and E5, at 64 connections, three
trials per variable, with the ring's own counters printed beside the core's
(`wf__bridge_report`, printed by the grant observer after the `sched:` line).
The counters for the changed runtime: `parks=112130`, `submissions=112656
submission_enters=4319 completions=112656 kernel_waits=31 host_wake_writes=100
inline=143473`. Every send and one receive in eight complete at once; the
other 112 thousand receives go to the ring, 26 to an `io_uring_enter`; and a
thread slept in the kernel 31 times in a run, so the threads are never
waiting for a completion the kernel has not posted.

| variable | rt/s (three trials) | what it says |
|---|---:|---|
| the runtime at e134360 | 199.6k to 225.9k over three runs of three | the baseline of this series |
| E8: a progress pass on every round of the idle spin, or every sixteenth | 206.2k to 213.4k | nothing: the spin holds no completion |
| E9: `io_uring_enter` at every submission | 107.2k, 110.6k, 118.1k | half the rate: the deferred doorbell is right, one syscall per operation is not affordable |
| E10: the enter outside the submission lock | 200.8k, 207.9k, 216.7k | nothing: the lock is not what the other threads wait on |
| E12: `IORING_SETUP_COOP_TASKRUN` | 241.3k, 244.4k, 251.5k | kept: a completion the kernel finishes for a thread is posted at that thread's next syscall, not by an interrupt |
| E15: kick at submission once eight entries are staged, on E12 | 168.6k, 177.8k, 188.2k | worse: four times the enters |
| E16: `IORING_RECVSEND_POLL_FIRST` on a receive the host has just refused, on E12 | 207.5k, 218.8k, 237.7k | nothing measurable |
| E17: a progress pass before every ready stack when the queue holds a completion, on E12 | 228.8k to 245.1k | nothing: reaping sooner during the busy phase changes no rate |
| E12 at WF_WORKERS 2 / 4 / 8 | 202.9k to 211.6k / 234.3k to 251.5k / 218.5k to 235.4k | two threads reach 85 percent of four, eight are no better: mostly serial |

Where the time goes, measured. Timing the two ring passes under a
compile-time counter (not shipped) on E12 at 64 connections:

```text
timing: kick_ns=119016584 kick_entries=121848 reap_ns=21305268
ring: submissions=121848 submission_enters=4268 ... inline=134281
```

The kick, the `io_uring_enter` that hands staged receives to the kernel,
costs 0.98 microseconds an entry and 28 microseconds a call, 119
milliseconds of a run whose exchange takes about 540: 22 percent of the wall
time, serialized under one lock on one ring, and the kernel does the same
work whichever thread rings the bell. The reap is 21 milliseconds. The CPU
accounting of the servers over one run of 128 thousand round trips at 64
connections, from the shell's `time`:

```text
server           wall    user    sys    rt/s
uring_echo      0.72    0.03   0.69   325k
epoll_echo      0.79    0.05   0.75   280k
wf_echo         0.90    0.17   1.00   223k
netload (against epoll)   0.51 wall, 0.06 user, 0.69 sys
netload (against wf)      0.66 wall, 0.08 user, 0.76 sys
```

The Whitefoot server spends 7.8 microseconds of system time and 1.3 of user
time per round trip against the references' 5.5 and 0.3, on 1.3 cores of the
four; nothing is saturated, the client's four threads included, and pinning
the server to two cores and the client to the other two changes nothing
(316k, 198k, 285k). The line is latency-bound: at 64 connections a round
trip takes 250 microseconds at the median against 62, and 64 divided by 250
microseconds is the rate. That latency is the batch the pipeline moves in:
a thread reaches the ring only when the ready list is empty, so a staged
receive waits for the current batch of stacks to run, is kicked with 26
others in one enter that takes 28 microseconds, and its completion is reaped
at the next pass; E17 tried to break the batch and moved nothing, because
the kick is the same serial work wherever it is placed. Two threads reach 85
percent of four and eight reach less, which is what one serial resource that
costs a quarter of the wall time does.

The connection count, on the runtime at e134360 with the trips scaled to keep
128 thousand round trips: 64 gives 219k to 229k, 128 gives 208k, 256 gives
198k, 512 gives 176k, and 1024 gives 161k. The rate falls with the peers in
flight because the batches grow with them, and each batch is serialized in
the kick.

What the reference does instead, and why it is structural. `uring_echo`
never submits a receive: one multishot receive per connection is armed at
accept and the kernel posts a completion per arrival with no `io_uring_enter`
behind it; each thread has its own ring with no lock, so the four kicks of
the sends run in parallel; and single-issuer deferred task running posts the
completions inside the thread's own enter. The Whitefoot runtime has one
ring for the pool, because the scheduler design's records are one operation,
one completion, one owning frame, and a reaper on any thread publishes any
record: a ring per thread needs the record to name its ring and the
reaper to walk several, and a receive that stays armed across arrivals is a
record that completes more than once, which the emitter's submit-then-join
shape, the core's one park per record, and the bridge's route field all
assume away. Those are the changes the next version makes for this line,
with the data above as the reason: the serial kick is a quarter of the wall
time and parallelizes only with a ring per thread; the receive that must
wait is the whole of what is left after E5, and only a receive armed once
per connection removes its submission.

Two things this series settled that are not runtime defects. `WF_WORKERS=1`
is the opt-out: the loop is not staged, the program serves one connection to
its end before it accepts the next, and a client that drives every
connection at once waits on the first, as the sequential program says. And a
server whose port is still in `TIME_WAIT` from the previous trial refuses
its bind; the trials draw ports from a wider range.

### The third series: the ring is not where the time is

The owner's answer to the second series was to keep going in this PR, so
the structural candidates it named were built and measured one at a time,
on the same 64-connection echo with the observer's counters, then on the
protocol. None of them is kept; what they measured is the result.

| variable | rt/s at 64 connections (three trials) | what it says |
|---|---:|---|
| the runtime at d016afb | 234.3k to 251.5k | the baseline of this series |
| E18: one ring per scheduler thread, own ring kicked and reaped first, the rest reaped by every pass | 197k to 267k at four workers, 228k to 235k at two, 219k to 287k at eight | enters triple, to 13 thousand, because each ring kicks a smaller batch; no gain at four threads on four cores |
| E20: `IORING_SETUP_SQPOLL`, idle 1 ms and 100 ms | 183k to 190k; 181k to 228k | the poll thread takes a core the client needs; enters fall to one and the rate falls with them |
| E19: one multishot `IORING_OP_POLL_ADD` per receive half, armed by the first receive that waits, the reaper moving the bytes with one `recv`, a `POLL_REMOVE` rung at the half's release | 188k to 226k | receives submit nothing and enters fall from 4 thousand to 120; the rate does not move |
| E26: the idle spin once per idle period, later turns park at once | 182k to 186k; 64k at 4 connections | the spin is not the cost |
| E25: a publisher inside its own progress pass defers the wake it would send, and sends one only when it published more than it will pop | 189k to 216k; 60k at 4 connections | the wake is not the cost |
| E29: `EPOLLEXCLUSIVE` on the ring descriptor in the shared epoll | 254k to 268k before the host restarted; no difference on the protocol after it | one parked thread wakes per completion instead of all; within the host's noise |

The stage dwell of one waiting receive, timed under a compile-time counter
(not shipped) at 64 connections on the runtime of d016afb: 70 microseconds
from staging to the kick, 305 from the kick to the reap, 95 from the
publication to the stack running again, about 470 in all, against a 60
microsecond client turnaround. Every stage is a queue: the receive waits for
the current batch of stacks to drain before anyone reaches the ring, the
completion waits for the next pass, the resumed stack waits behind the batch
in the ready list. Taking the kick out (E19), moving it to a kernel thread
(E20) or splitting it four ways (E18) left the rate where it was: the batch
cycle, not the kick, is the period, and the second series' "22 percent of
the wall time" was serial work that the cycle hid rather than paid for.

The two readings of E19 on the protocol, on the host after its restart, say
what the armed receive is worth. Skipping the host attempt when the half's
poll is armed and has reported nothing (fewer syscalls, more parks) reads
0.94 at one connection against the baseline's 1.26, 0.72 against 0.76 at
64, 0.75 against 0.64 at 1024 and 0.96 against 0.82 on 64 KiB; always
attempting first reads 1.30, 0.77, 0.63 and 0.84, the baseline within noise.
A four-hundred-line lock-free slot protocol that trades one connection for a
thousand is not kept; the design is in this record if the thousand-connection
line becomes the target.

What `perf` says. With `linux-tools-generic` installed on the development
host (perf 6.8 sampling `cpu-clock` at 4 kHz), the Whitefoot server at 64
connections spends 20 percent of its samples in `_raw_spin_unlock_irqrestore`
and 9 in `finish_task_switch`; the reference spends 14 and 14. Reading the
callers, 17.8 of the 20 are under the program's own `send`: the loopback
delivery wakes the client's thread (`tcp_data_ready`, `sock_def_readable`,
`__wake_up_sync_key`) and both servers pay it in full. The runtime's own
symbols at 64 connections are `pthread_mutex_lock` at 2.8 percent, the
progress pass, the completion and the submit at about one each; the
kernel's TCP path is the rest. At 4 connections the picture is different:
`finish_task_switch` is 21 percent, `pthread_mutex_lock` 8, the steal scan
`wf_sched_find` 3.7 and `wf_prim_pause` 3.2, and the Whitefoot line runs at
65 to 70 thousand round trips a second against the reference's 294
thousand, on 4.5 CPU-seconds against 0.76: four scheduler threads chasing
four connections through one ready list and one lock, switching context once
or twice per round trip, where the reference's four threads each own one
connection and switch once, on the arrival of their own data. That shape, a
connection that stays on the thread that reaped it, a per-thread ready list,
a steal only from an idle thread, is what the profile points at, and it is a
core change the enumerator has to cover, not a ring change: the next
version's work, with this as its reason.

Two host states. The development host restarted during this series, and
after the restart the same binary measures in two states at 64 connections,
180 to 195 thousand and 255 to 275 thousand, baseline and candidates alike
when run alternately, and the references move by a fifth between protocol
runs. Only ratios within one run are read above, and the runner's own table
on this revision is the check. One defect the series found in the tests:
`a_peer_that_resets_reaches_the_program_as_its_own_outcome_on_both_routes`
closed the peer as soon as its payload was written, so on the macOS runner
the close raced ahead of the echo, sent a graceful end instead of a reset,
and the program read the direction's end and exited zero; the peer now
peeks one echoed byte before it closes, so the queue holds data at the close
on every host.

The ratio is the batch's result, and the plan carries what it points at:
the connection that stays on its reaping thread, the per-thread ready list
and the steal from idleness are scheduler structure the design did not have
to decide for files, where every operation of a program went through one
thread's submissions; they are the next performance work on this line, in a
new PR by the owner's decision.

## 7. Gates

Run on this host (Linux, root) at the revisions this record spans, in a clean
worktree for every landed slice.

- `make -C compiler format lint`: PASS.
- `make -C compiler completion-test`, `completion-sanitize`,
  `completion-tsan`, `completion-windows-cross`, `completion-windows-wine`:
  PASS, the Wine run including the loopback round trip on both routes.
- `cargo test --profile gate --lib` (1497), `--bins` and `--test programs`
  (the three root-permission cases excepted, since `chmod 000` is not a
  barrier to root): pass.
- `make conformance-run`: Pass=531, Xfail=1, Skip=1. `make snapshot-run`:
  Pass=491, Flip=0. `make repository-invariants`: PASS.
- `gate.yml` and `io-hosts.yml` green at `493a5b1` on every job, the real
  Windows host included; `io-bench.yml`'s Windows job green again at
  `c966ee0` after the link fix.
- The same battery at the two isolation revisions, e134360 and the one
  after it, in the clean worktree: every item above passes with the same
  counts.

## 8. What is left open

- **The server loop whose trip count is not fixed.** [PAR-3] stages a
  fixed-trip loop and a per-file loop over names; a loop stopped by data a
  remainder produced is the one shape the rule itself does not stage, so a
  server that accepts until told to stop is language work (§6). It is the
  next PR's, by the owner's decision of 2026-09-06.
- **The readiness-driven adapter for ring-less hosts.** On Darwin, and on
  Windows for the accept, a socket wait is a blocking host call on some
  thread, so the adapter route's socket concurrency is the helper cap. The
  control test shows the consequence: `wf__completion_window` caps a staged
  loop's window at `WF_BRIDGE_MAX_HELPERS` when no ring is the engine, so on
  the adapter route the echo server keeps eight accepts in flight, the eight
  connections it serves stay open until the generator closes everything at
  the end, the ninth accept is never issued, and the run holds until the
  generator gives up (traced: eight `accept4` calls in two minutes, every
  helper inside a `recvfrom`). The same holds in the sequential world at a
  window of one. Four peers pass because four is under eight. A poll, kqueue
  or WSAPoll over the queued descriptors inside the park is the proper engine
  and is recorded in NETWORK.md §5; a new PR.
- **The Whitefoot line's two steady states.** Before E1 the same
  configuration measured twice at 64 connections gave 95 thousand and 36
  thousand round trips a second, and the ring-required setting 81 and 35
  thousand, with nothing changed between the runs. Neither series after E1
  reached a second state in some forty observed runs; after the development
  host restarted during the third series the two states were back, 180 to
  195 and 255 to 275 thousand for one binary, and the references themselves
  moved by a fifth between protocol runs, so the states are the host's and
  only ratios within one run are read.
- **The connection that stays on its thread.** The third series of §6 built
  and measured the ring per thread, the kernel submission thread and the
  armed receive and found the rate unmoved by all three; the profile puts the
  remaining cost in the scheduler's shape, one ready list and one lock for
  every connection, with the context switches that follow. A per-thread
  ready list with a steal from idleness is the next version's work, a core
  change the enumerator covers, with that data as its reason.
- **§12 item 1 of PARK-ON-MISS.md, the compute-miss regression**, narrowed to
  11 and 17 percent by the spin and still the owner's decision; the colouring
  design is sequenced after this batch; a new PR.
- **8192 connections in flight** is outside the shapes and the stack pool the
  control test could reach; the table stops at 1024.

## Approval classes

Specification (v0.46 over v0.45, recorded in `governance/APPROVALS.md` at
slice 1), conformance (eleven cases added, thirty-one respelled, five verdicts
moved from `unsupported` to `accept` when their subject became supported),
compiler, runtime, documentation.
