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
 * power of two and never three); the runtime with this one. */
#if !defined(WF_SCHED_LANE_SLOTS)
#define WF_SCHED_LANE_SLOTS 64u
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
 * SUSPENDED and NOTIFIED are the park handshake's (§6); READY once its event
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
#if !defined(WF_SCHED_MAX_STACKS)
#define WF_SCHED_MAX_STACKS 1024u
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

/* The core's one instance. The runtime has exactly one; the enumerator makes
 * one per execution. Every word two threads touch is reached through a
 * primitive of `prim.h`; the lists are touched under the one mutex. */
typedef struct wf_sched_core {
    /* Under the one mutex: the ready list and the stack free list. */
    wf_sched_stack *ready_head;
    wf_sched_stack *ready_tail;
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
