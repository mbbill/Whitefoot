//! Ordinary PRE-1 declaration records, parsed by the same grammar as source declarations.
//! An opaque record has a refused constructor [TYPE-2]; a host handle has no
//! fields and the cell `Box` has one; function signatures have no body.

use crate::source::PreludeSource;

pub(crate) const DECLARATIONS: &[(&str, PreludeSource, &str)] = &[
    // [PRE-1] writes the three storage shapes first, then the cell, then the
    // fourteen host handles. [TYPE-2] makes each of the four an opaque struct
    // with a constructor entry that exists to be refused, and [TYPE-9] keeps
    // their element storage compiler-owned: a declaration can state neither
    // the elements nor the omitted-capacity form, so what the body carries is
    // exactly the readonly measure fields [MSR-1].
    //
    // The fence writes the capacity parameter `const N: u64`. That spelling
    // does not parse: [GRAM-2]'s `gparam := "const" IDENT ":" type` takes a
    // lexical IDENT, and [TYPE-2] says so outright -- "in this
    // specification's prose `N` stands for a written const argument; source
    // writes a `const` IDENT, lowercase under [FORM-3], as the [PRE-1] rows
    // do". The rows below therefore write `const n: u64`, exactly as
    // `slots_new<T, const n: u64>` of the same fence does.
    //
    // [OWN-1] `Array` carries no capability modifier, so an instance has the
    // capabilities of its element; `Slots`, `Ring`, `Box` and the host
    // handles are `nocopy`, or `nodrop` where the handle must be closed.
    (
        "prelude/Array.wf",
        PreludeSource::Opaque,
        r#"opaque struct Array<T, const n: u64> {
  readonly len: u64;
}
"#,
    ),
    (
        "prelude/Slots.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct Slots<T, const n: u64> {
  readonly len: u64;
  readonly cap: u64;
}
"#,
    ),
    (
        "prelude/Ring.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct Ring<T, const n: u64> {
  readonly len: u64;
  readonly cap: u64;
  readonly head: u64;
}
"#,
    ),
    // [PRE-1] writes the cell after the three shapes and ahead of the fourteen
    // [TYPE-2] makes it an opaque struct with one field and a constructor
    // entry that exists to be refused.
    //
    // [GRAM-2]'s `gparam := TYPEID (":" (TYPEID | capability_bound))?` makes
    // the bound optional and [PROV-6] reads an absent bound as no capability,
    // so `Box<T>` admits a content of every class, as `box_new<T>` of the
    // same fence does.
    (
        "prelude/Box.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct Box<T> {
  inner: T;
}
"#,
    ),
    (
        "prelude/Args.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct Args {
}
"#,
    ),
    (
        "prelude/HostString.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct HostString {
}
"#,
    ),
    (
        "prelude/RelativePath.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct RelativePath {
}
"#,
    ),
    (
        "prelude/DirectoryRead.wf",
        PreludeSource::Opaque,
        r#"opaque nodrop struct DirectoryRead {
}
"#,
    ),
    (
        "prelude/ReadFile.wf",
        PreludeSource::Opaque,
        r#"opaque nodrop struct ReadFile {
}
"#,
    ),
    (
        "prelude/OutputStream.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct OutputStream {
}
"#,
    ),
    (
        "prelude/ExitStatus.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct ExitStatus {
}
"#,
    ),
    (
        "prelude/DirectorySource.wf",
        PreludeSource::Opaque,
        r#"opaque nodrop struct DirectorySource {
}
"#,
    ),
    (
        "prelude/HandleFactory.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct HandleFactory {
}
"#,
    ),
    (
        "prelude/InputStream.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct InputStream {
}
"#,
    ),
    (
        "prelude/SocketAddress.wf",
        PreludeSource::Opaque,
        r#"opaque nocopy struct SocketAddress {
}
"#,
    ),
    (
        "prelude/TcpListener.wf",
        PreludeSource::Opaque,
        r#"opaque nodrop struct TcpListener {
}
"#,
    ),
    (
        "prelude/TcpReceive.wf",
        PreludeSource::Opaque,
        r#"opaque nodrop struct TcpReceive {
}
"#,
    ),
    (
        "prelude/TcpSend.wf",
        PreludeSource::Opaque,
        r#"opaque nodrop struct TcpSend {
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

struct AcceptedConnection {
  connection: TcpConnection;
  peer: SocketAddress;
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
        r#"fn host_copy_bytes(value: &HostString, destination: &[u8], start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value), writes(destination) contract {
  requires start <= end;
  requires end <= deref(destination).len;
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
        r#"fn host_copy_utf8(value: &HostString, destination: &[u8], start: own u64, end: own u64) -> result: own Result<u64, Utf8CopyError> reads(value), writes(destination) contract {
  requires start <= end;
  requires end <= deref(destination).len;
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
        r#"fn open_read(factory: &HandleFactory, root: &DirectoryRead, path: &RelativePath) -> result: own Result<ReadFile, IoError> reads(root), reads(path), writes(factory);
"#,
    ),
    (
        "prelude/read_at.wf",
        PreludeSource::Function,
        r#"fn read_at(factory: &HandleFactory, file: &ReadFile, destination: &[u8], file_offset: own u64, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> writes(factory), writes(file), writes(destination) contract {
  requires start <= end;
  requires end <= deref(destination).len;
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/write_once.wf",
        PreludeSource::Function,
        r#"fn write_once(factory: &HandleFactory, output: &OutputStream, source: &[u8], start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(source), writes(factory), writes(output) contract {
  requires start <= end;
  requires end <= deref(source).len;
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
        r#"fn open_directory(factory: &HandleFactory, root: &DirectoryRead, name: &[u8], start: own u64, end: own u64) -> result: own Result<DirectoryRead, IoError> reads(root), reads(name), writes(factory) contract {
  requires start <= end;
  requires end <= deref(name).len;
};
"#,
    ),
    (
        "prelude/open_directory_source.wf",
        PreludeSource::Function,
        r#"fn open_directory_source(factory: &HandleFactory, directory: &DirectoryRead) -> result: own Result<DirectorySource, IoError> reads(directory), writes(factory);
"#,
    ),
    (
        "prelude/directory_next.wf",
        PreludeSource::Function,
        r#"fn directory_next(source: &DirectorySource, destination: &[u8], start: own u64, end: own u64) -> (result: own Result<unit, ListStop>, next: own u64, entries: own u64) writes(source), writes(destination) contract {
  requires start <= end;
  requires end <= deref(destination).len;
  ensures start <= next;
  ensures next <= end;
};
"#,
    ),
    (
        "prelude/open_file.wf",
        PreludeSource::Function,
        r#"fn open_file(factory: &HandleFactory, root: &DirectoryRead, name: &[u8], start: own u64, end: own u64) -> result: own Result<ReadFile, IoError> reads(root), reads(name), writes(factory) contract {
  requires start <= end;
  requires end <= deref(name).len;
};
"#,
    ),
    (
        "prelude/close_read.wf",
        PreludeSource::Function,
        r#"fn close_read(factory: &HandleFactory, file: own ReadFile) -> result: own Result<unit, IoError> writes(factory);
"#,
    ),
    (
        "prelude/close_directory.wf",
        PreludeSource::Function,
        r#"fn close_directory(factory: &HandleFactory, directory: own DirectoryRead) -> result: own Result<unit, IoError> writes(factory);
"#,
    ),
    (
        "prelude/close_directory_source.wf",
        PreludeSource::Function,
        r#"fn close_directory_source(factory: &HandleFactory, source: own DirectorySource) -> result: own Result<unit, IoError> writes(factory);
"#,
    ),
    (
        "prelude/read_next.wf",
        PreludeSource::Function,
        r#"fn read_next(factory: &HandleFactory, input: &InputStream, destination: &[u8], start: own u64, end: own u64) -> result: own Result<u64, ReadStop> writes(factory), writes(input), writes(destination) contract {
  requires start <= end;
  requires end <= deref(destination).len;
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
        r#"fn tcp_listen(factory: &HandleFactory, address: &SocketAddress) -> result: own Result<TcpListener, IoError> reads(address), writes(factory);
"#,
    ),
    (
        "prelude/tcp_accept.wf",
        PreludeSource::Function,
        r#"fn tcp_accept(factory: &HandleFactory, listener: &TcpListener) -> result: own Result<AcceptedConnection, IoError> writes(factory), writes(listener);
"#,
    ),
    (
        "prelude/tcp_connect.wf",
        PreludeSource::Function,
        r#"fn tcp_connect(factory: &HandleFactory, address: &SocketAddress) -> result: own Result<TcpConnection, IoError> reads(address), writes(factory);
"#,
    ),
    (
        "prelude/receive_next.wf",
        PreludeSource::Function,
        r#"fn receive_next(receive: &TcpReceive, destination: &[u8], start: own u64, end: own u64) -> result: own Result<u64, ReadStop> writes(receive), writes(destination) contract {
  requires start <= end;
  requires end <= deref(destination).len;
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/send_once.wf",
        PreludeSource::Function,
        r#"fn send_once(send: &TcpSend, source: &[u8], start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(source), writes(send) contract {
  requires start <= end;
  requires end <= deref(source).len;
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
"#,
    ),
    (
        "prelude/close_listener.wf",
        PreludeSource::Function,
        r#"fn close_listener(factory: &HandleFactory, listener: own TcpListener) -> result: own Result<unit, IoError> writes(factory);
"#,
    ),
    (
        "prelude/close_receive.wf",
        PreludeSource::Function,
        r#"fn close_receive(factory: &HandleFactory, receive: own TcpReceive) -> result: own Result<unit, IoError> writes(factory);
"#,
    ),
    (
        "prelude/close_send.wf",
        PreludeSource::Function,
        r#"fn close_send(factory: &HandleFactory, send: own TcpSend) -> result: own Result<unit, IoError> writes(factory);
"#,
    ),
    (
        "prelude/box_new.wf",
        PreludeSource::Function,
        r#"fn box_new<T>(value: own T) -> result: own Box<T> pure;
"#,
    ),
    (
        "prelude/array_filled.wf",
        PreludeSource::Function,
        r#"fn array_filled<T: copy, const n: u64>(value: own T) -> result: own Array<T, n> pure contract {
  ensures result.len == n;
};
"#,
    ),
    (
        "prelude/slots_new.wf",
        PreludeSource::Function,
        r#"fn slots_new<T, const n: u64>() -> result: own Slots<T, n> pure contract {
  ensures result.len == 0_u64;
  ensures result.cap == n;
};
"#,
    ),
    (
        "prelude/ring_new.wf",
        PreludeSource::Function,
        r#"fn ring_new<T, const n: u64>() -> result: own Ring<T, n> pure contract {
  ensures result.len == 0_u64;
  ensures result.cap == n;
  ensures result.head == 0_u64;
};
"#,
    ),
    (
        "prelude/box_array_filled.wf",
        PreludeSource::Function,
        r#"fn box_array_filled<T: copy>(count: own u64, value: own T) -> result: own Box<Array<T>> pure contract {
  ensures result.inner.len == count;
};
"#,
    ),
    (
        "prelude/box_slots_new.wf",
        PreludeSource::Function,
        r#"fn box_slots_new<T>(capacity: own u64) -> result: own Box<Slots<T>> pure contract {
  ensures result.inner.len == 0_u64;
  ensures result.inner.cap == capacity;
};
"#,
    ),
    (
        "prelude/box_ring_new.wf",
        PreludeSource::Function,
        r#"fn box_ring_new<T>(capacity: own u64) -> result: own Box<Ring<T>> pure contract {
  ensures result.inner.len == 0_u64;
  ensures result.inner.cap == capacity;
  ensures result.inner.head == 0_u64;
};
"#,
    ),
    (
        "prelude/slots_from_array.wf",
        PreludeSource::Function,
        r#"fn slots_from_array<T, const n: u64>(values: own Array<T, n>) -> result: own Slots<T, n> pure contract {
  ensures result.len == n;
  ensures result.cap == n;
};
"#,
    ),
    (
        "prelude/slots_into_array.wf",
        PreludeSource::Function,
        r#"fn slots_into_array<T, const n: u64>(values: own Slots<T, n>) -> result: own Array<T, n> pure contract {
  requires values.len == n;
  ensures result.len == n;
};
"#,
    ),
    (
        "prelude/place_back.wf",
        PreludeSource::Function,
        r#"fn place_back<W, T>(window: &W, value: own T) -> result: own unit writes(window.next), writes(window.len) contract {
  requires deref(window).len < deref(window).cap;
  ensures deref(window).len == deref(entry(window)).len + 1_u64;
};
"#,
    ),
    (
        "prelude/take_back.wf",
        PreludeSource::Function,
        r#"fn take_back<W, T>(window: &W) -> value: own T writes(window.last), writes(window.len) contract {
  requires deref(window).len > 0_u64;
  ensures deref(window).len + 1_u64 == deref(entry(window)).len;
};
"#,
    ),
    (
        "prelude/insert_at.wf",
        PreludeSource::Function,
        r#"fn insert_at<W, T>(window: &W, index: own u64, value: own T) -> result: own unit writes(window.filled), writes(window.next), writes(window.len) contract {
  requires index <= deref(window).len;
  requires deref(window).len < deref(window).cap;
  ensures deref(window).len == deref(entry(window)).len + 1_u64;
};
"#,
    ),
    (
        "prelude/remove_at.wf",
        PreludeSource::Function,
        r#"fn remove_at<W, T>(window: &W, index: own u64) -> value: own T writes(window.filled), writes(window.len) contract {
  requires index < deref(window).len;
  ensures deref(window).len + 1_u64 == deref(entry(window)).len;
};
"#,
    ),
    (
        "prelude/append.wf",
        PreludeSource::Function,
        r#"fn append<W, X>(destination: &W, source: &X) -> result: own unit writes(destination.free), writes(destination.len), writes(source.filled), writes(source.len) contract {
  requires deref(source).len <= deref(destination).cap - deref(destination).len;
  ensures deref(destination).len >= deref(entry(destination)).len;
  ensures deref(source).len == 0_u64;
};
"#,
    ),
    (
        "prelude/split_off.wf",
        PreludeSource::Function,
        r#"fn split_off<W, X>(source: &W, index: own u64, destination: &X) -> result: own unit writes(source.filled), writes(source.len), writes(destination.free), writes(destination.len) contract {
  requires index <= deref(source).len;
  requires deref(source).len - index <= deref(destination).cap - deref(destination).len;
  ensures deref(source).len == index;
  ensures deref(destination).len >= deref(entry(destination)).len;
};
"#,
    ),
    (
        "prelude/grow.wf",
        PreludeSource::Function,
        r#"fn grow<T>(cell: &Box<Slots<T>>, capacity: own u64) -> result: own unit writes(cell) contract {
  requires capacity >= deref(cell).inner.cap;
  ensures deref(cell).inner.cap == capacity;
  ensures deref(cell).inner.len == deref(entry(cell)).inner.len;
};
"#,
    ),
    (
        "prelude/place_front.wf",
        PreludeSource::Function,
        r#"fn place_front<W, T>(window: &W, value: own T) -> result: own unit writes(window) contract {
  requires deref(window).len < deref(window).cap;
  ensures deref(window).len == deref(entry(window)).len + 1_u64;
  ensures deref(window).cap == deref(entry(window)).cap;
  ensures deref(window).head >= 0_u64;
  ensures deref(window).head <= deref(window).cap;
};
"#,
    ),
    (
        "prelude/take_front.wf",
        PreludeSource::Function,
        r#"fn take_front<W, T>(window: &W) -> value: own T writes(window) contract {
  requires deref(window).len > 0_u64;
  ensures deref(window).len + 1_u64 == deref(entry(window)).len;
  ensures deref(window).cap == deref(entry(window)).cap;
  ensures deref(window).head >= 0_u64;
  ensures deref(window).head <= deref(window).cap;
};
"#,
    ),
    (
        "prelude/swap.wf",
        PreludeSource::Function,
        r#"fn swap<T>(first: &T, second: &T) -> result: own unit writes(first), writes(second);
"#,
    ),
    (
        "prelude/free_empty.wf",
        PreludeSource::Function,
        r#"fn free_empty<W>(window: own W) -> result: own unit pure contract {
  requires window.len == 0_u64;
};
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
        // [PRE-1]'s 29 host records are non-generic, so each one is checked as
        // itself and is one body-less signature here. The twenty
        // compiler-owned rows beside them — the nine construction functions
        // [OP-13], the nine window operations [OP-10], `swap` [OP-11] and
        // `free_empty` [OP-14] — are every one of them generic, so [FN-2]
        // gives them a checked function only per concrete instance and this
        // unit, which calls none of them, has no instance of any.
        assert_eq!(signatures, 29);
        for row in crate::lowering::COMPILER_OWNED_PRELUDE_ROWS {
            assert!(
                !checked
                    .data
                    .functions
                    .iter()
                    .any(|function| function.name == row),
                "{row} is generic and this unit instantiates it nowhere"
            );
        }
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
