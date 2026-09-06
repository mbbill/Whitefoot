/* The scheduler core as the process's runtime: the one core instance, its
 * startup, and the module ABI over it.
 *
 * This is the platform layer of `research/investigations/io-model/
 * PARK-ON-MISS.md` §7 on POSIX. The core (`core.c`) owns the deque, the stack
 * states, the ready queue and the park protocol and reaches shared state only
 * through `prim.h`; this unit supplies the four things §7 says a platform
 * supplies -- thread creation with a reserved host stack, the wait and wake
 * (through `prim_host.c`'s seam), the switch (likewise), and the stack
 * reservation made once at the core's entry -- and nothing else. It replaces
 * `par_runtime.c`, whose policy it keeps exactly.
 *
 * Nothing here allocates after start: the stacks are one reservation carved at
 * the core's entry, the lanes are the core's own storage, and the worker
 * threads are created once. */

#ifndef WHITEFOOT_SCHED_ENTRY_H
#define WHITEFOOT_SCHED_ENTRY_H

#include "core.h"

#if defined(__cplusplus)
extern "C" {
#endif

/* The one `wf_sched_core` of the process. Every unit that publishes a record,
 * joins one, or hands work out names this instance: there is one scheduler
 * for compute hand-outs and I/O completions (§1), so there is one core.
 *
 * Zero until `wf__sched_start` runs. A link whose entry never starts the core
 * -- the completion harness and the probes, which have no floor -- either
 * leaves it zero, in which case `wf__sched_current_stack` answers "no pool
 * stack" and every join waits in place, or initialises it itself with
 * `wf_sched_init` and enters it with `wf__sched_enter`. */
extern wf_sched_core wf__sched_core;

/* The core's entry, and the link-time fact `wf_floor.c` tests (§5).
 *
 * Starts the core -- the stack reservation, and the worker pool when this run
 * asked for one -- and runs `body(argc, argv)` on a pool stack whose bottom is
 * the scheduler loop, posting its status when it returns; `wf_sched_run` then
 * returns on the calling thread's own host stack, and the status is written
 * through `status`. Answers 1 when it ran the program and 0 when the core
 * could not start, in which case the caller runs the body itself.
 *
 * `wf_floor.c` carries the weak answer 0 -- "no core is linked" -- so the
 * linker, and nothing at run time, selects the entry's shape. The floor is
 * compiled as one translation unit with no include path to this header, so the
 * seam is this one function rather than the core's ABI, and the body arrives
 * as a pointer rather than as a symbol this unit would have to name: a link
 * that carries the core without an emitted module (the harness, the probes)
 * would otherwise carry an undefined `wf__main_body`. */
int wf__sched_entry_stack(
    int (*body)(int, char **),
    int argc,
    char **argv,
    int *status
);

/* Starts the core: the reservation of §5 and, when this run asked for a pool,
 * the worker threads. Returns 0 on success. Called once, by the entry above;
 * exposed so a test can start the same policy the shipped entry starts. */
int wf__sched_start(void);

/* Runs `body` exactly once for this `state` word, which must start at zero.
 *
 * The runtime's own once-control, over the core's atomics and its yield, so
 * that one implementation serves every platform where `pthread_once` and
 * `InitOnceExecuteOnce` would be two.  A caller that arrives while another
 * thread is inside the body yields until that thread publishes completion:
 * there is no deadline in it and no budget.  The fast path after the first
 * call is one acquire load. */
void wf__sched_once(unsigned *state, void (*body)(void));

/* The one rule every startup setting of this runtime follows, exported so that
 * every unit that reads one reads it the same way.
 *
 * Unset or empty means "this setting places no instruction", and each caller
 * reads its own default from that. A written value is an integer from 0 through
 * the setting's own ceiling and nothing else. Anything else is a configuration
 * error: it ends the run with one line on the diagnostic channel, nothing on
 * the output channel, and a nonzero status, rather than being repaired into a
 * different run from the one the caller asked for.
 *
 * Answers 1 and writes `*value` when the setting named a number, 0 when it is
 * unset or empty, and does not return otherwise. */
int wf__sched_setting(const char *name, unsigned long ceiling, unsigned long *value);

/* The ceiling of the completion runtime's own WF_IO_HELPERS setting, or 0 in a
 * link that has no completion runtime.
 *
 * The bridge owns the number, and the rule above says a bad setting ends the
 * run before the program body -- which the bridge's own initializer, running at
 * the body's first operation, is too late for. So the bridge answers this
 * weakly-declared question strongly and `wf__sched_start` checks the setting
 * where it checks every other one. */
unsigned long wf__sched_helper_ceiling(void);

/* Enters the core on the calling thread: the one place `wf_sched_run` is
 * called, and what makes "this thread is running the core's loop" a fact a
 * join can read. Thread 0 returns here with the posted status; a worker never
 * returns. */
int wf__sched_enter(unsigned thread, void (*entry)(void *), void *argument);

/* The pool stack the calling thread is on, or NULL.
 *
 * NULL on a thread the core does not run -- a target helper, a harness thread,
 * the entry thread of a link whose core never started -- which is the test
 * §2's third line needs before it can park: there is no stack to park.
 * `wf_sched_current_stack` alone cannot answer it, because a thread the core
 * never registered reads thread index 0 and would see the entry thread's own
 * stack. */
wf_sched_stack *wf__sched_current_stack(void);

/* How many worker threads this process started, 0 when it started none. */
unsigned wf__sched_pool_running(void);

/* Lanes granted since process start: hand-outs that ran on a thread other
 * than the one that offered them, which is the core's steal count.
 *
 * No Whitefoot construct can name it and no program reads it; it exists so
 * that a measurement, or a gate, can tell a pool that grants lanes from one
 * that silently never does. A hand-out the offering thread ran itself at its
 * own join overlapped with nothing, so it is not counted here. */
unsigned long wf__par_grants(void);

/* The core's summed counters as one line of text, when the run asked for
 * them with `WF_SCHED_REPORT` (0 or unset: no line; 1: the line), read at the
 * core's entry under the one settings rule. Answers 1 and writes the line
 * when it was asked for and fits, 0 otherwise. It is for an observer or a
 * gate that has to say what the threads did rather than only what they
 * granted; no program reads it. */
int wf__sched_report(char *buffer, size_t capacity);

/* ------------------------------------------- the emitted module's ABI */

/* Each is a thin function over the core, and the contract is the one
 * `par_runtime.c` stated: acquire takes one of this thread's own frame slots
 * or answers NULL, publish offers it, join returns once it has run, release
 * gives the slot back once the caller has read the result out. */
void *wf__par_acquire_lane(unsigned long bytes);
void wf__par_publish(void *frame, void (*run)(void *));
void wf__par_join(void *frame);
void wf__par_release(void *frame);

/* Whether this run requested enough workers for this module's overlap:
 * one for staged I/O, two for compute-only hand-outs. Answered from the
 * setting, once at bootstrap, before lazy worker creation. Zero opts out. */
int wf__par_pool_active(unsigned minimum_workers);

/* How many times a permitted counted loop's range may be halved [PAR-2]. */
unsigned long wf__par_split_budget(unsigned long span, unsigned long weight);

#if defined(__cplusplus)
}
#endif

#endif
