use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use whitefoot::{
    CompilationFailure, CompilerLimits, HOST_LINK_LIBRARIES, HOST_OPTIMIZATION_ARGUMENTS,
    OverlapLowering, SourceInput, compile, compile_with_overlap, compile_with_permission_ledger,
};

use crate::support::append_runtime_objects;

static NEXT_EXECUTION: AtomicU64 = AtomicU64::new(0);

/// One invocation argument, from the bytes a case names to the host's own
/// argument value.
///
/// A case that means a byte sequence no other encoding can carry — the
/// wider-code-unit and byte-named cases — is a POSIX fact and says so through
/// the fixture helpers below. Every other case, the loopback ones included,
/// names ASCII text, and this is what keeps those cases from assuming the
/// host's arguments are bytes: on a family whose arguments are not, the bytes
/// are read as UTF-8 and handed over as text, which is the same argument.
#[cfg(unix)]
fn invocation_argument(bytes: &[u8]) -> OsString {
    OsStr::from_bytes(bytes).to_os_string()
}

#[cfg(not(unix))]
fn invocation_argument(bytes: &[u8]) -> OsString {
    OsString::from(
        std::str::from_utf8(bytes)
            .expect("an invocation argument this host can carry must be text"),
    )
}

/// Links one emitted module with the same ordinary library as the driver.
fn link_module(module: &Path, executable: &Path, llvm: &str, directory: &Path) {
    let mut command = Command::new("/usr/bin/clang");
    command.arg("-x").arg("ir").arg(module);
    command.arg("-pthread");
    let (sources, objects) = append_runtime_objects(&mut command, directory, None, None);
    let compilation = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .args(HOST_LINK_LIBRARIES)
        .arg("-o")
        .arg(executable)
        .output()
        .expect("invoke host clang");
    assert!(
        compilation.status.success(),
        "clang rejected emitted LLVM:\n{}\n{}",
        String::from_utf8_lossy(&compilation.stderr),
        llvm
    );
    for path in objects.into_iter().chain(sources) {
        std::fs::remove_file(path).expect("remove staged native build input");
    }
    for name in ["completion", "sched"] {
        std::fs::remove_dir(directory.join(name)).expect("remove staged native source directory");
    }
}

pub fn compile_program(name: &str) -> String {
    compile_programs(&[name])
}

/// Compiles the explicit sequential lowering used by paired corpus controls.
pub fn compile_program_without_overlap(name: &str) -> String {
    let source = read_program(name);
    compile_with_overlap(
        &[SourceInput::new(name, &source)],
        CompilerLimits::default(),
        OverlapLowering::Off,
    )
    .expect("sequential program corpus source must compile")
}

pub fn compile_programs(names: &[&str]) -> String {
    let sources = names
        .iter()
        .map(|name| read_program(name))
        .collect::<Vec<_>>();
    let inputs = names
        .iter()
        .zip(&sources)
        .map(|(name, source)| SourceInput::new(name, source))
        .collect::<Vec<_>>();
    compile(&inputs, CompilerLimits::default()).expect("program corpus source must compile")
}

/// [`compile_programs_with_overlap`] returning a compilation failure to the
/// caller. The complete corpus walk names the failing unit in its assertion;
/// every source or target failure fails that test.
pub fn try_compile_programs_with_overlap(names: &[&str]) -> Result<String, CompilationFailure> {
    let sources = names
        .iter()
        .map(|name| read_program(name))
        .collect::<Vec<_>>();
    let inputs = names
        .iter()
        .zip(&sources)
        .map(|(name, source)| SourceInput::new(name, source))
        .collect::<Vec<_>>();
    compile_with_overlap(&inputs, CompilerLimits::default(), OverlapLowering::On)
}

/// Compiles one corpus program with the [PAR-1 candidate] overlap lowering
/// switched on without scalar-leaf suppression (`--par-scalar-leaf-limit off`).
///
/// [`compile_program`] is the shipped default and hands nothing out, so a case
/// about actualization has to name this entry. The two differ in the emitted
/// lowering only: the judgment, the accepted program, and the ledger are the
/// same either way.
pub fn compile_program_with_overlap(name: &str) -> String {
    compile_programs_with_overlap(&[name])
}

/// [`compile_program_with_overlap`] over a corpus unit of several sources.
///
/// A program the corpus keeps as several files does not compile a file at a
/// time, so a case that asks what the whole corpus compiles to under `--par`
/// needs the same multi-source entry [`compile_programs`] gives the default
/// lowering.
pub fn compile_programs_with_overlap(names: &[&str]) -> String {
    try_compile_programs_with_overlap(names).expect("program corpus source must compile")
}

/// Every `.wf` file the program corpus holds, in one stable order.
///
/// Read from the directory rather than listed, so a case intended to cover
/// the corpus cannot quietly stop covering it when a program is added.
pub fn corpus_program_files() -> Vec<String> {
    let root = corpus_directory();
    let mut names = std::fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()))
                .path()
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "wf"))
        .map(|path| {
            path.file_name()
                .expect("a corpus file has a name")
                .to_str()
                .expect("a corpus file name is UTF-8")
                .to_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

/// Compiles one corpus program and returns its permission ledger lines.
///
/// The ledger is the developer-channel rendering of the permission judgment,
/// so a case that asks what the compiler decided about a corpus program reads
/// exactly the lines a developer would see rather than re-deriving them. The
/// judgment is pure, so these are the default compilation's lines and the
/// `--par` compilation's alike.
pub fn program_permission_ledger(name: &str) -> Vec<String> {
    let source = read_program(name);
    let inputs = [SourceInput::new(name, &source)];
    let (_, ledger) =
        compile_with_permission_ledger(&inputs, CompilerLimits::default(), OverlapLowering::Off)
            .expect("program corpus source must compile");
    ledger
}

pub fn compile_sources(sources: &[(&str, &[u8])]) -> String {
    let inputs = sources
        .iter()
        .map(|(name, source)| SourceInput::new(name, source))
        .collect::<Vec<_>>();
    compile(&inputs, CompilerLimits::default()).expect("integration source must compile")
}

/// Compiles sources that must be rejected and returns the rendered failure.
///
/// A negative direction over a real corpus program needs the compiler's own
/// diagnostic, not a panic, so the case can pin the rule and the residual.
pub fn compile_rejection(sources: &[(&str, &[u8])]) -> String {
    let inputs = sources
        .iter()
        .map(|(name, source)| SourceInput::new(name, source))
        .collect::<Vec<_>>();
    match compile(&inputs, CompilerLimits::default()) {
        Ok(_) => panic!("source that must be rejected compiled"),
        Err(failure) => failure.to_string(),
    }
}

pub fn compile_and_run(llvm: &str) -> Output {
    let sequence = NEXT_EXECUTION.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "whitefoot-integration-test-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create unique integration-test directory");
    let module = directory.join("program.ll");
    let executable = directory.join("program");
    std::fs::write(&module, llvm).expect("write integration-test module");
    link_module(&module, &executable, llvm, &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run integration-test executable");
    std::fs::remove_file(&executable).expect("remove integration-test executable");
    std::fs::remove_file(&module).expect("remove integration-test module");
    std::fs::remove_dir(&directory).expect("remove integration-test directory");
    output
}

/// Links one emitted module against the parallel runtime plus an observer that
/// reports the runtime's own grant count at process exit, then runs it once at
/// `workers`.
///
/// A corpus case that asks whether a permitted overlap was *actualized* cannot
/// read that from the published bytes, because a runtime that refused every
/// lane publishes the same bytes — that is the whole point of the permission.
/// The observer reads `wf__par_grants`, which no Whitefoot construct can name,
/// so "a lane was granted" is read rather than assumed. It mirrors the in-crate
/// runtime case's own harness; the corpus needs its own because the program
/// under test is a corpus file rather than an inline fixture.
///
/// The count reaches the destructor only for a program that exits normally;
/// an abnormal process stop runs no destructor. Every corpus program this is
/// used on exits normally.
pub fn run_counting_grants(llvm: &str, workers: Option<&str>) -> (u64, Output) {
    let sequence = NEXT_EXECUTION.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "whitefoot-grants-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create unique grant-count directory");
    let module = directory.join("counted.ll");
    let observer = directory.join("observer.c");
    let executable = directory.join("counted");
    std::fs::write(&module, llvm).expect("write the module");
    std::fs::write(
        &observer,
        "#include <stdio.h>\nextern unsigned long wf__par_grants(void);\n__attribute__((destructor)) static void wf__par_report(void) {\n    fprintf(stderr, \"grants=%lu\\n\", wf__par_grants());\n}\n",
    )
    .expect("write the observer");
    let mut command = Command::new("/usr/bin/clang");
    command
        .arg("-std=c11")
        .arg("-pthread")
        .arg("-x")
        .arg("ir")
        .arg(&module);
    // Keep the observer fresh and in its original position after the floor.
    let (_sources, objects) =
        append_runtime_objects(&mut command, &directory, Some("c11"), Some(&observer));
    let linked = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .args(HOST_LINK_LIBRARIES)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "the runtime and its observer must link:\n{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    for object in objects {
        std::fs::remove_file(object).expect("remove materialized native library object");
    }
    let mut command = Command::new(&executable);
    command.current_dir(&directory);
    match workers {
        Some(count) => command.env("WF_WORKERS", count),
        None => command.env_remove("WF_WORKERS"),
    };
    let output = command.output().expect("run the counted program");
    let report = String::from_utf8_lossy(&output.stderr).into_owned();
    let granted = report
        .lines()
        .find_map(|line| line.strip_prefix("grants="))
        .and_then(|count| count.trim().parse::<u64>().ok())
        .unwrap_or_else(|| panic!("the observer must report a grant count, got {report:?}"));
    std::fs::remove_dir_all(&directory).expect("remove the grant-count directory");
    (granted, output)
}

/// One built executable that a case invokes repeatedly.
///
/// An ordinary entry receives arguments and a directory through `Inputs`,
/// so a case needs one executable it can invoke many times
/// with different arguments and working directories, rather than the single
/// argument-free run [`compile_and_run`] performs.
pub struct CompiledProgram {
    directory: PathBuf,
    executable: PathBuf,
}

pub fn build_program(llvm: &str) -> CompiledProgram {
    let sequence = NEXT_EXECUTION.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "whitefoot-program-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create unique program directory");
    let module = directory.join("program.ll");
    let executable = directory.join("program");
    std::fs::write(&module, llvm).expect("write program module");
    link_module(&module, &executable, llvm, &directory);
    CompiledProgram {
        directory,
        executable,
    }
}

impl CompiledProgram {
    /// Runs the program in `working_directory` with `arguments` as argv[1..].
    ///
    /// Arguments are raw bytes, because the program reads them through the
    /// lossless host-string route and a case must be able to supply an
    /// argument that is not valid UTF-8.
    pub fn run(&self, working_directory: &Path, arguments: &[&[u8]]) -> Output {
        Command::new(&self.executable)
            .current_dir(working_directory)
            .args(arguments.iter().map(|bytes| invocation_argument(bytes)))
            .output()
            .expect("run compiled program")
    }

    /// Runs the program with the runtime's worker setting named explicitly.
    ///
    /// `workers` is `None` for the shipped default — the variable unset, which
    /// is every other case in this corpus — and `Some(count)` for a run that
    /// names a count. A case that compares the two needs both spellings from
    /// one built executable, because the difference under test is the execution
    /// and not the program.
    ///
    /// In a `--par` build the default is a pool sized to the machine, so
    /// `None` is a parallel execution and `Some("1")` is the sequential one. In
    /// a default build no runtime is linked and neither spelling reaches
    /// anything.
    pub fn run_with_workers(&self, workers: Option<&str>) -> Output {
        self.run_with_workers_and_arguments(workers, &[])
    }

    /// Runs with an explicit worker setting and raw invocation arguments.
    ///
    /// This is the argument-bearing counterpart to [`Self::run_with_workers`]:
    /// it keeps the runtime environment under test control while allowing a
    /// program to select a larger deterministic workload through `Inputs.args`.
    pub fn run_with_workers_and_arguments(
        &self,
        workers: Option<&str>,
        arguments: &[&[u8]],
    ) -> Output {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.directory)
            .args(arguments.iter().map(|bytes| invocation_argument(bytes)));
        match workers {
            Some(count) => command.env("WF_WORKERS", count),
            None => command.env_remove("WF_WORKERS"),
        };
        command.output().expect("run compiled program")
    }

    /// Starts the program on one runtime route with raw invocation arguments,
    /// and hands back the running child.
    ///
    /// A loopback case has to play the peer while the program runs, so it
    /// needs the child rather than the finished output: the program is a
    /// server the case connects to, or a client the case accepts from, and
    /// either way both sides are alive at once. `native_ring` selects the
    /// route exactly as the standard-input cases do — `true` is the shipped
    /// default, `false` sets `WF_IO_NO_NATIVE_RING` so the same program runs
    /// through the shared file adapter instead of the kernel completion ring.
    pub fn spawn_on_route(&self, native_ring: bool, arguments: &[&[u8]]) -> Child {
        self.spawn_on_route_with_workers(native_ring, None, arguments)
    }

    /// Starts the program on one runtime route with the worker count named.
    ///
    /// A case whose property is about several peers being served at once has
    /// to state the pool it is served by, because the shipped default sizes it
    /// to the machine: a host with many cores serves four peers on four
    /// workers whatever the runtime does with a wait, so the property would be
    /// proved by the runner rather than by the program. `workers` is `None`
    /// for that default and `Some(count)` for a case that pins it.
    pub fn spawn_on_route_with_workers(
        &self,
        native_ring: bool,
        workers: Option<&str>,
        arguments: &[&[u8]],
    ) -> Child {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.directory)
            .args(arguments.iter().map(|bytes| invocation_argument(bytes)))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if native_ring {
            command.env_remove("WF_IO_NO_NATIVE_RING");
        } else {
            command.env("WF_IO_NO_NATIVE_RING", "1");
        }
        match workers {
            Some(count) => command.env("WF_WORKERS", count),
            None => command.env_remove("WF_WORKERS"),
        };
        command.spawn().expect("spawn compiled program")
    }

    /// Runs the program with standard output on a pipe whose read end this
    /// process closes before consuming anything.
    ///
    /// The program therefore publishes into a destination with no reader,
    /// which is the portable way to observe what a write to a closed pipe
    /// reaches source as. Standard error is captured and returned.
    pub fn run_with_closed_output(
        &self,
        working_directory: &Path,
        arguments: &[&[u8]],
    ) -> (ExitStatus, Vec<u8>) {
        // The destination must have no reader from the program's first write
        // on: closing the read end after `spawn` races the child, and a child
        // that publishes before the close succeeds and exits 0 (observed on a
        // three-core CI runner). So the pipe is made here and its read end is
        // closed before the child exists.
        let (reader, writer) = std::io::pipe().expect("create the closed destination");
        drop(reader);
        let mut child = Command::new(&self.executable)
            .current_dir(working_directory)
            .args(arguments.iter().map(|bytes| invocation_argument(bytes)))
            .stdout(Stdio::from(writer))
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compiled program");
        let mut diagnostics = Vec::new();
        child
            .stderr
            .take()
            .expect("piped standard error")
            .read_to_end(&mut diagnostics)
            .expect("read the program's diagnostics");
        let status = child.wait().expect("wait for compiled program");
        (status, diagnostics)
    }

    /// Runs the program with its standard input redirected from a pipe this
    /// process fills with `bytes` and then closes.
    ///
    /// This is one of the two standard-input implementations exercised here: a
    /// stream whose end the writer decides, where a read may return less than
    /// the requested range and the end arrives only when the writer closes.
    /// `native_ring` selects the runtime route — `true` is the shipped
    /// default, `false` sets `WF_IO_NO_NATIVE_RING` so the same program runs
    /// through the shared file adapter instead of the kernel completion ring.
    pub fn run_with_piped_input(&self, bytes: &[u8], native_ring: bool) -> Output {
        let (reader, mut writer) = std::io::pipe().expect("create the input pipe");
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.directory)
            .stdin(Stdio::from(reader))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if native_ring {
            command.env_remove("WF_IO_NO_NATIVE_RING");
        } else {
            command.env("WF_IO_NO_NATIVE_RING", "1");
        }
        let child = command.spawn().expect("spawn compiled program");
        // The writer is closed before the child is waited on, or a program
        // that reads to end would never observe one.
        writer.write_all(bytes).expect("fill the input pipe");
        drop(writer);
        child.wait_with_output().expect("wait for compiled program")
    }

    /// Runs the program with its standard input redirected from a regular
    /// file holding `bytes`.
    ///
    /// This is the other implementation exercised here, and a different runtime
    /// path on Linux: a regular file's descriptor is one the kernel ring
    /// completes without any readiness wait, while a pipe's is not.
    pub fn run_with_file_input(&self, bytes: &[u8], native_ring: bool) -> Output {
        let path = self.directory.join("standard-input");
        std::fs::write(&path, bytes).expect("write the input fixture");
        let file = std::fs::File::open(&path).expect("open the input fixture");
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.directory)
            .stdin(Stdio::from(file))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if native_ring {
            command.env_remove("WF_IO_NO_NATIVE_RING");
        } else {
            command.env("WF_IO_NO_NATIVE_RING", "1");
        }
        command.output().expect("run compiled program")
    }
}

impl Drop for CompiledProgram {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

/// One directory whose complete content a case fixes.
pub struct FixtureDirectory {
    path: PathBuf,
}

pub fn fixture_directory() -> FixtureDirectory {
    let sequence = NEXT_EXECUTION.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "whitefoot-fixtures-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&path).expect("create unique fixture directory");
    FixtureDirectory { path }
}

impl FixtureDirectory {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes one fixture file, whose name may be any byte sequence.
    ///
    /// This one names POSIX on purpose and is not an assumption the harness
    /// makes elsewhere: a name that is any byte sequence is a fact about a
    /// POSIX file system, and it is the subject of the cases that call it.
    /// No loopback case reaches it.
    pub fn write(&self, name: &[u8], bytes: &[u8]) -> PathBuf {
        let path = self.path.join(OsStr::from_bytes(name));
        std::fs::write(&path, bytes).expect("write fixture file");
        path
    }
}

/// The tree shapes only a directory-walking case builds.
impl FixtureDirectory {
    /// Creates one nested fixture directory and returns its path.
    ///
    /// A traversal case needs a real directory tree under the invocation
    /// directory, because the program walks it with the host's own
    /// enumeration facility rather than with anything the harness injects.
    pub fn directory(&self, relative: &str) -> PathBuf {
        let path = self.path.join(relative);
        std::fs::create_dir_all(&path).expect("create nested fixture directory");
        path
    }

    /// Writes one fixture file at a relative path, creating its parents.
    pub fn write_nested(&self, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create nested fixture parent");
        }
        std::fs::write(&path, bytes).expect("write nested fixture file");
        path
    }

    /// Places a real symbolic link at `name` pointing at `target`.
    pub fn symlink(&self, name: &str, target: &Path) {
        std::os::unix::fs::symlink(target, self.path.join(name)).expect("create fixture symlink");
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Denies every mode bit on `path`, and confirms the denial actually reaches
/// this process before the case relies on it.
///
/// The cases that call this describe a path the walking program cannot open.
/// Mode bits do not deny a process holding `CAP_DAC_OVERRIDE`, which uid 0
/// carries by default, so a container that runs the gate as root cannot
/// construct that scenario at all: the walk reads the path it was told was
/// closed, and the case then fails on its output comparison, which reads like a
/// traversal defect and is not one. Checking the denial where it is established
/// names the cause instead, and covers the opposite risk too — without the
/// check, a case could report success in an environment where the path it
/// describes was never closed.
///
/// This is not a skip. The case still fails here, because the behavior it
/// covers genuinely went unverified.
pub fn close_path(path: &Path) {
    set_fixture_mode(path, 0o000);
    let still_reaches = if path.is_dir() {
        std::fs::read_dir(path).is_ok()
    } else {
        std::fs::File::open(path).is_ok()
    };
    assert!(
        !still_reaches,
        "{} is mode 000 and this process still reads it, so the denied-path \
         case cannot be constructed here. A process holding CAP_DAC_OVERRIDE, \
         which uid 0 carries by default, is not denied by mode bits. Re-run \
         this case as an unprivileged user.",
        path.display()
    );
}

/// Restores `mode` on a path closed by `close_path`.
pub fn reopen_path(path: &Path, mode: u32) {
    set_fixture_mode(path, mode);
}

fn set_fixture_mode(path: &Path, mode: u32) {
    let mut permissions = std::fs::metadata(path)
        .expect("fixture path metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, mode);
    std::fs::set_permissions(path, permissions).expect("set fixture path mode");
}

/// Extracts the exact source symbol's definition, independent of its linkage.
/// Ordinary callable definitions use public linkage under the shared ABI;
/// declarations and calls to the same symbol must not count as definitions.
pub fn emitted_function<'module>(module: &'module str, name: &str) -> &'module str {
    let symbol = format!(" @wf_{name}(");
    let function_start = module
        .match_indices(&symbol)
        .find_map(|(symbol_start, _)| {
            let line_start = module[..symbol_start]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line_start..symbol_start]
                .starts_with("define ")
                .then_some(line_start)
        })
        .unwrap_or_else(|| panic!("missing emitted function {name}"));
    let function_end = module[function_start..]
        .find("\n}\n\n")
        .map(|offset| function_start + offset + 3)
        .expect("source function definition must close");
    &module[function_start..function_end]
}

#[test]
fn emitted_function_selects_exact_definitions_across_ordinary_linkages() {
    for linkage in ["", "internal "] {
        let definition = format!(
            "define {linkage}i64 @wf_selected(i64 %value) {{\nentry:\n  ret i64 %value\n}}\n"
        );
        let module = format!(
            "declare i64 @wf_selected(i64)\n\ndefine i64 @wf_other() {{\nentry:\n  %result = call i64 @wf_selected(i64 1)\n  ret i64 %result\n}}\n\ndefine i64 @wf_selected_suffix() {{\nentry:\n  ret i64 0\n}}\n\n{definition}\n"
        );
        assert_eq!(emitted_function(&module, "selected"), definition);
    }
}

fn read_program(name: &str) -> Vec<u8> {
    let path = corpus_directory().join(name);
    std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "cannot read program corpus file {}: {error}",
            path.display()
        )
    })
}

fn corpus_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler package must live directly under the repository root")
        .join("tests")
        .join("programs")
}
