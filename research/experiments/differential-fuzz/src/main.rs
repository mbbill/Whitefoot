//! Differential fuzzing of Whitefoot's overlap lowerings.
//!
//! See README.md for what this experiment serves and when it is removed. The
//! entry point is a small subcommand dispatcher; the work is in `generator`
//! (what programs to write), `oracle` (how one program is judged), `minimize`
//! (how a differing program is cut down), and `campaign` (how many, how long,
//! and what the run reports).

mod campaign;
mod generator;
mod minimize;
mod oracle;
mod rng;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage:
  wf-difffuzz generate --seed N
  wf-difffuzz check --whitefootc PATH --work DIR --seed N
  wf-difffuzz campaign --whitefootc PATH --work DIR [--programs N] [--budget SECONDS]
                       [--jobs N] [--seed N] [--reps N]
  wf-difffuzz probes --whitefootc PATH --work DIR --directory DIR [--reps N]";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("wf-difffuzz: {message}");
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    let command = arguments
        .first()
        .ok_or_else(|| "no subcommand".to_owned())?;
    let options = Options::parse(&arguments[1..])?;
    match command.as_str() {
        "generate" => {
            let program = generator::generate(options.seed);
            print!("{}", program.source);
            Ok(())
        }
        "check" => campaign::check_one(&options.require_paths()?, options.seed),
        "campaign" => campaign::run(&options.require_paths()?, &options),
        "probes" => campaign::run_probes(&options.require_paths()?, &options),
        other => Err(format!("unknown subcommand {other}")),
    }
}

/// Where the compiler is and where the campaign is allowed to write. Both are
/// always supplied by the Makefile; nothing here guesses a path.
pub struct Paths {
    pub whitefootc: PathBuf,
    pub work: PathBuf,
}

pub struct Options {
    whitefootc: Option<PathBuf>,
    work: Option<PathBuf>,
    pub directory: Option<PathBuf>,
    pub seed: u64,
    pub programs: u64,
    pub budget: u64,
    pub jobs: usize,
    pub reps: u64,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut options = Options {
            whitefootc: None,
            work: None,
            directory: None,
            seed: 1,
            programs: 2000,
            budget: 5400,
            jobs: 4,
            reps: 2,
        };
        let mut index = 0;
        while index < arguments.len() {
            let flag = arguments[index].as_str();
            let value = || {
                arguments
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag {
                "--whitefootc" => options.whitefootc = Some(PathBuf::from(value()?)),
                "--work" => options.work = Some(PathBuf::from(value()?)),
                "--directory" => options.directory = Some(PathBuf::from(value()?)),
                "--seed" => options.seed = parse_number(&value()?, flag)?,
                "--programs" => options.programs = parse_number(&value()?, flag)?,
                "--budget" => options.budget = parse_number(&value()?, flag)?,
                "--jobs" => options.jobs = parse_number(&value()?, flag)? as usize,
                "--reps" => options.reps = parse_number(&value()?, flag)?,
                other => return Err(format!("unknown option {other}")),
            }
            index += 2;
        }
        if options.jobs == 0 {
            options.jobs = 1;
        }
        Ok(options)
    }

    fn require_paths(&self) -> Result<Paths, String> {
        Ok(Paths {
            whitefootc: self
                .whitefootc
                .clone()
                .ok_or_else(|| "--whitefootc is required".to_owned())?,
            work: self
                .work
                .clone()
                .ok_or_else(|| "--work is required".to_owned())?,
        })
    }
}

fn parse_number(text: &str, flag: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|error| format!("{flag}: {text}: {error}"))
}
