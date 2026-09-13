//! Ordinary PRE-1 declaration records, parsed by the same grammar as source declarations.
//! Opaque records have no fields or constructor; function signatures have no body.

use crate::source::PreludeSource;

pub(crate) const DECLARATIONS: &[(&str, PreludeSource, &str)] = &[
    (
        "prelude/Args.wf",
        PreludeSource::Opaque,
        r#"struct Args {
}
"#,
    ),
    (
        "prelude/HostString.wf",
        PreludeSource::Opaque,
        r#"struct HostString {
}
"#,
    ),
    (
        "prelude/RelativePath.wf",
        PreludeSource::Opaque,
        r#"struct RelativePath {
}
"#,
    ),
    (
        "prelude/DirectoryRead.wf",
        PreludeSource::Opaque,
        r#"linear struct DirectoryRead {
}
"#,
    ),
    (
        "prelude/ReadFile.wf",
        PreludeSource::Opaque,
        r#"linear struct ReadFile {
}
"#,
    ),
    (
        "prelude/OutputStream.wf",
        PreludeSource::Opaque,
        r#"struct OutputStream {
}
"#,
    ),
    (
        "prelude/ExitStatus.wf",
        PreludeSource::Opaque,
        r#"struct ExitStatus {
}
"#,
    ),
    (
        "prelude/DirectorySource.wf",
        PreludeSource::Opaque,
        r#"linear struct DirectorySource {
}
"#,
    ),
    (
        "prelude/HandleFactory.wf",
        PreludeSource::Opaque,
        r#"struct HandleFactory {
}
"#,
    ),
    (
        "prelude/InputStream.wf",
        PreludeSource::Opaque,
        r#"struct InputStream {
}
"#,
    ),
    (
        "prelude/SocketAddress.wf",
        PreludeSource::Opaque,
        r#"struct SocketAddress {
}
"#,
    ),
    (
        "prelude/TcpListener.wf",
        PreludeSource::Opaque,
        r#"linear struct TcpListener {
}
"#,
    ),
    (
        "prelude/TcpReceive.wf",
        PreludeSource::Opaque,
        r#"linear struct TcpReceive {
}
"#,
    ),
    (
        "prelude/TcpSend.wf",
        PreludeSource::Opaque,
        r#"linear struct TcpSend {
}
"#,
    ),
    (
        "prelude/structs.wf",
        PreludeSource::Items,
        r#"struct TcpConnection {
  receive: TcpReceive;
  send: TcpSend;
}

struct Inputs {
  args: Args;
  cwd: DirectoryRead;
  stdout: OutputStream;
  stderr: OutputStream;
  handles: HandleFactory;
  stdin: InputStream;
}
"#,
    ),
    (
        "prelude/types.wf",
        PreludeSource::Items,
        r#"enum ArgError {
  InvalidIndex();
}

enum Utf8Error {
  Utf8Invalid();
}

enum CopyError {
  CopyTooSmall(required: u64);
}

enum Utf8CopyError {
  Utf8CopyTooSmall(required: u64);
  Utf8CopyInvalid();
}

enum PathError {
  PathInvalid();
}

enum ReadStop {
  ReadEnd();
  ReadFailed(error: IoError);
}

enum IoError {
  NotFound(code: u32, origin: u8);
  PermissionDenied(code: u32, origin: u8);
  AlreadyExists(code: u32, origin: u8);
  NotDirectory(code: u32, origin: u8);
  IsDirectory(code: u32, origin: u8);
  DirectoryNotEmpty(code: u32, origin: u8);
  ReadOnly(code: u32, origin: u8);
  ResourceBusy(code: u32, origin: u8);
  InvalidInput(code: u32, origin: u8);
  InvalidPath(code: u32, origin: u8);
  Unsupported(code: u32, origin: u8);
  TimedOut(code: u32, origin: u8);
  BrokenPipe(code: u32, origin: u8);
  WriteZero(code: u32, origin: u8);
  UnexpectedEnd(code: u32, origin: u8);
  ConnectionRefused(code: u32, origin: u8);
  ConnectionReset(code: u32, origin: u8);
  ConnectionAborted(code: u32, origin: u8);
  NotConnected(code: u32, origin: u8);
  AddressInUse(code: u32, origin: u8);
  AddressUnavailable(code: u32, origin: u8);
  ResourceExhausted(code: u32, origin: u8);
  FileTooLarge(code: u32, origin: u8);
  NoSpace(code: u32, origin: u8);
  QuotaExceeded(code: u32, origin: u8);
  CrossDevice(code: u32, origin: u8);
  DeviceFailure(code: u32, origin: u8);
  Other(code: u32, origin: u8);
}

enum ListStop {
  ListEnd();
  ListFailed(error: IoError);
}

enum FileOpenOutcome {
  FileOpened(value: ReadFile);
  FileOpenFailed(error: IoError);
}

enum DirectoryOpenOutcome {
  DirectoryOpened(value: DirectoryRead);
  DirectoryOpenFailed(error: IoError);
}

enum SourceOpenOutcome {
  SourceOpened(value: DirectorySource);
  SourceOpenFailed(error: IoError);
}

enum ListenOutcome {
  Listening(listener: TcpListener);
  ListenFailed(error: IoError);
}

enum AcceptOutcome {
  Accepted(connection: TcpConnection, peer: SocketAddress);
  AcceptFailed(error: IoError);
}

enum ConnectOutcome {
  Connected(connection: TcpConnection);
  ConnectFailed(error: IoError);
}
"#,
    ),
    (
        "prelude/args_count.wf",
        PreludeSource::Function,
        r#"fn args_count(args: &Args) -> result: own u64 reads(args);
"#,
    ),
    (
        "prelude/arg_get.wf",
        PreludeSource::Function,
        r#"fn arg_get(args: &Args, position: own u64) -> result: own Result<HostString, ArgError> reads(args);
"#,
    ),
    (
        "prelude/host_bytes_len.wf",
        PreludeSource::Function,
        r#"fn host_bytes_len(value: &HostString) -> result: own u64 reads(value);
"#,
    ),
    (
        "prelude/host_copy_bytes.wf",
        PreludeSource::Function,
        r#"fn host_copy_bytes(value: &HostString, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value, destination), writes(destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/host_utf8_len.wf",
        PreludeSource::Function,
        r#"fn host_utf8_len(value: &HostString) -> result: own Result<u64, Utf8Error> reads(value);
"#,
    ),
    (
        "prelude/host_copy_utf8.wf",
        PreludeSource::Function,
        r#"fn host_copy_utf8(value: &HostString, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, Utf8CopyError> reads(value, destination), writes(destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/relative_path.wf",
        PreludeSource::Function,
        r#"fn relative_path(value: own HostString) -> result: own Result<RelativePath, PathError> pure;
"#,
    ),
    (
        "prelude/open_read.wf",
        PreludeSource::Function,
        r#"fn open_read(factory: &uniq HandleFactory, root: &DirectoryRead, path: &RelativePath) -> result: own FileOpenOutcome reads(factory, root, path), writes(factory);
"#,
    ),
    (
        "prelude/read_at.wf",
        PreludeSource::Function,
        r#"fn read_at(factory: &uniq HandleFactory, file: &uniq ReadFile, destination: &uniq MutSlice<u8>, file_offset: own u64, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> reads(factory, file, destination), writes(factory, file, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/write_once.wf",
        PreludeSource::Function,
        r#"fn write_once(factory: &uniq HandleFactory, output: &uniq OutputStream, source: &Slice<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(factory, output, source), writes(factory, output) contract {
  requires start <= end;
  requires end <= len_of(deref(source));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/exit_status.wf",
        PreludeSource::Function,
        r#"fn exit_status(code: own u8) -> result: own ExitStatus pure;
"#,
    ),
    (
        "prelude/open_directory.wf",
        PreludeSource::Function,
        r#"fn open_directory(factory: &uniq HandleFactory, root: &DirectoryRead, name: &Slice<u8>, start: own u64, end: own u64) -> result: own DirectoryOpenOutcome reads(factory, root, name), writes(factory) contract {
  requires start <= end;
  requires end <= len_of(deref(name));
};
"#,
    ),
    (
        "prelude/open_directory_source.wf",
        PreludeSource::Function,
        r#"fn open_directory_source(factory: &uniq HandleFactory, directory: &DirectoryRead) -> result: own SourceOpenOutcome reads(factory, directory), writes(factory);
"#,
    ),
    (
        "prelude/directory_next.wf",
        PreludeSource::Function,
        r#"fn directory_next(source: &uniq DirectorySource, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> (result: own Result<unit, ListStop>, next: own u64, entries: own u64) reads(source, destination), writes(source, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures start <= next;
  ensures next <= end;
};
"#,
    ),
    (
        "prelude/open_file.wf",
        PreludeSource::Function,
        r#"fn open_file(factory: &uniq HandleFactory, root: &DirectoryRead, name: &Slice<u8>, start: own u64, end: own u64) -> result: own FileOpenOutcome reads(factory, root, name), writes(factory) contract {
  requires start <= end;
  requires end <= len_of(deref(name));
};
"#,
    ),
    (
        "prelude/close_read.wf",
        PreludeSource::Function,
        r#"fn close_read(factory: &uniq HandleFactory, file: own ReadFile) -> result: own Result<unit, IoError> reads(factory, file), writes(factory, file);
"#,
    ),
    (
        "prelude/close_directory.wf",
        PreludeSource::Function,
        r#"fn close_directory(factory: &uniq HandleFactory, directory: own DirectoryRead) -> result: own Result<unit, IoError> reads(factory, directory), writes(factory, directory);
"#,
    ),
    (
        "prelude/close_directory_source.wf",
        PreludeSource::Function,
        r#"fn close_directory_source(factory: &uniq HandleFactory, source: own DirectorySource) -> result: own Result<unit, IoError> reads(factory, source), writes(factory, source);
"#,
    ),
    (
        "prelude/read_next.wf",
        PreludeSource::Function,
        r#"fn read_next(factory: &uniq HandleFactory, input: &uniq InputStream, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> reads(factory, input, destination), writes(factory, input, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/socket_address_v4.wf",
        PreludeSource::Function,
        r#"fn socket_address_v4(a: own u8, b: own u8, c: own u8, d: own u8, port: own u16) -> result: own SocketAddress pure;
"#,
    ),
    (
        "prelude/socket_address_v6.wf",
        PreludeSource::Function,
        r#"fn socket_address_v6(a: own u16, b: own u16, c: own u16, d: own u16, e: own u16, f: own u16, g: own u16, h: own u16, port: own u16) -> result: own SocketAddress pure;
"#,
    ),
    (
        "prelude/tcp_listen.wf",
        PreludeSource::Function,
        r#"fn tcp_listen(factory: &uniq HandleFactory, address: &SocketAddress) -> result: own ListenOutcome reads(factory, address), writes(factory);
"#,
    ),
    (
        "prelude/tcp_accept.wf",
        PreludeSource::Function,
        r#"fn tcp_accept(factory: &uniq HandleFactory, listener: &uniq TcpListener) -> result: own AcceptOutcome reads(factory, listener), writes(factory, listener);
"#,
    ),
    (
        "prelude/tcp_connect.wf",
        PreludeSource::Function,
        r#"fn tcp_connect(factory: &uniq HandleFactory, address: &SocketAddress) -> result: own ConnectOutcome reads(factory, address), writes(factory);
"#,
    ),
    (
        "prelude/receive_next.wf",
        PreludeSource::Function,
        r#"fn receive_next(receive: &uniq TcpReceive, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> reads(receive, destination), writes(receive, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/send_once.wf",
        PreludeSource::Function,
        r#"fn send_once(send: &uniq TcpSend, source: &Slice<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(send, source), writes(send) contract {
  requires start <= end;
  requires end <= len_of(deref(source));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/close_listener.wf",
        PreludeSource::Function,
        r#"fn close_listener(factory: &uniq HandleFactory, listener: own TcpListener) -> result: own Result<unit, IoError> reads(factory, listener), writes(factory, listener);
"#,
    ),
    (
        "prelude/close_receive.wf",
        PreludeSource::Function,
        r#"fn close_receive(factory: &uniq HandleFactory, receive: own TcpReceive) -> result: own Result<unit, IoError> reads(factory, receive), writes(factory, receive);
"#,
    ),
    (
        "prelude/close_send.wf",
        PreludeSource::Function,
        r#"fn close_send(factory: &uniq HandleFactory, send: own TcpSend) -> result: own Result<unit, IoError> reads(factory, send), writes(factory, send);
"#,
    ),
];

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use crate::{
        ACTIVE_KERNEL_SPEC_HASH, CanonicalOutcome, CompilerLimits, FinalizeOutcome, LexOutcome,
        ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput,
        TerminalOutcome, audit_canonical, check_semantics, classify_terminals, finalize, lex,
        parse, resolve,
    };

    #[test]
    fn declarations_are_parsed_resolved_and_checked_as_ordinary_signatures() {
        let limits = CompilerLimits::default();
        let bundle = SourceBundle::with_prelude(
            &[SourceInput::new("ordinary.wf", b"fn transfer(value: own ReadFile) -> result: own ReadFile pure {\n  return move value;\n}\n")],
            limits.source,
        ).expect("ordinary prelude source bundle");
        let LexOutcome::Complete(lexed) = lex(&bundle, limits.lexer) else {
            panic!("ordinary prelude lexing");
        };
        let TerminalOutcome::Complete(classified) =
            classify_terminals(&lexed, ACTIVE_KERNEL_SPEC_HASH, limits.terminals)
        else {
            panic!("ordinary prelude terminals");
        };
        let parsed = parse(&classified, limits.parser);
        let ParseOutcome::Complete(parsed) = parsed else {
            panic!("ordinary prelude grammar: {parsed:?}");
        };
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, limits.finalizer) else {
            panic!("ordinary prelude topology");
        };
        let canonical = audit_canonical(finalized, limits.canonical);
        let CanonicalOutcome::Complete(canonical) = canonical else {
            panic!("ordinary prelude canonical bytes: {canonical:?}");
        };
        let resolved = resolve(canonical);
        let ResolutionOutcome::Complete(resolved) = resolved else {
            panic!("ordinary prelude resolution: {resolved:?}");
        };
        let checked = check_semantics(resolved);
        let SemanticOutcome::Complete(checked) = checked else {
            panic!("ordinary declaration and owned transfer: {checked:?}");
        };
        let signatures = checked
            .data
            .functions
            .iter()
            .filter(|function| function.body.is_none())
            .count();
        assert_eq!(signatures, 29);
        assert!(
            !checked
                .data
                .functions
                .iter()
                .any(|function| function.name == "main")
        );
        let transferred = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "transfer")
            .expect("ordinary source function");
        assert!(transferred.body.is_some());
        assert!(checked.data.nominals.iter().any(|nominal| {
            nominal.name == "ReadFile"
                && nominal.linear
                && matches!(nominal.kind, crate::semantic::CheckedNominalKind::Opaque)
        }));
    }
}
