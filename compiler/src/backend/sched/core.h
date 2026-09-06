/* The scheduler core: one scheduler for compute hand-outs and I/O completions.
 *
 * This is the unit `research/investigations/io-model/PARK-ON-MISS.md` §7
 * describes. It owns the pool of Whitefoot stacks and their states, the ready
 * list, the park handshake, the per-lane deque of compute hand-outs, and the
 * one rule a join runs (§2). It reaches shared state through the seven
 * primitives of `prim.h` and through nothing else, which is what lets the
 * same source compile against the enumerator's replacement primitives and be
 * driven through every interleaving of them (§11).
 *
 * Two storages, one protocol (§1). A compute hand-out's record is a slot of
 * the lane that offered it; an I/O operation's record is a block of the frame
 * that submitted it. Both begin with a `wf_sched_record`: a state that goes
 * PENDING to DONE exactly once, and the stack waiting on it, if any. Who
 * stores DONE differs — a thread that ran the hand-out, or the drain that
 * received the completion — and nothing else does.
 *
 * Nothing here allocates after start. The stacks come from one reservation
 * made at the core's entry and carved into equal slots; the ready list and
 * the free list are intrusive; a miss that finds no stack does not make one. */

#ifndef WHITEFOOT_SCHED_CORE_H
#define WHITEFOOT_SCHED_CORE_H

#include <stddef.h>
#include <stdint.h>

/* Untimed experiment diagnostics. Timed links contain no increments or
 * parking-worker writes from this observer. */
#ifndef WF_SCHED_OBSERVE
#define WF_SCHED_OBSERVE 0
#endif

/* 0: baseline FIFO; 1: worker queues sharing the free-list mutex;
 * 2: the same worker queues with independent mutexes. */
#ifndef WF_SCHED_READY_SHARDS
#define WF_SCHED_READY_SHARDS 0
#endif
#if WF_SCHED_READY_SHARDS < 0 || WF_SCHED_READY_SHARDS > 2
#error "WF_SCHED_READY_SHARDS must be zero, one or two"
#endif

#if defined(__cplusplus)
extern "C" {
#endif

/* ------------------------------------------------------------ the record */

/* A record's state: PENDING from publication; COMPLETING while its one
 * publisher claims the waiter; DONE after that, the publisher's last touch of
 * the record, which is what lets the joiner's frame die or the slot be
 * released the moment DONE is read. Never back. A compute slot returns to its
 * lane's free list and is re-initialised at its next acquisition. */
#define WF_SCHED_PENDING 1u
#define WF_SCHED_DONE 2u
#define WF_SCHED_COMPLETING 3u

struct wf_sched_stack;

/* The waiter a thread registers when it waits in place, inside the I/O arm
 * of the fourth line: no stack to resume, but a sleeper to wake (§2, §6). */
#define WF_SCHED_WAITER_IN_PLACE ((struct wf_sched_stack *)(uintptr_t)1)

/* The record every join runs the rule over. `state` and `waiter` are the two
 * words the park handshake reads and writes (§6, steps 2 and 3); what follows
 * them is the payload one of the two storages carries, and the core never
 * reads it. */
typedef struct wf_sched_record {
    unsigned state;
    struct wf_sched_stack *waiter;
} wf_sched_record;

/* A compute hand-out: the record, then the outlined call and its frame. The
 * emitted module names the frame; `wf_sched_slot_of` recovers the slot. */
#define WF_SCHED_FRAME_BYTES 256u
/* The enumerator builds with the smaller constant (design §11: two slots, a
 * power of two and never three); the runtime with this one.
 *
 * It is also the ceiling on a staged loop's window when the staged call is a
 * lane hand-out, because every iteration in flight holds one slot of the
 * offering thread's lane; the compiler restates it as `LANE_SLOTS` and a
 * test pins the two to each other. 1024 is the connection count the network
 * control test measures at (`research/investigations/io-model/NETWORK.md`
 * section 6), so a server whose fixed-trip accept loop names that many can
 * keep every one of them in flight. A lane is about 320 bytes per slot, so a
 * started thread's lane is about 320 KiB. Only configured lanes participate;
 * the baseline initializer still clears the entire reserved lane array. */
#if !defined(WF_SCHED_LANE_SLOTS)
#define WF_SCHED_LANE_SLOTS 1024u
#endif
#define WF_SCHED_NO_SLOT (~0u)

struct wf_sched_lane;

typedef struct wf_sched_slot {
    wf_sched_record record;
    void (*run)(void *frame);
    struct wf_sched_lane *home;
    /* Free-list link, valid while the slot is free; WF_SCHED_NO_SLOT ends it. */
    unsigned next_free;
    _Alignas(16) unsigned char frame[WF_SCHED_FRAME_BYTES];
} wf_sched_slot;

/* ------------------------------------------------------------- the stack */

/* A stack's phase. RUNNING while a thread executes on it; SUSPENDING,
 * SUSPENDED and NOTIFIED are the park handshake's (§6); a cooperative yield
 * also enters NOTIFIED, owing its enqueue to the far side of the switch.
 * READY once its event
 * has arrived and it waits on the ready list; EMPTY when its scheduler loop
 * found nothing and it is in the pool. */
#define WF_SCHED_STACK_RUNNING 1u
#define WF_SCHED_STACK_SUSPENDING 2u
#define WF_SCHED_STACK_SUSPENDED 3u
#define WF_SCHED_STACK_NOTIFIED 4u
#define WF_SCHED_STACK_READY 5u
#define WF_SCHED_STACK_EMPTY 6u

/* The state header at the top of every pool stack: a stack handle and a
 * header pointer are the same address, and the stack grows down from just
 * below it. */
typedef struct wf_sched_stack {
    unsigned phase;
    /* The stack pointer saved by the last switch away from this stack. */
    void *saved_sp;
    /* Ready-list and free-list link. A stack is on at most one list. */
    struct wf_sched_stack *next;
    /* The bounds the floor classifies a guard hit against, written by the
     * switch from the reservation record. */
    unsigned char *low;
    unsigned char *high;
    /* Its position in the reservation, so a test can name it. */
    unsigned index;
#if WF_SCHED_OBSERVE || WF_SCHED_READY_SHARDS
    /* Written by the running owner before publishing SUSPENDING; the park
     * handshake publishes it to the enqueue, then the queue mutex to pop. */
    unsigned park_thread;
#endif
} wf_sched_stack;

/* ------------------------------------------------------------- the lanes */

/* One lane: a Chase-Lev deque of slots and the slot storage. Pushed and
 * popped by its owning thread only, stolen from by any (I2). Its free list is
 * popped by the owning thread only and pushed by any thread, because a
 * resumed stack releases on whichever thread resumed it (I3), so both ends
 * of that list are atomic. */
typedef struct wf_sched_lane {
    unsigned long long top;
    unsigned long long bottom;
    wf_sched_slot *buffer[WF_SCHED_LANE_SLOTS];
    unsigned free_head;
    unsigned long long seed;
    wf_sched_slot slots[WF_SCHED_LANE_SLOTS];
} wf_sched_lane;

/* -------------------------------------------------------------- the core */

#if !defined(WF_SCHED_MAX_THREADS)
#define WF_SCHED_MAX_THREADS 64u
#endif
/* Startup footprint experiment: reserve the same maximum layout, but only
 * initialize lane storage belonging to configured workers. */
#if !defined(WF_SCHED_INIT_USED_LANES)
#define WF_SCHED_INIT_USED_LANES 0
#endif
/* The most stacks WF_STACKS may ask for. A parked callee holds a pool stack,
 * so a server keeping 1024 connections in flight needs 1024 of them beside
 * the entry thread's own and the workers', which is why the ceiling is above
 * the lane's 1024 slots. The default count stays the thread count plus the
 * spare in `entry.c`; this is only what a written setting may reach. */
#if !defined(WF_SCHED_MAX_STACKS)
#define WF_SCHED_MAX_STACKS 2048u
#endif

/* The bounded spin before the idle park. A turn of the idle window whose
 * drain and last look found nothing repeats those looks -- the ready list and
 * this thread's own deque and a steal, or the record a stack waits on -- for
 * SPIN rounds of `wf_prim_pause` and then YIELD rounds of `wf_prim_yield`,
 * and only then parks on the epoch it captured before all of them. These are
 * scheduling policy numbers of the same kind as `WF_PAR_SPLIT_OVERSUBSCRIBE`
 * in `sched/entry.c`: nothing about which programs are accepted, what they
 * compute, or what any of them observes depends on either one, and the park
 * they precede is unchanged.
 *
 * What they are for. A park is a kernel sleep and its wake is a kernel round
 * trip, which a virtual machine makes long. The Windows qualification bench
 * (`.github/workflows/io-bench.yml`) measures the unified `--par` build of
 * `research/experiments/io-completion-bench/programs/windows_runtime_mixed.wf`
 * at 1.06 times its completion-only build on a four-vCPU Windows VM, against a
 * bar of 0.95: per iteration of that program's loop two wakes are on the
 * critical path -- an idle worker's, to steal the hand-out just published, and
 * the publisher's own, when the stolen job makes its joiner's stack READY --
 * and on that host both are kernel round trips. A spinning thread finds either
 * one by looking, with no kernel in it.
 *
 * The retired runtime (`git show 92b19e1^:compiler/src/backend/par_runtime.c`,
 * its `WF_PAR_SPIN_ROUNDS` and `WF_PAR_YIELD_ROUNDS`) spun 4096 rounds and
 * then yielded 16 times before its condition variable, with the floor under
 * the spin argued from that machine's 2.2 microsecond park-and-wake: a thread
 * that looks for less time than a park costs sleeps to save less than the
 * sleep costs, and pays the wake anyway when work appears.
 *
 * What this measurement chose: 256 rounds and 16, from the sweep in the "§12
 * addendum: the idle spin" section of
 * `research/experiments/park-on-miss-measurements/README.md`. A look round
 * costs about 43 nanoseconds on that host, so 256 of them is a window of about
 * 11 microseconds against a park-and-wake of 16.2 -- the retired runtime's own
 * floor, which is that a thread should not sleep to save less than the sleep
 * costs, applied to the machine that was measured rather than to the one that
 * comment was written on. Longer windows were measured too: they buy a further
 * 4 percent on the mixed program and lose 15 percent on `par_layout` at eight
 * workers on four cores, and they burn more of a core to do it. The Windows
 * job may want another point of that sweep, and these two overrides are how it
 * asks for one.
 *
 * Under `WF_SCHED_ENUMERATE` a round is not a delay but a step -- the looks
 * are primitives and the enumerator walks every interleaving of them, and a
 * yield there blocks until another process writes -- so the enumerate build
 * overrides the spin to one round and the yield to none through these same
 * `#if !defined` guards. `compiler/Makefile`, target `sched-enumerate`, is
 * where both numbers and the reason for each of them are. */
#if !defined(WF_SCHED_IDLE_SPIN_ROUNDS)
#define WF_SCHED_IDLE_SPIN_ROUNDS 256u
#endif
#if !defined(WF_SCHED_IDLE_YIELD_ROUNDS)
#define WF_SCHED_IDLE_YIELD_ROUNDS 16u
#endif
/* Temporary idle-progress experiment: zero retains the original single
 * progress pass before spinning. A nonzero interval runs another progress
 * pass after that many idle looks. The existing capture-to-park window and
 * the last-look rule still cover every pass. This is runtime scheduling
 * policy, never compiler acceptance fuel. */
#if !defined(WF_SCHED_IDLE_PROGRESS_INTERVAL)
#define WF_SCHED_IDLE_PROGRESS_INTERVAL 0u
#endif

/* The counters one thread keeps. */
typedef struct wf_sched_statistics {
    unsigned long long parks;
    unsigned long long cancels;
    unsigned long long resumes;
    unsigned long long steals;
    unsigned long long inline_runs;
    unsigned long long exhausted_io_waits;
    unsigned long long exhausted_compute_waits;
    /* The I/O arm's late third line: a READY stack found inside the arm. */
    unsigned long long late_parks;
    /* Joins that found their record DONE at line one. */
    unsigned long long line_one;
    unsigned long long resume_migrations;
    unsigned long long idle_steps;
    unsigned long long idle_looks;
    unsigned long long idle_progress;
    unsigned long long idle_waits;
    unsigned long long checkpoints;
    unsigned long long checkpoint_switches;
} wf_sched_statistics;

/* What one thread knows: its lane, the stack it is on, the host stack it
 * started on, and the two things it owes the stack it just switched to: an
 * EMPTY stack to return to the pool, and a park to commit (§5, §6). Per
 * thread, never per stack: a stack migrates, and re-reads these after every
 * switch. */
typedef struct wf_sched_thread {
    struct wf_sched_lane *lane;
    wf_sched_stack *stack;
    void *host_sp;
    unsigned index;
    void (*entry)(void *argument);
    void *entry_argument;
    wf_sched_stack *pending_empty;
    wf_sched_stack *pending_commit;
    wf_sched_statistics counts;
} wf_sched_thread;

/* The baseline has one ready queue sharing the free-list mutex. The shard
 * candidate has one separately locked queue per configured worker. Padding
 * keeps different queues' head/tail writes on different cache lines. */
typedef struct wf_sched_ready_queue {
#if WF_SCHED_READY_SHARDS
    _Alignas(128)
#endif
    wf_sched_stack *head;
    wf_sched_stack *tail;
} wf_sched_ready_queue;

#if WF_SCHED_READY_SHARDS
#define WF_SCHED_READY_QUEUE_COUNT WF_SCHED_MAX_THREADS
#else
#define WF_SCHED_READY_QUEUE_COUNT 1u
#endif
#if WF_SCHED_READY_SHARDS == 2
#define WF_SCHED_LOCK_COUNT (WF_SCHED_MAX_THREADS + 1u)
#else
#define WF_SCHED_LOCK_COUNT 1u
#endif

/* The core's one instance. The runtime has exactly one; the enumerator makes
 * one per execution. Shared words are reached through prim.h; each list is
 * protected by its assigned mutex. */
typedef struct wf_sched_core {
    wf_sched_ready_queue ready[WF_SCHED_READY_QUEUE_COUNT];
    /* The stack free list always uses mutex zero. */
    wf_sched_stack *free_head;
    /* The reservation: `stack_count` stacks, headers at their tops. */
    unsigned char *reservation;
    size_t stack_bytes;
    size_t stack_stride;
    unsigned stack_count;
    wf_sched_stack *stacks[WF_SCHED_MAX_STACKS];
    /* Threads and their lanes; index 0 is the entry thread's. */
    unsigned thread_count;
    wf_sched_thread threads[WF_SCHED_MAX_THREADS];
    wf_sched_lane lanes[WF_SCHED_MAX_THREADS];
    /* The exit status once posted. */
    unsigned status_posted;
    int status;
    /* Which threads are inside their step-4 park, one bit each, so a publish
     * wakes nobody when nobody sleeps. */
    unsigned long long idle;
} wf_sched_core;

/* The counters a test reads back, kept per thread so that no two threads
 * ever write one word, and summed on request. None is on a hot path. */
void wf_sched_statistics_sum(const wf_sched_core *core, wf_sched_statistics *out);

/* --------------------------------------------------------------- the ABI */

/* Sizes the core over storage the caller supplies, reserves its stacks, and
 * prepares its lanes. `stack_count` is raised to `thread_count + 1` when it
 * is smaller (§5's floor). Returns 0 on success. */
int wf_sched_init(
    wf_sched_core *core,
    unsigned thread_count,
    unsigned stack_count,
    size_t stack_bytes
);

/* Takes a pool stack for worker `thread` on the thread that is about to
 * create it, so that the worker's own start, which may come arbitrarily late,
 * finds its stack reserved. Returns 0, or 1 when no stack is free, in which
 * case the creator does not create the thread. */
int wf_sched_start_thread(wf_sched_core *core, unsigned thread);

/* The first act of every thread: switch to its pool stack, the entry's taken
 * here and a worker's by `wf_sched_start_thread`, entering the scheduler loop
 * there. `entry`, if given, runs first on that stack; the entry thread's is
 * the program's main body, a worker's is nothing. Returns on the host stack
 * it was called on when the exit status is posted and this is thread 0; a
 * worker never returns. */
int wf_sched_run(wf_sched_core *core, unsigned thread, void (*entry)(void *), void *argument);

/* Posts the program's exit status: makes every sleeper re-check, and lets the
 * entry thread's loop leave (§5, §6). */
void wf_sched_post_status(wf_sched_core *core, int status);

/* The rule (§2) over one record: read it if DONE; run it here if it is the
 * newest entry of this thread's own deque; else park this stack; else the
 * exhausted arm the record's kind selects. `is_io` names that kind. */
void wf_sched_join(wf_sched_core *core, wf_sched_record *record, int is_io);

/* Progress I/O and give an already-ready stack a turn, if one exists. The
 * caller is on a running pool stack. This does not start an unstarted callee
 * or promise admission when the bounded stack/window capacity is exhausted. */
void wf_sched_checkpoint(wf_sched_core *core);

/* The one publisher call: store DONE, load the waiter, and mark that stack
 * READY (§6). The drain calls it for an I/O record; `wf_sched_execute` calls
 * it for a compute slot. */
void wf_sched_complete(wf_sched_core *core, wf_sched_record *record);

/* Compute hand-outs, the module's ABI. */
void *wf_sched_acquire(wf_sched_core *core, unsigned long bytes);
void wf_sched_publish(wf_sched_core *core, void *frame, void (*run)(void *));
void wf_sched_join_frame(wf_sched_core *core, void *frame);
void wf_sched_release(wf_sched_core *core, void *frame);

/* An I/O record before submission: PENDING, no waiter. */
void wf_sched_record_init(wf_sched_record *record);

/* The slot a frame pointer names. */
static inline wf_sched_slot *wf_sched_slot_of(void *frame) {
    return (wf_sched_slot *)((unsigned char *)frame - offsetof(wf_sched_slot, frame));
}

/* The calling thread and the stack it is on. */
wf_sched_thread *wf_sched_current_thread(wf_sched_core *core);
wf_sched_stack *wf_sched_current_stack(wf_sched_core *core);

#if defined(__cplusplus)
}
#endif

#endif
