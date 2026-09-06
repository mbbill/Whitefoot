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
at the batch's last revision, after the two runtime changes the isolation
below arrived at:

```text
line                conns   bytes    trips     rt_per_s     p50_us     p99_us   connect_us   vs_uring   vs_epoll
uring.k1                1      64    20000      29233.9       32.0       62.0         89.0       1.00       0.99
epoll.k1                1      64    20000      29520.5       32.0       57.0         62.0       1.01       1.00
wf.k1                   1      64    20000      31653.1        6.0      186.0         73.0       1.08       1.07
uring.k64              64      64     2000     318424.6       62.0     2403.0        462.0       1.00       0.98
epoll.k64              64      64     2000     325926.7       50.0     2772.0        527.0       1.02       1.00
wf.k64                 64      64     2000     218075.9      250.0      841.0        608.0       0.68       0.67
uring.k1024          1024      64      200     346920.2     1760.0     8738.0       4017.0       1.00       1.07
epoll.k1024          1024      64      200     324860.8     1750.0    10554.0       5385.0       0.94       1.00
wf.k1024             1024      64      200     166786.8     5966.0    11299.0       5608.0       0.48       0.51
uring.k64.64k          64   65536      200      62160.7      661.0     4714.0        434.0       1.00       0.88
epoll.k64.64k          64   65536      200      70539.5      329.0     4561.0        467.0       1.13       1.00
wf.k64.64k             64   65536      200      48220.6     1099.0     3575.0        574.0       0.78       0.68

line                  bytes_per_s
uring.k64.64k        4073761479.3
epoll.k64.64k        4622877114.9
wf.k64.64k           3160188417.8
```

What the table says. At one connection the Whitefoot line leads both
references, because a receive whose bytes have already arrived and a send the
host accepts at once complete on the submitting thread without an
`io_uring_enter`, and on one connection driven by one client nearly every
operation is one of those (the median of 6 microseconds is that path, and the
99th percentile of 186 the park). At 64 connections it is two thirds of the
references, at 1024 half, and on the 64 KiB payload three quarters of io_uring
and two thirds of epoll: no longer flat across the connection counts, but
still short of the references by a margin that grows with the number of peers
waiting at once. What that margin is, read from the structure and not yet
measured apart, is under "what is left" below.

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

What is left, read from the structure and not yet measured apart. After E5
every operation that the host can answer at once costs no ring, and what
remains on the ring is exactly the receive whose peer has not sent yet: 107
to 115 thousand parks in 256 thousand operations at 64 connections, one per
receive that had to wait. Each of those is a submission under the ring's one
submission lock, a park of the callee's stack, an `io_uring_enter` by some
thread's progress pass, a reap under the ring's one completion lock, and a
publication through the core's lock that wakes the parked stack; the
io_uring reference arms one multishot receive per connection and never waits
per connection, on one ring per thread with no lock. The remaining gap grows
with the number of peers waiting at once, from 0.68 at 64 to 0.48 at 1024,
which is what a shared ring and a wake per completion would do; measuring
those apart is the next work, and the reason it is not in this PR is that a
ring per thread and a receive that stays armed change what a record is (one
record, one operation, one completion, one owner frame), which the emitter,
the core and the bridge all assume.

The ratio is the batch's result, and the plan carries what it points at: the
ring per thread, the locks, and the wake per completion are runtime structure
the scheduler design did not have to decide for files, where every operation
of a program went through one thread's submissions; they are the next
performance work on this line, in a new PR by the owner's decision.

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
- **The Whitefoot line's two steady states.** The same configuration measured
  twice at 64 connections gave 95 thousand and 36 thousand round trips a
  second, and the ring-required setting 81 and 35 thousand, with nothing
  changed between the runs; this is the first variable to isolate before any
  of the structure above is changed, and it is why the runner's ratios and
  this host's differ by about two.
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
