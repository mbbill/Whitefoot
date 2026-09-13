//! Ordinary prelude TCP functions over loopback peers.
//!
//! The same linked implementation may use the native engine or file adapter.
//! These private choices preserve bytes and outcomes. C2 removed PAR-3, so
//! the previous reverse-peer scheduling assertion and staged-lane shape
//! assertion are retired; the source fanout loop now serves peers in order.

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::process::Child;
use std::time::{Duration, Instant};

use super::support::{
    CompiledProgram, build_program, compile_and_run, compile_program, emitted_function,
    program_permission_ledger,
};

/// One port the host is not using, released before the program binds it.
///
/// A listening socket that never accepted leaves no connection in `TIME_WAIT`,
/// so the port is free the moment this drops and the program's own `bind`
/// answers without `SO_REUSEADDR` — which the runtime deliberately does not
/// set, because it would change what a second bind of one port means
///.
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
    //.
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
    // B7c4b-1: the runtime copy of the header is a run taken from one bump
    // extent reserved in this activation's frame, so the program reaches the
    // host allocator on no path at all and every validation-failure return
    // leaves the extent with the frame. The free this assertion used to count
    // was the heap buffer's, and there is no heap buffer any more.
    assert!(!main.contains("call void @free"));
    assert!(!llvm.contains("call ptr @malloc"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn tcp_calls_use_ordinary_linked_declarations() {
    let llvm = compile_program("tcp_echo.wf");
    for name in [
        "tcp_listen",
        "tcp_accept",
        "receive_next",
        "send_once",
        "close_listener",
        "close_receive",
        "close_send",
    ] {
        assert!(
            llvm.contains(&format!("call void @wf_{name}(")),
            "missing ordinary call {name}"
        );
        assert!(
            llvm.contains(&format!("declare void @wf_{name}(")),
            "missing ordinary declaration {name}"
        );
    }
    assert!(!llvm.contains("@wf__completion_"));
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
        // the end and the program returns without failure.
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
        // `ConnectionReset` and class 4 is `BrokenPipe`. Which of the
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
fn the_fanout_loop_has_only_ordinary_counted_permission() {
    let ledger = program_permission_ledger("tcp_fanout.wf");
    assert!(
        ledger.iter().any(|line| line.starts_with("PAR loop")
            && line.contains("denied")
            && line.contains("condition 1")),
        "{ledger:?}"
    );
    assert!(
        !ledger
            .iter()
            .any(|line| line.starts_with("PAR stage") || line.starts_with("PAR place")),
        "{ledger:?}"
    );
}

#[test]
fn a_refused_connect_restores_factory_capacity_on_both_routes() {
    let llvm = compile_program("tcp_refused.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        // A port this process reserved and released: nothing is listening on
        // it, so the host answers the connect with its own refusal.
        let port = free_port();
        let text = port.to_string();
        let child = program.spawn_on_route(native_ring, &[text.as_bytes()]);
        let (status, _) = finished(child);
        // Both attempts must report ConnectionRefused; failed construction
        // leaves the same factory available for the second ordinary call.
        assert_eq!(status, 0, "native ring: {native_ring}");
    }
}
