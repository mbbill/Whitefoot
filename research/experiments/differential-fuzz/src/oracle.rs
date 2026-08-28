//! The oracle: what it means for one generated program to pass.
//!
//! [PAR-1], [PAR-2], and [PAR-3] all state the same guarantee in the same
//! words -- under a permitted overlap, bindings and every Whitefoot state place
//! equal the source-order result, and whether an overlap happened at all is not
//! observable. So the reference is not a model of the language: it is the same
//! program compiled with `--no-overlap`, whose execution *is* the source order.
//! Everything else must publish the same bytes.
//!
//! Two guards keep a difference honest. Before the program is used as an
//! oracle its sequential build is run twice and must agree, so a program that
//! is its own source of nondeterminism is discarded rather than reported. And
//! before a difference is recorded, the reference and the differing
//! configuration are each re-run several times, so one flake is not a finding
//! and a genuinely unstable reference is reclassified.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lowering {
    /// `--no-overlap`: the source-order reference.
    Sequential,
    /// The shipped default: completion overlap.
    Completion,
    /// `--par`: the actualizing parallel lowering.
    Parallel,
}

impl Lowering {
    pub fn flag(self) -> Option<&'static str> {
        match self {
            Lowering::Sequential => Some("--no-overlap"),
            Lowering::Completion => None,
            Lowering::Parallel => Some("--par"),
        }
    }

    pub fn spelling(self) -> &'static str {
        match self {
            Lowering::Sequential => "no-overlap",
            Lowering::Completion => "completion",
            Lowering::Parallel => "par",
        }
    }
}

pub struct Rejection {
    /// The numbered rule the diagnostic cites, or `unattributed` when it cites
    /// none. This is what makes generator bias visible: a rule that dominates
    /// the rejection tally is a shape the generator writes wrong.
    pub rule: String,
    pub message: String,
}

pub struct RunOutcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: Option<i32>,
    pub timed_out: bool,
}

impl RunOutcome {
    fn agrees(&self, other: &RunOutcome) -> Option<(&'static str, String, String)> {
        if self.stdout != other.stdout {
            return Some((
                "stdout",
                render_bytes(&self.stdout),
                render_bytes(&other.stdout),
            ));
        }
        if self.stderr != other.stderr {
            return Some((
                "stderr",
                render_bytes(&self.stderr),
                render_bytes(&other.stderr),
            ));
        }
        if self.status != other.status || self.timed_out != other.timed_out {
            return Some(("status", render_status(self), render_status(other)));
        }
        None
    }
}

fn render_status(outcome: &RunOutcome) -> String {
    if outcome.timed_out {
        return "timed out".to_owned();
    }
    match outcome.status {
        Some(code) => format!("exit {code}"),
        None => "killed by signal".to_owned(),
    }
}

/// Enough of the bytes to identify the divergence, with the total length, so a
/// report never carries a hundred kilobytes of one buffer fill.
fn render_bytes(bytes: &[u8]) -> String {
    let head: String = bytes
        .iter()
        .take(64)
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                (*byte as char).to_string()
            } else {
                format!("\\x{byte:02x}")
            }
        })
        .collect();
    format!("{} bytes: {head}", bytes.len())
}

/// One environment setting of the run matrix.
#[derive(Clone, Copy)]
pub struct Setting {
    pub workers: &'static str,
    pub helpers: &'static str,
}

pub const MATRIX: &[Setting] = &[
    Setting {
        workers: "0",
        helpers: "0",
    },
    Setting {
        workers: "0",
        helpers: "1",
    },
    Setting {
        workers: "0",
        helpers: "4",
    },
    Setting {
        workers: "1",
        helpers: "0",
    },
    Setting {
        workers: "1",
        helpers: "1",
    },
    Setting {
        workers: "1",
        helpers: "4",
    },
    Setting {
        workers: "2",
        helpers: "0",
    },
    Setting {
        workers: "2",
        helpers: "1",
    },
    Setting {
        workers: "2",
        helpers: "4",
    },
    Setting {
        workers: "4",
        helpers: "0",
    },
    Setting {
        workers: "4",
        helpers: "1",
    },
    Setting {
        workers: "4",
        helpers: "4",
    },
];

pub struct Ledger {
    pub pair_permitted: u64,
    pub pair_denied: u64,
    pub loop_permitted: u64,
    pub loop_denied: u64,
    pub stage_permitted: u64,
    pub stage_denied: u64,
}

impl Ledger {
    fn read(lines: &[String]) -> Self {
        let mut ledger = Ledger {
            pair_permitted: 0,
            pair_denied: 0,
            loop_permitted: 0,
            loop_denied: 0,
            stage_permitted: 0,
            stage_denied: 0,
        };
        for line in lines {
            let permitted = line.contains("permitted");
            if line.starts_with("PAR permitted") {
                ledger.pair_permitted += 1;
            } else if line.starts_with("PAR denied") {
                ledger.pair_denied += 1;
            } else if line.starts_with("PAR loop") {
                if permitted {
                    ledger.loop_permitted += 1;
                } else {
                    ledger.loop_denied += 1;
                }
            } else if line.starts_with("PAR stage") {
                if permitted {
                    ledger.stage_permitted += 1;
                } else {
                    ledger.stage_denied += 1;
                }
            }
        }
        ledger
    }
}

pub struct Divergence {
    pub lowering: Lowering,
    pub setting: String,
    pub mode: String,
    pub field: String,
    pub reference: String,
    pub observed: String,
}

impl Divergence {
    pub fn describe(&self) -> String {
        format!(
            "{} under {} ({}): {} differs -- reference {} / observed {}",
            self.lowering.spelling(),
            self.setting,
            self.mode,
            self.field,
            self.reference,
            self.observed
        )
    }
}

pub enum Judgment {
    /// The compiler refused the generated source.
    Rejected(Rejection),
    /// An overlapping lowering refused source the sequential lowering accepted.
    /// That is itself a defect: acceptance is not a property of the lowering.
    LoweringRefusal(Lowering, Rejection),
    /// The program is not its own oracle, so no difference it shows is evidence.
    Unstable(String),
    /// The reference itself did not finish.
    ReferenceTimeout,
    /// Every run agreed with the source-order reference.
    Agreed,
    /// A run disagreed and the disagreement survived re-verification.
    Diverged(Divergence),
}

pub struct Verdict {
    pub judgment: Judgment,
    pub ledger: Option<Ledger>,
    pub runs: u64,
    pub fifo_runs: u64,
}

pub struct Oracle {
    pub whitefootc: PathBuf,
    pub fixture: PathBuf,
    pub build: PathBuf,
    pub fifo: PathBuf,
    pub timeout: Duration,
    pub reps: u64,
    pub fifo_delay: Duration,
}

impl Oracle {
    /// Compiles one source under one lowering, returning the permission-ledger
    /// lines the compiler printed.
    pub fn compile(
        &self,
        source: &Path,
        lowering: Lowering,
        binary: &Path,
        ledger: bool,
    ) -> Result<Vec<String>, Rejection> {
        let mut command = Command::new(&self.whitefootc);
        if let Some(flag) = lowering.flag() {
            command.arg(flag);
        }
        if ledger {
            command.arg("--par-ledger");
        }
        command.arg("-o").arg(binary).arg(source);
        let output = match command.output() {
            Ok(output) => output,
            Err(error) => {
                return Err(Rejection {
                    rule: "harness".to_owned(),
                    message: format!("cannot start the compiler: {error}"),
                })
            }
        };
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(Rejection {
                rule: cited_rule(&message),
                message,
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_owned())
            .collect())
    }

    pub fn execute(&self, binary: &Path, setting: Setting) -> RunOutcome {
        let mut command = Command::new(binary);
        command
            .arg("alpha")
            .arg("beta")
            .current_dir(&self.fixture)
            .env("WF_WORKERS", setting.workers)
            .env("WF_IO_HELPERS", setting.helpers)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                return RunOutcome {
                    stdout: Vec::new(),
                    stderr: format!("cannot start {}: {error}", binary.display()).into_bytes(),
                    status: None,
                    timed_out: true,
                }
            }
        };
        let out = drain(child.stdout.take());
        let err = drain(child.stderr.take());
        let (status, timed_out) = self.wait(&mut child);
        RunOutcome {
            stdout: out.recv().unwrap_or_default(),
            stderr: err.recv().unwrap_or_default(),
            status,
            timed_out,
        }
    }

    /// The same run with stdout on a FIFO whose reader waits before draining,
    /// so a publication larger than the host pipe buffer genuinely suspends
    /// inside the runtime instead of completing inline.
    pub fn execute_on_fifo(&self, binary: &Path, setting: Setting, tag: &str) -> RunOutcome {
        let path = self.fifo.join(format!("pipe-{tag}"));
        let _ = fs::remove_file(&path);
        let made = Command::new("/usr/bin/mkfifo").arg(&path).status();
        match made {
            Ok(status) if status.success() => {}
            _ => {
                return RunOutcome {
                    stdout: Vec::new(),
                    stderr: b"cannot create the fifo".to_vec(),
                    status: None,
                    timed_out: true,
                }
            }
        }
        let reader_path = path.clone();
        let delay = self.fifo_delay;
        let (sender, receiver) = mpsc::channel();
        let reader = thread::spawn(move || {
            // Opening the read end rendezvouses with the writer below; the wait
            // afterwards is what fills the pipe.
            let mut file = match fs::File::open(&reader_path) {
                Ok(file) => file,
                Err(_) => {
                    let _ = sender.send(Vec::new());
                    return;
                }
            };
            thread::sleep(delay);
            let mut bytes = Vec::new();
            let _ = file.read_to_end(&mut bytes);
            let _ = sender.send(bytes);
        });
        let write_end = match fs::OpenOptions::new().write(true).open(&path) {
            Ok(file) => file,
            Err(error) => {
                let _ = reader.join();
                return RunOutcome {
                    stdout: Vec::new(),
                    stderr: format!("cannot open the fifo for writing: {error}").into_bytes(),
                    status: None,
                    timed_out: true,
                };
            }
        };
        // The `Command` owns the write end until it is dropped, and while the
        // parent still holds a writer the reader never reaches end of file. So
        // the command lives exactly as long as the spawn: dropping it here is
        // what lets the drain below finish.
        let spawned = {
            let mut command = Command::new(binary);
            command
                .arg("alpha")
                .arg("beta")
                .current_dir(&self.fixture)
                .env("WF_WORKERS", setting.workers)
                .env("WF_IO_HELPERS", setting.helpers)
                .stdin(Stdio::null())
                .stdout(Stdio::from(write_end))
                .stderr(Stdio::piped());
            command.spawn()
        };
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => {
                let _ = reader.join();
                let _ = fs::remove_file(&path);
                return RunOutcome {
                    stdout: Vec::new(),
                    stderr: format!("cannot start {}: {error}", binary.display()).into_bytes(),
                    status: None,
                    timed_out: true,
                };
            }
        };
        let err = drain(child.stderr.take());
        let (status, timed_out) = self.wait(&mut child);
        let stdout = receiver.recv_timeout(self.timeout).unwrap_or_default();
        let _ = reader.join();
        let _ = fs::remove_file(&path);
        RunOutcome {
            stdout,
            stderr: err.recv().unwrap_or_default(),
            status,
            timed_out,
        }
    }

    fn wait(&self, child: &mut Child) -> (Option<i32>, bool) {
        let deadline = Instant::now() + self.timeout;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return (status.code(), false),
                Ok(None) => {}
                Err(_) => return (None, true),
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return (None, true);
            }
            thread::sleep(Duration::from_millis(2));
        }
    }
}

fn drain(stream: Option<impl Read + Send + 'static>) -> mpsc::Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    match stream {
        Some(mut stream) => {
            thread::spawn(move || {
                let mut bytes = Vec::new();
                let _ = stream.read_to_end(&mut bytes);
                let _ = sender.send(bytes);
            });
        }
        None => {
            let _ = sender.send(Vec::new());
        }
    }
    receiver
}

/// The first `[RULE-N]` a diagnostic cites.
fn cited_rule(message: &str) -> String {
    let bytes = message.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'[' {
            if let Some(end) = message[index..].find(']') {
                let candidate = &message[index + 1..index + end];
                if !candidate.is_empty()
                    && candidate
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
                    && candidate.contains('-')
                {
                    return candidate.to_owned();
                }
            }
        }
        index += 1;
    }
    "unattributed".to_owned()
}

/// The subset of the matrix the actualizing parallel lowering is run over. The
/// completion build is the shipped one and gets the whole matrix; the parallel
/// build is the only one `WF_WORKERS` reaches, so it gets the worker axis at
/// both ends of the helper axis.
const PARALLEL_MATRIX: &[Setting] = &[
    Setting {
        workers: "0",
        helpers: "0",
    },
    Setting {
        workers: "1",
        helpers: "4",
    },
    Setting {
        workers: "2",
        helpers: "0",
    },
    Setting {
        workers: "2",
        helpers: "4",
    },
    Setting {
        workers: "4",
        helpers: "0",
    },
    Setting {
        workers: "4",
        helpers: "4",
    },
];

impl Oracle {
    /// The whole judgment of one program: compile three ways, establish that
    /// the program is its own stable oracle, then require every overlapping
    /// execution to publish exactly what the source-order one published.
    pub fn assess(&self, source: &Path, tag: &str, bulk: bool) -> Verdict {
        // The three file names are the same length on purpose. A program can
        // read its own invocation, and the length of argument zero is the one
        // thing about the run that the harness rather than the compiler
        // decides; equal-length names keep even that out of the comparison.
        let sequential = self.build.join(format!("{tag}-a"));
        let completion = self.build.join(format!("{tag}-b"));
        let parallel = self.build.join(format!("{tag}-c"));
        let mut runs = 0;
        let mut fifo_runs = 0;

        if let Err(rejection) = self.compile(source, Lowering::Sequential, &sequential, false) {
            return Verdict {
                judgment: Judgment::Rejected(rejection),
                ledger: None,
                runs,
                fifo_runs,
            };
        }
        let ledger = match self.compile(source, Lowering::Completion, &completion, true) {
            Ok(lines) => Ledger::read(&lines),
            Err(rejection) => {
                return Verdict {
                    judgment: Judgment::LoweringRefusal(Lowering::Completion, rejection),
                    ledger: None,
                    runs,
                    fifo_runs,
                }
            }
        };
        if let Err(rejection) = self.compile(source, Lowering::Parallel, &parallel, false) {
            return Verdict {
                judgment: Judgment::LoweringRefusal(Lowering::Parallel, rejection),
                ledger: Some(ledger),
                runs,
                fifo_runs,
            };
        }

        let quiet = Setting {
            workers: "0",
            helpers: "0",
        };
        let reference = self.execute(&sequential, quiet);
        runs += 1;
        if reference.timed_out {
            return Verdict {
                judgment: Judgment::ReferenceTimeout,
                ledger: Some(ledger),
                runs,
                fifo_runs,
            };
        }
        // A program that does not agree with itself cannot judge anything.
        for setting in [
            quiet,
            Setting {
                workers: "4",
                helpers: "4",
            },
        ] {
            let again = self.execute(&sequential, setting);
            runs += 1;
            if let Some((field, first, second)) = reference.agrees(&again) {
                return Verdict {
                    judgment: Judgment::Unstable(format!(
                        "the sequential build published different {field} on two runs: {first} / {second}"
                    )),
                    ledger: Some(ledger),
                    runs,
                    fifo_runs,
                };
            }
        }

        let plan: [(Lowering, &Path, &[Setting]); 2] = [
            (Lowering::Completion, &completion, MATRIX),
            (Lowering::Parallel, &parallel, PARALLEL_MATRIX),
        ];
        for (lowering, binary, matrix) in plan {
            for setting in matrix {
                for _ in 0..self.reps.max(1) {
                    let observed = self.execute(binary, *setting);
                    runs += 1;
                    if let Some((field, expected, actual)) = reference.agrees(&observed) {
                        let candidate = Divergence {
                            lowering,
                            setting: format!(
                                "WF_WORKERS={} WF_IO_HELPERS={}",
                                setting.workers, setting.helpers
                            ),
                            mode: "captured pipe".to_owned(),
                            field: field.to_owned(),
                            reference: expected,
                            observed: actual,
                        };
                        return self.confirm(
                            &sequential,
                            binary,
                            *setting,
                            candidate,
                            &reference,
                            ledger,
                            runs,
                            fifo_runs,
                            None,
                            tag,
                        );
                    }
                }
            }
        }

        // The delayed-reader runs. A program that publishes more than one pipe
        // buffer really waits here; the rest still exercise a slow consumer.
        if bulk {
            for setting in [
                Setting {
                    workers: "0",
                    helpers: "0",
                },
                Setting {
                    workers: "0",
                    helpers: "4",
                },
                Setting {
                    workers: "4",
                    helpers: "4",
                },
            ] {
                let observed = self.execute_on_fifo(&completion, setting, tag);
                fifo_runs += 1;
                if let Some((field, expected, actual)) = reference.agrees(&observed) {
                    let candidate = Divergence {
                        lowering: Lowering::Completion,
                        setting: format!(
                            "WF_WORKERS={} WF_IO_HELPERS={}",
                            setting.workers, setting.helpers
                        ),
                        mode: "delayed fifo reader".to_owned(),
                        field: field.to_owned(),
                        reference: expected,
                        observed: actual,
                    };
                    return self.confirm(
                        &sequential,
                        &completion,
                        setting,
                        candidate,
                        &reference,
                        ledger,
                        runs,
                        fifo_runs,
                        Some(()),
                        tag,
                    );
                }
            }
        }

        Verdict {
            judgment: Judgment::Agreed,
            ledger: Some(ledger),
            runs,
            fifo_runs,
        }
    }

    /// One disagreement is not yet a finding. Re-run the reference to make sure
    /// it is still the same program, then re-run the differing configuration to
    /// make sure the difference is the program's and not the machine's.
    #[allow(clippy::too_many_arguments)]
    fn confirm(
        &self,
        sequential: &Path,
        binary: &Path,
        setting: Setting,
        candidate: Divergence,
        reference: &RunOutcome,
        ledger: Ledger,
        mut runs: u64,
        mut fifo_runs: u64,
        fifo: Option<()>,
        tag: &str,
    ) -> Verdict {
        for _ in 0..3 {
            let again = self.execute(
                sequential,
                Setting {
                    workers: "0",
                    helpers: "0",
                },
            );
            runs += 1;
            if let Some((field, first, second)) = reference.agrees(&again) {
                return Verdict {
                    judgment: Judgment::Unstable(format!(
                        "the sequential build published different {field} on re-verification: {first} / {second}"
                    )),
                    ledger: Some(ledger),
                    runs,
                    fifo_runs,
                };
            }
        }
        for _ in 0..3 {
            let observed = if fifo.is_some() {
                fifo_runs += 1;
                self.execute_on_fifo(binary, setting, &format!("{tag}-confirm"))
            } else {
                runs += 1;
                self.execute(binary, setting)
            };
            if reference.agrees(&observed).is_some() {
                return Verdict {
                    judgment: Judgment::Diverged(candidate),
                    ledger: Some(ledger),
                    runs,
                    fifo_runs,
                };
            }
        }
        // The difference did not survive: report it anyway, because an
        // intermittent divergence is still a divergence, but say so.
        Verdict {
            judgment: Judgment::Diverged(Divergence {
                mode: format!("{} (intermittent)", candidate.mode),
                ..candidate
            }),
            ledger: Some(ledger),
            runs,
            fifo_runs,
        }
    }
}

/// Writes the fixture tree the generated programs open, read, and enumerate.
/// The contents are fixed, so a program that reads them is deterministic and
/// any difference between two runs is the implementation's.
pub fn write_fixture(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|error| format!("cannot create the fixture: {error}"))?;
    let lengths: [usize; 8] = [0, 1, 7, 63, 64, 200, 1024, 5000];
    for (index, length) in lengths.iter().enumerate() {
        let mut bytes = Vec::with_capacity(*length);
        for position in 0..*length {
            bytes.push(((index * 31 + position * 17 + 65) % 251) as u8);
        }
        let name = format!("f0{index}.dat");
        fs::write(root.join(name), &bytes)
            .map_err(|error| format!("cannot write a fixture file: {error}"))?;
    }
    let nested = root.join("sub");
    fs::create_dir_all(&nested)
        .map_err(|error| format!("cannot create the fixture subdirectory: {error}"))?;
    fs::write(nested.join("inner.dat"), b"nested fixture payload\n")
        .map_err(|error| format!("cannot write a nested fixture file: {error}"))?;
    Ok(())
}
