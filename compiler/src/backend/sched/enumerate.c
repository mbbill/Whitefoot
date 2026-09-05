/* The enumerator: the merge gate of PARK-ON-MISS.md §11.
 *
 * `core.c` is compiled against this unit's replacement of `prim.h`
 * (`-DWF_SCHED_ENUMERATE`), so every one of the seven primitives it reaches
 * shared state through is a call into the controlled scheduler below. One OS
 * thread runs everything: the controller on the process stack, T simulated
 * threads as coroutines on host stacks, each calling `wf_sched_run` exactly as
 * a real thread would, and a device whose steps complete submitted records.
 * A primitive announces the operation it is about to perform and switches to
 * the controller; the controller chooses the next enabled process, and the
 * chosen one performs its operation on resumption and runs to its next
 * primitive. The lock blocks a thread while another holds it, the park blocks
 * until the epoch moves, and a yield blocks until another process has written
 * shared state since the yielder last returned from a yield, which is the
 * only spin that can observe anything new.
 *
 * Two searches. The gate's is explicit-state: every scheduling point is
 * checkpointed (the live bytes of every stack, the core, the device, the
 * checker's bookkeeping, the schedule's state), digested and never explored
 * twice, and a step that touches no shared object is taken alone. The
 * reference is a depth-first walk over the choice made at every step with
 * re-execution from a fresh core, which explores every interleaving and is
 * feasible at one thread, where it must reach exactly the arms the gate's
 * search reaches. Every §11 invariant is checked after every step, and a
 * terminal state is checked for the things liveness would otherwise hide: an
 * entry that never returned, a completion never drained, a stack parked with
 * nothing left to wake it, a spinner with nothing left to change what it spins
 * on. */

#define _DEFAULT_SOURCE 1
#define _DARWIN_C_SOURCE 1

#include "core.h"
#include "prim.h"
#include "enumerate.h"
#include "switch.h"

#include <signal.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#if !defined(WF_SCHED_ENUMERATE)
#error "enumerate.c is compiled with -DWF_SCHED_ENUMERATE"
#endif

#define ENUM_MAX_THREADS 4u
#define ENUM_MAX_RECORDS 8u
#define ENUM_MAX_PROCS (ENUM_MAX_THREADS + ENUM_MAX_RECORDS)
#define ENUM_MAX_STACKS 8u
#define ENUM_MAX_DEPTH 40000u
#define ENUM_MAX_WORDS 160u
#define ENUM_HOST_STACK_BYTES (256u * 1024u)
#define ENUM_POOL_STACK_BYTES (256u * 1024u)

_Static_assert(ENUM_MAX_PROCS <= 32u, "process sets are one unsigned");
_Static_assert(WF_SCHED_MAX_THREADS >= ENUM_MAX_THREADS, "the core admits the threads");
_Static_assert(WF_SCHED_MAX_STACKS >= ENUM_MAX_STACKS, "the core admits the stacks");

/* ------------------------------------------------------------- the model */

enum op_kind {
    OP_NONE,
    OP_LOAD,
    OP_STORE,
    OP_CAS,
    OP_SWITCH,
    OP_EPOCH,
    OP_PARK,
    OP_WAKE,
    OP_LOCK,
    OP_UNLOCK,
    OP_YIELD,
    OP_PROGRESS,
    OP_SUBMIT,
    OP_COMPLETE,
    OP_OR,
    OP_AND
};

static const char *const op_names[] = {
    "none", "load", "store", "cas", "switch", "epoch", "park", "wake",
    "lock", "unlock", "yield", "progress", "submit", "complete", "or", "and"
};

typedef struct op {
    enum op_kind kind;
    const void *word;
    unsigned width;              /* 4 or 8 bytes, 0 when not a word */
    unsigned long long value;    /* the value stored, or the CAS's desired */
    unsigned long long expected; /* the CAS's expected */
    uint64_t observed;           /* the park's epoch */
    unsigned record;             /* the device step's record */
    enum wf_prim_section section; /* the lock's section */
} op;

/* What a shared word is, so the checks can name it. */
enum word_class {
    W_PHASE,
    W_IDLE,
    W_STATUS,
    W_TOP,
    W_BOTTOM,
    W_FREE,
    W_SLOT_STATE,
    W_SLOT_WAITER,
    W_IO_STATE,
    W_IO_WAITER
};

typedef struct word {
    const void *address;
    enum word_class cls;
    unsigned index;
    unsigned sub;
} word;

typedef enum actor_state { A_NEW, A_RUNNABLE, A_FINISHED, A_DEAD } actor_state;

typedef struct actor {
    unsigned index;
    actor_state state;
    void *sp;
    unsigned char *host_stack;
    op pending;
    int has_pending;
    /* A wake since this thread's last epoch capture: the host's epoch
     * comparison, kept as one bit so that equal states have equal bytes. */
    int wake_flag;
    /* A write by another process since this thread last returned from a
     * yield: the only spin that can observe anything new. */
    int others_wrote;
    int in_window;
    int taken_first;
    int park_blocked;
    int status;
} actor;

enum record_state { R_OUTSTANDING, R_COMPLETED, R_DRAINED };

typedef struct io_record {
    wf_sched_record *record;   /* NULL once retired: its frame may be gone */
    enum record_state state;
    unsigned long long completed_at;
    int retired;
} io_record;

typedef struct stack_info {
    int ready_owed;
    int on_ready_before;
    int parked_by;
    int parked_by_transit;
    /* 1 when the registration this stack last took back was an I/O record's,
     * 0 for a compute slot's, -1 when it never took one back. */
    int cancel_kind;
    /* The thread that claimed this stack's registration and has not yet
     * moved its phase, else -1: while it holds the claim no record names the
     * stack and the stack is still parked. */
    int claimed_by;
    unsigned io_depth;
} stack_info;

/* One executed transition of the trace, and the choice state at the state it
 * was taken from. Index 1 is the first transition. */
typedef struct step {
    unsigned proc;
    enum op_kind kind;
    unsigned pre_enabled;
    unsigned backtrack;
    unsigned done;
} step;

wf_sched_core wf_enum_core;

static unsigned cfg_threads = 1u;
static unsigned cfg_stacks = 2u;
static unsigned cfg_bound = 20000u;
static unsigned long long cfg_limit = 0;
enum search_mode { SEARCH_DFS, SEARCH_STATE };
static enum search_mode cfg_search = SEARCH_STATE;
static int cfg_verbose = 0;
static const char *cfg_schedule = NULL;

static void *controller_sp;
static actor *current;
static actor actors[ENUM_MAX_THREADS];
static io_record records[ENUM_MAX_RECORDS];
static unsigned record_count;
static unsigned long long completion_serial;
static int lock_holder = -1;
static word words[ENUM_MAX_WORDS];
static unsigned word_count;
static stack_info stacks[ENUM_MAX_STACKS];
static step trace[ENUM_MAX_DEPTH + 2u];
static unsigned trace_len;
static unsigned depth;
static int failed;
static char failure[512];
static wf_enum_coverage cov;
static unsigned char *pool_arena;
static size_t pool_arena_bytes;
static const wf_enum_schedule *schedule;
static unsigned long long execution_serial;
static unsigned long long states_seen;
static unsigned long long states_pruned;

/* ------------------------------------------------------------ reporting */

static const char *proc_name(unsigned proc, char *buffer, size_t bytes) {
    if (proc < cfg_threads) {
        (void)snprintf(buffer, bytes, "t%u", proc);
    } else {
        (void)snprintf(buffer, bytes, "d%u", proc - cfg_threads);
    }
    return buffer;
}

static void print_trace(FILE *out) {
    unsigned index;
    char name[16];
    (void)fprintf(out, "enumerate: trace (%u steps):", depth);
    for (index = 1; index <= depth; index += 1u) {
        (void)fprintf(
            out, " %s:%s", proc_name(trace[index].proc, name, sizeof name),
            op_names[trace[index].kind]
        );
    }
    (void)fprintf(out, "\n");
    (void)fprintf(out, "enumerate: replay:");
    for (index = 1; index <= depth; index += 1u) {
        (void)fprintf(out, " %u", trace[index].proc);
    }
    (void)fprintf(out, "\n");
}

static int stack_index_of(const void *pointer) {
    unsigned index;
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        const wf_sched_stack *stack = wf_enum_core.stacks[index];
        if ((const unsigned char *)pointer >= stack->low
            && (const unsigned char *)pointer < stack->high) {
            return (int)index;
        }
    }
    return -1;
}

static void print_state(FILE *out) {
    unsigned index;
    char name[16];
    (void)fprintf(out, "enumerate: execution %llu, step %u, lock %d\n",
        execution_serial, depth, lock_holder);
    for (index = 0; index < cfg_threads; index += 1u) {
        const actor *a = &actors[index];
        int on = a->state == A_RUNNABLE ? stack_index_of(a->sp) : -2;
        (void)fprintf(
            out, "enumerate:   %s state=%d on=%d pending=%s", proc_name(index, name, sizeof name),
            (int)a->state, on, a->has_pending ? op_names[a->pending.kind] : "-"
        );
        if (a->has_pending && a->pending.kind == OP_PARK) {
            (void)fprintf(out, "(%llu)", (unsigned long long)a->pending.observed);
        }
        (void)fprintf(out, " core.stack=%d\n",
            wf_enum_core.threads[index].stack == NULL ? -1 : (int)wf_enum_core.threads[index].stack->index);
    }
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        const wf_sched_stack *stack = wf_enum_core.stacks[index];
        (void)fprintf(out, "enumerate:   stack %u phase=%u owed=%d\n", index, stack->phase, stacks[index].ready_owed);
    }
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].record == NULL) {
            (void)fprintf(out, "enumerate:   record %u retired device=%d\n", index, (int)records[index].state);
            continue;
        }
        (void)fprintf(
            out, "enumerate:   record %u state=%u waiter=%d device=%d\n", index,
            records[index].record->state,
            records[index].record->waiter == NULL ? -1 : (int)records[index].record->waiter->index,
            (int)records[index].state
        );
    }
    (void)fprintf(out, "enumerate:   ready_head=%d free_head=%d status_posted=%u idle=%llx\n",
        wf_enum_core.ready_head == NULL ? -1 : (int)wf_enum_core.ready_head->index,
        wf_enum_core.free_head == NULL ? -1 : (int)wf_enum_core.free_head->index,
        wf_enum_core.status_posted, wf_enum_core.idle);
}

/* Ends the execution as a failure. Callable from the controller and from an
 * actor; an actor never returns from it. */
static void fail_execution(const char *format, ...) {
    va_list arguments;
    if (!failed) {
        failed = 1;
        va_start(arguments, format);
        (void)vsnprintf(failure, sizeof failure, format, arguments);
        va_end(arguments);
    }
    if (current != NULL) {
        actor *a = current;
        a->state = A_DEAD;
        a->has_pending = 0;
        current = NULL;
        wf_switch_raw(&a->sp, controller_sp);
        /* Never resumed. */
        abort();
    }
}

void wf_enum_fail(const char *reason) {
    fail_execution("schedule: %s", reason);
}

void wf_prim_fail(const char *reason) {
    if (current == NULL) {
        (void)fprintf(stderr, "enumerate: the core failed outside an actor: %s\n", reason);
        exit(2);
    }
    fail_execution("core: %s", reason);
}

/* -------------------------------------------------------------- actors */

static void actor_yield(void) {
    actor *a = current;
    current = NULL;
    wf_switch_raw(&a->sp, controller_sp);
}

static void controller_resume(actor *a) {
    current = a;
    wf_switch_raw(&controller_sp, a->sp);
    current = NULL;
}

static unsigned current_index(void) {
    if (current == NULL) {
        (void)fprintf(stderr, "enumerate: a primitive was reached outside an actor\n");
        exit(2);
    }
    return current->index;
}

/* Announces `o` as the calling actor's next step and waits to be chosen. */
static void announce(const op *o) {
    actor *a = current;
    a->pending = *o;
    a->has_pending = 1;
    actor_yield();
    a->has_pending = 0;
}

static int current_exec_stack(void) {
    void *here = __builtin_frame_address(0);
    return stack_index_of(here);
}

unsigned wf_prim_thread_index(void) {
    return current_index();
}

void wf_prim_set_thread_index(unsigned index) {
    if (index != current_index()) {
        fail_execution("core: thread %u set its index to %u", current_index(), index);
    }
}

/* The entry body under the harness: the entry thread creates the workers
 * first, reserving each one's stack as the runtime's pool start will, then
 * runs the schedule's body. */
static void entry_main(void *argument) {
    unsigned index;
    for (index = 1; index < cfg_threads; index += 1u) {
        if (wf_sched_start_thread(&wf_enum_core, index) != 0) {
            fail_execution("a thread start found the free list empty (item 21)");
        }
        actors[index].state = A_RUNNABLE;
    }
    schedule->main(argument);
}

static void actor_main(void *argument) {
    actor *a = argument;
    void (*entry)(void *) = a->index == 0u ? entry_main : NULL;
    a->status = wf_sched_run(&wf_enum_core, a->index, entry, NULL);
    if (a->index != 0u) {
        fail_execution("core: worker %u returned from wf_sched_run", a->index);
    }
    a->state = A_FINISHED;
    a->has_pending = 0;
    current = NULL;
    wf_switch_raw(&a->sp, controller_sp);
    abort();
}

/* ------------------------------------------------------------- the words */

static const word *word_of(const void *address) {
    unsigned index;
    for (index = 0; index < word_count; index += 1u) {
        if (words[index].address == address) {
            return &words[index];
        }
    }
    return NULL;
}

static void add_word(const void *address, enum word_class cls, unsigned index, unsigned sub) {
    if (word_count >= ENUM_MAX_WORDS) {
        (void)fprintf(stderr, "enumerate: too many words\n");
        exit(2);
    }
    words[word_count].address = address;
    words[word_count].cls = cls;
    words[word_count].index = index;
    words[word_count].sub = sub;
    word_count += 1u;
}

static const word *require_word(const void *address, const char *what) {
    const word *w = word_of(address);
    if (w == NULL) {
        fail_execution("core: %s reached a word the enumerator does not model (%p)", what, address);
    }
    return w;
}

static void build_words(void) {
    unsigned index;
    unsigned slot;
    word_count = 0;
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        add_word(&wf_enum_core.stacks[index]->phase, W_PHASE, index, 0);
    }
    add_word(&wf_enum_core.idle, W_IDLE, 0, 0);
    add_word(&wf_enum_core.status_posted, W_STATUS, 0, 0);
    for (index = 0; index < wf_enum_core.thread_count; index += 1u) {
        wf_sched_lane *lane = &wf_enum_core.lanes[index];
        add_word(&lane->top, W_TOP, index, 0);
        add_word(&lane->bottom, W_BOTTOM, index, 0);
        add_word(&lane->free_head, W_FREE, index, 0);
        for (slot = 0; slot < WF_SCHED_LANE_SLOTS; slot += 1u) {
            add_word(&lane->slots[slot].record.state, W_SLOT_STATE, index, slot);
            add_word(&lane->slots[slot].record.waiter, W_SLOT_WAITER, index, slot);
        }
    }
}

/* Every record the enumerator knows: the lanes' slots and the submitted I/O
 * records. Answers the record whose waiter is `stack`, and whether it is an
 * I/O record. */
static wf_sched_record *record_waited_by(const wf_sched_stack *stack, int *is_io) {
    unsigned lane;
    unsigned slot;
    unsigned index;
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].record != NULL && records[index].record->waiter == stack) {
            *is_io = 1;
            return records[index].record;
        }
    }
    for (lane = 0; lane < wf_enum_core.thread_count; lane += 1u) {
        for (slot = 0; slot < WF_SCHED_LANE_SLOTS; slot += 1u) {
            wf_sched_record *record = &wf_enum_core.lanes[lane].slots[slot].record;
            if (record->waiter == stack) {
                *is_io = 0;
                return record;
            }
        }
    }
    *is_io = -1;
    return NULL;
}

unsigned wf_enum_parked_count(void) {
    unsigned index;
    unsigned count = 0;
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        /* SUSPENDED only: a stack committed and waiting. A SUSPENDING or
         * NOTIFIED stack is still being executed, and a READY one is about
         * to be. */
        if (wf_enum_core.stacks[index]->phase == WF_SCHED_STACK_SUSPENDED) {
            count += 1u;
        }
    }
    return count;
}

/* --------------------------------------------- observing a phase change */

static void note_phase(const word *w, unsigned old, unsigned new, enum op_kind kind) {
    unsigned index = w->index;
    stack_info *info = &stacks[index];
    int on_it = current_exec_stack() == (int)index;
    int is_io;
    if (old == new) {
        return;
    }
    if (old == WF_SCHED_STACK_RUNNING && new == WF_SCHED_STACK_SUSPENDING && kind == OP_CAS) {
        if (!on_it) {
            fail_execution("a park began on stack %u from another stack", index);
        }
        info->parked_by = (int)current_index();
        cov.begin += 1u;
        return;
    }
    if (old == WF_SCHED_STACK_SUSPENDING && new == WF_SCHED_STACK_SUSPENDED && kind == OP_CAS) {
        if (on_it) {
            fail_execution("stack %u was committed SUSPENDED by the thread still running on it", index);
        }
        cov.suspended += 1u;
        return;
    }
    if (old == WF_SCHED_STACK_SUSPENDING && new == WF_SCHED_STACK_NOTIFIED && kind == OP_CAS) {
        if (on_it) {
            fail_execution("stack %u was notified by itself", index);
        }
        if (info->claimed_by != (int)current_index()) {
            fail_execution("stack %u was notified by a thread that does not own its registration", index);
        }
        info->claimed_by = -1;
        is_io = info->cancel_kind;
        if (is_io < 0) {
            fail_execution("stack %u was notified while no record names it", index);
        }
        if (is_io) {
            cov.publish_io += 1u;
        } else {
            cov.publish_compute += 1u;
        }
        cov.notified += 1u;
        return;
    }
    if (old == WF_SCHED_STACK_SUSPENDED && new == WF_SCHED_STACK_READY && kind == OP_CAS) {
        if (on_it) {
            fail_execution("stack %u was made READY by itself", index);
        }
        if (info->claimed_by != (int)current_index()) {
            fail_execution("stack %u was made READY by a thread that does not own its registration", index);
        }
        info->claimed_by = -1;
        is_io = info->cancel_kind;
        if (is_io < 0) {
            fail_execution("stack %u was made READY while no record names it", index);
        }
        if (is_io) {
            cov.publish_io += 1u;
        } else {
            cov.publish_compute += 1u;
        }
        if (info->ready_owed) {
            fail_execution("stack %u reached READY twice without an enqueue", index);
        }
        info->ready_owed = 1;
        cov.ready_from_suspended += 1u;
        return;
    }
    if (old == WF_SCHED_STACK_NOTIFIED && new == WF_SCHED_STACK_READY && kind == OP_CAS) {
        if (on_it) {
            fail_execution("stack %u committed NOTIFIED to READY by itself", index);
        }
        if (info->ready_owed) {
            fail_execution("stack %u reached READY twice without an enqueue", index);
        }
        info->ready_owed = 1;
        cov.ready_from_notified += 1u;
        return;
    }
    if (old == WF_SCHED_STACK_NOTIFIED && new == WF_SCHED_STACK_RUNNING && kind == OP_CAS) {
        /* The claim protocol leaves no notification for a cancel to consume:
         * a publisher notifies only a registration it owns, and a parker
         * cancels only a registration it took back. */
        fail_execution("stack %u cancelled a NOTIFIED park, which the claim protocol forbids", index);
    }
    if (old == WF_SCHED_STACK_SUSPENDING && new == WF_SCHED_STACK_RUNNING && kind == OP_CAS) {
        if (!on_it) {
            fail_execution("stack %u was cancelled from another stack", index);
        }
        is_io = info->cancel_kind;
        if (is_io < 0) {
            fail_execution("stack %u cancelled while no record names it", index);
        }
        if (is_io) {
            cov.cancel_suspending_io += 1u;
        } else {
            cov.cancel_suspending_compute += 1u;
        }
        info->parked_by = -1;
        info->cancel_kind = -1;
        return;
    }
    if (old == WF_SCHED_STACK_READY && new == WF_SCHED_STACK_RUNNING && kind == OP_STORE) {
        if (on_it) {
            fail_execution("stack %u was resumed by a thread already on it", index);
        }
        if (info->ready_owed) {
            fail_execution("stack %u was resumed while its enqueue was still owed", index);
        }
        info->parked_by_transit = 0;
        cov.resume += 1u;
        if (info->parked_by != (int)current_index()) {
            cov.resume_foreign += 1u;
        }
        info->parked_by = -1;
        info->cancel_kind = -1;
        return;
    }
    if (old == WF_SCHED_STACK_RUNNING && new == WF_SCHED_STACK_EMPTY && kind == OP_STORE) {
        if (!on_it) {
            fail_execution("stack %u was marked EMPTY from another stack", index);
        }
        cov.empty += 1u;
        return;
    }
    if (old == WF_SCHED_STACK_EMPTY && new == WF_SCHED_STACK_RUNNING && kind == OP_STORE) {
        if (on_it) {
            fail_execution("stack %u was taken by a thread already on it", index);
        }
        cov.take += 1u;
        if (!current->taken_first) {
            current->taken_first = 1;
            if (wf_enum_parked_count() > 0u) {
                cov.start_after_park += 1u;
            }
        }
        return;
    }
    fail_execution(
        "stack %u moved from phase %u to %u by a %s, which the state machine has no edge for",
        index, old, new, op_names[kind]
    );
}

/* Observes a write of `value` to a word that is not a phase. */
static void note_word_write(const word *w, unsigned long long old, unsigned long long value, enum op_kind kind) {
    unsigned me = current_index();
    switch (w->cls) {
    case W_SLOT_STATE:
    case W_IO_STATE:
        if (value == WF_SCHED_COMPLETING && old != WF_SCHED_PENDING) {
            fail_execution("a record began completing from state %llu", old);
        }
        if (value == WF_SCHED_DONE && old != WF_SCHED_COMPLETING) {
            fail_execution("a record was stored DONE from state %llu, not COMPLETING", old);
        }
        if (value != WF_SCHED_DONE && value != WF_SCHED_PENDING && value != WF_SCHED_COMPLETING) {
            fail_execution("a record was given the state %llu", value);
        }
        break;
    case W_SLOT_WAITER:
    case W_IO_WAITER:
        if (value == 1ull) {
            /* The in-place waiter's marker: no stack to check. */
            break;
        }
        if (value == 0ull && old == 1ull) {
            break;
        }
        if (value == 0ull && old != 0ull) {
            /* Whoever takes a registration back, or claims it, is the one
             * that will move the phase: remember the record's kind for the
             * cancel it may lead to. */
            int on = stack_index_of((const void *)(uintptr_t)old);
            if (on >= 0 && current_exec_stack() == on) {
                stacks[on].cancel_kind = w->cls == W_IO_WAITER ? 1 : 0;
            } else if (on >= 0 && kind == OP_CAS) {
                if (stacks[on].claimed_by >= 0) {
                    fail_execution("stack %d's registration was claimed twice", on);
                }
                stacks[on].claimed_by = (int)me;
                stacks[on].cancel_kind = w->cls == W_IO_WAITER ? 1 : 0;
            }
        }
        if (value != 0ull) {
            int on = stack_index_of((const void *)(uintptr_t)value);
            if (on < 0) {
                fail_execution("a waiter was registered that is no pool stack (word class %d index %u at %p, value %llx, old %llx)",
                    (int)w->cls, w->index, w->address, value, old);
            }
            if (current_exec_stack() != on) {
                fail_execution("stack %d was registered as a waiter by a thread not on it", on);
            }
            if (wf_enum_core.stacks[on]->phase != WF_SCHED_STACK_SUSPENDING) {
                fail_execution("stack %d registered as a waiter in phase %u, not SUSPENDING", on,
                    wf_enum_core.stacks[on]->phase);
            }
        }
        break;
    case W_BOTTOM:
        if (w->index != me) {
            fail_execution("thread %u wrote the bottom of lane %u (I2)", me, w->index);
        }
        break;
    case W_FREE:
        if (kind == OP_CAS) {
            wf_sched_lane *lane = &wf_enum_core.lanes[w->index];
            unsigned expected = (unsigned)old;
            unsigned desired = (unsigned)value;
            int is_pop = expected != WF_SCHED_NO_SLOT && lane->slots[expected].next_free == desired;
            int is_push = desired != WF_SCHED_NO_SLOT && lane->slots[desired].next_free == expected;
            if (is_pop && is_push) {
                fail_execution("a free-list swing on lane %u reads as both a pop and a push", w->index);
            }
            if (!is_pop && !is_push) {
                fail_execution("a free-list swing on lane %u is neither a pop nor a push", w->index);
            }
            if (is_pop && w->index != me) {
                fail_execution("thread %u popped the slot free list of lane %u (I3)", me, w->index);
            }
        }
        break;
    case W_STATUS:
        if (value != 0ull) {
            const actor *entry = &actors[0];
            if (me != 0u) {
                cov.post_by_worker += 1u;
            }
            if (entry->state == A_RUNNABLE && entry->has_pending && entry->pending.kind == OP_PARK
                && !entry->wake_flag) {
                cov.post_while_asleep += 1u;
            } else if (entry->in_window) {
                cov.post_in_window += 1u;
            } else {
                cov.post_elsewhere += 1u;
            }
        }
        break;
    case W_IDLE:
        current->in_window = 0;
        current->wake_flag = 0;
        break;
    default:
        break;
    }
}

/* -------------------------------------------------------- primitive 1 */

static unsigned long long read_word(const void *address, unsigned width) {
    if (width == 4u) {
        return *(const unsigned *)address;
    }
    if (width == 8u) {
        return *(const unsigned long long *)address;
    }
    return (unsigned long long)(uintptr_t)*(void *const *)address;
}

static void write_word(void *address, unsigned width, unsigned long long value) {
    if (width == 4u) {
        *(unsigned *)address = (unsigned)value;
    } else if (width == 8u) {
        *(unsigned long long *)address = value;
    } else {
        *(void **)address = (void *)(uintptr_t)value;
    }
}

static unsigned long long prim_load(const void *address, unsigned width) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_LOAD;
    o.word = address;
    o.width = width;
    (void)require_word(address, "a load");
    announce(&o);
    return read_word(address, width);
}

static void prim_store(void *address, unsigned width, unsigned long long value) {
    op o;
    const word *w = require_word(address, "a store");
    unsigned long long old;
    memset(&o, 0, sizeof o);
    o.kind = OP_STORE;
    o.word = address;
    o.width = width;
    o.value = value;
    announce(&o);
    /* Looked up again: another thread's submit or retirement may have moved
     * the table's entries while this step waited to be chosen. */
    w = require_word(address, "a store");
    old = read_word(address, width);
    write_word(address, width, value);
    if (w->cls == W_PHASE) {
        note_phase(w, (unsigned)old, (unsigned)value, OP_STORE);
    } else {
        note_word_write(w, old, value, OP_STORE);
    }
}

static int prim_cas(void *address, unsigned width, unsigned long long *expected, unsigned long long desired) {
    op o;
    const word *w = require_word(address, "a compare-exchange");
    unsigned long long old;
    memset(&o, 0, sizeof o);
    o.kind = OP_CAS;
    o.word = address;
    o.width = width;
    o.value = desired;
    o.expected = *expected;
    announce(&o);
    w = require_word(address, "a compare-exchange");
    old = read_word(address, width);
    if (old != *expected) {
        *expected = old;
        return 0;
    }
    write_word(address, width, desired);
    if (w->cls == W_PHASE) {
        note_phase(w, (unsigned)old, (unsigned)desired, OP_CAS);
    } else {
        note_word_write(w, old, desired, OP_CAS);
    }
    return 1;
}

unsigned wf_prim_load_u(const unsigned *word_, enum wf_prim_order order) {
    (void)order;
    return (unsigned)prim_load(word_, 4u);
}

void wf_prim_store_u(unsigned *word_, unsigned value, enum wf_prim_order order) {
    (void)order;
    prim_store(word_, 4u, value);
}

int wf_prim_cas_u(unsigned *word_, unsigned *expected, unsigned desired, enum wf_prim_order success, enum wf_prim_order failure_order) {
    unsigned long long seen = *expected;
    int done;
    (void)success;
    (void)failure_order;
    done = prim_cas(word_, 4u, &seen, desired);
    *expected = (unsigned)seen;
    return done;
}

void *wf_prim_load_p(void *const *word_, enum wf_prim_order order) {
    (void)order;
    return (void *)(uintptr_t)prim_load(word_, 0u);
}

void wf_prim_store_p(void **word_, void *value, enum wf_prim_order order) {
    (void)order;
    prim_store(word_, 0u, (unsigned long long)(uintptr_t)value);
}

int wf_prim_cas_p(void **word_, void **expected, void *desired, enum wf_prim_order success, enum wf_prim_order failure_order) {
    unsigned long long seen = (unsigned long long)(uintptr_t)*expected;
    int done;
    (void)success;
    (void)failure_order;
    done = prim_cas(word_, 0u, &seen, (unsigned long long)(uintptr_t)desired);
    *expected = (void *)(uintptr_t)seen;
    return done;
}

unsigned long long wf_prim_load_q(const unsigned long long *word_, enum wf_prim_order order) {
    (void)order;
    return prim_load(word_, 8u);
}

void wf_prim_store_q(unsigned long long *word_, unsigned long long value, enum wf_prim_order order) {
    (void)order;
    prim_store(word_, 8u, value);
}

int wf_prim_cas_q(unsigned long long *word_, unsigned long long *expected, unsigned long long desired, enum wf_prim_order success, enum wf_prim_order failure_order) {
    (void)success;
    (void)failure_order;
    return prim_cas(word_, 8u, expected, desired);
}

static unsigned long long prim_or_and(unsigned long long *address, unsigned long long bits, enum op_kind kind) {
    op o;
    const word *w = require_word(address, "a read-modify-write");
    unsigned long long old;
    memset(&o, 0, sizeof o);
    o.kind = kind;
    o.word = address;
    o.width = 8u;
    o.value = bits;
    announce(&o);
    w = require_word(address, "a read-modify-write");
    old = *address;
    *address = kind == OP_OR ? (old | bits) : (old & bits);
    if (w->cls == W_PHASE) {
        fail_execution("a phase word was changed by a read-modify-write");
    }
    note_word_write(w, old, *address, kind);
    return old;
}

unsigned long long wf_prim_or_q(unsigned long long *word_, unsigned long long bits, enum wf_prim_order order) {
    (void)order;
    return prim_or_and(word_, bits, OP_OR);
}

unsigned long long wf_prim_and_q(unsigned long long *word_, unsigned long long bits, enum wf_prim_order order) {
    (void)order;
    return prim_or_and(word_, bits, OP_AND);
}

/* ---------------------------------------------------- primitives 2 to 7 */

/* The epoch moves: every thread's next park returns, and one asleep wakes.
 * Only a thread inside its capture-to-park window can observe it; for any
 * other the next capture clears the flag before anything reads it, so the
 * flag is left clear and equal states stay equal. */
static void wake_all(void) {
    unsigned index;
    for (index = 0; index < cfg_threads; index += 1u) {
        if (actors[index].in_window) {
            actors[index].wake_flag = 1;
        }
    }
}

/* A write-class step by `writer` (ENUM_MAX_THREADS for the device): every
 * other thread's spin may now observe something new. */
static void note_write(unsigned writer) {
    unsigned index;
    for (index = 0; index < cfg_threads; index += 1u) {
        if (index != writer) {
            actors[index].others_wrote = 1;
        }
    }
}

void wf_prim_switch(void **save, void *load) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_SWITCH;
    announce(&o);
    wf_switch_raw(save, load);
}

void *wf_prim_prepare_stack(
    void *top,
    size_t bytes,
    void (*entry)(void *),
    void *argument
) {
    /* The enumerator's stacks are the host's, so the slot *is* the stack and
     * its size is not needed to prepare a frame at its top. */
    (void)bytes;
    return wf_switch_prepare(top, entry, argument);
}

void wf_prim_set_bounds(unsigned char *low, unsigned char *high) {
    (void)low;
    (void)high;
}

uint64_t wf_prim_epoch(void) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_EPOCH;
    announce(&o);
    current->in_window = 1;
    current->wake_flag = 0;
    return 0;
}

void wf_prim_park(uint64_t observed) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_PARK;
    o.observed = observed;
    if (observed != 0u) {
        fail_execution("core: a park on an epoch that was not the last one captured");
    }
    current->park_blocked = !current->wake_flag;
    announce(&o);
    if (!current->wake_flag) {
        fail_execution("harness: a park was resumed with the epoch unchanged");
    }
    current->in_window = 0;
    current->wake_flag = 0;
}

void wf_prim_wake(void) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_WAKE;
    announce(&o);
    wake_all();
}

size_t wf_prim_stack_stride(size_t bytes) {
    size_t page = 4096u;
    return page + (bytes + page - 1u) / page * page;
}

unsigned char *wf_prim_reserve(unsigned count, size_t bytes) {
    size_t stride = wf_prim_stack_stride(bytes);
    if (pool_arena == NULL) {
        unsigned index;
        pool_arena_bytes = stride * (size_t)ENUM_MAX_STACKS;
        pool_arena = mmap(NULL, pool_arena_bytes, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (pool_arena == MAP_FAILED) {
            (void)fprintf(stderr, "enumerate: the pool reservation failed\n");
            exit(2);
        }
        for (index = 0; index < ENUM_MAX_STACKS; index += 1u) {
            (void)mprotect(pool_arena + (size_t)index * stride, 4096u, PROT_NONE);
        }
    }
    if (stride * (size_t)count > pool_arena_bytes) {
        return NULL;
    }
    return pool_arena;
}

void wf_prim_lock(enum wf_prim_section section) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_LOCK;
    o.section = section;
    announce(&o);
    if (lock_holder >= 0) {
        fail_execution("harness: a lock was granted while held");
    }
    lock_holder = (int)current_index();
}

void wf_prim_unlock(void) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_UNLOCK;
    announce(&o);
    if (lock_holder != (int)current_index()) {
        fail_execution("core: thread %u unlocked a mutex it does not hold", current_index());
    }
    lock_holder = -1;
}

void wf_prim_yield(void) {
    op o;
    memset(&o, 0, sizeof o);
    o.kind = OP_YIELD;
    announce(&o);
    current->others_wrote = 0;
}

/* The oldest completed record the drain has not delivered yet. */
static io_record *oldest_completed(void) {
    unsigned index;
    io_record *oldest = NULL;
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].state == R_COMPLETED
            && (oldest == NULL || records[index].completed_at < oldest->completed_at)) {
            oldest = &records[index];
        }
    }
    return oldest;
}

int wf_prim_progress(struct wf_sched_core *core) {
    int progressed = 0;
    for (;;) {
        op o;
        io_record *next;
        memset(&o, 0, sizeof o);
        o.kind = OP_PROGRESS;
        announce(&o);
        next = oldest_completed();
        if (next == NULL) {
            break;
        }
        next->state = R_DRAINED;
        next->completed_at = 0;
        progressed = 1;
        wf_sched_complete(core, next->record);
    }
    return progressed;
}

/* ------------------------------------------------------- the schedule API */

unsigned wf_enum_threads(void) {
    return cfg_threads;
}

unsigned wf_enum_stacks(void) {
    return cfg_stacks;
}

void wf_enum_submit(wf_sched_record *record) {
    op o;
    unsigned index;
    if (record_count >= ENUM_MAX_RECORDS) {
        fail_execution("schedule: more than %u I/O records", ENUM_MAX_RECORDS);
    }
    if (record->state != WF_SCHED_PENDING || record->waiter != NULL) {
        fail_execution("schedule: a record was submitted that is not PENDING and unwaited");
    }
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].record != NULL && records[index].record == record) {
            fail_execution("schedule: a record was submitted twice");
        }
    }
    /* Padding included, so that the record's bytes are a function of its
     * state. */
    memset(record, 0, sizeof *record);
    record->state = WF_SCHED_PENDING;
    memset(&o, 0, sizeof o);
    o.kind = OP_SUBMIT;
    o.record = record_count;
    announce(&o);
    records[record_count].record = record;
    records[record_count].state = R_OUTSTANDING;
    records[record_count].completed_at = 0;
    records[record_count].retired = 0;
    add_word(&record->state, W_IO_STATE, record_count, 0);
    add_word(&record->waiter, W_IO_WAITER, record_count, 0);
    record_count += 1u;
}

static void remove_word(const void *address) {
    unsigned index;
    for (index = 0; index < word_count; index += 1u) {
        if (words[index].address == address) {
            words[index] = words[word_count - 1u];
            word_count -= 1u;
            return;
        }
    }
    fail_execution("harness: a word to forget was never known");
}

void wf_enum_join_io(wf_sched_record *record) {
    int on = current_exec_stack();
    unsigned index;
    if (on < 0) {
        fail_execution("schedule: an I/O join on a host stack");
    }
    stacks[on].io_depth += 1u;
    wf_sched_join(&wf_enum_core, record, 1);
    stacks[on].io_depth -= 1u;
    /* The join returned, so the record is DONE and unwaited; the frame that
     * holds it may now die, so the enumerator forgets its address. */
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].record == record) {
            if (records[index].state != R_DRAINED || record->state != WF_SCHED_DONE) {
                fail_execution("an I/O join returned before its record was drained and DONE");
            }
            if (record->waiter != NULL) {
                fail_execution("an I/O join returned with its waiter still registered (§6)");
            }
            remove_word(&record->state);
            remove_word(&record->waiter);
            memset(&records[index], 0, sizeof records[index]);
            records[index].state = R_DRAINED;
            records[index].retired = 1;
            return;
        }
    }
    fail_execution("schedule: a join of a record that was never submitted");
}

void wf_enum_io(wf_sched_record *record) {
    wf_sched_record_init(record);
    wf_enum_submit(record);
    wf_enum_join_io(record);
}

typedef struct hand_out_frame {
    void (*run)(void *payload);
    void *reserved;
    _Alignas(16) unsigned char payload[1];
} hand_out_frame;

static void hand_out_trampoline(void *frame) {
    hand_out_frame *f = frame;
    int on = current_exec_stack();
    if (on >= 0 && stacks[on].io_depth != 0u) {
        fail_execution("a hand-out ran above an I/O join on stack %d (I1)", on);
    }
    f->run(f->payload);
}

void *wf_enum_hand_out(void (*run)(void *payload), unsigned long payload_bytes) {
    hand_out_frame *f = wf_sched_acquire(&wf_enum_core, payload_bytes + offsetof(hand_out_frame, payload));
    if (f == NULL) {
        return NULL;
    }
    /* The whole frame is zeroed before publication, the payload because a
     * thief may run the body before the caller sees it again, the rest so
     * that a slot's earlier use leaves nothing behind in the state. */
    memset(f, 0, WF_SCHED_FRAME_BYTES);
    f->run = run;
    f->reserved = NULL;
    wf_sched_publish(&wf_enum_core, f, hand_out_trampoline);
    return f->payload;
}

void wf_enum_join(void *payload) {
    wf_sched_join_frame(&wf_enum_core, (unsigned char *)payload - offsetof(hand_out_frame, payload));
}

void wf_enum_release(void *payload) {
    wf_sched_release(&wf_enum_core, (unsigned char *)payload - offsetof(hand_out_frame, payload));
}

/* ------------------------------------------------------------ the checks */

static int is_asleep(const actor *a) {
    return a->state == A_RUNNABLE && a->has_pending && a->pending.kind == OP_PARK
        && !a->wake_flag;
}

static unsigned stack_bit(const wf_sched_stack *stack) {
    if (stack == NULL || stack->index >= wf_enum_core.stack_count
        || wf_enum_core.stacks[stack->index] != stack) {
        fail_execution("a list names a stack the core does not own");
    }
    return 1u << stack->index;
}

/* Walks one intrusive list, failing on a cycle or a duplicate. */
static unsigned walk_list(wf_sched_stack *head, unsigned phase, const char *name, wf_sched_stack **last) {
    unsigned seen = 0;
    unsigned guard = 0;
    wf_sched_stack *stack = head;
    *last = NULL;
    while (stack != NULL) {
        unsigned bit = stack_bit(stack);
        if (seen & bit) {
            fail_execution("stack %u is on the %s list twice", stack->index, name);
        }
        seen |= bit;
        if (stack->phase != phase) {
            fail_execution("stack %u is on the %s list in phase %u", stack->index, name, stack->phase);
        }
        *last = stack;
        stack = stack->next;
        guard += 1u;
        if (guard > wf_enum_core.stack_count) {
            fail_execution("the %s list is cyclic", name);
        }
    }
    return seen;
}

/* The word table names each shared word once, and every I/O word names a
 * live record whose words are at those addresses. */
static void check_words(void) {
    unsigned index;
    unsigned other;
    (void)other;
    for (index = 0; index < word_count; index += 1u) {
        const word *w = &words[index];
        if (w->cls == W_IO_STATE || w->cls == W_IO_WAITER) {
            const io_record *r;
            if (w->index >= record_count) {
                fail_execution("harness: an I/O word names record %u of %u", w->index, record_count);
            }
            r = &records[w->index];
            if (r->record == NULL) {
                fail_execution("harness: an I/O word names retired record %u", w->index);
            }
            if ((w->cls == W_IO_STATE && w->address != (const void *)&r->record->state)
                || (w->cls == W_IO_WAITER && w->address != (const void *)&r->record->waiter)) {
                fail_execution("harness: an I/O word of record %u is at the wrong address", w->index);
            }
        }
    }
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].record != NULL) {
            if (word_of(&records[index].record->state) == NULL || word_of(&records[index].record->waiter) == NULL) {
                fail_execution("harness: live record %u has no word entries", index);
            }
        }
    }
}

static void check_state(void) {
    unsigned index;
    unsigned on_free;
    check_words();
    unsigned on_ready;
    unsigned exec_seen = 0;
    wf_sched_stack *last;
    unsigned alive = 0;
    unsigned asleep = 0;
    int is_io;

    on_free = walk_list(wf_enum_core.free_head, WF_SCHED_STACK_EMPTY, "free", &last);
    on_ready = walk_list(wf_enum_core.ready_head, WF_SCHED_STACK_READY, "ready", &last);
    if (wf_enum_core.ready_tail != last) {
        fail_execution("the ready list's tail does not name its last stack");
    }
    if (on_free & on_ready) {
        fail_execution("a stack is on both lists");
    }

    /* Enqueues and transits, by what appeared on or left the ready list. */
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        stack_info *info = &stacks[index];
        int now = (on_ready >> index) & 1u;
        if (now && !info->on_ready_before) {
            if (info->ready_owed) {
                info->ready_owed = 0;
            } else if (!info->parked_by_transit) {
                fail_execution("stack %u was enqueued without a READY transition, or twice for one park", index);
            }
            info->parked_by_transit = 0;
        }
        if (!now && info->on_ready_before) {
            /* Popped by the thread whose step this was; it holds the stack
             * until it stores RUNNING or offers it back. */
            info->parked_by_transit = 1;
        }
        info->on_ready_before = now;
    }

    /* Where every thread actually executes. */
    for (index = 0; index < cfg_threads; index += 1u) {
        const actor *a = &actors[index];
        int on;
        unsigned other;
        if (a->state != A_RUNNABLE) {
            continue;
        }
        alive += 1u;
        if (is_asleep(a)) {
            asleep += 1u;
            if (((wf_enum_core.idle >> index) & 1ull) == 0ull) {
                fail_execution("thread %u sleeps without its idle bit", index);
            }
        }
        on = stack_index_of(a->sp);
        if (on >= 0) {
            unsigned bit = 1u << on;
            unsigned phase = wf_enum_core.stacks[on]->phase;
            if (exec_seen & bit) {
                fail_execution("two threads execute on stack %d", on);
            }
            exec_seen |= bit;
            if (on_free & bit) {
                fail_execution("thread %u executes on stack %d while it is on the free list", index, on);
            }
            if (on_ready & bit) {
                fail_execution("thread %u executes on stack %d while it is on the ready list", index, on);
            }
            if (phase == WF_SCHED_STACK_SUSPENDED) {
                fail_execution("thread %u executes on stack %d, which was committed SUSPENDED", index, on);
            }
            if (phase == WF_SCHED_STACK_READY) {
                fail_execution("thread %u executes on stack %d, which is READY", index, on);
            }
        }
        for (other = 0; other < index; other += 1u) {
            if (actors[other].state == A_RUNNABLE && wf_enum_core.threads[other].stack != NULL
                && wf_enum_core.threads[other].stack == wf_enum_core.threads[index].stack) {
                fail_execution("threads %u and %u both hold stack %u", other, index,
                    wf_enum_core.threads[index].stack->index);
            }
        }
    }

    /* Every parked stack is waited on by exactly one record; every READY stack
     * is on the list, owed to it, or held by the thread that popped it. */
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        const wf_sched_stack *stack = wf_enum_core.stacks[index];
        unsigned phase = stack->phase;
        if (phase == WF_SCHED_STACK_SUSPENDED
            && record_waited_by(stack, &is_io) == NULL && stacks[index].claimed_by < 0) {
            fail_execution("stack %u is SUSPENDED, no record names it and no publisher claimed it: nothing will resume it", index);
        }
        if (phase == WF_SCHED_STACK_NOTIFIED && stacks[index].claimed_by >= 0) {
            fail_execution("stack %u is NOTIFIED while its registration is still claimed", index);
        }
        if (phase == WF_SCHED_STACK_READY && !((on_ready >> index) & 1u) && !stacks[index].ready_owed
            && !stacks[index].parked_by_transit) {
            fail_execution("stack %u is READY, on no list, owed to none and held by none", index);
        }
        if (((on_free >> index) & 1u) && (exec_seen & (1u << index))) {
            fail_execution("stack %u is free while a thread executes on it", index);
        }
    }

    {
        unsigned parked = wf_enum_parked_count();
        if (parked > cov.max_parked) {
            cov.max_parked = parked;
        }
    }

    /* Item 20, item 16, and the deque's twin: nothing is left behind a sleep.
     * A thread asleep inside an I/O arm is waiting in place above an I/O
     * join, and a hand-out or the status left behind it is the exhausted
     * path's stated price (§2, §3), not a lost wake: its own completion
     * wakes it. A READY stack is never left behind any sleep, because the
     * arm re-tests the ready list where step 4 does (§6). */
    if (alive != 0u && asleep == alive) {
        int idle_sleeper = 0;
        int entry_idle = 0;
        for (index = 0; index < record_count; index += 1u) {
            if (records[index].state == R_OUTSTANDING) {
                cov.all_asleep += 1u;
                break;
            }
        }
        for (index = 0; index < cfg_threads; index += 1u) {
            const actor *a = &actors[index];
            int on;
            if (!is_asleep(a)) {
                continue;
            }
            on = stack_index_of(a->sp);
            if (on >= 0 && stacks[on].io_depth == 0u) {
                idle_sleeper = 1;
                if (index == 0u) {
                    entry_idle = 1;
                }
            }
        }
        if (wf_enum_core.ready_head != NULL) {
            fail_execution("every thread is asleep with a non-empty ready list (item 20)");
        }
        if (wf_enum_core.status_posted != 0u && actors[0].state == A_RUNNABLE && entry_idle) {
            fail_execution("every thread is asleep with the status posted (item 16)");
        }
        for (index = 0; index < wf_enum_core.thread_count && idle_sleeper; index += 1u) {
            const wf_sched_lane *lane = &wf_enum_core.lanes[index];
            if ((long long)(lane->bottom - lane->top) > 0) {
                fail_execution("every thread is asleep with a hand-out on lane %u's deque", index);
            }
        }
    }

    /* I4: the deque holds at most the lane's slots, each at most once. */
    for (index = 0; index < wf_enum_core.thread_count; index += 1u) {
        const wf_sched_lane *lane = &wf_enum_core.lanes[index];
        long long count = (long long)(lane->bottom - lane->top);
        unsigned long long position;
        if (count > (long long)WF_SCHED_LANE_SLOTS || count < -1) {
            fail_execution("lane %u's deque holds %lld entries (I4)", index, count);
        }
        for (position = lane->top; (long long)(lane->bottom - position) > 0; position += 1u) {
            unsigned long long later;
            const wf_sched_slot *slot = lane->buffer[position & (WF_SCHED_LANE_SLOTS - 1u)];
            if (slot < lane->slots || slot >= lane->slots + WF_SCHED_LANE_SLOTS) {
                fail_execution("lane %u's deque names a slot of another lane", index);
            }
            for (later = position + 1u; (long long)(lane->bottom - later) > 0; later += 1u) {
                if (lane->buffer[later & (WF_SCHED_LANE_SLOTS - 1u)] == slot) {
                    fail_execution("lane %u's deque holds one slot twice (I4)", index);
                }
            }
        }
    }

    for (index = 0; index < record_count; index += 1u) {
        unsigned state;
        if (records[index].record == NULL) {
            continue;
        }
        state = records[index].record->state;
        if (state != WF_SCHED_PENDING && state != WF_SCHED_DONE && state != WF_SCHED_COMPLETING) {
            fail_execution("I/O record %u has state %u", index, state);
        }
    }
}

/* At a state with nothing enabled. */
static void check_terminal(void) {
    unsigned index;
    unsigned free_count = 0;
    const char *reason;
    wf_sched_stack *stack;
    for (index = 0; index < cfg_threads; index += 1u) {
        const actor *a = &actors[index];
        if (a->state == A_RUNNABLE && a->has_pending) {
            switch (a->pending.kind) {
            case OP_LOCK:
                fail_execution("thread %u waits for the mutex with nothing left to release it", index);
                break;
            case OP_YIELD:
                fail_execution("thread %u spins with nothing left to change what it spins on", index);
                break;
            case OP_PARK:
                if (index == 0u) {
                    fail_execution("the entry thread sleeps with nothing left to wake it");
                }
                break;
            default:
                fail_execution("thread %u is stopped at a %s that nothing enables", index, op_names[a->pending.kind]);
                break;
            }
        } else if (a->state == A_RUNNABLE) {
            fail_execution("thread %u is runnable and stopped", index);
        }
    }
    if (actors[0].state != A_FINISHED) {
        fail_execution("the entry thread never returned");
    }
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].state == R_OUTSTANDING) {
            fail_execution("I/O record %u was never completed", index);
        }
        if (records[index].state == R_COMPLETED) {
            fail_execution("I/O record %u completed and was never drained: its continuation is buried", index);
        }
        if (records[index].record != NULL && records[index].record->state != WF_SCHED_DONE) {
            fail_execution("I/O record %u ended PENDING", index);
        }
    }
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        unsigned phase = wf_enum_core.stacks[index]->phase;
        if (phase != WF_SCHED_STACK_EMPTY && phase != WF_SCHED_STACK_RUNNING) {
            fail_execution("stack %u ended in phase %u: a parked continuation nothing resumed", index, phase);
        }
    }
    for (stack = wf_enum_core.free_head; stack != NULL; stack = stack->next) {
        free_count += 1u;
    }
    if (free_count + (cfg_threads - 1u) != wf_enum_core.stack_count) {
        fail_execution("%u stacks are free and %u threads sleep on theirs, of %u: a stack was lost",
            free_count, cfg_threads - 1u, wf_enum_core.stack_count);
    }
    reason = schedule->check();
    if (reason != NULL) {
        fail_execution("schedule: %s", reason);
    }
}

/* ------------------------------------------------------------ the search */

static unsigned enabled_set(void) {
    unsigned bits = 0;
    unsigned index;
    for (index = 0; index < cfg_threads; index += 1u) {
        const actor *a = &actors[index];
        int enabled;
        if (a->state != A_RUNNABLE || !a->has_pending) {
            continue;
        }
        switch (a->pending.kind) {
        case OP_LOCK:
            enabled = lock_holder < 0;
            break;
        case OP_PARK:
            enabled = a->wake_flag;
            break;
        case OP_YIELD:
            enabled = a->others_wrote;
            break;
        default:
            enabled = 1;
            break;
        }
        if (enabled) {
            bits |= 1u << index;
        }
    }
    for (index = 0; index < record_count; index += 1u) {
        if (records[index].state == R_OUTSTANDING) {
            bits |= 1u << (cfg_threads + index);
        }
    }
    return bits;
}

static int is_write_kind(enum op_kind kind) {
    switch (kind) {
    case OP_STORE:
    case OP_CAS:
    case OP_LOCK:
    case OP_UNLOCK:
    case OP_WAKE:
    case OP_PROGRESS:
    case OP_SUBMIT:
    case OP_COMPLETE:
    case OP_OR:
    case OP_AND:
        return 1;
    default:
        return 0;
    }
}

static void execute_step(unsigned p, unsigned n);

static unsigned lowest_bit(unsigned bits) {
    unsigned index = 0;
    while (((bits >> index) & 1u) == 0u) {
        index += 1u;
    }
    return index;
}

static void add_statistics(wf_sched_statistics *into, const wf_sched_statistics *after, const wf_sched_statistics *before) {
    into->parks += after->parks - before->parks;
    into->cancels += after->cancels - before->cancels;
    into->resumes += after->resumes - before->resumes;
    into->steals += after->steals - before->steals;
    into->inline_runs += after->inline_runs - before->inline_runs;
    into->exhausted_io_waits += after->exhausted_io_waits - before->exhausted_io_waits;
    into->exhausted_compute_waits += after->exhausted_compute_waits - before->exhausted_compute_waits;
    into->late_parks += after->late_parks - before->late_parks;
    into->line_one += after->line_one - before->line_one;
}

/* Executes process `p`'s pending transition as transition `n`. The core's
 * own counters are credited per explored step, as the difference the step
 * made, so that a state reached twice counts its arms once and a state
 * pruned before its terminal still counts the steps that led to it. */
static void execute(unsigned p, unsigned n) {
    wf_sched_statistics before;
    wf_sched_statistics after;
    wf_sched_statistics_sum(&wf_enum_core, &before);
    execute_step(p, n);
    wf_sched_statistics_sum(&wf_enum_core, &after);
    add_statistics(&cov.stats, &after, &before);
    cov.late_third_line = cov.stats.late_parks;
}

static void execute_step(unsigned p, unsigned n) {
    if (p < cfg_threads) {
        actor *a = &actors[p];
        op o = a->pending;
        trace[n].proc = p;
        trace[n].kind = o.kind;
        if (o.kind == OP_PARK && a->park_blocked) {
            cov.sleeps += 1u;
        }
        a->park_blocked = 0;
        controller_resume(a);
        if (is_write_kind(o.kind)) {
            note_write(p);
        }
    } else {
        unsigned k = p - cfg_threads;
        trace[n].proc = p;
        trace[n].kind = OP_COMPLETE;
        completion_serial += 1u;
        records[k].state = R_COMPLETED;
        records[k].completed_at = completion_serial;
        wake_all();
        note_write(ENUM_MAX_THREADS);
    }
}

static unsigned char *host_stacks[ENUM_MAX_THREADS];

static void reserve_host_stacks(void) {
    unsigned index;
    for (index = 0; index < ENUM_MAX_THREADS; index += 1u) {
        unsigned char *base = mmap(NULL, ENUM_HOST_STACK_BYTES + 4096u, PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (base == MAP_FAILED) {
            (void)fprintf(stderr, "enumerate: a host stack could not be reserved\n");
            exit(2);
        }
        (void)mprotect(base, 4096u, PROT_NONE);
        host_stacks[index] = base + 4096u;
    }
}

static void reset_execution(void) {
    unsigned index;
    failed = 0;
    failure[0] = 0;
    current = NULL;
    lock_holder = -1;
    record_count = 0;
    completion_serial = 0;
    depth = 0;
    memset(records, 0, sizeof records);
    memset(stacks, 0, sizeof stacks);
    for (index = 0; index < ENUM_MAX_STACKS; index += 1u) {
        stacks[index].parked_by = -1;
        stacks[index].cancel_kind = -1;
        stacks[index].claimed_by = -1;
    }
    if (wf_sched_init(&wf_enum_core, cfg_threads, cfg_stacks, ENUM_POOL_STACK_BYTES) != 0) {
        (void)fprintf(stderr, "enumerate: the core refused threads=%u stacks=%u\n", cfg_threads, cfg_stacks);
        exit(2);
    }
    if (wf_enum_core.stack_count != cfg_stacks) {
        (void)fprintf(stderr, "enumerate: the core raised the stack count to %u\n", wf_enum_core.stack_count);
        exit(2);
    }
    build_words();
    for (index = 0; index < cfg_threads; index += 1u) {
        actor *a = &actors[index];
        memset(a, 0, sizeof *a);
        a->index = index;
        a->state = index == 0u ? A_RUNNABLE : A_NEW;
        a->host_stack = host_stacks[index];
        a->sp = wf_switch_prepare(a->host_stack + ENUM_HOST_STACK_BYTES, actor_main, a);
        a->status = -1;
    }
    schedule->reset();
}

static unsigned replay_picks[ENUM_MAX_DEPTH];
static unsigned replay_count;

/* One execution: replays the trace's prefix, extends it with the first
 * enabled choice at every new state, and checks after every step. */
static void run_execution(void) {
    unsigned index;
    execution_serial += 1u;
    reset_execution();
    controller_resume(&actors[0]);
    if (failed) {
        return;
    }
    for (;;) {
        unsigned enabled;
        /* A worker the entry thread has created runs to its first step. */
        for (index = 1; index < cfg_threads; index += 1u) {
            if (actors[index].state == A_RUNNABLE && !actors[index].has_pending && actors[index].sp != NULL
                && stack_index_of(actors[index].sp) < 0 && !actors[index].taken_first) {
                controller_resume(&actors[index]);
                if (failed) {
                    return;
                }
            }
        }
        enabled = enabled_set();
        unsigned n = depth + 1u;
        unsigned p;
        if (enabled == 0u) {
            check_terminal();
            break;
        }
        if (n > ENUM_MAX_DEPTH) {
            fail_execution("harness: the trace is longer than %u", ENUM_MAX_DEPTH);
            break;
        }
        if (replay_count != 0u) {
            if (n <= replay_count) {
                p = replay_picks[n - 1u];
                if (((enabled >> p) & 1u) == 0u) {
                    fail_execution("harness: replay step %u names process %u, which is not enabled", n, p);
                    break;
                }
            } else {
                p = lowest_bit(enabled);
            }
            trace[n].pre_enabled = enabled;
            trace[n].backtrack = enabled;
            trace[n].done = 0;
            trace_len = n;
        } else if (n <= trace_len) {
            if (trace[n].pre_enabled != enabled) {
                fail_execution("harness: the execution is not deterministic at step %u (enabled %x, was %x)",
                    n, enabled, trace[n].pre_enabled);
                break;
            }
            p = trace[n].proc;
        } else {
            trace[n].pre_enabled = enabled;
            trace[n].done = 0;
            trace[n].backtrack = enabled;
            p = lowest_bit(enabled);
            trace[n].proc = p;
            trace_len = n;
        }
        trace[n].done |= 1u << p;
        if (cfg_verbose) {
            char name[16];
            (void)fprintf(stderr, "enumerate: step %u: %s %s\n", n, proc_name(p, name, sizeof name),
                p < cfg_threads ? op_names[actors[p].pending.kind] : "complete");
        }
        execute(p, n);
        depth = n;
        if (failed) {
            break;
        }
        check_state();
        if (failed) {
            break;
        }
        if (depth >= cfg_bound) {
            cov.bounded += 1u;
            break;
        }
    }
    cov.executions += 1u;
    if (depth > cov.max_steps) {
        cov.max_steps = depth;
    }
}

/* Moves the trace to the next unexplored choice; 0 when the sweep is done. */
static int advance_trace(void) {
    while (trace_len > 0u) {
        unsigned i = trace_len;
        unsigned remaining = trace[i].backtrack & ~trace[i].done;
        if (remaining != 0u) {
            unsigned p = lowest_bit(remaining);
            trace[i].proc = p;
            trace[i].done |= 1u << p;
            return 1;
        }
        trace_len -= 1u;
    }
    return 0;
}

/* -------------------------------------------- the explicit-state search */

/* Every byte the next steps depend on: the live part of every stack, the
 * core, the device, the checker's own bookkeeping, and the schedule's state.
 * A state is checkpointed at every scheduling point and hashed; a state
 * reached again by another order of independent steps is not explored
 * twice, which is what keeps two threads' idle polling from multiplying the
 * walk. The core's per-thread counters and each lane's steal seed are
 * restored but not hashed: neither changes what the core does next at the
 * swept thread counts, and both would make equal states look different. */

typedef struct region {
    unsigned char *base;
    size_t bytes;
} region;

#define ENUM_MAX_REGIONS (16u + ENUM_MAX_THREADS + ENUM_MAX_STACKS)

typedef struct digest {
    unsigned long long a;
    unsigned long long b;
} digest;

typedef struct snapshot {
    unsigned count;
    region regions[ENUM_MAX_REGIONS];
    unsigned char *data;
    size_t bytes;
    size_t capacity;
    digest digest;
} snapshot;

static unsigned collect_regions(region *out) {
    unsigned n = 0;
    unsigned index;
    out[n].base = (unsigned char *)actors;
    out[n++].bytes = sizeof actors[0] * cfg_threads;
    out[n].base = (unsigned char *)records;
    out[n++].bytes = sizeof records[0] * record_count;
    out[n].base = (unsigned char *)&record_count;
    out[n++].bytes = sizeof record_count;
    out[n].base = (unsigned char *)&completion_serial;
    out[n++].bytes = sizeof completion_serial;
    out[n].base = (unsigned char *)&lock_holder;
    out[n++].bytes = sizeof lock_holder;
    out[n].base = (unsigned char *)words;
    out[n++].bytes = sizeof words[0] * word_count;
    out[n].base = (unsigned char *)&word_count;
    out[n++].bytes = sizeof word_count;
    out[n].base = (unsigned char *)stacks;
    out[n++].bytes = sizeof stacks[0] * wf_enum_core.stack_count;
    /* The core up to the lanes in use, then its tail: the lanes beyond
     * `thread_count` are never touched. */
    out[n].base = (unsigned char *)&wf_enum_core;
    out[n++].bytes = offsetof(wf_sched_core, lanes) + sizeof(wf_sched_lane) * wf_enum_core.thread_count;
    out[n].base = (unsigned char *)&wf_enum_core.status_posted;
    out[n++].bytes = sizeof wf_enum_core - offsetof(wf_sched_core, status_posted);
    out[n].base = wf_enum_schedule_state;
    out[n++].bytes = wf_enum_schedule_state_bytes;
    for (index = 0; index < cfg_threads; index += 1u) {
        const actor *a = &actors[index];
        unsigned char *top = a->host_stack + ENUM_HOST_STACK_BYTES;
        unsigned char *base;
        if (a->state == A_DEAD) {
            continue;
        }
        base = stack_index_of(a->sp) < 0 ? a->sp : wf_enum_core.threads[index].host_sp;
        if (base == NULL || base < a->host_stack || base > top) {
            fail_execution("harness: thread %u's host stack pointer is out of its stack", index);
            return n;
        }
        out[n].base = base;
        out[n++].bytes = (size_t)(top - base);
    }
    for (index = 0; index < wf_enum_core.stack_count; index += 1u) {
        const wf_sched_stack *stack = wf_enum_core.stacks[index];
        unsigned char *base = stack->saved_sp;
        unsigned t;
        for (t = 0; t < cfg_threads; t += 1u) {
            if (actors[t].state != A_DEAD && stack_index_of(actors[t].sp) == (int)index) {
                base = actors[t].sp;
            }
        }
        if (base < stack->low || base > stack->high) {
            fail_execution("harness: stack %u's saved pointer is out of its range", index);
            return n;
        }
        out[n].base = base;
        out[n++].bytes = (size_t)(stack->high - base);
    }
    return n;
}

/* The live bytes of a stack include the sanitizer's own redzones inside
 * frames, so the copies and the digest are kept out of its sight; they read
 * and write only ranges the model owns. */
#if defined(__has_attribute)
#if __has_attribute(no_sanitize_address)
#define ENUM_NO_ASAN __attribute__((no_sanitize_address))
#endif
#endif
#if !defined(ENUM_NO_ASAN)
#define ENUM_NO_ASAN
#endif
#if defined(__SANITIZE_ADDRESS__)
#define ENUM_UNDER_ASAN 1
#elif defined(__has_feature)
#if __has_feature(address_sanitizer)
#define ENUM_UNDER_ASAN 1
#endif
#endif

ENUM_NO_ASAN static void copy_bytes(unsigned char *to, const unsigned char *from, size_t count) {
#if defined(ENUM_UNDER_ASAN)
    /* The library copy is intercepted and would check the stack's redzones. */
    size_t index;
    for (index = 0; index < count; index += 1u) {
        to[index] = from[index];
    }
#else
    memcpy(to, from, count);
#endif
}

ENUM_NO_ASAN static void digest_bytes(digest *d, const unsigned char *bytes, size_t count) {
    size_t index = 0;
    while (index + 8u <= count) {
        unsigned long long w;
        memcpy(&w, bytes + index, 8u);
        d->a = (d->a ^ w) * 0x9E3779B97F4A7C15ull;
        d->a ^= d->a >> 29;
        d->b = (d->b + w) * 0xC2B2AE3D27D4EB4Full;
        index += 8u;
    }
    while (index < count) {
        d->a = (d->a ^ bytes[index]) * 1099511628211ull;
        d->b = (d->b ^ bytes[index]) * 0x9E3779B97F4A7C15ull;
        d->b ^= d->b >> 29;
        index += 1u;
    }
}

/* The core's used part, less the words that do not decide its next step:
 * the per-thread counters, each lane's steal seed, a deque's dead cells, and
 * a free slot's frame and record. Residue in those would make equal states
 * hash apart. */
static void digest_core(digest *d, size_t bytes) {
    const unsigned char *base = (const unsigned char *)&wf_enum_core;
    size_t cursor = 0;
    unsigned index;
    (void)bytes;
    for (index = 0; index < wf_enum_core.thread_count; index += 1u) {
        size_t skip = offsetof(wf_sched_core, threads) + index * sizeof(wf_sched_thread)
            + offsetof(wf_sched_thread, counts);
        digest_bytes(d, base + cursor, skip - cursor);
        cursor = skip + sizeof(wf_sched_statistics);
    }
    digest_bytes(d, base + cursor, offsetof(wf_sched_core, lanes) - cursor);
    for (index = 0; index < wf_enum_core.thread_count; index += 1u) {
        const wf_sched_lane *lane = &wf_enum_core.lanes[index];
        unsigned long long position;
        unsigned free_mask = 0;
        unsigned head = lane->free_head;
        unsigned slot;
        digest_bytes(d, (const unsigned char *)&lane->top, sizeof lane->top);
        digest_bytes(d, (const unsigned char *)&lane->bottom, sizeof lane->bottom);
        /* The cells from `top` to `bottom` inclusive: the owner's pop lowers
         * `bottom` before it reads the cell it claimed, and a thief racing it
         * for the last entry reads that same cell, so the cell at `bottom` is
         * live for as long as either may still read it. */
        for (position = lane->top;
             (long long)(lane->bottom - position) >= 0 && position - lane->top <= WF_SCHED_LANE_SLOTS;
             position += 1u) {
            const wf_sched_slot *cell = lane->buffer[position & (WF_SCHED_LANE_SLOTS - 1u)];
            digest_bytes(d, (const unsigned char *)&cell, sizeof cell);
        }
        digest_bytes(d, (const unsigned char *)&lane->free_head, sizeof lane->free_head);
        while (head != WF_SCHED_NO_SLOT && head < WF_SCHED_LANE_SLOTS && !((free_mask >> head) & 1u)) {
            free_mask |= 1u << head;
            head = lane->slots[head].next_free;
        }
        for (slot = 0; slot < WF_SCHED_LANE_SLOTS; slot += 1u) {
            const wf_sched_slot *entry = &lane->slots[slot];
            if ((free_mask >> slot) & 1u) {
                digest_bytes(d, (const unsigned char *)&entry->next_free, sizeof entry->next_free);
            } else {
                digest_bytes(d, (const unsigned char *)entry, offsetof(wf_sched_slot, next_free));
                digest_bytes(d, entry->frame, sizeof entry->frame);
            }
        }
    }
    cursor = offsetof(wf_sched_core, status_posted);
    digest_bytes(d, base + cursor, sizeof wf_enum_core - cursor);
}

/* A pool stack's live bytes, less its saved stack pointer while a thread
 * executes on it: that word is the last switch away, dead until the next. */
static void digest_pool_stack(digest *d, const region *r, int executing) {
    const wf_sched_stack *header = (const wf_sched_stack *)(r->base + r->bytes - sizeof(wf_sched_stack));
    const unsigned char *saved = (const unsigned char *)&header->saved_sp;
    if (!executing || saved < r->base) {
        digest_bytes(d, r->base, r->bytes);
        return;
    }
    digest_bytes(d, r->base, (size_t)(saved - r->base));
    digest_bytes(d, saved + sizeof header->saved_sp, (size_t)(r->base + r->bytes - saved) - sizeof header->saved_sp);
}

/* Fills `snap` with the state and its digest, growing its buffer as needed;
 * one buffer per search depth is reused across the whole sweep. */
static void save_state(snapshot *snap) {
    unsigned index;
    unsigned char *cursor;
    digest d = {14695981039346656037ull, 0x243F6A8885A308D3ull};
    snap->count = collect_regions(snap->regions);
    snap->bytes = 0;
    for (index = 0; index < snap->count; index += 1u) {
        snap->bytes += snap->regions[index].bytes;
    }
    if (snap->capacity < snap->bytes) {
        free(snap->data);
        snap->capacity = snap->bytes + snap->bytes / 2u + 1024u;
        snap->data = malloc(snap->capacity);
        if (snap->data == NULL) {
            (void)fprintf(stderr, "enumerate: out of memory\n");
            exit(2);
        }
    }
    cursor = snap->data;
    for (index = 0; index < snap->count; index += 1u) {
        const region *r = &snap->regions[index];
        int pool = stack_index_of(r->base);
        copy_bytes(cursor, r->base, r->bytes);
        if (r->base == (unsigned char *)&wf_enum_core) {
            digest_core(&d, r->bytes);
        } else if (r->base == (unsigned char *)&wf_enum_core.status_posted) {
            /* Digested with the core's head above. */
        } else if (r->base == (unsigned char *)words) {
            /* The word table is checker bookkeeping whose order is an
             * accident of retirements and whose content the records and
             * stacks fix, so it is restored but not digested. */
        } else if (pool >= 0) {
            int executing = 0;
            unsigned t;
            for (t = 0; t < cfg_threads; t += 1u) {
                if (actors[t].state != A_DEAD && actors[t].sp == r->base) {
                    executing = 1;
                }
            }
            digest_pool_stack(&d, r, executing);
        } else {
            digest_bytes(&d, cursor, r->bytes);
        }
        d.a ^= (unsigned long long)(uintptr_t)r->base;
        d.a *= 1099511628211ull;
        cursor += r->bytes;
    }
    snap->digest = d;
}

static void restore_state(const snapshot *snap) {
    unsigned index;
    const unsigned char *cursor = snap->data;
    for (index = 0; index < snap->count; index += 1u) {
        copy_bytes(snap->regions[index].base, cursor, snap->regions[index].bytes);
        cursor += snap->regions[index].bytes;
    }
}


/* The states seen: open addressing over both halves of the digest. */
static digest *seen_table;
static size_t seen_capacity;
static size_t seen_count;

static int seen_insert_into(digest *table, size_t capacity, digest d) {
    size_t slot = (size_t)(d.a ^ (d.b << 1)) & (capacity - 1u);
    for (;;) {
        if (table[slot].a == 0ull && table[slot].b == 0ull) {
            table[slot] = d;
            return 0;
        }
        if (table[slot].a == d.a && table[slot].b == d.b) {
            return 1;
        }
        slot = (slot + 1u) & (capacity - 1u);
    }
}

/* Records `d`; answers 1 when it was already recorded. */
static int seen_insert(digest d) {
    if (d.a == 0ull && d.b == 0ull) {
        d.a = 1;
    }
    if (seen_table == NULL || seen_count * 2u >= seen_capacity) {
        size_t capacity = seen_table == NULL ? (1u << 20) : seen_capacity * 2u;
        digest *table = calloc(capacity, sizeof *table);
        size_t index;
        if (table == NULL) {
            (void)fprintf(stderr, "enumerate: out of memory for %zu states\n", capacity);
            exit(2);
        }
        for (index = 0; index < seen_capacity; index += 1u) {
            if (seen_table[index].a != 0ull || seen_table[index].b != 0ull) {
                (void)seen_insert_into(table, capacity, seen_table[index]);
            }
        }
        free(seen_table);
        seen_table = table;
        seen_capacity = capacity;
    }
    if (seen_insert_into(seen_table, seen_capacity, d)) {
        return 1;
    }
    seen_count += 1u;
    return 0;
}

static void seen_reset(void) {
    free(seen_table);
    seen_table = NULL;
    seen_capacity = 0;
    seen_count = 0;
}

typedef struct frame {
    snapshot snap;
    unsigned enabled;
    unsigned tried;
} frame;

static frame frames[ENUM_MAX_DEPTH + 2u];

static void count_terminal(void) {
    cov.executions += 1u;
    if (depth > cov.max_steps) {
        cov.max_steps = depth;
    }
}

/* Depth-first over every reachable state, each explored once. */
static void run_stateful(void) {
    unsigned index;
    seen_reset();
    execution_serial = 1;
    reset_execution();
    controller_resume(&actors[0]);
    if (failed) {
        return;
    }
    depth = 0;
    save_state(&frames[0].snap);
    for (;;) {
        unsigned enabled;
        unsigned remaining;
        unsigned p;
        digest d;
        /* Arriving at a new state. A worker the entry thread has created
         * runs to its first step first. */
        for (index = 1; index < cfg_threads; index += 1u) {
            if (actors[index].state == A_RUNNABLE && !actors[index].has_pending
                && stack_index_of(actors[index].sp) < 0 && !actors[index].taken_first) {
                controller_resume(&actors[index]);
                if (failed) {
                    return;
                }
            }
        }
        enabled = enabled_set();
        if (enabled == 0u) {
            check_terminal();
            if (failed) {
                return;
            }
            count_terminal();
            if (depth == 0u) {
                return;
            }
            depth -= 1u;
            restore_state(&frames[depth].snap);
        } else {
            /* The device's completion of a record touches only the device
             * and the wake flags, which a thread observes at a progress
             * pass, an epoch capture, a park, or a spin the completion
             * unblocks; it commutes with every other thread step. Every
             * interleaving is therefore equivalent to one in which each
             * completion sits immediately before such an observing step,
             * or at the end, so completions are explored only there. */
            {
                unsigned threads_enabled = enabled & ((1u << cfg_threads) - 1u);
                int observing = threads_enabled == 0u;
                for (index = 0; index < cfg_threads && !observing; index += 1u) {
                    const actor *a = &actors[index];
                    if (a->state != A_RUNNABLE || !a->has_pending) {
                        continue;
                    }
                    switch (a->pending.kind) {
                    case OP_PROGRESS:
                    case OP_EPOCH:
                    case OP_PARK:
                        observing = 1;
                        break;
                    case OP_YIELD:
                        observing = !a->others_wrote;
                        break;
                    default:
                        break;
                    }
                }
                if (!observing) {
                    enabled = threads_enabled;
                }
            }
            /* A step that touches no shared object, an unlock, a switch or a
             * yield's return, commutes with every step of every other
             * process and disables none of them, so taking it alone loses
             * no state that differs in anything the checks read. */
            for (index = 0; index < cfg_threads; index += 1u) {
                if (((enabled >> index) & 1u) && actors[index].has_pending) {
                    const op *o = &actors[index].pending;
                    if (o->kind == OP_UNLOCK || o->kind == OP_SWITCH || o->kind == OP_YIELD) {
                        enabled = 1u << index;
                        break;
                    }
                    /* An owner's load of its own deque's bottom: no other
                     * thread ever writes that word (I2), so nothing it can
                     * do, now or later, is dependent on the load. */
                    if (o->kind == OP_LOAD) {
                        const word *w = word_of(o->word);
                        if (w != NULL && w->cls == W_BOTTOM && w->index == index) {
                            enabled = 1u << index;
                            break;
                        }
                    }
                }
            }
            /* The arriving state was digested and saved on the way in;
             * only its choices are new. */
            frames[depth].enabled = enabled;
            frames[depth].tried = 0;
        }
    next_move:
        remaining = frames[depth].enabled & ~frames[depth].tried;
        if (remaining == 0u) {
            if (depth == 0u) {
                return;
            }
            depth -= 1u;
            restore_state(&frames[depth].snap);
            goto next_move;
        }
        p = lowest_bit(remaining);
        frames[depth].tried |= 1u << p;
        if (depth + 1u > ENUM_MAX_DEPTH) {
            fail_execution("harness: the path is longer than %u", ENUM_MAX_DEPTH);
            return;
        }
        if (cfg_verbose) {
            char name[16];
            (void)fprintf(stderr, "enumerate: depth %u: %s %s\n", depth + 1u, proc_name(p, name, sizeof name),
                p < cfg_threads ? op_names[actors[p].pending.kind] : "complete");
        }
        execute(p, depth + 1u);
        depth += 1u;
        if (failed) {
            return;
        }
        check_state();
        if (failed) {
            return;
        }
        if (depth >= cfg_bound) {
            cov.bounded += 1u;
            depth -= 1u;
            restore_state(&frames[depth].snap);
            goto next_move;
        }
        save_state(&frames[depth].snap);
        if (failed) {
            return;
        }
        d = frames[depth].snap.digest;
        if (seen_insert(d)) {
            states_pruned += 1u;
            depth -= 1u;
            restore_state(&frames[depth].snap);
            goto next_move;
        }
        states_seen += 1u;
        if ((states_seen & 0x3ffffull) == 0ull) {
            (void)fprintf(stderr, "enumerate: ... schedule=%s threads=%u stacks=%u states=%llu pruned=%llu depth=%u\n",
                schedule->name, cfg_threads, cfg_stacks, states_seen, states_pruned, depth);
        }
    }
}

static void print_coverage(FILE *out) {
    (void)fprintf(out,
        "enumerate: schedule=%s threads=%u stacks=%u search=%s executions=%llu bounded=%llu max_steps=%llu"
        " begin=%llu suspended=%llu notified=%llu ready_from_suspended=%llu ready_from_notified=%llu"
        " cancel_suspending_io=%llu cancel_suspending_compute=%llu cancel_notified_io=%llu"
        " cancel_notified_compute=%llu resume=%llu resume_foreign=%llu empty=%llu take=%llu"
        " start_after_park=%llu post_in_window=%llu post_while_asleep=%llu post_elsewhere=%llu"
        " post_by_worker=%llu late_third_line=%llu publish_io=%llu publish_compute=%llu"
        " parks=%llu cancels=%llu resumes=%llu steals=%llu inline_runs=%llu exhausted_io=%llu"
        " exhausted_compute=%llu late_parks=%llu line_one=%llu max_parked=%llu all_asleep=%llu"
        " sleeps=%llu states=%llu pruned=%llu\n",
        schedule->name, cfg_threads, cfg_stacks, cfg_search == SEARCH_STATE ? "state" : "dfs",
        cov.executions, cov.bounded,
        cov.max_steps, cov.begin, cov.suspended, cov.notified, cov.ready_from_suspended,
        cov.ready_from_notified, cov.cancel_suspending_io, cov.cancel_suspending_compute,
        cov.cancel_notified_io, cov.cancel_notified_compute, cov.resume, cov.resume_foreign, cov.empty,
        cov.take, cov.start_after_park, cov.post_in_window, cov.post_while_asleep, cov.post_elsewhere,
        cov.post_by_worker, cov.late_third_line, cov.publish_io, cov.publish_compute, cov.stats.parks,
        cov.stats.cancels, cov.stats.resumes, cov.stats.steals, cov.stats.inline_runs,
        cov.stats.exhausted_io_waits, cov.stats.exhausted_compute_waits, cov.stats.late_parks,
        cov.stats.line_one, cov.max_parked, cov.all_asleep, cov.sleeps, states_seen, states_pruned);
}

/* Every interleaving of one schedule at the configured T and S. Returns 0
 * when the sweep passed. */
static int sweep(const wf_enum_schedule *s) {
    const char *reason;
    schedule = s;
    memset(&cov, 0, sizeof cov);
    trace_len = 0;
    execution_serial = 0;
    states_seen = 0;
    states_pruned = 0;
    if (cfg_search == SEARCH_STATE && replay_count == 0u) {
        run_stateful();
        if (failed) {
            (void)fprintf(stderr, "enumerate: FAIL schedule=%s threads=%u stacks=%u: %s\n",
                s->name, cfg_threads, cfg_stacks, failure);
            print_state(stderr);
            print_trace(stderr);
            print_coverage(stdout);
            return 1;
        }
        print_coverage(stdout);
        if (cov.bounded != 0ull) {
            (void)fprintf(stderr, "enumerate: FAIL schedule=%s threads=%u stacks=%u: %llu paths hit the step bound %u\n",
                s->name, cfg_threads, cfg_stacks, cov.bounded, cfg_bound);
            return 1;
        }
        reason = s->covered(&cov);
        if (reason != NULL) {
            (void)fprintf(stderr, "enumerate: FAIL schedule=%s threads=%u stacks=%u: not covered: %s\n",
                s->name, cfg_threads, cfg_stacks, reason);
            return 1;
        }
        return 0;
    }
    do {
        run_execution();
        if (cfg_limit != 0ull && execution_serial >= cfg_limit) {
            (void)fprintf(stderr, "enumerate: stopped at the execution limit %llu (not a sweep)\n", cfg_limit);
            print_coverage(stdout);
            return 1;
        }
        if ((cov.executions & 0x3ffffull) == 0ull) {
            (void)fprintf(stderr, "enumerate: ... schedule=%s threads=%u stacks=%u executions=%llu max_steps=%llu\n",
                s->name, cfg_threads, cfg_stacks, cov.executions, cov.max_steps);
        }
        if (failed) {
            (void)fprintf(stderr, "enumerate: FAIL schedule=%s threads=%u stacks=%u: %s\n",
                s->name, cfg_threads, cfg_stacks, failure);
            print_state(stderr);
            print_trace(stderr);
            print_coverage(stdout);
            return 1;
        }
    } while (replay_count == 0u && advance_trace());
    print_coverage(stdout);
    if (cov.bounded != 0ull) {
        (void)fprintf(stderr, "enumerate: FAIL schedule=%s threads=%u stacks=%u: %llu executions hit the step bound %u\n",
            s->name, cfg_threads, cfg_stacks, cov.bounded, cfg_bound);
        return 1;
    }
    reason = s->covered(&cov);
    if (reason != NULL && replay_count == 0u) {
        (void)fprintf(stderr, "enumerate: FAIL schedule=%s threads=%u stacks=%u: not covered: %s\n",
            s->name, cfg_threads, cfg_stacks, reason);
        return 1;
    }
    return 0;
}

static void on_crash(int signal_number) {
    (void)fprintf(stderr, "enumerate: crashed with signal %d\n", signal_number);
    if (schedule != NULL) {
        print_state(stderr);
        print_trace(stderr);
    }
    _exit(3);
}

static void install_crash_handler(void) {
    static unsigned char alternate[64u * 1024u];
    stack_t alt;
    struct sigaction action;
    alt.ss_sp = alternate;
    alt.ss_size = sizeof alternate;
    alt.ss_flags = 0;
    (void)sigaltstack(&alt, NULL);
    memset(&action, 0, sizeof action);
    action.sa_handler = on_crash;
    action.sa_flags = SA_ONSTACK;
    (void)sigaction(SIGSEGV, &action, NULL);
    (void)sigaction(SIGBUS, &action, NULL);
    (void)sigaction(SIGILL, &action, NULL);
}

static void usage(void) {
    (void)fprintf(stderr,
        "usage: enumerate --threads T --stacks S [--schedule NAME] [--search state|dfs]"
        " [--bound N] [--limit N] [--verbose] [--replay 'p1 p2 ...']\n");
    exit(2);
}

int main(int argc, char **argv) {
    int index;
    unsigned s;
    int failures = 0;
    int matched = 0;
    for (index = 1; index < argc; index += 1) {
        const char *argument = argv[index];
        const char *value = index + 1 < argc ? argv[index + 1] : NULL;
        if (strcmp(argument, "--threads") == 0 && value != NULL) {
            cfg_threads = (unsigned)strtoul(value, NULL, 10);
            index += 1;
        } else if (strcmp(argument, "--stacks") == 0 && value != NULL) {
            cfg_stacks = (unsigned)strtoul(value, NULL, 10);
            index += 1;
        } else if (strcmp(argument, "--schedule") == 0 && value != NULL) {
            cfg_schedule = value;
            index += 1;
        } else if (strcmp(argument, "--search") == 0 && value != NULL) {
            if (strcmp(value, "dfs") == 0) {
                cfg_search = SEARCH_DFS;
            } else if (strcmp(value, "state") == 0) {
                cfg_search = SEARCH_STATE;
            } else {
                usage();
            }
            index += 1;
        } else if (strcmp(argument, "--bound") == 0 && value != NULL) {
            cfg_bound = (unsigned)strtoul(value, NULL, 10);
            index += 1;
        } else if (strcmp(argument, "--limit") == 0 && value != NULL) {
            cfg_limit = strtoull(value, NULL, 10);
            index += 1;
        } else if (strcmp(argument, "--verbose") == 0) {
            cfg_verbose = 1;
        } else if (strcmp(argument, "--replay") == 0 && value != NULL) {
            const char *cursor = value;
            replay_count = 0;
            while (*cursor != 0) {
                char *end;
                unsigned long pick;
                while (*cursor == ' ') {
                    cursor += 1;
                }
                if (*cursor == 0) {
                    break;
                }
                pick = strtoul(cursor, &end, 10);
                if (end == cursor || replay_count >= ENUM_MAX_DEPTH) {
                    usage();
                }
                replay_picks[replay_count] = (unsigned)pick;
                replay_count += 1u;
                cursor = end;
            }
            index += 1;
        } else {
            usage();
        }
    }
    if (cfg_threads == 0u || cfg_threads > ENUM_MAX_THREADS || cfg_stacks < cfg_threads + 1u
        || cfg_stacks > ENUM_MAX_STACKS) {
        usage();
    }
    if (cfg_bound > ENUM_MAX_DEPTH) {
        cfg_bound = ENUM_MAX_DEPTH;
    }
    install_crash_handler();
    reserve_host_stacks();
    for (s = 0; s < wf_enum_schedule_count; s += 1u) {
        const wf_enum_schedule *candidate = &wf_enum_schedules[s];
        if (cfg_schedule != NULL && strcmp(cfg_schedule, candidate->name) != 0) {
            continue;
        }
        if (cfg_threads < candidate->min_threads || cfg_stacks < candidate->min_stacks
            || (candidate->max_threads != 0u && cfg_threads > candidate->max_threads)
            || (candidate->max_stacks != 0u && cfg_stacks > candidate->max_stacks)) {
            continue;
        }
        matched += 1;
        if (sweep(candidate) != 0) {
            failures += 1;
        }
    }
    if (matched == 0) {
        (void)fprintf(stderr, "enumerate: no schedule admits threads=%u stacks=%u\n", cfg_threads, cfg_stacks);
        return 2;
    }
    if (failures != 0) {
        (void)printf("enumerate: FAIL threads=%u stacks=%u failures=%d\n", cfg_threads, cfg_stacks, failures);
        return 1;
    }
    (void)printf("enumerate: PASS threads=%u stacks=%u schedules=%d\n", cfg_threads, cfg_stacks, matched);
    return 0;
}
