//! Delta debugging of a differing program.
//!
//! The validity oracle is the same judgment that found the difference, so a
//! candidate survives only when it still compiles under every lowering, is
//! still its own stable oracle, and still diverges. Removing a line usually
//! unbalances a brace or drops a binding a later line reads; the compiler
//! rejects that candidate and the chunk is kept. That makes brace-blind line
//! removal safe here without a parser of our own.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::oracle::{Judgment, Oracle};

pub struct Minimized {
    pub source: String,
    pub trials: u64,
    pub removed: usize,
}

/// Chunk-wise removal, halving the chunk size when a pass removes nothing, in
/// the ordinary ddmin shape. Bounded by both a trial count and a wall clock,
/// because each trial is three compilations and a run matrix.
pub fn minimize(
    oracle: &Oracle,
    scratch: &Path,
    tag: &str,
    source: &str,
    bulk: bool,
    max_trials: u64,
    budget: Duration,
) -> Minimized {
    let deadline = Instant::now() + budget;
    let original = source.lines().count();
    let mut lines: Vec<String> = source.lines().map(|line| line.to_owned()).collect();
    let mut trials = 0;
    let mut chunk = 8usize;
    loop {
        let mut progressed = false;
        let mut index = 0;
        while index < lines.len() {
            if trials >= max_trials || Instant::now() >= deadline {
                return finish(lines, trials, original);
            }
            let end = (index + chunk).min(lines.len());
            let mut candidate = lines.clone();
            candidate.drain(index..end);
            trials += 1;
            if diverges(oracle, scratch, tag, &candidate, bulk) {
                lines = candidate;
                progressed = true;
            } else {
                index = end;
            }
        }
        if !progressed {
            if chunk == 1 {
                break;
            }
            chunk /= 2;
        }
    }
    finish(lines, trials, original)
}

fn finish(lines: Vec<String>, trials: u64, original: usize) -> Minimized {
    let removed = original.saturating_sub(lines.len());
    let mut source = lines.join("\n");
    source.push('\n');
    Minimized {
        source,
        trials,
        removed,
    }
}

fn diverges(oracle: &Oracle, scratch: &Path, tag: &str, lines: &[String], bulk: bool) -> bool {
    let mut source = lines.join("\n");
    source.push('\n');
    let path = scratch.join(format!("{tag}-candidate.wf"));
    if fs::write(&path, source.as_bytes()).is_err() {
        return false;
    }
    let verdict = oracle.assess(&path, &format!("{tag}-candidate"), bulk);
    matches!(verdict.judgment, Judgment::Diverged(_))
}
