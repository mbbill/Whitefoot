//! One-shot verifier for the batch 0070 DIAG-1 restructure and ratchet deltas.
//!
//! Purpose: prove that `DELTA-DIAG1.md`'s `wf-diag` rows carry exactly the
//! content of the base specification sentences they replace, and that every
//! base sentence quoted as evidence in `DELTA-RATCHET.md` is the file's actual
//! bytes rather than a hand transcription.
//!
//! It is deliberately a one-shot: it is pinned to one base digest and one set
//! of hunks, so it cannot be reused across spec versions. Delete it in the same
//! change that integrates or abandons these deltas.
//!
//! Build and run (no dependencies, no build script):
//!     rustc -O --edition 2021 -o /tmp/verify-delta verify-delta.rs
//!     /tmp/verify-delta <spec> <DELTA-DIAG1.md> <DELTA-RATCHET.md>
//!     /tmp/verify-delta <spec> <DELTA-DIAG1.md> <DELTA-RATCHET.md> --negative-control
//!
//! What a PASS establishes: every moved base line is reconstructible
//! byte-for-byte from its new row plus one declared reading template; no base
//! line is silently dropped; no new line is undeclared; the multiset of rule
//! citations and of location forms is identical before and after; every quoted
//! base sentence matches the file. What it does NOT establish: that moving a
//! mapping from prose into a fence is the right design, or that the fence's
//! declared reading is the reading a human would infer.

use std::collections::BTreeMap;
use std::process::ExitCode;

const BASE_DIGEST: &str = "5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1";
const DIAG1_FIRST: usize = 1546; // 1-based, inclusive
const DIAG1_LAST: usize = 1890; // 1-based, inclusive (line before [DIAG-2])

struct Block {
    kind: String,
    args: Vec<String>,
    lines: Vec<String>,
}

struct Hunk {
    id: String,
    start: usize,
    end: usize,
    lines: Vec<String>,
}

struct Entry {
    fence: String,
    old_line: usize,
    template: String,
    trail: String,
    cells: Vec<String>,
}

struct Cut {
    id: String,
    line: usize,
    reference: usize,
    old: String,
    new: String,
}

fn read(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    text.lines().map(|l| l.to_string()).collect()
}

/// Blocks are delimited by `<<<kind args` and a lone `>>>`, so a block may hold
/// markdown fences without any escaping.
fn blocks(lines: &[String]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut open: Option<Block> = None;
    for line in lines {
        if let Some(rest) = line.strip_prefix("<<<") {
            assert!(open.is_none(), "nested delta block: {line}");
            let mut parts = rest.split_whitespace();
            let kind = parts.next().expect("empty delta block header").to_string();
            open = Some(Block {
                kind,
                args: parts.map(|s| s.to_string()).collect(),
                lines: Vec::new(),
            });
        } else if line.trim_end() == ">>>" {
            out.push(open.take().expect("stray >>>"));
        } else if let Some(b) = open.as_mut() {
            b.lines.push(line.clone());
        }
    }
    assert!(open.is_none(), "unterminated delta block");
    out
}

fn split_row(spec: &str) -> Vec<String> {
    spec.split(" | ").map(|c| c.trim().to_string()).collect()
}

fn fill(template: &str, cells: &[String]) -> String {
    let mut out = template.to_string();
    for (i, c) in cells.iter().enumerate() {
        out = out.replace(&format!("{{{}}}", i + 1), c);
    }
    out
}

/// Multiset of bare rule-id shapes: an uppercase run of >= 2, `-`, digits.
fn citations(lines: &[String]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for line in lines {
        let b: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < b.len() {
            if b[i].is_ascii_uppercase() {
                let s = i;
                while i < b.len() && (b[i].is_ascii_uppercase() || b[i].is_ascii_digit()) {
                    i += 1;
                }
                let word: String = b[s..i].iter().collect();
                if word.len() >= 2
                    && word.chars().all(|c| c.is_ascii_uppercase())
                    && i < b.len()
                    && b[i] == '-'
                    && i + 1 < b.len()
                    && b[i + 1].is_ascii_digit()
                {
                    let ds = i + 1;
                    i += 1;
                    while i < b.len() && b[i].is_ascii_digit() {
                        i += 1;
                    }
                    let num: String = b[ds..i].iter().collect();
                    *m.entry(format!("{word}-{num}")).or_insert(0) += 1;
                }
            } else {
                i += 1;
            }
        }
    }
    m
}

fn forms(lines: &[String]) -> BTreeMap<String, usize> {
    let names = [
        "SourceBytes",
        "SourceNode",
        "BundleRoot",
        "SourceCoordinate",
        "NodePath",
        "BundleRootExtent",
    ];
    let mut m = BTreeMap::new();
    for line in lines {
        for n in names {
            let c = line.matches(n).count();
            if c > 0 {
                *m.entry(n.to_string()).or_insert(0) += c;
            }
        }
    }
    m
}

fn bytes(lines: &[String]) -> usize {
    lines.iter().map(|l| l.len() + 1).sum()
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() < 4 {
        eprintln!("usage: verify-delta <spec> <DELTA-DIAG1.md> <DELTA-RATCHET.md> [--negative-control]");
        return ExitCode::from(2);
    }
    let negative = argv.iter().any(|a| a == "--negative-control");
    let negative_drop = argv.iter().any(|a| a == "--negative-drop");
    let spec = read(&argv[1]);
    let diag1_md = read(&argv[2]);
    let ratchet_md = read(&argv[3]);

    let mut fail: Vec<String> = Vec::new();
    let mut checks = 0usize;

    println!("== base ==");
    println!("spec               {}", argv[1]);
    println!("expected digest    {BASE_DIGEST}");
    println!("spec lines         {}", spec.len());
    println!(
        "DIAG-1 span        {DIAG1_FIRST}-{DIAG1_LAST}  ({} B)",
        bytes(&spec[DIAG1_FIRST - 1..DIAG1_LAST])
    );
    if negative {
        println!("MODE               negative control (one cell is corrupted on purpose)");
    }
    if negative_drop {
        println!("MODE               negative control (one retained sentence is dropped on purpose)");
    }

    // ---- parse DELTA-DIAG1 ----
    let db = blocks(&diag1_md);
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut scaffold: Vec<usize> = Vec::new();
    let mut connective: Vec<(usize, String, String)> = Vec::new();
    let mut new_prose: Vec<String> = Vec::new();

    for b in &db {
        match b.kind.as_str() {
            "hunk" => hunks.push(Hunk {
                id: b.args[0].clone(),
                start: b.args[1].parse().unwrap(),
                end: b.args[2].parse().unwrap(),
                lines: b.lines.clone(),
            }),
            "ledger" => {
                let fence = b.args[0].clone();
                let mut templates: BTreeMap<String, String> = BTreeMap::new();
                for line in &b.lines {
                    if let Some(rest) = line.strip_prefix("template ") {
                        let (key, text) = rest.split_once(' ').expect("template needs a key");
                        templates.insert(key.to_string(), text.to_string());
                        continue;
                    }
                    let cells = split_row(line);
                    let old_line: usize = cells[0].parse().unwrap_or_else(|_| {
                        panic!("ledger row does not start with a base line number: {line}")
                    });
                    let tkey = &cells[1];
                    let template = templates
                        .get(tkey)
                        .unwrap_or_else(|| panic!("undeclared template {tkey}"))
                        .clone();
                    entries.push(Entry {
                        fence: fence.clone(),
                        old_line,
                        template,
                        trail: cells[2].clone(),
                        cells: cells[3..].to_vec(),
                    });
                }
            }
            "scaffold" => {
                for line in &b.lines {
                    if !line.trim().is_empty() {
                        scaffold.push(line.trim().parse().unwrap());
                    }
                }
            }
            "connective" => {
                for line in &b.lines {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let (n, rest) = line.split_once(" :: ").expect("connective needs ::");
                    let (o, nw) = rest.split_once(" => ").expect("connective needs =>");
                    connective.push((n.trim().parse().unwrap(), o.to_string(), nw.to_string()));
                }
            }
            "new-prose" => new_prose.extend(b.lines.iter().cloned()),
            _ => {}
        }
    }

    if negative {
        // Corrupt exactly one cell: drop the final character of the first
        // entry's last cell. A verifier that still passes is decorative.
        let e = &mut entries[0];
        let last = e.cells.len() - 1;
        let mut c = e.cells[last].clone();
        c.pop();
        e.cells[last] = c;
    }

    // ---- build the new DIAG-1 block by applying hunks ----
    hunks.sort_by_key(|h| h.start);
    let mut new_block: Vec<String> = Vec::new();
    let mut cursor = DIAG1_FIRST;
    for h in &hunks {
        assert!(
            h.start >= cursor && h.end <= DIAG1_LAST,
            "hunk {} is out of order or outside DIAG-1",
            h.id
        );
        for n in cursor..h.start {
            new_block.push(spec[n - 1].clone());
        }
        new_block.extend(h.lines.iter().cloned());
        cursor = h.end + 1;
    }
    for n in cursor..=DIAG1_LAST {
        new_block.push(spec[n - 1].clone());
    }

    if negative_drop {
        // Delete one carried-through sentence, the silent-loss failure mode.
        // Only C3 can see this; if C3 stays green the coverage check is blind.
        let victim = spec[1662 - 1].clone();
        new_block.retain(|l| l != &victim);
    }

    // ---- C1: every moved base line reconstructs byte-for-byte ----
    println!("\n== C1 reconstruction of moved base lines ==");
    for e in &entries {
        checks += 1;
        let base = &spec[e.old_line - 1];
        let rebuilt = fill(&e.template, &e.cells);
        let want = format!("{rebuilt}{}", e.trail);
        let ok = &want == base;
        println!(
            "  {} {:>5}  trail={:<6} {}",
            if ok { "ok  " } else { "FAIL" },
            e.old_line,
            format!("\"{}\"", e.trail),
            e.fence
        );
        if !ok {
            fail.push(format!(
                "C1 line {}: reconstruction differs\n    base: {base}\n    new : {want}",
                e.old_line
            ));
        }
    }

    // ---- C2: every ledger row occurs verbatim in the new block ----
    println!("\n== C2 each row present in the new text ==");
    let mut c2_bad = 0;
    for e in &entries {
        checks += 1;
        let row = format!("| {} |", e.cells.join(" | "));
        if !new_block.iter().any(|l| l == &row) {
            c2_bad += 1;
            fail.push(format!("C2 line {}: row absent from new text: {row}", e.old_line));
        }
    }
    println!("  {} rows checked, {} absent", entries.len(), c2_bad);

    // ---- C3: every base line of DIAG-1 is accounted for ----
    println!("\n== C3 coverage of base lines {DIAG1_FIRST}-{DIAG1_LAST} ==");
    let moved: Vec<usize> = entries.iter().map(|e| e.old_line).collect();
    let mut counts = [0usize; 5]; // moved, scaffold, connective, retained, unclassified
    for n in DIAG1_FIRST..=DIAG1_LAST {
        let line = &spec[n - 1];
        checks += 1;
        if moved.contains(&n) {
            counts[0] += 1;
        } else if scaffold.contains(&n) {
            counts[1] += 1;
            assert!(
                line.trim() == "1." || line.trim() == "2." || line.trim() == "3.",
                "declared scaffold line {n} is not a bare list numeral: {line}"
            );
        } else if let Some((_, o, _)) = connective.iter().find(|(l, _, _)| *l == n) {
            counts[2] += 1;
            if o != line {
                fail.push(format!(
                    "C3 line {n}: declared connective old text differs from the file\n    file: {line}\n    decl: {o}"
                ));
            }
        } else if line.trim().is_empty() || new_block.iter().any(|l| l == line) {
            counts[3] += 1;
        } else {
            counts[4] += 1;
            fail.push(format!("C3 line {n} is unclassified (dropped?): {line}"));
        }
    }
    println!(
        "  moved {}  scaffold {}  connective {}  retained {}  unclassified {}",
        counts[0], counts[1], counts[2], counts[3], counts[4]
    );

    // ---- C4: every new line is declared ----
    println!("\n== C4 provenance of new lines ==");
    let base_set: Vec<&String> = spec[DIAG1_FIRST - 1..DIAG1_LAST].iter().collect();
    let rows: Vec<String> = entries
        .iter()
        .map(|e| format!("| {} |", e.cells.join(" | ")))
        .collect();
    let mut undeclared = 0;
    for line in &new_block {
        checks += 1;
        let known = line.trim().is_empty()
            || line.trim() == "```"
            || rows.contains(line)
            || new_prose.contains(line)
            || connective.iter().any(|(_, _, nw)| nw == line)
            || base_set.iter().any(|b| *b == line);
        if !known {
            undeclared += 1;
            fail.push(format!("C4 undeclared new line: {line}"));
        }
    }
    println!("  {} new lines, {} undeclared", new_block.len(), undeclared);

    // ---- C5/C6: mapping-set equality ----
    // Compared against the base block with the declared connective
    // substitutions applied, so the one declared wording change does not mask
    // an undeclared one: any other added or dropped citation still fails.
    let base_adjusted: Vec<String> = spec[DIAG1_FIRST - 1..DIAG1_LAST]
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let n = DIAG1_FIRST + i;
            match connective.iter().find(|(ln, _, _)| *ln == n) {
                Some((_, _, nw)) => nw.clone(),
                None => l.clone(),
            }
        })
        .collect();

    println!("\n== C5 rule-citation multiset ==");
    let old_c = citations(&base_adjusted);
    let new_c = citations(&new_block);
    checks += 1;
    if old_c == new_c {
        println!(
            "  ok    {} distinct ids, {} occurrences, identical before and after",
            old_c.len(),
            old_c.values().sum::<usize>()
        );
    } else {
        for (k, v) in &old_c {
            let n = new_c.get(k).copied().unwrap_or(0);
            if *v != n {
                fail.push(format!("C5 citation {k}: base {v}, new {n}"));
            }
        }
        for (k, v) in &new_c {
            if !old_c.contains_key(k) {
                fail.push(format!("C5 citation {k}: base 0, new {v}"));
            }
        }
    }

    println!("\n== C6 location-form multiset ==");
    let old_f = forms(&base_adjusted);
    let new_f = forms(&new_block);
    checks += 1;
    if old_f == new_f {
        for (k, v) in &old_f {
            println!("  ok    {k:<18} {v}");
        }
    } else {
        for (k, v) in &old_f {
            let n = new_f.get(k).copied().unwrap_or(0);
            if *v != n {
                fail.push(format!("C6 form {k}: base {v}, new {n}"));
            }
        }
    }

    // ---- C7: bytes ----
    println!("\n== C7 byte accounting ==");
    for h in &hunks {
        let before = bytes(&spec[h.start - 1..h.end]);
        let after = bytes(&h.lines);
        println!(
            "  {}  base {}-{}  {} B -> {} B  ({:+} B)",
            h.id,
            h.start,
            h.end,
            before,
            after,
            after as i64 - before as i64
        );
    }
    let d_before = bytes(&spec[DIAG1_FIRST - 1..DIAG1_LAST]);
    let d_after = bytes(&new_block);
    println!(
        "  DIAG-1 total        {} B -> {} B  ({:+} B)",
        d_before,
        d_after,
        d_after as i64 - d_before as i64
    );

    // ---- ratchet: cuts and quotes ----
    println!("\n== C8 ratchet cuts against the base file ==");
    let rb = blocks(&ratchet_md);
    let mut cuts: Vec<Cut> = Vec::new();
    let mut quotes: Vec<(usize, String)> = Vec::new();
    for b in &rb {
        match b.kind.as_str() {
            "cut" => {
                let mut old = String::new();
                let mut new = String::new();
                for line in &b.lines {
                    if let Some(r) = line.strip_prefix("old :: ") {
                        old = r.to_string();
                    }
                    if let Some(r) = line.strip_prefix("new :: ") {
                        new = r.to_string();
                    }
                }
                cuts.push(Cut {
                    id: b.args[0].clone(),
                    line: b.args[1].parse().unwrap(),
                    reference: b.args[2].parse().unwrap(),
                    old,
                    new,
                });
            }
            "quote-check" => {
                for line in &b.lines {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let (n, t) = line.split_once(" :: ").expect("quote needs ::");
                    quotes.push((n.trim().parse().unwrap(), t.to_string()));
                }
            }
            _ => {}
        }
    }
    let mut saved: i64 = 0;
    for c in &cuts {
        checks += 1;
        let base = &spec[c.line - 1];
        let ok = &c.old == base;
        let delta = (c.new.len() + 1) as i64 - (c.old.len() + 1) as i64;
        saved += delta;
        println!(
            "  {} {} line {:>5} ref {:>5}  {} B -> {} B  ({:+} B)",
            if ok { "ok  " } else { "FAIL" },
            c.id,
            c.line,
            c.reference,
            c.old.len() + 1,
            c.new.len() + 1,
            delta
        );
        if !ok {
            fail.push(format!(
                "C8 {} line {}: declared old text differs from the file\n    file: {base}\n    decl: {}",
                c.id, c.line, c.old
            ));
        }
        checks += 1;
        if c.reference < 1 || c.reference > spec.len() {
            fail.push(format!("C8 {}: reference line {} out of range", c.id, c.reference));
        }
    }
    println!("  ratchet total {saved:+} B");

    println!("\n== C9 quoted base sentences ==");
    let mut q_bad = 0;
    for (n, t) in &quotes {
        checks += 1;
        if !spec[n - 1].contains(t.as_str()) {
            q_bad += 1;
            fail.push(format!(
                "C9 line {n}: quoted fragment not present\n    file: {}\n    decl: {t}",
                spec[n - 1]
            ));
        }
    }
    println!("  {} quotations checked, {} not found", quotes.len(), q_bad);

    println!("\n== verdict ==");
    println!("checks run         {checks}");
    if fail.is_empty() {
        println!("failures           0");
        if negative || negative_drop {
            println!("\nNEGATIVE CONTROL DID NOT FAIL — the verifier is decorative. Fix it.");
            return ExitCode::from(1);
        }
        println!("\nPASS");
        println!("Established: each moved base line is reconstructible byte-for-byte from");
        println!("its row and the fence's declared reading; no base line was dropped; no");
        println!("new line is undeclared; the rule-citation and location-form multisets are");
        println!("unchanged; every ratchet quotation is the file's own bytes.");
        ExitCode::SUCCESS
    } else {
        println!("failures           {}", fail.len());
        for f in &fail {
            println!("\n{f}");
        }
        if negative || negative_drop {
            println!("\nNEGATIVE CONTROL FAILED AS REQUIRED — the checks have teeth.");
            return ExitCode::SUCCESS;
        }
        ExitCode::from(1)
    }
}
