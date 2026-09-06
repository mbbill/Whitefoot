/* The scheduler core. See `core.h` for the model and
 * `research/investigations/io-model/PARK-ON-MISS.md` for the design each
 * function cites by section. Shared state is reached only through `prim.h`. */

#include "core.h"
#include "prim.h"

#include <string.h>

/* Each counter has one writer, but the optional exit observer can read while
 * workers are still alive. Relaxed atomic loads/stores make that snapshot
 * defined without an atomic read-modify-write on the owner's hot path.
 * Counters never select a core action; the enumerator restores them but
 * excludes them from its state digest and its shared protocol steps. */
static void wf_sched_count(unsigned long long *counter, unsigned amount) {
    unsigned long long value = __atomic_load_n(counter, __ATOMIC_RELAXED);
    __atomic_store_n(counter, value + amount, __ATOMIC_RELAXED);
}

/* --------------------------------------------------------------- threads */

wf_sched_thread *wf_sched_current_thread(wf_sched_core *core) {
    unsigned index = wf_prim_thread_index();
    if (index >= core->thread_count) {
        wf_prim_fail("a thread the core does not know");
    }
    return &core->threads[index];
}

wf_sched_stack *wf_sched_current_stack(wf_sched_core *core) {
    return wf_sched_current_thread(core)->stack;
}

/* ------------------------------------------------------------ the lists */

/* Both lists are under the one mutex (§5, §7.1 item 5). The mutex says who
 * may touch their words; when a stack may be offered is the switch's rule
 * below, and it is the reason every EMPTY push is made from the stack switched
 * to. */

static wf_sched_stack *wf_sched_pool_pop(wf_sched_core *core) {
    wf_sched_stack *stack;
    wf_prim_lock(WF_PRIM_SECTION_FREE_POP);
    stack = core->free_head;
    if (stack != NULL) {
        core->free_head = stack->next;
        stack->next = NULL;
    }
    wf_prim_unlock();
    return stack;
}

static void wf_sched_pool_push(wf_sched_core *core, wf_sched_stack *stack) {
    wf_prim_lock(WF_PRIM_SECTION_FREE_PUSH);
    stack->next = core->free_head;
    core->free_head = stack;
    wf_prim_unlock();
}

/* Links one READY stack at the tail. Only the empty-to-nonempty transition
 * bumps the epoch; a later sleeper's last look finds an existing stack (§6). */
static void wf_sched_ready_push(wf_sched_core *core, wf_sched_stack *stack) {
    int was_empty;
    wf_prim_lock(WF_PRIM_SECTION_READY_PUSH);
    stack->next = NULL;
    was_empty = core->ready_head == NULL;
    if (was_empty) {
        core->ready_head = stack;
    } else {
        core->ready_tail->next = stack;
    }
    core->ready_tail = stack;
    wf_prim_unlock();
    if (was_empty) {
        wf_prim_wake();
    }
}

static wf_sched_stack *wf_sched_ready_pop(wf_sched_core *core) {
    wf_sched_stack *stack;
    wf_prim_lock(WF_PRIM_SECTION_READY_POP);
    stack = core->ready_head;
    if (stack != NULL) {
        core->ready_head = stack->next;
        if (core->ready_head == NULL) {
            core->ready_tail = NULL;
        }
        stack->next = NULL;
    }
    wf_prim_unlock();
    return stack;
}

/* ----------------------------------------------------------- the switch */

static void wf_sched_scheduler_loop(void *argument);

/* What every thread does on the far side of any switch: the obligations the
 * switching thread left it. An EMPTY stack is pushed to the pool here, after
 * the switch and from the stack switched to (§5); a park is committed here,
 * on the target stack, never before the switch (§6 step 5). */
static void wf_sched_commit_park(wf_sched_core *core, wf_sched_stack *parked);

static void wf_sched_after_switch(wf_sched_core *core) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    wf_sched_stack *empty = thread->pending_empty;
    wf_sched_stack *parked = thread->pending_commit;
    thread->pending_empty = NULL;
    thread->pending_commit = NULL;
    if (empty != NULL) {
        wf_sched_pool_push(core, empty);
    }
    if (parked != NULL) {
        wf_sched_commit_park(core, parked);
    }
}

/* Switches the calling thread from the stack it is on to `target`, which is
 * either EMPTY (its loop continues where it switched away) or READY (its join
 * resumes). The obligations are on the thread record, which the target reads
 * first thing. */
static void wf_sched_switch_to(wf_sched_core *core, wf_sched_stack *target) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    wf_sched_stack *from = thread->stack;
    thread->stack = target;
    wf_prim_store_u(&target->phase, WF_SCHED_STACK_RUNNING, WF_PRIM_RELAXED);
    wf_prim_set_bounds(target->low, target->high);
    wf_prim_switch(&from->saved_sp, target->saved_sp);
    /* `from` was resumed, by this thread or another. Everything per thread is
     * re-read from here on. */
    wf_sched_after_switch(core);
}

/* ---------------------------------------------------------------- deque */

/* Chase-Lev, as `par_runtime.c` had it: the owner pushes and pops at the
 * newest end, thieves take from the oldest, and the buffer holds exactly as
 * many cells as the lane has slots so a push needs no fullness test (I4). */

_Static_assert(
    (WF_SCHED_LANE_SLOTS & (WF_SCHED_LANE_SLOTS - 1u)) == 0u,
    "the deque masks rather than divides, so the slot count is a power of two (I4)"
);

static void wf_sched_push(wf_sched_lane *lane, wf_sched_slot *slot) {
    unsigned long long bottom = wf_prim_load_q(&lane->bottom, WF_PRIM_RELAXED);
    lane->buffer[bottom & (WF_SCHED_LANE_SLOTS - 1u)] = slot;
    wf_prim_store_q(&lane->bottom, bottom + 1u, WF_PRIM_SEQ_CST);
}

static wf_sched_slot *wf_sched_pop(wf_sched_lane *lane) {
    unsigned long long bottom = wf_prim_load_q(&lane->bottom, WF_PRIM_RELAXED);
    unsigned long long top = wf_prim_load_q(&lane->top, WF_PRIM_RELAXED);
    wf_sched_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
    bottom -= 1u;
    wf_prim_store_q(&lane->bottom, bottom, WF_PRIM_SEQ_CST);
    top = wf_prim_load_q(&lane->top, WF_PRIM_SEQ_CST);
    if ((long long)(bottom - top) < 0) {
        wf_prim_store_q(&lane->bottom, bottom + 1u, WF_PRIM_RELAXED);
        return NULL;
    }
    slot = lane->buffer[bottom & (WF_SCHED_LANE_SLOTS - 1u)];
    if ((long long)(bottom - top) > 0) {
        return slot;
    }
    if (!wf_prim_cas_q(&lane->top, &top, top + 1u, WF_PRIM_SEQ_CST, WF_PRIM_RELAXED)) {
        slot = NULL;
    }
    wf_prim_store_q(&lane->bottom, bottom + 1u, WF_PRIM_RELAXED);
    return slot;
}

/* The newest entry, read without taking it: line two of the rule asks whether
 * the target is it before popping. */
static wf_sched_slot *wf_sched_newest(wf_sched_lane *lane) {
    unsigned long long bottom = wf_prim_load_q(&lane->bottom, WF_PRIM_RELAXED);
    unsigned long long top = wf_prim_load_q(&lane->top, WF_PRIM_RELAXED);
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
    return lane->buffer[(bottom - 1u) & (WF_SCHED_LANE_SLOTS - 1u)];
}

static wf_sched_slot *wf_sched_steal(wf_sched_core *core, wf_sched_lane *victim) {
    unsigned long long top = wf_prim_load_q(&victim->top, WF_PRIM_ACQUIRE);
    unsigned long long bottom = wf_prim_load_q(&victim->bottom, WF_PRIM_ACQUIRE);
    wf_sched_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
    slot = victim->buffer[top & (WF_SCHED_LANE_SLOTS - 1u)];
    if (!wf_prim_cas_q(&victim->top, &top, top + 1u, WF_PRIM_SEQ_CST, WF_PRIM_RELAXED)) {
        return NULL;
    }
    wf_sched_count(&wf_sched_current_thread(core)->counts.steals, 1u);
    return slot;
}

static wf_sched_slot *wf_sched_find(wf_sched_core *core, wf_sched_thread *thread) {
    unsigned count = core->thread_count;
    unsigned offset;
    unsigned step;
    if (count < 2u) {
        return NULL;
    }
    thread->lane->seed = thread->lane->seed * 6364136223846793005ull + 1442695040888963407ull;
    offset = (unsigned)((thread->lane->seed >> 33) % count);
    for (step = 0; step < count; step += 1u) {
        unsigned index = offset + step;
        wf_sched_lane *victim;
        wf_sched_slot *slot;
        if (index >= count) {
            index -= count;
        }
        victim = &core->lanes[index];
        if (victim == thread->lane) {
            continue;
        }
        slot = wf_sched_steal(core, victim);
        if (slot != NULL) {
            return slot;
        }
    }
    return NULL;
}

/* ------------------------------------------------------- the publisher */

/* Runs one hand-out and publishes it: the compute side of the one protocol. */
static void wf_sched_execute(wf_sched_core *core, wf_sched_slot *slot) {
    slot->run(slot->frame);
    wf_sched_complete(core, &slot->record);
}

void wf_sched_complete(wf_sched_core *core, wf_sched_record *record) {
    wf_sched_stack *waiter;
    unsigned phase;
    /* COMPLETING first, DONE last: a joiner returns only on DONE, and an I/O
     * record is a block of the joiner's frame, so the record must not be
     * touched after DONE is stored (the enumerator reached a publisher
     * reading the waiter of a frame that had already died, §11). */
    wf_prim_store_u(&record->state, WF_SCHED_COMPLETING, WF_PRIM_SEQ_CST);
    waiter = wf_prim_load_p((void *const *)&record->waiter, WF_PRIM_SEQ_CST);
    /* The registration is claimed before the waiter's phase is touched: a
     * publisher that merely loaded the pointer could act on it late, after
     * the parker had cancelled, run on, and parked again on another record,
     * and would then notify a stack for an event that has not happened. The
     * parker's cancel takes the registration back with the same
     * compare-exchange, so exactly one of the two owns this park's wake
     * (the enumerator reached the late publisher on S3, §11). */
    if (waiter != NULL
        && !wf_prim_cas_p((void **)&record->waiter, (void **)&waiter, NULL, WF_PRIM_SEQ_CST, WF_PRIM_SEQ_CST)) {
        waiter = NULL;
    }
    wf_prim_store_u(&record->state, WF_SCHED_DONE, WF_PRIM_SEQ_CST);
    if (waiter == NULL) {
        return;
    }
    if (waiter == WF_SCHED_WAITER_IN_PLACE) {
        /* A thread waiting in place has no stack to resume, only a park to
         * end: the DONE store bumps nothing by itself (the enumerator
         * reached the in-place waiter sleeping past another thread's
         * drain, §11). */
        wf_prim_wake();
        return;
    }
    /* Exactly one of the two transitions into READY runs for one park: the
     * SUSPENDING arm hands the enqueue to the commit, the SUSPENDED arm makes
     * it here (§6). The owner of the wake finds the stack in no other phase:
     * a cancel needs the registration this call has just taken. */
    phase = wf_prim_load_u(&waiter->phase, WF_PRIM_ACQUIRE);
    for (;;) {
        if (phase == WF_SCHED_STACK_SUSPENDING) {
            if (wf_prim_cas_u(
                    &waiter->phase, &phase, WF_SCHED_STACK_NOTIFIED,
                    WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE
                )) {
                return;
            }
            continue;
        }
        if (phase == WF_SCHED_STACK_SUSPENDED) {
            if (wf_prim_cas_u(
                    &waiter->phase, &phase, WF_SCHED_STACK_READY,
                    WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE
                )) {
                wf_sched_ready_push(core, waiter);
                return;
            }
            continue;
        }
        wf_prim_fail("a publisher found its waiter in a phase no park reaches");
    }
    (void)core;
}

/* ------------------------------------------------------------ the park */

/* Step 5's commit, run on the target stack by whichever thread switched
 * there (§6): SUSPENDING becomes SUSPENDED, or a NOTIFIED park becomes READY
 * and is enqueued here, exactly once. */
static void wf_sched_commit_park(wf_sched_core *core, wf_sched_stack *parked) {
    unsigned expected = WF_SCHED_STACK_SUSPENDING;
    if (wf_prim_cas_u(
            &parked->phase, &expected, WF_SCHED_STACK_SUSPENDED,
            WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE
        )) {
        return;
    }
    if (expected != WF_SCHED_STACK_NOTIFIED) {
        wf_prim_fail("a commit found its stack neither SUSPENDING nor NOTIFIED");
    }
    if (!wf_prim_cas_u(
            &parked->phase, &expected, WF_SCHED_STACK_READY,
            WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE
        )) {
        wf_prim_fail("a NOTIFIED stack changed phase under its commit");
    }
    wf_sched_ready_push(core, parked);
}

void wf_sched_checkpoint(wf_sched_core *core) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    wf_sched_stack *stack = thread->stack;
    wf_sched_stack *target;
    unsigned expected = WF_SCHED_STACK_RUNNING;
#if WF_SCHED_OBSERVE
    wf_sched_count(&thread->counts.checkpoints, 1u);
#endif
    (void)wf_prim_progress(core);
    target = wf_sched_ready_pop(core);
    if (target == NULL) {
        return;
    }
    /* The current stack owns its readiness: no completion record is involved.
     * NOTIFIED defers READY and the enqueue until after its registers and SP
     * have been saved, using the same far-side commit as an early I/O wake.
     * Publishing READY before the switch would let another worker run a stack
     * whose old owner still executes on it. */
    if (!wf_prim_cas_u(&stack->phase, &expected, WF_SCHED_STACK_NOTIFIED,
            WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE)) {
        wf_prim_fail("a checkpoint began on a stack that was not RUNNING");
    }
#if WF_SCHED_OBSERVE
    wf_sched_count(&thread->counts.checkpoint_switches, 1u);
#endif
    thread->pending_commit = stack;
    wf_sched_switch_to(core, target);
}

/* Parks the calling stack on `record` and switches to `target`, which the
 * caller has already taken from the ready list or the pool (§6's five steps).
 * Returns 1 when the stack parked and has now been resumed, 0 when step 3
 * found the record already DONE and the park was cancelled on this stack;
 * then `target` is given back and the caller continues here. */
static int wf_sched_park(
    wf_sched_core *core,
    wf_sched_record *record,
    wf_sched_stack *target,
    int target_was_ready
) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    wf_sched_stack *stack = thread->stack;
    unsigned expected = WF_SCHED_STACK_RUNNING;

#if WF_SCHED_OBSERVE
    stack->park_thread = thread->index;
#endif

    /* 1. mark SUSPENDING */
    if (!wf_prim_cas_u(
            &stack->phase, &expected, WF_SCHED_STACK_SUSPENDING,
            WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE
        )) {
        wf_prim_fail("a park began on a stack that was not RUNNING");
    }
    /* 2. register the wake */
    wf_prim_store_p((void **)&record->waiter, stack, WF_PRIM_SEQ_CST);
    /* 3. re-read the record's state. COMPLETING means the publisher is
     * between its claim and its DONE store, a few instructions away. */
    for (;;) {
        unsigned state = wf_prim_load_u(&record->state, WF_PRIM_SEQ_CST);
        if (state != WF_SCHED_COMPLETING) {
            expected = state;
            break;
        }
        wf_prim_yield();
    }
    if (expected == WF_SCHED_DONE) {
        /* 4. already satisfied: take the registration back, and if that
         * succeeds nobody will touch this stack's phase, so cancel here and
         * never switch. If the publisher claimed it first, it owns this
         * park's wake and will make the stack READY, so the park goes on to
         * step 5 and is resumed the ordinary way (§6). */
        void *registered = stack;
        if (wf_prim_cas_p(
                (void **)&record->waiter, &registered, NULL,
                WF_PRIM_SEQ_CST, WF_PRIM_SEQ_CST
            )) {
            expected = WF_SCHED_STACK_SUSPENDING;
            if (!wf_prim_cas_u(
                    &stack->phase, &expected, WF_SCHED_STACK_RUNNING,
                    WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE
                )) {
                wf_prim_fail("a cancel that owned its registration found its stack not SUSPENDING");
            }
            wf_sched_count(&thread->counts.cancels, 1u);
            if (target_was_ready) {
                wf_sched_ready_push(core, target);
            } else {
                wf_sched_pool_push(core, target);
            }
            return 0;
        }
    }
    /* 5. switch, then commit on the target stack */
    wf_sched_count(&thread->counts.parks, 1u);
    thread->pending_commit = stack;
    wf_sched_switch_to(core, target);
    /* Resumed: the registration is cleared on every park exit (§6). */
    wf_prim_store_p((void **)&record->waiter, NULL, WF_PRIM_RELAXED);
    thread = wf_sched_current_thread(core);
    wf_sched_count(&thread->counts.resumes, 1u);
#if WF_SCHED_OBSERVE
    wf_sched_count(&thread->counts.resume_migrations, stack->park_thread != thread->index);
#endif
    return 1;
}

/* Line three's precondition: a switch target, READY first, else free. */
static wf_sched_stack *wf_sched_take_target(wf_sched_core *core, int *was_ready) {
    wf_sched_stack *target = wf_sched_ready_pop(core);
    if (target != NULL) {
        *was_ready = 1;
        return target;
    }
    *was_ready = 0;
    return wf_sched_pool_pop(core);
}

/* ---------------------------------------------------------- the rule (§2) */

/* The looks one turn of the idle window makes, in the order it makes them:
 * the record this stack waits on, the ready list, the entry thread's exit
 * test, and -- for the scheduler loop's own turn -- this thread's own deque
 * and a steal. Returns 1 when a look found something and the turn acted on
 * it, which is what makes the caller's loop restart, and 0 when every look
 * missed. The turn's registration -- this thread's bit in `core->idle` and,
 * for an I/O record, the in-place waiter marker -- is raised by the caller
 * before the first of these looks and is ended here by every arm that acts,
 * because each of them either switches away or runs a hand-out here and none
 * may leave a thread published as idle while it does. */
static void wf_sched_idle_end(
    wf_sched_core *core,
    wf_sched_record *on_record,
    unsigned long long idle_bit
) {
    (void)wf_prim_and_q(&core->idle, ~idle_bit, WF_PRIM_SEQ_CST);
    if (on_record != NULL) {
        /* The in-place registration ends with the turn; a publisher that
         * claimed it already found the marker and woke. */
        void *marker = WF_SCHED_WAITER_IN_PLACE;
        (void)wf_prim_cas_p((void **)&on_record->waiter, &marker, NULL, WF_PRIM_SEQ_CST, WF_PRIM_SEQ_CST);
    }
}

static int wf_sched_idle_looks(
    wf_sched_core *core,
    wf_sched_record *on_record,
    int *left_for_status
) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    unsigned long long bit = 1ull << thread->index;
    wf_sched_stack *ready;
#if WF_SCHED_OBSERVE
    wf_sched_count(&thread->counts.idle_looks, 1u);
#endif
    if (on_record != NULL) {
        unsigned state = wf_prim_load_u(&on_record->state, WF_PRIM_ACQUIRE);
        if (state == WF_SCHED_DONE || state == WF_SCHED_COMPLETING) {
            wf_sched_idle_end(core, on_record, bit);
            return 1;
        }
    }
    ready = wf_sched_ready_pop(core);
    if (ready != NULL) {
        wf_sched_idle_end(core, on_record, bit);
        if (on_record != NULL) {
            /* The third line taken late: this stack parks with its own I/O as
             * the wake and switches to the stack that appeared (§2). The park
             * registers this stack on the record itself and re-reads the
             * record's state after it does, so the in-place registration the
             * line above has just ended is not one it needs. */
            wf_sched_count(&thread->counts.late_parks, 1u);
            (void)wf_sched_park(core, on_record, ready, 1);
            return 1;
        }
        thread->pending_empty = thread->stack;
        wf_prim_store_u(&thread->stack->phase, WF_SCHED_STACK_EMPTY, WF_PRIM_RELAXED);
        wf_sched_switch_to(core, ready);
        return 1;
    }
    if (on_record == NULL && thread->index == 0u
        && wf_prim_load_u(&core->status_posted, WF_PRIM_SEQ_CST) != 0u) {
        wf_sched_idle_end(core, on_record, bit);
        *left_for_status = 1;
        return 1;
    }
    if (on_record == NULL) {
        /* The deques, looked at after the bit went up (§6). */
        wf_sched_slot *slot = wf_sched_pop(thread->lane);
        if (slot == NULL) {
            slot = wf_sched_find(core, thread);
        }
        if (slot != NULL) {
            wf_sched_idle_end(core, on_record, bit);
            wf_sched_execute(core, slot);
            return 1;
        }
    }
    return 0;
}

/* Step 4 of the scheduler loop and the I/O arm of the fourth line are one
 * sequence (§6): raise this turn's registration, capture the epoch, flush and
 * progress, look, and park on the captured epoch -- with a bounded spin over
 * those same looks between the last of them and the park. `on_record`, when
 * given, is the record this stack would park on if a READY stack appears, and
 * the entry test is the entry thread's alone. Returns 1 when the loop must
 * restart.
 *
 * Where the spin sits is the whole of what it costs and most of what it buys.
 * It is after the drain, because the drain is the only thing that delivers an
 * I/O completion and a spin in front of it delays every completion by its own
 * length: at sixteen rounds that is 19 percent of the many-files workload and
 * at four thousand it is forty-one times it (the addendum in
 * `research/experiments/park-on-miss-measurements/README.md`). After the
 * drain, a turn that had something to find has already found it and never
 * reaches the spin at all.
 *
 * It is inside the capture-to-park window rather than in front of it, and that
 * is what keeps §6's lost-wake argument the argument it was: every look the
 * spin makes is a look after the epoch capture, so a publisher that acts after
 * the capture either moved the epoch, which makes the park below return at
 * once, or left a push one of these looks finds. What the spin is for, and
 * where its two constants come from, is `core.h`. */
static int wf_sched_idle_step(
    wf_sched_core *core,
    wf_sched_record *on_record,
    int *left_for_status
) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    unsigned long long bit = 1ull << thread->index;
    unsigned spin_rounds = WF_SCHED_IDLE_SPIN_ROUNDS;
    unsigned look_rounds = WF_SCHED_IDLE_SPIN_ROUNDS + WF_SCHED_IDLE_YIELD_ROUNDS;
    unsigned round;
    uint64_t epoch;
#if WF_SCHED_OBSERVE
    wf_sched_count(&thread->counts.idle_steps, 1u);
    wf_sched_count(&thread->counts.idle_progress, 1u);
#endif
    *left_for_status = 0;
    /* The bit goes up before the epoch is captured and before the last look,
     * so a publisher that misses the bit is one whose push the look finds.
     * The in-place waiter registers the same way, so a drain on another
     * thread wakes it instead of storing DONE past its sleep. */
    (void)wf_prim_or_q(&core->idle, bit, WF_PRIM_SEQ_CST);
    if (on_record != NULL) {
        wf_prim_store_p((void **)&on_record->waiter, WF_SCHED_WAITER_IN_PLACE, WF_PRIM_SEQ_CST);
    }
    epoch = wf_prim_epoch();
    if (wf_prim_progress(core)) {
        wf_sched_idle_end(core, on_record, bit);
        return 1;
    }
    if (wf_sched_idle_looks(core, on_record, left_for_status)) {
        return 1;
    }
    for (round = 0u; round < look_rounds; round += 1u) {
        if (round < spin_rounds) {
            wf_prim_pause();
        } else {
            wf_prim_yield();
        }
#if WF_SCHED_IDLE_PROGRESS_INTERVAL != 0
        if ((round + 1u) % WF_SCHED_IDLE_PROGRESS_INTERVAL == 0u) {
#if WF_SCHED_OBSERVE
            wf_sched_count(&thread->counts.idle_progress, 1u);
#endif
            if (wf_prim_progress(core)) {
                wf_sched_idle_end(core, on_record, bit);
                return 1;
            }
        }
#endif
        if (wf_sched_idle_looks(core, on_record, left_for_status)) {
            return 1;
        }
    }
#if WF_SCHED_OBSERVE
    wf_sched_count(&thread->counts.idle_waits, 1u);
#endif
    wf_prim_park(epoch);
    wf_sched_idle_end(core, on_record, bit);
    return 1;
}

void wf_sched_join(wf_sched_core *core, wf_sched_record *record, int is_io) {
    for (;;) {
        wf_sched_thread *thread = wf_sched_current_thread(core);
        wf_sched_stack *target;
        int was_ready;
        int left;
        /* line one; COMPLETING is DONE a few instructions from now */
        for (;;) {
            unsigned state = wf_prim_load_u(&record->state, WF_PRIM_ACQUIRE);
            if (state == WF_SCHED_DONE) {
                wf_sched_count(&thread->counts.line_one, 1u);
                return;
            }
            if (state != WF_SCHED_COMPLETING) {
                break;
            }
            wf_prim_yield();
        }
        /* line two, with the home test (§4): only this thread's own deque */
        if (!is_io) {
            wf_sched_slot *slot = (wf_sched_slot *)record;
            if (slot->home == thread->lane && wf_sched_newest(thread->lane) == slot) {
                wf_sched_slot *popped = wf_sched_pop(thread->lane);
                if (popped == slot) {
                    wf_sched_count(&thread->counts.inline_runs, 1u);
                    wf_sched_execute(core, slot);
                    return;
                }
                if (popped != NULL) {
                    wf_prim_fail("the newest entry changed under its owner");
                }
                /* A thief took it between the look and the pop: it is now
                 * someone else's to finish, so fall through to line three. */
            }
        }
        /* line three */
        target = wf_sched_take_target(core, &was_ready);
        if (target != NULL) {
            (void)wf_sched_park(core, record, target, was_ready);
            continue;
        }
        /* line four */
        if (is_io) {
            wf_sched_count(&thread->counts.exhausted_io_waits, 1u);
            (void)wf_sched_idle_step(core, record, &left);
            continue;
        }
        wf_sched_count(&thread->counts.exhausted_compute_waits, 1u);
        {
            wf_sched_slot *slot = wf_sched_pop(thread->lane);
            if (slot == NULL) {
                slot = wf_sched_find(core, thread);
            }
            if (slot != NULL) {
                wf_sched_execute(core, slot);
            } else if (!wf_prim_progress(core)) {
                /* The empty-handed turn drains before it pauses: the target
                 * may be held by a stack parked on I/O whose completion only
                 * a drain publishes, and with every other thread parked or
                 * spinning nobody else drains it (the enumerator reached the
                 * spin on S1 at one thread and three stacks, §11). */
                wf_prim_yield();
            }
        }
    }
}

/* ------------------------------------------------------ the scheduler loop */

static void wf_sched_scheduler_loop(void *argument) {
    wf_sched_core *core = argument;
    wf_sched_after_switch(core);
    for (;;) {
        wf_sched_thread *thread = wf_sched_current_thread(core);
        wf_sched_stack *ready;
        wf_sched_slot *slot;
        int left;
        if (thread->entry != NULL) {
            void (*entry)(void *) = thread->entry;
            void *entry_argument = thread->entry_argument;
            thread->entry = NULL;
            thread->entry_argument = NULL;
            entry(entry_argument);
            continue;
        }
        /* 1. a READY stack */
        ready = wf_sched_ready_pop(core);
        if (ready != NULL) {
            thread->pending_empty = thread->stack;
            wf_prim_store_u(&thread->stack->phase, WF_SCHED_STACK_EMPTY, WF_PRIM_RELAXED);
            wf_sched_switch_to(core, ready);
            continue;
        }
        /* 2. own deque newest */
        slot = wf_sched_pop(thread->lane);
        if (slot != NULL) {
            wf_sched_execute(core, slot);
            continue;
        }
        /* 3. a steal */
        slot = wf_sched_find(core, thread);
        if (slot != NULL) {
            wf_sched_execute(core, slot);
            continue;
        }
        /* 4. the idle window, then the park */
        (void)wf_sched_idle_step(core, NULL, &left);
        if (left) {
            /* The entry thread leaves for its host stack; the pool stack it is
             * on is EMPTY and is pushed from there, after the switch (§5). */
            thread = wf_sched_current_thread(core);
            thread->pending_empty = thread->stack;
            wf_prim_store_u(&thread->stack->phase, WF_SCHED_STACK_EMPTY, WF_PRIM_RELAXED);
            wf_prim_switch(&thread->stack->saved_sp, thread->host_sp);
            /* A thread that later takes this stack from the pool arrives
             * here: an EMPTY stack's loop continues where it switched away,
             * on whichever thread took it, with the obligations that thread
             * left it. The enumerator reached this with a worker whose start
             * followed the status post (§11 item 21). */
            wf_sched_after_switch(core);
            continue;
        }
    }
}

/* ---------------------------------------------------------------- entry */

int wf_sched_init(
    wf_sched_core *core,
    unsigned thread_count,
    unsigned stack_count,
    size_t stack_bytes
) {
    unsigned index;
    if (thread_count == 0u || thread_count > WF_SCHED_MAX_THREADS) {
        return 1;
    }
    if (stack_count < thread_count + 1u) {
        stack_count = thread_count + 1u;
    }
    if (stack_count > WF_SCHED_MAX_STACKS) {
        return 1;
    }
#if WF_SCHED_INIT_USED_LANES
    /* Unconfigured lanes are unreachable: every worker/lane lookup is bounded
     * by thread_count, and no hand-out can carry an unconfigured lane's home.
     * Keep the fixed ABI layout without faulting its unused pages into RAM.
     * Clear the trailing status/idle fields too, including on reinitialization. */
    memset(core, 0, offsetof(wf_sched_core, lanes));
    memset(core->lanes, 0, sizeof(core->lanes[0]) * thread_count);
    memset(&core->status_posted, 0, sizeof(*core) - offsetof(wf_sched_core, status_posted));
#else
    memset(core, 0, sizeof(*core));
#endif
    core->thread_count = thread_count;
    core->stack_count = stack_count;
    core->stack_bytes = stack_bytes;
    core->stack_stride = wf_prim_stack_stride(stack_bytes);
    core->reservation = wf_prim_reserve(stack_count, stack_bytes);
    if (core->reservation == NULL) {
        return 1;
    }
    for (index = 0; index < stack_count; index += 1u) {
        unsigned char *low = core->reservation + (size_t)index * core->stack_stride;
        unsigned char *high = low + core->stack_stride;
        wf_sched_stack *stack = (wf_sched_stack *)(high - sizeof(wf_sched_stack));
        unsigned char *top = (unsigned char *)((uintptr_t)stack & ~(uintptr_t)15);
        memset(stack, 0, sizeof(*stack));
        stack->phase = WF_SCHED_STACK_EMPTY;
        stack->low = low;
        stack->high = high;
        stack->index = index;
        stack->saved_sp = wf_prim_prepare_stack(
            top, core->stack_bytes, wf_sched_scheduler_loop, core
        );
        stack->next = core->free_head;
        core->free_head = stack;
        core->stacks[index] = stack;
    }
    for (index = 0; index < thread_count; index += 1u) {
        wf_sched_lane *lane = &core->lanes[index];
        unsigned slot;
        lane->top = 0;
        lane->bottom = 0;
        lane->seed = 0x9e3779b97f4a7c15ull * (unsigned long long)(index + 1u);
        for (slot = 0; slot < WF_SCHED_LANE_SLOTS; slot += 1u) {
            lane->slots[slot].home = lane;
            lane->slots[slot].record.state = WF_SCHED_DONE;
            lane->slots[slot].record.waiter = NULL;
            lane->slots[slot].next_free = slot + 1u;
        }
        lane->slots[WF_SCHED_LANE_SLOTS - 1u].next_free = WF_SCHED_NO_SLOT;
        lane->free_head = 0;
        core->threads[index].lane = lane;
        core->threads[index].index = index;
    }
    return 0;
}

int wf_sched_start_thread(wf_sched_core *core, unsigned index) {
    wf_sched_thread *thread;
    wf_sched_stack *stack;
    if (index == 0u || index >= core->thread_count) {
        return 1;
    }
    thread = &core->threads[index];
    if (thread->stack != NULL) {
        wf_prim_fail("a thread was started twice");
    }
    stack = wf_sched_pool_pop(core);
    if (stack == NULL) {
        return 1;
    }
    thread->stack = stack;
    return 0;
}

int wf_sched_run(wf_sched_core *core, unsigned index, void (*entry)(void *), void *argument) {
    wf_sched_thread *thread;
    wf_sched_stack *stack;
    if (index >= core->thread_count) {
        wf_prim_fail("a thread the core does not know started");
    }
    wf_prim_set_thread_index(index);
    thread = &core->threads[index];
    thread->entry = entry;
    thread->entry_argument = argument;
    if (index == 0u) {
        /* The entry thread takes its own stack at the core's entry, before
         * anything else can park (§5). */
        stack = wf_sched_pool_pop(core);
        if (stack == NULL) {
            wf_prim_fail("the entry thread found no stack to take");
        }
        thread->stack = stack;
    } else {
        /* A worker's stack was taken for it by `wf_sched_start_thread` on
         * the thread that created it: its own start may come arbitrarily
         * late, after the entry thread has parked every free stack (the
         * enumerator reached that on S1 at two threads, §11 item 21). */
        stack = thread->stack;
        if (stack == NULL) {
            wf_prim_fail("a worker started without a stack reserved by its creator");
        }
    }
    wf_prim_store_u(&stack->phase, WF_SCHED_STACK_RUNNING, WF_PRIM_RELAXED);
    wf_prim_set_bounds(stack->low, stack->high);
    wf_prim_switch(&thread->host_sp, stack->saved_sp);
    /* Only the entry thread returns here, on its host stack, with the status
     * posted and the pool stack it left owed to the pool. */
    thread = wf_sched_current_thread(core);
    if (thread->pending_empty != NULL) {
        wf_sched_pool_push(core, thread->pending_empty);
        thread->pending_empty = NULL;
    }
    return core->status;
}

void wf_sched_post_status(wf_sched_core *core, int status) {
    core->status = status;
    wf_prim_store_u(&core->status_posted, 1u, WF_PRIM_SEQ_CST);
    wf_prim_wake();
}

/* ------------------------------------------------------------ hand-outs */

void *wf_sched_acquire(wf_sched_core *core, unsigned long bytes) {
    wf_sched_thread *thread = wf_sched_current_thread(core);
    wf_sched_lane *lane = thread->lane;
    unsigned head;
    wf_sched_slot *slot;
    if (bytes > (unsigned long)WF_SCHED_FRAME_BYTES) {
        return NULL;
    }
    /* The pop of a single-consumer, multi-producer list: read the head with
     * acquire, read its successor, swing the head (I3, §7). */
    head = wf_prim_load_u(&lane->free_head, WF_PRIM_ACQUIRE);
    for (;;) {
        unsigned next;
        if (head == WF_SCHED_NO_SLOT) {
            return NULL;
        }
        next = lane->slots[head].next_free;
        if (wf_prim_cas_u(&lane->free_head, &head, next, WF_PRIM_ACQ_REL, WF_PRIM_ACQUIRE)) {
            break;
        }
    }
    slot = &lane->slots[head];
    slot->record.state = WF_SCHED_PENDING;
    slot->record.waiter = NULL;
    return slot->frame;
}

void wf_sched_publish(wf_sched_core *core, void *frame, void (*run)(void *)) {
    wf_sched_slot *slot = wf_sched_slot_of(frame);
    slot->run = run;
    wf_prim_store_u(&slot->record.state, WF_SCHED_PENDING, WF_PRIM_RELAXED);
    wf_sched_push(slot->home, slot);
    if (wf_prim_load_q(&core->idle, WF_PRIM_SEQ_CST) != 0ull) {
        wf_prim_wake();
    }
}

void wf_sched_join_frame(wf_sched_core *core, void *frame) {
    wf_sched_join(core, &wf_sched_slot_of(frame)->record, 0);
}

void wf_sched_release(wf_sched_core *core, void *frame) {
    wf_sched_slot *slot = wf_sched_slot_of(frame);
    wf_sched_lane *lane = slot->home;
    unsigned self = (unsigned)(slot - lane->slots);
    unsigned head = wf_prim_load_u(&lane->free_head, WF_PRIM_RELAXED);
    (void)core;
    for (;;) {
        slot->next_free = head;
        if (wf_prim_cas_u(&lane->free_head, &head, self, WF_PRIM_RELEASE, WF_PRIM_RELAXED)) {
            return;
        }
    }
}

void wf_sched_record_init(wf_sched_record *record) {
    record->state = WF_SCHED_PENDING;
    record->waiter = NULL;
}

void wf_sched_statistics_sum(const wf_sched_core *core, wf_sched_statistics *out) {
    unsigned index;
    memset(out, 0, sizeof(*out));
    for (index = 0; index < core->thread_count; index += 1u) {
        const wf_sched_statistics *counts = &core->threads[index].counts;
        out->parks += __atomic_load_n(&counts->parks, __ATOMIC_RELAXED);
        out->cancels += __atomic_load_n(&counts->cancels, __ATOMIC_RELAXED);
        out->resumes += __atomic_load_n(&counts->resumes, __ATOMIC_RELAXED);
        out->steals += __atomic_load_n(&counts->steals, __ATOMIC_RELAXED);
        out->inline_runs += __atomic_load_n(&counts->inline_runs, __ATOMIC_RELAXED);
        out->exhausted_io_waits += __atomic_load_n(&counts->exhausted_io_waits, __ATOMIC_RELAXED);
        out->exhausted_compute_waits += __atomic_load_n(&counts->exhausted_compute_waits, __ATOMIC_RELAXED);
        out->late_parks += __atomic_load_n(&counts->late_parks, __ATOMIC_RELAXED);
        out->line_one += __atomic_load_n(&counts->line_one, __ATOMIC_RELAXED);
        out->resume_migrations += __atomic_load_n(&counts->resume_migrations, __ATOMIC_RELAXED);
        out->idle_steps += __atomic_load_n(&counts->idle_steps, __ATOMIC_RELAXED);
        out->idle_looks += __atomic_load_n(&counts->idle_looks, __ATOMIC_RELAXED);
        out->idle_progress += __atomic_load_n(&counts->idle_progress, __ATOMIC_RELAXED);
        out->idle_waits += __atomic_load_n(&counts->idle_waits, __ATOMIC_RELAXED);
        out->checkpoints += __atomic_load_n(&counts->checkpoints, __ATOMIC_RELAXED);
        out->checkpoint_switches += __atomic_load_n(&counts->checkpoint_switches, __ATOMIC_RELAXED);
    }
}
