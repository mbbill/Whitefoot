use std::ffi::OsStr;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use whitefoot::{
    CompilerLimits, HOST_OPTIMIZATION_ARGUMENTS, SourceInput, compile,
    compile_with_traversal_surface,
};

static NEXT_EXECUTION: AtomicU64 = AtomicU64::new(0);

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

/// Compiles one corpus program against the traversal [SYS-2] inventory,
/// named explicitly rather than read from `TRAVERSAL_SURFACE`.
///
/// The shipped inventory is the traversal one, so this selects the same
/// inventory [`compile_program`] does; naming it keeps the traversal cases
/// stating which inventory their declarations belong to, and keeps the
/// inventory a parameter of the compilation rather than a global.
pub fn compile_program_with_traversal_surface(name: &str) -> String {
    let source = read_program(name);
    let inputs = [SourceInput::new(name, &source)];
    compile_with_traversal_surface(&inputs, CompilerLimits::default(), true)
        .expect("traversal program source must compile under the candidate inventory")
}

/// Compiles one corpus program against the base [SYS-2] inventory: the
/// tables without the traversal rows appended.
///
/// This is the other side of every inventory differential — the traversal
/// spellings are undeclared names here, and every earlier program must keep
/// its exact emitted module.
pub fn compile_program_without_traversal_surface(name: &str) -> String {
    let source = read_program(name);
    let inputs = [SourceInput::new(name, &source)];
    compile_with_traversal_surface(&inputs, CompilerLimits::default(), false)
        .expect("program corpus source must compile under the base inventory")
}

/// [`compile_program_without_traversal_surface`]'s rejection direction.
pub fn compile_program_rejection_without_traversal_surface(name: &str) -> String {
    let source = read_program(name);
    let inputs = [SourceInput::new(name, &source)];
    match compile_with_traversal_surface(&inputs, CompilerLimits::default(), false) {
        Ok(_) => panic!("source that must be rejected compiled"),
        Err(failure) => failure.to_string(),
    }
}

/// [`compile_program_with_traversal_surface`]'s rejection direction.
pub fn compile_rejection_with_traversal_surface(sources: &[(&str, &[u8])]) -> String {
    let inputs = sources
        .iter()
        .map(|(name, source)| SourceInput::new(name, source))
        .collect::<Vec<_>>();
    match compile_with_traversal_surface(&inputs, CompilerLimits::default(), true) {
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
    let compilation = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg(&module)
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        compilation.status.success(),
        "clang rejected emitted LLVM:\n{}\n{}",
        String::from_utf8_lossy(&compilation.stderr),
        llvm
    );
    let output = Command::new(&executable)
        .output()
        .expect("run integration-test executable");
    std::fs::remove_file(&executable).expect("remove integration-test executable");
    std::fs::remove_file(&module).expect("remove integration-test module");
    std::fs::remove_dir(&directory).expect("remove integration-test directory");
    output
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
    let compilation = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg(&module)
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        compilation.status.success(),
        "clang rejected emitted LLVM:\n{}\n{}",
        String::from_utf8_lossy(&compilation.stderr),
        llvm
    );
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
