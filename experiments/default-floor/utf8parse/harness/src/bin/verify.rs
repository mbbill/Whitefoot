//! Correctness oracle for a generated xlang UTF-8 event parser.

use std::env;
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt as _;
use std::process::{Command, ExitCode};
use std::str;

use xlang_utf8parse_rust_baseline::{parse_into, INVALID_EVENT};

#[repr(C)]
#[derive(Clone, Copy)]
struct Buf {
    p: *mut u8,
    n: i64,
}

unsafe extern "C" {
    fn xlang_parse_facts(out: Buf, src: Buf) -> u64;
    fn xlang_parse_nofacts(out: Buf, src: Buf) -> u64;
}

type ParserFn = unsafe extern "C" fn(Buf, Buf) -> u64;
const CANARY: u32 = 0xA5A5_A5A5;
const GUARD: usize = 32;
const EXPECTED_CASES: usize = 84_041;
const TRANSITION_FLUSH: &[u8; 4] = &[0x80, 0x80, 0x80, 0x41];

/// Specification oracle represented as a pending byte sequence, deliberately
/// independent of the shipped parser's state and action tables.
#[derive(Clone, Copy)]
struct Oracle {
    pending: [u8; 4],
    pending_len: usize,
    expected_len: usize,
}

impl Oracle {
    fn new() -> Self {
        Self {
            pending: [0; 4],
            pending_len: 0,
            expected_len: 0,
        }
    }

    fn reset(&mut self) {
        self.pending_len = 0;
        self.expected_len = 0;
    }

    fn advance(&mut self, byte: u8) -> Option<u32> {
        if self.pending_len == 0 {
            if byte <= 0x7f {
                return Some(u32::from(byte));
            }
            self.expected_len = match byte {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => return Some(INVALID_EVENT),
            };
            self.pending[0] = byte;
            self.pending_len = 1;
            return None;
        }

        let lead = self.pending[0];
        let acceptable = if self.pending_len == 1 {
            match lead {
                0xe0 => (0xa0..=0xbf).contains(&byte),
                0xed => (0x80..=0x9f).contains(&byte),
                0xf0 => (0x90..=0xbf).contains(&byte),
                0xf4 => (0x80..=0x8f).contains(&byte),
                _ => (0x80..=0xbf).contains(&byte),
            }
        } else {
            (0x80..=0xbf).contains(&byte)
        };
        if !acceptable {
            self.reset();
            return Some(INVALID_EVENT);
        }

        self.pending[self.pending_len] = byte;
        self.pending_len += 1;
        if self.pending_len != self.expected_len {
            return None;
        }

        let point = match self.expected_len {
            2 => (u32::from(self.pending[0] & 0x1f) << 6) | u32::from(self.pending[1] & 0x3f),
            3 => {
                (u32::from(self.pending[0] & 0x0f) << 12)
                    | (u32::from(self.pending[1] & 0x3f) << 6)
                    | u32::from(self.pending[2] & 0x3f)
            }
            4 => {
                (u32::from(self.pending[0] & 0x07) << 18)
                    | (u32::from(self.pending[1] & 0x3f) << 12)
                    | (u32::from(self.pending[2] & 0x3f) << 6)
                    | u32::from(self.pending[3] & 0x3f)
            }
            _ => unreachable!("only two-, three-, and four-byte sequences are pending"),
        };
        self.reset();
        Some(point)
    }
}

fn independent_oracle(src: &[u8]) -> Vec<u32> {
    let mut oracle = Oracle::new();
    let mut events = Vec::with_capacity(src.len());
    for &byte in src {
        if let Some(event) = oracle.advance(byte) {
            events.push(event);
        }
    }
    // There is intentionally no EOF action: a trailing pending prefix emits
    // no event, matching the task's single-buffer contract.
    events
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn hex_events(events: &[u32]) -> String {
    let mut text = String::with_capacity(events.len() * 7);
    for (index, event) in events.iter().enumerate() {
        use std::fmt::Write as _;
        if index != 0 {
            text.push(',');
        }
        let _ = write!(text, "{event:06x}");
    }
    text
}

struct XlangObservation {
    returned: u64,
    actual: Vec<u32>,
    sentinel_unchanged: bool,
    source_unchanged: bool,
}

fn call_xlang(parser: ParserFn, src: &[u8], visible_len: usize) -> XlangObservation {
    let mut source = src.to_vec();
    let source_before = source.clone();
    let mut storage = vec![CANARY; GUARD + visible_len + GUARD];
    let out = &mut storage[GUARD..GUARD + visible_len];
    let returned = unsafe {
        parser(
            Buf {
                p: out.as_mut_ptr().cast::<u8>(),
                n: i64::try_from(out.len()).expect("bounded output length fits i64"),
            },
            Buf {
                p: source.as_mut_ptr(),
                n: i64::try_from(source.len()).expect("bounded source length fits i64"),
            },
        )
    };
    let produced = usize::try_from(returned).ok();
    let actual_len = produced.unwrap_or(visible_len).min(visible_len);
    let sentinel_unchanged = storage[..GUARD].iter().all(|&event| event == CANARY)
        && produced.is_some_and(|length| {
            length <= visible_len
                && storage[GUARD + length..]
                    .iter()
                    .all(|&event| event == CANARY)
        });

    XlangObservation {
        returned,
        actual: storage[GUARD..GUARD + actual_len].to_vec(),
        sentinel_unchanged,
        source_unchanged: source == source_before,
    }
}

fn check_rust_case(label: &str, src: &[u8]) -> Result<(), String> {
    let expected = independent_oracle(src);
    for (capacity_mode, visible_len) in [("exact", src.len()), ("surplus-32", src.len() + 32)] {
        let source = src.to_vec();
        let source_before = source.clone();
        let mut storage = vec![CANARY; GUARD + visible_len + GUARD];
        let out = &mut storage[GUARD..GUARD + visible_len];
        let produced = parse_into(out, &source).map_err(|_| {
            format!(
                "{label}: shipped Rust rejected capacity_mode={capacity_mode} output_capacity={visible_len}"
            )
        })?;
        if produced != expected.len() || out[..produced] != expected {
            return Err(format!(
                "{label}: independent oracle disagrees with shipped Rust: capacity_mode={capacity_mode} output_capacity={visible_len} input={} expected={} actual={}",
                hex_bytes(src),
                hex_events(&expected),
                hex_events(&out[..produced]),
            ));
        }
        if storage[..GUARD].iter().any(|&event| event != CANARY)
            || storage[GUARD + produced..]
                .iter()
                .any(|&event| event != CANARY)
        {
            return Err(format!(
                "{label}: shipped Rust modified a guard or unused suffix: capacity_mode={capacity_mode} output_capacity={visible_len}"
            ));
        }
        if source != source_before {
            return Err(format!(
                "{label}: shipped Rust mutated the source buffer: capacity_mode={capacity_mode} output_capacity={visible_len}"
            ));
        }
    }
    Ok(())
}

fn check_xlang_variant(
    label: &str,
    name: &str,
    parser: ParserFn,
    src: &[u8],
    expected: &[u32],
) -> Result<(), String> {
    for (capacity_mode, visible_len) in [("exact", src.len()), ("surplus-32", src.len() + 32)] {
        let observed = call_xlang(parser, src, visible_len);
        let returned_matches =
            usize::try_from(observed.returned).is_ok_and(|returned| returned == expected.len());
        if !returned_matches
            || observed.actual != expected
            || !observed.sentinel_unchanged
            || !observed.source_unchanged
        {
            return Err(format!(
                "{label}/{name}: capacity_mode={capacity_mode} output_capacity={visible_len} input={} expected={} actual={} returned={} expected_length={} sentinel_unchanged={} source_unchanged={}",
                hex_bytes(src),
                hex_events(expected),
                hex_events(&observed.actual),
                observed.returned,
                expected.len(),
                observed.sentinel_unchanged,
                observed.source_unchanged,
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct XorShift64Star(u64);

impl XorShift64Star {
    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.0 = value;
        value.wrapping_mul(2_685_821_657_736_338_717)
    }
}

const STATE_PREFIXES: &[&[u8]] = &[
    &[0xc2],
    &[0xdf],
    &[0xe0],
    &[0xe1],
    &[0xec],
    &[0xed],
    &[0xee],
    &[0xef],
    &[0xf0],
    &[0xf1],
    &[0xf3],
    &[0xf4],
    &[0xe0, 0xa0],
    &[0xe0, 0xbf],
    &[0xe1, 0x80],
    &[0xec, 0xbf],
    &[0xed, 0x80],
    &[0xed, 0x9f],
    &[0xee, 0x80],
    &[0xef, 0xbf],
    &[0xf0, 0x90],
    &[0xf0, 0xbf],
    &[0xf1, 0x80],
    &[0xf3, 0xbf],
    &[0xf4, 0x80],
    &[0xf4, 0x8f],
    &[0xf0, 0x90, 0x80],
    &[0xf0, 0xbf, 0xbf],
    &[0xf1, 0x80, 0x80],
    &[0xf3, 0xbf, 0xbf],
    &[0xf4, 0x80, 0x80],
    &[0xf4, 0x8f, 0xbf],
];

const FIXED_CASES: &[&[u8]] = &[
    &[0x00, 0x7f],
    &[0xc2, 0x80],
    &[0xdf, 0xbf],
    &[0xe0, 0xa0, 0x80],
    &[0xe0, 0xbf, 0xbf],
    &[0xed, 0x9f, 0xbf],
    &[0xee, 0x80, 0x80],
    &[0xef, 0xbf, 0xbf],
    &[0xf0, 0x90, 0x80, 0x80],
    &[0xf0, 0xbf, 0xbf, 0xbf],
    &[0xf4, 0x80, 0x80, 0x80],
    &[0xf4, 0x8f, 0xbf, 0xbf],
    &[
        0x41, 0xc2, 0xa2, 0xe2, 0x82, 0xac, 0xf0, 0x9f, 0x92, 0xa9, 0x5a,
    ],
    &[0xc2, 0x41, 0x42],
    &[0xe2, 0x82, 0x41, 0x42],
    &[0xf0, 0x90, 0x80, 0x41, 0x42],
    &[0xe0, 0x9f, 0x41],
    &[0xed, 0xa0, 0x41],
    &[0xf0, 0x8f, 0x41],
    &[0xf4, 0x90, 0x41],
    &[0xc2, 0xc2, 0x80],
    &[0xe1, 0x80, 0xe1, 0x80, 0x80],
    &[0x80, 0xbf, 0xc0, 0xc1, 0xf5, 0xff],
    &[0x41, 0xc2],
];

fn fuzz_byte(random: u64) -> u8 {
    let sample = ((random >> 8) & 0xff) as u8;
    match random & 7 {
        0 => sample & 0x7f,
        1 => 0x80 | (sample & 0x3f),
        2 => 0xc2 + (sample % 30),
        3 => 0xe0 + (sample & 0x0f),
        4 => 0xf0 + (sample % 5),
        5 => [0xc0, 0xc1, 0xf5, 0xff][((random >> 16) & 3) as usize],
        _ => sample,
    }
}

fn visit_cases<E>(mut visit: impl FnMut(&str, &[u8]) -> Result<(), E>) -> Result<usize, E> {
    let mut cases = 0;
    visit("empty", b"")?;
    cases += 1;

    for byte in 0_u16..=255 {
        visit(&format!("singleton-{byte:02x}"), &[byte as u8])?;
        cases += 1;
    }

    for first in 0_u16..=255 {
        for second in 0_u16..=255 {
            visit(
                &format!("pair-{first:02x}-{second:02x}"),
                &[first as u8, second as u8],
            )?;
            cases += 1;
        }
    }

    for (prefix_index, prefix) in STATE_PREFIXES.iter().enumerate() {
        visit(&format!("state-prefix-{prefix_index:02}-eof"), prefix)?;
        cases += 1;
        for next in 0_u16..=255 {
            let mut case = Vec::with_capacity(prefix.len() + 1 + TRANSITION_FLUSH.len());
            case.extend_from_slice(prefix);
            case.push(next as u8);
            case.extend_from_slice(TRANSITION_FLUSH);
            visit(
                &format!("state-prefix-{prefix_index:02}-next-{next:02x}-flush"),
                &case,
            )?;
            cases += 1;
        }
    }

    for (index, case) in FIXED_CASES.iter().enumerate() {
        visit(&format!("fixed-{index:02}"), case)?;
        cases += 1;
    }

    let mut rng = XorShift64Star(0x5554_4638_5041_5253);
    for index in 0..10_000 {
        let size = (rng.next() % 2049) as usize;
        let mut case = Vec::with_capacity(size);
        for _ in 0..size {
            case.push(fuzz_byte(rng.next()));
        }
        visit(&format!("fuzz-{index:05}-len-{size}"), &case)?;
        cases += 1;
    }

    Ok(cases)
}

#[derive(Debug, Eq, PartialEq)]
struct ProgressSummary {
    calls: usize,
    last_case_index: usize,
    last_variant: &'static str,
}

fn parse_worker_progress(stdout: &[u8]) -> Result<ProgressSummary, String> {
    let text = str::from_utf8(stdout).map_err(|error| format!("progress is not UTF-8: {error}"))?;
    if text.is_empty() {
        return Err("worker emitted no progress".to_string());
    }
    if !text.ends_with('\n') {
        return Err("worker progress lacks its final newline".to_string());
    }
    let mut calls = 0;
    let mut last_case_index = 0;
    let mut last_variant = "";
    for (ordinal, line) in text.split_terminator('\n').enumerate() {
        let (case_text, variant) = line
            .split_once('\t')
            .ok_or_else(|| format!("progress row {ordinal} lacks one tab"))?;
        if variant.contains('\t') {
            return Err(format!("progress row {ordinal} contains extra fields"));
        }
        let expected_case = ordinal / 2;
        if expected_case >= EXPECTED_CASES || case_text != expected_case.to_string() {
            return Err(format!(
                "progress row {ordinal} names case {case_text}, expected {expected_case}"
            ));
        }
        let expected_variant = if ordinal % 2 == 0 {
            "facts-on"
        } else {
            "facts-off"
        };
        if variant != expected_variant {
            return Err(format!(
                "progress row {ordinal} names variant {variant:?}, expected {expected_variant:?}"
            ));
        }
        calls += 1;
        last_case_index = expected_case;
        last_variant = expected_variant;
    }
    Ok(ProgressSummary {
        calls,
        last_case_index,
        last_variant,
    })
}

enum CaseLookupStop {
    Found { label: String, src: Vec<u8> },
}

fn corpus_case_at(target: usize) -> Result<(String, Vec<u8>), String> {
    let mut index = 0;
    let outcome: Result<usize, CaseLookupStop> = visit_cases(|label, src| {
        if index == target {
            return Err(CaseLookupStop::Found {
                label: label.to_string(),
                src: src.to_vec(),
            });
        }
        index += 1;
        Ok(())
    });
    match outcome {
        Err(CaseLookupStop::Found { label, src }) => Ok((label, src)),
        Ok(cases) => Err(format!(
            "case index {target} is outside the generated corpus of {cases} cases"
        )),
    }
}

fn signal_termination_message(label: &str, src: &[u8], variant: &str, signal: i32) -> String {
    let expected = independent_oracle(src);
    format!(
        "{label}/{variant}: capacity_mode=unavailable output_capacity=unavailable input={} expected={} expected_length={} expected_termination=success actual=unavailable actual_termination=signal-{signal} returned=unavailable sentinel_unchanged=unavailable source_unchanged=unavailable",
        hex_bytes(src),
        hex_events(&expected),
        expected.len(),
    )
}

enum WorkerFailure {
    Candidate(String),
    Harness(String),
}

fn run_xlang_worker() -> ExitCode {
    let stdout = io::stdout();
    let mut progress = stdout.lock();
    let mut case_index = 0;
    let outcome = visit_cases(|label, src| {
        let expected = independent_oracle(src);
        for (name, parser) in [
            ("facts-on", xlang_parse_facts as ParserFn),
            ("facts-off", xlang_parse_nofacts as ParserFn),
        ] {
            writeln!(progress, "{case_index}\t{name}")
                .and_then(|()| progress.flush())
                .map_err(|error| {
                    WorkerFailure::Harness(format!("write worker progress: {error}"))
                })?;
            check_xlang_variant(label, name, parser, src, &expected)
                .map_err(WorkerFailure::Candidate)?;
        }
        case_index += 1;
        Ok::<(), WorkerFailure>(())
    });

    match outcome {
        Ok(cases) if cases == EXPECTED_CASES && case_index == EXPECTED_CASES => ExitCode::SUCCESS,
        Ok(cases) => {
            eprintln!(
                "HARNESS: xlang worker generated {cases} cases and completed {case_index}, expected {EXPECTED_CASES}"
            );
            ExitCode::from(2)
        }
        Err(WorkerFailure::Candidate(error)) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
        Err(WorkerFailure::Harness(error)) => {
            eprintln!("HARNESS: {error}");
            ExitCode::from(2)
        }
    }
}

fn harness_worker_failure(message: &str, stderr: &[u8]) -> ExitCode {
    eprintln!("harness xlang worker failure: {message}");
    if !stderr.is_empty() {
        eprintln!("worker stderr:\n{}", String::from_utf8_lossy(stderr));
    }
    ExitCode::from(2)
}

fn run_parent() -> ExitCode {
    match visit_cases(check_rust_case) {
        Ok(EXPECTED_CASES) => {}
        Ok(cases) => {
            eprintln!("harness preflight generated {cases} cases, expected {EXPECTED_CASES}");
            return ExitCode::from(2);
        }
        Err(error) => {
            eprintln!("harness Rust/oracle preflight failed: {error}");
            return ExitCode::from(2);
        }
    }

    let executable = match env::current_exe() {
        Ok(executable) => executable,
        Err(error) => {
            eprintln!("harness could not locate the verifier executable: {error}");
            return ExitCode::from(2);
        }
    };
    let worker = match Command::new(executable).arg("--xlang-worker").output() {
        Ok(worker) => worker,
        Err(error) => {
            eprintln!("harness could not start the xlang worker: {error}");
            return ExitCode::from(2);
        }
    };
    let progress = match parse_worker_progress(&worker.stdout) {
        Ok(progress) => progress,
        Err(error) => {
            return harness_worker_failure(&format!("invalid progress: {error}"), &worker.stderr)
        }
    };

    match worker.status.code() {
        Some(0) => {
            if !worker.stderr.is_empty()
                || progress.calls != EXPECTED_CASES * 2
                || progress.last_case_index != EXPECTED_CASES - 1
                || progress.last_variant != "facts-off"
            {
                return harness_worker_failure(
                    "successful worker did not complete the frozen progress sequence cleanly",
                    &worker.stderr,
                );
            }
            println!("correct cases={EXPECTED_CASES}");
            ExitCode::SUCCESS
        }
        Some(1) => {
            let stderr = match str::from_utf8(&worker.stderr) {
                Ok(stderr) if !stderr.is_empty() => stderr,
                Ok(_) => {
                    return harness_worker_failure(
                        "candidate-failure worker emitted no diagnostic",
                        &worker.stderr,
                    )
                }
                Err(error) => {
                    return harness_worker_failure(
                        &format!("candidate diagnostic is not UTF-8: {error}"),
                        &worker.stderr,
                    )
                }
            };
            let (label, src) = match corpus_case_at(progress.last_case_index) {
                Ok(case) => case,
                Err(error) => return harness_worker_failure(&error, &worker.stderr),
            };
            let expected_prefix = format!("{label}/{}: capacity_mode=", progress.last_variant);
            let input_binding = format!(" input={} expected=", hex_bytes(&src));
            if !stderr.starts_with(&expected_prefix)
                || !stderr.contains(&input_binding)
                || !stderr.ends_with('\n')
            {
                return harness_worker_failure(
                    "candidate diagnostic does not bind the last progress record",
                    &worker.stderr,
                );
            }
            if let Err(error) = io::stderr().write_all(&worker.stderr) {
                eprintln!("harness could not forward the candidate diagnostic: {error}");
                return ExitCode::from(2);
            }
            ExitCode::from(1)
        }
        Some(code) => harness_worker_failure(
            &format!("worker exited with unexpected status {code}"),
            &worker.stderr,
        ),
        None => {
            let Some(signal) = worker.status.signal() else {
                return harness_worker_failure(
                    "worker terminated without an exit code or signal",
                    &worker.stderr,
                );
            };
            let (label, src) = match corpus_case_at(progress.last_case_index) {
                Ok(case) => case,
                Err(error) => return harness_worker_failure(&error, &worker.stderr),
            };
            eprintln!(
                "{}",
                signal_termination_message(&label, &src, progress.last_variant, signal)
            );
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    match (arguments.next(), arguments.next()) {
        (None, None) => run_parent(),
        (Some(flag), None) if flag == "--xlang-worker" => run_xlang_worker(),
        _ => {
            eprintln!("harness verifier accepts only the internal --xlang-worker flag");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_covers_valid_boundaries_and_invalid_consumption() {
        assert_eq!(
            independent_oracle(&[0x41, 0xc2, 0xa2, 0xf4, 0x8f, 0xbf, 0xbf]),
            [0x41, 0xa2, 0x10ffff]
        );
        assert_eq!(independent_oracle(&[0xc2, 0x41]), [INVALID_EVENT]);
        assert_eq!(
            independent_oracle(&[0xe0, 0x9f, 0x41]),
            [INVALID_EVENT, 0x41]
        );
        assert!(independent_oracle(&[0xf0, 0x90, 0x80]).is_empty());
    }

    #[test]
    fn corpus_count_and_landmarks_are_frozen() {
        assert_eq!(
            visit_cases(|_, _| Ok::<(), ()>(())).unwrap(),
            EXPECTED_CASES
        );
        assert_eq!(corpus_case_at(0).unwrap(), ("empty".to_string(), vec![]));
        assert_eq!(
            corpus_case_at(65_793).unwrap(),
            ("state-prefix-00-eof".to_string(), vec![0xc2])
        );
        assert_eq!(
            corpus_case_at(74_017).unwrap(),
            ("fixed-00".to_string(), vec![0x00, 0x7f])
        );
        assert!(corpus_case_at(EXPECTED_CASES).is_err());
    }

    #[test]
    fn rust_preflight_covers_both_capacities_for_the_full_corpus() {
        assert_eq!(
            visit_cases(check_rust_case).expect("Rust and independent oracle must agree"),
            EXPECTED_CASES
        );
    }

    #[test]
    fn diagnostic_encodings_include_complete_values() {
        let bytes: Vec<u8> = (0..=255).cycle().take(4_096).collect();
        let encoded = hex_bytes(&bytes);
        assert_eq!(encoded.len(), bytes.len() * 2);
        assert_eq!(&encoded[..8], "00010203");
        assert_eq!(&encoded[encoded.len() - 8..], "fcfdfeff");
        assert_eq!(
            hex_events(&[0, 0x10ffff, INVALID_EVENT]),
            "000000,10ffff,110000"
        );
    }

    #[test]
    fn progress_parser_accepts_only_stable_sequence_prefixes() {
        assert_eq!(
            parse_worker_progress(b"0\tfacts-on\n0\tfacts-off\n1\tfacts-on\n").unwrap(),
            ProgressSummary {
                calls: 3,
                last_case_index: 1,
                last_variant: "facts-on",
            }
        );
        assert!(parse_worker_progress(b"0\tfacts-off\n").is_err());
        assert!(parse_worker_progress(b"0\tfacts-on").is_err());
    }
}
