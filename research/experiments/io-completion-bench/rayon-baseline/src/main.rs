//! Sequential Rust and Rayon controls for tests/programs/par_layout.wf.
//! The tree topology, strict floating-point expression order, table walk,
//! mutation and repetitions match that source. Only disjoint sibling calls
//! may run concurrently; neither the word sum nor the repeated layouts do.

use std::{env, process::ExitCode};

enum Node {
    Leaf {
        width: f64,
        height: f64,
        out: f64,
    },
    Branch {
        left: Box<Node>,
        right: Box<Node>,
        width: f64,
        height: f64,
        out: f64,
    },
}

fn build(depth: u32, width: f64) -> Box<Node> {
    Box::new(if depth == 0 {
        Node::Leaf {
            width,
            height: 0.5,
            out: 0.0,
        }
    } else {
        Node::Branch {
            left: build(depth - 1, width * 1.0009765625),
            right: build(depth - 1, width * 0.9990234375),
            width,
            height: 0.25,
            out: 0.0,
        }
    })
}

fn cascade(inherited: f64, width: f64, height: f64) -> f64 {
    let pad_left = width * 0.0625;
    let pad_right = width * 0.03125;
    let border = height * 0.125;
    let left_right = pad_left + pad_right;
    let padded = border.mul_add(2.0, left_right);
    let content = width - padded;
    let clamped = content.max(0.0);
    let font = inherited.mul_add(0.5, 8.0);
    let line = font * 1.25;
    let words = clamped / font;
    let lines_raw = words / 16.0;
    let lines = lines_raw.ceil();
    let resolved_height = lines * line;
    let margin = height * 0.5;
    let outer = resolved_height + margin;
    let scaled = outer.mul_add(1.0625, border);
    let root = scaled.abs().sqrt();
    let mixed = root.mul_add(3.25, clamped);
    let capped = mixed.min(4096.0);
    let floored = capped.max(0.0625);
    let rounded = floored.floor();
    let fraction = floored - rounded;
    let adjusted = fraction.mul_add(0.5, rounded);
    adjusted + padded
}

fn measure(words: &[f64], font: f64) -> f64 {
    let mut widest = 0.0;
    let mut total = 0.0;
    for &raw in words {
        let scaled = raw * font;
        if scaled > widest {
            widest = scaled;
        }
        total += scaled;
    }
    total + widest
}

fn layout<const PARALLEL: bool>(
    node: &mut Node,
    words: &[f64],
    inherited: f64,
    leaves: usize,
    grain: usize,
) -> f64 {
    match node {
        Node::Leaf { width, height, out } => {
            let own_height = cascade(inherited, *width, *height);
            let measured = measure(words, own_height);
            let total = own_height + measured;
            *out = total;
            total
        }
        Node::Branch {
            left,
            right,
            width,
            height,
            out,
        } => {
            let own_height = cascade(inherited, *width, *height);
            let measured = measure(words, own_height);
            let child_inherited = own_height * 0.5;
            let (a, b) = if PARALLEL && leaves > grain {
                rayon::join(
                    || layout::<true>(left, words, child_inherited, leaves / 2, grain),
                    || layout::<true>(right, words, child_inherited, leaves / 2, grain),
                )
            } else {
                (
                    layout::<false>(left, words, child_inherited, leaves / 2, grain),
                    layout::<false>(right, words, child_inherited, leaves / 2, grain),
                )
            };
            let children = a + b;
            let mine = own_height + measured;
            let total = children + mine;
            *out = total;
            total
        }
    }
}

fn word_table(length: usize) -> Vec<f64> {
    let mut width = 6.5;
    (0..length)
        .map(|_| {
            let result = width;
            width *= 1.0625;
            if width > 64.0 {
                width = 6.5;
            }
            result
        })
        .collect()
}

fn run<const PARALLEL: bool>(batches: usize, grain: usize) -> (u64, u64) {
    let mut tree = build(6, 512.0);
    let words = word_table(8192);
    let mut results = [0.0_f64; 2];
    for (result, count) in results.iter_mut().zip([8192, 4096]) {
        for _ in 0..batches {
            let mut seed = 16.0;
            for _ in 0..800 {
                *result = layout::<PARALLEL>(&mut tree, &words[..count], seed, 64, grain);
                seed += 0.0625;
            }
        }
    }
    (results[0].to_bits(), results[1].to_bits())
}

fn positive(value: Option<String>, name: &str, maximum: usize) -> Result<usize, String> {
    let parsed = value
        .ok_or_else(|| format!("missing {name}"))?
        .parse::<usize>()
        .map_err(|_| format!("invalid {name}"))?;
    if parsed == 0 || parsed > maximum {
        return Err(format!("{name} must be 1..{maximum}"));
    }
    Ok(parsed)
}

fn main_result() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mode = args.next().ok_or("missing mode")?;
    let threads = positive(args.next(), "threads", 256)?;
    let grain = positive(args.next(), "grain leaves", 64)?;
    let batches = positive(args.next(), "batches", 16)?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let result = match mode.as_str() {
        "seq" if threads == 1 && grain == 64 => run::<false>(batches, grain),
        "rayon" => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .map_err(|error| error.to_string())?;
            if pool.current_num_threads() != threads {
                return Err("Rayon pool width differs from request".into());
            }
            // Enter once for the entire batch. Per-node/per-layout install
            // calls would benchmark avoidable cross-thread submission costs.
            pool.install(|| run::<true>(batches, grain))
        }
        _ => return Err("mode must be seq (threads=1, grain=64) or rayon".into()),
    };
    println!("{:016x} {:016x}", result.0, result.1);
    Ok(())
}

fn main() -> ExitCode {
    match main_result() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!(
                "rayon-baseline: {error}\nusage: whitefoot-rayon-baseline seq|rayon THREADS GRAIN_LEAVES BATCHES"
            );
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_workload_matches_the_independent_wf_corpus_bytes() {
        assert_eq!(
            run::<false>(1, 64),
            (0x420a993efa7437a1, 0x41fa962893d45299)
        );
    }

    fn stored_bits(node: &Node, out: &mut Vec<u64>) {
        match node {
            Node::Leaf { out: value, .. } => out.push(value.to_bits()),
            Node::Branch {
                left,
                right,
                out: value,
                ..
            } => {
                out.push(value.to_bits());
                stored_bits(left, out);
                stored_bits(right, out);
            }
        }
    }

    #[test]
    fn sibling_parallelism_preserves_every_written_node_and_reuse() {
        for threads in [1, 2, 4] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            for grain in [1, 3, 4, 16, 64] {
                let mut sequential = build(6, 512.0);
                let mut parallel = build(6, 512.0);
                let words = word_table(257);
                for (count, inherited) in [(257, 16.0), (0, 8.0), (128, 65.9375)] {
                    let expected =
                        layout::<false>(&mut sequential, &words[..count], inherited, 64, grain);
                    let actual = pool.install(|| {
                        layout::<true>(&mut parallel, &words[..count], inherited, 64, grain)
                    });
                    assert_eq!(actual.to_bits(), expected.to_bits());
                    let (mut expected_slots, mut actual_slots) = (Vec::new(), Vec::new());
                    stored_bits(&sequential, &mut expected_slots);
                    stored_bits(&parallel, &mut actual_slots);
                    assert_eq!(actual_slots.len(), 127);
                    assert_eq!(actual_slots, expected_slots);
                }
            }
        }
    }
}
