//! Process-local benchmark driver for the frozen utf8parse study.
//!
//! The scoring executable is linked to the two xlang objects by
//! `benchmark.py`. One invocation loads and verifies the immutable corpus,
//! allocates and touches three equally sized event outputs, measures each variant
//! exactly once in the requested order, then verifies and digests all output
//! outside the timed intervals.

use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use xlang_utf8parse_rust_baseline::parse_into;

const SCORE_BYTES: usize = 134_217_728;
const SMOKE_MAX_BYTES: usize = 16_777_216;
const BLOCK_BYTES: usize = 4_096;
const CLASS_CYCLE_BYTES: usize = BLOCK_BYTES * 4;
const CORPUS_SEED: u64 = 0x5554_4638_4245_4e32;
const OUTPUT_SENTINEL: u32 = 0xDEAD_BEEF;

#[repr(C)]
#[derive(Clone, Copy)]
struct Buf {
    p: *mut u8,
    n: i64,
}

#[cfg(not(any(feature = "smoke-shim", test)))]
unsafe extern "C" {
    fn xlang_parse_facts(out: Buf, src: Buf) -> u64;
    fn xlang_parse_nofacts(out: Buf, src: Buf) -> u64;
}

#[cfg(any(feature = "smoke-shim", test))]
unsafe extern "C" fn xlang_parse_facts(out: Buf, src: Buf) -> u64 {
    // Harness wiring validation only. Scoring builds cannot compile this shim.
    unsafe { smoke_parse(out, src) }
}

#[cfg(any(feature = "smoke-shim", test))]
unsafe extern "C" fn xlang_parse_nofacts(out: Buf, src: Buf) -> u64 {
    // Harness wiring validation only. Scoring builds cannot compile this shim.
    unsafe { smoke_parse(out, src) }
}

#[cfg(any(feature = "smoke-shim", test))]
unsafe fn smoke_parse(out: Buf, src: Buf) -> u64 {
    let out_len = usize::try_from(out.n).expect("smoke output length");
    let src_len = usize::try_from(src.n).expect("smoke source length");
    assert!(out_len >= src_len);
    // SAFETY: the smoke caller supplies live, disjoint vectors of these sizes.
    let out_slice = unsafe { std::slice::from_raw_parts_mut(out.p.cast::<u32>(), out_len) };
    let src_slice = unsafe { std::slice::from_raw_parts(src.p.cast_const(), src_len) };
    parse_into(out_slice, src_slice).expect("smoke output capacity") as u64
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Score,
    Smoke,
}

impl Mode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "score" => Ok(Self::Score),
            "smoke" => Ok(Self::Smoke),
            _ => Err(format!("invalid mode {value:?}; expected score or smoke")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Score => "score",
            Self::Smoke => "smoke",
        }
    }

    fn validate_bytes(self, bytes: usize) -> Result<(), String> {
        match self {
            Self::Score if bytes != SCORE_BYTES => Err(format!(
                "score mode requires exactly {SCORE_BYTES} corpus bytes; got {bytes}"
            )),
            Self::Smoke
                if bytes == 0
                    || bytes > SMOKE_MAX_BYTES
                    || !bytes.is_multiple_of(CLASS_CYCLE_BYTES) =>
            {
                Err(format!(
                    "smoke corpus bytes must be a positive multiple of {CLASS_CYCLE_BYTES} and at most {SMOKE_MAX_BYTES}; got {bytes}"
                ))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Variant {
    FactsOn,
    FactsOff,
    Rust,
}

impl Variant {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "facts-on" => Ok(Self::FactsOn),
            "facts-off" => Ok(Self::FactsOff),
            "rust" => Ok(Self::Rust),
            _ => Err(format!("invalid variant {value:?}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::FactsOn => "facts-on",
            Self::FactsOff => "facts-off",
            Self::Rust => "rust",
        }
    }

    fn identity_index(self) -> usize {
        match self {
            Self::FactsOn => 0,
            Self::FactsOff => 1,
            Self::Rust => 2,
        }
    }
}

struct XorShift64Star {
    state: u64,
}

impl XorShift64Star {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(2_685_821_657_736_338_717)
    }
}

fn fill_ascii(data: &mut Vec<u8>, end: usize, rng: &mut XorShift64Star) {
    while data.len() < end {
        data.push(((rng.next() >> 8) & 0x7f) as u8);
    }
}

fn scalar_for_width(random: u64, width: u8) -> char {
    let value = match width {
        2 => 0x80 + (random as u32 % (0x800 - 0x80)),
        3 => {
            let mut value = 0x800 + (random as u32 % (0x1_0000 - 0x800));
            if (0xd800..=0xdfff).contains(&value) {
                value += 0x800;
            }
            value
        }
        4 => 0x1_0000 + (random as u32 % (0x11_0000 - 0x1_0000)),
        _ => unreachable!("UTF-8 width is 2, 3, or 4"),
    };
    char::from_u32(value).expect("generated Unicode scalar")
}

fn push_scalar_or_fill(
    data: &mut Vec<u8>,
    end: usize,
    random: u64,
    width: u8,
    rng: &mut XorShift64Star,
) -> bool {
    let scalar = scalar_for_width(random, width);
    let mut encoded = [0u8; 4];
    let token = scalar.encode_utf8(&mut encoded).as_bytes();
    if end - data.len() < token.len() {
        fill_ascii(data, end, rng);
        false
    } else {
        data.extend_from_slice(token);
        true
    }
}

fn push_token_or_fill(
    data: &mut Vec<u8>,
    end: usize,
    token: &[u8],
    rng: &mut XorShift64Star,
) -> bool {
    if end - data.len() < token.len() {
        fill_ascii(data, end, rng);
        false
    } else {
        data.extend_from_slice(token);
        true
    }
}

fn generate_corpus(bytes: usize) -> Vec<u8> {
    assert_eq!(bytes % CLASS_CYCLE_BYTES, 0);
    let mut rng = XorShift64Star::new(CORPUS_SEED);
    let mut data = Vec::with_capacity(bytes);

    for block in 0..(bytes / BLOCK_BYTES) {
        let end = data.len() + BLOCK_BYTES;
        match block % 4 {
            // A: arbitrary ASCII, maximizing the one-event-per-byte path.
            0 => fill_ascii(&mut data, end, &mut rng),
            // B: valid, ASCII-heavy text with all multibyte widths.
            1 => {
                while data.len() < end {
                    let random = rng.next();
                    if random & 3 == 0 {
                        let width = 2 + ((random >> 8) % 3) as u8;
                        if !push_scalar_or_fill(&mut data, end, random >> 16, width, &mut rng) {
                            break;
                        }
                    } else {
                        data.push(((random >> 8) & 0x7f) as u8);
                    }
                }
            }
            // C: valid multibyte-heavy text, still ending in ground state.
            2 => {
                while data.len() < end {
                    let random = rng.next();
                    let width = 2 + ((random >> 8) % 3) as u8;
                    if !push_scalar_or_fill(&mut data, end, random >> 16, width, &mut rng) {
                        break;
                    }
                }
            }
            // D: malformed and boundary-invalid sequences. Every token either
            // completes a scalar or returns the upstream parser to ground.
            3 => {
                const MALFORMED: [&[u8]; 12] = [
                    &[0x80],
                    &[0xbf],
                    &[0xc0],
                    &[0xc1],
                    &[0xf5],
                    &[0xff],
                    &[0xc2, b'A'],
                    &[0xe0, 0x9f],
                    &[0xed, 0xa0],
                    &[0xf0, 0x8f],
                    &[0xf4, 0x90],
                    &[0xe2, 0x82, b'A'],
                ];
                while data.len() < end {
                    let random = rng.next();
                    if random & 3 == 0 {
                        let width = 2 + ((random >> 8) % 3) as u8;
                        if !push_scalar_or_fill(&mut data, end, random >> 16, width, &mut rng) {
                            break;
                        }
                    } else {
                        let token = MALFORMED[((random >> 8) as usize) % MALFORMED.len()];
                        if !push_token_or_fill(&mut data, end, token, &mut rng) {
                            break;
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(data.len(), end);
    }

    assert_eq!(data.len(), bytes);
    data
}

#[derive(Clone)]
struct Sha256 {
    state: [u32; 8],
    length_bytes: u64,
    buffer: [u8; 64],
    buffered: usize,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            length_bytes: 0,
            buffer: [0; 64],
            buffered: 0,
        }
    }

    fn update(&mut self, mut input: &[u8]) {
        self.length_bytes = self
            .length_bytes
            .checked_add(input.len() as u64)
            .expect("SHA-256 input length");

        if self.buffered != 0 {
            let take = (64 - self.buffered).min(input.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&input[..take]);
            self.buffered += take;
            input = &input[take..];
            if self.buffered == 64 {
                let block = self.buffer;
                self.compress(&block);
                self.buffered = 0;
            }
        }

        while input.len() >= 64 {
            let block: &[u8; 64] = input[..64].try_into().expect("64-byte SHA block");
            self.compress(block);
            input = &input[64..];
        }
        if !input.is_empty() {
            self.buffer[..input.len()].copy_from_slice(input);
            self.buffered = input.len();
        }
    }

    fn finalize(mut self) -> [u8; 32] {
        let bit_length = self
            .length_bytes
            .checked_mul(8)
            .expect("SHA-256 bit length");
        self.buffer[self.buffered] = 0x80;
        self.buffered += 1;
        if self.buffered > 56 {
            self.buffer[self.buffered..].fill(0);
            let block = self.buffer;
            self.compress(&block);
            self.buffer = [0; 64];
        } else {
            self.buffer[self.buffered..56].fill(0);
        }
        self.buffer[56..64].copy_from_slice(&bit_length.to_be_bytes());
        let block = self.buffer;
        self.compress(&block);

        let mut digest = [0; 32];
        for (chunk, word) in digest.chunks_exact_mut(4).zip(self.state) {
            chunk.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    fn compress(&mut self, block: &[u8; 64]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut words = [0u32; 64];
        for (index, chunk) in block.chunks_exact(4).enumerate() {
            words[index] = u32::from_be_bytes(chunk.try_into().expect("four-byte SHA word"));
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        for (slot, value) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

fn sha256_events(events: &[u32]) -> [u8; 32] {
    const CHUNK_EVENTS: usize = 4_096;
    let mut hasher = Sha256::new();
    let mut encoded = vec![0u8; CHUNK_EVENTS * size_of::<u32>()];
    for chunk in events.chunks(CHUNK_EVENTS) {
        for (event, bytes) in chunk.iter().zip(encoded.chunks_exact_mut(4)) {
            bytes.copy_from_slice(&event.to_le_bytes());
        }
        hasher.update(&encoded[..size_of_val(chunk)]);
    }
    hasher.finalize()
}

fn hex_digest(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(64);
    for &byte in digest {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

fn option_value(args: &[String], flag: &str) -> Result<String, String> {
    let position = args
        .iter()
        .position(|arg| arg == flag)
        .ok_or_else(|| format!("missing required {flag}"))?;
    args.get(position + 1)
        .cloned()
        .ok_or_else(|| format!("missing value after {flag}"))
}

fn parse_order(value: &str) -> Result<[Variant; 3], String> {
    let values: Vec<_> = value
        .split(',')
        .map(Variant::parse)
        .collect::<Result<_, _>>()?;
    let order: [Variant; 3] = values
        .try_into()
        .map_err(|_| "order must contain exactly three comma-separated variants".to_string())?;
    let mut seen = [false; 3];
    for variant in order {
        if seen[variant.identity_index()] {
            return Err("order repeats a variant".to_string());
        }
        seen[variant.identity_index()] = true;
    }
    Ok(order)
}

fn prepare_corpus(args: &[String]) -> Result<(), String> {
    let mode = Mode::parse(&option_value(args, "--mode")?)?;
    let bytes: usize = option_value(args, "--bytes")?
        .parse()
        .map_err(|error| format!("invalid --bytes: {error}"))?;
    mode.validate_bytes(bytes)?;
    let output = PathBuf::from(option_value(args, "--output")?);
    if output.exists() {
        return Err(format!("refusing to overwrite corpus {}", output.display()));
    }
    let corpus = generate_corpus(bytes);
    let digest = hex_digest(&sha256(&corpus));
    fs::write(&output, &corpus)
        .map_err(|error| format!("write corpus {}: {error}", output.display()))?;
    println!(
        "{{\"schema_version\":1,\"kind\":\"corpus\",\"mode\":\"{}\",\"not_a_score\":{},\"bytes\":{},\"block_bytes\":{},\"sha256\":\"{}\"}}",
        mode.as_str(),
        mode == Mode::Smoke,
        bytes,
        BLOCK_BYTES,
        digest
    );
    Ok(())
}

#[derive(Clone)]
struct Sample {
    variant: Variant,
    ordinal: usize,
    elapsed_ns: u128,
    output_events: usize,
    output_sha256: String,
}

fn run_variant(variant: Variant, out: &mut [u32], src: &[u8]) -> Result<usize, String> {
    match variant {
        Variant::FactsOn | Variant::FactsOff => {
            let function = match variant {
                Variant::FactsOn => xlang_parse_facts,
                Variant::FactsOff => xlang_parse_nofacts,
                Variant::Rust => unreachable!(),
            };
            let out_n = i64::try_from(out.len()).map_err(|_| "output too large for ABI")?;
            let src_n = i64::try_from(src.len()).map_err(|_| "source too large for ABI")?;
            // SAFETY: both slices are live and disjoint; the correctness gate
            // has already established the frozen candidate's ABI behavior.
            let produced = unsafe {
                function(
                    Buf {
                        p: out.as_mut_ptr().cast::<u8>(),
                        n: out_n,
                    },
                    Buf {
                        p: src.as_ptr().cast_mut(),
                        n: src_n,
                    },
                )
            };
            usize::try_from(produced).map_err(|_| "xlang returned an oversized length".to_string())
        }
        Variant::Rust => parse_into(out, src)
            .map_err(|_| "Rust adapter rejected an input-sized output".to_string()),
    }
}

fn run_block(args: &[String]) -> Result<(), String> {
    let mode = Mode::parse(&option_value(args, "--mode")?)?;
    #[cfg(feature = "smoke-shim")]
    if mode != Mode::Smoke {
        return Err("a smoke-shim executable refuses score mode".to_string());
    }
    #[cfg(not(feature = "smoke-shim"))]
    if mode != Mode::Score {
        return Err("a scoring executable refuses smoke mode".to_string());
    }

    let corpus_path = PathBuf::from(option_value(args, "--corpus")?);
    let expected_sha = option_value(args, "--expected-sha256")?;
    if expected_sha.len() != 64 || !expected_sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--expected-sha256 must be 64 hexadecimal characters".to_string());
    }
    let block_index: usize = option_value(args, "--block-index")?
        .parse()
        .map_err(|error| format!("invalid --block-index: {error}"))?;
    let order = parse_order(&option_value(args, "--order")?)?;

    let corpus = fs::read(&corpus_path)
        .map_err(|error| format!("read corpus {}: {error}", corpus_path.display()))?;
    mode.validate_bytes(corpus.len())?;
    let corpus_sha = hex_digest(&sha256(&corpus));
    if corpus_sha != expected_sha.to_ascii_lowercase() {
        return Err(format!(
            "corpus SHA-256 mismatch: expected {expected_sha}, got {corpus_sha}"
        ));
    }

    // Allocation, initialization, and page touching all happen before timing.
    let mut outputs = [
        vec![OUTPUT_SENTINEL; corpus.len()],
        vec![OUTPUT_SENTINEL; corpus.len()],
        vec![OUTPUT_SENTINEL; corpus.len()],
    ];
    let mut samples = Vec::with_capacity(3);
    let mut produced_by_variant = [0usize; 3];
    for (ordinal, variant) in order.into_iter().enumerate() {
        let output = &mut outputs[variant.identity_index()];
        let started = Instant::now();
        let produced = run_variant(variant, output, &corpus)?;
        let elapsed_ns = started.elapsed().as_nanos();
        black_box(produced);
        if elapsed_ns == 0 {
            return Err("monotonic clock returned a zero-length interval".to_string());
        }
        if produced > output.len() {
            return Err(format!(
                "{} returned {} events for an output of {} elements",
                variant.as_str(),
                produced,
                output.len()
            ));
        }
        produced_by_variant[variant.identity_index()] = produced;
        samples.push(Sample {
            variant,
            ordinal,
            elapsed_ns,
            output_events: produced,
            output_sha256: String::new(),
        });
    }

    // Correctness checks and digesting are intentionally outside timing.
    for sample in &mut samples {
        let output = &outputs[sample.variant.identity_index()];
        if output[sample.output_events..]
            .iter()
            .any(|&byte| byte != OUTPUT_SENTINEL)
        {
            return Err(format!(
                "{} changed the output-event suffix",
                sample.variant.as_str()
            ));
        }
        sample.output_sha256 = hex_digest(&sha256_events(&output[..sample.output_events]));
        black_box(&sample.output_sha256);
    }
    let reference_length = produced_by_variant[0];
    let reference_digest = samples
        .iter()
        .find(|sample| sample.variant == Variant::FactsOn)
        .expect("facts-on sample")
        .output_sha256
        .clone();
    for sample in &samples {
        if sample.output_events != reference_length || sample.output_sha256 != reference_digest {
            return Err(format!(
                "output mismatch: {} produced {} events with digest {}, expected {} events with digest {}",
                sample.variant.as_str(),
                sample.output_events,
                sample.output_sha256,
                reference_length,
                reference_digest
            ));
        }
    }
    let corpus_sha_after = hex_digest(&sha256(&corpus));
    if corpus_sha_after != corpus_sha {
        return Err("source corpus changed during parse".to_string());
    }

    let samples_json = samples
        .iter()
        .map(|sample| {
            format!(
                "{{\"variant\":\"{}\",\"ordinal\":{},\"elapsed_ns\":{},\"input_bytes\":{},\"output_events\":{},\"output_sha256\":\"{}\"}}",
                sample.variant.as_str(),
                sample.ordinal,
                sample.elapsed_ns,
                corpus.len(),
                sample.output_events,
                sample.output_sha256
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"schema_version\":1,\"kind\":\"benchmark-block\",\"mode\":\"{}\",\"not_a_score\":{},\"block_index\":{},\"pid\":{},\"clock\":\"std::time::Instant\",\"corpus_bytes\":{},\"corpus_sha256\":\"{}\",\"order\":\"{}\",\"samples\":[{}]}}",
        mode.as_str(),
        mode == Mode::Smoke,
        block_index,
        process::id(),
        corpus.len(),
        corpus_sha,
        order.map(Variant::as_str).join(","),
        samples_json
    );
    Ok(())
}

fn usage(program: &Path) -> String {
    format!(
        "usage:\n  {} prepare-corpus --mode score|smoke --bytes N --output PATH\n  {} run-block --mode score|smoke --corpus PATH --expected-sha256 HEX --block-index N --order A,B,C",
        program.display(),
        program.display()
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let outcome = match args.get(1).map(String::as_str) {
        Some("prepare-corpus") => prepare_corpus(&args[2..]),
        Some("run-block") => run_block(&args[2..]),
        _ => Err(usage(Path::new(
            args.first().map(String::as_str).unwrap_or("bench"),
        ))),
    };
    if let Err(error) = outcome {
        eprintln!("benchmark harness error: {error}");
        process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xlang_utf8parse_rust_baseline::INVALID_EVENT;

    fn parse_events(source: &[u8]) -> Vec<u32> {
        let mut out = vec![OUTPUT_SENTINEL; source.len()];
        let produced = parse_into(&mut out, source).unwrap();
        out.truncate(produced);
        out
    }

    #[test]
    fn sha256_matches_standard_vectors() {
        assert_eq!(
            hex_digest(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex_digest(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_events(&[0x0403_0201, 0x0807_0605]),
            sha256(&[1, 2, 3, 4, 5, 6, 7, 8])
        );
    }

    #[test]
    fn corpus_is_block_exact_deterministic_and_ground_aligned() {
        let first = generate_corpus(CLASS_CYCLE_BYTES);
        let second = generate_corpus(CLASS_CYCLE_BYTES);
        assert_eq!(first.len(), CLASS_CYCLE_BYTES);
        assert_eq!(first, second);
        assert!(first[..BLOCK_BYTES].iter().all(u8::is_ascii));
        assert!(std::str::from_utf8(&first[BLOCK_BYTES..BLOCK_BYTES * 3]).is_ok());
        assert!(parse_events(&first[BLOCK_BYTES * 3..]).contains(&INVALID_EVENT));

        for block in first.chunks_exact(BLOCK_BYTES) {
            let expected = parse_events(block);
            let mut extended = block.to_vec();
            extended.push(b'Z');
            let actual = parse_events(&extended);
            assert_eq!(&actual[..expected.len()], expected);
            assert_eq!(actual.get(expected.len()), Some(&(b'Z' as u32)));
            assert_eq!(actual.len(), expected.len() + 1);
        }
    }

    #[test]
    fn smoke_variants_match() {
        let source = [b'A', 0xc2, 0xa2, 0xe2, 0x82, b'A', 0xff, b'Z'];
        let expected = [
            b'A' as u32,
            0x00a2,
            INVALID_EVENT,
            INVALID_EVENT,
            b'Z' as u32,
        ];
        let mut outputs: [Vec<u32>; 3] =
            std::array::from_fn(|_| vec![OUTPUT_SENTINEL; source.len()]);
        for variant in [Variant::FactsOn, Variant::FactsOff, Variant::Rust] {
            let produced =
                run_variant(variant, &mut outputs[variant.identity_index()], &source).unwrap();
            assert_eq!(&outputs[variant.identity_index()][..produced], expected);
        }
    }
}
