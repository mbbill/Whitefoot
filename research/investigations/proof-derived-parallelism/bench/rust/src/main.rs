//! Rust half of the paired browser-layout benchmark.
//!
//! Deliberately the same algorithm, the same data structure, and the same
//! operation sequence as the generated Whitefoot program: a heap-boxed binary
//! box tree (`Box<LNode>`, no arena), one combined layout pass per node made of
//! a float cascade and a measure loop over one shared metric table, and a
//! bottom-up fold that writes every node's resolved height into its own slot.
//!
//! Operation mapping (Whitefoot -> Rust), all strict IEEE-754 binary64, no
//! fast-math anywhere:
//!   fadd.strict / fsub.strict / fmul.strict / fdiv.strict -> + - * /
//!   ffma.strict(a, b, c)                                  -> a.mul_add(b, c)
//!   fsqrt.strict                                          -> .sqrt()
//!   fceil / ffloor / fabs                                 -> .ceil() / .floor() / .abs()
//!   fmin / fmax                                           -> .min() / .max()
//!   fgt                                                   -> >
//!   reinterpret<f64, u64>                                 -> f64::to_bits
//!   +wrap / -wrap on u64                                  -> wrapping_add / wrapping_sub
//!
//! usage: paired-layout MODE SHAPE DEPTH WORDS REPS [THREADS]
//!   MODE   seq | rayon | rayoncut | nodes
//!   SHAPE  bal | skew

use std::env;

enum LNode {
    Leaf {
        w: f64,
        h: f64,
        out: f64,
    },
    Branch {
        left: Box<LNode>,
        right: Box<LNode>,
        w: f64,
        h: f64,
        out: f64,
    },
}

fn boxed_leaf(w: f64) -> Box<LNode> {
    Box::new(LNode::Leaf {
        w,
        h: 0.5,
        out: 0.0,
    })
}

/// Balanced builder: mirrors the generated `build(depth, w)`.
fn build(depth: u64, w: f64) -> Box<LNode> {
    if depth == 0 {
        return boxed_leaf(w);
    }
    let next = depth.wrapping_sub(1);
    let wl = w * 1.0009765625;
    let wr = w * 0.9990234375;
    let l = build(next, wl);
    let r = build(next, wr);
    Box::new(LNode::Branch {
        left: l,
        right: r,
        w,
        h: 0.25,
        out: 0.0,
    })
}

/// Skewed builder: mirrors the generated `build(depth, w, phase)`. At every
/// other level the right subtree is three levels shallower than the left,
/// saturating at a leaf.
fn build_skew(depth: u64, w: f64, phase: u64) -> Box<LNode> {
    if depth == 0 {
        return boxed_leaf(w);
    }
    let next = depth.wrapping_sub(1);
    let mut right_depth = next;
    if phase == 0 {
        if next >= 3 {
            right_depth = next.wrapping_sub(3);
        } else {
            right_depth = 0;
        }
    }
    let next_phase = 1u64.wrapping_sub(phase);
    let wl = w * 1.0009765625;
    let wr = w * 0.9990234375;
    let l = build_skew(next, wl, next_phase);
    let r = build_skew(right_depth, wr, next_phase);
    Box::new(LNode::Branch {
        left: l,
        right: r,
        w,
        h: 0.25,
        out: 0.0,
    })
}

/// One node's float cascade, statement for statement as in the Whitefoot source.
fn cascade(inh: f64, w: f64, h: f64) -> f64 {
    let pad_l = w * 0.0625;
    let pad_r = w * 0.03125;
    let border = h * 0.125;
    let lr = pad_l + pad_r;
    let lrb = border.mul_add(2.0, lr);
    let content = w - lrb;
    let clamped_lo = content.max(0.0);
    let font = inh.mul_add(0.5, 8.0);
    let line = font * 1.25;
    let words = clamped_lo / font;
    let lines_raw = words / 16.0;
    let lines = lines_raw.ceil();
    let height = lines * line;
    let margin = h * 0.5;
    let outer = height + margin;
    let scaled = outer.mul_add(1.0625, border);
    let abs_scaled = scaled.abs();
    let root = abs_scaled.sqrt();
    let mixed = root.mul_add(3.25, clamped_lo);
    let capped = mixed.min(4096.0);
    let floored = capped.max(0.0625);
    let rounded = floored.floor();
    let frac = floored - rounded;
    let adj = frac.mul_add(0.5, rounded);
    adj + lrb
}

/// The words-measure loop, bounded by the table's own length exactly as the
/// Whitefoot version is (which is what discharges its index obligation there
/// without a claim).
fn measure_words(words: &[f64], font: f64) -> f64 {
    let mut widest = 0.0f64;
    let mut total = 0.0f64;
    let room = words.len();
    let mut k = 0usize;
    while k < room {
        let raw = words[k];
        let scaled = raw * font;
        if scaled > widest {
            widest = scaled;
        }
        total = total + scaled;
        k += 1;
    }
    total + widest
}

fn layout(node: &mut LNode, words: &[f64], inh: f64) -> f64 {
    match node {
        LNode::Leaf { w, h, out } => {
            let v = cascade(inh, *w, *h);
            let m = measure_words(words, v);
            let total = v + m;
            *out = total;
            total
        }
        LNode::Branch {
            left,
            right,
            w,
            h,
            out,
        } => {
            let own_h = cascade(inh, *w, *h);
            let m = measure_words(words, own_h);
            let child_inh = own_h * 0.5;
            let a = layout(left, words, child_inh);
            let b = layout(right, words, child_inh);
            let kids = a + b;
            let mine = own_h + m;
            let total = kids + mine;
            *out = total;
            total
        }
    }
}

/// Idiomatic rayon: fork at every branch and let work stealing handle grain.
fn layout_rayon(node: &mut LNode, words: &[f64], inh: f64) -> f64 {
    match node {
        LNode::Leaf { w, h, out } => {
            let v = cascade(inh, *w, *h);
            let m = measure_words(words, v);
            let total = v + m;
            *out = total;
            total
        }
        LNode::Branch {
            left,
            right,
            w,
            h,
            out,
        } => {
            let own_h = cascade(inh, *w, *h);
            let m = measure_words(words, own_h);
            let child_inh = own_h * 0.5;
            let (a, b) = rayon::join(
                || layout_rayon(left, words, child_inh),
                || layout_rayon(right, words, child_inh),
            );
            let kids = a + b;
            let mine = own_h + m;
            let total = kids + mine;
            *out = total;
            total
        }
    }
}

const CUTOFF: u32 = 5;

/// Depth-cutoff rayon: fork only above depth `CUTOFF`, run the plain
/// sequential fold below it.
fn layout_rayon_cutoff(node: &mut LNode, words: &[f64], inh: f64, depth: u32) -> f64 {
    if depth >= CUTOFF {
        return layout(node, words, inh);
    }
    match node {
        LNode::Leaf { w, h, out } => {
            let v = cascade(inh, *w, *h);
            let m = measure_words(words, v);
            let total = v + m;
            *out = total;
            total
        }
        LNode::Branch {
            left,
            right,
            w,
            h,
            out,
        } => {
            let own_h = cascade(inh, *w, *h);
            let m = measure_words(words, own_h);
            let child_inh = own_h * 0.5;
            let (a, b) = rayon::join(
                || layout_rayon_cutoff(left, words, child_inh, depth + 1),
                || layout_rayon_cutoff(right, words, child_inh, depth + 1),
            );
            let kids = a + b;
            let mine = own_h + m;
            let total = kids + mine;
            *out = total;
            total
        }
    }
}

fn count(node: &LNode) -> (u64, u64) {
    match node {
        LNode::Leaf { .. } => (1, 1),
        LNode::Branch { left, right, .. } => {
            let (ln, ll) = count(left);
            let (rn, rl) = count(right);
            (1 + ln + rn, ll + rl)
        }
    }
}

fn make_words(n: u64) -> Vec<f64> {
    let mut words = vec![0.25f64; n as usize];
    let mut width = 6.5f64;
    let mut w = 0usize;
    while w < words.len() {
        words[w] = width;
        width = width * 1.0625;
        if width > 64.0 {
            width = 6.5;
        }
        w += 1;
    }
    words
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() < 6 {
        eprintln!("usage: paired-layout MODE SHAPE DEPTH WORDS REPS [THREADS]");
        std::process::exit(2);
    }
    let mode = argv[1].clone();
    let shape = argv[2].clone();
    let depth: u64 = argv[3].parse().expect("DEPTH");
    let words_n: u64 = argv[4].parse().expect("WORDS");
    let reps: u64 = argv[5].parse().expect("REPS");
    let threads: usize = if argv.len() > 6 {
        argv[6].parse().expect("THREADS")
    } else {
        1
    };

    let mut tree = if shape == "bal" {
        build(depth, 512.0)
    } else {
        build_skew(depth, 512.0, 0)
    };

    if mode == "nodes" {
        let (n, l) = count(&tree);
        println!("nodes={} leaves={} branches={}", n, l, n - l);
        return;
    }

    let words = make_words(words_n);

    let run = |tree: &mut Box<LNode>| -> f64 {
        let mut i = 0u64;
        let mut last = 0.0f64;
        let mut seed = 16.0f64;
        while i != reps {
            let total = match mode.as_str() {
                "seq" => layout(tree, &words, seed),
                "rayon" => layout_rayon(tree, &words, seed),
                "rayoncut" => layout_rayon_cutoff(tree, &words, seed, 0),
                other => {
                    eprintln!("unknown mode {other}");
                    std::process::exit(2);
                }
            };
            last = total;
            seed = seed + 0.0625;
            i = i.wrapping_add(1);
        }
        last
    };

    let last = if mode == "seq" {
        run(&mut tree)
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("rayon pool");
        pool.install(|| run(&mut tree))
    };

    println!("{:016x}", last.to_bits());
}
