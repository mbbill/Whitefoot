use std::env;
use std::fs;
use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Summary {
    lines: u64,
    words: u64,
    bytes: u64,
    first_space: bool,
    last_space: bool,
}

fn is_space(byte: u8) -> bool {
    matches!(byte, 9..=13 | 32)
}

fn summarize(data: &[u8]) -> Summary {
    if data.is_empty() {
        return Summary { lines: 0, words: 0, bytes: 0, first_space: true, last_space: true };
    }
    let mut lines = 0;
    let mut words = 0;
    let mut previous_space = true;
    for &byte in data {
        if byte == b'\n' { lines += 1; }
        let space = is_space(byte);
        if !space && previous_space { words += 1; }
        previous_space = space;
    }
    Summary {
        lines,
        words,
        bytes: data.len() as u64,
        first_space: is_space(data[0]),
        last_space: previous_space,
    }
}

fn combine(a: Summary, b: Summary) -> Summary {
    if a.bytes == 0 { return b; }
    if b.bytes == 0 { return a; }
    Summary {
        lines: a.lines + b.lines,
        words: a.words + b.words - u64::from(!a.last_space && !b.first_space),
        bytes: a.bytes + b.bytes,
        first_space: a.first_space,
        last_space: b.last_space,
    }
}

fn run_once(data: &[u8], threads: usize) -> Summary {
    if threads == 1 { return summarize(data); }
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(threads);
        for i in 0..threads {
            let begin = data.len() * i / threads;
            let end = data.len() * (i + 1) / threads;
            handles.push(scope.spawn(move || summarize(&data[begin..end])));
        }
        handles.into_iter().map(|h| h.join().unwrap()).fold(
            Summary { lines: 0, words: 0, bytes: 0, first_space: true, last_space: true },
            combine,
        )
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        eprintln!("usage: {} FILE [THREADS] [REPEATS]", args[0]);
        std::process::exit(2);
    }
    let threads: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(1);
    let repeats: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(1);
    let data = fs::read(&args[1]).unwrap();
    let mut out = summarize(&[]);
    let mut best = u128::MAX;
    for _ in 0..repeats {
        let start = Instant::now();
        out = black_box(run_once(black_box(&data), threads));
        best = best.min(start.elapsed().as_nanos());
    }
    println!("{} {} {} threads={} best_ns={}", out.lines, out.words, out.bytes, threads, best);
}
