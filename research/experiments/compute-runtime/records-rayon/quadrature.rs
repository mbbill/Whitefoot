//! Recursive scalar reference, linked by check-quadrature through the existing
//! pinned Rayon crate. Retire with that experiment. No per-node FFI callbacks.

use std::sync::OnceLock;

#[cfg(feature = "quadrature-stats")]
mod work {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    // Each worker writes only its own cache-separated counter. The single
    // benchmark caller resets/reads them between fully joined root calls.
    #[repr(align(128))]
    struct Counter(AtomicU64);
    static NODES: [Counter; 4] = [const { Counter(AtomicU64::new(0)) }; 4];

    pub fn reset() {
        for counter in &NODES {
            counter.0.store(0, Relaxed);
        }
    }

    pub fn credit(nodes: u64) {
        let counter = &NODES[rayon::current_thread_index().unwrap_or(0)].0;
        counter.store(counter.load(Relaxed) + nodes, Relaxed);
    }

    pub fn snapshot() -> [u64; 4] {
        std::array::from_fn(|i| NODES[i].0.load(Relaxed))
    }

    #[test]
    fn worker_counter_identity_and_reset() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        reset();
        // Broadcast visits each worker once; distinct credits catch swapped
        // labels and lost additions independently of recursive tree totals.
        pool.broadcast(|context| {
            credit(1 << context.index());
            credit(3 << context.index());
        });
        assert_eq!(snapshot(), [4, 8, 16, 32]);
        reset();
        assert_eq!(snapshot(), [0; 4]);
    }
}

struct Result {
    value: f64,
    #[cfg(feature = "quadrature-stats")]
    nodes: u64,
    #[cfg(feature = "quadrature-stats")]
    forks: u64,
    #[cfg(feature = "quadrature-stats")]
    migrated: u64,
}

impl Result {
    fn leaf(value: f64) -> Self {
        Self {
            value,
            #[cfg(feature = "quadrature-stats")]
            nodes: 1,
            #[cfg(feature = "quadrature-stats")]
            forks: 0,
            #[cfg(feature = "quadrature-stats")]
            migrated: 0,
        }
    }

    fn merge(left: Self, right: Self, parallel: bool) -> Self {
        #[cfg(not(feature = "quadrature-stats"))]
        let _ = parallel;
        Self {
            value: left.value + right.value,
            #[cfg(feature = "quadrature-stats")]
            nodes: 1 + left.nodes + right.nodes,
            #[cfg(feature = "quadrature-stats")]
            forks: u64::from(parallel) + left.forks + right.forks,
            #[cfg(feature = "quadrature-stats")]
            migrated: left.migrated + right.migrated,
        }
    }
}

fn density(x: f64, center: f64, width: f64) -> f64 {
    let z = (x - center) / width;
    1.0 / (1.0 + z * z)
}

fn simpson(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    ((b - a) / 6.0) * ((fa + 4.0 * fm) + fb)
}

// Compile the sequential subtree separately, just as the C++ controls do.
// The same scalar operations and left-plus-right rounding order serve both.
#[allow(clippy::too_many_arguments)]
fn adaptive<const PARALLEL: bool, const LEFT_OFFER: bool>(
    a: f64,
    b: f64,
    center: f64,
    width: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    whole: f64,
    tolerance: f64,
    depth: u32,
    budget: u32,
) -> Result {
    if PARALLEL && budget == 0 {
        let result =
            adaptive::<false, false>(a, b, center, width, fa, fm, fb, whole, tolerance, depth, 0);
        // A serial subtree never joins or changes executing worker. Attribute
        // its entire node count once, without a counter access at every leaf.
        #[cfg(feature = "quadrature-stats")]
        work::credit(result.nodes);
        return result;
    }
    #[cfg(feature = "quadrature-stats")]
    if PARALLEL {
        work::credit(1);
    }
    let middle = (a + b) * 0.5;
    let fl = density((a + middle) * 0.5, center, width);
    let fr = density((middle + b) * 0.5, center, width);
    let left = simpson(a, middle, fa, fl, fm);
    let right = simpson(middle, b, fm, fr, fb);
    let combined = left + right;
    let delta = combined - whole;
    if depth == 0 || delta.abs() <= 15.0 * tolerance {
        return Result::leaf(combined + delta / 15.0);
    }
    let run_left = || {
        adaptive::<PARALLEL, LEFT_OFFER>(
            a,
            middle,
            center,
            width,
            fa,
            fl,
            fm,
            left,
            tolerance * 0.5,
            depth - 1,
            if PARALLEL { budget - 1 } else { 0 },
        )
    };
    let run_right = || {
        adaptive::<PARALLEL, LEFT_OFFER>(
            middle,
            b,
            center,
            width,
            fm,
            fr,
            fb,
            right,
            tolerance * 0.5,
            depth - 1,
            if PARALLEL { budget - 1 } else { 0 },
        )
    };
    if !PARALLEL {
        return Result::merge(run_left(), run_right(), false);
    }
    #[cfg(feature = "quadrature-stats")]
    let owner = std::thread::current().id();
    let observe = |result: Result| {
        #[cfg(feature = "quadrature-stats")]
        let result = Result {
            migrated: result.migrated + u64::from(std::thread::current().id() != owner),
            ..result
        };
        result
    };
    let (left, right) = if LEFT_OFFER {
        let (right, left) = rayon::join(|| observe(run_right()), || observe(run_left()));
        (left, right)
    } else {
        rayon::join(|| observe(run_left()), || observe(run_right()))
    };
    Result::merge(left, right, true)
}

// The caller participates as worker zero: width4 means it plus three helpers.
// Rayon cannot unregister use_current_thread today; this private pool lives
// for the benchmark process, like the existing records Rayon control.
static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();

// A single root ABI call, without raw-pointer dereferencing or unsafe exports.
// The build resolves this ordinary extern-C symbol from this crate's LLVM IR.
// The C observer receives counters only in the separate diagnostic build.
#[allow(clippy::too_many_arguments)]
pub extern "C" fn quadrature(
    mode: u32,
    workers: u32,
    budget: u32,
    a: f64,
    b: f64,
    center: f64,
    width: f64,
    tolerance: f64,
    depth: u32,
    observe: Option<extern "C" fn(u64, u64, u64, u64, u64, u64, u64)>,
) -> f64 {
    assert!(mode <= 2 && matches!(workers, 1 | 4) && budget <= 24 && depth <= 24);
    if mode != 0 {
        let pool = POOL.get_or_init(|| {
            assert!(rayon::current_thread_index().is_none());
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers as usize)
                .use_current_thread()
                .build()
                .expect("quadrature Rayon pool")
        });
        assert_eq!(pool.current_num_threads(), workers as usize);
        assert_eq!(pool.current_thread_index(), Some(0));
    }
    #[cfg(feature = "quadrature-stats")]
    work::reset();
    let fa = density(a, center, width);
    let fm = density((a + b) * 0.5, center, width);
    let fb = density(b, center, width);
    let whole = simpson(a, b, fa, fm, fb);
    let result = match mode {
        0 => adaptive::<false, false>(a, b, center, width, fa, fm, fb, whole, tolerance, depth, 0),
        1 => adaptive::<true, false>(
            a, b, center, width, fa, fm, fb, whole, tolerance, depth, budget,
        ),
        _ => adaptive::<true, true>(
            a, b, center, width, fa, fm, fb, whole, tolerance, depth, budget,
        ),
    };
    #[cfg(feature = "quadrature-stats")]
    {
        if mode == 0 {
            work::credit(result.nodes);
        }
        let nodes = work::snapshot();
        assert_eq!(nodes.iter().sum::<u64>(), result.nodes);
        observe.expect("quadrature diagnostic observer")(
            result.nodes,
            result.forks,
            result.migrated,
            nodes[0],
            nodes[1],
            nodes[2],
            nodes[3],
        );
    }
    #[cfg(not(feature = "quadrature-stats"))]
    let _ = observe;
    result.value
}
