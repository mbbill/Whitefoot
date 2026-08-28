//! The campaign: how many programs, for how long, and what the run reports.
//!
//! The report is the deliverable. A campaign that finds nothing still has to
//! say what it covered -- how many programs the compiler accepted, which rule
//! refused the rest, which shapes reached the compiler, and how many loops the
//! permission ledger actually granted -- because "we fuzzed it and found
//! nothing" is worth exactly as much as the coverage behind it.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::generator;
use crate::minimize;
use crate::oracle::{self, Judgment, Ledger, Oracle};
use crate::{Options, Paths};

#[derive(Default)]
struct Tally {
    attempts: u64,
    accepted: u64,
    agreed: u64,
    diverged: u64,
    unstable: u64,
    reference_timeouts: u64,
    lowering_refusals: u64,
    rejected: u64,
    rejections: BTreeMap<String, u64>,
    shapes: BTreeMap<String, u64>,
    runs: u64,
    fifo_runs: u64,
    pair_permitted: u64,
    pair_denied: u64,
    loop_permitted: u64,
    loop_denied: u64,
    stage_permitted: u64,
    stage_denied: u64,
    programs_with_permitted_stage: u64,
    programs_with_permitted_pair: u64,
    findings: Vec<String>,
}

impl Tally {
    fn absorb_ledger(&mut self, ledger: &Ledger) {
        self.pair_permitted += ledger.pair_permitted;
        self.pair_denied += ledger.pair_denied;
        self.loop_permitted += ledger.loop_permitted;
        self.loop_denied += ledger.loop_denied;
        self.stage_permitted += ledger.stage_permitted;
        self.stage_denied += ledger.stage_denied;
        if ledger.stage_permitted > 0 {
            self.programs_with_permitted_stage += 1;
        }
        if ledger.pair_permitted > 0 {
            self.programs_with_permitted_pair += 1;
        }
    }
}

fn prepare(paths: &Paths) -> Result<Oracle, String> {
    for directory in ["programs", "build", "fifo", "findings", "fixture"] {
        fs::create_dir_all(paths.work.join(directory))
            .map_err(|error| format!("cannot create {directory}: {error}"))?;
    }
    let fixture = paths.work.join("fixture");
    oracle::write_fixture(&fixture)?;
    Ok(Oracle {
        whitefootc: paths.whitefootc.clone(),
        fixture,
        build: paths.work.join("build"),
        fifo: paths.work.join("fifo"),
        timeout: Duration::from_secs(20),
        reps: 2,
        fifo_delay: Duration::from_millis(60),
    })
}

/// One program, reported in full. This is the loop a person uses when a seed
/// from a campaign report needs looking at.
pub fn check_one(paths: &Paths, seed: u64) -> Result<(), String> {
    let oracle = prepare(paths)?;
    let program = generator::generate(seed);
    let source = paths.work.join("programs").join(format!("seed-{seed}.wf"));
    fs::write(&source, program.source.as_bytes())
        .map_err(|error| format!("cannot write the program: {error}"))?;
    let verdict = oracle.assess(&source, &format!("seed-{seed}"), program.bulk_output);
    println!("seed {seed}: {}", source.display());
    println!(
        "shapes: {}",
        program
            .shapes
            .iter()
            .map(|shape| shape.spelling())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(ledger) = &verdict.ledger {
        println!(
            "ledger: pairs {}/{} permitted, loops {}/{} permitted, stages {}/{} permitted",
            ledger.pair_permitted,
            ledger.pair_permitted + ledger.pair_denied,
            ledger.loop_permitted,
            ledger.loop_permitted + ledger.loop_denied,
            ledger.stage_permitted,
            ledger.stage_permitted + ledger.stage_denied
        );
    }
    match verdict.judgment {
        Judgment::Rejected(rejection) => {
            println!("rejected [{}]: {}", rejection.rule, rejection.message);
        }
        Judgment::LoweringRefusal(lowering, rejection) => {
            println!(
                "the {} lowering refused source the sequential lowering accepted [{}]: {}",
                lowering.spelling(),
                rejection.rule,
                rejection.message
            );
        }
        Judgment::Unstable(reason) => println!("unstable: {reason}"),
        Judgment::ReferenceTimeout => println!("the reference run did not finish"),
        Judgment::Agreed => println!("agreed across {} runs", verdict.runs + verdict.fifo_runs),
        Judgment::Diverged(divergence) => println!("DIVERGED: {}", divergence.describe()),
    }
    Ok(())
}

pub fn run(paths: &Paths, options: &Options) -> Result<(), String> {
    let mut oracle = prepare(paths)?;
    oracle.reps = options.reps;
    let oracle = Arc::new(oracle);
    let started = Instant::now();
    let deadline = started + Duration::from_secs(options.budget);
    let tally = Arc::new(Mutex::new(Tally::default()));
    let next = Arc::new(AtomicU64::new(options.seed));
    let accepted = Arc::new(AtomicU64::new(0));
    let programs = paths.work.join("programs");
    let findings = paths.work.join("findings");

    println!(
        "campaign: up to {} accepted programs, {} seconds, {} jobs, seed {}, {} repetitions",
        options.programs, options.budget, options.jobs, options.seed, options.reps
    );

    let mut workers = Vec::new();
    for _ in 0..options.jobs {
        let oracle = Arc::clone(&oracle);
        let tally = Arc::clone(&tally);
        let next = Arc::clone(&next);
        let accepted = Arc::clone(&accepted);
        let programs = programs.clone();
        let findings = findings.clone();
        let target = options.programs;
        workers.push(thread::spawn(move || loop {
            if Instant::now() >= deadline || accepted.load(Ordering::Relaxed) >= target {
                return;
            }
            let seed = next.fetch_add(1, Ordering::Relaxed);
            judge_one(&oracle, &tally, &accepted, &programs, &findings, seed);
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }

    let tally = tally.lock().map_err(|_| "the tally was poisoned")?;
    let report = render(&tally, started.elapsed(), options);
    print!("{report}");
    fs::write(paths.work.join("report.txt"), report.as_bytes())
        .map_err(|error| format!("cannot write the report: {error}"))?;
    Ok(())
}

fn judge_one(
    oracle: &Arc<Oracle>,
    tally: &Arc<Mutex<Tally>>,
    accepted: &Arc<AtomicU64>,
    programs: &Path,
    findings: &Path,
    seed: u64,
) {
    let program = generator::generate(seed);
    let tag = format!("seed-{seed}");
    let source = programs.join(format!("{tag}.wf"));
    if fs::write(&source, program.source.as_bytes()).is_err() {
        return;
    }
    let verdict = oracle.assess(&source, &tag, program.bulk_output);
    let mut record = tally.lock().expect("the tally is live");
    record.attempts += 1;
    record.runs += verdict.runs;
    record.fifo_runs += verdict.fifo_runs;
    let counted = !matches!(verdict.judgment, Judgment::Rejected(_));
    if counted {
        record.accepted += 1;
        accepted.fetch_add(1, Ordering::Relaxed);
        for shape in &program.shapes {
            *record
                .shapes
                .entry(shape.spelling().to_owned())
                .or_insert(0) += 1;
        }
        if let Some(ledger) = &verdict.ledger {
            record.absorb_ledger(ledger);
        }
    }
    match verdict.judgment {
        Judgment::Rejected(rejection) => {
            record.rejected += 1;
            *record.rejections.entry(rejection.rule).or_insert(0) += 1;
        }
        Judgment::LoweringRefusal(lowering, rejection) => {
            record.lowering_refusals += 1;
            let note = format!(
                "seed {seed}: the {} lowering refused source the sequential lowering accepted [{}]: {}",
                lowering.spelling(),
                rejection.rule,
                rejection.message
            );
            record.findings.push(note.clone());
            drop(record);
            save_finding(findings, seed, &program.source, &note, None);
        }
        Judgment::Unstable(_) => record.unstable += 1,
        Judgment::ReferenceTimeout => record.reference_timeouts += 1,
        Judgment::Agreed => record.agreed += 1,
        Judgment::Diverged(divergence) => {
            record.diverged += 1;
            let note = format!("seed {seed}: {}", divergence.describe());
            record.findings.push(note.clone());
            drop(record);
            println!("FINDING {note}");
            let reduced = minimize::minimize(
                oracle,
                findings,
                &tag,
                &program.source,
                program.bulk_output,
                240,
                Duration::from_secs(900),
            );
            println!(
                "  minimized seed {seed}: {} lines removed in {} trials",
                reduced.removed, reduced.trials
            );
            save_finding(
                findings,
                seed,
                &program.source,
                &note,
                Some(&reduced.source),
            );
        }
    }
}

fn save_finding(findings: &Path, seed: u64, original: &str, note: &str, minimized: Option<&str>) {
    let directory = findings.join(format!("finding-{seed}"));
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let _ = fs::write(directory.join("note.txt"), note.as_bytes());
    let _ = fs::write(directory.join("original.wf"), original.as_bytes());
    if let Some(source) = minimized {
        let _ = fs::write(directory.join("minimized.wf"), source.as_bytes());
    }
}

/// Re-runs every recorded probe under the same oracle that found it.
pub fn run_probes(paths: &Paths, options: &Options) -> Result<(), String> {
    let directory = options
        .directory
        .clone()
        .ok_or_else(|| "--directory is required".to_owned())?;
    let mut oracle = prepare(paths)?;
    oracle.reps = options.reps;
    // No probes recorded is the ordinary state of a campaign that has found
    // nothing yet, not an error.
    let listing = match fs::read_dir(&directory) {
        Ok(listing) => listing,
        Err(_) => {
            println!("no probes recorded in {}", directory.display());
            return Ok(());
        }
    };
    let mut entries: Vec<_> = listing
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "wf").unwrap_or(false))
        .collect();
    entries.sort();
    if entries.is_empty() {
        println!("no probes recorded in {}", directory.display());
        return Ok(());
    }
    for path in &entries {
        let tag = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "probe".to_owned());
        let verdict = oracle.assess(path, &tag, true);
        // A probe's expected outcome is written down in probes/README.md rather
        // than encoded here, because a probe may record a rejection as readily
        // as a divergence. This prints what happens now; the reader compares.
        let outcome = match verdict.judgment {
            Judgment::Diverged(divergence) => format!("diverges: {}", divergence.describe()),
            Judgment::Agreed => "compiles and agrees on every run".to_owned(),
            Judgment::Rejected(rejection) => {
                format!("rejected [{}]: {}", rejection.rule, rejection.message)
            }
            Judgment::LoweringRefusal(lowering, rejection) => format!(
                "the {} lowering refuses it [{}]: {}",
                lowering.spelling(),
                rejection.rule,
                rejection.message
            ),
            Judgment::Unstable(reason) => format!("unstable: {reason}"),
            Judgment::ReferenceTimeout => "the reference run did not finish".to_owned(),
        };
        println!("{}: {outcome}", path.display());
    }
    println!(
        "{} probes run; compare each outcome against {}/README.md",
        entries.len(),
        directory.display()
    );
    Ok(())
}

fn render(tally: &Tally, elapsed: Duration, options: &Options) -> String {
    let mut text = String::new();
    text.push_str("\n== differential-fuzz campaign ==\n");
    text.push_str(&format!(
        "first seed {}, {} jobs, {} repetitions, {:.1} minutes\n",
        options.seed,
        options.jobs,
        options.reps,
        elapsed.as_secs_f64() / 60.0
    ));
    text.push_str(&format!(
        "attempts {}, accepted {} ({:.1}%), rejected {}\n",
        tally.attempts,
        tally.accepted,
        percentage(tally.accepted, tally.attempts),
        tally.rejected
    ));
    text.push_str(&format!(
        "agreed {}, diverged {}, unstable {}, reference timeouts {}, lowering refusals {}\n",
        tally.agreed,
        tally.diverged,
        tally.unstable,
        tally.reference_timeouts,
        tally.lowering_refusals
    ));
    text.push_str(&format!(
        "executions {} captured, {} through a delayed fifo reader\n",
        tally.runs, tally.fifo_runs
    ));

    text.push_str("\nrejections by cited rule\n");
    if tally.rejections.is_empty() {
        text.push_str("  (none)\n");
    }
    for (rule, count) in &tally.rejections {
        text.push_str(&format!(
            "  {rule:<16} {count:>6}  {:.1}% of attempts\n",
            percentage(*count, tally.attempts)
        ));
    }

    text.push_str("\npermission ledger over accepted programs\n");
    text.push_str(&format!(
        "  PAR-1 pairs      {:>6} permitted, {:>6} denied\n",
        tally.pair_permitted, tally.pair_denied
    ));
    text.push_str(&format!(
        "  PAR-2 loops      {:>6} permitted, {:>6} denied\n",
        tally.loop_permitted, tally.loop_denied
    ));
    text.push_str(&format!(
        "  PAR-3 stages     {:>6} permitted, {:>6} denied\n",
        tally.stage_permitted, tally.stage_denied
    ));
    text.push_str(&format!(
        "  programs holding at least one permitted PAR-3 stage: {} ({:.1}% of accepted)\n",
        tally.programs_with_permitted_stage,
        percentage(tally.programs_with_permitted_stage, tally.accepted)
    ));
    text.push_str(&format!(
        "  programs holding at least one permitted PAR-1 pair:  {} ({:.1}% of accepted)\n",
        tally.programs_with_permitted_pair,
        percentage(tally.programs_with_permitted_pair, tally.accepted)
    ));

    text.push_str("\nshapes reaching an accepted program\n");
    for (shape, count) in &tally.shapes {
        text.push_str(&format!(
            "  {shape:<34} {count:>6}  {:.1}% of accepted\n",
            percentage(*count, tally.accepted)
        ));
    }

    text.push_str("\nfindings\n");
    if tally.findings.is_empty() {
        text.push_str("  (none)\n");
    }
    for finding in &tally.findings {
        text.push_str(&format!("  {finding}\n"));
    }
    text
}

fn percentage(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64) * 100.0 / (whole as f64)
}
