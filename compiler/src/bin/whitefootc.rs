#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use whitefoot::{
    Architecture, COMPLETION_BRIDGE_HEADER, COMPLETION_BRIDGE_SOURCE, COMPLETION_CONTRACT_HEADER,
    COMPLETION_FILE_ADAPTER_HEADER, COMPLETION_FILE_ADAPTER_SOURCE, COMPLETION_FILE_POSIX_HEADER,
    COMPLETION_LINUX_IO_URING_HEADER, COMPLETION_RUNTIME_SOURCE, COMPLETION_SOCKET_ADDRESS_HEADER,
    COMPLETION_WINDOWS_IOCP_HEADER, CompilerLimits, FLOOR_STACK_BYTES, HOST_OPTIMIZATION_ARGUMENTS,
    OverlapLowering, SCHED_CORE_HEADER, SCHED_CORE_SOURCE, SCHED_ENTRY_HEADER, SCHED_ENTRY_SOURCE,
    SCHED_PRIM_HEADER, SCHED_SWITCH_HEADER, SourceInput, WINDOWS_RUNTIME_HEADER,
    compile_with_io_notices, compile_with_permission_ledger, module_requires_completion_runtime,
    module_requires_parallel_runtime, stack_ledger,
};

// `HOST_LINK_LIBRARIES` is here rather than above because its one reader is
// this platform's `TARGET_LINK_LIBRARIES`: Windows names no library of its
// own, so an unconditional import of it is an unused import there, and that
// is a clippy error on the host that would find it last.
#[cfg(not(target_os = "windows"))]
use whitefoot::{
    COMPLETION_FILE_POSIX_SOURCE, COMPLETION_LINUX_IO_URING_SOURCE, COMPLETION_WAIT_HOST_SOURCE,
    FLOOR_RUNTIME_SOURCE, HOST_LINK_LIBRARIES, SCHED_PRIM_HOST_SOURCE,
};

#[cfg(target_os = "windows")]
use whitefoot::{
    COMPLETION_FILE_WINDOWS_SOURCE, COMPLETION_WAIT_WINDOWS_SOURCE, COMPLETION_WINDOWS_IOCP_SOURCE,
    FLOOR_WINDOWS_RUNTIME_SOURCE, SCHED_PRIM_WINDOWS_SOURCE, WINDOWS_RUNTIME_SOURCE,
};

const USAGE: &str = "usage: whitefootc [--emit-llvm] [--par] [--no-overlap] [--par-ledger] \
[--stack-ledger] [-o OUTPUT] SOURCE...";

// The compiler walks typed source and lowering trees recursively. Windows
// gives the process's primary thread a 1 MiB stack by default, which is small
// enough for an ordinary region-and-match-heavy program to exhaust while the
// same source compiles on the other hosts. Own the driver thread's stack so a
// source program's acceptance does not depend on the host executable format.
const COMPILER_DRIVER_STACK_BYTES: usize = 8 * 1024 * 1024;

fn clang_executable() -> &'static str {
    if cfg!(target_os = "windows") {
        "clang"
    } else {
        "/usr/bin/clang"
    }
}

/// One compiler-owned runtime file and the path its quoted includes expect it
/// to have below the driver's private staging root.
///
/// The sources deliberately keep the repository's `backend/` topology:
/// `completion/bridge.c` reaches the scheduler core as `../sched/core.h` and
/// the Windows host runtime as `../windows_runtime.h`, so flattening these
/// bytes into one directory would change the meaning of those includes even
/// though every required file was written.
#[derive(Clone, Copy)]
struct RuntimeUnit {
    relative_path: &'static str,
    source: &'static str,
}

const fn unit(relative_path: &'static str, source: &'static str) -> RuntimeUnit {
    RuntimeUnit {
        relative_path,
        source,
    }
}

/// One rule holds for all six lists below, and it is what makes the staged
/// tree the same shape on every platform: *every header of a group is shared*,
/// and only the group's `.c` files and its clang input list are this
/// platform's.
///
/// A header is inert -- it is never a clang input, and it reaches the compiler
/// only when a compiled unit includes it -- so staging one a link will not
/// open costs a write and nothing else. Staging one a link *does* open is not
/// optional, and the two are not distinguishable by platform: `bridge.c` is
/// one shared unit whose `#if` arms name `windows_iocp.h` and
/// `../windows_runtime.h` in text that a POSIX build never compiles but that
/// any reader, and `runtime_staging_closes_every_quoted_include`, still walks.
/// Splitting headers by platform would make that walk answer differently on
/// two hosts for one set of bytes.
///
/// The units every program links, whatever it does: the resource-exhaustion
/// floor, and on Windows the host runtime whose bootstrap the emitted module
/// names.
const FLOOR_SHARED_UNITS: &[RuntimeUnit] = &[unit("windows_runtime.h", WINDOWS_RUNTIME_HEADER)];

#[cfg(not(target_os = "windows"))]
const FLOOR_PLATFORM_UNITS: &[RuntimeUnit] = &[unit("wf_floor.c", FLOOR_RUNTIME_SOURCE)];
#[cfg(not(target_os = "windows"))]
const FLOOR_COMPILE_UNITS: &[&str] = &["wf_floor.c"];

#[cfg(target_os = "windows")]
const FLOOR_PLATFORM_UNITS: &[RuntimeUnit] = &[
    unit("wf_floor_windows.c", FLOOR_WINDOWS_RUNTIME_SOURCE),
    unit("windows_runtime.c", WINDOWS_RUNTIME_SOURCE),
];
#[cfg(target_os = "windows")]
const FLOOR_COMPILE_UNITS: &[&str] = &["wf_floor_windows.c", "windows_runtime.c"];

/// The scheduler core and the platform layer over it. One core, one entry, and
/// one leaf per platform holding the seven primitives
/// (`research/investigations/io-model/PARK-ON-MISS.md` section 7.1).
const CORE_SHARED_UNITS: &[RuntimeUnit] = &[
    unit("sched/core.h", SCHED_CORE_HEADER),
    unit("sched/prim.h", SCHED_PRIM_HEADER),
    unit("sched/switch.h", SCHED_SWITCH_HEADER),
    unit("sched/entry.h", SCHED_ENTRY_HEADER),
    unit("sched/core.c", SCHED_CORE_SOURCE),
    unit("sched/entry.c", SCHED_ENTRY_SOURCE),
];

#[cfg(not(target_os = "windows"))]
const CORE_PLATFORM_UNITS: &[RuntimeUnit] = &[unit("sched/prim_host.c", SCHED_PRIM_HOST_SOURCE)];
#[cfg(not(target_os = "windows"))]
const CORE_COMPILE_UNITS: &[&str] = &["sched/core.c", "sched/prim_host.c", "sched/entry.c"];

#[cfg(target_os = "windows")]
const CORE_PLATFORM_UNITS: &[RuntimeUnit] =
    &[unit("sched/prim_windows.c", SCHED_PRIM_WINDOWS_SOURCE)];
#[cfg(target_os = "windows")]
const CORE_COMPILE_UNITS: &[&str] = &["sched/core.c", "sched/prim_windows.c", "sched/entry.c"];

/// The completion runtime: one wake epoch, one bridge, one file adapter, every
/// platform's header, and per platform the wait set, the host leaf of the
/// adapter, and the kernel completion ring.
const COMPLETION_SHARED_UNITS: &[RuntimeUnit] = &[
    unit("completion/contract.h", COMPLETION_CONTRACT_HEADER),
    unit("completion/file_adapter.h", COMPLETION_FILE_ADAPTER_HEADER),
    unit("completion/bridge.h", COMPLETION_BRIDGE_HEADER),
    unit("completion/file_posix.h", COMPLETION_FILE_POSIX_HEADER),
    unit(
        "completion/socket_address.h",
        COMPLETION_SOCKET_ADDRESS_HEADER,
    ),
    unit(
        "completion/linux_io_uring.h",
        COMPLETION_LINUX_IO_URING_HEADER,
    ),
    unit("completion/windows_iocp.h", COMPLETION_WINDOWS_IOCP_HEADER),
    unit("completion/runtime.c", COMPLETION_RUNTIME_SOURCE),
    unit("completion/file_adapter.c", COMPLETION_FILE_ADAPTER_SOURCE),
    unit("completion/bridge.c", COMPLETION_BRIDGE_SOURCE),
];

#[cfg(not(target_os = "windows"))]
const COMPLETION_PLATFORM_UNITS: &[RuntimeUnit] = &[
    unit("completion/wait_host.c", COMPLETION_WAIT_HOST_SOURCE),
    unit("completion/file_posix.c", COMPLETION_FILE_POSIX_SOURCE),
    unit(
        "completion/linux_io_uring.c",
        COMPLETION_LINUX_IO_URING_SOURCE,
    ),
];
#[cfg(not(target_os = "windows"))]
const COMPLETION_COMPILE_UNITS: &[&str] = &[
    "completion/runtime.c",
    "completion/wait_host.c",
    "completion/file_adapter.c",
    "completion/file_posix.c",
    "completion/bridge.c",
    "completion/linux_io_uring.c",
];

#[cfg(target_os = "windows")]
const COMPLETION_PLATFORM_UNITS: &[RuntimeUnit] = &[
    unit("completion/wait_windows.c", COMPLETION_WAIT_WINDOWS_SOURCE),
    unit("completion/file_windows.c", COMPLETION_FILE_WINDOWS_SOURCE),
    unit("completion/windows_iocp.c", COMPLETION_WINDOWS_IOCP_SOURCE),
];
#[cfg(target_os = "windows")]
const COMPLETION_COMPILE_UNITS: &[&str] = &[
    "completion/runtime.c",
    "completion/wait_windows.c",
    "completion/file_adapter.c",
    "completion/file_windows.c",
    "completion/bridge.c",
    "completion/windows_iocp.c",
];

/// The arguments this host's link needs beside the dialect and the inputs.
#[cfg(not(target_os = "windows"))]
const TARGET_COMPILE_ARGUMENTS: &[&str] = &["-pthread"];
#[cfg(target_os = "windows")]
const TARGET_COMPILE_ARGUMENTS: &[&str] = &["-municode"];

/// The libraries this host's link needs. Windows names exactly one: every
/// other facility the runtime uses is in the import libraries clang links by
/// default, and Winsock is not — the TCP routes of [SYS-17] and [SYS-18] are
/// what put it here.
#[cfg(not(target_os = "windows"))]
const TARGET_LINK_LIBRARIES: &[&str] = HOST_LINK_LIBRARIES;
#[cfg(target_os = "windows")]
const TARGET_LINK_LIBRARIES: &[&str] = &["-lws2_32"];

fn main() {
    let driver = match std::thread::Builder::new()
        .name("whitefootc-driver".to_owned())
        .stack_size(COMPILER_DRIVER_STACK_BYTES)
        .spawn(run)
    {
        Ok(driver) => driver,
        Err(error) => {
            eprintln!("whitefootc: cannot start the compiler driver: {error}");
            std::process::exit(1);
        }
    };
    match driver.join() {
        Ok(Ok(())) => {}
        Ok(Err(message)) => {
            eprintln!("whitefootc: {message}");
            std::process::exit(1);
        }
        // The panic hook on the driver thread has already printed the panic.
        // Preserve Rust's ordinary panic exit status without printing a
        // second, less useful panic from this joining thread.
        Err(_) => std::process::exit(101),
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let options = Options::parse(&arguments)?;
    let mut paths = Vec::with_capacity(options.sources.len());
    let mut bytes = Vec::with_capacity(options.sources.len());
    for (index, source) in options.sources.iter().enumerate() {
        bytes.push(
            std::fs::read(source)
                .map_err(|error| format!("cannot read {}: {error}", source.display()))?,
        );
        paths.push(source_names(source, index));
    }
    let inputs: Vec<_> = paths
        .iter()
        .zip(&bytes)
        .map(|((logical, display), bytes)| SourceInput::from_host_path(logical, display, bytes))
        .collect();
    let overlap = options.overlap();
    let module = if options.par_ledger {
        // The permission ledger is developer output. It goes to stdout, which
        // `Options::parse` has already kept clear of the emitted module, and
        // never to the mandatory record channel. Its judgment lines are the
        // same with or without `--par`, because the judgment is pure. Its
        // actualization lines — what the lowering did with each permission it
        // was handed — exist only where actualization was asked for, so `--par`
        // adds lines to this ledger rather than changing any of them.
        let (module, ledger) =
            compile_with_permission_ledger(&inputs, CompilerLimits::default(), overlap)
                .map_err(|failure| failure.to_string())?;
        for line in &ledger {
            println!("{line}");
        }
        module
    } else {
        // The denied verdict of an I/O loop reaches the writer here, without a
        // flag, because it is a missed optimization on the program they just
        // compiled: a loop they wrote to read or write files lost its
        // pipeline. The compilation succeeded, so the lines are notes on
        // stderr, never a rejection, and a granted verdict says nothing at
        // all. `--par-ledger` above already prints these lines inside the full
        // report, so this branch is the only one that repeats them.
        let (module, notices) =
            compile_with_io_notices(&inputs, CompilerLimits::default(), overlap)
                .map_err(|failure| failure.to_string())?;
        for line in io_notice_report(options.no_overlap, &notices) {
            eprintln!("{line}");
        }
        module
    };
    if options.stack_ledger {
        for line in print_stack_ledger(&module)? {
            println!("{line}");
        }
    }
    if options.emit_llvm {
        if let Some(output) = options.output {
            std::fs::write(&output, &module)
                .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
        } else {
            print!("{module}");
        }
        return Ok(());
    }
    compile_executable(
        &module,
        options.output.as_deref().unwrap_or(Path::new("a.out")),
    )
}

/// The stderr lines an ordinary compilation reports for the denied I/O loops
/// it found: every notice prefixed `whitefootc: note:`, then one line saying
/// the compilation succeeded and where the full report is.
///
/// A build that asked for no overlap lowering reports none of them. The flag
/// is the writer stating that this build is the sequential reference one, so a
/// loop that lost its pipeline is not news about the program in front of them
/// — it is the build they asked for, and repeating the whole verdict on every
/// such compile is noise they cannot act on without contradicting the flag
/// they just wrote. Nothing else moves: the judgment still runs and reaches
/// the same verdicts, `--par-ledger` still prints the complete report under
/// the same flag, and the emitted module is the one `--no-overlap` always
/// emitted. This is a channel decision and the only one taken by the flag
/// rather than by what a line says.
fn io_notice_report(no_overlap: bool, notices: &[String]) -> Vec<String> {
    if no_overlap || notices.is_empty() {
        return Vec::new();
    }
    let mut report: Vec<String> = notices
        .iter()
        .map(|notice| format!("whitefootc: note: {notice}"))
        .collect();
    report.push(
        "whitefootc: note: the compilation succeeded; run --par-ledger for the \
         complete permission report"
            .to_owned(),
    );
    report
}

/// Compiles the module once more, to assembly, purely to read the two things
/// the ledger needs out of the host compiler: what each surviving machine
/// function's frame costs and which functions call which.
///
/// It is a second invocation rather than a flag on the link that already
/// happens, for two reasons. `-fstack-usage` writes its report beside the file
/// it compiled, so putting it on the ordinary link would drop a `.su` into
/// whatever directory the writer asked for output in, on a flag they may have
/// passed for the report alone. And the call graph has to be the post-inline
/// one that belongs to those frame numbers, which means assembly, which the
/// link does not produce. Both come out of the one compilation below, into a
/// directory this function owns and removes, and none of it runs unless the
/// ledger was asked for.
fn print_stack_ledger(llvm: &str) -> Result<Vec<String>, String> {
    let directory = std::env::temp_dir().join(format!("whitefootc-ledger-{}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create the ledger directory: {error}"))?;
    let module = directory.join("module.ll");
    let assembly = directory.join("module.s");
    let result = (|| {
        std::fs::write(&module, llvm)
            .map_err(|error| format!("cannot write the ledger module: {error}"))?;
        let status = Command::new(clang_executable())
            .arg("-x")
            .arg("ir")
            .arg(&module)
            .arg("-S")
            .arg("-o")
            .arg(&assembly)
            .arg("-fstack-usage")
            .arg("-Wno-override-module")
            .args(HOST_OPTIMIZATION_ARGUMENTS)
            .status()
            .map_err(|error| format!("cannot start {}: {error}", clang_executable()))?;
        if !status.success() {
            return Err(format!("clang exited with {status}"));
        }
        let usage = std::fs::read_to_string(directory.join("module.su"))
            .map_err(|error| format!("cannot read the stack-usage report: {error}"))?;
        let text = std::fs::read_to_string(&assembly)
            .map_err(|error| format!("cannot read the ledger assembly: {error}"))?;
        Ok(stack_ledger(
            &usage,
            &text,
            FLOOR_STACK_BYTES,
            Architecture::HOST,
        ))
    })();
    let _ = std::fs::remove_dir_all(&directory);
    result
}

/// The runtime units this link stages and the subset clang compiles.
///
/// One list-building rule for every platform: the floor always, the scheduler
/// core under the union of the two predicates, the completion units under the
/// second, and each of the three groups a shared part plus this platform's
/// leaves.
fn runtime_units(core: bool, completion: bool) -> (Vec<RuntimeUnit>, Vec<&'static str>) {
    let mut staged: Vec<RuntimeUnit> = FLOOR_SHARED_UNITS.to_vec();
    staged.extend_from_slice(FLOOR_PLATFORM_UNITS);
    let mut compiled: Vec<&'static str> = FLOOR_COMPILE_UNITS.to_vec();
    if core {
        staged.extend_from_slice(CORE_SHARED_UNITS);
        staged.extend_from_slice(CORE_PLATFORM_UNITS);
        compiled.extend_from_slice(CORE_COMPILE_UNITS);
    }
    if completion {
        staged.extend_from_slice(COMPLETION_SHARED_UNITS);
        staged.extend_from_slice(COMPLETION_PLATFORM_UNITS);
        compiled.extend_from_slice(COMPLETION_COMPILE_UNITS);
    }
    (staged, compiled)
}

/// Stages the compiler-owned runtime beside the emitted module and links them
/// into one executable.
///
/// One staging for every platform, and the lists above are the only thing that
/// differs. The floor joins unconditionally, because every program can exhaust
/// its stack. The scheduler core joins on the union of the two predicates
/// (`research/investigations/io-model/PARK-ON-MISS.md` section 7, "Where the
/// core is linked"): it is one scheduler for compute hand-outs and I/O
/// completions, so a module that hands work out needs it and so does a module
/// that submits an operation, and a completion-only program parks its stack at
/// every join. The completion units join on the second predicate alone.
///
/// Every one of those bytes travels inside this executable, so no installed
/// path, no build directory, and no environment decides which runtime a
/// program gets.
fn compile_executable(llvm: &str, output: &Path) -> Result<(), String> {
    let completion_required = module_requires_completion_runtime(llvm);
    let core_required = module_requires_parallel_runtime(llvm) || completion_required;
    let directory = std::env::temp_dir().join(format!("whitefootc-{}", std::process::id()));
    let result = (|| {
        let (staged, compiled) = runtime_units(core_required, completion_required);
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("cannot create the runtime directory: {error}"))?;
        for unit in &staged {
            let path = directory.join(unit.relative_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "cannot create the runtime directory for {}: {error}",
                        unit.relative_path
                    )
                })?;
            }
            std::fs::write(&path, unit.source)
                .map_err(|error| format!("cannot write runtime {}: {error}", unit.relative_path))?;
        }
        let mut command = Command::new(clang_executable());
        // The compiler-owned C units are written to C11 and the repository
        // gate compiles them as `-std=c11`. Naming the dialect here too is
        // what makes that gate a statement about this link: clang's default is
        // a GNU dialect, which predefines object-like macros such as `linux`
        // that a C11 source may legitimately use as an identifier.
        command
            .arg("-std=c11")
            .args(TARGET_COMPILE_ARGUMENTS)
            .arg("-I")
            .arg(&directory);
        if completion_required {
            command.arg("-I").arg(directory.join("completion"));
        }
        for relative_path in &compiled {
            command
                .arg("-x")
                .arg("c")
                .arg(directory.join(relative_path));
        }
        link(&mut command, llvm, output)
    })();
    let _ = std::fs::remove_dir_all(&directory);
    result
}

fn link(command: &mut Command, llvm: &str, output: &Path) -> Result<(), String> {
    let mut child = command
        .arg("-x")
        .arg("ir")
        .arg("-")
        .arg("-Wno-override-module")
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .args(TARGET_LINK_LIBRARIES)
        .arg("-o")
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start {}: {error}", clang_executable()))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "clang stdin was not available".to_owned())?
        .write_all(llvm.as_bytes())
        .map_err(|error| format!("cannot send LLVM to clang: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("cannot wait for clang: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("clang exited with {status}"))
    }
}

/// The two names one source argument carries: the bundle's own name, then the
/// name every reader is shown.
///
/// The display name is the argument, unchanged. That is the whole point: an
/// absolute path is how a script, a Makefile, and an agent all invoke this
/// compiler, and a diagnostic or ledger line that renames the file to
/// `input0.wf` names a file that exists nowhere on disk.
fn source_names(path: &Path, index: usize) -> (String, String) {
    (
        logical_path(path, index),
        path.to_string_lossy().into_owned(),
    )
}

/// The bundle's own name for one source.
///
/// This is program identity, not presentation: it orders the bundle and
/// detects a duplicate, and its spelling is the closed portable one, so a host
/// path that cannot be spelled there falls back to a positional name. Nothing
/// a reader sees comes from here — the diagnostic and the permission ledger
/// both name the display path, which is exactly the argument the caller typed.
fn logical_path(path: &Path, index: usize) -> String {
    let candidate = path.to_string_lossy();
    if !path.is_absolute() && portable_logical_path(&candidate) {
        candidate.into_owned()
    } else {
        format!("input{index}.wf")
    }
}

fn portable_logical_path(path: &str) -> bool {
    !path.is_empty()
        && path.split('/').all(|component| {
            !component.is_empty()
                && !matches!(component, "." | "..")
                && component
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

struct Options {
    emit_llvm: bool,
    /// Actualize the permission judgment's eligible groups on worker lanes.
    ///
    /// Compute outlining is off by default; compiler-owned completion I/O
    /// remains enabled independently. The compute path is free when requested
    /// and no pool is asked for.
    ///
    /// Outlining a call is not free — it passes its arguments through a memory
    /// frame, is reached through a function pointer, and rejoins its two edges
    /// through a phi that can foreclose a transform the sequential build gets.
    /// What that costs before any worker runs depends entirely on grain: on a
    /// heavy recursive fold it was already nothing, while `fib(38)` measured
    /// 0.0790 s default against 0.2337 s `--par` with the pool off, or 2.96x,
    /// almost all of it one foreclosed tail-recursion transform. So a `--par`
    /// build now carries the sequential lowering as well, byte for byte, and
    /// its bootstrap selects between the two once, on whether a pool was asked
    /// for. Re-measured the same way, `fib(38)` reads 0.0810 s with the pool
    /// off against 0.0811 s for the default build, and the twelve
    /// paired-layout programs read 1.00x-1.01x of their own default builds at
    /// one worker, where before they ranged 0.68x-1.02x. (Both readings are
    /// taken at `WF_WORKERS=1`. An absent setting asks for a pool sized to the
    /// machine, so it is not the pool-off spelling.)
    ///
    /// Two prices, both real. Code size, paid only by a `--par` build: 7% to
    /// 14% more machine code, which is 0.5% to 0.6% of the linked file. And a
    /// second copy shifts every address in that binary, which re-rolls this
    /// workload's known sensitivity to code placement — `par_layout.wf` reads
    /// 1.19x of its default build with the pool off, at equal instruction
    /// count and IPC 2.89 down to 2.43, and the *same* module linked with its
    /// two inputs in the other order reads 1.00x. That is the unlocated stall
    /// the batch 0075 skew investigation attributed and could not name; it is
    /// not a cost of the second copy, and it moves in both directions.
    ///
    /// The permission is never an obligation, so the default compilation takes
    /// none of it and emits exactly the module it emitted before this path
    /// existed, with one world in it. `WF_WORKERS` remains the runtime knob. On
    /// the optional POSIX path, `0`, `1`, or an unparsable value keeps the
    /// sequential world. A Windows module that actually hands work out has a
    /// stricter production contract: its native runtime must initialize usable
    /// worker lanes or terminate when the pool is first required; it cannot
    /// silently select the sequential world.
    par: bool,
    /// Emit the module a compiler with no overlap lowering at all emits.
    ///
    /// This is the sequential reference build, and it exists for one reason:
    /// measurement. The default compilation actualizes compiler-owned
    /// completion I/O, so without this switch there is no way to compile one
    /// source into the program that reaches the host through ordinary direct
    /// calls and compare the two. Every I/O call becomes an ordinary call:
    /// nothing is submitted, nothing is joined, and the completion runtime
    /// does not join the link. It is not a performance option a writer picks
    /// for a shipped program — the default build is what ships — and it
    /// changes no acceptance, no claim, and no published value.
    no_overlap: bool,
    /// Print the non-normative permission ledger on stdout.
    par_ledger: bool,
    /// Print the non-normative stack ledger on stdout.
    ///
    /// What a writer cannot otherwise see: a frame size is a whole-program,
    /// optimizer-chosen property, so no reading of a function tells anyone
    /// what one activation of it costs, and no reading of a program tells
    /// anyone which of its functions can reach themselves once the inliner has
    /// finished. The flagship program in this repository used to cap its
    /// directory recursion at sixteen levels, documenting that the truncation
    /// was indistinguishable from a complete search; its frame is 1744 bytes,
    /// which the runtime's own stack holds six hundred thousand of. The bound
    /// was not careful, it was blind, and this report is what replaced it.
    stack_ledger: bool,
    output: Option<PathBuf>,
    sources: Vec<PathBuf>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut emit_llvm = false;
        let mut par = false;
        let mut no_overlap = false;
        let mut par_ledger = false;
        let mut stack_ledger = false;
        let mut output = None;
        let mut sources = Vec::new();
        let mut cursor = 0;
        while cursor < arguments.len() {
            match arguments[cursor].as_str() {
                "--emit-llvm" => emit_llvm = true,
                "--par" => par = true,
                "--no-overlap" => no_overlap = true,
                "--par-ledger" => par_ledger = true,
                "--stack-ledger" => stack_ledger = true,
                "-o" => {
                    cursor += 1;
                    let path = arguments
                        .get(cursor)
                        .ok_or_else(|| "-o requires an output path".to_owned())?;
                    if output.replace(PathBuf::from(path)).is_some() {
                        return Err("-o may be written only once".to_owned());
                    }
                }
                "-h" | "--help" => {
                    return Err(USAGE.to_owned());
                }
                argument if argument.starts_with('-') => {
                    return Err(format!("unknown option: {argument}"));
                }
                source => sources.push(PathBuf::from(source)),
            }
            cursor += 1;
        }
        if sources.is_empty() {
            return Err(USAGE.to_owned());
        }
        // Two streams, one stdout. The emitted module is the payload of
        // `--emit-llvm` without `-o`, so the ledger may not be interleaved
        // into it; naming an output file separates them.
        if par_ledger && emit_llvm && output.is_none() {
            return Err(
                "--par-ledger cannot share stdout with --emit-llvm: name a module output with -o"
                    .to_owned(),
            );
        }
        if stack_ledger && emit_llvm && output.is_none() {
            return Err(
                "--stack-ledger cannot share stdout with --emit-llvm: name a module output with -o"
                    .to_owned(),
            );
        }
        // The two switches name opposite lowerings, so an invocation that
        // writes both has stated no build at all rather than a build with a
        // precedence rule to remember.
        if par && no_overlap {
            return Err("--no-overlap and --par select opposite lowerings: write one".to_owned());
        }
        Ok(Self {
            emit_llvm,
            par,
            no_overlap,
            par_ledger,
            stack_ledger,
            output,
            sources,
        })
    }

    /// The lowering this invocation compiles: the shipped completion build
    /// unless one of the two switches named another.
    ///
    /// The judgment is pure, so this selects an emitted lowering and nothing
    /// about what the compiler decided.
    fn overlap(&self) -> OverlapLowering {
        if self.no_overlap {
            OverlapLowering::Off
        } else if self.par {
            OverlapLowering::On
        } else {
            OverlapLowering::Completion
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::{Component, Path, PathBuf};

    use super::{
        CompilerLimits, Options, OverlapLowering, SourceInput, compile_with_io_notices,
        compile_with_permission_ledger, io_notice_report, module_requires_parallel_runtime,
        runtime_units, source_names,
    };
    use whitefoot::module_requires_completion_runtime;

    const PAR_LAYOUT: &[u8] = include_bytes!("../../../tests/programs/par_layout.wf");

    fn compile_parallel_fixture(name: &str, source: &[u8]) -> String {
        whitefoot::compile_with_overlap(
            &[SourceInput::new(name, source)],
            CompilerLimits::default(),
            OverlapLowering::On,
        )
        .expect("parallel runtime selection fixture must compile")
    }

    fn parse(arguments: &[&str]) -> Result<Options, String> {
        let owned: Vec<String> = arguments.iter().map(|value| (*value).to_owned()).collect();
        Options::parse(&owned)
    }

    /// The link stages the scheduler core on exactly the marker the emitter
    /// leaves, and stages nothing beyond the floor for a module that neither
    /// hands out work nor submits an operation.
    ///
    /// This case used to be about the Windows link alone, and about a second
    /// worker-pool unit only that platform had. Step (iv) deleted that unit:
    /// Windows takes the same `sched/entry.c` over the same `core.c` as every
    /// other target, so what the link still decides is which staging group a
    /// module needs, and that is what is asserted here.
    #[test]
    fn the_link_stages_the_core_on_the_hand_out_marker() {
        let plain = "define i32 @main() { ret i32 0 }";
        assert!(!module_requires_parallel_runtime(plain));
        assert!(!module_requires_completion_runtime(plain));
        let (staged, compiled) = runtime_units(false, false);
        assert_eq!(
            staged.len(),
            super::FLOOR_SHARED_UNITS.len() + super::FLOOR_PLATFORM_UNITS.len()
        );
        assert_eq!(compiled, super::FLOOR_COMPILE_UNITS.to_vec());

        let layout = compile_parallel_fixture("tests/programs/par_layout.wf", PAR_LAYOUT);
        assert!(module_requires_parallel_runtime(&layout));
        assert!(
            module_requires_completion_runtime(&layout),
            "par_layout's real write_once must still require completion"
        );
        let (_, compiled) = runtime_units(true, true);
        for required in ["sched/core.c", "sched/entry.c", "completion/bridge.c"] {
            assert!(
                compiled.contains(&required),
                "a module that hands out work and submits must compile `{required}`"
            );
        }
    }

    /// The driver stages the embedded sources with the same relative topology
    /// they have under `backend/`. Every quoted compiler-owned include must
    /// therefore resolve either beside the including file or from the one `-I`
    /// root passed to clang.
    ///
    /// This is stronger than naming the paths that exposed the original
    /// flattening bug: adding a new compiler-owned quoted include without its
    /// staged target makes this test fail before CI reaches clang. It runs on
    /// whichever host builds the compiler and so covers that host's own leaf
    /// selection.
    #[test]
    fn runtime_staging_closes_every_quoted_include() {
        fn normalized(path: &Path) -> Option<PathBuf> {
            let mut result = PathBuf::new();
            for component in path.components() {
                match component {
                    Component::CurDir => {}
                    Component::ParentDir => {
                        if !result.pop() {
                            return None;
                        }
                    }
                    Component::Normal(piece) => result.push(piece),
                    Component::Prefix(_) | Component::RootDir => return None,
                }
            }
            Some(result)
        }

        let (units, compiled) = runtime_units(true, true);
        let staged: HashSet<PathBuf> = units
            .iter()
            .map(|unit| PathBuf::from(unit.relative_path))
            .collect();
        assert_eq!(
            staged.len(),
            units.len(),
            "a staged runtime path is duplicated"
        );
        assert!(staged.contains(Path::new("sched/core.c")));
        assert!(staged.contains(Path::new("completion/bridge.c")));
        assert!(
            !staged.contains(Path::new("bridge.c")),
            "completion files must not be flattened into the staging root"
        );

        for relative_path in &compiled {
            assert!(
                staged.contains(Path::new(relative_path)),
                "clang input `{relative_path}` is not staged"
            );
        }

        for unit in &units {
            let parent = Path::new(unit.relative_path)
                .parent()
                .unwrap_or_else(|| Path::new(""));
            for line in unit.source.lines() {
                let Some(include) = line
                    .trim()
                    .strip_prefix("#include \"")
                    .and_then(|rest| rest.strip_suffix('"'))
                else {
                    continue;
                };
                let beside_source = normalized(&parent.join(include));
                let from_include_root = normalized(Path::new(include));
                assert!(
                    beside_source
                        .as_ref()
                        .is_some_and(|path| staged.contains(path))
                        || from_include_root
                            .as_ref()
                            .is_some_and(|path| staged.contains(path)),
                    "{} includes `{include}`, which the staged tree cannot resolve",
                    unit.relative_path
                );
            }
        }
    }

    /// Every reader-facing name is the argument the caller typed, including an
    /// absolute one, while the bundle's own key stays inside the closed
    /// portable spelling.
    ///
    /// The two answers differ exactly when the host path cannot be spelled as
    /// a logical path, and that is the case a writer meets first: a diagnostic
    /// that renamed `/tmp/wc.wf` to `input0.wf` cited a file that does not
    /// exist.
    #[test]
    fn a_source_is_shown_by_the_path_the_caller_wrote() {
        let (logical, display) = source_names(Path::new("/tmp/wc.wf"), 0);
        assert_eq!(display, "/tmp/wc.wf");
        assert_eq!(logical, "input0.wf");

        let (logical, display) = source_names(Path::new("../out of tree.wf"), 3);
        assert_eq!(display, "../out of tree.wf");
        assert_eq!(logical, "input3.wf");

        // A portable relative path is already both names.
        let (logical, display) = source_names(Path::new("programs/wc.wf"), 0);
        assert_eq!(display, "programs/wc.wf");
        assert_eq!(logical, "programs/wc.wf");
    }

    /// The permission ledger is an opt-in developer channel: off unless the
    /// invocation asks for it, and never a second source argument.
    #[test]
    fn the_permission_ledger_is_requested_by_its_own_option() {
        let options = parse(&["value.wf"]).expect("one source is a complete invocation");
        assert!(!options.par_ledger);
        assert_eq!(options.sources.len(), 1);

        let options = parse(&["--par-ledger", "value.wf"]).expect("the option is accepted");
        assert!(options.par_ledger);
        assert!(!options.emit_llvm);
        assert_eq!(options.sources.len(), 1);

        // With a named module output the two streams no longer share stdout,
        // so requesting both is a complete invocation.
        let options = parse(&["--par-ledger", "--emit-llvm", "-o", "out.ll", "value.wf"])
            .expect("a named output separates the module from the ledger");
        assert!(options.par_ledger);
        assert!(options.emit_llvm);
    }

    /// The stack ledger is developer output on its own switch, and it is
    /// independent of `--par`: a writer reads what a program's frames cost in
    /// the build they are shipping, not in one the flag changed underneath
    /// them. Asking for it with `--par` reports both of that module's worlds
    /// instead of one.
    #[test]
    fn the_stack_ledger_is_requested_by_its_own_option() {
        let options = parse(&["value.wf"]).expect("one source is a complete invocation");
        assert!(!options.stack_ledger);

        let options = parse(&["--stack-ledger", "value.wf"]).expect("the option is accepted");
        assert!(options.stack_ledger);
        assert!(!options.par, "reading the ledger must not enable lowering");
        assert!(!options.emit_llvm);

        let options = parse(&["--par", "--stack-ledger", "value.wf"]).expect("both are accepted");
        assert!(options.stack_ledger && options.par);
    }

    /// Either ledger may not be interleaved into a module that is itself going
    /// to stdout, so that invocation is refused rather than corrupted.
    #[test]
    fn a_ledger_refuses_to_share_stdout_with_the_module() {
        let message = parse(&["--stack-ledger", "--emit-llvm", "value.wf"])
            .err()
            .expect("the two stdout streams must not be mixed");
        assert!(message.contains("--stack-ledger"), "{message}");
        assert!(message.contains("-o"), "{message}");
    }

    /// The ledger may not be interleaved into a module that is itself going
    /// to stdout, so that invocation is refused rather than corrupted.
    #[test]
    fn the_permission_ledger_refuses_to_share_stdout_with_the_module() {
        let message = parse(&["--par-ledger", "--emit-llvm", "value.wf"])
            .err()
            .expect("the two stdout streams must not be mixed");
        assert!(message.contains("--par-ledger"), "{message}");
        assert!(message.contains("-o"), "{message}");
    }

    /// Overlap lowering is off unless the invocation asks for it, and asking
    /// for the ledger is not asking for it.
    ///
    /// The two options are independent on purpose: the judgment is pure, so a
    /// developer can read what the compiler decided about a program without
    /// changing one byte of the program that gets built.
    #[test]
    fn overlap_lowering_is_off_unless_the_invocation_asks_for_it() {
        let options = parse(&["value.wf"]).expect("one source is a complete invocation");
        assert!(!options.par);

        let options = parse(&["--par-ledger", "value.wf"]).expect("the ledger option is accepted");
        assert!(!options.par, "reading the ledger must not enable lowering");

        let options = parse(&["--par", "value.wf"]).expect("the option is accepted");
        assert!(options.par);
        assert!(!options.par_ledger, "lanes are not a ledger request");
        assert_eq!(options.sources.len(), 1);
    }

    /// The sequential reference build is its own switch, off unless asked
    /// for, and it may not be written together with the switch that selects
    /// the opposite lowering.
    #[test]
    fn the_sequential_reference_build_is_requested_by_its_own_option() {
        let options = parse(&["value.wf"]).expect("one source is a complete invocation");
        assert!(!options.no_overlap, "the default build is the shipped one");

        let options = parse(&["--no-overlap", "value.wf"]).expect("the option is accepted");
        assert!(options.no_overlap);
        assert!(!options.par, "the reference build asks for no lanes");

        let message = parse(&["--par", "--no-overlap", "value.wf"])
            .err()
            .expect("opposite lowerings may not be written together");
        assert!(message.contains("--no-overlap"), "{message}");
        assert!(message.contains("--par"), "{message}");
    }

    /// The usage text is one definition, so the option list a reader is shown
    /// cannot drift from the option list the parser accepts.
    #[test]
    fn the_usage_text_lists_every_accepted_option() {
        for option in [
            "--emit-llvm",
            "--par",
            "--no-overlap",
            "--par-ledger",
            "--stack-ledger",
            "-o",
        ] {
            assert!(
                super::USAGE.contains(option),
                "usage text omits {option}: {}",
                super::USAGE
            );
            parse(&[option, "out.ll", "value.wf"])
                .expect("every option the usage text lists must parse");
        }
        let message = parse(&["--par-legder", "value.wf"])
            .err()
            .expect("a misspelled option must be refused");
        assert!(message.contains("unknown option"), "{message}");
    }

    /// One loop that publishes to standard output per iteration. The [PAR-3]
    /// staged judgment denies it on `&uniq 'say out`, which is storage
    /// carrying one position, and the denial is the same under every lowering
    /// because the judgment is pure.
    const DENIED_OUTPUT_LOOP: &[u8] = br#"command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  doc "Writes one line per iteration to standard output.";
  let page = buffer_new(8_u64, 0_u8);
  for @scan (index in 0_u64..4_u64) {
    let written = write_once(output: &uniq out, source: &page, start: 0_u64, end: 8_u64);
  }
  return exit_status(code: 0_u8);
}
"#;

    /// A denied I/O loop is news by default and under `--par`, and it is not
    /// news under `--no-overlap`.
    ///
    /// The third blind writer's program compiled, ran correctly, and printed
    /// the same `PAR` notes on every rebuild, including the builds that had
    /// already said they wanted no overlap at all. A writer who wrote
    /// `--no-overlap` has stated that this build is the sequential reference
    /// one, so a loop without a pipeline is the build they asked for rather
    /// than a missed optimization on it.
    ///
    /// Three things this pins beyond the flag. The judgment is pure, so all
    /// three ways reach the same verdicts and it is only the channel that
    /// closes. The three ways emit one module here, byte for byte, because a
    /// denied loop reaches the host through ordinary direct calls under every
    /// lowering — so the quiet costs the writer no information about the
    /// program they built. And `--par-ledger` still carries every line the
    /// quiet build withheld.
    #[test]
    fn a_no_overlap_build_reports_no_denied_io_loop() {
        let mut modules = Vec::new();
        let mut verdicts = Vec::new();
        for (arguments, reported) in [
            (vec!["value.wf"], true),
            (vec!["--par", "value.wf"], true),
            (vec!["--no-overlap", "value.wf"], false),
        ] {
            let options = parse(&arguments).expect("every invocation is complete");
            let (module, notices) = compile_with_io_notices(
                &[SourceInput::new("value.wf", DENIED_OUTPUT_LOOP)],
                CompilerLimits::default(),
                options.overlap(),
            )
            .expect("a denied loop is a note, never a rejection");
            assert!(
                notices.iter().any(|notice| notice.contains("denied")),
                "the judgment denies this loop under every lowering: {notices:?}"
            );
            let report = io_notice_report(options.no_overlap, &notices);
            if reported {
                assert_eq!(report.len(), notices.len() + 1, "{report:?}");
                assert!(
                    report
                        .iter()
                        .all(|line| line.starts_with("whitefootc: note: ")),
                    "{report:?}"
                );
                assert!(
                    report[0].contains("PAR ") && report[0].contains("denied"),
                    "{report:?}"
                );
                assert!(
                    report[report.len() - 1].contains("--par-ledger"),
                    "the closing line names the full report: {report:?}"
                );
            } else {
                assert!(report.is_empty(), "{report:?}");
            }
            modules.push(module);
            verdicts.push(notices);
        }
        assert!(
            verdicts.iter().all(|notices| *notices == verdicts[0]),
            "the judgment is pure, so the flag closes a channel and reaches no verdict"
        );
        assert!(
            modules.iter().all(|module| *module == modules[0]),
            "a denied loop compiles to one module under all three lowerings"
        );

        // The report itself is not the quiet channel: under the same
        // `--no-overlap` lowering `--par-ledger` still prints every line,
        // including the denials the quiet build withheld.
        let (_, ledger) = compile_with_permission_ledger(
            &[SourceInput::new("value.wf", DENIED_OUTPUT_LOOP)],
            CompilerLimits::default(),
            OverlapLowering::Off,
        )
        .expect("a denied loop is a note, never a rejection");
        assert!(
            verdicts[2].iter().all(|notice| ledger.contains(notice)),
            "the full report keeps what the quiet build did not print: {ledger:?}"
        );
    }
}
