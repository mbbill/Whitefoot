#![forbid(unsafe_code)]

#[cfg(feature = "quadrature")]
pub mod quadrature;

use rayon::prelude::*;
use std::ffi::c_void;
use std::sync::OnceLock;

type Chunk = extern "C" fn(*mut c_void, usize);
// use_current_thread registers the calling thread as worker zero. Its registry
// cannot currently be detached, so this control deliberately has process
// lifetime, just like the WF and oneTBB controls. Initialization is timed.
static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();

fn fork(first: usize, end: usize, chunk: Chunk, context: usize) {
    if end - first <= 1 {
        if first != end {
            chunk(context as *mut c_void, first);
        }
    } else {
        let middle = first + (end - first) / 2;
        rayon::join(
            || fork(first, middle, chunk, context),
            || fork(middle, end, chunk, context),
        );
    }
}

// Research C caller owns context and guarantees finite independent callbacks;
// all callbacks are joined before returning. Rust never dereferences context.
// Keep the exact C callback prototype, including the opaque pointer argument.
// The build binds this ordinary public symbol from emitted LLVM IR; no unsafe
// export attribute or source-level Rust ABI is used across the C boundary.
pub extern "C" fn run(
    width: u32,
    chunks: usize,
    chunk: Option<Chunk>,
    context: *mut c_void,
    strategy: u32,
) {
    assert!(matches!(width, 1 | 2 | 4), "width must be 1, 2 or 4");
    assert!(strategy <= 1, "unknown dispatch strategy");
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
    if chunks == 0 {
        return;
    }
    let chunk = chunk.expect("missing chunk callback");
    // Round-trip the opaque C address through an integer for Send/Sync. The
    // callback's C-side disjoint-output contract authorizes concurrent calls.
    let context = context as usize;
    if strategy == 0 {
        fork(0, chunks, chunk, context);
    } else {
        (0..chunks)
            .into_par_iter()
            .for_each(|i| chunk(context as *mut c_void, i));
    }
}
