# Streams and TCP under T4

Status: PROPOSAL, revised 2026-09-05 after the owner's first round. Owner
decisions listed at the end. The specification is `spec/kernel-spec.md`;
nothing here is language until an amendment lands there.

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
| a native descriptor, for a listener and for every connection | one credit of the entry's descriptor factory (today's `FileFactory`, renamed below), consumed by listen, accept and connect, handed back on failure, returned by the explicit closes | the reserve says `ResourceExhausted` in source order; the opens never |
| a local port | the `SocketAddress` value the program binds; two binds of one port are the program's own source-order conflict | `AddressInUse` is the host's answer to the second bind, the program's own outcome |
| an ephemeral port for connect | none on the API: the target's pool, outside the program | `AddressUnavailable`, honest target exhaustion |
| the accept queue | none on the API: a kernel queue the peer fills; the program observes it only through accept's outcomes | none; a full backlog refuses the peer, never the program |
| socket buffers | none on the API: send and receive report partial progress exactly as `write_once` and `read_at` do | none |

Overlap cannot invent an outcome the sequential program never produces: two
accepts on one listener each hold their own permit and take whichever
connection the kernel hands each; two connects each hold their own permit;
the two directions of one connection are two owners and overlap under
[PAR-1] like any two places. No scheduler ledger, no award, no retry.

## 3. Types

A connection is two owners from the start: the owner asked whether full
duplex could be added later without an architectural change, and the honest
answer is that a later `split` would need a second `receive` and `send`
signature over half types, because a system operation has one signature. So
the halves are the design, not a later refinement: `accept` and `connect`
return a receive half and a send half, each an ordinary owner with its own
loan, and full duplex inside one connection is two places overlapping under
[PAR-1] with nothing added. The descriptor credit belongs to the pair: the
explicit close takes both halves and returns the permit; derived release of
one half is the host's half-close of that direction (`shutdown`), and the
release of the second half closes the descriptor and spends the credit,
exactly as derived release of a `ReadFile` does. The runtime keeps the
"both halves gone" fact for itself; the checker sees only the pair close,
which is the relation T4 asks for.

Types beside the existing ones, and two renamed:

- `InputStream`: a readable byte stream with an implicit position. The entry
  input `command.stdin` at ordinal 5 supplies one as `own InputStream`, only
  when the entry selects it.
- `OutputStream`: the existing `Output`, renamed; `command.stdout` and
  `command.stderr` supply it as today.
- `SocketAddress`: an immutable value, an IPv4 or IPv6 address and a port,
  constructed by `socket_address_v4(a, b, c, d, port)` and
  `socket_address_v6(...)` with no host call. No name resolution in this
  batch.
- `TcpListener`: a state resource with one live state, one credit.
- `TcpConnection`: a system-declared struct of two fields, `receive: TcpReceive` and `send: TcpSend`; each half is a state
  resource with one live state; the pair holds one credit.
- The descriptor factory and its permit, renamed from `FileFactory` and
  `FilePermit` because sockets draw from the same table (decision 3 below).

## 4. Operations

Written with the working names `HandleFactory`, `HandlePermit`,
`command.handles` and `reserve_handle`; the owner picks the spelling.

```
fn read_next(input: &uniq InputStream, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own ReadOutcome reads(input, destination), writes(input, destination);
fn write_once(output: &uniq OutputStream, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output);

fn socket_address_v4(a: own u8, b: own u8, c: own u8, d: own u8, port: own u16) -> result: own SocketAddress pure;

fn tcp_listen(permit: own HandlePermit, address: &SocketAddress) -> result: own ListenOutcome reads(permit, address), writes(permit);
fn tcp_accept(permit: own HandlePermit, listener: &TcpListener) -> result: own AcceptOutcome reads(permit, listener), writes(permit);
fn tcp_connect(permit: own HandlePermit, address: &SocketAddress) -> result: own ConnectOutcome reads(permit, address), writes(permit);
fn receive_next(receive: &uniq TcpReceive, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own ReadOutcome reads(receive, destination), writes(receive, destination);
fn send_once(send: &uniq TcpSend, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(send, source), writes(send);
fn close_connection(connection: own TcpConnection) -> result: own HandlePermit reads(connection), writes(connection);
fn close_listener(listener: own TcpListener) -> result: own HandlePermit reads(listener), writes(listener);

enum ListenOutcome { Listening(listener: TcpListener); ListenFailed(error: IoError, permit: HandlePermit); }
enum AcceptOutcome { Accepted(connection: TcpConnection, peer: SocketAddress); AcceptFailed(error: IoError, permit: HandlePermit); }
enum ConnectOutcome { Connected(connection: TcpConnection); ConnectFailed(error: IoError, permit: HandlePermit); }
```

The pairing is on the API, by the owner's decision of 2026-09-05: the
connection is a system-declared struct with two fields,

```
struct TcpConnection { receive: TcpReceive; send: TcpSend; }
```

so `accept` and `connect` return one `TcpConnection`, the program borrows
`&uniq connection.receive` and `&uniq connection.send` as two places (an
effect path names a struct field, [EFF-1]; two exclusive loans on disjoint
fields coexist under [OWN-5]), and the close takes the whole value:

```
fn close_connection(connection: own TcpConnection) -> result: own HandlePermit reads(connection), writes(connection);
enum AcceptOutcome { Accepted(connection: TcpConnection, peer: SocketAddress); AcceptFailed(error: IoError, permit: HandlePermit); }
enum ConnectOutcome { Connected(connection: TcpConnection); ConnectFailed(error: IoError, permit: HandlePermit); }
```

Moving one half out of the record is the language's own partial move,
which kills the whole binding ([OWN] rules: "partial moves kill the whole
binding"), so a broken pair can never reach the explicit close; the moved
half lives on as an ordinary owner, the remainder is derived-released, and
the credit is spent, exactly as derived release spends it for a file. No
`split` or `join` operation exists. A system-declared struct is a new
category of system nominal beside the opaque and enum ones, and the
amendment's META-5 delta declares it.

The forms follow what the specification already has: the open outcomes hand
the permit back on failure exactly as `FileOpenOutcome` does; the explicit
closes return the credit exactly as `close_read` does; derived release
closes and spends the credit exactly as for `ReadFile`; `read_next` and
`receive_next` carry `read_at`'s range and outcome rules ([SYS-8]: empty
range makes no host transfer, one progress-producing attempt,
`ReadBytes(next)` only for `next > start`, `ReadEnd` with the buffer
unchanged); `send_once` carries `write_once`'s (`Ok(next)` means the local
facility accepted the prefix, `WriteZero` for a zero-length host write,
`BrokenPipe` through the same signal normalization). The names follow the
stream pair: `read_next` and `receive_next` advance a position, `write_once`
and `send_once` make one attempt. Target contract: `socket_address_v4`
never-suspends; everything else may-suspend, terminal, one-shot, with the
same milestones as the opens. `tcp_accept` borrows the listener `&`, so
several accepts may overlap through shared loans of one listener with their
own permits, which is how a server takes connections concurrently.
`IoError` already carries `ConnectionRefused`, `ConnectionReset`,
`ConnectionAborted`, `NotConnected`, `AddressInUse`, `AddressUnavailable`,
`TimedOut`, `BrokenPipe`.

The operation names are flat, as every system operation is today
(`open_read`, `close_read`, `read_at`). The owner noted they are not uniform
and may later move into namespaces; that is a specification-wide spelling
change for every operation and is not part of this batch.

## 5. Runtime

One lowering, as everywhere: submit, then join, through the frame's record.
The request kinds are added to `wf_file_request` (listen, accept, connect,
receive, send, stream read, half-close) and the routes are:

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
routing changes shape. The two halves of a connection share one descriptor
in the runtime with a two-count the releases decrement.

## 6. The control test, and the bar

Loopback is a controlled peer. The test harness (Rust, in `tests/programs`)
plays the other side with `std::net` and can add any delay it wants, so a
Whitefoot server is measured against a known client and a Whitefoot client
against a known server.

The owner's bar: the reference is the fastest existing solution, any
language, any library, because the target is first place. On a Linux
loopback that is a hand-written C server on io_uring using what the kernel
offers for exactly this shape: multishot accept, multishot receive into a
provided buffer ring, `SQPOLL`, one ring per core with `SO_REUSEPORT`, no
copies the protocol does not need. That is the shape the fastest published
io_uring servers take, and it is written into
`research/experiments/io-completion-bench/` beside the existing C read
baselines so the numbers are produced by one runner on one host. An epoll
server of the same design is the second reference, since it is what most
deployed servers still are. Kernel-bypass stacks are not a reference on a
loopback: they replace the kernel's TCP and measure something else. The
measures: connections accepted per second, request-response round trips per
second at 1, 64, 1024 and 8192 connections in flight, bytes per second at a
large payload, and the latency distribution of one request under load;
Whitefoot's number is reported as a ratio to the io_uring reference, and the
gap is the batch's result, not something to hide.

The program shapes the checker admits today bound the connections in
flight: a fixed-trip loop over accepts staged under [PAR-3], or an overlap
group of accepts. Widening those shapes to a real server loop is the
language work this batch exposes, and it is the point of doing the network
now rather than later.

## 7. Slices

1. Amendment v0.46: the two renames, `command.stdin` and `read_next`, the
   types and operations of §4, conformance cases, corpus programs.
2. POSIX runtime: adapter route for every kind, Linux ring route for accept,
   connect, receive, send and stream read; loopback tests in `tests/programs`.
3. Windows: the completion-port route; the io-hosts job proves it.
4. The control benchmark against the io_uring and epoll references, in
   io-completion-bench.
5. Batch record.

## 8. Decisions for the owner

Settled on 2026-09-05: the network first, with standard input as the first
stream; `InputStream` and `OutputStream`; TCP only, address literals only,
no UDP and no name resolution; the backlog is target-fixed; full duplex is
designed in from the start as two halves (§3); the reference is the fastest
existing solution regardless of language.

Settled in the second round, 2026-09-05: `HandleFactory`, `HandlePermit`,
`command.handles`, `reserve_handle`, renaming the v0.45 surface across the
conformance corpus and the programs in the same amendment; the connection
is the two-field struct of §4, so the pairing is on the API and no split or
join exists; the operation names of §4; the reference of §6. Nothing is
open; slice 1 starts.
