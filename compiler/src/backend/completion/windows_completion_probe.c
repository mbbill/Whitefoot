#if defined(_WIN32)

#include "windows_completion.h"
#include "windows_iocp.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define PROBE_CHECK(expression, code)                                         \
    do {                                                                      \
        if (!(expression)) {                                                  \
            return (code);                                                    \
        }                                                                     \
    } while (0)

static unsigned ready_count;
static void *last_ready_frame;

static void record_ready_frame(void *frame) {
    ready_count += 1;
    last_ready_frame = frame;
}

static wf_completion_publication value_publication(uint64_t *value) {
    wf_completion_publication publication;
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = 1;
    publication.result = value;
    publication.result_size = sizeof(*value);
    return publication;
}

static int finish_inline(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint64_t value
) {
    wf_completion_publication publication = value_publication(&value);
    return wf_completion_begin_submit(runtime, token)
                == WF_COMPLETION_TRANSITIONED
            && wf_completion_publish_inline_terminal(
                   runtime,
                   token,
                   &publication
               ) == WF_COMPLETION_PUBLISHED
        ? 0
        : 1;
}

static int test_core_product_and_generation(void) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[3];
    wf_completion_token batch[2];
    wf_completion_token third;
    wf_completion_token replacement;
    wf_completion_token refused[2];
    wf_completion_event events[3];
    wf_completion_publication first_publication;
    wf_completion_publication stale_publication;
    wf_completion_outcome outcome;
    wf_completion_statistics statistics;
    uint64_t first_value = UINT64_C(0x1020304050607080);
    uint64_t stale_value = UINT64_C(0xffffffffffffffff);
    uint64_t consumed = 0;
    int first_frame = 1;
    int second_frame = 2;
    size_t drained;
    size_t index;

    ready_count = 0;
    last_ready_frame = NULL;
    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 3) == 0, 20);
    PROBE_CHECK(
        wf_completion_set_ready_callback(&runtime, record_ready_frame) == 0,
        21
    );
    PROBE_CHECK(
        wf_completion_set_ready_callback(&runtime, record_ready_frame)
            == EBUSY,
        22
    );

    /* A batch is all-or-none. The failed second batch must leave the third
     * slot available to a single claimant. */
    PROBE_CHECK(
        wf_completion_claim_many(&runtime, batch, 2) == WF_COMPLETION_CLAIMED,
        23
    );
    PROBE_CHECK(batch[0].slot != batch[1].slot, 24);
    PROBE_CHECK(
        wf_completion_claim_many(&runtime, refused, 2)
            == WF_COMPLETION_CLAIM_WAIT_CAPACITY,
        25
    );
    PROBE_CHECK(
        wf_completion_claim(&runtime, &third) == WF_COMPLETION_CLAIMED,
        26
    );
    PROBE_CHECK(
        wf_completion_claim(&runtime, &replacement)
            == WF_COMPLETION_CLAIM_WAIT_CAPACITY,
        27
    );

    PROBE_CHECK(
        wf_completion_set_adapter_tag(&runtime, batch[0], 73)
            == WF_COMPLETION_TRANSITIONED,
        28
    );
    PROBE_CHECK(
        wf_completion_depend(
            &runtime,
            batch[0],
            WF_COMPLETION_AUTHORITY_RELEASED,
            &first_frame
        ) == WF_COMPLETION_DEPEND_REGISTERED,
        29
    );
    PROBE_CHECK(
        wf_completion_depend(
            &runtime,
            batch[0],
            WF_COMPLETION_RESULT_READY,
            &second_frame
        ) == WF_COMPLETION_DEPEND_DUPLICATE,
        30
    );
    PROBE_CHECK(
        wf_completion_begin_submit(&runtime, batch[0])
            == WF_COMPLETION_TRANSITIONED
            && wf_completion_target_accepted(&runtime, batch[0])
                == WF_COMPLETION_TRANSITIONED,
        31
    );
    first_publication = value_publication(&first_value);
    first_publication.milestones &= ~WF_COMPLETION_AUTHORITY_RELEASED;
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            batch[0],
            &first_publication
        ) == WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL
            && wf_completion_set_adapter_tag(&runtime, batch[0], 74)
                == WF_COMPLETION_TRANSITION_INVALID_STATE,
        32
    );
    first_publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            batch[0],
            &first_publication
        ) == WF_COMPLETION_PUBLISHED,
        33
    );
    PROBE_CHECK(ready_count == 0 && last_ready_frame == NULL, 134);
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            batch[0],
            &first_publication
        ) == WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL,
        35
    );
    PROBE_CHECK(
        wf_completion_drain(&runtime, events, 1, 3) == 1
            && events[0].token.slot == batch[0].slot
            && events[0].token.generation == batch[0].generation,
        38
    );
    PROBE_CHECK(ready_count == 1 && last_ready_frame == &first_frame, 136);
    PROBE_CHECK(
        wf_completion_depend(
            &runtime,
            batch[0],
            WF_COMPLETION_RESULT_READY,
            &second_frame
        ) == WF_COMPLETION_DEPEND_ALREADY_READY,
        137
    );
    PROBE_CHECK(ready_count == 2 && last_ready_frame == &second_frame, 138);
    PROBE_CHECK(
        wf_completion_consume(
            &runtime,
            batch[0],
            &consumed,
            sizeof(consumed),
            &outcome
        ) == WF_COMPLETION_CONSUMED,
        39
    );
    PROBE_CHECK(
        consumed == first_value
            && outcome.milestones == WF_COMPLETION_OWNERSHIP_COMPLETE
            && outcome.terminal_kind == 1 && outcome.adapter_tag == 73
            && outcome.result_size == sizeof(consumed),
        40
    );

    /* Reuse the exact slot, then prove that its old generation can neither
     * publish bytes nor win a second terminal transition. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &replacement) == WF_COMPLETION_CLAIMED
            && replacement.slot == batch[0].slot
            && replacement.generation == batch[0].generation + 1,
        41
    );
    stale_publication = value_publication(&stale_value);
    PROBE_CHECK(
        wf_completion_publish_terminal(
            &runtime,
            batch[0],
            &stale_publication
        ) == WF_COMPLETION_PUBLISH_STALE,
        42
    );

    PROBE_CHECK(finish_inline(&runtime, batch[1], 2) == 0, 43);
    PROBE_CHECK(finish_inline(&runtime, third, 3) == 0, 44);
    PROBE_CHECK(finish_inline(&runtime, replacement, 4) == 0, 45);
    drained = wf_completion_drain(&runtime, events, 3, 3);
    PROBE_CHECK(drained == 3, 46);
    for (index = 0; index < drained; ++index) {
        consumed = 0;
        PROBE_CHECK(
            wf_completion_consume(
                &runtime,
                events[index].token,
                &consumed,
                sizeof(consumed),
                &outcome
            ) == WF_COMPLETION_CONSUMED,
            47
        );
        PROBE_CHECK(consumed >= 2 && consumed <= 4, 48);
    }
    statistics = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        statistics.claims == 4 && statistics.claim_capacity_waits == 2
            && statistics.publications == 4
            && statistics.stale_publications == 1
            && statistics.duplicate_terminals == 1
            && statistics.consumptions == 4,
        49
    );
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 50);
    return 0;
}

static int drain_file_result(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    enum wf_windows_file_operation_kind kind,
    int64_t expected,
    uint32_t expected_adapter_tag
) {
    wf_completion_event event;
    wf_windows_file_result result;
    wf_completion_outcome outcome;
    if (wf_completion_drain(runtime, &event, 1, runtime->slot_count) != 1
        || event.token.slot != token.slot
        || event.token.generation != token.generation
        || event.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE
        || event.terminal_kind != 1
        || wf_completion_consume(
            runtime,
            token,
            &result,
            sizeof(result),
            &outcome
        ) != WF_COMPLETION_CONSUMED
        || result.kind != kind || result.value != expected
        || result.error_code != 0
        || outcome.milestones != WF_COMPLETION_OWNERSHIP_COMPLETE
        || outcome.terminal_kind != 1
        || outcome.adapter_tag != expected_adapter_tag
        || outcome.result_size != sizeof(result)) {
        return 1;
    }
    return 0;
}

/* The target terminal posts one scheduler wake and freeing the bounded IOCP
 * entry posts one capacity wake. Receiving the real packet followed by both
 * null-OVERLAPPED packets in one call is the regression case where an early
 * return used to erase an already accumulated terminal count. */
static int progress_real_and_trailing_wakes(
    wf_completion_runtime *runtime,
    wf_windows_iocp_adapter *adapter
) {
    enum wf_completion_park_result park;
    uint64_t epoch = wf_completion_wake_epoch(runtime);
    size_t published = SIZE_MAX;
    int error;
    park = wf_windows_completion_iocp_wait_begin(runtime, epoch);
    if (park != WF_COMPLETION_PARK_WOKEN) {
        return 1;
    }
    error = wf_windows_iocp_progress(adapter, 3, INFINITE, &published);
    wf_windows_completion_iocp_wait_end(runtime);
    return error == 0 && published == 1 ? 0 : 1;
}

static int test_real_iocp(const char *path) {
    wf_completion_runtime runtime;
    wf_completion_slot slots[2];
    wf_windows_iocp_adapter adapter;
    wf_windows_iocp_entry entries[1];
    wf_windows_iocp_file file;
    wf_windows_file_request request;
    wf_completion_token first_write;
    wf_completion_token second_write;
    wf_completion_token read_token;
    wf_completion_token eof_token;
    wf_windows_iocp_statistics adapter_statistics;
    wf_completion_statistics core_statistics;
    const unsigned char payload[4] = {'w', 'i', 'n', '!'};
    unsigned char result_bytes[4] = {0};
    size_t published = SIZE_MAX;
    HANDLE handle;
    uint64_t epoch;
    int error;

    PROBE_CHECK(wf_completion_runtime_init(&runtime, slots, 2) == 0, 60);
    PROBE_CHECK(
        wf_windows_iocp_init(&adapter, &runtime, entries, 1, 0) == 0,
        61
    );
    PROBE_CHECK(
        wf_windows_completion_bind_iocp(
            &runtime,
            wf_windows_iocp_port(&adapter),
            wf_windows_iocp_wake_key(&adapter)
        ) == 0,
        62
    );
    handle = CreateFileA(
        path,
        GENERIC_READ | GENERIC_WRITE,
        0,
        NULL,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    PROBE_CHECK(
        handle != INVALID_HANDLE_VALUE
            && wf_windows_iocp_associate_file(&adapter, handle, &file) == 0,
        63
    );

    PROBE_CHECK(
        wf_completion_claim(&runtime, &first_write) == WF_COMPLETION_CLAIMED
            && wf_completion_claim(&runtime, &second_write)
                == WF_COMPLETION_CLAIMED,
        64
    );
    PROBE_CHECK(
        wf_completion_set_adapter_tag(&runtime, first_write, 101)
                == WF_COMPLETION_TRANSITIONED
            && wf_completion_set_adapter_tag(&runtime, second_write, 102)
                == WF_COMPLETION_TRANSITIONED,
        65
    );
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_WRITE_AT;
    request.file = file;
    request.buffer.write_buffer = payload;
    request.count = 2;
    request.offset = 0;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, first_write, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        66
    );
    request.buffer.write_buffer = payload + 2;
    request.offset = 2;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, second_write, &request)
            == WF_WINDOWS_IOCP_WAIT_CAPACITY,
        67
    );
    PROBE_CHECK(
        progress_real_and_trailing_wakes(&runtime, &adapter) == 0,
        68
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            first_write,
            WF_WINDOWS_FILE_WRITE_AT,
            2,
            101
        ) == 0,
        69
    );

    /* Queue a compute wake before the second real operation. The first
     * bounded progress step must consume only that wake; the next one receives
     * the real completion and its trailing publication/capacity wakes. */
    epoch = wf_completion_wake_epoch(&runtime);
    PROBE_CHECK(
        wf_windows_completion_iocp_wait_begin(&runtime, epoch)
            == WF_COMPLETION_PARK_WOKEN,
        70
    );
    wf_completion_notify_compute(&runtime);
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, second_write, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        71
    );
    published = SIZE_MAX;
    error = wf_windows_iocp_progress(&adapter, 1, INFINITE, &published);
    wf_windows_completion_iocp_wait_end(&runtime);
    PROBE_CHECK(error == 0 && published == 0, 72);
    PROBE_CHECK(
        progress_real_and_trailing_wakes(&runtime, &adapter) == 0,
        73
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            second_write,
            WF_WINDOWS_FILE_WRITE_AT,
            2,
            102
        ) == 0,
        74
    );

    PROBE_CHECK(
        wf_completion_claim(&runtime, &read_token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, read_token, 103)
                == WF_COMPLETION_TRANSITIONED,
        75
    );
    request.kind = WF_WINDOWS_FILE_READ_AT;
    request.buffer.read_buffer = result_bytes;
    request.count = sizeof(result_bytes);
    request.offset = 0;
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, read_token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        76
    );
    PROBE_CHECK(
        progress_real_and_trailing_wakes(&runtime, &adapter) == 0,
        77
    );
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            read_token,
            WF_WINDOWS_FILE_READ_AT,
            sizeof(result_bytes),
            103
        ) == 0
            && memcmp(result_bytes, payload, sizeof(payload)) == 0,
        78
    );

    /* Overlapped disk EOF arrives as ERROR_HANDLE_EOF on Windows, but the
     * language file-read contract observes it as successful progress of zero
     * bytes. */
    PROBE_CHECK(
        wf_completion_claim(&runtime, &eof_token) == WF_COMPLETION_CLAIMED
            && wf_completion_set_adapter_tag(&runtime, eof_token, 104)
                == WF_COMPLETION_TRANSITIONED,
        79
    );
    request.buffer.read_buffer = result_bytes;
    request.count = 1;
    request.offset = sizeof(payload);
    PROBE_CHECK(
        wf_windows_iocp_submit(&adapter, eof_token, &request)
            == WF_WINDOWS_IOCP_TARGET_OWNS,
        80
    );
    /* ReadFile may report ERROR_HANDLE_EOF immediately, in which case the
     * adapter has already published and there is deliberately no kernel
     * completion packet to wait for. Otherwise drive the pending request. */
    if (wf_completion_ready_event_count(&runtime) == 0) {
        PROBE_CHECK(
            progress_real_and_trailing_wakes(&runtime, &adapter) == 0,
            81
        );
    }
    PROBE_CHECK(
        drain_file_result(
            &runtime,
            eof_token,
            WF_WINDOWS_FILE_READ_AT,
            0,
            104
        ) == 0,
        82
    );

    /* A null OVERLAPPED is accepted only with the runtime's reserved wake
     * key. The malformed packet is consumed, reported, and publishes no
     * terminal. */
    PROBE_CHECK(
        PostQueuedCompletionStatus(
            wf_windows_iocp_port(&adapter),
            0,
            (ULONG_PTR)1,
            NULL
        ) != FALSE,
        83
    );
    published = SIZE_MAX;
    PROBE_CHECK(
        wf_windows_iocp_progress(&adapter, 1, 0, &published) == EPROTO
            && published == 0,
        84
    );

    adapter_statistics = wf_windows_iocp_statistics_snapshot(&adapter);
    core_statistics = wf_completion_statistics_snapshot(&runtime);
    PROBE_CHECK(
        adapter_statistics.submissions == 4
            && adapter_statistics.capacity_waits == 1
            && adapter_statistics.completions == 4
            && adapter_statistics.publication_failures == 0
            && core_statistics.publications == 4
            && core_statistics.target_capacity_waits == 1
            && core_statistics.compute_notifications == 1
            && wf_windows_iocp_in_flight(&adapter) == 0,
        85
    );
    PROBE_CHECK(wf_windows_iocp_destroy(&adapter) == 0, 86);
    PROBE_CHECK(wf_completion_runtime_destroy(&runtime) == 0, 87);
    PROBE_CHECK(CloseHandle(handle) != FALSE, 88);
    return 0;
}

int main(int argc, char **argv) {
    int error;
    if (argc != 2) {
        return 2;
    }
    error = test_core_product_and_generation();
    if (error != 0) {
        return error;
    }
    error = test_real_iocp(argv[1]);
    if (error != 0) {
        return error;
    }
    printf("windows-native-completion-probe status=pass\n");
    return 0;
}

#else

int main(void) {
    return 77;
}

#endif
