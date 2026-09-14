//! Source allocation reuse and observed output batching in wfgrep.
//!
//! C2 retires QUAL-1..3's per-operation compiler-wrapper cost table. The
//! prelude bodies are linked implementations with one ordinary callable ABI.
//! These two independent program properties remain executable: each source
//! buffer allocates and initializes once, and a reusable output batch issues
//! one write per full batch. CASES.md records every retired assertion.

use super::deterministic_target::{HostScript, run_emitted_on_deterministic_host};
use super::{host_optimized_module, optimized_main};
use std::sync::OnceLock;

const WFGREP: &[u8] = include_bytes!("../../../../tests/programs/wfgrep.wf");

fn emitted() -> &'static str {
    static MODULE: OnceLock<String> = OnceLock::new();
    MODULE.get_or_init(|| super::compile(WFGREP))
}

fn program_functions() -> &'static [&'static str] {
    static FUNCTIONS: OnceLock<Vec<&'static str>> = OnceLock::new();
    FUNCTIONS.get_or_init(|| {
        let module = optimized();
        let mut functions = vec![entry()];
        for line in std::str::from_utf8(WFGREP)
            .expect("WF source is UTF-8")
            .lines()
        {
            let Some(signature) = line.strip_prefix("fn ") else {
                continue;
            };
            let name = signature.split(['(', '[', '<']).next().expect("fn name");
            if name == "main" {
                continue;
            }
            let needle = format!(" @wf_{name}(");
            if module
                .match_indices(&needle)
                .any(|(at, _)| definition_start(module, at).is_some())
            {
                functions.push(source_function(module, &format!("wf_{name}")));
            }
        }
        functions
    })
}

fn optimized() -> &'static str {
    static MODULE: OnceLock<String> = OnceLock::new();
    MODULE.get_or_init(|| host_optimized_module(emitted()))
}

fn entry() -> &'static str {
    optimized_main(optimized())
}

fn definition_start(module: &str, at: usize) -> Option<usize> {
    let line = module[..at].rfind('\n').map_or(0, |newline| newline + 1);
    module[line..at].starts_with("define").then_some(line)
}

fn call_argument<'line>(line: &'line str, callee: &str, wanted: usize) -> Option<&'line str> {
    let open = line.find(&format!("@{callee}("))? + callee.len() + 2;
    comma_item(&line[open..], wanted)
}

fn comma_item(rest: &str, wanted: usize) -> Option<&str> {
    let mut depth = 0_usize;
    let mut ordinal = 0_usize;
    let mut start = 0_usize;
    for (offset, character) in rest.char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => {
                return (ordinal == wanted).then(|| rest[start..offset].trim());
            }
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                if ordinal == wanted {
                    return Some(rest[start..offset].trim());
                }
                ordinal += 1;
                start = offset + 1;
            }
            _ => {}
        }
    }
    (ordinal == wanted).then(|| rest[start..].trim())
}

fn is_aggregate_destination<'module>(function: &'module str, mut pointer: &'module str) -> bool {
    let mut seen = Vec::new();
    loop {
        let first_parameter = function.lines().next().and_then(|header| {
            let (callee, _) = header.split_once('@')?.1.split_once('(')?;
            call_argument(header, callee, 0)?
                .split_whitespace()
                .next_back()
        });
        if pointer == "%wf.result" && first_parameter == Some(pointer) {
            return true;
        }
        if seen.contains(&pointer) {
            return false;
        }
        seen.push(pointer);
        let prefix = format!("  {pointer} = ");
        let Some(definition) = function.lines().find_map(|line| line.strip_prefix(&prefix)) else {
            return false;
        };
        if definition.starts_with("alloca ") {
            return true;
        }
        if !definition.starts_with("getelementptr ") {
            return false;
        }
        let Some((_, base)) = definition.split_once(", ptr ") else {
            return false;
        };
        let Some((base, _)) = base.split_once(',') else {
            return false;
        };
        pointer = base.trim();
    }
}

fn source_function<'module>(module: &'module str, symbol: &str) -> &'module str {
    let needle = format!(" @{symbol}(");
    let start = module
        .match_indices(&needle)
        .find_map(|(at, _)| definition_start(module, at))
        .unwrap_or_else(|| panic!("the module must define {symbol}"));
    let end = module[start..]
        .find("\n}\n")
        .map(|offset| start + offset + 2)
        .unwrap_or_else(|| panic!("the definition of {symbol} must close"));
    &module[start..end]
}

fn call_target(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if !(trimmed.starts_with("call ")
        || trimmed.starts_with("tail call ")
        || trimmed.contains(" = call ")
        || trimmed.contains(" = tail call "))
    {
        return None;
    }
    let at = trimmed.find('@')? + 1;
    let rest = &trimmed[at..];
    let end = rest.find(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '_' || character == '.')
    })?;
    rest[end..].starts_with('(').then(|| &rest[..end])
}

#[test]
fn the_reused_buffers_are_initialized_once_at_allocation() {
    // `wfgrep` asks for exactly eleven runs, and gets exactly eleven store
    // takes. Derived from source, function by function: `main` takes the
    // pattern (4096), the root name (256), the root path (1024), and the
    // diagnostic report (1280); `walk` takes its enumeration batch (8192),
    // its collected names (65664), its visit order (64 u64 slots, 512
    // bytes), one child path (1024), and its own report (1280);
    // `search_file` takes the read input (4096) and the publication batch
    // (8192).
    //
    // This is where the recursive search moved the cost: `walk`'s five and
    // `search_file`'s two are per call, so a walk of D directories holding F
    // files allocates 5D + 2F + 4 times, where the argv-list version allocated
    // four times for the whole run. That is a measured property of the new
    // shape, recorded here rather than hidden — it is one of the numbers the
    // flagship re-attribution has to explain.
    //
    // The source still owns initialization. With exclusive run rows LLVM can
    // fold malloc plus the zero-fill loop into calloc. Count either optimized
    // spelling, including both calloc factors, while retaining the exact
    // source-site and per-size counts, and one allocation per retained helper.
    let mut expanded = 0;
    let mut out_of_line = 0;
    let mut helper_takes = std::collections::BTreeMap::new();
    let mut helper_definitions = std::collections::BTreeSet::new();
    let mut retained_helpers = std::collections::BTreeSet::new();
    let mut sizes = std::collections::BTreeMap::new();
    for function in program_functions() {
        let signature = function.lines().next().unwrap_or_default();
        let helper = ["wf_zeroed_bytes", "wf_zeroed_words"]
            .into_iter()
            .find(|name| signature.contains(&format!(" @{name}(")));
        if let Some(helper) = helper {
            helper_definitions.insert(helper);
        }
        for line in function.lines() {
            if let Some(callee @ ("malloc" | "calloc")) = call_target(line) {
                if let Some(helper) = helper {
                    *helper_takes.entry(helper).or_insert(0) += 1;
                } else {
                    expanded += 1;
                    let size = |ordinal| {
                        call_argument(line, callee, ordinal)
                            .and_then(|argument| argument.split_whitespace().next_back())
                            .and_then(|value| value.parse::<u64>().ok())
                            .expect("an expanded source allocation has a constant extent")
                    };
                    let bytes = if callee == "calloc" {
                        size(0) * size(1)
                    } else {
                        size(0)
                    };
                    *sizes.entry(bytes).or_insert(0) += 1;
                }
            }
            if let Some(callee @ ("wf_zeroed_bytes" | "wf_zeroed_words")) = call_target(line) {
                out_of_line += 1;
                retained_helpers.insert(callee);
                // Extent is the helper's final source argument. Optimizers can
                // remove its unused provider argument but do not change that value.
                let count = (0..)
                    .map_while(|ordinal| call_argument(line, callee, ordinal))
                    .last()
                    .and_then(|argument| argument.split_whitespace().next_back())
                    .and_then(|value| value.parse::<u64>().ok())
                    .expect("a retained initialization call has its constant source extent");
                let bytes = count * if callee == "wf_zeroed_words" { 8 } else { 1 };
                *sizes.entry(bytes).or_insert(0) += 1;
            }
        }
    }
    assert_eq!(
        helper_takes,
        helper_definitions
            .iter()
            .copied()
            .map(|helper| (helper, 1))
            .collect(),
        "every retained initialization helper has exactly one allocation"
    );
    // Ordinary public definitions survive even when all of their calls have
    // been inlined. Validate definitions and remaining calls independently.
    assert!(retained_helpers.is_subset(&helper_definitions));
    assert_eq!(
        expanded + out_of_line,
        11,
        "eleven source runs, eleven store takes"
    );
    assert_eq!(
        sizes,
        std::collections::BTreeMap::from([
            (4096, 2),
            (8192, 2),
            (1024, 2),
            (1280, 2),
            (65664, 1),
            (512, 1),
            (256, 1),
        ]),
        "all eleven source takes retain their exact byte extents"
    );
    // Nothing reallocates, and nothing re-initializes. That the fill runs once
    // per take is a source fact under this surface rather than an allocator
    // guarantee: the fill loop is inside `zeroed_bytes` and `zeroed_words`,
    // between the take and the hand-back, so a caller cannot reach a filled
    // run without having taken it and cannot re-reach the fill without taking
    // another. Owned aggregate destinations also make LLVM retain memset
    // initializations of 40-byte inactive Options, 236-byte inactive Results
    // and 16-byte frame metadata. These do not refill a run's heap payload.
    // Keep the no-refill claim on pointer provenance, rather than forbidding
    // the unrelated aggregate initialization instruction by name.
    for forbidden in ["@realloc(", "@reallocf(", "bzero"] {
        assert!(
            !optimized().contains(forbidden),
            "the reused buffers must not reach {forbidden}"
        );
    }
    for definition in optimized().split("\ndefine ").skip(1) {
        let function = definition
            .split("\n}")
            .next()
            .expect("a definition has a body");
        for line in function.lines() {
            let Some(callee) = call_target(line).filter(|name| name.contains("memset")) else {
                continue;
            };
            let pointer = call_argument(line, callee, 0)
                .and_then(|argument| argument.split_whitespace().next_back())
                .expect("a memset has a destination");
            assert!(
                is_aggregate_destination(function, pointer),
                "bulk initialization must not reach reused heap run backing: {line}"
            );
        }
    }
    // Allocation begins in each function's prologue: the first allocation a
    // function makes precedes every host transfer it reaches, so no buffer is
    // first created in the drain, the match loop, or the flush.
    //
    // `search_file` remains a retained helper, reached after `main` opens the
    // fallback file or `walk` opens a child. Its two allocations are per file,
    // not per read. The per-buffer count above carries "one allocation per
    // source buffer"; this ordering check additionally requires allocation to
    // begin before that function's first emitted transfer.
    for function in program_functions() {
        let Some(first_allocation) = [
            "@malloc(",
            "@calloc(",
            "@wf_zeroed_bytes(",
            "@wf_zeroed_words(",
        ]
        .iter()
        .filter_map(|site| function.find(site))
        .min() else {
            continue;
        };
        for transfer in [
            "@wf_open_read(",
            "@wf_read_at(",
            "@wf_write_once(",
            "@wf_directory_next(",
        ] {
            let Some(first) = function.find(transfer) else {
                continue;
            };
            assert!(
                first_allocation < first,
                "allocation must begin before the first {transfer}"
            );
        }
    }
}

#[test]
fn the_output_batch_costs_one_host_write_per_full_batch() {
    // Three thousand matching lines of two bytes each, published through an
    // eight-thousand-one-hundred-and-ninety-two-byte batch. The scripted host
    // holds one regular file, so the search's root `open_directory` reports
    // `ENOTDIR` and the run reaches the file through the path route — which is
    // exactly the shape this row needs: one file, one read cursor, one batch.
    const MATCHES: u64 = 3_000;
    let mut fixture = Vec::new();
    for _ in 0..MATCHES {
        fixture.extend_from_slice(b"x\n");
    }

    let run = run_emitted_on_deterministic_host(
        emitted(),
        &HostScript::new().file(&fixture),
        &[b"x", b"lines.txt"],
    );
    assert_eq!(
        run.output.status.code(),
        Some(0),
        "every line matches; trace was {:?}",
        run.trace()
    );

    // The host echoes each accepted payload into its own trace, and this
    // program's payload is lines, so a trace entry is not reliably a trace
    // *line*. Counting occurrences of each marker instead is exact here
    // because no marker can occur inside a payload of `x` and newline.
    let trace = run.trace();
    let published: Vec<u64> = trace
        .match_indices("wf_test write fd=1 count=")
        .map(|(at, marker)| {
            trace[at + marker.len()..]
                .split_whitespace()
                .next()
                .expect("a write trace carries its requested count")
                .parse()
                .expect("a requested count is a number")
        })
        .collect();

    // A handful of host writes for three thousand matches: the batch fills and
    // is published whole, then the remainder is published on the way out. A
    // syscall-per-match implementation would show three thousand here, which
    // is exactly what §12.2 rejects.
    //
    // The exact split is no longer a fixed vector, because a published record
    // now carries the file's path and the line's ordinal, so record lengths
    // differ by the ordinal's digit count and a fixed vector would be a
    // transcription of this fixture rather than a property. What the row
    // asserts instead is the property itself: every batch but the last is
    // published within one record of full, the sum is exactly the output the
    // matches produced with no byte published twice, and the call count is two
    // orders of magnitude below the match count.
    const BATCH: u64 = 8_192;
    let expected: u64 = (1..=MATCHES)
        .map(|ordinal| {
            "lines.txt:".len() as u64 + ordinal.to_string().len() as u64 + ":x\n".len() as u64
        })
        .sum();
    assert!(published.len() >= 2, "trace was {trace:?}");
    assert!(
        (published.len() as u64) * 100 < MATCHES,
        "one write per full batch, not per match: {published:?}"
    );
    // The program flushes when the next record would not fit under its own
    // conservative reservation — the path length plus twenty-four bytes for
    // the ordinal and the two separators, plus the line's own span. Here that
    // is nine plus twenty-four plus two, so a full batch lands within
    // thirty-five bytes of the cap and never above it.
    const RESERVE: u64 = 40;
    for count in &published[..published.len() - 1] {
        assert!(
            *count > BATCH - RESERVE && *count <= BATCH,
            "a flushed batch must be within one record of full: {published:?}"
        );
    }
    // The batch is not refilled or re-initialized after a flush: each
    // publication starts from an empty batch and carries only what followed the
    // previous one, so the requested counts sum to exactly the output the
    // matches produced and no byte is published twice.
    assert_eq!(
        published.iter().sum::<u64>(),
        expected,
        "trace was {trace:?}"
    );

    // Both linear handles close explicitly. Affine output owners have empty drop.
    assert_eq!(
        trace.matches("wf_test close ").count(),
        2,
        "trace was {trace:?}"
    );
    assert_eq!(
        trace
            .lines()
            .filter(|line| line.starts_with("wf_test close fd=")
                && !line.starts_with("wf_test close fd=42 ")
                && line.ends_with("outcome=ok"))
            .count(),
        1
    );
    assert_eq!(trace.matches("wf_test close fd=42 outcome=ok").count(), 1);
    assert!(!trace.contains("wf_test close fd=1 "));
    assert!(!trace.contains("wf_test close fd=2 "));
    // Three reads for a two-chunk file: two that delivered bytes and one that
    // observed the end. One host attempt per source `read_at`, never a retry
    // and never a second attempt to confirm the end. Nothing was ever
    // published to standard error: the successful path reports nothing.
    // And the whole run is the §12.2 per-byte-call rejection, measured rather
    // than argued: six thousand bytes in and roughly fifty thousand out cost a
    // dozen host calls altogether. Not one host call per byte, per line, per
    // match, or per field.
    assert!(
        trace.matches("wf_test ").count() < 20,
        "the whole run is a dozen host calls: {trace:?}"
    );
    assert_eq!(
        trace.matches("wf_test pread fd=42 ").count(),
        3,
        "trace was {trace:?}"
    );
    // The refused directory open of the one regular file, then the path-route
    // open that reached it.
    assert_eq!(
        trace
            .lines()
            .filter(|line| line.starts_with("wf_test openat root=") && line.ends_with("-> notdir"))
            .count(),
        1,
        "trace was {trace:?}"
    );
    assert_eq!(
        trace
            .lines()
            .filter(|line| line.starts_with("wf_test openat root=") && line.ends_with("fd=42"))
            .count(),
        1,
        "trace was {trace:?}"
    );
    assert!(!trace.contains("wf_test write fd=2 "));
}
