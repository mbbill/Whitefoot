//! The entry's standard input stream, end to end [SYS-15].
//!
//! `stdin_echo.wf` reads `command.stdin` to its end with `read_next` and
//! publishes every byte it observed, so one run exercises both halves of the
//! stream pair against a real host. The two shapes a standard input takes are
//! different runtime paths and both are run here: a pipe, whose end the writer
//! decides and whose reads may be short, and a redirected regular file, which
//! the kernel completes without a readiness wait.
//!
//! Each shape is run on both routes the host has. The shipped default reaches
//! the Linux completion ring, which carries the stream read as a read at
//! offset -1; `WF_IO_NO_NATIVE_RING` runs the same program through the shared
//! file adapter's own `read`. The two must agree byte for byte, because the
//! route is an implementation choice and not a language one.

use super::support::{build_program, compile_program};

/// Larger than the program's 4096-byte chunk, so every run makes several
/// `read_next` calls and the program's own loop, not one lucky call, is what
/// reaches the end.
fn payload() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10_000);
    for index in 0..10_000_u32 {
        bytes.push(u8::try_from(index % 251).expect("a byte"));
    }
    bytes
}

#[test]
fn the_stream_read_lowers_through_the_one_submit_and_join_shape() {
    let llvm = compile_program("stdin_echo.wf");
    // One lowering: the unpositioned request kind, submitted into the frame's
    // own record and joined where the outcome is needed. No positioned read
    // and no direct arm.
    assert!(llvm.contains("call void @wf__completion_file_read_submit"));
    assert!(!llvm.contains("wf__completion_file_pread_submit"));
    assert!(llvm.contains("call void @wf__completion_file_join"));
    // The entry supplies the two standard handles the invocation already
    // holds — descriptor 1 for `command.stdout` and descriptor 0 for
    // `command.stdin` — and opens nothing for either [SYS-15]. The general
    // store the chunk lives in follows them as the seventh row's operand.
    let entry = llvm
        .split("define i32 @wf__main_body")
        .nth(1)
        .expect("the emitted entry body");
    assert!(entry.contains("call i8 @wf_main(i32 1, i32 0, "));
    assert!(!entry.contains("@open"));
}

#[test]
fn a_piped_standard_input_is_echoed_to_its_end_on_both_routes() {
    let llvm = compile_program("stdin_echo.wf");
    let program = build_program(&llvm);
    let bytes = payload();
    for native_ring in [true, false] {
        let output = program.run_with_piped_input(&bytes, native_ring);
        assert!(
            output.status.success(),
            "the echo must reach the end of a pipe (native ring: {native_ring}): {:?}",
            output.status
        );
        assert_eq!(output.stdout, bytes, "native ring: {native_ring}");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn a_redirected_file_is_echoed_to_its_end_on_both_routes() {
    let llvm = compile_program("stdin_echo.wf");
    let program = build_program(&llvm);
    let bytes = payload();
    for native_ring in [true, false] {
        let output = program.run_with_file_input(&bytes, native_ring);
        assert!(
            output.status.success(),
            "the echo must reach the end of a file (native ring: {native_ring}): {:?}",
            output.status
        );
        assert_eq!(output.stdout, bytes, "native ring: {native_ring}");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn an_empty_standard_input_reaches_its_end_without_publishing() {
    let llvm = compile_program("stdin_echo.wf");
    let program = build_program(&llvm);
    for native_ring in [true, false] {
        let output = program.run_with_piped_input(&[], native_ring);
        assert!(output.status.success(), "native ring: {native_ring}");
        assert!(output.stdout.is_empty(), "native ring: {native_ring}");
    }
}
