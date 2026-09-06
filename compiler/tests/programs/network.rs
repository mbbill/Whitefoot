//! The network, end to end over a loopback [SYS-16, SYS-17, SYS-18].
//!
//! Loopback is a controlled peer: the harness plays the other side with
//! `std::net`, so a Whitefoot server is measured against a known client and a
//! Whitefoot client against a known server
//! (`research/investigations/io-model/NETWORK.md` §6).
//!
//! Every case runs on both routes the host has. The shipped default reaches
//! the Linux completion ring, which carries accept, connect, receive and send
//! as ring operations; `WF_IO_NO_NATIVE_RING` runs the same program through
//! the shared file adapter's own `accept4`, `connect`, `recv` and `send`. The
//! two must agree byte for byte and status for status, because the route is an
//! implementation choice and not a language one.
//!
//! The specification declares no operation reporting a listener's own local
//! address ([SYS-17]; NETWORK.md §4 lists the complete operation set), so a
//! server program cannot tell the harness which port it took. The harness
//! therefore picks a free port itself and passes it as an argument, which is
//! the same fact from the other side.
//!
//! Nothing these cases *do* is POSIX: the peer is `std::net`, the port comes
//! from a bound-and-released listener, the half-close is
//! `Shutdown::Write`, and the port argument reaches the program through
//! `support::invocation_argument`, which is text on a family whose arguments
//! are not bytes. What is still POSIX is the harness's own link
//! (`support.rs`, `stage_runtime_units` and `link_module`): it stages the
//! POSIX runtime units and names an absolute `clang`, so these cases build
//! nowhere else yet. That is one bounded piece of harness work and not a
//! property of the cases; the Windows evidence for the same programs is
//! `.github/workflows/io-hosts.yml`'s `completion-windows` job, which
//! compiles `tcp_echo.wf` and `tcp_refused.wf` with the production driver and
//! runs them on both routes against a `System.Net.Sockets` peer.

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::process::Child;
use std::time::{Duration, Instant};

use super::support::{
    CompiledProgram, build_program, compile_and_run, compile_program, compile_program_with_overlap,
    compile_program_without_overlap, emitted_function, program_permission_ledger,
};

/// One port the host is not using, released before the program binds it.
///
/// A listening socket that never accepted leaves no connection in `TIME_WAIT`,
/// so the port is free the moment this drops and the program's own `bind`
/// answers without `SO_REUSEADDR` — which the runtime deliberately does not
/// set, because it would change what a second bind of one port means
/// [SYS-17].
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve a loopback port");
    listener
        .local_addr()
        .expect("the reserved port's address")
        .port()
}

/// Connects to a program that is still starting.
///
/// The program binds its listener some time after the harness spawned it, so
/// the first attempts are refused. This retries for a bounded wall-clock span
/// and fails the case if the program never listened; nothing about the
/// program's own acceptance depends on it.
fn connect_when_ready(port: u16) -> TcpStream {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match TcpStream::connect(address) {
            Ok(stream) => return stream,
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("the program never listened on {port}: {error}"),
        }
    }
}

/// The exit code one finished child reported, with its diagnostics on failure.
fn finished(child: Child) -> (i32, Vec<u8>) {
    let output = child.wait_with_output().expect("wait for compiled program");
    (output.status.code().unwrap_or(-1), output.stdout)
}

fn payload() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10_000);
    for index in 0..10_000_u32 {
        bytes.push(u8::try_from(index % 251).expect("a byte"));
    }
    bytes
}

/// Runs one echo exchange against `tcp_echo.wf` and returns what came back
/// beside the program's own status.
fn echo_exchange(program: &CompiledProgram, native_ring: bool, bytes: &[u8]) -> (Vec<u8>, i32) {
    let port = free_port();
    let text = port.to_string();
    let child = program.spawn_on_route(native_ring, &[text.as_bytes()]);
    let mut stream = connect_when_ready(port);
    stream.write_all(bytes).expect("send the payload");
    // The connection's receiving direction ends where this peer stops sending,
    // and the server's `receive_next` answers `ReadEnd` for exactly that
    // [SYS-8, SYS-18].
    stream.shutdown(Shutdown::Write).expect("stop sending");
    let mut returned = Vec::new();
    stream
        .read_to_end(&mut returned)
        .expect("read the echoed bytes");
    drop(stream);
    let (status, _) = finished(child);
    (returned, status)
}

#[test]
fn ipv4_checksum_uses_one_slice_consumer_for_static_and_runtime_storage() {
    let llvm = compile_program("ipv4_checksum.wf");
    let checksum = emitted_function(&llvm, "ipv4_checksum");
    let main = emitted_function(&llvm, "main");
    // The discharged slice reads emit no bounds branch; the loop invariants
    // establish the address domains before the element addresses form.
    assert!(checksum.contains("getelementptr inbounds i8"));
    assert!(!checksum.contains("call void @free"));
    assert_eq!(main.matches("call i16 @wf_ipv4_checksum").count(), 2);
    // Each explicit validation-failure return owns its ordinary cleanup path;
    // no caller-side copied fact can bypass that cleanup.
    assert!(main.matches("call void @free").count() >= 1);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn every_tcp_operation_lowers_through_the_one_submit_and_join_shape() {
    let llvm = compile_program("tcp_echo.wf");
    // One lowering: each operation is filled into the frame's own record,
    // submitted, and joined where its outcome is needed. No direct arm and no
    // second lowering.
    for submit in [
        "call void @wf__completion_socket_listen_submit",
        "call void @wf__completion_socket_accept_submit",
        "call void @wf__completion_socket_receive_submit",
        "call void @wf__completion_socket_send_submit",
        "call void @wf__completion_socket_shutdown_submit",
    ] {
        assert!(llvm.contains(submit), "missing {submit}");
    }
    // The accept has its own join because it publishes the peer's address
    // beside the descriptor; every other TCP kind retires through the one file
    // join.
    assert!(llvm.contains("call void @wf__completion_socket_accept_join"));
    assert!(llvm.contains("call void @wf__completion_file_join"));
    // The listener's explicit close is the ordinary close every other
    // descriptor-shaped resource takes; the connection's is two half-closes,
    // one per direction, and the runtime's own two-count decides which of them
    // releases the target's object [SYS-18].
    assert!(llvm.contains("call void @wf__completion_file_close_submit"));
    let explicit = llvm
        .split("define private i1 @wf.sys.close_connection.v1(")
        .nth(1)
        .and_then(|tail| tail.split("\n}\n").next())
        .expect("the emitted explicit close of a connection");
    assert_eq!(
        explicit
            .matches("call void @wf.sys.socket.half_close(i32")
            .count(),
        2,
        "the pair's close is exactly one half-close per direction"
    );
    // Compiler-derived release of a direction reaches the very same helper, so
    // a pair released either way makes the same two attempts and the runtime's
    // own two-count decides which of them releases the target's object
    // [SYS-5, SYS-18].
    assert!(
        llvm.matches("call void @wf.sys.socket.half_close(i32")
            .count()
            > 2,
        "derived release of a direction must reach the one half-close"
    );
}

#[test]
fn a_loopback_echo_returns_every_byte_on_both_routes() {
    let llvm = compile_program("tcp_echo.wf");
    let program = build_program(&llvm);
    let bytes = payload();
    for native_ring in [true, false] {
        let (returned, status) = echo_exchange(&program, native_ring, &bytes);
        assert_eq!(status, 0, "native ring: {native_ring}");
        assert_eq!(returned, bytes, "native ring: {native_ring}");
    }
}

#[test]
fn a_peer_that_stops_sending_is_the_receiving_direction_s_end_on_both_routes() {
    let llvm = compile_program("tcp_echo.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        // Nothing at all is sent, so the very first `receive_next` observes
        // the end and the program returns without failure [SYS-8].
        let (returned, status) = echo_exchange(&program, native_ring, &[]);
        assert_eq!(status, 0, "native ring: {native_ring}");
        assert!(returned.is_empty(), "native ring: {native_ring}");
    }
}

#[test]
fn a_peer_that_resets_reaches_the_program_as_its_own_outcome_on_both_routes() {
    let llvm = compile_program("tcp_echo.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        let port = free_port();
        let text = port.to_string();
        let child = program.spawn_on_route(native_ring, &[text.as_bytes()]);
        let mut stream = connect_when_ready(port);
        // Small enough that this peer's own send completes without waiting
        // for anything, and never read back: the program echoes it into this
        // socket's receive queue, and a host closing a connection whose
        // receive queue still holds data sends a reset rather than a graceful
        // end. That is the reset the program then observes. The peek is what
        // makes the queue hold data at the close: a close that raced ahead of
        // the echo would send a graceful end instead, which the program then
        // reads as the direction's end before any reset reaches it, and a
        // receive the submitting thread answers at once made that race real
        // on the macOS runner.
        let bytes = vec![7_u8; 64 * 1024];
        stream.write_all(&bytes).expect("send the payload");
        stream
            .peek(&mut [0_u8; 1])
            .expect("the first echoed byte arrives before the peer closes");
        drop(stream);
        let (status, _) = finished(child);
        // `tcp_echo.wf` reports 20 plus the portable class for a refused
        // receive and 30 plus it for a refused send; class 2 is
        // `ConnectionReset` and class 4 is `BrokenPipe` [SYS-7]. Which of the
        // three the program observes is the host's own timing and every one of
        // them is the peer's reset reaching source as an ordinary outcome.
        assert!(
            matches!(status, 22 | 32 | 34),
            "a reset must reach source as ConnectionReset or BrokenPipe, got {status} \
             (native ring: {native_ring})"
        );
    }
}

#[test]
fn a_whitefoot_client_sends_and_receives_on_both_routes() {
    let llvm = compile_program("tcp_client.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listen for the client");
        let port = listener.local_addr().expect("the listening address").port();
        let text = port.to_string();
        let child = program.spawn_on_route(native_ring, &[text.as_bytes()]);
        let (mut stream, _) = listener.accept().expect("accept the client");
        let mut sent = [0_u8; 8];
        stream
            .read_exact(&mut sent)
            .expect("read what the client sent");
        assert_eq!(&sent, b"ABCDEFGH", "native ring: {native_ring}");
        stream.write_all(b"abcdefgh").expect("answer the client");
        // The client reads to the end of its receiving direction, which this
        // peer decides.
        stream.shutdown(Shutdown::Write).expect("stop sending");
        let (status, published) = finished(child);
        assert_eq!(status, 0, "native ring: {native_ring}");
        assert_eq!(published, b"abcdefgh", "native ring: {native_ring}");
    }
}

#[test]
fn four_connections_reach_one_listener_on_both_routes() {
    let llvm = compile_program("tcp_fanout.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        let port = free_port();
        let text = port.to_string();
        let child = program.spawn_on_route(native_ring, &[text.as_bytes()]);
        for peer in 0..4_u8 {
            let mut stream = connect_when_ready(port);
            let sent = [peer, peer + 1, peer + 2];
            stream.write_all(&sent).expect("send this peer's bytes");
            let mut returned = Vec::new();
            stream
                .read_to_end(&mut returned)
                .expect("read this peer's answer");
            assert_eq!(returned, sent, "peer {peer} (native ring: {native_ring})");
        }
        let (status, _) = finished(child);
        assert_eq!(status, 0, "native ring: {native_ring}");
    }
}

#[test]
fn the_fanout_loop_states_its_permission_verdict() {
    // The judgment is target-independent and the same on both routes, so this
    // reads it once. The counted permission refuses the loop, because the
    // status it accumulates is written by no associative operation; the staged
    // permission admits it, because every edge that leaves the loop is in the
    // prologue, the factory is touched by the prologue alone, the listener is
    // only ever borrowed shared, the status is written by the remainder alone,
    // and the scratch is the iteration's own [PAR-3]. The disposition table is
    // pinned here because those four dispositions are what a server loop's
    // shape has to satisfy, and the ledger is where a writer reads them.
    //
    // What the permission grants, the lowering takes: the staged point is a
    // may-suspend user call, `serve_one`, and in a `--par` build the backend
    // offers it a lane frame there and retires it in the exact drain, so the
    // four accepts are in flight at once. The two cases at the end of this
    // module are the ones that say so — one on the emitted shape and one on
    // four peers that connect before any of them speaks.
    let ledger = program_permission_ledger("tcp_fanout.wf");
    let denial = ledger
        .iter()
        .find(|line| {
            line.starts_with("PAR loop")
                && line.contains("tcp_fanout.wf:")
                && line.contains("set outcome = reported")
        })
        .expect("the fixed-trip accept loop states a counted verdict");
    assert!(
        denial.contains("denied")
            && denial.contains("condition 1: the loop writes storage outliving the iteration"),
        "the accept loop's counted verdict must be the judgment's own, got {denial}"
    );
    let staging = ledger
        .iter()
        .find(|line| line.starts_with("PAR stage") && line.contains("serve_one(listener: &bound"))
        .expect("the fixed-trip accept loop states a staging verdict");
    assert!(
        staging.contains("permitted") && staging.contains("4 places classified"),
        "the accept loop must be staged at its accept, got {staging}"
    );
    for (disposition, place) in [
        ("serialized-P", "&uniq handles"),
        ("read-only", "&bound"),
        ("serialized-E", "set outcome = reported;"),
        ("replicated", "let scratch = buffer_new(256_u64, 0_u8);"),
    ] {
        assert!(
            ledger.iter().any(|line| {
                line.starts_with("PAR place")
                    && line.contains("tcp_fanout.wf:")
                    && line.contains(disposition)
                    && line.contains(place)
            }),
            "the ledger must classify {place} as {disposition}"
        );
    }
}

#[test]
fn a_refused_connect_hands_its_permit_back_on_both_routes() {
    let llvm = compile_program("tcp_refused.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        // A port this process reserved and released: nothing is listening on
        // it, so the host answers the connect with its own refusal.
        let port = free_port();
        let text = port.to_string();
        let child = program.spawn_on_route(native_ring, &[text.as_bytes()]);
        let (status, _) = finished(child);
        // The program exits zero only when both attempts answered
        // `ConnectFailed(ConnectionRefused, permit)` and the second used the
        // very permit the first handed back [SYS-10, SYS-17].
        assert_eq!(status, 0, "native ring: {native_ring}");
    }
}

/// Four peers connect before any of them speaks, and the last to connect is
/// the first to be answered.
///
/// A server that takes the four connections one at a time is parked in its
/// first peer's `receive_next` while the fourth peer waits, so this exchange
/// cannot complete; only a server whose four accepts are in flight at once
/// answers it. That is what the staged permission grants the fixed-trip
/// accept loop of `tcp_fanout.wf` in a `--par` build [PAR-3], and what the
/// lane hand-out of its staged call takes: the prologue of each iteration
/// takes its permit and publishes its `serve_one`, and the remainders, one
/// parked callee per peer, run overlapped. The reads carry a deadline so a
/// server that serves peers in turn fails this case in bounded time rather
/// than hanging it.
///
/// One and three workers, on both routes, because the property is about what the
/// runtime does with a wait and not about how many cores the runner has. A
/// pool sized to a large machine gives each of the four peers a worker of its
/// own, so a runtime that let every one of those workers block inside its own
/// peer's `receive_next` would still answer all four here and fail only on a
/// three-core host. One worker also detects a bootstrap that disables I/O
/// overlap along with CPU parallelism. Pinned below the peer count, the fourth
/// peer is answered only if waits are carried by something other than workers.
#[test]
fn four_peers_are_served_at_once_under_par_on_both_routes() {
    let llvm = compile_program_with_overlap("tcp_fanout.wf");
    let program = build_program(&llvm);
    for workers in ["1", "3"] {
        for native_ring in [true, false] {
            let port = free_port();
            let text = port.to_string();
            let child =
                program.spawn_on_route_with_workers(native_ring, Some(workers), &[text.as_bytes()]);
            let mut streams = (0..4_u8)
                .map(|_| connect_when_ready(port))
                .collect::<Vec<_>>();
            for peer in (0..4_u8).rev() {
                let stream = &mut streams[usize::from(peer)];
                stream
                    .set_read_timeout(Some(Duration::from_secs(20)))
                    .expect("bound the wait for this peer's answer");
                let sent = [peer, peer + 1, peer + 2];
                stream.write_all(&sent).expect("send this peer's bytes");
                let mut returned = Vec::new();
                stream.read_to_end(&mut returned).unwrap_or_else(|error| {
                    panic!(
                        "peer {peer} was not answered while earlier peers were still silent \
                         (native ring: {native_ring}, workers: {workers}): {error}"
                    )
                });
                assert_eq!(
                    returned, sent,
                    "peer {peer} (native ring: {native_ring}, workers: {workers})"
                );
            }
            drop(streams);
            let (status, _) = finished(child);
            assert_eq!(status, 0, "native ring: {native_ring}");
        }
    }
}

/// The fanout loop's carrying block offers its staged call to a lane, and its
/// drain retires it.
///
/// The end-to-end case above proves the four peers are answered at once; this
/// one pins the shape that does it, so a regression that quietly returns the
/// loop to serving in turn fails here as well as there. The publish is in the
/// block the loop carries an iteration out of, and the join and the release
/// are in the exact drain, which is the whole of the schedule: submit at the
/// staged point, retire in iteration order. The `--no-overlap` module names
/// none of the four lane entries, because a hand-out exists only in the world
/// that asked for one.
#[test]
fn the_fanout_loop_offers_its_staged_call_to_a_lane_and_retires_it_in_the_drain() {
    let overlapped = compile_program_with_overlap("tcp_fanout.wf");
    let main = emitted_function(&overlapped, "main");
    // The window is asked once at the loop's entry, with the trip count the
    // source states and the compiler's own ceiling: one lane frame slot per
    // in-flight iteration.
    assert!(
        main.contains(&format!(
            "call i64 @wf__completion_window(i64 4, i64 0, i64 {})",
            whitefoot::LANE_SLOTS
        )),
        "the staged loop must ask for its window once at entry:\n{main}"
    );
    let offer = labelled_block(main, "par.staged.offer.");
    assert!(
        offer.contains("call void @wf__par_publish("),
        "the carrying block must publish the staged call's frame:\n{main}"
    );
    let wait = labelled_block(main, "par.staged.wait.");
    assert!(
        wait.contains("call void @wf__par_join(") && wait.contains("call void @wf__par_release("),
        "the drain must join the frame, read it, and give it back:\n{main}"
    );
    // The refused edge is the same call on the same operands, run where it is
    // written, and its answer waits in the same ring element the drain reads.
    let inline = labelled_block(main, "par.staged.inline.");
    assert!(
        inline.contains("call i8 @wf_serve_one("),
        "a refused acquisition must run the staged call inline:\n{main}"
    );

    let sequential = compile_program_without_overlap("tcp_fanout.wf");
    for entry in [
        "@wf__par_acquire_lane",
        "@wf__par_publish",
        "@wf__par_join",
        "@wf__par_release",
    ] {
        assert!(
            !sequential.contains(entry),
            "the --no-overlap module must name no lane entry, found {entry}"
        );
    }
}

/// The instructions of the first block whose label starts with `prefix`.
///
/// A label is the only unindented line inside an emitted function body, so the
/// block runs from the line after its label to the next unindented line.
fn labelled_block(function: &str, prefix: &str) -> String {
    let mut lines = function
        .lines()
        .skip_while(|line| !line.starts_with(prefix));
    assert!(
        lines.next().is_some(),
        "no block labelled {prefix} in:\n{function}"
    );
    lines
        .take_while(|line| line.starts_with(' '))
        .collect::<Vec<_>>()
        .join("\n")
}
