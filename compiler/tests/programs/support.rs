use std::ffi::OsStr;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use whitefoot::{
    CompilerLimits, HOST_OPTIMIZATION_ARGUMENTS, Inventory, OverlapLowering,
    PARALLEL_RUNTIME_SOURCE, SourceInput, compile, compile_with_inventory, compile_with_overlap,
    compile_with_permission_ledger, module_requires_parallel_runtime,
};

static NEXT_EXECUTION: AtomicU64 = AtomicU64::new(0);

/// Links one emitted module into `executable`, adding the parallel runtime on
/// exactly the condition the driver uses.
///
/// One definition serves both program-corpus link paths, so a program that
/// overlaps nothing links nothing extra and no path can forget the runtime a
/// module actually calls.
fn link_module(module: &Path, executable: &Path, llvm: &str, directory: &Path) {
    let mut command = Command::new("/usr/bin/clang");
    command.arg("-x").arg("ir").arg(module);
    let parallel_unit = module_requires_parallel_runtime(llvm).then(|| {
        let path = directory.join("par_runtime.c");
        std::fs::write(&path, PARALLEL_RUNTIME_SOURCE).expect("write the parallel runtime");
        command.arg("-pthread").arg("-x").arg("c").arg(&path);
        path
    });
    let compilation = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
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
    if let Some(path) = parallel_unit {
        std::fs::remove_file(path).expect("remove the parallel runtime unit");
    }
}

pub fn compile_program(name: &str) -> String {
    compile_programs(&[name])
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

/// Compiles one corpus program with the [PAR-1 candidate] overlap lowering
/// switched on, which is what `whitefootc --par` compiles.
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
        .expect("program corpus source must compile")
}

/// Every `.wf` file the program corpus holds, in one stable order.
///
/// Read from the directory rather than listed, so a case that claims to cover
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

/// Compiles one corpus program against one named [SYS-2] inventory state.
///
/// Naming the inventory keeps each case stating which inventory its
/// declarations belong to, and keeps the inventory a parameter of the
/// compilation rather than a global.
pub fn compile_program_with(name: &str, inventory: Inventory) -> String {
    let source = read_program(name);
    let inputs = [SourceInput::new(name, &source)];
    compile_with_inventory(&inputs, CompilerLimits::default(), inventory)
        .unwrap_or_else(|failure| panic!("program corpus source must compile: {failure}"))
}

/// Compiles one corpus program against the traversal [SYS-2] inventory,
/// named explicitly rather than read from `Inventory::ACTIVE`.
///
/// The shipped inventory is the traversal one, so this selects the same
/// inventory [`compile_program`] does.
pub fn compile_program_with_traversal_surface(name: &str) -> String {
    compile_program_with(name, Inventory::Traversal)
}

/// Compiles one corpus program against the active v0.33 file-open-by-name
/// inventory: the traversal tables with `open_file` appended.
pub fn compile_program_with_open_by_name(name: &str) -> String {
    compile_program_with(name, Inventory::OpenByName)
}

/// [`compile_program_with`]'s rejection direction.
pub fn compile_program_rejection_with(name: &str, inventory: Inventory) -> String {
    let source = read_program(name);
    let inputs = [SourceInput::new(name, &source)];
    match compile_with_inventory(&inputs, CompilerLimits::default(), inventory) {
        Ok(_) => panic!("source that must be rejected compiled"),
        Err(failure) => failure.to_string(),
    }
}

/// Rejection direction against the base tables without traversal rows.
pub fn compile_program_rejection_without_traversal_surface(name: &str) -> String {
    compile_program_rejection_with(name, Inventory::Base)
}

/// [`compile_program_with_traversal_surface`]'s rejection direction.
pub fn compile_rejection_with_traversal_surface(sources: &[(&str, &[u8])]) -> String {
    let inputs = sources
        .iter()
        .map(|(name, source)| SourceInput::new(name, source))
        .collect::<Vec<_>>();
    match compile_with_inventory(&inputs, CompilerLimits::default(), Inventory::Traversal) {
        Ok(_) => panic!("source that must be rejected compiled"),
        Err(failure) => failure.to_string(),
    }
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
/// The count reaches the destructor only for a program that exits normally: a
/// trap aborts and runs no destructor. Every corpus program this is used on
/// exits normally.
pub fn run_counting_grants(llvm: &str, workers: Option<&str>) -> (u64, Output) {
    let sequence = NEXT_EXECUTION.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "whitefoot-grants-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create unique grant-count directory");
    let module = directory.join("counted.ll");
    let runtime = directory.join("par_runtime.c");
    let observer = directory.join("observer.c");
    let executable = directory.join("counted");
    std::fs::write(&module, llvm).expect("write the module");
    std::fs::write(&runtime, PARALLEL_RUNTIME_SOURCE).expect("write the parallel runtime");
    std::fs::write(
        &observer,
        "#include <stdio.h>\nextern unsigned long wf__par_grants;\n__attribute__((destructor)) static void wf__par_report(void) {\n    fprintf(stderr, \"grants=%lu\\n\", wf__par_grants);\n}\n",
    )
    .expect("write the observer");
    let linked = Command::new("/usr/bin/clang")
        .arg("-pthread")
        .arg("-x")
        .arg("ir")
        .arg(&module)
        .arg("-x")
        .arg("c")
        .arg(&runtime)
        .arg(&observer)
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "the runtime and its observer must link:\n{}",
        String::from_utf8_lossy(&linked.stderr)
    );
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
/// A command-entry program takes its input from `command.args` and
/// `command.cwd`, so a case needs one executable it can invoke many times
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
    /// Arguments are raw bytes, because a command entry reads them through the
    /// lossless host-string route and a case must be able to supply an
    /// argument that is not valid UTF-8.
    pub fn run(&self, working_directory: &Path, arguments: &[&[u8]]) -> Output {
        Command::new(&self.executable)
            .current_dir(working_directory)
            .args(arguments.iter().map(|bytes| OsStr::from_bytes(bytes)))
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
        let mut command = Command::new(&self.executable);
        command.current_dir(&self.directory);
        match workers {
            Some(count) => command.env("WF_WORKERS", count),
            None => command.env_remove("WF_WORKERS"),
        };
        command.output().expect("run compiled program")
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
        let mut child = Command::new(&self.executable)
            .current_dir(working_directory)
            .args(arguments.iter().map(|bytes| OsStr::from_bytes(bytes)))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compiled program");
        drop(child.stdout.take());
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
    pub fn write(&self, name: &[u8], bytes: &[u8]) -> PathBuf {
        let path = self.path.join(OsStr::from_bytes(name));
        std::fs::write(&path, bytes).expect("write fixture file");
        path
    }

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

pub fn emitted_function<'module>(module: &'module str, name: &str) -> &'module str {
    let symbol = format!(" @wf_{name}(");
    let function_start = module
        .match_indices(&symbol)
        .find_map(|(symbol_start, _)| {
            let line_start = module[..symbol_start]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line_start..symbol_start]
                .starts_with("define internal")
                .then_some(line_start)
        })
        .unwrap_or_else(|| panic!("missing emitted function {name}"));
    let function_end = module[function_start..]
        .find("\n}\n\n")
        .map(|offset| function_start + offset + 3)
        .expect("source function definition must close");
    &module[function_start..function_end]
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
