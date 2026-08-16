//! One-shot Stage 1 migration verifier. Deleted in the final migration commit.
//!
//! Proves that a reflowed specification carries the same content as its
//! predecessor: both sides are normalized by joining every non-fenced line
//! into one stream and collapsing whitespace runs, and the two streams must
//! then be byte-equal except for explicitly enumerated insertions, each with
//! its exact count.
//!
//! Usage: stage1_verify OLD NEW [--insert TEXT=COUNT]...

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

/// Join non-fenced lines into one stream, keeping fenced content verbatim on
/// its own lines, and collapse every whitespace run to one space.
fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.lines() {
        if line.starts_with("```") {
            in_fence = !in_fence;
            out.push('\n');
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_fence {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        out.push(' ');
        out.push_str(line);
    }
    collapse(&out)
}

fn collapse(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    let mut pending_break = false;
    for character in text.chars() {
        match character {
            '\n' => pending_break = true,
            c if c.is_whitespace() => pending_space = true,
            c => {
                if pending_break {
                    out.push('\n');
                } else if pending_space && !out.is_empty() {
                    out.push(' ');
                }
                pending_space = false;
                pending_break = false;
                out.push(c);
            }
        }
    }
    out
}

/// Compare two normalized streams, consuming enumerated insertions on the new
/// side. Returns the observed count of each insertion, or the first mismatch.
fn compare(old: &str, new: &str, insertions: &[String]) -> Result<Vec<usize>, String> {
    let old = old.as_bytes();
    let new = new.as_bytes();
    let mut counts = vec![0_usize; insertions.len()];
    let mut left = 0_usize;
    let mut right = 0_usize;
    while left < old.len() && right < new.len() {
        if old[left] == new[right] {
            left += 1;
            right += 1;
            continue;
        }
        let mut matched = false;
        for (index, insertion) in insertions.iter().enumerate() {
            if new[right..].starts_with(insertion.as_bytes()) {
                right += insertion.len();
                counts[index] += 1;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(mismatch(old, new, left, right));
        }
    }
    // A trailing insertion may still be pending on either side.
    while right < new.len() {
        let mut matched = false;
        for (index, insertion) in insertions.iter().enumerate() {
            if new[right..].starts_with(insertion.as_bytes()) {
                right += insertion.len();
                counts[index] += 1;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(mismatch(old, new, left, right));
        }
    }
    if left != old.len() {
        return Err(mismatch(old, new, left, right));
    }
    Ok(counts)
}

fn mismatch(old: &[u8], new: &[u8], left: usize, right: usize) -> String {
    let window = 90_usize;
    let old_from = left.saturating_sub(window);
    let new_from = right.saturating_sub(window);
    format!(
        "content differs at old byte {left}, new byte {right}\n  old: ...{}\n  new: ...{}",
        String::from_utf8_lossy(&old[old_from..(left + window).min(old.len())]),
        String::from_utf8_lossy(&new[new_from..(right + window).min(new.len())]),
    )
}

/// The rule-id shape of `compiler/src/bin/spec.rs`, reimplemented here so the
/// count below is an independent derivation rather than the gate's own answer.
fn is_rule_id(text: &str) -> bool {
    let Some((family, number)) = text.split_once('-') else {
        return false;
    };
    if family.is_empty() || !family.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return false;
    }
    let digits = number.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0
        && (digits == number.len()
            || (digits + 1 == number.len() && number.as_bytes()[digits].is_ascii_lowercase()))
}

fn bracketed(line: &str) -> Option<&str> {
    let rest = line.strip_prefix('[')?;
    let close = rest.find(']')?;
    Some(&rest[..close])
}

fn structure_report(text: &str) {
    let mut definitions: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates: Vec<&str> = Vec::new();
    let mut sub_ids = 0_usize;
    let mut prior_lines = 0_usize;
    for line in text.lines() {
        if line.starts_with("Prior:") {
            prior_lines += 1;
        }
        let Some(candidate) = bracketed(line) else {
            continue;
        };
        if is_rule_id(candidate) {
            if !definitions.insert(candidate) {
                duplicates.push(candidate);
            }
        } else if candidate.contains('.') {
            sub_ids += 1;
        }
    }

    let mut references: BTreeSet<&str> = BTreeSet::new();
    let mut remaining = text;
    while let Some(open) = remaining.find('[') {
        remaining = &remaining[open + 1..];
        let Some(close) = remaining.find(']') else {
            break;
        };
        let candidate = &remaining[..close];
        if is_rule_id(candidate) {
            references.insert(candidate);
        }
        remaining = &remaining[close + 1..];
    }
    let unknown: Vec<&&str> = references.difference(&definitions).collect();

    println!("  rule definitions: {}", definitions.len());
    println!("  duplicate definitions: {duplicates:?}");
    println!("  sub-id lines: {sub_ids}");
    println!("  distinct references: {}", references.len());
    println!("  unresolved references: {unknown:?}");
    println!("  lines beginning with `Prior:`: {prior_lines}");
    println!("  lines: {}", text.lines().count());
    println!("  bytes: {}", text.len());
    let status_block = text.find("\n\n").map_or(0, |offset| offset + 2)
        + text[text.find("\n\n").map_or(0, |offset| offset + 2)..]
            .find("\n\n")
            .map_or(0, |offset| offset + 1);
    let preamble = text.find("\n## ").unwrap_or(0);
    println!("  title+status header bytes: {status_block}");
    println!("  bytes before the first section heading: {preamble}");
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut paths: Vec<&String> = Vec::new();
    let mut insertions: Vec<String> = Vec::new();
    let mut expected: Vec<usize> = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--insert" {
            let value = arguments.get(index + 1).expect("--insert needs TEXT=COUNT");
            let (text, count) = value.rsplit_once('=').expect("--insert needs TEXT=COUNT");
            insertions.push(text.replace("\\n", "\n"));
            expected.push(count.parse().expect("insertion count is a number"));
            index += 2;
            continue;
        }
        paths.push(&arguments[index]);
        index += 1;
    }
    let (Some(old_path), Some(new_path), 2) = (paths.first(), paths.get(1), paths.len()) else {
        eprintln!("usage: stage1_verify OLD NEW [--insert TEXT=COUNT]...");
        std::process::exit(2);
    };
    let old = std::fs::read_to_string(old_path).expect("old specification");
    let new = std::fs::read_to_string(new_path).expect("new specification");

    println!("OLD {old_path}");
    structure_report(&old);
    println!("NEW {new_path}");
    structure_report(&new);

    let old_normalized = normalize(&old);
    let new_normalized = normalize(&new);
    println!(
        "normalized: old {} bytes, new {} bytes",
        old_normalized.len(),
        new_normalized.len()
    );
    match compare(&old_normalized, &new_normalized, &insertions) {
        Ok(counts) => {
            let mut wrong = false;
            for ((insertion, count), want) in insertions.iter().zip(&counts).zip(&expected) {
                let mark = if count == want { "ok" } else { "WRONG" };
                if count != want {
                    wrong = true;
                }
                println!("  insertion {insertion:?}: {count} (expected {want}) {mark}");
            }
            if wrong {
                println!("VERDICT: FAIL — an enumerated insertion has the wrong count");
                std::process::exit(1);
            }
            println!(
                "VERDICT: PASS — normalized content is identical apart from {} enumerated insertion kinds",
                insertions.len()
            );
        }
        Err(error) => {
            println!("VERDICT: FAIL — {error}");
            std::process::exit(1);
        }
    }
}
