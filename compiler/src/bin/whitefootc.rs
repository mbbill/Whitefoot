#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use whitefoot::{
    CompilerLimits, FLOOR_RUNTIME_SOURCE, FLOOR_STACK_BYTES, HOST_OPTIMIZATION_ARGUMENTS,
    OverlapLowering, PARALLEL_RUNTIME_SOURCE, SourceInput, compile_with_overlap,
    compile_with_permission_ledger, module_requires_parallel_runtime, stack_ledger,
};

const USAGE: &str =
    "usage: whitefootc [--emit-llvm] [--par] [--par-ledger] [--stack-ledger] [-o OUTPUT] SOURCE...";

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
        compile_with_overlap(&inputs, CompilerLimits::default(), overlap)
            .map_err(|failure| failure.to_string())?
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
        let status = Command::new("/usr/bin/clang")
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
            .map_err(|error| format!("cannot start /usr/bin/clang: {error}"))?;
        if !status.success() {
            return Err(format!("clang exited with {status}"));
        }
        let usage = std::fs::read_to_string(directory.join("module.su"))
            .map_err(|error| format!("cannot read the stack-usage report: {error}"))?;
        let text = std::fs::read_to_string(&assembly)
            .map_err(|error| format!("cannot read the ledger assembly: {error}"))?;
        Ok(stack_ledger(&usage, &text, FLOOR_STACK_BYTES))
    })();
    let _ = std::fs::remove_dir_all(&directory);
    result
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
    // The floor joins unconditionally, because every program can exhaust its
    // stack. It travels the same way and for the same reason: its bytes are
    // the compiler's, so what a program does when it runs out is decided here
    // and not by anything installed on the machine.
    let floor = std::env::temp_dir().join(format!("whitefootc-floor-{}.c", std::process::id()));
    std::fs::write(&floor, FLOOR_RUNTIME_SOURCE)
        .map_err(|error| format!("cannot write the floor runtime: {error}"))?;
    let mut command = Command::new("/usr/bin/clang");
    command.arg("-pthread").arg("-x").arg("c").arg(&floor);
    if let Some(path) = runtime.as_ref() {
        command.arg("-x").arg("c").arg(path);
    }
    let outcome = link(&mut command, llvm, output);
    let _ = std::fs::remove_file(floor);
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
    /// Off by default, and free when it is on and no pool is asked for.
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
    /// existed, with one world in it. `WF_WORKERS` remains the runtime knob
    /// for a program built this way: absent it asks for one lane per logical
    /// CPU, which is what a binary handed to somebody gets, and `0`, `1`, or an
    /// unparsable value is the opt-out that starts no pool at all.
    par: bool,
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
        let mut par_ledger = false;
        let mut stack_ledger = false;
        let mut output = None;
        let mut sources = Vec::new();
        let mut cursor = 0;
        while cursor < arguments.len() {
            match arguments[cursor].as_str() {
                "--emit-llvm" => emit_llvm = true,
                "--par" => par = true,
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
        Ok(Self {
            emit_llvm,
            par,
            par_ledger,
            stack_ledger,
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

    /// The usage text is one definition, so the option list a reader is shown
    /// cannot drift from the option list the parser accepts.
    #[test]
    fn the_usage_text_lists_every_accepted_option() {
        for option in [
            "--emit-llvm",
            "--par",
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
}
