#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

extern crate wf_scan_control;

use std::time::Instant;

const BUFFER_LENGTH: usize = 64 * 1024 * 1024;
const WARMUPS: usize = 3;

#[cfg(full_scan)]
const REPETITIONS: usize = 128;
#[cfg(early_scan)]
const REPETITIONS: usize = 24;

#[cfg(full_scan)]
const SHAPE: &str = "full";
#[cfg(early_scan)]
const SHAPE: &str = "early";

fn next_state(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

fn fill_data(data: &mut [u8]) {
    let mut state = 0x8f3f_73b5_cf1c_9ade_u64;
    for byte in data.iter_mut() {
        state = next_state(state);
        *byte = (state % 250) as u8;
    }
    #[cfg(early_scan)]
    if data.len() >= 4 {
        let length = data.len();
        data[length / 4] = 251;
        data[length / 2] = 252;
        data[length - 1] = 253;
    }
}

fn data_hash(data: &[u8]) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

#[cfg(full_scan)]
fn oracle_scan(data: &[u8]) -> u64 {
    let lines = data.iter().filter(|byte| **byte == 10).count() as u64;
    let candidates = data.iter().filter(|byte| **byte == 80).count() as u64;
    lines.wrapping_add(candidates.wrapping_mul(1_000_000_007))
}

#[cfg(early_scan)]
fn oracle_find(data: &[u8], needle: u8) -> u64 {
    data.iter()
        .position(|byte| *byte == needle)
        .map_or(data.len() as u64, |offset| offset as u64)
}

#[cfg(early_scan)]
fn oracle_scan(data: &[u8]) -> u64 {
    oracle_find(data, 251)
        .wrapping_add(oracle_find(data, 252))
        .wrapping_add(oracle_find(data, 253))
        .wrapping_add(oracle_find(data, 254))
}

#[cfg(full_scan)]
fn scan(data: &[u8]) -> u64 {
    wf_scan_control::full_scan(data)
}

#[cfg(early_scan)]
fn scan(data: &[u8]) -> u64 {
    wf_scan_control::early_scan(data)
}

fn verify_case(data: &[u8]) -> Result<(), String> {
    let expected = oracle_scan(data);
    let observed = scan(data);
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "{SHAPE} correctness failure: length={} expected={expected} observed={observed}",
            data.len()
        ))
    }
}

fn run_checks() -> Result<(), String> {
    verify_case(&[])?;
    verify_case(&[10])?;
    verify_case(&[80])?;
    verify_case(&[251, 10, 252, 80, 253, 10, 0])?;
    let all_bytes = (0_u16..=255).map(|value| value as u8).collect::<Vec<_>>();
    verify_case(&all_bytes)?;
    let mut generated = vec![0_u8; 4096];
    fill_data(&mut generated);
    verify_case(&generated)
}

fn run_benchmark() -> Result<(), String> {
    let mut data = vec![0_u8; BUFFER_LENGTH];
    fill_data(&mut data);
    let expected = oracle_scan(&data);
    let hash = data_hash(&data);
    for _ in 0..WARMUPS {
        verify_case(&data)?;
    }

    let mut aggregate = 0_u64;
    let start = Instant::now();
    for _ in 0..REPETITIONS {
        aggregate = aggregate.wrapping_add(scan(&data));
    }
    let elapsed = start.elapsed().as_nanos();
    let expected_aggregate = expected.wrapping_mul(REPETITIONS as u64);
    if aggregate != expected_aggregate {
        return Err("timed correctness failure".to_owned());
    }
    println!(
        "shape={SHAPE} checksum={aggregate} data_hash={hash} elapsed_ns={elapsed} repetitions={REPETITIONS} length={BUFFER_LENGTH}"
    );
    Ok(())
}

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments.as_slice() {
        [] => run_benchmark(),
        [argument] if argument == "--check" => run_checks(),
        _ => Err("usage: scan-floor-rust [--check]".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
