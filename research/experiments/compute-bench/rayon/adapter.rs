//! Serves compute-bench: the Rayon reference's two entry points. `map` is the
//! flat dispatch used by the three independent-map kernels, in two shapes -- a
//! hand-written binary bisection through `rayon::join` down to one callback
//! per leaf, which is structurally the fork shape the compiler's splitter
//! emits, and Rayon's own adaptive parallel iterator. `fork2` is the
//! fork-join dispatch the recursive kernel uses, in both offer directions.
//!
//! Adapted from the research bundle's records-rayon/adapter.rs and the fork
//! direction of its quadrature.rs. The named cut: that file asserted
//! `matches!(width, 1 | 2 | 4)`; both entry points here assert
//! `matches!(width, 1..=32)`, which is the same ceiling as `WFB_MAX_WIDTH` in
//! backend.h. The two are changed together or not at all.
//!
//! No unsafe Rust: the binding to C is recovered from the emitted symbol by
//! deps.sh rather than declared with an export attribute.
#![forbid(unsafe_code)]

use rayon::prelude::*;
use std::ffi::c_void;
use std::sync::OnceLock;

type Chunk = extern "C" fn(*mut c_void, usize);
type Task = extern "C" fn(*mut c_void);

// use_current_thread registers the calling thread as worker zero. Its registry
// cannot currently be detached, so this reference deliberately has process
// lifetime, just like the WF and oneTBB rows. Initialization is charged to the
// first call, which is the warm-up call and enters no statistic.
static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();

fn pool(width: u32) -> &'static rayon::ThreadPool {
    assert!(matches!(width, 1..=32), "width must be 1 through 32");
    let pool = POOL.get_or_init(|| {
        assert!(
            rayon::current_thread_index().is_none(),
            "caller already registered"
        );
        rayon::ThreadPoolBuilder::new()
            .num_threads(width as usize)
            .use_current_thread()
            .build()
            .expect("Rayon pool initialization")
    });
    assert_eq!(pool.current_num_threads(), width as usize, "width changed");
    assert!(pool.current_thread_index().is_some(), "caller outside pool");
    pool
}

fn bisect(first: usize, end: usize, chunk: Chunk, context: usize) {
    if end - first <= 1 {
        if first != end {
            chunk(context as *mut c_void, first);
        }
    } else {
        let middle = first + (end - first) / 2;
        rayon::join(
            || bisect(first, middle, chunk, context),
            || bisect(middle, end, chunk, context),
        );
    }
}

/// Calls `chunk(context, i)` exactly once for every `i` in `0..chunks` and
/// joins every callback before returning. The C caller owns `context` and
/// guarantees finite independent callbacks; Rust never dereferences it. The
/// opaque address round-trips through an integer for Send/Sync, which the
/// callback's C-side disjoint-output contract authorizes.
pub extern "C" fn map(
    width: u32,
    chunks: usize,
    chunk: Option<Chunk>,
    context: *mut c_void,
    strategy: u32,
) {
    assert!(strategy <= 1, "unknown dispatch strategy");
    let _ = pool(width);
    if chunks == 0 {
        return;
    }
    let chunk = chunk.expect("missing chunk callback");
    let context = context as usize;
    if strategy == 0 {
        bisect(0, chunks, chunk, context);
    } else {
        (0..chunks)
            .into_par_iter()
            .for_each(|i| chunk(context as *mut c_void, i));
    }
}

/// Runs both tasks and joins both before returning. `rayon::join` runs its
/// FIRST closure locally and offers the SECOND, so a nonzero `left_offer`
/// passes `(right, left)` and offers the left subtree, which is the direction
/// generated Whitefoot publishes.
pub extern "C" fn fork2(
    width: u32,
    left: Option<Task>,
    left_context: *mut c_void,
    right: Option<Task>,
    right_context: *mut c_void,
    left_offer: i32,
) {
    let _ = pool(width);
    let left = left.expect("missing left task");
    let right = right.expect("missing right task");
    let left_context = left_context as usize;
    let right_context = right_context as usize;
    let run_left = move || left(left_context as *mut c_void);
    let run_right = move || right(right_context as *mut c_void);
    if left_offer != 0 {
        rayon::join(run_right, run_left);
    } else {
        rayon::join(run_left, run_right);
    }
}
