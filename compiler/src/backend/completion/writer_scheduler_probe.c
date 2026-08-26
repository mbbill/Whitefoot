#include "contract.h"
#include "writer_scheduler.h"

#include <pthread.h>
#include <stdalign.h>
#include <stdatomic.h>
#include <stdint.h>
#include <string.h>

typedef union probe_frame {
    max_align_t alignment;
    unsigned char bytes[WF_WRITER_HEADER_STORAGE_BYTES];
} probe_frame;

static pthread_mutex_t probe_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t probe_signal = PTHREAD_COND_INITIALIZER;
static unsigned probe_epoch;
static unsigned allow_resume;

void wf__writer_scheduler_notify(void) {
    pthread_mutex_lock(&probe_lock);
    probe_epoch += 1u;
    pthread_cond_broadcast(&probe_signal);
    pthread_mutex_unlock(&probe_lock);
}

static void resume_and_complete(void *frame) {
    wf__writer_complete(frame);
}

static void *resume_worker(void *unused) {
    (void)unused;
    pthread_mutex_lock(&probe_lock);
    while (allow_resume == 0) {
        pthread_cond_wait(&probe_signal, &probe_lock);
    }
    pthread_mutex_unlock(&probe_lock);
    while (!wf__writer_scheduler_help_once()) {
    }
    return NULL;
}

static int completion_before_suspend_commit(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slot;
    wf_completion_token token;
    wf_completion_token second_token;
    wf_completion_publication publication;
    wf_completion_event event;
    wf_completion_outcome outcome;
    probe_frame frame;
    uint64_t result = 17u;
    uint64_t taken = 0;

    memset(&runtime, 0, sizeof(runtime));
    memset(&slot, 0, sizeof(slot));
    memset(&frame, 0, sizeof(frame));
    if (wf_completion_runtime_init(&runtime, &slot, 1u) != 0) return 10;
    wf__writer_frame_init(&frame);
    wf__writer_begin_suspend(&frame, resume_and_complete);
    if (wf_completion_claim(&runtime, &token) != WF_COMPLETION_CLAIMED) return 11;
    if (wf_completion_set_adapter_tag(&runtime, token, 11u)
        != WF_COMPLETION_TRANSITIONED) return 12;
    if (wf_completion_depend(
            &runtime,
            token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            &frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) return 13;
    if (wf_completion_begin_submit(&runtime, token) != WF_COMPLETION_TRANSITIONED) return 14;
    if (wf_completion_target_accepted(&runtime, token) != WF_COMPLETION_TRANSITIONED) return 15;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = 1u;
    publication.result = &result;
    publication.result_size = sizeof(result);
    if (wf_completion_publish_terminal(&runtime, token, &publication)
        != WF_COMPLETION_PUBLISHED) return 16;
    if (!wf__writer_commit_suspend(&frame)) return 17;
    if (wf__writer_scheduler_help_once()) return 45;
    if (wf_completion_drain(&runtime, &event, 1u, 1u) != 1u) return 18;
    if (!wf__writer_scheduler_help_once()) return 19;
    if (!wf__writer_is_done(&frame)) return 20;
    if (wf_completion_consume(
            &runtime,
            token,
            &taken,
            sizeof(taken),
            &outcome
        ) != WF_COMPLETION_CONSUMED) return 21;
    if (taken != result || outcome.adapter_tag != 11u) return 22;

    if (wf_completion_claim(&runtime, &second_token) != WF_COMPLETION_CLAIMED) return 23;
    if (second_token.slot != token.slot || second_token.generation == token.generation) return 24;
    if (wf_completion_set_adapter_tag(&runtime, second_token, 22u)
        != WF_COMPLETION_TRANSITIONED) return 25;
    if (wf_completion_begin_submit(&runtime, second_token)
        != WF_COMPLETION_TRANSITIONED) return 26;
    if (wf_completion_target_accepted(&runtime, second_token)
        != WF_COMPLETION_TRANSITIONED) return 27;
    if (wf_completion_publish_terminal(&runtime, second_token, &publication)
        != WF_COMPLETION_PUBLISHED) return 28;
    if (wf_completion_drain(&runtime, &event, 1u, 1u) != 1u) return 29;
    if (wf_completion_consume(
            &runtime,
            token,
            &taken,
            sizeof(taken),
            &outcome
        ) != WF_COMPLETION_CONSUME_STALE) return 30;
    if (wf_completion_consume(
            &runtime,
            second_token,
            &taken,
            sizeof(taken),
            &outcome
        ) != WF_COMPLETION_CONSUMED) return 31;
    if (outcome.adapter_tag != 22u) return 32;
    if (wf_completion_runtime_destroy(&runtime) != 0) return 33;
    return 0;
}

static int dependent_frame_waits_for_its_exact_drain(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[17];
    wf_completion_token token;
    wf_completion_publication publication;
    wf_completion_event event;
    wf_completion_outcome outcome;
    probe_frame frame;
    uint64_t result = 71u;
    uint64_t taken = 0;

    memset(&runtime, 0, sizeof(runtime));
    memset(slots, 0, sizeof(slots));
    memset(&frame, 0, sizeof(frame));
    if (wf_completion_runtime_init(&runtime, slots, 17u) != 0) return 50;
    atomic_store_explicit(&runtime.claim_cursor, 16u, memory_order_relaxed);
    wf__writer_frame_init(&frame);
    wf__writer_begin_suspend(&frame, resume_and_complete);
    if (wf_completion_claim(&runtime, &token) != WF_COMPLETION_CLAIMED
        || token.slot != 16u) return 51;
    if (wf_completion_depend(
            &runtime,
            token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            &frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) return 52;
    if (wf_completion_begin_submit(&runtime, token)
            != WF_COMPLETION_TRANSITIONED
        || wf_completion_target_accepted(&runtime, token)
            != WF_COMPLETION_TRANSITIONED) return 53;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = 1u;
    publication.result = &result;
    publication.result_size = sizeof(result);
    if (wf_completion_publish_terminal(&runtime, token, &publication)
        != WF_COMPLETION_PUBLISHED) return 54;
    if (!wf__writer_commit_suspend(&frame)) return 55;

    /* The first bounded scan deliberately stops one slot before this token.
     * Publication alone must not make the frame runnable. */
    if (wf_completion_drain(&runtime, &event, 1u, 16u) != 0u) return 56;
    if (wf__writer_scheduler_help_once()) return 57;
    if (wf_completion_drain(&runtime, &event, 1u, 1u) != 1u
        || event.token.slot != token.slot
        || event.token.generation != token.generation) return 58;
    if (!wf__writer_scheduler_help_once() || !wf__writer_is_done(&frame)) {
        return 59;
    }
    if (wf_completion_consume(
            &runtime,
            token,
            &taken,
            sizeof(taken),
            &outcome
        ) != WF_COMPLETION_CONSUMED
        || taken != result) return 60;
    if (wf_completion_runtime_destroy(&runtime) != 0) return 61;
    return 0;
}

static int migrated_resume_wakes_a_parked_root(void) {
    probe_frame frame;
    pthread_t worker;
    unsigned observed;

    memset(&frame, 0, sizeof(frame));
    allow_resume = 0;
    probe_epoch = 0;
    wf__writer_frame_init(&frame);
    wf__writer_begin_suspend(&frame, resume_and_complete);
    if (!wf__writer_commit_suspend(&frame)) return 40;
    if (pthread_create(&worker, NULL, resume_worker, NULL) != 0) return 41;
    wf__writer_scheduler_ready(&frame);

    pthread_mutex_lock(&probe_lock);
    observed = probe_epoch;
    allow_resume = 1;
    pthread_cond_broadcast(&probe_signal);
    while (!wf__writer_is_done(&frame) && probe_epoch == observed) {
        pthread_cond_wait(&probe_signal, &probe_lock);
    }
    pthread_mutex_unlock(&probe_lock);
    if (pthread_join(worker, NULL) != 0) return 42;
    if (!wf__writer_is_done(&frame)) return 43;
    if (wf__writer_resume_migrations() == 0u) return 44;
    return 0;
}

int main(void) {
    int first = completion_before_suspend_commit();
    int exact;
    if (first != 0) return first;
    exact = dependent_frame_waits_for_its_exact_drain();
    if (exact != 0) return exact;
    return migrated_resume_wakes_a_parked_root();
}
