/* What a schedule of PARK-ON-MISS.md §10 is written against when it runs
 * under the enumerator (`enumerate.c`): the core's own ABI (`core.h`), plus
 * the few calls below that reach the enumerator's device model and its
 * checks. A schedule is written the way `smoke.c` drives the core, with the
 * device thread of that file replaced by the controlled device here.
 *
 * The enumerator runs every schedule at every configuration the schedule
 * admits, enumerates every interleaving of primitive steps, checks the §11
 * invariants after every step, and reports the arms each schedule reached so
 * the cargo wrapper can assert the ones §11 names. */

#ifndef WHITEFOOT_SCHED_ENUMERATE_H
#define WHITEFOOT_SCHED_ENUMERATE_H

#include "core.h"

/* The arms one sweep (one schedule at one configuration) reached, counted
 * over all of its executions, beside the core's own counters summed the same
 * way. Every field is named in the sweep's report line as `name=value`. */
typedef struct wf_enum_coverage {
    unsigned long long executions;
    unsigned long long bounded;
    unsigned long long max_steps;
    /* Stack phase transitions observed at the primitives (§5, §6). */
    unsigned long long begin;                 /* RUNNING -> SUSPENDING */
    unsigned long long suspended;             /* SUSPENDING -> SUSPENDED, at the commit */
    unsigned long long notified;              /* SUSPENDING -> NOTIFIED, by the publisher */
    unsigned long long cooperative;           /* RUNNING -> NOTIFIED, by the current stack */
    unsigned long long ready_from_suspended;  /* SUSPENDED -> READY, by the publisher */
    unsigned long long ready_from_notified;   /* NOTIFIED -> READY, at the commit */
    unsigned long long cancel_suspending_io;  /* SUSPENDING -> RUNNING on an I/O record */
    unsigned long long cancel_suspending_compute;
    unsigned long long cancel_notified_io;    /* NOTIFIED -> RUNNING, consuming the notification */
    unsigned long long cancel_notified_compute;
    unsigned long long resume;                /* READY -> RUNNING */
    unsigned long long resume_foreign;        /* ... by a thread other than the one that parked it */
    unsigned long long empty;                 /* RUNNING -> EMPTY */
    unsigned long long take;                  /* EMPTY -> RUNNING */
    unsigned long long start_after_park;      /* a thread's first take while a stack is parked */
    /* Where the status post landed relative to the entry thread's step 4. */
    unsigned long long post_in_window;        /* between the epoch capture and the park */
    unsigned long long post_while_asleep;     /* the entry thread asleep in the park */
    unsigned long long post_elsewhere;
    unsigned long long post_by_worker;
    /* The I/O arm's late third line: a READY stack found inside the arm. */
    unsigned long long late_third_line;
    /* Publications (DONE stores) that found a parked waiter, by kind. */
    unsigned long long publish_io;
    unsigned long long publish_compute;
    /* The greatest number of stacks parked at once in any state. */
    unsigned long long max_parked;
    /* States in which every thread was asleep while an operation was in
     * flight (S6's bar: one completion must then wake at least one). */
    unsigned long long all_asleep;
    /* Parks that slept: the epoch had not moved when the park was reached. */
    unsigned long long sleeps;
    /* The core's counters, summed over every explored step. */
    wf_sched_statistics stats;
} wf_enum_coverage;

/* One schedule. `main` is the entry body, run on the entry thread's first
 * pool stack; it ends by posting the status. `reset` runs before every
 * execution; `check` runs at the end of one execution and answers NULL when
 * the execution satisfied the schedule, else the reason; `covered` runs after
 * the whole sweep at one configuration and answers NULL when the arms the
 * schedule exists to reach were reached, else the reason. The bounds say
 * which configurations the schedule admits (0 for `max_*` means no bound). */
typedef struct wf_enum_schedule {
    const char *name;
    unsigned min_threads;
    unsigned max_threads;
    unsigned min_stacks;
    unsigned max_stacks;
    void (*reset)(void);
    void (*main)(void *argument);
    const char *(*check)(void);
    const char *(*covered)(const wf_enum_coverage *coverage);
} wf_enum_schedule;

/* The schedules, defined in `schedules.c`, and the one block of state they
 * keep between calls, which the enumerator checkpoints with the model. */
extern const wf_enum_schedule wf_enum_schedules[];
extern const unsigned wf_enum_schedule_count;
extern void *const wf_enum_schedule_state;
extern const size_t wf_enum_schedule_state_bytes;

/* The core instance of the current execution. */
extern wf_sched_core wf_enum_core;

/* The configuration of the current sweep. */
unsigned wf_enum_threads(void);
unsigned wf_enum_stacks(void);

/* The device accepts `record`: from here on the controlled device may
 * complete it at any step, and a later target-progress pass drains it. */
void wf_enum_submit(wf_sched_record *record);

/* The join of a submitted record, with the I1 bookkeeping, after which the
 * enumerator forgets the record: its frame may die. */
void wf_enum_join_io(wf_sched_record *record);

/* One I/O operation as the emitter will lower it: initialise the record in
 * this frame, submit, join. */
void wf_enum_io(wf_sched_record *record);

/* A compute hand-out of `payload_bytes` whose body is `run(payload)`. Returns
 * the payload, zeroed before publication, or NULL when the lane has no slot
 * (the caller then runs the call inline, as the emitter does). The hand-out
 * is joined and released through the two calls after it; the payload is not
 * written by the caller before the join, because a thief may be running the
 * body. */
void *wf_enum_hand_out(void (*run)(void *payload), unsigned long payload_bytes);
void wf_enum_join(void *payload);
void wf_enum_release(void *payload);

/* Ends the current execution as a failure with `reason`. */
void wf_enum_fail(const char *reason);

/* The number of stacks committed and waiting at this moment (SUSPENDED), for
 * a schedule that asserts a depth. */
unsigned wf_enum_parked_count(void);

#endif
