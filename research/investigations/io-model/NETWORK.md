# Streams and TCP under T4

Status: PROPOSAL, 2026-09-05. Owner decisions listed at the end. The
specification is `spec/kernel-spec.md`; nothing here is language until an
amendment lands there.

## 1. Why the network first

Concurrency's real demand is the network: reads are fast, writes slower, a
peer across a socket slower still, and the programs that hold thousands of
operations in flight hold them on connections. The owner ruled on 2026-09-05
that the first API that waits is therefore the network rather than standard
input alone, because a loopback gives the scheduler a real wait with a
controllable peer, and because a socket is where the park-on-miss design
(`PARK-ON-MISS.md`) is meant to pay for itself.

Standard input comes along as the first instance of the same stream design:
one readable byte stream, one operation, the same runtime path.

## 2. The T4 test, applied

Constitution T4: every finite resource a system operation consumes is an
owned value in the operation's signature, drawn from a factory whose capacity
is fixed at start and never larger than what the target provides; an
operation holding its resource cannot fail for want of it; what the mapping
cannot cover is honest target exhaustion, and that only is the typed error.

The finite resources a TCP program consumes, and where each sits on the API:

| resource | owner on the API | exhaustion outcome |
|---|---|---|
| a native descriptor, for a listener and for every connection | one `FilePermit` credit of the entry's `FileFactory`, consumed by `tcp_listen`, `tcp_accept` and `tcp_connect`, handed back on failure, returned by the explicit closes | `reserve_file` says `ResourceExhausted` in source order; the opens never |
| a local port | the `SocketAddress` value the program binds; two binds of one port are the program's own source-order conflict | `AddressInUse` is the host's answer to the second bind, the program's own outcome |
| an ephemeral port for `tcp_connect` | none on the API: the target's pool, outside the program | `AddressUnavailable`, honest target exhaustion |
| the accept queue | none on the API: a kernel queue the peer fills; the program observes it only through `tcp_accept`'s outcomes | none; a full backlog refuses the peer, never the program |
| socket buffers | none on the API: `send` and `receive` report partial progress exactly as `write_once` and `read_at` do | none |

Overlap cannot invent an outcome the sequential program never produces: two
accepts on one listener each hold their own permit and take whichever
connection the kernel hands each; two connects each hold their own permit;
send and receive on one connection are sequenced by the exclusive loan. No
scheduler ledger, no award, no retry.

## 3. Types

Three opaque nominal types beside the existing ten, and one renamed:

- `InputStream`: a readable byte stream with an implicit position. The entry
  input `command.stdin` at ordinal 5 supplies one as `own InputStream`, only
  when the entry selects it.
- `OutputStream`: the existing `Output`, renamed; `command.stdout` and
  `command.stderr` supply it as today.
- `SocketAddress`: an immutable value, an IPv4 or IPv6 address and a port,
  constructed by `socket_address_v4(a, b, c, d, port)` and
  `socket_address_v6(...)` with no host call. No name resolution in this
  slice.
- `TcpListener`: a state resource with one live state, one credit.
- `TcpConnection`: a state resource with one live state, one credit. In this
  slice a connection is one owner and `receive` and `send` both borrow it
  `&uniq`, so the two directions of one connection are sequenced by the loan
  while distinct connections overlap freely. Full duplex inside one
  connection is a later `split` into two half owners with their own close;
  it is not invented here.

## 4. Operations

```
fn read_next(input: &uniq InputStream, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own ReadOutcome reads(input, destination), writes(input, destination);
fn write_once(output: &uniq OutputStream, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output);

fn socket_address_v4(a: own u8, b: own u8, c: own u8, d: own u8, port: own u16) -> result: own SocketAddress pure;

fn tcp_listen(permit: own FilePermit, address: &SocketAddress) -> result: own ListenOutcome reads(permit, address), writes(permit);
fn tcp_accept(permit: own FilePermit, listener: &TcpListener) -> result: own AcceptOutcome reads(permit, listener), writes(permit);
fn tcp_connect(permit: own FilePermit, address: &SocketAddress) -> result: own ConnectOutcome reads(permit, address), writes(permit);
fn receive(connection: &uniq TcpConnection, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own ReadOutcome reads(connection, destination), writes(connection, destination);
fn send(connection: &uniq TcpConnection, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(connection, source), writes(connection);
fn close_connection(connection: own TcpConnection) -> result: own FilePermit reads(connection), writes(connection);
fn close_listener(listener: own TcpListener) -> result: own FilePermit reads(listener), writes(listener);

enum ListenOutcome { Listening(listener: TcpListener); ListenFailed(error: IoError, permit: FilePermit); }
enum AcceptOutcome { Accepted(connection: TcpConnection, peer: SocketAddress); AcceptFailed(error: IoError, permit: FilePermit); }
enum ConnectOutcome { Connected(connection: TcpConnection); ConnectFailed(error: IoError, permit: FilePermit); }
```

The forms follow what the specification already has: the open outcomes hand
the permit back on failure exactly as `FileOpenOutcome` does; the explicit
closes return the credit exactly as `close_read` does; derived release closes
and spends the credit exactly as for `ReadFile`; `read_next` and `receive`
carry `read_at`'s range and outcome rules ([SYS-8]: empty range makes no host
transfer, one progress-producing attempt, `ReadBytes(next)` only for
`next > start`, `ReadEnd` with the buffer unchanged); `send` carries
`write_once`'s (`Ok(next)` means the local facility accepted the prefix,
`WriteZero` for a zero-length host write, `BrokenPipe` through the same
signal normalization). Target contract: `socket_address_v4` never-suspends;
everything else may-suspend, terminal, one-shot, with the same milestones
as the opens. `tcp_accept` borrows the listener `&`, so several accepts may
overlap through shared loans of one listener with their own permits, which is
how a server takes connections concurrently. `IoError` already carries
`ConnectionRefused`, `ConnectionReset`, `ConnectionAborted`, `NotConnected`,
`AddressInUse`, `AddressUnavailable`, `TimedOut`, `BrokenPipe`.

## 5. Runtime

One lowering, as everywhere: submit, then join, through the frame's record.
The request kinds are added to `wf_file_request` (listen, accept, connect,
receive, send, stream read) and the routes are:

- Linux: `IORING_OP_ACCEPT`, `IORING_OP_CONNECT`, `IORING_OP_RECV`,
  `IORING_OP_SEND`, and `IORING_OP_READ` at offset -1 for the stream, all on
  the ring, so a wait is a real ring wait for the first time; `listen` and
  `bind` are immediate host calls the adapter runs.
- Windows: `AcceptEx`, `ConnectEx`, `WSARecv`, `WSASend` on the completion
  port; the standard input handle through the adapter (a console handle has
  no overlapped form).
- Darwin: the shared file adapter's helpers make the blocking calls until a
  kqueue route exists; this is the route `WF_IO_NO_NATIVE_RING` runs on Linux
  and is not a second lowering.

Each new kind is one more case in the two host leaves (`file_posix.c`,
`file_windows.c`) and the two rings; nothing in the core or the bridge's
routing changes shape.

## 6. The control test

Loopback is a controlled peer. The test harness (Rust, in `tests/programs`)
plays the other side with `std::net` and can add any delay it wants, so a
Whitefoot server is measured against a known client and a Whitefoot client
against a known server. The R0 delta is measured against a C peer written on
the same ring (`research/experiments/io-completion-bench/` already holds the
C baselines for reads): connections per second and bytes per second at N
connections in flight, and the latency of one request under load. The
program shapes the checker admits today bound N: a fixed-trip loop over
accepts staged under [PAR-3], or an overlap group of accepts; widening those
shapes is the language work this batch exposes, and it is the point of doing
the network now rather than later.

## 7. Slices

1. Amendment v0.46: the rename, `command.stdin` and `read_next`, the three
   types and nine operations of §4, conformance cases, corpus programs.
2. POSIX runtime: adapter route for every kind, Linux ring route for accept,
   connect, receive, send and stream read; loopback tests in `tests/programs`.
3. Windows: the completion-port route; the io-hosts job proves it.
4. The control benchmark against the C peer, in io-completion-bench.
5. Batch record.

## 8. Decisions for the owner

1. One `TcpConnection` owner with `&uniq` receive and send in this slice
   (full duplex within a connection later, by an explicit split), or two half
   owners from the start.
2. TCP only, address literals only, no UDP and no name resolution in this
   batch.
3. Permits for listeners and connections come from the same `FileFactory`,
   one credit each, because the target's limit is one descriptor table.
4. The names: `InputStream`, `OutputStream`, `SocketAddress`, `TcpListener`,
   `TcpConnection`, `read_next`, `receive`, `send`, `tcp_listen`,
   `tcp_accept`, `tcp_connect`, `close_connection`, `close_listener`.
5. The backlog is target-fixed (not a source argument) in this slice.
