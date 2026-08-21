#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use whitefoot::{
    CompilerLimits, HOST_OPTIMIZATION_ARGUMENTS, OverlapLowering, PARALLEL_RUNTIME_SOURCE,
    SourceInput, compile_with_overlap, compile_with_permission_ledger,
    module_requires_parallel_runtime,
};

const USAGE: &str = "usage: whitefootc [--emit-llvm] [--par] [--par-ledger] [-o OUTPUT] SOURCE...";

fn main() {
    if let Err(message) = run() {
        eprintln!("whitefootc: {message}");
        std::process::exit(1);
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
        paths.push(logical_path(source, index));
    }
    let inputs: Vec<_> = paths
        .iter()
        .zip(&bytes)
        .map(|(path, bytes)| SourceInput::new(path, bytes))
        .collect();
    let overlap = if options.par {
        OverlapLowering::On
    } else {
        OverlapLowering::Off
    };
    let module = if options.par_ledger {
        // The permission ledger is developer output. It goes to stdout, which
        // `Options::parse` has already kept clear of the emitted module, and
        // never to the mandatory record channel. It reports the same judgment
        // with or without `--par`; only the emitted lowering differs.
        let (module, ledger) =
            compile_with_permission_ledger(&inputs, CompilerLimits::default(), overlap)
                .map_err(|failure| failure.to_string())?;
        for line in &ledger {
            println!("{line}");
        }
        module
    } else {
        compile_with_overlap(&inputs, CompilerLimits::default(), overlap)
            .map_err(|failure| failure.to_string())?
    };
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

fn compile_executable(llvm: &str, output: &Path) -> Result<(), String> {
    // The parallel runtime joins the link only when the module hands work to
    // it. Its bytes travel inside this executable, so no installed path, no
    // build directory, and no environment decides which runtime a program
    // gets.
    let runtime = if module_requires_parallel_runtime(llvm) {
        let path = std::env::temp_dir().join(format!("whitefootc-par-{}.c", std::process::id()));
        std::fs::write(&path, PARALLEL_RUNTIME_SOURCE)
            .map_err(|error| format!("cannot write the parallel runtime: {error}"))?;
        Some(path)
    } else {
        None
    };
    let mut command = Command::new("/usr/bin/clang");
    if let Some(path) = runtime.as_ref() {
        command.arg("-pthread").arg("-x").arg("c").arg(path);
    }
    let outcome = link(&mut command, llvm, output);
    if let Some(path) = runtime {
        let _ = std::fs::remove_file(path);
    }
    outcome
}

fn link(command: &mut Command, llvm: &str, output: &Path) -> Result<(), String> {
    let mut child = command
        .arg("-x")
        .arg("ir")
        .arg("-")
        .arg("-Wno-override-module")
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start /usr/bin/clang: {error}"))?;
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
    /// Off by default. Outlining a call is not free — it passes its arguments
    /// through a memory frame and is reached through a function pointer, so it
    /// cannot be inlined — and the measured cost with no runtime linked is
    /// about 1.2x on a heavy recursive fold and 2.1x on `fib(38)`. The
    /// permission is never an obligation, so the default compilation takes
    /// none of it and emits exactly the module it emitted before this path
    /// existed. `WF_WORKERS` remains the runtime knob for a program built
    /// this way.
    par: bool,
    /// Print the non-normative permission ledger on stdout.
    par_ledger: bool,
    output: Option<PathBuf>,
    sources: Vec<PathBuf>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut emit_llvm = false;
        let mut par = false;
        let mut par_ledger = false;
        let mut output = None;
        let mut sources = Vec::new();
        let mut cursor = 0;
        while cursor < arguments.len() {
            match arguments[cursor].as_str() {
                "--emit-llvm" => emit_llvm = true,
                "--par" => par = true,
                "--par-ledger" => par_ledger = true,
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
        Ok(Self {
            emit_llvm,
            par,
            par_ledger,
            output,
            sources,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    fn parse(arguments: &[&str]) -> Result<Options, String> {
        let owned: Vec<String> = arguments.iter().map(|value| (*value).to_owned()).collect();
        Options::parse(&owned)
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

    /// The usage text is one definition, so the option list a reader is shown
    /// cannot drift from the option list the parser accepts.
    #[test]
    fn the_usage_text_lists_every_accepted_option() {
        for option in ["--emit-llvm", "--par", "--par-ledger", "-o"] {
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
}
