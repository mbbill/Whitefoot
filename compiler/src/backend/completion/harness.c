#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* The online-CPU query the helper budget is checked against is a BSD
 * extension the POSIX macro above hides on Darwin, exactly as in bridge.c. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
#include "native_contract.h"

#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <pthread.h>
#include <poll.h>
#include <sched.h>
#include <stdarg.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

/* The bridge's whole process-wide operation capacity. A test which means to
 * reach the capacity boundary must name the same number the bridge does. */
#define WF_HARNESS_OPERATION_CAPACITY 64u
/* The private storage the window query affords one loop before the compiler's
 * ceiling and the loop's own slot size apply.  A test which means to reach
 * that boundary must name the same number the bridge does. */
#define WF_HARNESS_WINDOW_BYTE_BUDGET (4u * 1024u * 1024u)

#define CHECK(condition)                                                      \
    do {                                                                      \
        if (!(condition)) {                                                   \
            fprintf(                                                          \
                stderr,                                                       \
                "completion harness: %s:%d: check failed: %s\n",            \
                __FILE__,                                                     \
                __LINE__,                                                     \
                #condition                                                    \
            );                                                                \
            return 1;                                                         \
        }                                                                     \
    } while (0)

#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
static unsigned wf_directory_host_calls;
static unsigned wf_directory_poll_calls;
static int wf_directory_poll_descriptor;
static short wf_directory_poll_events;
static int wf_directory_poll_timeout;

/* Deterministic enumeration answers for whichever family this build has:
 * interruption, readiness refusal, then one successful byte. The pipe
 * descriptor used by the test is already readable when the adapter performs
 * its exact POLLIN wait.
 *
 * One body serves both families; only the signature differs, because Darwin's
 * facility carries a base-position cell and Linux's keeps the whole cursor in
 * the descriptor. The facility this stands in for is selected by the same
 * macro the file adapter uses, so the adapter is exercised exactly as shipped
 * with only the host call replaced. */
static ssize_t wf_completion_test_directory_batch(void *buffer, size_t count) {
    wf_directory_host_calls += 1;
    if (wf_directory_host_calls == 1) {
        errno = EINTR;
        return -1;
    }
    if (wf_directory_host_calls == 2) {
        errno = EAGAIN;
        return -1;
    }
    if (buffer == NULL || count == 0) {
        errno = EINVAL;
        return -1;
    }
    ((unsigned char *)buffer)[0] = 'd';
    return 1;
}

#if defined(__APPLE__)
ssize_t wf_completion_test_getdirentries64(
    int descriptor,
    void *buffer,
    size_t count,
    int64_t *position
) {
    ssize_t reported;
    (void)descriptor;
    if (position == NULL) {
        wf_directory_host_calls += 1;
        errno = EINVAL;
        return -1;
    }
    reported = wf_completion_test_directory_batch(buffer, count);
    if (reported > 0) {
        *position = 17;
    }
    return reported;
}
#else
ssize_t wf_completion_test_getdents64(int descriptor, void *buffer, size_t count) {
    (void)descriptor;
    return wf_completion_test_directory_batch(buffer, count);
}
#endif
#endif

/* The adapter's own `openat`, observed.
 *
 * Retire and retry is decided by what that call answers, and the schedule
 * worth testing is the one where another engine is mid-operation at the exact
 * moment it answers `EMFILE`.  A test arms the gate, and the first refusal
 * after that announces itself, so the thread that gives the descriptor back
 * acts strictly after the refusal rather than racing it with a delay. */
static pthread_mutex_t wf_open_gate_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf_open_gate_signal = PTHREAD_COND_INITIALIZER;
/* Read without the lock by every refusal, so a build that is only observing
 * does not serialize the refusals it is observing.  A refused open decides
 * what it does next against a ledger several threads are moving, and a global
 * lock taken on the way out of every `openat` narrows that to one thread at a
 * time — which is exactly the schedule a test of the rule needs to leave
 * alone. */
static _Atomic unsigned wf_open_gate_armed;
static unsigned wf_open_gate_refusals;

int wf_completion_test_openat(
    int directory,
    const char *path,
    int flags,
    ...
) {
    int descriptor;
    if ((flags & O_CREAT) != 0) {
        va_list arguments;
        mode_t mode;
        va_start(arguments, flags);
        mode = (mode_t)va_arg(arguments, int);
        va_end(arguments);
        descriptor = openat(directory, path, flags, mode);
    } else {
        descriptor = openat(directory, path, flags);
    }
    if (descriptor < 0 && (errno == EMFILE || errno == ENFILE)
        && atomic_load_explicit(&wf_open_gate_armed, memory_order_acquire)
            != 0) {
        int refusal = errno;
        (void)pthread_mutex_lock(&wf_open_gate_lock);
        if (atomic_load_explicit(&wf_open_gate_armed, memory_order_relaxed)
            != 0) {
            wf_open_gate_refusals += 1;
            (void)pthread_cond_broadcast(&wf_open_gate_signal);
        }
        (void)pthread_mutex_unlock(&wf_open_gate_lock);
        errno = refusal;
    }
    return descriptor;
}

/* The readiness wait a parked transfer makes, announced.
 *
 * A test that means to stand where one engine is mid-operation while another
 * is refused an open needs the first operation to be genuinely in flight, not
 * merely submitted.  The bounded adapter reaches its readiness wait through
 * this named call, so arming it makes "the read is parked" a fact the test
 * observes instead of a delay it hopes for. */
static pthread_mutex_t wf_poll_gate_lock = PTHREAD_MUTEX_INITIALIZER;
static int wf_poll_gate_descriptor = -1;
static unsigned wf_poll_gate_entries;

/* The named point inside the ledger's give-up decision, between its read of
 * the retirement generation and its read of what is still in flight.  Armed,
 * it retires one operation exactly there, which is the one interleaving that
 * can hide a retirement from both reads. */
static pthread_mutex_t wf_retirement_point_lock = PTHREAD_MUTEX_INITIALIZER;
static unsigned wf_retirement_point_armed;
static unsigned wf_retirement_point_fired;

/* What a test standing at the point does there: retire one operation, or
 * queue one more behind the waiter's own thread. */
#define WF_HARNESS_POINT_RETIRES 0u
#define WF_HARNESS_POINT_QUEUES_OWED 1u
static unsigned wf_retirement_point_mode;
static _Atomic size_t wf_harness_owed_work;

/* The queue a scripted waiter is answerable for.  The ledger asks for it where
 * it decides, so a test can make it grow exactly there. */
static size_t wf_harness_owed(void *context) {
    (void)context;
    return atomic_load_explicit(&wf_harness_owed_work, memory_order_seq_cst);
}

void wf_completion_test_retirement_point(void) {
    unsigned fire = 0;
    unsigned mode = WF_HARNESS_POINT_RETIRES;
    (void)pthread_mutex_lock(&wf_retirement_point_lock);
    if (wf_retirement_point_armed != 0) {
        wf_retirement_point_armed = 0;
        wf_retirement_point_fired = 1;
        mode = wf_retirement_point_mode;
        fire = 1;
    }
    (void)pthread_mutex_unlock(&wf_retirement_point_lock);
    if (fire == 0) {
        return;
    }
    if (mode == WF_HARNESS_POINT_QUEUES_OWED) {
        atomic_fetch_add_explicit(
            &wf_harness_owed_work,
            1,
            memory_order_seq_cst
        );
        wf_completion_operation_accepted();
        return;
    }
    /* The retirement a give-up decision is about: one that put a descriptor
     * back, which is the only kind that grants the re-attempt. */
    wf_completion_operation_retired(1);
}

int wf_completion_test_poll(
    struct pollfd *descriptors,
    nfds_t count,
    int timeout
) {
    if (count == 1) {
        (void)pthread_mutex_lock(&wf_poll_gate_lock);
        if (wf_poll_gate_descriptor >= 0
            && descriptors[0].fd == wf_poll_gate_descriptor) {
            wf_poll_gate_entries += 1;
        }
        (void)pthread_mutex_unlock(&wf_poll_gate_lock);
    }
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    if (wf_directory_host_calls != 0) {
        wf_directory_poll_calls += 1;
        wf_directory_poll_descriptor = count == 1 ? descriptors[0].fd : -1;
        wf_directory_poll_events = count == 1 ? descriptors[0].events : 0;
        wf_directory_poll_timeout = timeout;
    }
#endif
    return poll(descriptors, count, timeout);
}

static wf_completion_publication integer_publication(const int *value) {
    wf_completion_publication publication = {
        .milestones = WF_COMPLETION_OWNERSHIP_COMPLETE,
        .terminal_kind = 1,
        .result = value,
        .result_size = sizeof(*value),
    };
    return publication;
}

static int accept_operation(
    wf_completion_runtime *runtime,
    wf_completion_token token
) {
    CHECK(
        wf_completion_begin_submit(runtime, token)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(
        wf_completion_target_accepted(runtime, token)
        == WF_COMPLETION_TRANSITIONED
    );
    return 0;
}

static int drain_exact(
    wf_completion_runtime *runtime,
    size_t expected,
    wf_completion_event *events
) {
    size_t total = 0;
    size_t rounds = 0;
    while (total < expected && rounds < runtime->slot_count + 2) {
        total += wf_completion_drain(
            runtime,
            events + total,
            expected - total,
            runtime->slot_count
        );
        rounds += 1;
    }
    CHECK(total == expected);
    return 0;
}

static int consume_integer(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    int expected
) {
    int result = 0;
    wf_completion_outcome outcome;
    CHECK(
        wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result == expected);
    CHECK(
        (outcome.milestones & WF_COMPLETION_OWNERSHIP_COMPLETE)
        == WF_COMPLETION_OWNERSHIP_COMPLETE
    );
    CHECK(outcome.terminal_kind == 1);
    CHECK(outcome.result_size == sizeof(result));
    return 0;
}

static int test_capacity_and_product_state(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_completion_token first;
    wf_completion_token second;
    wf_completion_token refused;
    wf_completion_event event;
    wf_completion_outcome outcome;
    wf_completion_publication publication;
    uint32_t milestones = 0;
    unsigned phase = 0;
    int value = 37;
    int result = 0;

    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(wf_completion_claim(&runtime, &first) == WF_COMPLETION_CLAIMED);
    CHECK(wf_completion_claim(&runtime, &second) == WF_COMPLETION_CLAIMED);
    CHECK(
        wf_completion_claim(&runtime, &refused)
        == WF_COMPLETION_CLAIM_WAIT_CAPACITY
    );

    CHECK(
        wf_completion_begin_submit(&runtime, first)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(
        wf_completion_mark_wait_capacity(&runtime, first)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(
        wf_completion_observe(&runtime, first, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(phase == WF_COMPLETION_WAIT_CAPACITY);
    CHECK(milestones == 0);
    CHECK(accept_operation(&runtime, first) == 0);

    publication = integer_publication(&value);
    publication.milestones = WF_COMPLETION_RESULT_READY
        | WF_COMPLETION_TERMINAL;
    CHECK(
        wf_completion_publish_terminal(&runtime, first, &publication)
        == WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL
    );
    publication = integer_publication(&value);
    publication.result_size = WF_COMPLETION_RESULT_CAPACITY + 1u;
    CHECK(
        wf_completion_publish_terminal(&runtime, first, &publication)
        == WF_COMPLETION_PUBLISH_RESULT_TOO_LARGE
    );
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, first, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(
        wf_completion_observe(&runtime, first, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK(phase == WF_COMPLETION_TERMINAL_PHASE);
    CHECK((milestones & WF_COMPLETION_RESULT_READY) != 0);
    CHECK((milestones & WF_COMPLETION_PAYLOAD_RELEASED) != 0);
    CHECK((milestones & WF_COMPLETION_RESOURCE_RELEASED) != 0);
    CHECK((milestones & WF_COMPLETION_TERMINAL) != 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            first,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUME_NOT_DRAINED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(event.token.slot == first.slot);
    CHECK(event.token.generation == first.generation);
    CHECK(consume_integer(&runtime, first, value) == 0);

    /* Release the second slot through the true inline/no-future-event route. */
    CHECK(
        wf_completion_begin_submit(&runtime, second)
        == WF_COMPLETION_TRANSITIONED
    );
    value = 41;
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_inline_terminal(&runtime, second, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, second, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_generation_and_duplicate_terminal(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token old_token;
    wf_completion_token new_token;
    wf_completion_event event;
    wf_completion_publication publication;
    int first = 111;
    int malicious = 999;
    int second = 222;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &old_token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, old_token) == 0);
    publication = integer_publication(&first);
    CHECK(
        wf_completion_publish_terminal(&runtime, old_token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    publication = integer_publication(&malicious);
    CHECK(
        wf_completion_publish_terminal(&runtime, old_token, &publication)
        == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, old_token, first) == 0);

    CHECK(wf_completion_claim(&runtime, &new_token) == WF_COMPLETION_CLAIMED);
    CHECK(new_token.slot == old_token.slot);
    CHECK(new_token.generation == old_token.generation + 1);
    CHECK(accept_operation(&runtime, new_token) == 0);

    /* The stale token is rejected while holding the slot lock and before the
     * malicious result can touch the new logical operation's result cell. */
    publication = integer_publication(&malicious);
    CHECK(
        wf_completion_publish_terminal(&runtime, old_token, &publication)
        == WF_COMPLETION_PUBLISH_STALE
    );
    publication = integer_publication(&second);
    CHECK(
        wf_completion_publish_terminal(&runtime, new_token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, new_token, second) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

/* A slot outlives the operation that used it, and only the generation says
 * which operation a token still stands for.  The named drain is the one
 * token-named entry that *takes* an event rather than reading one, so a
 * missing generation check there is not a stale read but a live operation's
 * completion consumed by an owner with no claim on it — and the live owner
 * then waits for an event that no longer exists. */
static int test_named_drain_refuses_a_recycled_slot(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token retired_token;
    wf_completion_token live_token;
    wf_completion_event event;
    wf_completion_outcome outcome;
    wf_completion_publication publication;
    int retired = 5;
    int live = 9;
    int taken = 0;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);

    /* One operation runs to completion, which frees the single slot. */
    CHECK(wf_completion_claim(&runtime, &retired_token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, retired_token) == 0);
    publication = integer_publication(&retired);
    CHECK(
        wf_completion_publish_terminal(&runtime, retired_token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(wf_completion_drain_token(&runtime, retired_token, &event) == 1);
    CHECK(event.token.slot == retired_token.slot);
    CHECK(event.token.generation == retired_token.generation);
    CHECK(consume_integer(&runtime, retired_token, retired) == 0);

    /* An unrelated operation reuses that slot and publishes its own result. */
    CHECK(wf_completion_claim(&runtime, &live_token) == WF_COMPLETION_CLAIMED);
    CHECK(live_token.slot == retired_token.slot);
    CHECK(live_token.generation == retired_token.generation + 1);
    CHECK(accept_operation(&runtime, live_token) == 0);
    publication = integer_publication(&live);
    CHECK(
        wf_completion_publish_terminal(&runtime, live_token, &publication)
        == WF_COMPLETION_PUBLISHED
    );

    /* The retired token names the slot but no longer names an operation, so
     * it takes nothing and the event stays where it belongs. */
    CHECK(wf_completion_drain_token(&runtime, retired_token, &event) == 0);
    CHECK(wf_completion_ready_event_count(&runtime) == 1);
    CHECK(
        wf_completion_consume(
            &runtime,
            retired_token,
            &taken,
            sizeof(taken),
            &outcome
        ) == WF_COMPLETION_CONSUME_STALE
    );

    /* The live owner still finds its own event, exactly once. */
    CHECK(wf_completion_drain_token(&runtime, live_token, &event) == 1);
    CHECK(event.token.generation == live_token.generation);
    CHECK(wf_completion_drain_token(&runtime, live_token, &event) == 0);
    CHECK(consume_integer(&runtime, live_token, live) == 0);
    CHECK(wf_completion_ready_event_count(&runtime) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

#define TERMINAL_RACERS 12

typedef struct terminal_race_context {
    wf_completion_runtime *runtime;
    wf_completion_token token;
    _Atomic unsigned *start;
    int value;
    enum wf_completion_publish_result result;
} terminal_race_context;

static void *terminal_racer(void *opaque) {
    terminal_race_context *context = opaque;
    wf_completion_publication publication;
    while (atomic_load_explicit(context->start, memory_order_acquire) == 0) {
        sched_yield();
    }
    publication = integer_publication(&context->value);
    context->result = wf_completion_publish_terminal(
        context->runtime,
        context->token,
        &publication
    );
    return NULL;
}

static int test_exactly_one_terminal_under_race(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_event event;
    pthread_t threads[TERMINAL_RACERS];
    terminal_race_context contexts[TERMINAL_RACERS];
    _Atomic unsigned start;
    size_t index;
    size_t winners = 0;
    size_t duplicates = 0;
    int winning_value = 0;

    atomic_init(&start, 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    for (index = 0; index < TERMINAL_RACERS; ++index) {
        contexts[index].runtime = &runtime;
        contexts[index].token = token;
        contexts[index].start = &start;
        contexts[index].value = (int)index + 1;
        contexts[index].result = WF_COMPLETION_PUBLISH_INVALID_STATE;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                terminal_racer,
                &contexts[index]
            ) == 0
        );
    }
    atomic_store_explicit(&start, 1, memory_order_release);
    for (index = 0; index < TERMINAL_RACERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        if (contexts[index].result == WF_COMPLETION_PUBLISHED) {
            winners += 1;
            winning_value = contexts[index].value;
        } else if (
            contexts[index].result
            == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL
        ) {
            duplicates += 1;
        }
    }
    CHECK(winners == 1);
    CHECK(duplicates == TERMINAL_RACERS - 1);
    CHECK(wf_completion_ready_event_count(&runtime) == 1);
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, token, winning_value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

#define CLAIM_RACERS 32

typedef struct claim_race_context {
    wf_completion_runtime *runtime;
    _Atomic unsigned *start;
    wf_completion_token token;
    enum wf_completion_claim_result result;
} claim_race_context;

static void *claim_racer(void *opaque) {
    claim_race_context *context = opaque;
    while (atomic_load_explicit(context->start, memory_order_acquire) == 0) {
        sched_yield();
    }
    context->result = wf_completion_claim(
        context->runtime,
        &context->token
    );
    return NULL;
}

static int test_concurrent_single_claims_are_unique(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[CLAIM_RACERS];
    wf_completion_event events[CLAIM_RACERS];
    pthread_t threads[CLAIM_RACERS];
    claim_race_context contexts[CLAIM_RACERS];
    unsigned char seen[CLAIM_RACERS] = {0};
    int values[CLAIM_RACERS];
    wf_completion_publication publication;
    wf_completion_token refused;
    _Atomic unsigned start;
    size_t index;

    atomic_init(&start, 0);
    CHECK(
        wf_completion_runtime_init(&runtime, slots, CLAIM_RACERS) == 0
    );
    for (index = 0; index < CLAIM_RACERS; ++index) {
        contexts[index].runtime = &runtime;
        contexts[index].start = &start;
        contexts[index].result = WF_COMPLETION_CLAIM_INVALID;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                claim_racer,
                &contexts[index]
            ) == 0
        );
    }
    atomic_store_explicit(&start, 1, memory_order_release);
    for (index = 0; index < CLAIM_RACERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(contexts[index].result == WF_COMPLETION_CLAIMED);
        CHECK(contexts[index].token.slot < CLAIM_RACERS);
        CHECK(contexts[index].token.generation == 1);
        CHECK(seen[contexts[index].token.slot] == 0);
        seen[contexts[index].token.slot] = 1;
    }
    CHECK(
        wf_completion_claim(&runtime, &refused)
        == WF_COMPLETION_CLAIM_WAIT_CAPACITY
    );

    for (index = 0; index < CLAIM_RACERS; ++index) {
        values[index] = (int)index;
        CHECK(accept_operation(&runtime, contexts[index].token) == 0);
        publication = integer_publication(&values[index]);
        CHECK(
            wf_completion_publish_terminal(
                &runtime,
                contexts[index].token,
                &publication
            ) == WF_COMPLETION_PUBLISHED
        );
    }
    CHECK(drain_exact(&runtime, CLAIM_RACERS, events) == 0);
    for (index = 0; index < CLAIM_RACERS; ++index) {
        CHECK(
            consume_integer(
                &runtime,
                contexts[index].token,
                values[index]
            ) == 0
        );
    }
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

typedef struct park_context {
    wf_completion_runtime *runtime;
    uint64_t epoch;
    enum wf_completion_park_result result;
} park_context;

static void *park_scheduler(void *opaque) {
    park_context *context = opaque;
    context->result = wf_completion_park_if_unchanged(
        context->runtime,
        context->epoch,
        5000
    );
    return NULL;
}

static int wait_until_parked_count(
    wf_completion_runtime *runtime,
    unsigned expected
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;
    for (attempts = 0; attempts < 5000; ++attempts) {
        if (wf_completion_parked_scheduler_count(runtime) == expected) {
            return 0;
        }
        (void)nanosleep(&delay, NULL);
    }
    return 1;
}

static int wait_until_parked(wf_completion_runtime *runtime) {
    return wait_until_parked_count(runtime, 1u);
}

static void record_host_wake(void *context) {
    _Atomic unsigned *count = context;
    atomic_fetch_add_explicit(count, 1, memory_order_relaxed);
}

static int test_unified_wake_epoch(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_event event;
    wf_completion_publication publication;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked;
    pthread_t thread;
    _Atomic unsigned host_wakes;
    uint64_t epoch_before;
    int value = 73;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            record_host_wake,
            &host_wakes
        ) == 0
    );
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    epoch_before = wf_completion_wake_epoch(&runtime);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.wake_signals == before.wake_signals);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 0);
    CHECK(wf_completion_wake_epoch(&runtime) == epoch_before + 1);
    CHECK(
        wf_completion_park_if_unchanged(&runtime, epoch_before, 1)
        == WF_COMPLETION_PARK_EPOCH_CHANGED
    );
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, token, value) == 0);

    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    wf_completion_notify_compute(&runtime);
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(after.compute_notifications == before.compute_notifications + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 1);

    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 2);
    CHECK(drain_exact(&runtime, 1, &event) == 0);
    CHECK(consume_integer(&runtime, token, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_linux_independent_operations_use_available_target(
    const char *scratch_directory
) {
#if defined(__linux__)
    wf_completion_token tokens[2];
    unsigned char bytes[2] = {0};
    int64_t value;
    int error_code;
    uint64_t native_before;
    uint64_t fallback_before;
    uint64_t native_after;
    uint64_t fallback_after;
    char path[256];
    int descriptor;

    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-native-operations-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, "xy", 2) == 2);
    native_before = wf__completion_linux_io_uring_submissions();
    fallback_before = wf__completion_file_fallback_submissions();

    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            &bytes[0],
            1,
            0,
            &tokens[0]
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            &bytes[1],
            1,
            1,
            &tokens[1]
        ) == 1
    );
    wf__completion_file_join(&tokens[0], &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    wf__completion_file_join(&tokens[1], &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(bytes[0] == 'x' && bytes[1] == 'y');

    native_after = wf__completion_linux_io_uring_submissions();
    fallback_after = wf__completion_file_fallback_submissions();
    if (native_after == native_before + 2) {
        CHECK(fallback_after == fallback_before);
        CHECK(wf__completion_target_helper_count() == 0);
    } else {
        CHECK(getenv("WF_REQUIRE_LINUX_IO_URING") == NULL);
        CHECK(native_after == native_before);
        CHECK(fallback_after == fallback_before + 2);
    }
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
#else
    (void)scratch_directory;
#endif
    return 0;
}

static int test_one_epoch_wakes_every_announced_scheduler(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked[2];
    pthread_t threads[2];
    _Atomic unsigned host_wakes;
    uint64_t epoch;
    size_t index;

    atomic_init(&host_wakes, 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(
        wf_completion_set_wake_callback(
            &runtime,
            record_host_wake,
            &host_wakes
        ) == 0
    );
    epoch = wf_completion_wake_epoch(&runtime);
    for (index = 0; index < 2; ++index) {
        parked[index].runtime = &runtime;
        parked[index].epoch = epoch;
        parked[index].result = WF_COMPLETION_PARK_FAILED;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                park_scheduler,
                &parked[index]
            ) == 0
        );
    }
    CHECK(wait_until_parked_count(&runtime, 2u) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    wf_completion_notify_compute(&runtime);
    for (index = 0; index < 2; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(parked[index].result == WF_COMPLETION_PARK_WOKEN);
    }
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(after.compute_notifications == before.compute_notifications + 1);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(atomic_load_explicit(&host_wakes, memory_order_relaxed) == 1);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int test_drain_wakes_the_registered_token_owner(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_publication publication;
    wf_completion_event event;
    wf_completion_statistics before;
    wf_completion_statistics after;
    park_context parked;
    pthread_t thread;
    uint64_t epoch;
    int value = 83;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    CHECK(accept_operation(&runtime, token) == 0);
    publication = integer_publication(&value);
    CHECK(
        wf_completion_publish_terminal(&runtime, token, &publication)
        == WF_COMPLETION_PUBLISHED
    );
    epoch = wf_completion_wake_epoch(&runtime);
    CHECK(
        wf_completion_wait_to_consume(&runtime, token)
        == WF_COMPLETION_CONSUME_WAIT_REGISTERED
    );
    parked.runtime = &runtime;
    parked.epoch = epoch;
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&thread, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    before = wf_completion_statistics_snapshot(&runtime);
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
    CHECK(pthread_join(thread, NULL) == 0);
    after = wf_completion_statistics_snapshot(&runtime);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(after.wake_signals == before.wake_signals + 1);
    CHECK(
        wf_completion_wait_to_consume(&runtime, token)
        == WF_COMPLETION_CONSUME_READY
    );
    CHECK(consume_integer(&runtime, token, value) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

typedef struct consume_context {
    wf_completion_runtime *runtime;
    wf_completion_token token;
    int expected;
    int failed;
} consume_context;

static void *consume_on_other_lane(void *opaque) {
    consume_context *context = opaque;
    context->failed = consume_integer(
        context->runtime,
        context->token,
        context->expected
    );
    return NULL;
}

static int test_bounded_drain_and_lane_independence(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_completion_token tokens[4];
    wf_completion_event events[4];
    int values[4] = {5, 7, 11, 13};
    size_t index;
    size_t drained;
    consume_context other;
    pthread_t consumer;

    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    for (index = 0; index < 4; ++index) {
        wf_completion_publication publication;
        CHECK(wf_completion_claim(&runtime, &tokens[index]) == WF_COMPLETION_CLAIMED);
        CHECK(accept_operation(&runtime, tokens[index]) == 0);
        publication = integer_publication(&values[index]);
        CHECK(
            wf_completion_publish_terminal(&runtime, tokens[index], &publication)
            == WF_COMPLETION_PUBLISHED
        );
    }
    drained = wf_completion_drain(&runtime, events, 2, 4);
    CHECK(drained <= 2);
    CHECK(drained == 2);
    CHECK(wf_completion_ready_event_count(&runtime) == 2);
    CHECK(drain_exact(&runtime, 2, events + 2) == 0);
    CHECK(wf_completion_ready_event_count(&runtime) == 0);

    other.runtime = &runtime;
    other.token = tokens[0];
    other.expected = values[0];
    other.failed = 1;
    CHECK(pthread_create(&consumer, NULL, consume_on_other_lane, &other) == 0);
    CHECK(pthread_join(consumer, NULL) == 0);
    CHECK(other.failed == 0);
    for (index = 1; index < 4; ++index) {
        CHECK(consume_integer(&runtime, tokens[index], values[index]) == 0);
    }
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

static int drain_and_consume_file(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    wf_file_result *result
) {
    wf_completion_event event;
    wf_completion_outcome outcome;
    CHECK(drain_exact(runtime, 1, &event) == 0);
    CHECK(event.token.slot == token.slot);
    CHECK(event.token.generation == token.generation);
    CHECK(
        wf_completion_consume(
            runtime,
            token,
            result,
            sizeof(*result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(outcome.result_size == sizeof(*result));
    CHECK(result->kind != 0);
    return 0;
}

static int test_single_thread_file_progress_and_backpressure(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token open_token;
    wf_completion_token capacity_token;
    wf_completion_token operation_token;
    wf_file_request request;
    wf_file_result result;
    char name[96];
    char read_back[32] = {0};
    const char payload[] = "completion-core";
    int root;
    int descriptor;

    CHECK(scratch_directory != NULL);
    root = open(scratch_directory, O_RDONLY | O_DIRECTORY);
    CHECK(root >= 0);
    (void)snprintf(
        name,
        sizeof(name),
        "wf-completion-harness-%ld",
        (long)getpid()
    );
    (void)unlinkat(root, name, 0);

    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0, 0) == 0
    );
    CHECK(wf_completion_claim(&runtime, &open_token) == WF_COMPLETION_CLAIMED);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = root;
    request.operation.open_at.path = name;
    request.operation.open_at.flags = O_CREAT | O_EXCL | O_RDWR;
    request.operation.open_at.mode = 0600;
    request.operation.open_at.has_mode = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, open_token, &request)
        == WF_FILE_TARGET_OWNS
    );

    CHECK(
        wf_completion_claim(&runtime, &capacity_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = -1;
    CHECK(
        wf_file_adapter_submit(&adapter, capacity_token, &request)
        == WF_FILE_WAIT_CAPACITY
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(
        wf_file_adapter_submit(&adapter, capacity_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);

    CHECK(drain_and_consume_file(&runtime, open_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value >= 0);
    descriptor = (int)result.value;
    CHECK(drain_and_consume_file(&runtime, capacity_token, &result) == 0);
    CHECK(result.value == -1);
    CHECK(result.error_code == EBADF);

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PWRITE;
    request.operation.pwrite.descriptor = descriptor;
    request.operation.pwrite.buffer = payload;
    request.operation.pwrite.count = sizeof(payload) - 1;
    request.operation.pwrite.offset = 0;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == (int64_t)(sizeof(payload) - 1));

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = read_back;
    request.operation.pread.count = sizeof(payload) - 1;
    request.operation.pread.offset = 0;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == (int64_t)(sizeof(payload) - 1));
    CHECK(memcmp(read_back, payload, sizeof(payload) - 1) == 0);

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_STATUS;
    request.operation.status.descriptor = descriptor;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 0);
    CHECK(result.status_size == sizeof(struct stat));

    CHECK(
        wf_completion_claim(&runtime, &operation_token)
        == WF_COMPLETION_CLAIMED
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = descriptor;
    CHECK(
        wf_file_adapter_submit(&adapter, operation_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, operation_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 0);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(unlinkat(root, name, 0) == 0);
    CHECK(close(root) == 0);
    return 0;
}

static int test_bridge_independent_positioned_reads(
    const char *scratch_directory
) {
    wf_completion_token first_token;
    wf_completion_token second_token;
    int64_t first_value = -1;
    int64_t second_value = -1;
    int first_error = -1;
    int second_error = -1;
    char first[5] = {0};
    char second[5] = {0};
    const char payload[] = "zeroONE-twoTHREE";
    char path[256];
    int descriptor;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-pread-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    descriptor = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(
        pwrite(descriptor, payload, sizeof(payload) - 1, 0)
        == (ssize_t)(sizeof(payload) - 1)
    );

    /* Both stable tokens are submitted before either result is joined.  The
     * two requests name one descriptor but independent file offsets; pread
     * leaves the shared cursor out of their semantics.
     *
     * This asserts the *submitted* route, so it depends on the bridge being
     * willing to take it.  Under a written WF_IO_HELPERS it always is.  Under
     * the demand-driven policy the bridge declines a positioned read once it
     * has measured this host's reads as not waiting, so a case like this one
     * belongs before any bridge operation has been executed — where it is —
     * and a reordering that moves it after one must pin the route instead. */
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            4,
            4,
            &first_token
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            second,
            4,
            11,
            &second_token
        ) == 1
    );
    wf__completion_file_join(
        &second_token,
        &second_value,
        &second_error
    );
    wf__completion_file_join(&first_token, &first_value, &first_error);
    CHECK(first_error == 0);
    CHECK(second_error == 0);
    CHECK(first_value == 4);
    CHECK(second_value == 4);
    CHECK(memcmp(first, "ONE-", 4) == 0);
    CHECK(memcmp(second, "THRE", 4) == 0);
    CHECK(wf__completion_file_submissions() >= 2);
#if defined(__linux__)
    if (getenv("WF_REQUIRE_LINUX_IO_URING") != NULL) {
        CHECK(wf__completion_linux_io_uring_submissions() >= 2);
    }
#endif

    /* A u64 which cannot be represented by the signed native offset never
     * enters the target queue and therefore owns no operation token. */
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            1,
            UINT64_MAX,
            &first_token
        ) == 0
    );
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_bridge_open_status_and_close_are_typed_operations(
    const char *scratch_directory
) {
    wf_completion_token token;
    unsigned char status[WF_FILE_STATUS_CAPACITY];
    uint64_t status_size = 0;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char path[256];
    int descriptor;
    uint64_t native_before = wf__completion_linux_io_uring_submissions();

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-typed-lifecycle-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);

    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_CREAT | O_EXCL | O_RDWR,
            0600u,
            1u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(
        &token,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    descriptor = (int)value;

    CHECK(wf__completion_file_status_submit(descriptor, &token) == 1);
    wf__completion_file_status_join(
        &token,
        &value,
        &error_code,
        status,
        sizeof(status),
        &status_size
    );
    CHECK(value == 0 && error_code == 0);
    CHECK(status_size == sizeof(struct stat));

    CHECK(wf__completion_file_close_submit(descriptor, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    /* Closing a descriptor whose authority this program already gave up is a
     * typed refusal, not a crash and not a silent success. */
    CHECK(wf__completion_file_close_submit(descriptor, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value < 0 && error_code == EBADF);

#if defined(__linux__)
    /* Where the ring is the qualified target, the open and both closes above
     * are ring operations. Without this the whole test would still pass on a
     * silent fallback to the bounded POSIX adapter, which is exactly the
     * regression the ring path exists to prevent. */
    if (getenv("WF_REQUIRE_LINUX_IO_URING") != NULL) {
        CHECK(wf__completion_linux_io_uring_submissions() >= native_before + 3);
    }
#endif
    (void)native_before;

    descriptor = (int)wf__completion_file_open_at_direct(
        AT_FDCWD,
        path,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(
        descriptor >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    CHECK(
        wf__completion_file_status_direct(
            descriptor,
            status,
            sizeof(status)
        ) == 0
    );
    CHECK(wf__completion_file_close_direct(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* One file of exactly one byte, so the byte a later read returns names which
 * file was opened. */
static int wf_harness_write_marker_file(int root, const char *name, char byte) {
    int descriptor = openat(root, name, O_CREAT | O_TRUNC | O_WRONLY, 0600);
    if (descriptor < 0) {
        return -1;
    }
    if (write(descriptor, &byte, 1) != 1) {
        (void)close(descriptor);
        return -1;
    }
    return close(descriptor);
}

/* A submitted open resolves the name it was given, not the bytes that were in
 * the caller's buffer when the host got round to it.
 *
 * The caller regains its name buffer the moment submission returns, so this
 * rewrites the buffer to name a *different* existing file immediately after
 * every submit.  The two names are the same length, so the rewrite is exactly
 * the overwrite a loop reusing one scratch buffer performs.  If the operation
 * record did not own its path bytes, the open would resolve the second name
 * and the marker byte would be wrong — or, on the ring, the kernel would read
 * a buffer the writer is concurrently rewriting.
 *
 * The private adapter leg also observes [SYS-2]'s `loan-released(path)` fact
 * while the operation is still outstanding, which is what makes the release
 * milestone a checked contract rather than a comment.  Both legs run on every
 * platform: the bridge leg reaches the io_uring ring on Linux and the bounded
 * POSIX adapter on Darwin, which are the two adapters that stage a path. */
static int test_submitted_open_owns_its_path_bytes(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token token;
    wf_file_request request;
    wf_file_result result;
    uint32_t milestones = 0;
    unsigned phase = 0;
    char first[64];
    char second[64];
    char requested[64];
    char first_directory[64];
    char second_directory[64];
    char marker = 0;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    int root;
    int descriptor;
    int marker_descriptor;
    uint64_t native_before = wf__completion_linux_io_uring_submissions();

    CHECK(scratch_directory != NULL);
    root = open(scratch_directory, O_RDONLY | O_DIRECTORY);
    CHECK(root >= 0);
    CHECK(
        snprintf(first, sizeof(first), "wf-open-name-a-%ld", (long)getpid())
        > 0
    );
    CHECK(
        snprintf(second, sizeof(second), "wf-open-name-b-%ld", (long)getpid())
        > 0
    );
    CHECK(strlen(first) == strlen(second));
    CHECK(wf_harness_write_marker_file(root, first, 'A') == 0);
    CHECK(wf_harness_write_marker_file(root, second, 'B') == 0);

    /* The bounded POSIX adapter, driven by this thread alone so the host call
     * demonstrably runs after the rewrite. */
    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0, 0) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    memcpy(requested, first, strlen(first) + 1u);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = root;
    request.operation.open_at.path = requested;
    request.operation.open_at.flags = O_RDONLY;
    request.operation.open_at.expected_kind = WF_FILE_EXPECT_REGULAR;
    CHECK(
        wf_file_adapter_submit(&adapter, token, &request)
        == WF_FILE_TARGET_OWNS
    );
    memcpy(requested, second, strlen(second) + 1u);
    CHECK(
        wf_completion_observe(&runtime, token, &milestones, &phase)
        == WF_COMPLETION_TRANSITIONED
    );
    CHECK((milestones & WF_COMPLETION_PAYLOAD_RELEASED) != 0);
    CHECK((milestones & WF_COMPLETION_TERMINAL) == 0);
    CHECK(phase == WF_COMPLETION_IN_FLIGHT);
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(drain_and_consume_file(&runtime, token, &result) == 0);
    CHECK(result.error_code == 0 && result.value >= 0);
    CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    descriptor = (int)result.value;
    CHECK(pread(descriptor, &marker, 1, 0) == 1);
    CHECK(marker == 'A');
    CHECK(close(descriptor) == 0);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);

    /* The shipped bridge, on whichever target it selects. */
    memcpy(requested, first, strlen(first) + 1u);
    CHECK(
        wf__completion_file_open_at_submit(
            root,
            requested,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    memcpy(requested, second, strlen(second) + 1u);
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    descriptor = (int)value;
    marker = 0;
    CHECK(wf__completion_file_pread_direct(descriptor, &marker, 1, 0) == 1);
    CHECK(marker == 'A');
    CHECK(wf__completion_file_close_direct(descriptor) == 0);

    /* `open_directory` stages its name through exactly the same request, so
     * it is checked the same way: the marker file exists only under the first
     * directory, and finding it proves which name was resolved. */
    CHECK(
        snprintf(
            first_directory,
            sizeof(first_directory),
            "wf-open-dir-a-%ld",
            (long)getpid()
        ) > 0
    );
    CHECK(
        snprintf(
            second_directory,
            sizeof(second_directory),
            "wf-open-dir-b-%ld",
            (long)getpid()
        ) > 0
    );
    CHECK(strlen(first_directory) == strlen(second_directory));
    (void)unlinkat(root, first_directory, AT_REMOVEDIR);
    (void)unlinkat(root, second_directory, AT_REMOVEDIR);
    CHECK(mkdirat(root, first_directory, 0700) == 0);
    CHECK(mkdirat(root, second_directory, 0700) == 0);
    marker_descriptor = openat(root, first_directory, O_RDONLY | O_DIRECTORY);
    CHECK(marker_descriptor >= 0);
    CHECK(wf_harness_write_marker_file(marker_descriptor, "marker", 'A') == 0);
    CHECK(close(marker_descriptor) == 0);

    memcpy(requested, first_directory, strlen(first_directory) + 1u);
    CHECK(
        wf__completion_file_open_at_submit(
            root,
            requested,
            O_RDONLY | O_DIRECTORY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            &token
        ) == 1
    );
    memcpy(requested, second_directory, strlen(second_directory) + 1u);
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(
        value >= 0 && error_code == 0
        && open_outcome == WF_FILE_OPEN_SUCCEEDED
    );
    descriptor = (int)value;
    marker_descriptor = openat(descriptor, "marker", O_RDONLY);
    CHECK(marker_descriptor >= 0);
    CHECK(close(marker_descriptor) == 0);
    CHECK(wf__completion_file_close_direct(descriptor) == 0);

#if defined(__linux__)
    /* Where the ring is the qualified target, both bridge opens above are ring
     * operations. Without this the leg would still pass on a silent fallback
     * to the bounded adapter, which is the other half of what this test is
     * about. */
    if (getenv("WF_REQUIRE_LINUX_IO_URING") != NULL) {
        CHECK(wf__completion_linux_io_uring_submissions() >= native_before + 2);
    }
#endif
    (void)native_before;

    marker_descriptor = openat(root, first_directory, O_RDONLY | O_DIRECTORY);
    CHECK(marker_descriptor >= 0);
    CHECK(unlinkat(marker_descriptor, "marker", 0) == 0);
    CHECK(close(marker_descriptor) == 0);
    CHECK(unlinkat(root, first, 0) == 0);
    CHECK(unlinkat(root, second, 0) == 0);
    CHECK(unlinkat(root, first_directory, AT_REMOVEDIR) == 0);
    CHECK(unlinkat(root, second_directory, AT_REMOVEDIR) == 0);
    CHECK(close(root) == 0);
    return 0;
}

#define WF_HARNESS_LONG_COMPONENTS 5u
#define WF_HARNESS_LONG_COMPONENT_BYTES 240u
#define WF_HARNESS_LONG_PATH_BYTES                                            \
    (WF_HARNESS_LONG_COMPONENTS * (WF_HARNESS_LONG_COMPONENT_BYTES + 1u) + 16u)

/* A name no operation record can hold takes the direct path, with the host's
 * own outcome and a counted demotion.
 *
 * `open_read` resolves the caller's whole path buffer and [PATH-1] admits a
 * relative path of any length, so a name longer than WF_FILE_PATH_CAPACITY is
 * one an ordinary program can write; only `open_file` and `open_directory`
 * are clamped to a single component.  The submission refuses such a name
 * before anything is claimed and the caller's direct open carries it, so the
 * outcome is exactly the host's own — the file opens where the host resolves
 * a name that long, and the same errno comes back where it does not.  What
 * the program loses is only the completion path, and that is a throughput
 * fact the runtime has to report: neither submission counter moves for an
 * operation that was never submitted, so without the demotion counter this
 * looks exactly like a program that never opened anything.
 *
 * Linux resolves the whole 1200-byte name (`PATH_MAX` is 4096) and the marker
 * file is really opened through the direct path.  Darwin's `PATH_MAX` is this
 * record's own bound, so the name it cannot hold is one its host refuses too;
 * the demotion still has to be counted and the direct path still has to
 * deliver the host's answer unchanged.  Both legs run wherever they apply. */
static int test_a_name_no_record_can_hold_opens_directly(
    const char *scratch_directory
) {
    char relative[WF_HARNESS_LONG_PATH_BYTES];
    wf_completion_token token;
    size_t length = 0;
    size_t built = 0;
    size_t level;
    uint64_t demoted_before;
    uint64_t submissions_before;
    int root;
    int descriptor;
    int host_error = 0;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char marker = 0;

    CHECK(scratch_directory != NULL);
    root = open(scratch_directory, O_RDONLY | O_DIRECTORY);
    CHECK(root >= 0);
    /* One component is at most NAME_MAX on every host this runs on, so the
     * length that exceeds the record comes from nesting rather than from one
     * outsized name.  A host that refuses the depth stops the tree there and
     * `built` records how much of it exists. */
    for (level = 0; level < WF_HARNESS_LONG_COMPONENTS; ++level) {
        if (level != 0) {
            relative[length] = '/';
            length += 1u;
        }
        memset(relative + length, 'd', WF_HARNESS_LONG_COMPONENT_BYTES);
        length += WF_HARNESS_LONG_COMPONENT_BYTES;
        relative[length] = 0;
        if (mkdirat(root, relative, 0700) == 0 || errno == EEXIST) {
            built = length;
            continue;
        }
        CHECK(errno == ENAMETOOLONG);
    }
    memcpy(relative + length, "/marker", sizeof("/marker"));
    length += sizeof("/marker") - 1u;
    CHECK(length > (size_t)WF_FILE_PATH_CAPACITY);
    descriptor = openat(root, relative, O_CREAT | O_TRUNC | O_WRONLY, 0600);
    if (descriptor < 0) {
        host_error = errno;
        CHECK(host_error == ENAMETOOLONG);
    } else {
        CHECK(write(descriptor, "L", 1) == 1);
        CHECK(close(descriptor) == 0);
    }

    demoted_before = wf__completion_file_demoted_opens();
    submissions_before = wf__completion_file_submissions();
    CHECK(
        wf__completion_file_open_at_submit(
            root,
            relative,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 0
    );
    CHECK(wf__completion_file_demoted_opens() == demoted_before + 1u);
    CHECK(wf__completion_file_submissions() == submissions_before);

    descriptor = wf__completion_file_open_at_direct(
        root,
        relative,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    if (host_error == 0) {
        CHECK(
            descriptor >= 0 && error_code == 0
            && open_outcome == WF_FILE_OPEN_SUCCEEDED
        );
        CHECK(read(descriptor, &marker, 1) == 1);
        CHECK(marker == 'L');
        CHECK(close(descriptor) == 0);
        CHECK(unlinkat(root, relative, 0) == 0);
    } else {
        CHECK(
            descriptor < 0 && error_code == host_error
            && open_outcome == WF_FILE_OPEN_FAILED
        );
    }

    /* Remove exactly the directories that were created, deepest first. */
    length = built;
    while (length != 0) {
        relative[length] = 0;
        CHECK(unlinkat(root, relative, AT_REMOVEDIR) == 0);
        while (length != 0 && relative[length - 1u] != '/') {
            length -= 1u;
        }
        if (length != 0) {
            length -= 1u;
        }
    }
    CHECK(close(root) == 0);
    return 0;
}

static int test_bridge_capacity_falls_back_per_operation(void) {
    wf_completion_token tokens[WF_HARNESS_OPERATION_CAPACITY];
    wf_completion_token refused;
    int64_t value = -1;
    int error_code = -1;
    size_t index;

    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        CHECK(
            wf__completion_file_pread_submit(
                -1,
                NULL,
                0,
                0,
                &tokens[index]
            ) == 1
        );
    }
    CHECK(
        wf__completion_file_pread_submit(-1, NULL, 0, 0, &refused)
        == 0
    );
    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        wf__completion_file_join(&tokens[index], &value, &error_code);
        CHECK(value == 0 && error_code == 0);
    }
    return 0;
}

static int test_checked_open_rejects_and_closes_nonregular_descriptors(
    const char *scratch_directory
) {
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char path[256];

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-nonregular-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    CHECK(mkfifo(path, 0600) == 0);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        path,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(
        &token,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        scratch_directory,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_IS_DIRECTORY);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    /* The same submitted refusal, so a target which carries opens natively
     * answers the same discriminator as one which does not. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            scratch_directory,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_IS_DIRECTORY);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    CHECK(unlink(path) == 0);
    return 0;
}

/* Every way an open can fail names its own terminal discriminator, on the
 * submitted path and on the direct one.  The generated program switches on
 * exactly this value and aborts on anything outside the set, so a target
 * which answers the wrong one is a fail-stop defect rather than a wrong
 * program. */
static int test_open_failure_classes_are_typed_outcomes(
    const char *scratch_directory
) {
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_SUCCEEDED;
    char missing[256];
    char regular[256];
    int descriptor;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            missing,
            sizeof(missing),
            "%s/wf-completion-absent-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    CHECK(
        snprintf(
            regular,
            sizeof(regular),
            "%s/wf-completion-regular-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(missing);
    (void)unlink(regular);
    descriptor = open(regular, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(close(descriptor) == 0);

    /* A name that does not resolve fails before any descriptor exists. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            missing,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value < 0);
    CHECK(error_code == ENOENT);
    CHECK(open_outcome == WF_FILE_OPEN_FAILED);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        missing,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value < 0);
    CHECK(error_code == ENOENT);
    CHECK(open_outcome == WF_FILE_OPEN_FAILED);

    /* A directory open of a regular file is refused by the same one rule
     * that refuses a regular open of a directory. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        regular,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_DIRECTORY,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    errno = 0;
    CHECK(fcntl((int)value, F_GETFD) == -1);
    CHECK(errno == EBADF);

    /* The directory the refusal above named really does open as a directory,
     * so the refusal is about the kind and not about the request shape. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            scratch_directory,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(unlink(regular) == 0);
    return 0;
}

/* Opens exhaust the same bounded operation capacity every other completion
 * operation draws on, refuse honestly at the boundary, and the refused
 * request succeeds once capacity returns. */
static int test_open_capacity_refuses_and_resubmits(
    const char *scratch_directory
) {
    wf_completion_token tokens[WF_HARNESS_OPERATION_CAPACITY];
    wf_completion_token refused;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    char path[256];
    size_t index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-open-capacity-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    {
        int seed = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
        CHECK(seed >= 0);
        CHECK(close(seed) == 0);
    }

    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        CHECK(
            wf__completion_file_open_at_submit(
                AT_FDCWD,
                path,
                O_RDONLY,
                0u,
                0u,
                WF_FILE_EXPECT_REGULAR,
                &tokens[index]
            ) == 1
        );
    }
    /* No operation is left to claim, so the bridge refuses before touching a
     * token and the generated program takes its qualified direct call. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &refused
        ) == 0
    );
    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        wf__completion_file_open_join(
            &tokens[index],
            &value,
            &error_code,
            &open_outcome
        );
        CHECK(value >= 0 && error_code == 0);
        CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
        CHECK(wf__completion_file_close_submit((int)value, &tokens[index]) == 1);
    }
    for (index = 0; index < WF_HARNESS_OPERATION_CAPACITY; ++index) {
        wf__completion_file_join(&tokens[index], &value, &error_code);
        CHECK(value == 0 && error_code == 0);
    }
    /* Released capacity readmits the request that was refused. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &refused
        ) == 1
    );
    wf__completion_file_open_join(&refused, &value, &error_code, &open_outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf__completion_file_close_submit((int)value, &refused) == 1);
    wf__completion_file_join(&refused, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(unlink(path) == 0);
    return 0;
}

typedef struct wf_open_waiter {
    const char *path;
    unsigned yields;
    int result;
} wf_open_waiter;

static void *wf_open_waiter_main(void *opaque) {
    wf_open_waiter *waiter = opaque;
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    unsigned yield;
    waiter->result = 1;
    if (wf__completion_file_open_at_submit(
            AT_FDCWD,
            waiter->path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) != 1) {
        return NULL;
    }
    /* Give the target every chance to reach terminal before this owner asks
     * for the result, so the join exercises the already-published path rather
     * than only the park-and-wake one. */
    for (yield = 0; yield < waiter->yields; ++yield) {
        (void)sched_yield();
    }
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    if (value < 0 || error_code != 0
        || open_outcome != WF_FILE_OPEN_SUCCEEDED) {
        return NULL;
    }
    if (wf__completion_file_close_submit((int)value, &token) != 1) {
        return NULL;
    }
    wf__completion_file_join(&token, &value, &error_code);
    waiter->result = value == 0 && error_code == 0 ? 0 : 1;
    return NULL;
}

/* Independent owners each hold their own open operation at once. No owner
 * observes another's result, and an owner whose operation finished before it
 * asked still consumes exactly its own. */
static int test_open_results_reach_every_independent_owner(
    const char *scratch_directory
) {
    enum { WF_OPEN_WAITERS = 6u };
    pthread_t threads[WF_OPEN_WAITERS];
    wf_open_waiter waiters[WF_OPEN_WAITERS];
    char path[256];
    unsigned index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-open-waiters-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    {
        int seed = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
        CHECK(seed >= 0);
        CHECK(close(seed) == 0);
    }
    for (index = 0; index < WF_OPEN_WAITERS; ++index) {
        waiters[index].path = path;
        /* Half join immediately and half only after yielding, so both the
         * already-terminal and the still-in-flight join are covered. */
        waiters[index].yields = index % 2u == 0u ? 0u : 64u;
        waiters[index].result = 1;
        CHECK(
            pthread_create(
                &threads[index],
                NULL,
                wf_open_waiter_main,
                &waiters[index]
            ) == 0
        );
    }
    for (index = 0; index < WF_OPEN_WAITERS; ++index) {
        CHECK(pthread_join(threads[index], NULL) == 0);
        CHECK(waiters[index].result == 0);
    }
    CHECK(unlink(path) == 0);
    return 0;
}

/* The helper policy makes two different promises and this pins both.
 *
 * A written WF_IO_HELPERS pins the count exactly, which is what every test
 * that names a helper configuration depends on. An unset one asks for
 * demand-driven growth, bounded by the process ceiling.
 *
 * The unset arm has now been weakened twice, and both times because the
 * policy it pinned changed under it. It first asserted exactly one helper,
 * which was the fixed default before growth existed. It then asserted at
 * least one and no more than the machine's CPU count, which was the growth
 * rule of batch 0086.
 *
 * Batch 0096 measured that rule and replaced both of its ends: the count
 * starts at none, because a program whose operations do not wait wants none,
 * and the ceiling is the bridge's operation bound rather than the core count,
 * because a helper inside a host call holds no CPU. What is left that this
 * process-wide case can honestly assert is the ceiling — how many helpers a
 * program *has* here depends on whether this harness's own temporary-file
 * operations happened to wait, which is a property of the machine. Its unset
 * arm no longer runs at all, because `main` pins the route for every
 * invocation that did not name one; the ceiling it asserts is the one thing
 * that holds either way.
 *
 * The rest of the promise did not go untested: it moved to
 * `test_pool_stays_empty_when_operations_do_not_wait` and
 * `test_pool_grows_when_operations_wait`, which script the clock the policy
 * measures with and so decide the question rather than sampling it. */
/* The scripted clock the helper policy measures host calls with.
 *
 * Unscripted it is the host's own monotonic clock, so every other case in this
 * file measures what the shipped build measures.  A case that is about what
 * the growth rule *decides* scripts it instead, and then the decision is a
 * property of the rule rather than of how loaded the machine running the test
 * happened to be. */
static _Atomic uint64_t wf_scripted_clock_step_ns;
static _Atomic uint64_t wf_scripted_clock_now_ns;

uint64_t wf_completion_test_monotonic_ns(void) {
    uint64_t step = atomic_load_explicit(
        &wf_scripted_clock_step_ns,
        memory_order_relaxed
    );
    if (step == 0) {
        struct timespec now;
        if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
            return 0;
        }
        return (uint64_t)now.tv_sec * 1000000000u + (uint64_t)now.tv_nsec;
    }
    /* Every reading advances by the scripted step, so the pair of readings
     * around one host call measures exactly that step. */
    return atomic_fetch_add_explicit(
        &wf_scripted_clock_now_ns,
        step,
        memory_order_relaxed
    ) + step;
}

static void wf_script_clock(uint64_t step_ns) {
    atomic_store_explicit(
        &wf_scripted_clock_step_ns,
        step_ns,
        memory_order_relaxed
    );
}

/* Counts every application of the WF_IO_NOCACHE target policy and then makes
 * the same host call the shipped build makes.  The harness build names this
 * function as WF_FILE_UNCACHED_APPLY, so the policy under test is the one in
 * file_adapter.h and only the counting is added. */
static _Atomic unsigned wf_uncached_applications;

void wf_completion_test_uncached_apply(int descriptor) {
    atomic_fetch_add_explicit(
        &wf_uncached_applications,
        1u,
        memory_order_relaxed
    );
    wf_file_uncached_apply_host(descriptor);
}

static unsigned wf_uncached_applications_now(void) {
    return atomic_load_explicit(
        &wf_uncached_applications,
        memory_order_relaxed
    );
}

/* The open flags an open really carried, with the bits that are a property of
 * the host rather than of the request masked away.  Both runs of this test
 * assert the same values, which is what makes the pair a statement that the
 * policy adds nothing to and removes nothing from an open. */
static int wf_uncached_open_flags(int descriptor) {
    int flags = fcntl(descriptor, F_GETFL);
    if (flags < 0) {
        return -1;
    }
    return flags & (O_ACCMODE | O_NONBLOCK | O_APPEND
#if defined(O_DIRECT)
                    | O_DIRECT
#endif
    );
}

/* WF_IO_NOCACHE is target policy, not a language surface: it changes how long
 * a read takes and nothing else.
 *
 * The same test runs twice from the Makefile, once with the setting absent
 * and once with it written.  Absent, the policy makes no host call at all and
 * the count does not move; written, it is applied exactly once for each
 * descriptor an open hands back and never for one a kind check refused.  Both
 * runs assert the identical open flags, the identical typed outcomes, and the
 * identical bytes, so what separates them is a cache hint and nothing else. */
static int test_uncached_reads_are_target_policy_only(
    const char *scratch_directory
) {
    const char *setting = getenv("WF_IO_NOCACHE");
    int asked = setting != NULL && setting[0] == '1' && setting[1] == 0;
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_SUCCEEDED;
    unsigned char content[8] = {'n', 'o', 'c', 'a', 'c', 'h', 'e', '!'};
    unsigned char window[8];
    char regular[256];
    char missing[256];
    unsigned before;
    unsigned after;
    int descriptor;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            regular,
            sizeof(regular),
            "%s/wf-completion-nocache-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    CHECK(
        snprintf(
            missing,
            sizeof(missing),
            "%s/wf-completion-nocache-absent-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(regular);
    (void)unlink(missing);
    descriptor = open(regular, O_CREAT | O_EXCL | O_RDWR, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, content, sizeof(content)) == (ssize_t)sizeof(content));
    CHECK(close(descriptor) == 0);

    before = wf_uncached_applications_now();

    /* One kind-checked open through the submitted path.  Its flags are the
     * request's O_RDONLY plus the O_NONBLOCK a kind check needs, and nothing
     * else; the bytes it reads are the bytes that were written. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf_uncached_open_flags((int)value) == (O_RDONLY | O_NONBLOCK));
    memset(window, 0, sizeof(window));
    CHECK(
        wf__completion_file_pread_submit(
            (int)value,
            window,
            (uint64_t)sizeof(window),
            0u,
            &token
        ) == 1
    );
    {
        int64_t moved = -1;
        int read_error = -1;
        wf__completion_file_join(&token, &moved, &read_error);
        CHECK(moved == (int64_t)sizeof(window));
        CHECK(read_error == 0);
        CHECK(memcmp(window, content, sizeof(content)) == 0);
    }
    CHECK(close((int)value) == 0);

    /* The same open through the direct path. */
    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        regular,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf_uncached_open_flags((int)value) == (O_RDONLY | O_NONBLOCK));
    memset(window, 0, sizeof(window));
    CHECK(
        wf__completion_file_pread_direct(
            (int)value,
            window,
            (uint64_t)sizeof(window),
            0
        ) == (int64_t)sizeof(window)
    );
    CHECK(memcmp(window, content, sizeof(content)) == 0);
    CHECK(close((int)value) == 0);

    /* An unchecked open carries exactly the request's own flags. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_ANY,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    CHECK(wf_uncached_open_flags((int)value) == O_RDONLY);
    CHECK(close((int)value) == 0);

    /* A refused open hands back no descriptor, so the policy has nothing to
     * apply to it. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            regular,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_DIRECTORY,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(open_outcome == WF_FILE_OPEN_OTHER_KIND);
    value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        missing,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &error_code,
        &open_outcome
    );
    CHECK(value < 0);
    CHECK(open_outcome == WF_FILE_OPEN_FAILED);

    after = wf_uncached_applications_now();
    CHECK(after - before == (asked ? 3u : 0u));

    CHECK(unlink(regular) == 0);
    return 0;
}

static int test_process_wide_target_helper_budget(void) {
    const char *text = getenv("WF_IO_HELPERS");
    uint64_t held = wf__completion_target_helper_count();
    unsigned long ceiling = 8;
    if (text != NULL && *text != '\0') {
        char *end = NULL;
        unsigned long pinned;
        errno = 0;
        pinned = strtoul(text, &end, 10);
        if (errno == 0 && end != text && *end == '\0') {
            if (pinned > ceiling) {
                pinned = ceiling;
            }
            CHECK(held == pinned);
            return 0;
        }
    }
    CHECK(held <= ceiling);
    return 0;
}

/* Submits `count` positioned reads of one open descriptor, one at a time, and
 * waits for each to complete.
 *
 * The wait is on the completion event rather than on the queue emptying: a
 * helper takes a request out of the queue before it executes it, so an empty
 * queue is not an answer, and waiting for one is a race this thread loses
 * whenever the pool has grown.  This thread also offers to execute, so the
 * case runs the same either way. */
static int drive_reads_to_completion(
    wf_completion_runtime *runtime,
    wf_file_adapter *adapter,
    int descriptor,
    unsigned count
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned index;
    for (index = 0; index < count; ++index) {
        wf_completion_token token;
        wf_file_request request;
        wf_file_result result;
        unsigned char byte = 0;
        unsigned attempts;
        CHECK(wf_completion_claim(runtime, &token) == WF_COMPLETION_CLAIMED);
        memset(&request, 0, sizeof(request));
        request.kind = WF_FILE_PREAD;
        request.operation.pread.descriptor = descriptor;
        request.operation.pread.buffer = &byte;
        request.operation.pread.count = 1;
        request.operation.pread.offset = 0;
        CHECK(
            wf_file_adapter_submit(adapter, token, &request)
            == WF_FILE_TARGET_OWNS
        );
        for (attempts = 0; attempts < 5000; ++attempts) {
            if (wf_completion_ready_event_count(runtime) != 0) {
                break;
            }
            if (wf_file_adapter_progress(adapter, 1) == 0) {
                (void)nanosleep(&delay, NULL);
            }
        }
        CHECK(wf_completion_ready_event_count(runtime) != 0);
        CHECK(drain_and_consume_file(runtime, token, &result) == 0);
        CHECK(result.error_code == 0);
    }
    return 0;
}

/* A program whose operations do not wait gets no helper, however much width it
 * states.
 *
 * This is the half of the growth rule batch 0096 added, and it is the half
 * that decides what the warm path costs: queue depth says a program stated
 * independent work, not that the work waits on anything, and a helper handed
 * an operation the host has already answered adds a queue crossing and two
 * thread wakes to nothing.  The clock is scripted so the case is about the
 * rule rather than about this machine's page cache. */
static int test_pool_stays_empty_when_operations_do_not_wait(
    const char *directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[8];
    wf_file_adapter adapter;
    wf_file_work queue[8];
    pthread_t helpers[4];
    char path[512];
    int descriptor;

    CHECK(directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-pool-no-wait-%ld",
            directory,
            (long)getpid()
        ) > 0
    );
    descriptor = open(path, O_RDWR | O_CREAT | O_TRUNC, 0600);
    CHECK(descriptor >= 0);
    CHECK(write(descriptor, "wf", 2) == 2);

    CHECK(wf_completion_runtime_init(&runtime, slots, 8) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 8, helpers, 4, 0) == 0);
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 4) == 0);

    /* Nothing measured yet is not evidence of anything, and neither policy
     * fires on it: the first operations take the completion path unchanged. */
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_UNMEASURED);
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 0);

    /* One microsecond a call: real work, and an order of magnitude under the
     * wait that would make a second thread worth its handoff. */
    wf_script_clock(1000u);
    CHECK(drive_reads_to_completion(&runtime, &adapter, descriptor, 32) == 0);
    wf_script_clock(0);
    CHECK(wf_file_adapter_helper_count(&adapter) == 0);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_SHORT);
    /* With no helper, nothing queued and no wait measured, a transfer
     * submitted now would be executed by the submitting thread itself. */
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 1);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptor) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* How many requests the two growth cases queue behind a pool that cannot
 * empty it.
 *
 * Growth adds at most one helper per submission and only while the queue is
 * deeper than the pool, so a pool of `n` needs a submission numbered past
 * `2n` to reach `n + 1` in the worst interleaving, where every helper takes
 * one entry the instant it is created.  Twenty carries a pool of eight, which
 * is the largest ceiling either case below asks for. */
#define WF_POOL_GROWTH_DEPTH 20u

/* Drives the helper pool against a queue it cannot empty and answers how
 * large the pool became.
 *
 * `drive_reads_to_completion` waits for each request before submitting the
 * next, so its queue never holds more than one entry.  Growth is gated on
 * `queue_count > held`, which that shape satisfies only while the pool is
 * empty: the first helper it creates is also its last, however high the cap
 * is.  A case that asserts an upper bound above one therefore asserts nothing
 * under that driver -- the bound is not approached, let alone reached, and
 * deleting the code that enforces it changes no verdict.
 *
 * These requests are reads of an empty pipe instead, so a helper that takes
 * one blocks in the host call and never returns for another.  At most one
 * entry per helper ever leaves the queue, so after `i` submissions the queue
 * holds at least `i - held` entries and every submission past twice the
 * current pool size finds `queue_count > held` and adds a helper.  Growth
 * then stops only where the cap stops it, which is the question both cases
 * below ask.
 *
 * The first read is served by a byte written before it, because growth also
 * requires a measured wait and an adapter that has executed nothing has
 * measured none.  Its execution is the sampled one -- the interval starts at
 * the first -- and the caller performs it, because the pool is still empty.
 * The pipe is fed the rest of its bytes only after the pool count has been
 * read, so that count is the pool's final one. */
static int grow_pool_against_a_blocked_queue(
    wf_completion_runtime *runtime,
    wf_file_adapter *adapter,
    int read_descriptor,
    int write_descriptor,
    size_t *held
) {
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    wf_completion_token tokens[WF_POOL_GROWTH_DEPTH];
    unsigned char bytes[WF_POOL_GROWTH_DEPTH];
    char feed[WF_POOL_GROWTH_DEPTH];
    wf_completion_token primer_token;
    wf_file_request request;
    wf_file_result result;
    wf_completion_event event;
    wf_completion_outcome outcome;
    unsigned char primer_byte = 0;
    unsigned index;

    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = read_descriptor;
    request.operation.read.buffer = &primer_byte;
    request.operation.read.count = 1;

    CHECK(write(write_descriptor, "w", 1) == 1);
    CHECK(wf_completion_claim(runtime, &primer_token) == WF_COMPLETION_CLAIMED);
    /* Nothing is measured yet, so this submission cannot grow the pool
     * whatever its cap says, and the caller is the queue's only engine. */
    CHECK(
        wf_file_adapter_submit(adapter, primer_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_helper_count(adapter) == 0);
    CHECK(wf_file_adapter_progress(adapter, 1) == 1);
    CHECK(drain_and_consume_file(runtime, primer_token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 1);
    CHECK(primer_byte == 'w');
    /* One scripted millisecond a call is a wait by any reading of the
     * threshold, so the growth rule's measured half now holds. */
    CHECK(wf_file_adapter_wait_verdict(adapter) == WF_FILE_WAIT_LONG);

    /* The deep queue.  Nothing feeds the pipe from here, so every helper the
     * pool gains blocks on its first read and the queue only deepens. */
    for (index = 0; index < WF_POOL_GROWTH_DEPTH; ++index) {
        bytes[index] = 0;
        feed[index] = 'q';
        CHECK(
            wf_completion_claim(runtime, &tokens[index])
            == WF_COMPLETION_CLAIMED
        );
        request.operation.read.buffer = &bytes[index];
        CHECK(
            wf_file_adapter_submit(adapter, tokens[index], &request)
            == WF_FILE_TARGET_OWNS
        );
    }
    /* Admission has stopped and growth happens only on admission, so this is
     * the size the policy settled on. */
    *held = wf_file_adapter_helper_count(adapter);

    /* One byte for each submitted read: from here no read of this pipe can
     * block, so the pool -- whatever size it reached -- drains the queue, and
     * the caller helps in case the pool is empty. */
    CHECK(
        write(write_descriptor, feed, sizeof(feed))
        == (ssize_t)sizeof(feed)
    );
    for (index = 0; index < WF_POOL_GROWTH_DEPTH; ++index) {
        unsigned attempts;
        int drained = 0;
        for (attempts = 0; attempts < 100000u && drained == 0; ++attempts) {
            if (wf_completion_drain_token(runtime, tokens[index], &event)
                != 0) {
                drained = 1;
                break;
            }
            if (wf_file_adapter_progress(adapter, 1) == 0) {
                (void)nanosleep(&delay, NULL);
            }
        }
        CHECK(drained == 1);
        CHECK(event.token.slot == tokens[index].slot);
        CHECK(event.token.generation == tokens[index].generation);
        CHECK(
            wf_completion_consume(
                runtime,
                tokens[index],
                &result,
                sizeof(result),
                &outcome
            ) == WF_COMPLETION_CONSUMED
        );
        CHECK(result.error_code == 0);
        CHECK(result.value == 1);
        CHECK(bytes[index] == 'q');
    }
    return 0;
}

/* A program whose operations do wait gets helpers, up to the cap and no
 * further.
 *
 * Both ends of that promise need a queue the pool cannot empty.  With one
 * request in flight at a time the pool stops itself at one helper, so a cap of
 * four is neither reached nor tested; against the blocked queue the pool
 * climbs until the cap is the only thing left stopping it, and the count it
 * settles on is the cap exactly. */
static int test_pool_grows_when_operations_wait(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[WF_POOL_GROWTH_DEPTH + 1u];
    wf_file_adapter adapter;
    wf_file_work queue[WF_POOL_GROWTH_DEPTH + 1u];
    pthread_t helpers[4];
    int descriptors[2];
    size_t held = 0;

    CHECK(pipe(descriptors) == 0);
    CHECK(
        wf_completion_runtime_init(
            &runtime,
            slots,
            WF_POOL_GROWTH_DEPTH + 1u
        ) == 0
    );
    CHECK(
        wf_file_adapter_init(
            &adapter,
            &runtime,
            queue,
            WF_POOL_GROWTH_DEPTH + 1u,
            helpers,
            4,
            0
        ) == 0
    );
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 4) == 0);

    /* Nothing measured yet is not evidence of anything, and neither policy
     * fires on it. */
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_UNMEASURED);

    wf_script_clock(1000000u);
    CHECK(
        grow_pool_against_a_blocked_queue(
            &runtime,
            &adapter,
            descriptors[0],
            descriptors[1],
            &held
        ) == 0
    );
    wf_script_clock(0);
    CHECK(held == 4);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_LONG);
    /* A transfer submitted now would be overlapped by a helper, so the
     * submitting thread must not take it itself. */
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 0);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

/* The ceiling a policy asks for is a wish; the storage the caller handed to
 * `wf_file_adapter_init` is the fact.  The growth rule writes `helpers[held]`,
 * so a cap above that storage is a `pthread_create` past its end -- which is
 * why `wf_file_adapter_set_helper_cap` clamps the cap to the storage rather
 * than trusting it.  A cap of eight against storage of two leaves the clamp as
 * the only thing keeping the pool inside the caller's array.
 *
 * The array below is deliberately longer than the two entries init is told
 * about.  The bound under test is the declared capacity, and the slack decides
 * only whether a violation of it is *reported* -- a pool larger than the
 * storage it was given -- or executed as a write past the end of this frame,
 * which ends the run before any check is reached.  With the clamp in place
 * nothing beyond `helpers[1]` is ever written, so the slack is unreachable in
 * a passing run. */
static int test_helper_growth_stops_at_the_helper_storage(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[WF_POOL_GROWTH_DEPTH + 1u];
    wf_file_adapter adapter;
    wf_file_work queue[WF_POOL_GROWTH_DEPTH + 1u];
    pthread_t helpers[WF_POOL_GROWTH_DEPTH];
    int descriptors[2];
    size_t held = 0;

    CHECK(pipe(descriptors) == 0);
    CHECK(
        wf_completion_runtime_init(
            &runtime,
            slots,
            WF_POOL_GROWTH_DEPTH + 1u
        ) == 0
    );
    CHECK(
        wf_file_adapter_init(
            &adapter,
            &runtime,
            queue,
            WF_POOL_GROWTH_DEPTH + 1u,
            helpers,
            2,
            0
        ) == 0
    );
    /* Accepted, and silently reduced to the two helpers init was given. */
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 8) == 0);

    wf_script_clock(1000000u);
    CHECK(
        grow_pool_against_a_blocked_queue(
            &runtime,
            &adapter,
            descriptors[0],
            descriptors[1],
            &held
        ) == 0
    );
    wf_script_clock(0);
    /* Two, not the eight that were asked for: the queue stayed deeper than
     * the pool for every one of the twenty submissions, so the cap is the
     * only thing that stopped it, and the cap is the storage. */
    CHECK(held == 2);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

/* An initial count the storage cannot hold is refused outright: unlike the
 * cap, it is not a ceiling to grow towards but threads to start right now. */
static int test_helper_count_above_its_storage_is_refused(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_file_adapter adapter;
    wf_file_work queue[2];
    pthread_t helpers[1];

    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 2, helpers, 1, 2)
        == EINVAL
    );
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

/* A shut-down record answers "not usable" to every entry, and it answers that
 * before the lock behind the answer is destroyed.
 *
 * `wf_file_adapter_shutdown` destroys the condition variable and the mutex,
 * and every entry of this adapter reads the `initialized` flag and then
 * touches storage behind them.  The flag is therefore what stands between a
 * later caller and destroyed storage, which makes two things testable without
 * a race: after a shutdown every entry is refused at the guard, and a second
 * shutdown is refused rather than joining threads that are gone and
 * destroying a mutex twice.
 *
 * The pool is grown first on purpose.  A shutdown of an adapter that never
 * started a helper joins nothing, so the interesting teardown -- helpers
 * joined, then the objects they waited on destroyed -- would not run at all.
 *
 * What this does not test is the concurrent window the store's position
 * closes: a reader that passes the guard while shutdown is between its
 * destroys is a race, and a race is not what a single-threaded case decides.
 * It does hold the ordering to its unconditional half, which is a real
 * property -- a teardown that reported an error still leaves a record that
 * says it is unusable. */
static int test_shutdown_refuses_every_later_entry(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[WF_POOL_GROWTH_DEPTH + 1u];
    wf_file_adapter adapter;
    wf_file_work queue[WF_POOL_GROWTH_DEPTH + 1u];
    pthread_t helpers[4];
    int descriptors[2];
    size_t held = 0;

    CHECK(pipe(descriptors) == 0);
    CHECK(
        wf_completion_runtime_init(
            &runtime,
            slots,
            WF_POOL_GROWTH_DEPTH + 1u
        ) == 0
    );
    CHECK(
        wf_file_adapter_init(
            &adapter,
            &runtime,
            queue,
            WF_POOL_GROWTH_DEPTH + 1u,
            helpers,
            4,
            0
        ) == 0
    );
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 4) == 0);
    wf_script_clock(1000000u);
    CHECK(
        grow_pool_against_a_blocked_queue(
            &runtime,
            &adapter,
            descriptors[0],
            descriptors[1],
            &held
        ) == 0
    );
    wf_script_clock(0);
    /* The record is live, and says so, before the shutdown below. */
    CHECK(held == 4);
    CHECK(wf_file_adapter_helper_count(&adapter) == 4);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_LONG);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);

    /* Every entry that would take `queue_lock` or read the record's measured
     * state is turned away by the flag instead. */
    CHECK(wf_file_adapter_queued(&adapter) == 0);
    CHECK(wf_file_adapter_helper_count(&adapter) == 0);
    CHECK(wf_file_adapter_transfer_runs_on_caller(&adapter) == 0);
    CHECK(wf_file_adapter_wait_verdict(&adapter) == WF_FILE_WAIT_UNMEASURED);
    CHECK(wf_file_adapter_set_helper_cap(&adapter, 2) == EINVAL);
    /* And shutdown itself is one of those entries. */
    CHECK(wf_file_adapter_shutdown(&adapter) == EINVAL);

    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

static int test_helper_completion_wakes_scheduler(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_file_adapter adapter;
    wf_file_work queue[2];
    pthread_t helper;
    pthread_t scheduler;
    wf_completion_token token;
    wf_file_request request;
    wf_file_result result;
    park_context parked;
    int pipe_descriptors[2];
    char byte = 0;
    const char sent = 'x';
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 1000000};
    unsigned attempts;

    CHECK(pipe(pipe_descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(
        wf_file_adapter_init(&adapter, &runtime, queue, 2, &helper, 1, 1) == 0
    );
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = pipe_descriptors[0];
    request.operation.read.buffer = &byte;
    request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, token, &request)
        == WF_FILE_TARGET_OWNS
    );
    for (attempts = 0; attempts < 5000; ++attempts) {
        if (wf_file_adapter_queued(&adapter) == 0) {
            break;
        }
        (void)nanosleep(&delay, NULL);
    }
    CHECK(wf_file_adapter_queued(&adapter) == 0);

    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&scheduler, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);
    CHECK(write(pipe_descriptors[1], &sent, 1) == 1);
    CHECK(pthread_join(scheduler, NULL) == 0);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(drain_and_consume_file(&runtime, token, &result) == 0);
    CHECK(result.error_code == 0);
    CHECK(result.value == 1);
    CHECK(byte == sent);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(pipe_descriptors[0]) == 0);
    CHECK(close(pipe_descriptors[1]) == 0);
    return 0;
}

typedef struct readiness_writer_context {
    int descriptor;
    unsigned char byte;
} readiness_writer_context;

static void *write_after_readiness_wait(void *opaque) {
    readiness_writer_context *context = opaque;
    struct timespec delay = {.tv_sec = 0, .tv_nsec = 2000000};
    ssize_t written;
    (void)nanosleep(&delay, NULL);
    written = write(context->descriptor, &context->byte, 1);
    (void)written;
    return NULL;
}

static int test_readiness_refusal_is_not_a_terminal_outcome(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token token;
    wf_file_request request;
    wf_file_result result;
    wf_completion_event event;
    wf_completion_outcome outcome;
    readiness_writer_context writer_context;
    pthread_t writer;
    int descriptors[2];
    int flags;
    unsigned char byte = 0;

    CHECK(pipe(descriptors) == 0);
    flags = fcntl(descriptors[0], F_GETFL, 0);
    CHECK(flags >= 0);
    CHECK(fcntl(descriptors[0], F_SETFL, flags | O_NONBLOCK) == 0);
    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0, 0) == 0);
    CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = descriptors[0];
    request.operation.read.buffer = &byte;
    request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, token, &request)
        == WF_FILE_TARGET_OWNS
    );
    writer_context.descriptor = descriptors[1];
    writer_context.byte = 'r';
    CHECK(
        pthread_create(
            &writer,
            NULL,
            write_after_readiness_wait,
            &writer_context
        ) == 0
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(pthread_join(writer, NULL) == 0);
    CHECK(byte == 'r');
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
    CHECK(event.token.slot == token.slot);
    CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result.value == 1);
    CHECK(result.error_code == 0);
    CHECK(wf_completion_statistics_snapshot(&runtime).publications == 1);
    CHECK(wf_completion_statistics_snapshot(&runtime).consumptions == 1);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
    return 0;
}

typedef struct progress_context {
    wf_file_adapter *adapter;
    size_t progressed;
} progress_context;

static void *progress_one_file_operation(void *opaque) {
    progress_context *context = opaque;
    context->progressed = wf_file_adapter_progress(context->adapter, 1);
    return NULL;
}

static int test_capacity_release_wakes_before_blocking_work(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_file_adapter adapter;
    wf_file_work queue[1];
    wf_completion_token read_token;
    wf_completion_token write_token;
    wf_completion_event events[2];
    wf_completion_outcome outcome;
    wf_file_request read_request;
    wf_file_request write_request;
    wf_file_result read_result;
    wf_file_result write_result;
    park_context parked;
    progress_context progress;
    pthread_t scheduler;
    pthread_t target_progress;
    int pipe_descriptors[2];
    char received = 0;
    const char sent = 'c';

    CHECK(pipe(pipe_descriptors) == 0);
    CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 1, NULL, 0, 0) == 0);
    CHECK(wf_completion_claim(&runtime, &read_token) == WF_COMPLETION_CLAIMED);
    memset(&read_request, 0, sizeof(read_request));
    read_request.kind = WF_FILE_READ;
    read_request.operation.read.descriptor = pipe_descriptors[0];
    read_request.operation.read.buffer = &received;
    read_request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, read_token, &read_request)
        == WF_FILE_TARGET_OWNS
    );

    CHECK(wf_completion_claim(&runtime, &write_token) == WF_COMPLETION_CLAIMED);
    memset(&write_request, 0, sizeof(write_request));
    write_request.kind = WF_FILE_WRITE;
    write_request.operation.write.descriptor = pipe_descriptors[1];
    write_request.operation.write.buffer = &sent;
    write_request.operation.write.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, write_token, &write_request)
        == WF_FILE_WAIT_CAPACITY
    );

    parked.runtime = &runtime;
    parked.epoch = wf_completion_wake_epoch(&runtime);
    parked.result = WF_COMPLETION_PARK_FAILED;
    CHECK(pthread_create(&scheduler, NULL, park_scheduler, &parked) == 0);
    CHECK(wait_until_parked(&runtime) == 0);

    /* This scheduler-driven target progress thread removes the read from the
     * full queue, publishes capacity, then blocks in the typed read.  The
     * capacity notification must wake the scheduler before the read can ever
     * complete, because submitting the waiting write is what unblocks it. */
    progress.adapter = &adapter;
    progress.progressed = 0;
    CHECK(
        pthread_create(
            &target_progress,
            NULL,
            progress_one_file_operation,
            &progress
        ) == 0
    );
    CHECK(pthread_join(scheduler, NULL) == 0);
    CHECK(parked.result == WF_COMPLETION_PARK_WOKEN);
    CHECK(
        wf_file_adapter_submit(&adapter, write_token, &write_request)
        == WF_FILE_TARGET_OWNS
    );
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(pthread_join(target_progress, NULL) == 0);
    CHECK(progress.progressed == 1);
    CHECK(drain_exact(&runtime, 2, events) == 0);

    CHECK(
        wf_completion_consume(
            &runtime,
            read_token,
            &read_result,
            sizeof(read_result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(read_result.error_code == 0);
    CHECK(read_result.value == 1);
    CHECK(received == sent);
    CHECK(
        wf_completion_consume(
            &runtime,
            write_token,
            &write_result,
            sizeof(write_result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(write_result.error_code == 0);
    CHECK(write_result.value == 1);

    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(pipe_descriptors[0]) == 0);
    CHECK(close(pipe_descriptors[1]) == 0);
    return 0;
}

/* The runtime's own bound on how many iterations of one loop may be in flight
 * at once.
 *
 * The rules this checks are the only ones a lowering may rely on: every
 * argument is a bound and none can raise the answer, a zero argument places no
 * bound, and one is always a legal answer.  That last one is what makes the
 * query safe to emit at all — a loop whose window came back as zero would have
 * no legal schedule, and the sequential program is always a legal schedule. */
static int test_completion_window_answers_at_the_boundaries(void) {
    uint64_t unconstrained = wf__completion_window(0, 0, 0);
    uint64_t budget = WF_HARNESS_WINDOW_BYTE_BUDGET;
    uint64_t two;

    /* The runtime never offers its whole operation capacity to one loop: a
     * loop holding every record would push the rest of the program onto the
     * capacity-wait path, where a full ring degrades to a blocking direct
     * call. */
    CHECK(unconstrained >= 1u);
    CHECK(unconstrained <= WF_HARNESS_OPERATION_CAPACITY / 2u);

    /* A trip count of one is a loop with nothing to overlap. */
    CHECK(wf__completion_window(1, 0, 0) == 1u);
    /* The compiler's own static cap is honoured exactly. */
    CHECK(wf__completion_window(0, 0, 1) == 1u);
    CHECK(wf__completion_window(0, 0, 2) == (unconstrained < 2u ? unconstrained : 2u));
    CHECK(wf__completion_window(3, 0, 0) == (unconstrained < 3u ? unconstrained : 3u));
    /* Neither a huge trip count nor a huge ceiling raises the runtime's own
     * answer: every argument is a minimum with it, never a request. */
    CHECK(wf__completion_window(UINT64_MAX, 0, 0) == unconstrained);
    CHECK(wf__completion_window(0, 0, UINT64_MAX) == unconstrained);
    CHECK(wf__completion_window(UINT64_MAX, 0, UINT64_MAX) == unconstrained);

    /* The byte budget. One slot of the whole budget affords one iteration;
     * half of it affords two; a slot larger than the budget affords none, and
     * the floor turns that into the sequential program rather than into a
     * schedule with no slots. */
    CHECK(wf__completion_window(0, budget, 0) == 1u);
    CHECK(wf__completion_window(0, budget + 1u, 0) == 1u);
    CHECK(wf__completion_window(0, UINT64_MAX, 0) == 1u);
    /* The design's own example: a loop privatizing a 16 MiB buffer gets one. */
    CHECK(wf__completion_window(0, 16u * 1024u * 1024u, 0) == 1u);
    two = budget / 2u;
    CHECK(wf__completion_window(0, two, 0) == (unconstrained < 2u ? unconstrained : 2u));

    /* The smallest argument decides, whichever one it is. */
    CHECK(wf__completion_window(2, budget / 8u, 8) == 2u);
    CHECK(wf__completion_window(8, budget / 2u, 8) == (unconstrained < 2u ? unconstrained : 2u));
    CHECK(wf__completion_window(8, budget / 8u, 2) == 2u);

    /* One is always legal: no combination of arguments answers zero. */
    CHECK(wf__completion_window(0, 0, 0) >= 1u);
    CHECK(wf__completion_window(1, UINT64_MAX, 1) == 1u);
    CHECK(wf__completion_window(UINT64_MAX, UINT64_MAX, UINT64_MAX) == 1u);
    return 0;
}

/* The io_uring doorbell is deferred, and every path that can stop reaches the
 * kernel before it does.
 *
 * Staging a submission-queue entry is a store to a page the kernel already
 * maps, so it costs no system call; `io_uring_enter` is what tells the kernel
 * the entry exists. Ringing it once per submission is 15,360 calls on the
 * eight-wide benchmark and 2,048 when it is deferred (LOOP-PIPELINE.md §9.1).
 * Deferring is only safe while the two facts below hold, and they are what
 * this checks: a submission alone rings nothing, and the first thing that
 * could wait — a join, or a blocking direct host call the completion path
 * refused to carry — rings it first.
 *
 * The enter counter only exists where the ring is the target route, so the
 * counted half of this test runs there and the functional half runs
 * everywhere. */
static int test_a_submitted_operation_is_kicked_before_it_waits(
    const char *scratch_directory
) {
    char path[256];
    unsigned char first[8];
    unsigned char second[8];
    wf_completion_token open_token;
    wf_completion_token read_token;
    wf_completion_token other_token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    int descriptor;
    int writer;
    uint64_t submissions_before;
    uint64_t enters;
    int ring_route;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-doorbell-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(write(writer, "doorbell", 8) == 8);
    CHECK(close(writer) == 0);

    submissions_before = wf__completion_linux_io_uring_submissions();
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &open_token
        ) == 1
    );
    wf__completion_file_open_join(
        &open_token,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0 && error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    descriptor = (int)value;
    ring_route =
        wf__completion_linux_io_uring_submissions() > submissions_before;

    /* A submission alone rings nothing. */
    enters = wf__completion_linux_io_uring_submission_enters();
    memset(first, 0, sizeof(first));
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            sizeof(first),
            0,
            &read_token
        ) == 1
    );
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters);
    }

    /* A blocking direct host call rings it first. This one is refused
     * immediately by the host, so what it proves is the ordering and not the
     * waiting: the doorbell rang before the call, not after it. */
    CHECK(wf__completion_file_close_direct(-1) < 0);
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters + 1u);
    }
    wf__completion_file_join(&read_token, &value, &error_code);
    CHECK(value == 8 && error_code == 0);
    CHECK(memcmp(first, "doorbell", 8) == 0);

    /* Two submissions, one doorbell: the first join carries both, and the
     * second join needs no further call. This is the whole of what deferring
     * buys, stated as a count. */
    enters = wf__completion_linux_io_uring_submission_enters();
    memset(first, 0, sizeof(first));
    memset(second, 0, sizeof(second));
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            first,
            4,
            0,
            &read_token
        ) == 1
    );
    CHECK(
        wf__completion_file_pread_submit(
            descriptor,
            second,
            4,
            4,
            &other_token
        ) == 1
    );
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters);
    }
    wf__completion_file_join(&read_token, &value, &error_code);
    CHECK(value == 4 && error_code == 0);
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters + 1u);
    }
    wf__completion_file_join(&other_token, &value, &error_code);
    CHECK(value == 4 && error_code == 0);
    if (ring_route) {
        CHECK(wf__completion_linux_io_uring_submission_enters() == enters + 1u);
    }
    CHECK(memcmp(first, "door", 4) == 0);
    CHECK(memcmp(second, "bell", 4) == 0);

    CHECK(wf__completion_file_close_submit(descriptor, &read_token) == 1);
    wf__completion_file_join(&read_token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* Fills this process's descriptor table exactly: opens `path` at the lowest
 * free descriptor and lowers the soft limit to one past it, so no further open
 * can succeed until that one descriptor is given back.  `*saved` carries the
 * limit the caller must restore.  Returns the held descriptor. */
static int wf_harness_hold_the_last_descriptor(
    const char *path,
    struct rlimit *saved
) {
    struct rlimit narrowed;
    int probe;
    int held;
    if (getrlimit(RLIMIT_NOFILE, saved) != 0) {
        return -1;
    }
    probe = open(path, O_RDONLY);
    if (probe < 0) {
        return -1;
    }
    if (close(probe) != 0) {
        return -1;
    }
    narrowed = *saved;
    narrowed.rlim_cur = (rlim_t)probe + 1;
    if (narrowed.rlim_cur > saved->rlim_max) {
        return -1;
    }
    if (setrlimit(RLIMIT_NOFILE, &narrowed) != 0) {
        return -1;
    }
    held = open(path, O_RDONLY);
    if (held != probe) {
        if (held >= 0) {
            (void)close(held);
        }
        (void)setrlimit(RLIMIT_NOFILE, saved);
        return -1;
    }
    return held;
}

/* An open refused for want of a host descriptor retires the work this runtime
 * still owns and re-attempts once before publishing an outcome.
 *
 * Retire and retry (LOOP-PIPELINE.md §2.10).  A schedule that keeps several
 * iterations of a loop in flight holds several descriptors where source order
 * holds one, so a host limit the sequential program never reaches could turn a
 * correct program's `Ok` into an `Err(ResourceExhausted)` — and [SYS-10] is
 * explicit that a `FilePermit` promises no native descriptor, while [PAR-1]'s
 * exhaustion clause excuses only what an implementation spends on overlapping.
 * The descriptors the program's own opens consume are not that.
 *
 * The shape here is the smallest one that has the property: a close of the one
 * held descriptor is submitted, then an open which cannot succeed until that
 * close runs.  With no helper the scheduler takes the newest request first, so
 * the open really is attempted while the close is still owed, and only the
 * retirement makes it succeed.  Both outcomes are published unchanged. */
static int test_open_exhaustion_retires_owned_work_and_retries(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_file_adapter adapter;
    wf_file_work queue[4];
    wf_completion_token close_token;
    wf_completion_token open_token;
    wf_file_request request;
    wf_file_result result;
    wf_completion_event events[2];
    wf_completion_outcome outcome;
    struct rlimit saved;
    char path[256];
    int writer;
    int held;
    int opened;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-exhaustion-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Everything that needs a descriptor of its own happens before the limit
     * narrows. */
    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 4, NULL, 0, 0) == 0);
    CHECK(wf_completion_claim(&runtime, &close_token) == WF_COMPLETION_CLAIMED);
    CHECK(wf_completion_claim(&runtime, &open_token) == WF_COMPLETION_CLAIMED);

    held = wf_harness_hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    /* The premise: with the table full the host refuses an open outright. */
    CHECK(open(path, O_RDONLY) < 0);

    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_CLOSE;
    request.operation.close.descriptor = held;
    CHECK(
        wf_file_adapter_submit(&adapter, close_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = AT_FDCWD;
    request.operation.open_at.path = path;
    request.operation.open_at.flags = O_RDONLY;
    request.operation.open_at.expected_kind = WF_FILE_EXPECT_REGULAR;
    CHECK(
        wf_file_adapter_submit(&adapter, open_token, &request)
        == WF_FILE_TARGET_OWNS
    );

    /* One scheduler visit. It takes the open, is refused, retires the close it
     * still owed, and re-attempts. */
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(
        wf_file_adapter_statistics_snapshot(&adapter).exhaustion_retries == 1
    );

    CHECK(drain_exact(&runtime, 2, events) == 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            close_token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result.kind == WF_FILE_CLOSE);
    CHECK(result.value == 0 && result.error_code == 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            open_token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result.kind == WF_FILE_OPEN_AT);
    /* The whole point: the outcome the program sees is the one source order
     * produces, not the exhaustion the schedule caused. */
    CHECK(result.value >= 0 && result.error_code == 0);
    CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    opened = (int)result.value;

    CHECK(close(opened) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* The same rule on the route generated code takes, whichever target carries
 * it.
 *
 * Here nothing can give a descriptor back, so every open fails — and that is
 * the outcome the program is entitled to see, because it is the one source
 * order produces too.  What the count proves is the bound: an exhausted host
 * cannot turn one `open_file` call into unbounded work, so three refused opens
 * can produce at most three second attempts however the three are scheduled.
 *
 * This test asserted the opposite bound until the rule below was corrected —
 * that the count had *moved*, that is, that at least one re-attempt was made.
 * That is not a property of three independent opens.  A re-attempt is owed
 * only where this runtime is still holding a descriptor it could give back,
 * and whether it is depends on where the three land: a helper pool that
 * finishes each open before the next is submitted holds nothing at the moment
 * of each refusal, so publishing each refusal unchanged is the correct answer
 * and no re-attempt is owed.  Asserting otherwise made this test fail 13 of
 * 200 runs at `WF_IO_HELPERS=4` on macOS, inside canonical `make check`.
 *
 * The property it was reaching for — that a runtime still holding a descriptor
 * asks the host again rather than publishing the refusal — is what
 * `test_open_exhaustion_waits_for_another_engine` and
 * `test_bridge_open_behind_a_submitted_close_succeeds` establish, each by
 * arranging that schedule instead of hoping for it, and each failing without
 * its fix. */
static int test_bridge_open_exhaustion_is_retried_once(
    const char *scratch_directory
) {
    struct rlimit saved;
    char path[256];
    wf_completion_token tokens[3];
    wf_completion_token token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_SUCCEEDED;
    uint64_t retries_before;
    size_t index;
    int writer;
    int held;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-bridge-exhaustion-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Warm every descriptor the bridge itself needs before the limit narrows:
     * the ring, its wait set, and any helper the policy asks for are created
     * on the first submission, not here. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    retries_before = wf__completion_open_exhaustion_retries();
    held = wf_harness_hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);

    for (index = 0; index < 3; ++index) {
        CHECK(
            wf__completion_file_open_at_submit(
                AT_FDCWD,
                path,
                O_RDONLY,
                0u,
                0u,
                WF_FILE_EXPECT_REGULAR,
                &tokens[index]
            ) == 1
        );
    }
    for (index = 0; index < 3; ++index) {
        wf__completion_file_open_join(
            &tokens[index],
            &value,
            &error_code,
            &open_outcome
        );
        CHECK(value < 0);
        CHECK(open_outcome == WF_FILE_OPEN_FAILED);
        CHECK(error_code == EMFILE || error_code == ENFILE);
    }
    CHECK(
        wf__completion_open_exhaustion_retries() - retries_before
        <= (uint64_t)3
    );

    CHECK(close(held) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* What a refused open must retire is not always on this adapter's queue.
 *
 * With one engine it is, and draining the queue is the whole of the rule.
 * With several the same schedule puts it elsewhere: three simultaneous opens
 * are taken by three different helpers and every one of them looks at an empty
 * queue.  An adapter that gives up there publishes an exhaustion the
 * sequential program never produces, because the descriptors are still its
 * own — held by operations it is running, which is not the same as gone.
 *
 * The shape below fixes that schedule rather than racing for it.  A read of an
 * empty pipe parks the one helper, so exactly one operation is in flight and
 * nothing at all is queued.  The scheduler then takes an open the host must
 * refuse, and because the adapter's `openat` is named, the releasing thread
 * stands exactly at the refusal: only then does it give the held descriptor
 * back and let the parked read finish.  The open's second attempt is the first
 * one that could succeed, and the outcome the program sees is the one source
 * order produces. */
struct wf_open_release {
    int held;
    int pipe_writer;
    ssize_t released;
};

static void *wf_open_release_main(void *context) {
    struct wf_open_release *release = context;
    (void)pthread_mutex_lock(&wf_open_gate_lock);
    while (wf_open_gate_refusals == 0) {
        (void)pthread_cond_wait(&wf_open_gate_signal, &wf_open_gate_lock);
    }
    (void)pthread_mutex_unlock(&wf_open_gate_lock);
    /* The descriptor the refused open needs comes back here, and the parked
     * read is released so the adapter observes a retirement on its other
     * engine.  The byte's fate is recorded rather than discarded: a write
     * that did not land would leave that helper parked, and the test reads
     * this back before it concludes anything about the schedule. */
    (void)close(release->held);
    release->released = write(release->pipe_writer, "x", 1);
    return NULL;
}

/* The one question the retirement ledger asks an operation is answered from
 * the record, and only a host action that actually put a descriptor back may
 * answer yes: a close the host refused freed nothing, and counting it would
 * spend a refused open's single re-attempt on a return that never happened. */
static int test_descriptor_return_follows_the_outcome(void) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.kind = WF_FILE_CLOSE;
    result.error_code = 0;
    CHECK(wf_file_returned_a_descriptor(&result) == 1);
    result.error_code = EBADF;
    CHECK(wf_file_returned_a_descriptor(&result) == 0);
    memset(&result, 0, sizeof(result));
    result.kind = WF_FILE_OPEN_AT;
    result.value = 7;
    result.open_outcome = WF_FILE_OPEN_OTHER_KIND;
    CHECK(wf_file_returned_a_descriptor(&result) == 1);
    result.open_outcome = WF_FILE_OPEN_SUCCEEDED;
    CHECK(wf_file_returned_a_descriptor(&result) == 0);
    result.value = -1;
    result.open_outcome = WF_FILE_OPEN_FAILED;
    CHECK(wf_file_returned_a_descriptor(&result) == 0);
    memset(&result, 0, sizeof(result));
    result.kind = WF_FILE_PREAD;
    result.value = 4096;
    CHECK(wf_file_returned_a_descriptor(&result) == 0);
    CHECK(wf_file_returned_a_descriptor(NULL) == 0);
    return 0;
}

/* The same question, asked of the ring, which answers it from its own entry.
 *
 * The test above calls the shared inline answer, so it pins the record half of
 * the predicate and nothing else.  A ring that counted every close it completed
 * as a descriptor coming back — whatever the kernel said about it — passes that
 * test and every other one in this file, while a refused close spends a refused
 * open's single re-attempt on a descriptor that never came back.  Measured on
 * the shape that submits both, it mis-accounts one return per repetition.
 *
 * So this stands where only the ring can answer: two closes, one the kernel
 * performs and one it refuses, each checked to have gone to the ring, with the
 * process-wide return count read across each.  The deltas are exact — one and
 * zero — because nothing else of this runtime's is in flight here. */
static int test_a_ring_close_counts_a_return_only_when_it_ran(
    const char *scratch_directory
) {
    wf_completion_token token;
    char path[256];
    uint64_t ring_before;
    uint64_t returns_before;
    int64_t value = -1;
    int error_code = -1;
    int held;
    int writer;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-ring-close-return-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);
    held = open(path, O_RDONLY);
    CHECK(held >= 0);

    /* A close the host performs: one descriptor back. */
    ring_before = wf__completion_linux_io_uring_submissions();
    returns_before = wf_completion_descriptor_returns();
    CHECK(wf__completion_file_close_submit(held, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    if (wf__completion_linux_io_uring_submissions() == ring_before) {
        /* This host's closes do not take the ring, so the ring's own answer is
         * not reachable from here.  A run that demanded the ring may not pass
         * by skipping. */
        CHECK(getenv("WF_REQUIRE_LINUX_IO_URING") == NULL);
        CHECK(unlink(path) == 0);
        return 0;
    }
    CHECK(wf_completion_descriptor_returns() == returns_before + 1u);

    /* A close the host refuses: nothing came back, and the count says so. */
    ring_before = wf__completion_linux_io_uring_submissions();
    returns_before = wf_completion_descriptor_returns();
    value = 0;
    error_code = 0;
    CHECK(wf__completion_file_close_submit(30000, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value < 0 && error_code == EBADF);
    CHECK(wf__completion_linux_io_uring_submissions() == ring_before + 1u);
    CHECK(wf_completion_descriptor_returns() == returns_before);

    CHECK(unlink(path) == 0);
    return 0;
}

static int test_open_exhaustion_waits_for_another_engine(
    const char *scratch_directory
) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[4];
    wf_file_adapter adapter;
    wf_file_work queue[4];
    pthread_t helpers[1];
    pthread_t releaser;
    struct wf_open_release release;
    wf_completion_token read_token;
    wf_completion_token open_token;
    wf_file_request request;
    wf_file_result result;
    wf_completion_event events[2];
    wf_completion_outcome outcome;
    struct rlimit saved;
    char path[256];
    unsigned char received = 0;
    int pipe_pair[2];
    int writer;
    int held;
    int opened;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-other-engine-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Everything that needs a descriptor of its own happens before the limit
     * narrows: the runtime, the pipe that parks the helper, and the helper
     * itself. */
    CHECK(wf_completion_runtime_init(&runtime, slots, 4) == 0);
    CHECK(pipe(pipe_pair) == 0);
    CHECK(fcntl(pipe_pair[0], F_SETFL, O_NONBLOCK) == 0);
    CHECK(wf_file_adapter_init(&adapter, &runtime, queue, 4, helpers, 1, 1) == 0);
    CHECK(wf_completion_claim(&runtime, &read_token) == WF_COMPLETION_CLAIMED);
    CHECK(wf_completion_claim(&runtime, &open_token) == WF_COMPLETION_CLAIMED);

    /* One operation in flight on the helper, waiting for a byte that only the
     * releasing thread writes. */
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = pipe_pair[0];
    request.operation.read.buffer = &received;
    request.operation.read.count = 1;
    CHECK(
        wf_file_adapter_submit(&adapter, read_token, &request)
        == WF_FILE_TARGET_OWNS
    );
    while (wf_file_adapter_queued(&adapter) != 0) {
        sched_yield();
    }

    held = wf_harness_hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    /* The premise: with the table full the host refuses an open outright. */
    CHECK(open(path, O_RDONLY) < 0);

    (void)pthread_mutex_lock(&wf_open_gate_lock);
    wf_open_gate_refusals = 0;
    atomic_store_explicit(&wf_open_gate_armed, 1, memory_order_release);
    (void)pthread_mutex_unlock(&wf_open_gate_lock);
    release.held = held;
    release.pipe_writer = pipe_pair[1];
    release.released = -1;
    CHECK(pthread_create(&releaser, NULL, wf_open_release_main, &release) == 0);

    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_OPEN_AT;
    request.operation.open_at.directory = AT_FDCWD;
    request.operation.open_at.path = path;
    request.operation.open_at.flags = O_RDONLY;
    request.operation.open_at.expected_kind = WF_FILE_EXPECT_REGULAR;
    CHECK(
        wf_file_adapter_submit(&adapter, open_token, &request)
        == WF_FILE_TARGET_OWNS
    );

    /* One scheduler visit.  It takes the open, is refused, finds nothing
     * queued to retire, waits for the operation running on the helper, and
     * re-attempts once. */
    CHECK(wf_file_adapter_progress(&adapter, 1) == 1);
    CHECK(pthread_join(releaser, NULL) == 0);
    CHECK(release.released == 1);
    (void)pthread_mutex_lock(&wf_open_gate_lock);
    atomic_store_explicit(&wf_open_gate_armed, 0, memory_order_release);
    (void)pthread_mutex_unlock(&wf_open_gate_lock);
    CHECK(
        wf_file_adapter_statistics_snapshot(&adapter).exhaustion_retries == 1
    );

    CHECK(drain_exact(&runtime, 2, events) == 0);
    CHECK(
        wf_completion_consume(
            &runtime,
            read_token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result.kind == WF_FILE_READ);
    CHECK(result.value == 1 && result.error_code == 0);
    CHECK(received == 'x');
    CHECK(
        wf_completion_consume(
            &runtime,
            open_token,
            &result,
            sizeof(result),
            &outcome
        ) == WF_COMPLETION_CONSUMED
    );
    CHECK(result.kind == WF_FILE_OPEN_AT);
    /* The whole point: the outcome the program sees is the one source order
     * produces, not the exhaustion the schedule caused. */
    CHECK(result.value >= 0 && result.error_code == 0);
    CHECK(result.open_outcome == WF_FILE_OPEN_SUCCEEDED);
    opened = (int)result.value;

    CHECK(close(opened) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    CHECK(wf_file_adapter_shutdown(&adapter) == 0);
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    CHECK(close(pipe_pair[0]) == 0);
    CHECK(close(pipe_pair[1]) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* A close this runtime already owns is a descriptor it has not yet given
 * back, and an open submitted behind it must not be told the host is out.
 *
 * Source order is `close(held); open(path)` and it succeeds: the close returns
 * the one free descriptor and the open takes it.  A pipeline holds both at
 * once, so the open can reach the host while the close is still owed and the
 * host answers `EMFILE` — an outcome the sequential program never produces.
 *
 * This is the same rule as the adapter test above, on the route generated code
 * takes.  On a target with a kernel completion ring both operations sit in one
 * ring, and the refusal arrives as a completion of its own: re-attempting the
 * open inside the reap pass that saw the refusal races the close, because that
 * pass sees only the completions the ring had already posted.  The re-attempt
 * has to wait for the close's completion to be published, and this test fails
 * for exactly as long as it does not. */
static int test_bridge_open_behind_a_submitted_close_succeeds(
    const char *scratch_directory
) {
    struct rlimit saved;
    char path[256];
    wf_completion_token close_token;
    wf_completion_token open_token;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    int writer;
    int held;
    int opened;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-bridge-close-first-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Warm every descriptor the bridge itself needs before the limit narrows:
     * the ring, its wait set, and any helper the policy asks for are created
     * on the first submission, not here. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &open_token
        ) == 1
    );
    wf__completion_file_open_join(&open_token, &value, &error_code, &open_outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(wf__completion_file_close_submit((int)value, &open_token) == 1);
    wf__completion_file_join(&open_token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    held = wf_harness_hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    /* The premise: with the table full the host refuses an open outright. */
    CHECK(open(path, O_RDONLY) < 0);

    /* Source order, submitted in source order. */
    CHECK(wf__completion_file_close_submit(held, &close_token) == 1);
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &open_token
        ) == 1
    );

    wf__completion_file_open_join(&open_token, &value, &error_code, &open_outcome);
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    opened = (int)value;

    wf__completion_file_join(&close_token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(wf__completion_file_close_submit(opened, &close_token) == 1);
    wf__completion_file_join(&close_token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* The rule's third answer, at the one moment it is about.
 *
 * A give-up decision reads the descriptor-return count, then how many
 * operations are in flight and how many of those are themselves waiting.  A
 * returning retirement whose increment lands after the first read and whose
 * in-flight decrement lands before the second is invisible to both: the
 * decision sees an unchanged generation and nothing left that could give a
 * descriptor back, and a rule that stopped there would publish an exhaustion
 * a descriptor had already answered.
 *
 * The ledger's schedule point is named so this test can stand exactly between
 * those two reads instead of racing for the interleaving. */
static int test_a_retirement_between_the_ledger_reads_is_not_missed(void) {
    wf_retirement_waiter waiter;
    uint64_t seen;
    enum wf_retirement_state state;

    /* Two operations in flight: the refused open this waiter is deciding, and
     * one other that could still return a descriptor. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&waiter, seen, NULL, NULL, 0);

    (void)pthread_mutex_lock(&wf_retirement_point_lock);
    wf_retirement_point_mode = WF_HARNESS_POINT_RETIRES;
    wf_retirement_point_armed = 1;
    wf_retirement_point_fired = 0;
    (void)pthread_mutex_unlock(&wf_retirement_point_lock);

    state = wf_completion_retirement_state(&waiter);

    (void)pthread_mutex_lock(&wf_retirement_point_lock);
    wf_retirement_point_armed = 0;
    (void)pthread_mutex_unlock(&wf_retirement_point_lock);

    CHECK(wf_retirement_point_fired == 1);
    CHECK(state == WF_RETIREMENT_HAPPENED);

    wf_completion_retirement_wait_end(&waiter);
    /* This waiter's own refused open, publishing: it returns nothing. */
    wf_completion_operation_retired(0);
    return 0;
}

/* The two facts the ledger keeps apart, at the moment they differ.
 *
 * A refused open has exactly one re-attempt, and the only thing that can make
 * a second host attempt answer differently from the first is a descriptor
 * coming back.  An operation that merely ends — a read on a helper thread, a
 * write, a directory batch — changes what is in flight and nothing else.  A
 * ledger that counted it as a descriptor return would send the refused open to
 * the host once more for nothing and then publish the refusal, while the close
 * it was waiting for was still in flight: exactly the `Ok` this ledger exists
 * to keep, since source order runs the read, then the close, then the open,
 * and the open succeeds.
 *
 * The schedule is scripted rather than raced for, because it is the whole
 * point and it is one schedule for both engines: on Linux the read is on the
 * bounded adapter while the close and the open are on the kernel ring, and a
 * host whose ring makes that close asynchronous — an `overlayfs` file, whose
 * `flush` sends `IORING_OP_CLOSE` to a worker — produces this order by itself
 * often enough to lose an `Ok` at every helper count.  The record carries the
 * counts; this test carries the schedule, which is the part that must not
 * depend on a host reaching it. */
static int test_an_ending_that_returns_nothing_grants_no_reattempt(void) {
    wf_retirement_waiter waiter;
    uint64_t seen;

    /* Three operations in flight: the refused open this waiter is deciding, a
     * read that will end returning nothing, and a close that will return a
     * descriptor. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&waiter, seen, NULL, NULL, 0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* The read ends.  Something that can still give a descriptor back is in
     * flight, so this open keeps waiting — and has spent nothing. */
    wf_completion_operation_retired(0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* The close ends, returning one.  Now, and only now, the re-attempt. */
    wf_completion_operation_retired(1);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_HAPPENED);

    wf_completion_retirement_wait_end(&waiter);
    wf_completion_operation_retired(0);
    return 0;
}

/* A returned descriptor goes to the waiter that registered first, and only to
 * that one.
 *
 * Two refused opens waiting, one descriptor coming back, and one re-attempt
 * each: if both are told a descriptor came back, both spend their re-attempt on
 * it, one of them takes it and the other publishes a refusal it had a claim on.
 * Which one takes it is then whichever reaches the host first, and source order
 * says it is the first open that asked. Measured on the racing shape, the
 * unordered award published the `Ok` at the wrong call in 963 of 1,000 runs.
 *
 * The schedule is scripted rather than raced for, because the rule is about
 * which waiter is told, not about who wins a race: the second waiter keeps
 * waiting while the first is owed, and what the first does not consume — here,
 * a descriptor its re-attempt takes and the kind check disposes of, which is a
 * return again — falls to the next. */
static int test_a_returned_descriptor_is_awarded_in_waiter_order(void) {
    wf_retirement_waiter first;
    wf_retirement_waiter second;
    uint64_t seen;

    /* Four operations in flight: the two refused opens these waiters are
     * deciding, and two others that can still give a descriptor back. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&first, seen, NULL, NULL, 0);
    wf_completion_retirement_wait_begin(&second, seen, NULL, NULL, 0);
    CHECK(wf_completion_retirement_state(&first) == WF_RETIREMENT_AWAITED);
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_AWAITED);

    /* One descriptor comes back.  It is owed to the waiter that registered
     * first, and the later one may not spend its one re-attempt on it. */
    wf_completion_operation_retired(1);
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_AWAITED);
    CHECK(wf_completion_retirement_state(&first) == WF_RETIREMENT_HAPPENED);
    /* Asking again awards nothing a second time. */
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_AWAITED);

    /* The first waiter leaves to spend it, and its re-attempt takes the
     * descriptor and gives it straight back — the kind check refused it — so
     * a return falls to the next waiter, which is now the earliest. */
    wf_completion_retirement_wait_end(&first);
    wf_completion_operation_retired(1);
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_HAPPENED);

    wf_completion_retirement_wait_end(&second);
    /* The second waiter's own refused open, and the operation that never
     * returned anything: neither of them returns one. */
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    return 0;
}

/* A descriptor this runtime returned is promised to one open, and an open that
 * has taken one is charged for it whether or not it ever waited.
 *
 * The award mark is what stops two refused opens spending their one re-attempt
 * each on the same return.  An open whose *first* host attempt succeeds is
 * outside that accounting entirely: it never registers as a waiter and takes no
 * award, so the return it consumed is still on offer, and the refused open the
 * ledger then hands it to spends its single re-attempt on a descriptor that is
 * already gone.  Measured on the interleave shape before the charge below: 25
 * lost `Ok`s in 280,000 repetitions, each with a descriptor left over at the
 * end and a re-attempt count one short of every open having been refused.
 *
 * The schedule is scripted rather than raced for, because it is one moment
 * between two threads — a close landing, an open taking what it returned, and a
 * waiter asking — and no host reaches it on demand. */
static int test_an_open_that_took_a_return_is_charged_for_it(void) {
    wf_retirement_waiter waiter;
    uint64_t seen;

    /* Three operations in flight: the refused open this waiter is deciding, a
     * close that will return a descriptor, and an open on another thread whose
     * first host attempt will take it. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&waiter, seen, NULL, NULL, 0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* The close ends, returning one — and the other open's first attempt takes
     * it before this waiter next asks.  That open is charged for it, so what
     * came back is spent and this waiter is not told it may have it. */
    wf_completion_operation_retired(1);
    wf_completion_retirement_open_took_a_descriptor(0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* The open that took it ends holding its descriptor, so it returns
     * nothing.  Now nothing is left that could bring one back, and this
     * waiter's one attempt is made here — the moment source order makes it —
     * rather than earlier against a descriptor another open already held. */
    wf_completion_operation_retired(0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_UNREACHABLE);

    wf_completion_retirement_wait_end(&waiter);
    wf_completion_operation_retired(0);
    return 0;
}

/* A descriptor already promised is not promised again to a waiter that arrives
 * with an older view of the world.
 *
 * A waiter's `seen` is read before its own host attempt, and on the ring that
 * is the moment of submission — long before the kernel refuses it and longer
 * still before the refusal is reaped and the waiter registers.  Waiters
 * therefore arrive with `seen` values older than what the ledger has already
 * done, and they arrive one at a time: each finds the order empty, because the
 * one before it left to spend its award.  A mark that re-opened on each of
 * them promised one returned descriptor to all three, and the two that reached
 * the host after it was taken published the refusal.  Traced on the interleave
 * shape under load, that is the whole of the rare shortfall: one return,
 * three awards, two lost `Ok`s and a descriptor left over. */
static int test_a_promised_descriptor_is_not_promised_again(void) {
    wf_retirement_waiter first;
    wf_retirement_waiter second;
    uint64_t seen;

    /* Four in flight: two refused opens whose attempts were made at the same
     * moment, and two closes that will each return a descriptor. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();

    /* The first refusal is reaped and registers, and the close ends: it is
     * awarded the descriptor and leaves the order to spend it. */
    wf_completion_retirement_wait_begin(&first, seen, NULL, NULL, 0);
    wf_completion_operation_retired(1);
    CHECK(wf_completion_retirement_state(&first) == WF_RETIREMENT_HAPPENED);
    wf_completion_retirement_wait_end(&first);

    /* Now the second refusal is reaped, with the `seen` its own attempt took
     * before any of this.  It opens the order again, and what it may not be
     * told is that a descriptor came back for it: the one that came back is
     * spent.  Its own open is in flight and so is the first waiter's, so this
     * is not the moment it answers either. */
    wf_completion_retirement_wait_begin(&second, seen, NULL, NULL, 0);
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_AWAITED);

    /* A second descriptor comes back, and that one is this waiter's. */
    wf_completion_operation_retired(1);
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_HAPPENED);

    wf_completion_retirement_wait_end(&second);
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    return 0;
}

/* A free that came from outside this runtime, charged to the open that took it,
 * refuses no waiter.
 *
 * No ledger can see a descriptor a thread of the program's own gave back, so an
 * open the host satisfies from one looks exactly like an open that took a
 * return of this runtime's, and the charge falls on it either way.  That can
 * deprive a waiter of an award it was owed, which is the adversarial schedule
 * this stands for — and what it costs that waiter is nothing it is entitled to:
 * being deprived is not being refused.  Raced rather than scripted, on a probe
 * that arranges exactly this against a raw `close` the runtime never made, the
 * charge deprived the waiter in every one of 2,000 repetitions and the waiter
 * published its `Ok` in every one of them.  It keeps waiting while anything is in
 * flight, and its one attempt is then made at the moment nothing is left that
 * could bring a descriptor back, with the return it was deprived of still in
 * the host's table for it to take.  Charging is bounded by the unspent returns
 * for the same reason: past them there is nothing to deprive anyone of. */
static int test_an_outside_free_charged_to_an_open_refuses_no_waiter(void) {
    wf_retirement_waiter waiter;
    uint64_t seen;

    /* Four operations in flight: the refused open this waiter is deciding, a
     * close that returns a descriptor, an open the host satisfies from a
     * descriptor a thread of the program's own gave back, and a read. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&waiter, seen, NULL, NULL, 0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* The close returns one, and the other open takes the outside descriptor.
     * The ledger charges it against the return it can see, so this waiter is
     * deprived — and kept waiting, which is the whole of what happens to it. */
    wf_completion_operation_retired(1);
    wf_completion_retirement_open_took_a_descriptor(0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* That open ends holding its descriptor.  The read is still running and
     * could still give one back, so this is not the moment to answer either. */
    wf_completion_operation_retired(0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_AWAITED);

    /* The read ends returning nothing.  Nothing is left, so the deprived
     * waiter attempts here, with the runtime's own return still unclaimed. */
    wf_completion_operation_retired(0);
    CHECK(wf_completion_retirement_state(&waiter) == WF_RETIREMENT_UNREACHABLE);

    wf_completion_retirement_wait_end(&waiter);
    wf_completion_operation_retired(0);
    return 0;
}

/* An attempt made on an award is not charged again for what the award already
 * spent.
 *
 * The mark moves once per descriptor, and there are two places it can move: the
 * decision that grants an award, and the open that takes a descriptor.  The
 * re-attempt a waiter makes with an award in hand is both of those at once, so
 * it says which it is; charging it twice would take a second return out of the
 * ledger's account and refuse the next waiter the one it is owed. */
static int test_an_awarded_attempt_is_charged_once(void) {
    wf_retirement_waiter first;
    wf_retirement_waiter second;
    uint64_t seen;

    /* Four in flight: two refused opens, and two closes that each return a
     * descriptor — one for each of them. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&first, seen, NULL, NULL, 0);
    wf_completion_retirement_wait_begin(&second, seen, NULL, NULL, 0);

    /* The first close returns one, awarded to the earliest waiter, which
     * leaves the order to spend it. */
    wf_completion_operation_retired(1);
    CHECK(wf_completion_retirement_state(&first) == WF_RETIREMENT_HAPPENED);
    CHECK(first.awarded != 0);
    wf_completion_retirement_wait_end(&first);

    /* The second close returns the other one while that re-attempt is being
     * made, so there is an unspent return for a charge to take. */
    wf_completion_operation_retired(1);

    /* The re-attempt takes the descriptor it was awarded.  The mark moved when
     * the award was granted, so this says so and moves nothing — and the
     * return the second close made is still the second waiter's. */
    wf_completion_retirement_open_took_a_descriptor(first.awarded);
    CHECK(wf_completion_retirement_state(&second) == WF_RETIREMENT_HAPPENED);

    wf_completion_retirement_wait_end(&second);
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    return 0;
}

/* The work a waiter owes is read where the decision is made, never handed to
 * the ledger as a reading taken before it.
 *
 * A refused open on the bounded adapter is answerable for that adapter's
 * queue: the items behind a suspended caller cannot retire while it waits, and
 * the items in front of a thread that will run them are the answer rather than
 * a reason to sleep.  Either way the number is live — a submission lands while
 * the decision is being made — and the decision is made under the lock every
 * wake takes.  A ledger handed the count from before that lock sleeps on a
 * queue that has already grown, and the wake for that growth has already
 * passed: the process stops with a helper asleep and work in the queue, which
 * is what a 200-run TSan sweep at one helper caught once.
 *
 * The schedule point stands between the ledger's two reads, so this test can
 * put the submission exactly there instead of racing for it. */
static int test_the_work_a_waiter_owes_is_read_where_it_decides(void) {
    wf_retirement_waiter waiter;
    uint64_t seen;
    enum wf_retirement_state state;

    /* Three operations in flight: the refused open this waiter is deciding and
     * two queued behind the caller it suspended, which only that caller can
     * run. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    atomic_store_explicit(&wf_harness_owed_work, 2, memory_order_seq_cst);
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(
        &waiter,
        seen,
        wf_harness_owed,
        NULL,
        0
    );

    (void)pthread_mutex_lock(&wf_retirement_point_lock);
    wf_retirement_point_mode = WF_HARNESS_POINT_QUEUES_OWED;
    wf_retirement_point_armed = 1;
    wf_retirement_point_fired = 0;
    (void)pthread_mutex_unlock(&wf_retirement_point_lock);

    state = wf_completion_retirement_state(&waiter);

    (void)pthread_mutex_lock(&wf_retirement_point_lock);
    wf_retirement_point_armed = 0;
    (void)pthread_mutex_unlock(&wf_retirement_point_lock);

    CHECK(wf_retirement_point_fired == 1);
    /* Four in flight now, and all four are this waiter or work stuck behind
     * it: nothing anywhere else can give a descriptor back, so the refusal is
     * the program's outcome.  Read from before the point, the same ledger says
     * three of four, concludes something else is running, and waits for a
     * retirement that nothing will produce. */
    CHECK(state == WF_RETIREMENT_UNREACHABLE);

    wf_completion_retirement_wait_end(&waiter);
    atomic_store_explicit(&wf_harness_owed_work, 0, memory_order_seq_cst);
    /* The refused open and the queue behind it: none of them returned one. */
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    return 0;
}

/* A waiter that runs the work it owes keeps its place, and with it the return
 * that place is owed.
 *
 * A refused open on the bounded adapter runs its own queue before it decides,
 * and an open among that work is one sequential execution runs after it.  A
 * waiter that left the order to run it and registered again on the way back
 * would return behind every open refused meanwhile, and the ledger would hand
 * the descriptor — and with it the program's one `Ok` — to the later call: on a
 * scripted two-waiter shape, to the later one in 200 runs of 200.
 *
 * So running that work costs a waiter its turn to publish and nothing else.
 * The later open takes the refusal sequential execution gives it, which is what
 * the second answer below is, and the descriptor stays where registration order
 * put it. */
static int test_a_waiter_that_stands_aside_keeps_its_claim(void) {
    wf_retirement_waiter first;
    wf_retirement_waiter second;
    uint64_t seen;

    /* Three operations in flight: the two refused opens these waiters are
     * deciding, and one that can still give a descriptor back. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&first, seen, NULL, NULL, 0);
    wf_completion_retirement_wait_begin(&second, seen, NULL, NULL, 0);

    /* The earlier waiter's thread goes into another operation, and the
     * descriptor comes back while it is there. */
    wf_completion_retirement_defer_begin(&first);
    wf_completion_operation_retired(1);
    CHECK(
        wf_completion_retirement_state(&second) == WF_RETIREMENT_UNREACHABLE
    );
    wf_completion_retirement_wait_end(&second);

    /* It comes back to the ledger and the return is still its own. */
    wf_completion_retirement_defer_end(&first);
    CHECK(wf_completion_retirement_state(&first) == WF_RETIREMENT_HAPPENED);
    wf_completion_retirement_wait_end(&first);
    wf_completion_operation_retired(0);
    wf_completion_operation_retired(0);
    return 0;
}

/* A waiter parked on the runtime's endpoint is woken by a second waiter
 * registering.
 *
 * The two places a refused open can sleep are this ledger's condition variable
 * and the runtime's park endpoint, and a transition that reaches one of them
 * stops the process at the other.  This is that transition: one more waiter is
 * one fewer operation that can retire, so it turns the first waiter's answer
 * from "keep waiting" into "publish" — and the first waiter is inside a
 * blocking direct call, parked on the endpoint, where a broadcast on the
 * condition variable is never heard.
 *
 * The schedule is scripted and needs nothing asynchronous from the host, which
 * is what lets it run everywhere the harness runs rather than only where an
 * overlay can be mounted.  The table is narrowed until the host refuses an open
 * outright, one operation is entered in the ledger by hand so that the refused
 * open waits instead of publishing, and the second waiter is registered from
 * this thread once the first is in the order.  A runtime that leaves the
 * endpoint out of that announcement parks the direct call forever; the bound
 * below makes that a failed check rather than a hang. */
struct wf_harness_parked_direct_open {
    const char *path;
    int value;
    int error_code;
    unsigned outcome;
    _Atomic unsigned finished;
};

static void *wf_harness_parked_direct_open_main(void *context) {
    struct wf_harness_parked_direct_open *call = context;
    call->value = wf__completion_file_open_at_direct(
        AT_FDCWD,
        call->path,
        O_RDONLY,
        0u,
        0u,
        WF_FILE_EXPECT_REGULAR,
        &call->error_code,
        &call->outcome
    );
    atomic_store_explicit(&call->finished, 1u, memory_order_seq_cst);
    return NULL;
}

static void wf_harness_parked_direct_open_arm(
    struct wf_harness_parked_direct_open *call,
    const char *path
) {
    call->path = path;
    call->value = 0;
    call->error_code = 0;
    call->outcome = WF_FILE_OPEN_SUCCEEDED;
    atomic_store_explicit(&call->finished, 0u, memory_order_seq_cst);
}

static void wf_harness_pause_a_millisecond(void) {
    struct timespec pause;
    pause.tv_sec = 0;
    pause.tv_nsec = 1000L * 1000L;
    (void)nanosleep(&pause, NULL);
}

/* Up to four seconds for the waiter order to reach this many. */
static void wf_harness_wait_for_waiters(size_t wanted) {
    int attempt;
    for (attempt = 0;
         attempt < 4000 && wf_completion_retirement_waiters() < wanted;
         ++attempt) {
        wf_harness_pause_a_millisecond();
    }
}

/* Long enough that the direct call is asleep in its park rather than on its
 * way there when the transition below is made. */
static void wf_harness_settle(void) {
    struct timespec settle;
    settle.tv_sec = 0;
    settle.tv_nsec = 50L * 1000L * 1000L;
    (void)nanosleep(&settle, NULL);
}

/* Up to eight seconds for the direct call to publish.  A runtime that leaves
 * the endpoint out of the announcement never publishes here, and this bound is
 * what turns that into a failed check rather than a hang. */
static int wf_harness_the_direct_call_published(
    struct wf_harness_parked_direct_open *call
) {
    int attempt;
    for (attempt = 0; attempt < 8000; ++attempt) {
        if (atomic_load_explicit(&call->finished, memory_order_seq_cst)
            != 0u) {
            return 1;
        }
        wf_harness_pause_a_millisecond();
    }
    return 0;
}

/* Narrows the table, runs one of the two schedules below on it, and restores
 * the descriptor and the limit on every way out — the ways a broken runtime
 * takes included, so that a red run reports a failed check and leaves the rest
 * of the harness able to open files. */
static int wf_harness_endpoint_wake_test(
    const char *scratch_directory,
    const char *name,
    int (*schedule)(const char *path)
) {
    struct rlimit saved;
    char path[256];
    int held;
    int writer;
    int outcome;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-%s-%ld",
            scratch_directory,
            name,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* On its own failure paths this helper holds nothing and has restored the
     * limit itself, so there is nothing here to undo. */
    held = wf_harness_hold_the_last_descriptor(path, &saved);
    if (held < 0) {
        (void)unlink(path);
        CHECK(0);
    }
    outcome = schedule(path);
    (void)close(held);
    (void)setrlimit(RLIMIT_NOFILE, &saved);
    (void)unlink(path);
    return outcome;
}

static int wf_harness_registration_wake_schedule(const char *path) {
    struct wf_harness_parked_direct_open call;
    wf_retirement_waiter second;
    pthread_t opener;
    int probe;
    int finished = 0;

    /* The premise: with the table full the host refuses an open outright. */
    probe = open(path, O_RDONLY);
    if (probe >= 0) {
        (void)close(probe);
    }
    CHECK(probe < 0);

    /* The operation that keeps the refused open waiting.  Entered in the
     * ledger by hand, so it belongs to no engine and blocks no thread. */
    wf_completion_operation_accepted();
    wf_harness_parked_direct_open_arm(&call, path);
    if (pthread_create(
            &opener,
            NULL,
            wf_harness_parked_direct_open_main,
            &call
        ) != 0) {
        wf_completion_operation_retired(0);
        CHECK(0);
    }

    /* The first waiter is in the order and has nothing to drive, so it is
     * parked on the endpoint. */
    wf_harness_wait_for_waiters(1u);
    if (wf_completion_retirement_waiters() != 0) {
        wf_harness_settle();
        /* The transition.  Now every operation in flight is a waiter, so the
         * earliest of them publishes — if it is told. */
        wf_completion_retirement_wait_begin(
            &second,
            wf_completion_descriptor_returns(),
            NULL,
            NULL,
            0
        );
        finished = wf_harness_the_direct_call_published(&call);
        wf_completion_retirement_wait_end(&second);
    }
    /* Release the call and take the thread back before anything is checked:
     * it holds a pointer into this frame, and on a red run the retirement
     * below is what lets it answer at all. */
    wf_completion_operation_retired(0);
    (void)pthread_join(opener, NULL);
    CHECK(finished != 0);
    /* And what it published is the refusal, not a descriptor it never had. */
    CHECK(call.value < 0);
    CHECK(call.error_code == EMFILE || call.error_code == ENFILE);
    return 0;
}

static int test_a_registration_wakes_a_waiter_parked_on_the_endpoint(
    const char *scratch_directory
) {
    return wf_harness_endpoint_wake_test(
        scratch_directory,
        "endpoint-wake",
        wf_harness_registration_wake_schedule
    );
}

/* A waiter parked on the runtime's endpoint is woken by an earlier waiter
 * standing aside.
 *
 * The other transition that can hand a parked waiter a terminal answer, and
 * the one the announcement rule was written without.  Standing aside is not
 * leaving: the waiter keeps its place and its claim on a return, and gives up
 * only the duty to publish — which hands that duty to the next waiter in the
 * order, whose answer turns from "keep waiting" into "publish".  That waiter
 * can be one registered long before the aside began, asleep on the endpoint,
 * where the ledger's own condition variable is never heard.
 *
 * Scripted the same way as the test above and discriminating the same way: the
 * direct call must still be parked the moment before the aside, and must
 * publish during it. */
static int wf_harness_standing_aside_wake_schedule(const char *path) {
    struct wf_harness_parked_direct_open call;
    wf_retirement_waiter earlier;
    pthread_t opener;
    int probe;
    int finished = 0;
    int published_before_the_aside = 1;

    probe = open(path, O_RDONLY);
    if (probe >= 0) {
        (void)close(probe);
    }
    CHECK(probe < 0);

    /* The waiter that registers first, and the operation it is deciding —
     * entered by hand, so it belongs to no engine and blocks no thread. */
    wf_completion_operation_accepted();
    wf_completion_retirement_wait_begin(
        &earlier,
        wf_completion_descriptor_returns(),
        NULL,
        NULL,
        0
    );
    wf_harness_parked_direct_open_arm(&call, path);
    if (pthread_create(
            &opener,
            NULL,
            wf_harness_parked_direct_open_main,
            &call
        ) != 0) {
        wf_completion_retirement_wait_end(&earlier);
        wf_completion_operation_retired(0);
        CHECK(0);
    }

    /* The direct call registers behind it and parks: both operations in
     * flight are waiters, so nothing can bring a descriptor back — but the
     * earliest waiter is the other one, so this one keeps waiting. */
    wf_harness_wait_for_waiters(2u);
    if (wf_completion_retirement_waiters() >= 2u) {
        wf_harness_settle();
        published_before_the_aside = atomic_load_explicit(
            &call.finished,
            memory_order_seq_cst
        ) != 0u;
        /* The transition: the earlier waiter's thread goes into another
         * operation, which hands the earliest deciding place — and with it
         * the answer that nothing can bring a descriptor back — to the call
         * parked on the endpoint. */
        wf_completion_retirement_defer_begin(&earlier);
        finished = wf_harness_the_direct_call_published(&call);
        wf_completion_retirement_defer_end(&earlier);
    }
    wf_completion_retirement_wait_end(&earlier);
    wf_completion_operation_retired(0);
    (void)pthread_join(opener, NULL);
    CHECK(published_before_the_aside == 0);
    CHECK(finished != 0);
    CHECK(call.value < 0);
    CHECK(call.error_code == EMFILE || call.error_code == ENFILE);
    return 0;
}

static int test_standing_aside_wakes_a_waiter_parked_on_the_endpoint(
    const char *scratch_directory
) {
    return wf_harness_endpoint_wake_test(
        scratch_directory,
        "aside-wake",
        wf_harness_standing_aside_wake_schedule
    );
}

/* An open registered behind a waiter that stands aside can still answer.
 *
 * This is why standing aside gives up publishing: the operation that waiter is
 * running can be an open that is refused in its turn and registers behind it.
 * A ledger that made that one wait for the earliest waiter in the order would
 * have it waiting for the thread that is waiting for it, and neither ever
 * answers. */
static int test_a_waiter_behind_one_that_stands_aside_can_answer(void) {
    wf_retirement_waiter running;
    wf_retirement_waiter behind;
    uint64_t seen;

    /* Two operations in flight, both of them refused opens: the one whose
     * thread went on to run the other, and the other. */
    wf_completion_operation_accepted();
    wf_completion_operation_accepted();
    seen = wf_completion_descriptor_returns();
    wf_completion_retirement_wait_begin(&running, seen, NULL, NULL, 0);
    wf_completion_retirement_defer_begin(&running);
    wf_completion_retirement_wait_begin(&behind, seen, NULL, NULL, 0);

    CHECK(
        wf_completion_retirement_state(&behind) == WF_RETIREMENT_UNREACHABLE
    );
    wf_completion_retirement_wait_end(&behind);
    wf_completion_operation_retired(0);

    wf_completion_retirement_defer_end(&running);
    CHECK(
        wf_completion_retirement_state(&running) == WF_RETIREMENT_UNREACHABLE
    );
    wf_completion_retirement_wait_end(&running);
    wf_completion_operation_retired(0);
    return 0;
}

/* A refused open must see an operation in flight on the *other* engine.
 *
 * This is the shape a program reaches without writing anything unusual: a read
 * the bounded helper pool carries and an open the target's native ring
 * carries, in flight together.  Source order runs the read, then the close
 * that happens inside it, then the open, and the open succeeds.  A pipeline
 * decides the open's refusal from one engine's own state instead, and the read
 * running on the other one is invisible there: `wf__completion_file_read_submit`
 * has no ring route, so on Linux the read is on the helper adapter while the
 * open is on the ring, and a ring that asks only about itself sees one
 * operation in flight — the refused open — and publishes an `Err` the
 * sequential program never produces.
 *
 * The schedule is fixed rather than raced for.  The read is not merely
 * submitted but parked: it reaches the adapter's readiness wait, which is a
 * named host call, so the test knows it is in flight.  The releasing thread
 * then stands at the moment the runtime holds the refused open — a count of
 * its own — and only then gives the descriptor back and lets the read finish.
 * A runtime that published the refusal instead never reaches that count, and
 * the bound in the releasing thread turns that into a failed check rather than
 * a hang. */
struct wf_harness_parked_read {
    wf_completion_token token;
    int64_t value;
    int error_code;
};

static void *wf_harness_parked_read_main(void *context) {
    struct wf_harness_parked_read *parked = context;
    wf__completion_file_join(
        &parked->token,
        &parked->value,
        &parked->error_code
    );
    return NULL;
}

struct wf_harness_release_after_the_hold {
    int held;
    int pipe_writer;
    uint64_t waits_before;
    unsigned wait_for_the_hold;
    ssize_t released;
    unsigned observed_the_hold;
};

static void *wf_harness_release_after_the_hold_main(void *context) {
    struct wf_harness_release_after_the_hold *release = context;
    int attempt;
    for (attempt = 0;
         release->wait_for_the_hold != 0 && attempt < 4000;
         ++attempt) {
        struct timespec pause;
        if (wf__completion_open_exhaustion_waits() != release->waits_before) {
            release->observed_the_hold = 1;
            break;
        }
        pause.tv_sec = 0;
        pause.tv_nsec = 1000L * 1000L;
        (void)nanosleep(&pause, NULL);
    }
    (void)close(release->held);
    release->released = write(release->pipe_writer, "x", 1);
    return NULL;
}

static int test_bridge_open_waits_for_the_other_engine(
    const char *scratch_directory
) {
    struct wf_harness_parked_read parked;
    struct wf_harness_release_after_the_hold release;
    pthread_t reader;
    pthread_t releaser;
    wf_completion_token token;
    wf_completion_token open_token;
    struct rlimit saved;
    char path[256];
    unsigned char received = 0;
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    uint64_t ring_submissions;
    int pipe_pair[2];
    int writer;
    int held;
    int opened;
    int attempt;
    int the_open_has_an_engine_of_its_own;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-other-engine-bridge-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Warm every descriptor the bridge itself needs before the limit narrows:
     * the ring, its wait set, and any helper the policy asks for are created
     * on the first submission, not here.  The warm open also answers which
     * engine an open takes on this host. */
    ring_submissions = wf__completion_linux_io_uring_submissions();
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0 && error_code == 0);
    /* Whether the schedule this test is about exists at all on this host and
     * helper count.  It needs a second engine: one carrying the read while
     * another attempts the open.  A target whose ring carries opens always has
     * one, and so does a helper pool of any size but exactly one — with a
     * single pinned helper parked in the read, the open has nowhere to run
     * until the read finishes and the runtime is never asked the question.
     * Where it is asked, it must hold; where it is not, the outcome is still
     * the program's. */
    the_open_has_an_engine_of_its_own =
        wf__completion_linux_io_uring_submissions() != ring_submissions
        || wf__completion_target_helper_count() != 1;
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    CHECK(pipe(pipe_pair) == 0);
    CHECK(fcntl(pipe_pair[0], F_SETFL, O_NONBLOCK) == 0);
    (void)pthread_mutex_lock(&wf_poll_gate_lock);
    wf_poll_gate_descriptor = pipe_pair[0];
    wf_poll_gate_entries = 0;
    (void)pthread_mutex_unlock(&wf_poll_gate_lock);

    parked.value = -1;
    parked.error_code = -1;
    CHECK(
        wf__completion_file_read_submit(
            pipe_pair[0],
            &received,
            1u,
            &parked.token
        ) == 1
    );
    CHECK(
        pthread_create(&reader, NULL, wf_harness_parked_read_main, &parked)
        == 0
    );
    /* The read is in flight once it has reached its readiness wait.  With no
     * helper the joining thread above is the engine that takes it there. */
    (void)pthread_mutex_lock(&wf_poll_gate_lock);
    for (attempt = 0; attempt < 4000 && wf_poll_gate_entries == 0; ++attempt) {
        struct timespec pause;
        (void)pthread_mutex_unlock(&wf_poll_gate_lock);
        pause.tv_sec = 0;
        pause.tv_nsec = 1000L * 1000L;
        (void)nanosleep(&pause, NULL);
        (void)pthread_mutex_lock(&wf_poll_gate_lock);
    }
    CHECK(wf_poll_gate_entries != 0);
    (void)pthread_mutex_unlock(&wf_poll_gate_lock);

    held = wf_harness_hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    /* The premise: with the table full the host refuses an open outright. */
    CHECK(open(path, O_RDONLY) < 0);

    release.held = held;
    release.pipe_writer = pipe_pair[1];
    release.waits_before = wf__completion_open_exhaustion_waits();
    release.wait_for_the_hold =
        the_open_has_an_engine_of_its_own != 0 ? 1u : 0u;
    release.released = -1;
    release.observed_the_hold = 0;
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &open_token
        ) == 1
    );
    CHECK(
        pthread_create(
            &releaser,
            NULL,
            wf_harness_release_after_the_hold_main,
            &release
        ) == 0
    );

    value = -1;
    error_code = -1;
    open_outcome = WF_FILE_OPEN_FAILED;
    wf__completion_file_open_join(
        &open_token,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(pthread_join(releaser, NULL) == 0);
    CHECK(pthread_join(reader, NULL) == 0);
    CHECK(release.released == 1);
    /* The runtime held the refused open rather than publishing it, which is
     * the fact the whole rule is about. */
    if (the_open_has_an_engine_of_its_own != 0) {
        CHECK(release.observed_the_hold == 1);
    }
    /* And the outcome the program sees is the one source order produces. */
    CHECK(value >= 0);
    CHECK(error_code == 0);
    CHECK(open_outcome == WF_FILE_OPEN_SUCCEEDED);
    opened = (int)value;
    CHECK(parked.value == 1 && parked.error_code == 0);
    CHECK(received == 'x');

    (void)pthread_mutex_lock(&wf_poll_gate_lock);
    wf_poll_gate_descriptor = -1;
    (void)pthread_mutex_unlock(&wf_poll_gate_lock);
    CHECK(wf__completion_file_close_submit(opened, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    CHECK(close(pipe_pair[0]) == 0);
    CHECK(close(pipe_pair[1]) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

/* One close and two opens, which is the smallest shape a K-slot loop makes.
 *
 * Source order closes the one held descriptor, opens with it, and is refused
 * the second time: exactly one `Ok` and one `Err(ResourceExhausted)`.  A
 * pipeline holds all three at once, so both opens can reach the host while the
 * close is still owed and both can be refused — and both refusals published is
 * an outcome the sequential program never produces.
 *
 * What made that reachable was a refused open running a sibling open out of
 * its own queue and publishing that sibling's refusal with no second attempt
 * at all, while spending its own on a close that had not finished.  Owed work
 * now follows the same rule as the operation that ran it: it may wait for a
 * retirement, and only draining further is denied it, because the caller
 * suspended inside it is the thread that would run the rest of the queue.
 *
 * Repeated, because which thread takes which of the three is the schedule this
 * is about; one repetition would pass by luck at any helper count. */
static int test_bridge_one_of_two_opens_behind_a_close_succeeds(
    const char *scratch_directory
) {
    wf_completion_token close_token;
    wf_completion_token open_tokens[2];
    struct rlimit saved;
    char path[256];
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    int writer;
    int repetition;
    size_t index;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-two-opens-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Warm every descriptor the bridge itself needs before the limit narrows. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &close_token
        ) == 1
    );
    wf__completion_file_open_join(
        &close_token,
        &value,
        &error_code,
        &open_outcome
    );
    CHECK(value >= 0 && error_code == 0);
    CHECK(wf__completion_file_close_submit((int)value, &close_token) == 1);
    wf__completion_file_join(&close_token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    for (repetition = 0; repetition < 16; ++repetition) {
        int opened = -1;
        int successes = 0;
        int refusals = 0;
        int held = wf_harness_hold_the_last_descriptor(path, &saved);
        CHECK(held >= 0);
        CHECK(open(path, O_RDONLY) < 0);

        CHECK(wf__completion_file_close_submit(held, &close_token) == 1);
        for (index = 0; index < 2; ++index) {
            CHECK(
                wf__completion_file_open_at_submit(
                    AT_FDCWD,
                    path,
                    O_RDONLY,
                    0u,
                    0u,
                    WF_FILE_EXPECT_REGULAR,
                    &open_tokens[index]
                ) == 1
            );
        }
        for (index = 0; index < 2; ++index) {
            value = -1;
            error_code = -1;
            open_outcome = 99u;
            wf__completion_file_open_join(
                &open_tokens[index],
                &value,
                &error_code,
                &open_outcome
            );
            if (value >= 0 && error_code == 0
                && open_outcome == WF_FILE_OPEN_SUCCEEDED) {
                successes += 1;
                opened = (int)value;
            } else {
                refusals += 1;
                CHECK(error_code == EMFILE || error_code == ENFILE);
                CHECK(open_outcome == WF_FILE_OPEN_FAILED);
            }
        }
        value = -1;
        error_code = -1;
        wf__completion_file_join(&close_token, &value, &error_code);
        CHECK(value == 0 && error_code == 0);
        CHECK(successes == 1);
        CHECK(refusals == 1);
        CHECK(opened >= 0);
        CHECK(wf__completion_file_close_submit(opened, &close_token) == 1);
        wf__completion_file_join(&close_token, &value, &error_code);
        CHECK(value == 0 && error_code == 0);
        CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    }
    CHECK(unlink(path) == 0);
    return 0;
}

/* Every operation record this runtime has, all holding a refused open at once.
 *
 * The rule holds an open the host refused while something in flight could
 * still give a descriptor back, so a program that fills the whole operation
 * capacity with opens against a full descriptor table makes every record a
 * held one, and the thread that submits the next is waiting for a record to
 * come free. Nothing can retire, so every one of them is the program's own
 * outcome and must be published — the answer source order gives too, since a
 * sequential execution reaches each of these opens with the table still full.
 *
 * What this catches is the shape where waiting does not end. A refused open
 * that runs the work its own adapter owes is not, meanwhile, an engine for the
 * work queued behind it, and one that decided that once and then slept is
 * waiting for an operation only it could have run. This test is the pressure
 * that makes that difference: it queues far more work than there are threads
 * to take it, while every thread is inside the rule.
 */
static int test_bridge_every_record_holding_a_refused_open_publishes(
    const char *scratch_directory
) {
    wf_completion_token tokens[WF_HARNESS_OPERATION_CAPACITY];
    wf_completion_token token;
    struct rlimit saved;
    char path[256];
    int64_t value = -1;
    int error_code = -1;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    size_t submitted = 0;
    size_t index;
    int writer;
    int held;

    CHECK(scratch_directory != NULL);
    CHECK(
        snprintf(
            path,
            sizeof(path),
            "%s/wf-completion-every-record-%ld",
            scratch_directory,
            (long)getpid()
        ) > 0
    );
    (void)unlink(path);
    writer = open(path, O_CREAT | O_EXCL | O_WRONLY, 0600);
    CHECK(writer >= 0);
    CHECK(close(writer) == 0);

    /* Warm every descriptor the bridge itself needs before the limit
     * narrows. */
    CHECK(
        wf__completion_file_open_at_submit(
            AT_FDCWD,
            path,
            O_RDONLY,
            0u,
            0u,
            WF_FILE_EXPECT_REGULAR,
            &token
        ) == 1
    );
    wf__completion_file_open_join(&token, &value, &error_code, &open_outcome);
    CHECK(value >= 0 && error_code == 0);
    CHECK(wf__completion_file_close_submit((int)value, &token) == 1);
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 0 && error_code == 0);

    held = wf_harness_hold_the_last_descriptor(path, &saved);
    CHECK(held >= 0);
    CHECK(open(path, O_RDONLY) < 0);

    while (submitted < WF_HARNESS_OPERATION_CAPACITY) {
        if (wf__completion_file_open_at_submit(
                AT_FDCWD,
                path,
                O_RDONLY,
                0u,
                0u,
                WF_FILE_EXPECT_REGULAR,
                &tokens[submitted]
            ) != 1) {
            break;
        }
        submitted += 1;
    }
    CHECK(submitted > 0);
    for (index = 0; index < submitted; ++index) {
        value = -1;
        error_code = -1;
        open_outcome = 99u;
        wf__completion_file_open_join(
            &tokens[index],
            &value,
            &error_code,
            &open_outcome
        );
        CHECK(value < 0);
        CHECK(error_code == EMFILE || error_code == ENFILE);
        CHECK(open_outcome == WF_FILE_OPEN_FAILED);
    }

    CHECK(close(held) == 0);
    CHECK(setrlimit(RLIMIT_NOFILE, &saved) == 0);
    CHECK(unlink(path) == 0);
    return 0;
}

static int test_native_contract_inventory(void) {
    wf_completion_target_contract darwin = wf_completion_target_contract_for(
        WF_TARGET_DARWIN_FILE_FALLBACK
    );
    wf_completion_target_contract linux = wf_completion_target_contract_for(
        WF_TARGET_LINUX_IO_URING
    );
    wf_completion_target_contract windows = wf_completion_target_contract_for(
        WF_TARGET_WINDOWS_IOCP
    );
    CHECK(darwin.native_completion == 0);
    CHECK(darwin.may_use_blocking_helpers == 1);
    CHECK(darwin.supports_scheduler_progress == 1);
#if defined(__APPLE__)
    CHECK(darwin.implemented == 1);
#else
    CHECK(darwin.implemented == 0);
#endif
#if defined(__linux__)
    CHECK(linux.implemented == 1);
#else
    CHECK(linux.implemented == 0);
#endif
    CHECK(linux.native_completion == 1);
#if defined(_WIN32)
    CHECK(windows.implemented == 1);
#else
    CHECK(windows.implemented == 0);
#endif
    CHECK(windows.native_completion == 1);
    return 0;
}

#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
/* The base-position cell after one progressing attempt: the value the Darwin
 * stub above writes, and the caller's own sentinel on a family whose facility
 * takes no such cell. */
static int64_t wf_expected_position(void) {
#if defined(__APPLE__)
    return 17;
#else
    return 4242;
#endif
}
#endif

/* One admitted enumeration is one progress-producing host attempt, whichever
 * family supplies the facility: interruption repeats the attempt, readiness
 * refusal waits for exactly one POLLIN on the enumerated descriptor and
 * repeats it, and only progress crosses the ABI. The Darwin facility's
 * base-position cell is written on the progressing attempt; Linux's facility
 * has none, and this case asserts the cell is left as the caller gave it. */
static int test_directory_progress_is_internal(void) {
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    int descriptors[2];
    wf_completion_token token;
    unsigned char readiness = 'r';
    unsigned char byte = 0;
    int64_t position = 4242;
    int64_t value = -1;
    int error_code = -1;
    wf_directory_host_calls = 0;
    wf_directory_poll_calls = 0;
    wf_directory_poll_descriptor = -1;
    wf_directory_poll_events = 0;
    wf_directory_poll_timeout = 0;
    CHECK(pipe(descriptors) == 0);
    CHECK(write(descriptors[1], &readiness, 1) == 1);
    CHECK(
        wf__completion_directory_next_direct(
            descriptors[0],
            &byte,
            1,
            &position
        ) == 1
    );
    CHECK(wf_directory_host_calls == 3);
    CHECK(wf_directory_poll_calls == 1);
    CHECK(wf_directory_poll_descriptor == descriptors[0]);
    CHECK(wf_directory_poll_events == POLLIN);
    CHECK(wf_directory_poll_timeout == -1);
    CHECK(byte == 'd');
    CHECK(position == wf_expected_position());

    wf_directory_host_calls = 0;
    wf_directory_poll_calls = 0;
    byte = 0;
    position = 4242;
    CHECK(
        wf__completion_directory_next_submit(
            descriptors[0],
            &byte,
            1,
            &position,
            &token
        ) == 1
    );
    wf__completion_file_join(&token, &value, &error_code);
    CHECK(value == 1 && error_code == 0);
    CHECK(wf_directory_host_calls == 3);
    CHECK(wf_directory_poll_calls == 1);
    CHECK(byte == 'd');
    CHECK(position == wf_expected_position());
    CHECK(close(descriptors[0]) == 0);
    CHECK(close(descriptors[1]) == 0);
#endif
    return 0;
}

static uint64_t monotonic_nanoseconds(void) {
    struct timespec now;
    CHECK(clock_gettime(CLOCK_MONOTONIC, &now) == 0);
    return (uint64_t)now.tv_sec * UINT64_C(1000000000)
        + (uint64_t)now.tv_nsec;
}

static int benchmark_core_roundtrip(uint64_t *nanoseconds_per_operation) {
    enum { ITERATIONS = 100000 };
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_event event;
    uint64_t start;
    uint64_t elapsed;
    int iteration;

    CHECK(wf_completion_runtime_init(&runtime, &slot, 1) == 0);
    start = monotonic_nanoseconds();
    for (iteration = 0; iteration < ITERATIONS; ++iteration) {
        wf_completion_token token;
        wf_completion_publication publication;
        wf_completion_outcome outcome;
        int result = 0;
        CHECK(wf_completion_claim(&runtime, &token) == WF_COMPLETION_CLAIMED);
        CHECK(accept_operation(&runtime, token) == 0);
        publication = integer_publication(&iteration);
        CHECK(
            wf_completion_publish_terminal(&runtime, token, &publication)
            == WF_COMPLETION_PUBLISHED
        );
        CHECK(wf_completion_drain(&runtime, &event, 1, 1) == 1);
        CHECK(
            wf_completion_consume(
                &runtime,
                token,
                &result,
                sizeof(result),
                &outcome
            ) == WF_COMPLETION_CONSUMED
        );
        CHECK(result == iteration);
    }
    elapsed = monotonic_nanoseconds() - start;
    *nanoseconds_per_operation = elapsed / ITERATIONS;
    CHECK(
        wf_completion_statistics_snapshot(&runtime).wake_signals == 0
    );
    CHECK(wf_completion_runtime_destroy(&runtime) == 0);
    return 0;
}

/* The name of the test now running, for the watchdog below.  Written by the
 * main thread and read by a signal handler, which is why it is a plain
 * pointer to a string literal: the store is a single aligned word and the
 * handler only prints it. */
static const char *volatile wf_harness_running = "startup";

/* A deadlock is now a failure mode of this suite, so it gets a name.
 *
 * The rule a refused open follows makes threads wait for each other on
 * purpose, and a mistake in it does not produce a wrong answer — it produces
 * no answer, which without this would surface as a build job that never ends
 * and a log with nothing in it. The bound is three orders of magnitude above
 * what the whole suite takes, so it can only fire on a schedule that has
 * genuinely stopped. */
static void wf_harness_watchdog(int signal_number) {
    static const char message[] = "completion harness: no progress in test: ";
    ssize_t ignored;
    (void)signal_number;
    ignored = write(2, message, sizeof(message) - 1u);
    ignored = write(2, wf_harness_running, strlen(wf_harness_running));
    ignored = write(2, "\n", 1);
    (void)ignored;
    _exit(9);
}

int main(int argc, char **argv) {
    uint64_t roundtrip_ns = 0;
    int trace = getenv("WF_COMPLETION_TRACE") != NULL;
    struct sigaction watchdog;
    memset(&watchdog, 0, sizeof(watchdog));
    watchdog.sa_handler = wf_harness_watchdog;
    if (sigaction(SIGALRM, &watchdog, NULL) != 0) {
        fprintf(stderr, "completion harness: no watchdog\n");
        return 2;
    }
    (void)alarm(300u);
#define RUN_TEST(...)                                                         \
    do {                                                                      \
        wf_harness_running = #__VA_ARGS__;                                    \
        if (trace) {                                                          \
            fprintf(stderr, "completion harness: begin %s\n", #__VA_ARGS__); \
        }                                                                     \
        if ((__VA_ARGS__) != 0) {                                             \
            return 1;                                                         \
        }                                                                     \
        if (trace) {                                                          \
            fprintf(stderr, "completion harness: end %s\n", #__VA_ARGS__);   \
        }                                                                     \
    } while (0)
    if (argc != 2) {
        fprintf(stderr, "usage: %s SCRATCH_DIRECTORY\n", argv[0]);
        return 2;
    }
    /* The bridge cases below assert which route an operation took, and under
     * the demand-driven policy that is the runtime's choice: it declines a
     * positioned read once it has measured this host's reads as not waiting,
     * so whether a case sees the submitted route would depend on what the
     * cases before it happened to execute. A written WF_IO_HELPERS pins the
     * route with the count, so an invocation that did not name one gets one
     * here and every case asserts a route that is fixed.
     *
     * What that leaves untested here is the policy itself, and it is tested
     * where it can be decided rather than sampled:
     * `test_pool_stays_empty_when_operations_do_not_wait` and
     * `test_pool_grows_when_operations_wait` script the clock the policy
     * measures with, and `compiler/tests/programs` runs whole compiled
     * programs under the unset policy. */
    if (getenv("WF_IO_HELPERS") == NULL) {
        (void)setenv("WF_IO_HELPERS", "1", 0);
    }
    RUN_TEST(test_capacity_and_product_state());
    RUN_TEST(test_generation_and_duplicate_terminal());
    RUN_TEST(test_named_drain_refuses_a_recycled_slot());
    RUN_TEST(test_exactly_one_terminal_under_race());
    RUN_TEST(test_concurrent_single_claims_are_unique());
    RUN_TEST(test_unified_wake_epoch());
    RUN_TEST(test_one_epoch_wakes_every_announced_scheduler());
    RUN_TEST(test_drain_wakes_the_registered_token_owner());
    RUN_TEST(test_bounded_drain_and_lane_independence());
    RUN_TEST(test_linux_independent_operations_use_available_target(argv[1]));
    RUN_TEST(test_single_thread_file_progress_and_backpressure(argv[1]));
    RUN_TEST(test_bridge_independent_positioned_reads(argv[1]));
    RUN_TEST(test_bridge_open_status_and_close_are_typed_operations(argv[1]));
    RUN_TEST(test_submitted_open_owns_its_path_bytes(argv[1]));
    RUN_TEST(test_a_name_no_record_can_hold_opens_directly(argv[1]));
    RUN_TEST(test_bridge_capacity_falls_back_per_operation());
    RUN_TEST(test_checked_open_rejects_and_closes_nonregular_descriptors(argv[1]));
    RUN_TEST(test_open_failure_classes_are_typed_outcomes(argv[1]));
    RUN_TEST(test_open_capacity_refuses_and_resubmits(argv[1]));
    RUN_TEST(test_open_results_reach_every_independent_owner(argv[1]));
    RUN_TEST(test_uncached_reads_are_target_policy_only(argv[1]));
    RUN_TEST(test_process_wide_target_helper_budget());
    RUN_TEST(test_pool_stays_empty_when_operations_do_not_wait(argv[1]));
    RUN_TEST(test_pool_grows_when_operations_wait());
    RUN_TEST(test_helper_growth_stops_at_the_helper_storage());
    RUN_TEST(test_helper_count_above_its_storage_is_refused());
    RUN_TEST(test_shutdown_refuses_every_later_entry());
    RUN_TEST(test_helper_completion_wakes_scheduler());
    RUN_TEST(test_readiness_refusal_is_not_a_terminal_outcome());
    RUN_TEST(test_capacity_release_wakes_before_blocking_work());
    RUN_TEST(test_completion_window_answers_at_the_boundaries());
    RUN_TEST(test_a_submitted_operation_is_kicked_before_it_waits(argv[1]));
    RUN_TEST(test_descriptor_return_follows_the_outcome());
    RUN_TEST(test_a_ring_close_counts_a_return_only_when_it_ran(argv[1]));
    RUN_TEST(test_open_exhaustion_retires_owned_work_and_retries(argv[1]));
    RUN_TEST(test_open_exhaustion_waits_for_another_engine(argv[1]));
    RUN_TEST(test_bridge_open_exhaustion_is_retried_once(argv[1]));
    RUN_TEST(test_bridge_open_behind_a_submitted_close_succeeds(argv[1]));
    RUN_TEST(test_a_retirement_between_the_ledger_reads_is_not_missed());
    RUN_TEST(test_an_ending_that_returns_nothing_grants_no_reattempt());
    RUN_TEST(test_a_returned_descriptor_is_awarded_in_waiter_order());
    RUN_TEST(test_an_open_that_took_a_return_is_charged_for_it());
    RUN_TEST(test_a_promised_descriptor_is_not_promised_again());
    RUN_TEST(test_an_outside_free_charged_to_an_open_refuses_no_waiter());
    RUN_TEST(test_an_awarded_attempt_is_charged_once());
    RUN_TEST(test_the_work_a_waiter_owes_is_read_where_it_decides());
    RUN_TEST(test_a_waiter_that_stands_aside_keeps_its_claim());
    RUN_TEST(test_a_waiter_behind_one_that_stands_aside_can_answer());
    RUN_TEST(
        test_a_registration_wakes_a_waiter_parked_on_the_endpoint(argv[1])
    );
    RUN_TEST(
        test_standing_aside_wakes_a_waiter_parked_on_the_endpoint(argv[1])
    );
    RUN_TEST(test_bridge_open_waits_for_the_other_engine(argv[1]));
    RUN_TEST(test_bridge_one_of_two_opens_behind_a_close_succeeds(argv[1]));
    RUN_TEST(test_bridge_every_record_holding_a_refused_open_publishes(argv[1]));
    RUN_TEST(test_native_contract_inventory());
    RUN_TEST(test_directory_progress_is_internal());
    RUN_TEST(benchmark_core_roundtrip(&roundtrip_ns));
#undef RUN_TEST
    printf(
        "completion-core-harness: PASS core_roundtrip_ns=%" PRIu64
        " terminal_racers=%d\n",
        roundtrip_ns,
        TERMINAL_RACERS
    );
    return 0;
}
