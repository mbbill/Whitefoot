/* The resource-exhaustion floor, linked into every Whitefoot program.
 *
 * Two jobs, both about the one abnormal end a *correct* program can reach.
 *
 * The first is that the ceiling should be the compiler's number rather than
 * the environment's. A program's depth limit was whatever `ulimit -s` happened
 * to leave it, so the same binary on the same input succeeded on one shell and
 * died on another. `wf__floor_run` runs the entry on a thread this file sizes,
 * so the limit travels with the program.
 *
 * The second is that running out should be a defined, reported abort instead
 * of a bare host signal. A guard-page hit is converted into one fixed record
 * on standard error followed by `abort`; every other fault is put back exactly
 * as it was, so a genuine memory defect keeps its own signal, its own exit
 * status, and its core dump. A diagnostic that made a wild pointer and an
 * exhausted stack look alike would be worse than none, because it would
 * misdirect the reader.
 *
 * Everything below the handler boundary is async-signal-safe: no allocation,
 * no stdio, no locks, and no pthread queries. The per-thread stack bounds the
 * handler reads are captured at attach time, on the thread itself, where the
 * ordinary rules still apply.
 */

#if !defined(__APPLE__)
#define _GNU_SOURCE
#endif

#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <unistd.h>

/* The entry body the emitted module defines. `wf__floor_run` is the only
 * caller, and the module's own weak fallback calls it directly when this
 * translation unit is not linked. */
extern int wf__main_body(int argc, char **argv);

/* ------------------------------------------------------------ the ceiling */

/* The stack the entry runs on.
 *
 * One deliberate number, chosen by the compiler rather than inherited from the
 * environment. It is a reservation, not a commitment: the pages are populated
 * as the program actually descends, so a program that never recurses deeply
 * pays no resident memory for the headroom and no time to reserve it.
 *
 * Raising it costs address space and nothing else; the reason not to raise it
 * further is that a runaway recursion should still end in a bounded time. */
#define WF_FLOOR_STACK_BYTES ((size_t)1024u * 1024u * 1024u)

/* The same number, for the parallel runtime's lanes.
 *
 * A lane runs ordinary Whitefoot calls and a stolen call starts at the bottom
 * of the lane's own stack, so a lane sized like the entry makes stealing
 * strictly headroom-positive: no schedule reaches a depth the no-steal
 * schedule could not. A lane sized any other way puts the program's liveness
 * in the hands of a steal race. Exporting the constant rather than repeating
 * it is what makes "the same number" a fact about the program instead of a
 * comment in two files. */
size_t wf__floor_stack_bytes(void) { return WF_FLOOR_STACK_BYTES; }

/* ------------------------------------------------- the descriptor factory */

/* The `FileFactory`'s one native fact: how many descriptors this program may
 * still open [SYS-10].
 *
 * The capacity is fixed the first time it is asked for: the process's soft
 * descriptor limit, less a reserve for the descriptors the runtime and the
 * entry hold themselves (the standard streams, the working directory, a
 * completion ring and its event descriptor, helper pipes). The reserve is a
 * constant rather than a scan of the descriptor table, because a scan costs
 * one system call per possible descriptor and the limit may be a million; the
 * constant only has to be at least what the runtime actually holds, which is
 * a handful. Being below the true limit is the whole promise: an open that
 * holds a permit is never refused a descriptor by this process's own
 * consumption. `reserve_file` spends one credit without a host call, and
 * nothing raises the count again: an explicit close hands the credit back as
 * the permit it returns, and a permit or an open resource that derived
 * release consumes spends its credit for the rest of the program [SYS-10].
 * The count is atomic because a permit may be reserved on whichever thread
 * resumed the reserving frame. Nothing here allocates, blocks, or waits. */
#define WF_FILE_RUNTIME_RESERVE 64L
#define WF_FILE_CAPACITY_CEILING (1L << 20)

static _Atomic long wf__file_credits;
static pthread_once_t wf__file_capacity_once = PTHREAD_ONCE_INIT;

static void wf__file_capacity_init(void) {
    struct rlimit limit;
    long capacity = 0;
    if (getrlimit(RLIMIT_NOFILE, &limit) == 0) {
        if (limit.rlim_cur == RLIM_INFINITY
            || limit.rlim_cur > (rlim_t)WF_FILE_CAPACITY_CEILING) {
            capacity = WF_FILE_CAPACITY_CEILING;
        } else {
            capacity = (long)limit.rlim_cur;
        }
    }
    capacity -= WF_FILE_RUNTIME_RESERVE;
    if (capacity < 0) {
        capacity = 0;
    }
    atomic_store_explicit(&wf__file_credits, capacity, memory_order_release);
}

/* Takes one credit. Returns 1 when the factory had one, 0 when it is spent. */
int wf__file_reserve(void) {
    long credits;
    (void)pthread_once(&wf__file_capacity_once, wf__file_capacity_init);
    credits = atomic_load_explicit(&wf__file_credits, memory_order_acquire);
    while (credits > 0) {
        if (atomic_compare_exchange_weak_explicit(
                &wf__file_credits,
                &credits,
                credits - 1,
                memory_order_acq_rel,
                memory_order_acquire
            )) {
            return 1;
        }
    }
    return 0;
}

/* ------------------------------------------------------- per-thread state */

/* [low, high) of this thread's usable stack. Written once at attach, on the
 * thread itself; the handler only reads it. */
static _Thread_local unsigned long wf__floor_stack_low;
static _Thread_local unsigned long wf__floor_stack_high;
static _Thread_local int wf__floor_stack_known;

/* The alternate stack the handler runs on, because the ordinary one is exactly
 * what has just run out. One mapping per thread, populated on first use. */
#define WF_FLOOR_ALTSTACK_BYTES ((size_t)64u * 1024u)

/* How far below the usable stack a fault still counts as this thread running
 * out rather than as a wild pointer.
 *
 * This is the probe's geometry, not a margin of comfort. Every generated
 * definition carries the target's `probe-stack` attribute, so a frame walks
 * its pages on the way down in strides of one page: the first touch below the
 * stack is at most one stride under it, and no descent can step past the guard
 * page into whatever lies beyond. Below that a leaf may touch its ABI red zone
 * without moving the stack pointer at all, which is 128 bytes on x86-64 and
 * none on AArch64. One stride plus the red zone is therefore the whole set of
 * addresses a thread running out can fault at.
 *
 * Sizing it any wider is not free slack: every extra byte is a range of wild
 * faults reported as exhaustion, which is exactly the misdirection this file
 * exists to avoid. The band was 1 MiB and ate real corruption faults up to
 * roughly 128,000 times further from the stack than a legitimate overflow
 * lands.
 *
 * The stride is the host's page size, read once outside signal context because
 * `sysconf` is not async-signal-safe. Zero means it was never read, which
 * leaves the discrimination exact rather than absent: the fault must then be
 * inside the stack itself. */
#define WF_FLOOR_RED_ZONE_BYTES ((unsigned long)128u)
static volatile unsigned long wf__floor_guard_band;

/* ------------------------------------------------------------- the record */

/* The bytes an exhausted execution writes before aborting.
 *
 * Fixed by the resource class alone. Two independent constraints force that
 * and agree on it: a signal handler may reach only async-signal-safe
 * facilities, which admits a constant and a raw `write` and essentially
 * nothing more; and [PAR-1] requires a program's observables to be identical
 * under every permitted schedule, so the record may not name a worker, a
 * thread, a depth, or an address.
 *
 * The absent fields are the point. It carries no `rule_id`, no function, and
 * no node path: exhaustion violates no source obligation, so the record names
 * only the unavailable external resource. */
static const char WF_FLOOR_STACK_RECORD[] = "{\"resource\":\"stack\"}\n";

/* One latch for every resource record any thread of this process can write.
 *
 * It has to be one rather than one per writer. The signal handler below writes
 * the stack record; an emitted module that allocates writes the heap record.
 * Those are different threads reaching different resource limits, so a latch
 * per writer leaves them unserialized against each other and two records can
 * interleave on the same channel — which is exactly what "exactly one record"
 * is supposed to rule out. The module asks for the address rather than keeping
 * its own, and a module linked without this unit falls back to one of its own. */
static volatile int wf__floor_latch;

volatile int *wf__floor_record_latch(void) { return &wf__floor_latch; }

static void wf__floor_emit(const char *bytes, size_t length) {
    while (length > 0) {
        ssize_t written = write(2, bytes, length);
        if (written <= 0) {
            if (written < 0 && errno == EINTR) {
                continue;
            }
            return;
        }
        bytes += (size_t)written;
        length -= (size_t)written;
    }
}

/* ------------------------------------------------------------ the handler */

static void wf__floor_handler(int signo, siginfo_t *info, void *context) {
    unsigned long fault = (unsigned long)(uintptr_t)(info ? info->si_addr : 0);
    int guard_hit = 0;
    (void)context;

    if (wf__floor_stack_known) {
        guard_hit = (fault < wf__floor_stack_high)
                    && (fault + wf__floor_guard_band >= wf__floor_stack_low);
    }

    if (!guard_hit) {
        /* Not this mechanism's fault class. Put the default disposition back
         * and deliver the signal under it, so the process dies exactly as it
         * would have without this file: same signal, same status, same core
         * dump. The floor adds nothing here and hides nothing.
         *
         * The re-raise is what makes that true for a signal that has no
         * faulting instruction to re-execute. An externally delivered SIGBUS
         * or SIGSEGV reaches this path too — on this host it arrives with a
         * null `si_addr`, indistinguishable from a null dereference — and
         * simply returning would swallow it *and* leave the disposition at
         * SIG_DFL process-wide, so every later thread's overflow would arrive
         * as a bare host signal with zero bytes. The restore is per-signal and
         * process-wide while the classification above is per-thread, so the
         * only safe thing to do after restoring is to make sure this process
         * does not outlive it. Raising while the signal is still blocked
         * queues it for delivery on return, with this thread's faulting
         * context intact. */
        struct sigaction restore;
        memset(&restore, 0, sizeof(restore));
        restore.sa_handler = SIG_DFL;
        sigemptyset(&restore.sa_mask);
        sigaction(signo, &restore, NULL);
        pthread_kill(pthread_self(), signo);
        return;
    }

    if (__sync_bool_compare_and_swap(&wf__floor_latch, 0, 1)) {
        wf__floor_emit(WF_FLOOR_STACK_RECORD, sizeof(WF_FLOOR_STACK_RECORD) - 1);
        abort();
    }
    /* A second faulting thread parks rather than racing the writer; the
     * winner's abort takes the process down underneath it. */
    for (;;) {
        pause();
    }
}

/* -------------------------------------------------------------- installing */

/* Captures the calling thread's stack bounds. Called on the thread itself and
 * outside any signal context, so the ordinary pthread queries are available. */
static void wf__floor_capture_bounds(void) {
#if defined(__APPLE__)
    /* Darwin reports the high address — one past the top — and the size. */
    void *top = pthread_get_stackaddr_np(pthread_self());
    size_t size = pthread_get_stacksize_np(pthread_self());
    wf__floor_stack_high = (unsigned long)(uintptr_t)top;
    wf__floor_stack_low = wf__floor_stack_high - (unsigned long)size;
    wf__floor_stack_known = 1;
#else
    pthread_attr_t attributes;
    void *base = NULL;
    size_t size = 0;
    if (pthread_getattr_np(pthread_self(), &attributes) != 0) {
        return;
    }
    if (pthread_attr_getstack(&attributes, &base, &size) == 0) {
        wf__floor_stack_low = (unsigned long)(uintptr_t)base;
        wf__floor_stack_high = wf__floor_stack_low + (unsigned long)size;
        wf__floor_stack_known = 1;
    }
    /* Unconditional: the query can succeed and the read still fail, and the
     * attribute object is owned either way. */
    pthread_attr_destroy(&attributes);
#endif
}

/* The per-thread half: somewhere for the handler to run, and the bounds that
 * tell it whether a fault was this thread running out.
 *
 * Every thread that runs Whitefoot code calls this — the entry thread here,
 * and each pool lane as it attaches. A thread without it is not unsafe; its
 * overflow simply falls back to the bare host signal. */
void wf__floor_attach_thread(void) {
    stack_t alternate;
    void *memory;
    wf__floor_capture_bounds();
    memory = mmap(NULL, WF_FLOOR_ALTSTACK_BYTES, PROT_READ | PROT_WRITE,
                  MAP_PRIVATE | MAP_ANON, -1, 0);
    if (memory == MAP_FAILED) {
        return;
    }
    alternate.ss_sp = memory;
    alternate.ss_size = WF_FLOOR_ALTSTACK_BYTES;
    alternate.ss_flags = 0;
    sigaltstack(&alternate, NULL);
}

/* The process-wide half: one disposition for the two fault signals.
 *
 * Both are required. A stack that runs out on the entry thread arrives as
 * SIGSEGV and one that runs out on a pool lane arrives as SIGBUS, so a
 * SIGSEGV-only disposition would miss every worker overflow — which is
 * precisely the case the parallel default introduced.
 *
 * SA_NODEFER is deliberately absent: with the signal blocked for the duration
 * of the handler, the restore-and-return path above is what re-raises it, and
 * a fault inside the handler cannot re-enter the handler.
 *
 * A failure here is not a reason to refuse to start. Failing to install
 * changes only the quality of the diagnosis, not the meaning of the program,
 * so the program runs with the behaviour it had before this file existed. */
static void wf__floor_install(void) {
    struct sigaction action;
    long page = sysconf(_SC_PAGESIZE);
    if (page > 0) {
        wf__floor_guard_band = (unsigned long)page + WF_FLOOR_RED_ZONE_BYTES;
    }
    memset(&action, 0, sizeof(action));
    action.sa_sigaction = wf__floor_handler;
    action.sa_flags = SA_SIGINFO | SA_ONSTACK;
    sigemptyset(&action.sa_mask);
    if (sigaction(SIGSEGV, &action, NULL) != 0) {
        return;
    }
    if (sigaction(SIGBUS, &action, NULL) != 0) {
        return;
    }
    wf__floor_attach_thread();
}

/* ------------------------------------------------------------- the entry */

struct wf__floor_call {
    int argc;
    char **argv;
    int status;
};

static void *wf__floor_entry(void *opaque) {
    struct wf__floor_call *call = (struct wf__floor_call *)opaque;
    wf__floor_attach_thread();
    call->status = wf__main_body(call->argc, call->argv);
    return NULL;
}

/* Runs the program's entry on a stack of this file's choosing.
 *
 * Every failure on the way falls back to running the entry on the thread the
 * host started us with — the program the caller asked for still runs, with the
 * ceiling it had before. Losing the headroom is a worse outcome than the one
 * this file promises, but refusing to run at all would be worse still. */
int wf__floor_run(int argc, char **argv) {
    pthread_attr_t attributes;
    pthread_t thread;
    struct wf__floor_call call;

    call.argc = argc;
    call.argv = argv;
    call.status = 0;

    wf__floor_install();

    if (pthread_attr_init(&attributes) != 0) {
        return wf__main_body(argc, argv);
    }
    if (pthread_attr_setstacksize(&attributes, WF_FLOOR_STACK_BYTES) != 0
        || pthread_create(&thread, &attributes, wf__floor_entry, &call) != 0) {
        pthread_attr_destroy(&attributes);
        return wf__main_body(argc, argv);
    }
    pthread_attr_destroy(&attributes);
    /* The thread was created joinable by this thread, so the two documented
     * failures — joining a non-joinable thread and joining oneself — are both
     * unreachable here. Re-running the entry on a failure would run the whole
     * program twice, which is why nothing here retries. */
    pthread_join(thread, NULL);
    return call.status;
}
