#![forbid(unsafe_code)]

use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

const WARMUPS: usize = 3;
const REPETITIONS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Record {
    ordinal: u64,
    start: u64,
    end: u64,
    line_start: u64,
    line_end: u64,
    line_number: u64,
}

fn digest_word(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(1_099_511_628_211)
}

fn oracle_records(haystack: &[u8], needle: &[u8]) -> Vec<Record> {
    let mut records = Vec::new();
    if needle.is_empty() || needle.contains(&b'\n') {
        return records;
    }
    let mut line_number = 1_u64;
    let mut line_start = 0_usize;
    while line_start < haystack.len() {
        let relative_end = haystack[line_start..]
            .iter()
            .position(|byte| *byte == b'\n');
        let line_end = relative_end.map_or(haystack.len(), |offset| line_start + offset);
        let mut start = line_start;
        while needle.len() <= line_end.saturating_sub(start) {
            let end = start + needle.len();
            if haystack[start..end] == *needle {
                records.push(Record {
                    ordinal: records.len() as u64,
                    start: start as u64,
                    end: end as u64,
                    line_start: line_start as u64,
                    line_end: line_end as u64,
                    line_number,
                });
                start = end;
            } else {
                start += 1;
            }
        }
        line_start = if line_end < haystack.len() {
            line_end + 1
        } else {
            haystack.len()
        };
        line_number += 1;
    }
    records
}

fn digest_records(records: &[Record], haystack_length: usize, needle_length: usize) -> u64 {
    let mut digest = 14_695_981_039_346_656_037_u64;
    for record in records {
        digest = digest_word(digest, record.ordinal);
        digest = digest_word(digest, record.start);
        digest = digest_word(digest, record.end);
        digest = digest_word(digest, record.line_start);
        digest = digest_word(digest, record.line_end);
        digest = digest_word(digest, record.line_number);
    }
    digest = digest_word(digest, records.len() as u64);
    digest = digest_word(digest, haystack_length as u64);
    digest_word(digest, needle_length as u64)
}

fn data_hash(data: &[u8]) -> u64 {
    data.iter()
        .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
            digest_word(hash, u64::from(*byte))
        })
}

fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    if input.len() % 2 != 0 || !input.is_ascii() {
        return Err("needle hex must contain an even number of ASCII digits".to_owned());
    }
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|error| error.to_string())?;
            u8::from_str_radix(text, 16).map_err(|error| format!("invalid needle hex: {error}"))
        })
        .collect()
}

fn verify_case<K>(
    prepare: &impl Fn(&[u8]) -> K,
    scan: &impl Fn(&K, &[u8], &[u8]) -> u64,
    haystack: &[u8],
    needle: &[u8],
) -> Result<(), String> {
    let records = oracle_records(haystack, needle);
    let expected = digest_records(&records, haystack.len(), needle.len());
    let kernel = prepare(needle);
    let observed = scan(&kernel, haystack, needle);
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "correctness failure: haystack={haystack:?} needle={needle:?} records={records:?} expected={expected} observed={observed}"
        ))
    }
}

fn verify_expected<K>(
    prepare: &impl Fn(&[u8]) -> K,
    scan: &impl Fn(&K, &[u8], &[u8]) -> u64,
    haystack: &[u8],
    needle: &[u8],
    expected_records: &[Record],
) -> Result<(), String> {
    let observed_records = oracle_records(haystack, needle);
    if observed_records != expected_records {
        return Err(format!(
            "oracle record failure: expected={expected_records:?} observed={observed_records:?}"
        ));
    }
    verify_case(prepare, scan, haystack, needle)
}

fn enumerate_words(alphabet: &[u8], length: usize, mut visit: impl FnMut(&[u8])) {
    let count = alphabet.len().pow(length as u32);
    let mut word = vec![0_u8; length];
    for mut ordinal in 0..count {
        for byte in word.iter_mut().rev() {
            *byte = alphabet[ordinal % alphabet.len()];
            ordinal /= alphabet.len();
        }
        visit(&word);
    }
}

fn run_checks<K>(
    prepare: &impl Fn(&[u8]) -> K,
    scan: &impl Fn(&K, &[u8], &[u8]) -> u64,
) -> Result<(), String> {
    verify_expected(
        prepare,
        scan,
        b"xx",
        b"x",
        &[
            Record {
                ordinal: 0,
                start: 0,
                end: 1,
                line_start: 0,
                line_end: 2,
                line_number: 1,
            },
            Record {
                ordinal: 1,
                start: 1,
                end: 2,
                line_start: 0,
                line_end: 2,
                line_number: 1,
            },
        ],
    )?;
    verify_expected(
        prepare,
        scan,
        b"a\n\nb",
        b"b",
        &[Record {
            ordinal: 0,
            start: 3,
            end: 4,
            line_start: 3,
            line_end: 4,
            line_number: 3,
        }],
    )?;
    verify_expected(
        prepare,
        scan,
        b"aaaaa",
        b"aaa",
        &[Record {
            ordinal: 0,
            start: 0,
            end: 3,
            line_start: 0,
            line_end: 5,
            line_number: 1,
        }],
    )?;
    verify_expected(
        prepare,
        scan,
        b"abaaba",
        b"aba",
        &[
            Record {
                ordinal: 0,
                start: 0,
                end: 3,
                line_start: 0,
                line_end: 6,
                line_number: 1,
            },
            Record {
                ordinal: 1,
                start: 3,
                end: 6,
                line_start: 0,
                line_end: 6,
                line_number: 1,
            },
        ],
    )?;
    verify_expected(
        prepare,
        scan,
        b"\0\r\xffx\n",
        b"\r\xff",
        &[Record {
            ordinal: 0,
            start: 1,
            end: 3,
            line_start: 0,
            line_end: 4,
            line_number: 1,
        }],
    )?;
    let cases: &[(&[u8], &[u8])] = &[
        (b"", b"x"),
        (b"x", b""),
        (b"a\nb", b"a\nb"),
        (b"x", b"x"),
        (b"xx", b"x"),
        (b"ab\ncd", b"bc"),
        (b"first needle last", b"needle"),
        (b"needle\nnone\nneedle", b"needle"),
        (b"aaaaa", b"aaa"),
        (b"aaaaaaaaaaaaaaaaab", b"aaaaaaaab"),
        (b"short\nneedle\n", b"needle"),
        (b"\xd0\xa8x", b"\xd0\xa8"),
    ];
    for (haystack, needle) in cases {
        verify_case(prepare, scan, haystack, needle)?;
    }

    let alphabet = [b'a', b'b', b'\n'];
    for haystack_length in 0..=7 {
        let mut failure = None;
        enumerate_words(&alphabet, haystack_length, |haystack| {
            for needle_length in 1..=4 {
                enumerate_words(&alphabet[..2], needle_length, |needle| {
                    if failure.is_none() {
                        failure = verify_case(prepare, scan, haystack, needle).err();
                    }
                });
            }
        });
        if let Some(message) = failure {
            return Err(message);
        }
    }

    let mut hostile = vec![b'a'; 8 * 1024 * 1024];
    for block_end in (4095..hostile.len()).step_by(4096) {
        hostile[block_end] = b'\n';
    }
    let hostile_needle = [b'a'; 31].into_iter().chain([b'b']).collect::<Vec<_>>();
    for offset in [1_024, 4_195_328] {
        hostile[offset + 31] = b'b';
    }
    verify_expected(
        prepare,
        scan,
        &hostile,
        &hostile_needle,
        &[
            Record {
                ordinal: 0,
                start: 1_024,
                end: 1_056,
                line_start: 0,
                line_end: 4_095,
                line_number: 1,
            },
            Record {
                ordinal: 1,
                start: 4_195_328,
                end: 4_195_360,
                line_start: 4_194_304,
                line_end: 4_198_399,
                line_number: 1_025,
            },
        ],
    )
}

fn load(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn run_input<K>(
    prepare: &impl Fn(&[u8]) -> K,
    scan: &impl Fn(&K, &[u8], &[u8]) -> u64,
    path: &Path,
    needle_hex: &str,
    timed: bool,
) -> Result<(), String> {
    let haystack = load(path)?;
    let needle = decode_hex(needle_hex)?;
    if needle.is_empty() || needle.contains(&b'\n') {
        return Err("runtime needle must be non-empty and contain no LF".to_owned());
    }
    let kernel = prepare(&needle);
    let records = oracle_records(&haystack, &needle);
    let expected = digest_records(&records, haystack.len(), needle.len());
    if haystack.len() == 67_108_864 && needle.len() == 23 {
        let lf_count = haystack.iter().filter(|byte| **byte == b'\n').count();
        let first_count = haystack.iter().filter(|byte| **byte == needle[0]).count();
        let last_start = records.last().map(|record| record.start);
        if records.len() != 74
            || lf_count != 1_324_231
            || first_count != 20_172_286
            || last_start != Some(63_538_171)
        {
            return Err(format!(
                "frozen real-input invariants failed: records={} lf={lf_count} first={first_count} last={last_start:?}",
                records.len()
            ));
        }
    }
    let observed = scan(&kernel, black_box(&haystack), black_box(&needle));
    if observed != expected {
        return Err(format!(
            "input correctness failure: expected={expected} observed={observed}"
        ));
    }

    if !timed {
        println!(
            "digest={observed} records={} input_hash={} needle_hash={} elapsed_ns=0 repetitions=0 length={}",
            records.len(),
            data_hash(&haystack),
            data_hash(&needle),
            haystack.len()
        );
        return Ok(());
    }

    for _ in 0..WARMUPS {
        let warmup = scan(&kernel, black_box(&haystack), black_box(&needle));
        if warmup != expected {
            return Err("warmup correctness failure".to_owned());
        }
    }
    let mut aggregate = 14_695_981_039_346_656_037_u64;
    let start = Instant::now();
    for repetition in 0..REPETITIONS {
        let invocation = scan(&kernel, black_box(&haystack), black_box(&needle));
        aggregate = digest_word(aggregate, repetition as u64);
        aggregate = digest_word(aggregate, invocation);
    }
    let elapsed = start.elapsed().as_nanos();
    let mut expected_aggregate = 14_695_981_039_346_656_037_u64;
    for repetition in 0..REPETITIONS {
        expected_aggregate = digest_word(expected_aggregate, repetition as u64);
        expected_aggregate = digest_word(expected_aggregate, expected);
    }
    if aggregate != expected_aggregate {
        return Err("timed correctness failure".to_owned());
    }
    println!(
        "digest={expected} records={} input_hash={} needle_hash={} elapsed_ns={elapsed} repetitions={REPETITIONS} length={}",
        records.len(),
        data_hash(&haystack),
        data_hash(&needle),
        haystack.len()
    );
    Ok(())
}

pub fn main_with<K>(prepare: impl Fn(&[u8]) -> K, scan: impl Fn(&K, &[u8], &[u8]) -> u64) {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments.as_slice() {
        [command] if command == "--check" => run_checks(&prepare, &scan),
        [command, path, needle] if command == "--verify-input" => {
            run_input(&prepare, &scan, Path::new(path), needle, false)
        }
        [command, path, needle] if command == "--bench-input" => {
            run_input(&prepare, &scan, Path::new(path), needle, true)
        }
        _ => Err("usage: binary --check | --verify-input PATH NEEDLE_HEX | --bench-input PATH NEEDLE_HEX".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
