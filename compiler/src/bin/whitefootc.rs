#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use whitefoot::{CompilerLimits, SourceInput, compile_v0_15};

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
    let module =
        compile_v0_15(&inputs, CompilerLimits::default()).map_err(|failure| failure.to_string())?;
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
    let mut child = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg("-")
        .arg("-Wno-override-module")
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
    output: Option<PathBuf>,
    sources: Vec<PathBuf>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut emit_llvm = false;
        let mut output = None;
        let mut sources = Vec::new();
        let mut cursor = 0;
        while cursor < arguments.len() {
            match arguments[cursor].as_str() {
                "--emit-llvm" => emit_llvm = true,
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
                    return Err("usage: whitefootc [--emit-llvm] [-o OUTPUT] SOURCE...".to_owned());
                }
                argument if argument.starts_with('-') => {
                    return Err(format!("unknown option: {argument}"));
                }
                source => sources.push(PathBuf::from(source)),
            }
            cursor += 1;
        }
        if sources.is_empty() {
            return Err("usage: whitefootc [--emit-llvm] [-o OUTPUT] SOURCE...".to_owned());
        }
        Ok(Self {
            emit_llvm,
            output,
            sources,
        })
    }
}
