#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

/*
 * Compiler-owned finite file-completion bridge.
 *
 * The emitted module can submit only the typed read/write descriptors below.
 * There is deliberately no callback, function pointer, or generic thunk in
 * this ABI: a file helper can execute only file_adapter.c's closed switch.
 *
 * A weak LLVM fallback makes an emitted module independently linkable.  When
 * this strong unit is linked, one bounded process runtime supplies stable
 * operation slots before target handoff.  A zero-helper configuration is a
 * real supported mode: the joining scheduler advances the same adapter queue.
 */

#include "contract.h"
#include "bridge.h"
#include "file_adapter.h"
#if defined(__linux__)
#include "linux_io_uring.h"
#endif

#include <errno.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define WF_BRIDGE_SLOT_COUNT WF_FILE_BATCH_MEMBER_CAPACITY
#define WF_BRIDGE_QUEUE_COUNT WF_FILE_BATCH_MEMBER_CAPACITY
#define WF_BRIDGE_MAX_HELPERS 8u
#define WF_BRIDGE_DRAIN_BUDGET 16u

static wf_completion_runtime wf_bridge_runtime;
static wf_completion_slot wf_bridge_slots[WF_BRIDGE_SLOT_COUNT];
static wf_file_adapter wf_bridge_adapter;
static wf_file_work wf_bridge_queue[WF_BRIDGE_QUEUE_COUNT];
static pthread_t wf_bridge_helpers[WF_BRIDGE_MAX_HELPERS];
static pthread_once_t wf_bridge_once = PTHREAD_ONCE_INIT;
static pthread_once_t wf_bridge_file_once = PTHREAD_ONCE_INIT;
static pthread_once_t wf_bridge_ordered_once = PTHREAD_ONCE_INIT;
static pthread_once_t wf_bridge_target_once = PTHREAD_ONCE_INIT;
static int wf_bridge_error;
static int wf_bridge_file_error;
static unsigned wf_bridge_ready;
static unsigned wf_bridge_file_ready;
#if defined(__linux__)
static wf_linux_io_uring_adapter wf_bridge_linux_adapter;
static wf_linux_io_uring_entry wf_bridge_linux_entries[WF_BRIDGE_SLOT_COUNT];
static unsigned wf_bridge_linux_ready;
#endif

enum wf_bridge_route {
    WF_BRIDGE_ROUTE_NONE = 0,
    WF_BRIDGE_ROUTE_FILE_FALLBACK = 1,
    WF_BRIDGE_ROUTE_LINUX_IO_URING = 2,
    WF_BRIDGE_ROUTE_ORDERED_OUTPUT = 3
};



enum wf_ordered_output_root_state {
    WF_ORDERED_OUTPUT_FREE = 0,
    WF_ORDERED_OUTPUT_RESERVING = 1,
    WF_ORDERED_OUTPUT_COMMITTED = 2
};

typedef struct wf_ordered_output_item {
    wf_completion_token token;
    wf_file_request request;
} wf_ordered_output_item;

typedef struct wf_ordered_output_root {
    enum wf_ordered_output_root_state state;
    uint64_t key;
    uint64_t logical_root;
    size_t expected;
    size_t submitted;
    size_t next;
    unsigned executing;
    wf_completion_token reserved[WF_ORDERED_OUTPUT_MEMBER_CAPACITY];
    wf_ordered_output_item items[WF_ORDERED_OUTPUT_MEMBER_CAPACITY];
} wf_ordered_output_root;

typedef struct wf_ordered_output_adapter {
    wf_ordered_output_root roots[WF_ORDERED_OUTPUT_ROOT_CAPACITY];
    pthread_mutex_t lock;
} wf_ordered_output_adapter;

static wf_ordered_output_adapter wf_bridge_ordered;
static unsigned wf_bridge_ordered_ready;
static _Atomic uint64_t wf_bridge_ordered_generation;

static pthread_mutex_t wf_bridge_target_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf_bridge_target_available = PTHREAD_COND_INITIALIZER;
static pthread_t wf_bridge_target_helpers[WF_BRIDGE_MAX_HELPERS];
static size_t wf_bridge_target_helper_count;
static unsigned wf_bridge_target_stopping;
static _Atomic uint64_t wf_bridge_target_epoch;
static _Atomic uint64_t wf_bridge_target_executions;

static int wf_bridge_ensure_ordered(void);
static size_t wf_ordered_output_progress(size_t budget);
static size_t wf_ordered_output_pending(void);
static int wf_ordered_output_shutdown(void);
static int wf_bridge_ensure_target_helpers(void);
static void wf_bridge_notify_target(void);
static void wf_bridge_shutdown_target_helpers(void);

/* Supplied by the compute runtime when one is linked.  The bridge's own weak
 * answer keeps completion-only programs independent of that runtime. */
__attribute__((weak)) int wf__par_help_once(void) {
    return 0;
}

static size_t wf_bridge_helper_count(void) {
    const char *text = getenv("WF_IO_HELPERS");
    char *end = NULL;
    unsigned long parsed;
    if (text == NULL || *text == '\0') {
        return 1u;
    }
    errno = 0;
    parsed = strtoul(text, &end, 10);
    if (errno != 0 || end == text || *end != '\0') {
        return 1u;
    }
    if (parsed > WF_BRIDGE_MAX_HELPERS) {
        return WF_BRIDGE_MAX_HELPERS;
    }
    return (size_t)parsed;
}

static wf_ordered_output_root *wf_ordered_find_locked(uint64_t key) {
    size_t index;
    for (index = 0; index < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++index) {
        if (wf_bridge_ordered.roots[index].state != WF_ORDERED_OUTPUT_FREE
            && wf_bridge_ordered.roots[index].key == key) {
            return &wf_bridge_ordered.roots[index];
        }
    }
    return NULL;
}

static int wf_ordered_take_locked(
    size_t *root_index,
    wf_ordered_output_item *item
) {
    size_t index;
    for (index = 0; index < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++index) {
        wf_ordered_output_root *root = &wf_bridge_ordered.roots[index];
        if (root->state == WF_ORDERED_OUTPUT_COMMITTED
            && root->executing == 0 && root->next < root->expected
            && root->submitted == root->expected) {
            root->executing = 1;
            *root_index = index;
            *item = root->items[root->next];
            return 1;
        }
    }
    return 0;
}

static void wf_ordered_run(
    size_t root_index,
    const wf_ordered_output_item *item
) {
    wf_file_result result;
    wf_completion_publication publication;
    enum wf_completion_publish_result published;
    int final;
    if (item->request.operation.write.count == 0) {
        memset(&result, 0, sizeof(result));
        result.kind = WF_FILE_WRITE;
        result.value = 0;
    } else {
        result = wf_file_execute_direct(&item->request);
    }
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = result.error_code == 0 ? 1u : 2u;
    publication.result = &result;
    publication.result_size = sizeof(result);

    /* The final root retirement must precede terminal publication: that
     * publication may make the caller runnable immediately, and a loop may
     * then reuse the same alloca-derived logical-root address. The opaque
     * batch generation plus this release order closes that ABA window. */
    (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
    {
        wf_ordered_output_root *root =
            &wf_bridge_ordered.roots[root_index];
        if (root->state != WF_ORDERED_OUTPUT_COMMITTED
            || root->executing == 0 || root->next >= root->expected) {
            (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
            abort();
        }
        final = root->next + 1 == root->expected;
        if (final) {
            memset(root, 0, sizeof(*root));
        }
    }
    (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
    if (final) {
        wf_completion_notify_capacity(&wf_bridge_runtime);
    }

    published = wf_completion_publish_terminal(
        &wf_bridge_runtime,
        item->token,
        &publication
    );
    if (published != WF_COMPLETION_PUBLISHED) {
        abort();
    }

    if (!final) {
        (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
        {
            wf_ordered_output_root *root =
                &wf_bridge_ordered.roots[root_index];
            if (root->state != WF_ORDERED_OUTPUT_COMMITTED
                || root->executing == 0 || root->next >= root->expected) {
                (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
                abort();
            }
            root->next += 1;
            root->executing = 0;
        }
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        wf_completion_notify_capacity(&wf_bridge_runtime);
    }
}

static void wf_bridge_initialize_ordered(void) {
    int error;
    memset(&wf_bridge_ordered, 0, sizeof(wf_bridge_ordered));
    atomic_init(&wf_bridge_ordered_generation, 0);
    error = pthread_mutex_init(&wf_bridge_ordered.lock, NULL);
    if (error != 0) {
        return;
    }
    if (!wf_bridge_ensure_target_helpers()) {
        (void)pthread_mutex_destroy(&wf_bridge_ordered.lock);
        memset(&wf_bridge_ordered, 0, sizeof(wf_bridge_ordered));
        return;
    }
    wf_bridge_ordered_ready = 1;
}

static int wf_bridge_ensure_ordered(void) {
    return pthread_once(
               &wf_bridge_ordered_once,
               wf_bridge_initialize_ordered
           ) == 0
        && wf_bridge_ordered_ready != 0;
}

static size_t wf_ordered_output_progress(size_t budget) {
    size_t progressed = 0;
    while (progressed < budget) {
        size_t root_index;
        wf_ordered_output_item item;
        (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
        if (!wf_ordered_take_locked(&root_index, &item)) {
            (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
            break;
        }
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        wf_ordered_run(root_index, &item);
        progressed += 1;
    }
    return progressed;
}

static int wf_bridge_target_progress_one(void) {
    if (wf_bridge_ordered_ready != 0
        && wf_ordered_output_progress(1u) != 0) {
        return 1;
    }
    return wf_bridge_file_ready != 0
        && wf_file_adapter_progress(&wf_bridge_adapter, 1u) != 0;
}

static void *wf_bridge_target_helper_main(void *unused) {
    uint64_t seen = 0;
    (void)unused;
    for (;;) {
        (void)pthread_mutex_lock(&wf_bridge_target_lock);
        while (wf_bridge_target_stopping == 0
               && atomic_load_explicit(
                      &wf_bridge_target_epoch,
                      memory_order_acquire
                  ) == seen) {
            (void)pthread_cond_wait(
                &wf_bridge_target_available,
                &wf_bridge_target_lock
            );
        }
        if (wf_bridge_target_stopping != 0) {
            (void)pthread_mutex_unlock(&wf_bridge_target_lock);
            return NULL;
        }
        seen = atomic_load_explicit(
            &wf_bridge_target_epoch,
            memory_order_acquire
        );
        (void)pthread_mutex_unlock(&wf_bridge_target_lock);
        while (wf_bridge_target_progress_one()) {
            atomic_fetch_add_explicit(
                &wf_bridge_target_executions,
                1,
                memory_order_relaxed
            );
        }
    }
}

static void wf_bridge_initialize_target_helpers(void) {
    size_t requested = wf_bridge_helper_count();
    size_t created = 0;
    atomic_init(&wf_bridge_target_epoch, 0);
    atomic_init(&wf_bridge_target_executions, 0);
    for (created = 0; created < requested; ++created) {
        if (pthread_create(
                &wf_bridge_target_helpers[created],
                NULL,
                wf_bridge_target_helper_main,
                NULL
            ) != 0) {
            break;
        }
    }
    wf_bridge_target_helper_count = created;
}

static int wf_bridge_ensure_target_helpers(void) {
    return pthread_once(
               &wf_bridge_target_once,
               wf_bridge_initialize_target_helpers
           ) == 0;
}

static void wf_bridge_notify_target(void) {
    if (!wf_bridge_ensure_target_helpers()) {
        return;
    }
    (void)pthread_mutex_lock(&wf_bridge_target_lock);
    atomic_fetch_add_explicit(
        &wf_bridge_target_epoch,
        1,
        memory_order_release
    );
    (void)pthread_cond_broadcast(&wf_bridge_target_available);
    (void)pthread_mutex_unlock(&wf_bridge_target_lock);
    /* With zero helpers the sleeping scheduler is the target progress engine.
     * With helpers this is still one unified announcement, and costs no host
     * wake while every scheduler lane remains active. */
    wf_completion_notify_target(&wf_bridge_runtime);
}

static void wf_bridge_shutdown_target_helpers(void) {
    size_t helper;
    (void)pthread_mutex_lock(&wf_bridge_target_lock);
    wf_bridge_target_stopping = 1;
    (void)pthread_cond_broadcast(&wf_bridge_target_available);
    (void)pthread_mutex_unlock(&wf_bridge_target_lock);
    for (helper = 0; helper < wf_bridge_target_helper_count; ++helper) {
        (void)pthread_join(wf_bridge_target_helpers[helper], NULL);
    }
    wf_bridge_target_helper_count = 0;
}

static size_t wf_ordered_output_pending(void) {
    size_t pending = 0;
    size_t index;
    if (wf_bridge_ordered_ready == 0) {
        return 0;
    }
    (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
    for (index = 0; index < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++index) {
        if (wf_bridge_ordered.roots[index].state
            != WF_ORDERED_OUTPUT_FREE) {
            pending += 1;
        }
    }
    (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
    return pending;
}

static int wf_ordered_output_shutdown(void) {
    int first_error = 0;
    if (wf_bridge_ordered_ready == 0 || wf_ordered_output_pending() != 0) {
        return EBUSY;
    }
    {
        int error = pthread_mutex_destroy(&wf_bridge_ordered.lock);
        if (first_error == 0 && error != 0) {
            first_error = error;
        }
    }
    return first_error;
}

static void wf_bridge_initialize_file(void) {
    wf_bridge_file_error = wf_file_adapter_init(
        &wf_bridge_adapter,
        &wf_bridge_runtime,
        wf_bridge_queue,
        WF_BRIDGE_QUEUE_COUNT,
        wf_bridge_helpers,
        0
    );
    if (wf_bridge_file_error == 0 && wf_bridge_ensure_target_helpers()) {
        wf_bridge_file_ready = 1;
    }
}

static int wf_bridge_ensure_file(void) {
    return pthread_once(&wf_bridge_file_once, wf_bridge_initialize_file) == 0
        && wf_bridge_file_ready != 0;
}

static void wf_bridge_shutdown(void) {
    if (wf_bridge_ready == 0) {
        return;
    }
    if (wf_bridge_file_ready != 0 || wf_bridge_ordered_ready != 0) {
        wf_bridge_shutdown_target_helpers();
    }
    if (wf_bridge_file_ready != 0) {
        (void)wf_file_adapter_shutdown(&wf_bridge_adapter);
        wf_bridge_file_ready = 0;
    }
    if (wf_bridge_ordered_ready != 0) {
        (void)wf_ordered_output_shutdown();
        wf_bridge_ordered_ready = 0;
    }
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        (void)wf_linux_io_uring_destroy(&wf_bridge_linux_adapter);
        wf_bridge_linux_ready = 0;
    }
#endif
    (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
    wf_bridge_ready = 0;
}

static void wf_bridge_initialize(void) {
    wf_bridge_error = wf_completion_runtime_init(
        &wf_bridge_runtime,
        wf_bridge_slots,
        WF_BRIDGE_SLOT_COUNT
    );
    if (wf_bridge_error != 0) {
        return;
    }
#if defined(__linux__)
    if (wf_linux_io_uring_init(
            &wf_bridge_linux_adapter,
            &wf_bridge_runtime,
            wf_bridge_linux_entries,
            WF_BRIDGE_SLOT_COUNT
        ) == 0) {
        if (wf_completion_set_wake_callback(
                &wf_bridge_runtime,
                wf_linux_io_uring_notify,
                &wf_bridge_linux_adapter
            ) == 0) {
            wf_bridge_linux_ready = 1;
        } else {
            (void)wf_linux_io_uring_destroy(&wf_bridge_linux_adapter);
        }
    }
#else
    /* Darwin has no regular-file kernel completion facility in the supported
     * target set, so its qualified path is the bounded typed adapter. */
    if (!wf_bridge_ensure_file()) {
        (void)wf_completion_runtime_destroy(&wf_bridge_runtime);
        return;
    }
#endif
    wf_bridge_ready = 1;
    if (atexit(wf_bridge_shutdown) != 0) {
        /* Registration failure changes cleanup at process exit, not the
         * completion contract of any admitted operation. */
    }
}

static size_t wf_bridge_drain(void) {
    wf_completion_event events[WF_BRIDGE_DRAIN_BUDGET];
    return wf_completion_drain(
        &wf_bridge_runtime,
        events,
        WF_BRIDGE_DRAIN_BUDGET,
        WF_BRIDGE_DRAIN_BUDGET
    );
}

/* Progresses both bounded target work and one ready compute offer before a
 * park.  The compute hook never enters file helpers and the file adapter never
 * receives a Whitefoot function pointer. */
static int wf_bridge_progress(void) {
    int progressed = 0;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        size_t published = 0;
        int error = wf_linux_io_uring_progress(
            &wf_bridge_linux_adapter,
            1u,
            0,
            &published
        );
        if (error != 0) {
            /* Target ownership has already transferred. Falling back now
             * would duplicate an operation, and ignoring the error would
             * strand its capability forever. A target-runtime failure is a
             * fail-stop TCB defect, not a writer-visible IoError. */
            abort();
        }
        progressed |= published != 0;
    }
#endif
    /* Native CQ reaping comes first after a kernel wake. It can make a saved
     * writer frame ready before the bounded target/compute passes below. */
    progressed |= wf_bridge_target_progress_one();
    progressed |= wf_bridge_drain() != 0;
    progressed |= wf__par_help_once() != 0;
    return progressed;
}

static void wf_bridge_park(uint64_t observed_epoch) {
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        int error = wf_linux_io_uring_park(
            &wf_bridge_linux_adapter,
            observed_epoch,
            UINT32_MAX
        );
        if (error != 0) {
            abort();
        }
        /* Reap the CQ first, then rerun the bounded target/drain/compute pass.
         * The caller's next loop iteration handles any writer-ready frame. */
        (void)wf_bridge_progress();
        return;
    }
#endif
    {
        enum wf_completion_park_result parked =
            wf_completion_park_if_unchanged(
                &wf_bridge_runtime,
                observed_epoch,
                UINT32_MAX
            );
        if (parked == WF_COMPLETION_PARK_FAILED) {
            abort();
        }
    }
}

void wf__writer_scheduler_notify(void) {
    if (wf_bridge_ready != 0) {
        wf_completion_notify_compute(&wf_bridge_runtime);
    }
}

static int wf_bridge_claim(wf_completion_token *token) {
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        enum wf_completion_claim_result claimed = wf_completion_claim(
            &wf_bridge_runtime,
            token
        );
        if (claimed == WF_COMPLETION_CLAIMED) {
            return 1;
        }
        if (claimed != WF_COMPLETION_CLAIM_WAIT_CAPACITY
            && claimed != WF_COMPLETION_CLAIM_WAIT_ADMISSION) {
            return 0;
        }
        if (!wf_bridge_progress()) {
            if (wf_completion_ready_event_count(&wf_bridge_runtime) == 0
                && (wf_bridge_file_ready == 0
                    || wf_file_adapter_queued(&wf_bridge_adapter) == 0)) {
                wf_bridge_park(epoch);
            }
        }
    }
}

int wf__completion_file_batch_claim(
    void *token_storage,
    uint32_t count,
    uint32_t requires_fallback
) {
    wf_completion_token *tokens = token_storage;
    if (tokens == NULL || count < 2u
        || count > WF_FILE_BATCH_MEMBER_CAPACITY
        || pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0) {
        return 0;
    }
#if defined(__linux__)
    if ((requires_fallback != 0 && !wf_bridge_ensure_file())
        || (requires_fallback == 0 && wf_bridge_linux_ready == 0)) {
        return 0;
    }
#else
    (void)requires_fallback;
    if (!wf_bridge_ensure_file()) {
        return 0;
    }
#endif
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        enum wf_completion_claim_result claimed = wf_completion_claim_many(
            &wf_bridge_runtime,
            tokens,
            (size_t)count
        );
        if (claimed == WF_COMPLETION_CLAIMED) {
            return 1;
        }
        if (claimed != WF_COMPLETION_CLAIM_WAIT_CAPACITY) {
            return 0;
        }
        if (!wf_bridge_progress()) {
            if (wf_completion_ready_event_count(&wf_bridge_runtime) == 0
                && wf_file_adapter_queued(&wf_bridge_adapter) == 0) {
                wf_bridge_park(epoch);
            }
        }
    }
}

static int wf_bridge_file_request_is_empty(const wf_file_request *request) {
    switch (request->kind) {
        case WF_FILE_READ:
            return request->operation.read.count == 0;
        case WF_FILE_WRITE:
            return request->operation.write.count == 0;
        case WF_FILE_PREAD:
            return request->operation.pread.count == 0;
        default:
            return 0;
    }
}

static int wf_bridge_publish_inline_file(
    wf_completion_token token,
    void *dependent_frame,
    enum wf_file_operation_kind kind,
    int64_t value,
    int error_code
) {
    wf_file_result result;
    wf_completion_publication publication;
    memset(&result, 0, sizeof(result));
    result.kind = kind;
    result.value = value;
    result.error_code = error_code;
    if (wf_completion_set_adapter_tag(
            &wf_bridge_runtime,
            token,
            WF_BRIDGE_ROUTE_FILE_FALLBACK
        ) != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_bridge_runtime,
            token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    if (wf_completion_begin_submit(&wf_bridge_runtime, token)
        != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    publication.milestones = WF_COMPLETION_OWNERSHIP_COMPLETE;
    publication.terminal_kind = error_code == 0 ? 1u : 2u;
    publication.result = &result;
    publication.result_size = sizeof(result);
    if (wf_completion_publish_inline_terminal(
            &wf_bridge_runtime,
            token,
            &publication
        ) != WF_COMPLETION_PUBLISHED) {
        abort();
    }
    return 1;
}

static int wf_bridge_submit_file(
    const wf_file_request *request,
    wf_completion_token *token,
    void *dependent_frame,
    int reserved
) {
    if (pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0 || !wf_bridge_ensure_file()) {
        return 0;
    }
    if (reserved == 0 && !wf_bridge_claim(token)) {
        return 0;
    }
    if (wf_bridge_file_request_is_empty(request)) {
        return wf_bridge_publish_inline_file(
            *token,
            dependent_frame,
            request->kind,
            0,
            0
        );
    }
    if (wf_completion_set_adapter_tag(
            &wf_bridge_runtime,
            *token,
            WF_BRIDGE_ROUTE_FILE_FALLBACK
        ) != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_bridge_runtime,
            *token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        enum wf_file_submit_result submitted = wf_file_adapter_submit(
            &wf_bridge_adapter,
            *token,
            request
        );
        if (submitted == WF_FILE_TARGET_OWNS) {
            wf_bridge_notify_target();
            return 1;
        }
        if (submitted != WF_FILE_WAIT_CAPACITY) {
            if (reserved != 0) {
                abort();
            }
            return 0;
        }
        if (!wf_bridge_progress()) {
            wf_bridge_park(epoch);
        }
    }
}

#if defined(__linux__)
/* Returns -1 only while target ownership is still available to the caller, so
 * the caller may honestly choose the bounded POSIX adapter. */
static int wf_bridge_submit_linux_pread(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    wf_completion_token *token,
    void *dependent_frame,
    int reserved
) {
    wf_linux_file_request request;
    if (pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0 || wf_bridge_linux_ready == 0
        || count == 0 || count > UINT32_MAX) {
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_LINUX_FILE_READ_AT;
    request.descriptor = descriptor;
    request.buffer.read_buffer = buffer;
    request.count = (size_t)count;
    request.offset = file_offset;
    if (reserved == 0 && !wf_bridge_claim(token)) {
        return 0;
    }
    if (wf_completion_set_adapter_tag(
            &wf_bridge_runtime,
            *token,
            WF_BRIDGE_ROUTE_LINUX_IO_URING
        ) != WF_COMPLETION_TRANSITIONED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_bridge_runtime,
            *token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        enum wf_linux_io_uring_submit_result submitted =
            wf_linux_io_uring_submit(
                &wf_bridge_linux_adapter,
                *token,
                &request
            );
        if (submitted == WF_LINUX_IO_URING_TARGET_OWNS) {
            if (wf_linux_io_uring_progress_error(
                    &wf_bridge_linux_adapter
                ) != 0) {
                abort();
            }
            return 1;
        }
        if (submitted != WF_LINUX_IO_URING_WAIT_CAPACITY) {
            /* `linux_ready` and the request checks above close every honest
             * pre-ownership fallback. Reaching another result after claiming
             * the bundle is an internal adapter/core contract defect. */
            abort();
        }
        if (!wf_bridge_progress()) {
            wf_bridge_park(epoch);
        }
    }
}
#endif

int wf__completion_file_read_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || (buffer == NULL && count != 0)) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_READ;
    request.operation.read.descriptor = descriptor;
    request.operation.read.buffer = buffer;
    request.operation.read.count = (size_t)count;
    if ((uint64_t)request.operation.read.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL,
        0
    );
}

int wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || (buffer == NULL && count != 0)
        || file_offset > (uint64_t)INT64_MAX) {
        return 0;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            file_offset,
            (wf_completion_token *)token_storage,
            NULL,
            0
        );
        if (native >= 0) {
            return native;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = (int64_t)file_offset;
    if ((uint64_t)request.operation.pread.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL,
        0
    );
}

int wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_file_request request;
    if (token_storage == NULL || (buffer == NULL && count != 0)) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count) {
        return 0;
    }
    /* write_once is an unpositioned Output operation. The native adapter's
     * WRITE_AT request fixes an explicit offset, which would change regular
     * file-offset semantics and is not a meaningful stream offset. Until a
     * Linux request kind is qualified for exact write(2) current-position and
     * append/stream behavior, keep this on the bounded typed fallback. */
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        NULL,
        0
    );
}

void wf__completion_file_pread_submit_reserved(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage
) {
    wf_completion_token *token = token_storage;
    wf_file_request request;
    if (token == NULL || (buffer == NULL && count != 0)) {
        abort();
    }
    if (count == 0) {
        (void)wf_bridge_publish_inline_file(
            *token,
            NULL,
            WF_FILE_PREAD,
            0,
            0
        );
        return;
    }
    if (file_offset > (uint64_t)INT64_MAX) {
        (void)wf_bridge_publish_inline_file(
            *token,
            NULL,
            WF_FILE_PREAD,
            -1,
            EINVAL
        );
        return;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            file_offset,
            token,
            NULL,
            1
        );
        if (native >= 0) {
            if (native != 1) {
                abort();
            }
            return;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = (int64_t)file_offset;
    if ((uint64_t)request.operation.pread.count != count
        || !wf_bridge_submit_file(&request, token, NULL, 1)) {
        abort();
    }
}

void wf__completion_file_write_submit_reserved(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_completion_token *token = token_storage;
    wf_file_request request;
    if (token == NULL || (buffer == NULL && count != 0)) {
        abort();
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count
        || !wf_bridge_submit_file(&request, token, NULL, 1)) {
        abort();
    }
}

int wf__completion_file_pread_submit_writer(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage,
    void *frame
) {
    wf_file_request request;
    if (token_storage == NULL || frame == NULL
        || (buffer == NULL && count != 0)
        || file_offset > (uint64_t)INT64_MAX) {
        return 0;
    }
#if defined(__linux__)
    {
        int native = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            file_offset,
            (wf_completion_token *)token_storage,
            frame,
            0
        );
        if (native >= 0) {
            return native;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = (int64_t)file_offset;
    if ((uint64_t)request.operation.pread.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        frame,
        0
    );
}

int wf__completion_file_write_submit_writer(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage,
    void *frame
) {
    wf_file_request request;
    if (token_storage == NULL || frame == NULL
        || (buffer == NULL && count != 0)) {
        return 0;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count) {
        return 0;
    }
    return wf_bridge_submit_file(
        &request,
        (wf_completion_token *)token_storage,
        frame,
        0
    );
}

static int64_t wf_bridge_return_direct(wf_file_result result) {
    if (result.value < 0) {
        errno = result.error_code;
    }
    return result.value;
}

int64_t wf__completion_file_pread_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t file_offset
) {
    wf_file_request request;
#if defined(__linux__)
    if (file_offset >= 0) {
        wf_completion_token token;
        int submitted = wf_bridge_submit_linux_pread(
            descriptor,
            buffer,
            count,
            (uint64_t)file_offset,
            &token,
            NULL,
            0
        );
        if (submitted == 1) {
            int64_t value;
            int error_code;
            wf__completion_file_join(&token, &value, &error_code);
            if (value < 0) {
                errno = error_code;
            }
            return value;
        }
    }
#endif
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_PREAD;
    request.operation.pread.descriptor = descriptor;
    request.operation.pread.buffer = buffer;
    request.operation.pread.count = (size_t)count;
    request.operation.pread.offset = file_offset;
    if ((uint64_t)request.operation.pread.count != count) {
        errno = EINVAL;
        return -1;
    }
    return wf_bridge_return_direct(wf_file_execute_direct(&request));
}

int64_t wf__completion_file_write_direct(
    int descriptor,
    const void *buffer,
    uint64_t count
) {
    wf_file_request request;
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;
    if ((uint64_t)request.operation.write.count != count) {
        errno = EINVAL;
        return -1;
    }
    return wf_bridge_return_direct(wf_file_execute_direct(&request));
}

int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
) {
#if defined(__APPLE__)
    wf_file_request request;
    if ((buffer == NULL && count != 0) || position == NULL
        || (uint64_t)(size_t)count != count) {
        errno = EINVAL;
        return -1;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_GETDIRENTRIES64;
    request.operation.getdirentries64.descriptor = descriptor;
    request.operation.getdirentries64.buffer = buffer;
    request.operation.getdirentries64.count = (size_t)count;
    request.operation.getdirentries64.position = position;
    return wf_bridge_return_direct(wf_file_execute_direct(&request));
#else
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)position;
    errno = ENOTSUP;
    return -1;
#endif
}

uint64_t wf__completion_output_batch_begin(
    uint64_t logical_root,
    uint32_t expected
) {
    if (logical_root == 0 || expected < 2
        || expected > WF_ORDERED_OUTPUT_MEMBER_CAPACITY
        || pthread_once(&wf_bridge_once, wf_bridge_initialize) != 0
        || wf_bridge_ready == 0 || !wf_bridge_ensure_ordered()) {
        return 0;
    }
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        size_t index;
        (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
        for (index = 0; index < WF_ORDERED_OUTPUT_ROOT_CAPACITY; ++index) {
            wf_ordered_output_root *root =
                &wf_bridge_ordered.roots[index];
            if (root->state == WF_ORDERED_OUTPUT_FREE) {
                memset(root, 0, sizeof(*root));
                if (wf_completion_claim_many(
                        &wf_bridge_runtime,
                        root->reserved,
                        expected
                    ) == WF_COMPLETION_CLAIMED) {
                    uint64_t generation = atomic_fetch_add_explicit(
                        &wf_bridge_ordered_generation,
                        1,
                        memory_order_relaxed
                    ) + 1u;
                    if (generation == 0) {
                        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
                        abort();
                    }
                    root->state = WF_ORDERED_OUTPUT_RESERVING;
                    root->key = generation;
                    root->logical_root = logical_root;
                    root->expected = expected;
                    (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
                    return generation;
                }
                memset(root, 0, sizeof(*root));
                break;
            }
        }
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        if (!wf_bridge_progress()) {
            wf_bridge_park(epoch);
        }
    }
}

void wf__completion_output_batch_submit(
    uint64_t root_key,
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_completion_token *token = token_storage;
    wf_file_request request;
    wf_ordered_output_root *root;
    enum wf_completion_transition_result transition;
    if (token == NULL || (buffer == NULL && count != 0)
        || (uint64_t)(size_t)count != count) {
        abort();
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = (size_t)count;

    (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
    root = wf_ordered_find_locked(root_key);
    if (root == NULL || root->state != WF_ORDERED_OUTPUT_RESERVING
        || root->submitted >= root->expected) {
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        abort();
    }
    *token = root->reserved[root->submitted];
    transition = wf_completion_set_adapter_tag(
        &wf_bridge_runtime,
        *token,
        WF_BRIDGE_ROUTE_ORDERED_OUTPUT
    );
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        abort();
    }
    transition = wf_completion_begin_submit(&wf_bridge_runtime, *token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        abort();
    }
    transition = wf_completion_target_accepted(&wf_bridge_runtime, *token);
    if (transition != WF_COMPLETION_TRANSITIONED) {
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        abort();
    }
    root->items[root->submitted].token = *token;
    root->items[root->submitted].request = request;
    root->submitted += 1;
    (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
}

void wf__completion_output_batch_commit(uint64_t root_key) {
    wf_ordered_output_root *root;
    (void)pthread_mutex_lock(&wf_bridge_ordered.lock);
    root = wf_ordered_find_locked(root_key);
    if (root == NULL || root->state != WF_ORDERED_OUTPUT_RESERVING
        || root->submitted != root->expected) {
        (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
        abort();
    }
    root->state = WF_ORDERED_OUTPUT_COMMITTED;
    (void)pthread_mutex_unlock(&wf_bridge_ordered.lock);
    wf_bridge_notify_target();
}

int wf__completion_file_take(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    wf_completion_token token;
    enum wf_bridge_route route;
    union {
        wf_file_result file;
#if defined(__linux__)
        wf_linux_file_result linux;
#endif
        max_align_t alignment;
        unsigned char bytes[WF_COMPLETION_RESULT_CAPACITY];
    } result;
    wf_completion_outcome outcome;
    enum wf_completion_consume_result consumed;
    if (token_storage == NULL || value == NULL || error_code == NULL) {
        abort();
    }
    token = *(const wf_completion_token *)token_storage;
    (void)wf_bridge_drain();
    consumed = wf_completion_consume(
        &wf_bridge_runtime,
        token,
        &result,
        sizeof(result),
        &outcome
    );
    if (consumed == WF_COMPLETION_CONSUME_NOT_TERMINAL
        || consumed == WF_COMPLETION_CONSUME_NOT_DRAINED) {
        return 0;
    }
    if (consumed != WF_COMPLETION_CONSUMED) {
        abort();
    }
    route = (enum wf_bridge_route)outcome.adapter_tag;
    if (route == WF_BRIDGE_ROUTE_FILE_FALLBACK
        || route == WF_BRIDGE_ROUTE_ORDERED_OUTPUT) {
        if (outcome.result_size != sizeof(result.file)) {
            abort();
        }
        *value = result.file.value;
        *error_code = result.file.error_code;
#if defined(__linux__)
    } else if (route == WF_BRIDGE_ROUTE_LINUX_IO_URING) {
        if (outcome.result_size != sizeof(result.linux)) {
            abort();
        }
        *value = result.linux.value;
        *error_code = result.linux.error_code;
#endif
    } else {
        abort();
    }
    return 1;
}

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    for (;;) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        if (wf__completion_file_take(token_storage, value, error_code)) {
            return;
        }
        if (!wf_bridge_progress()) {
            if (wf_bridge_drain() != 0) {
                continue;
            }
            if (wf_completion_ready_event_count(&wf_bridge_runtime) == 0
                && (wf_bridge_file_ready == 0
                    || wf_file_adapter_queued(&wf_bridge_adapter) == 0)) {
                enum wf_completion_consume_wait_result wait =
                    wf_completion_wait_to_consume(
                        &wf_bridge_runtime,
                        *(const wf_completion_token *)token_storage
                    );
                if (wait == WF_COMPLETION_CONSUME_READY) {
                    continue;
                }
                if (wait != WF_COMPLETION_CONSUME_WAIT_REGISTERED) {
                    abort();
                }
                wf_bridge_park(epoch);
            }
        }
    }
}
void wf__writer_run_root(void *frame) {
    while (!wf__writer_is_done(frame)) {
        uint64_t epoch = wf_completion_wake_epoch(&wf_bridge_runtime);
        int progressed = wf_bridge_progress();
        progressed |= wf__writer_scheduler_help_once();
        if (!progressed && !wf__writer_is_done(frame)) {
            /* A bounded drain can stop before another ready slot. Keep
             * harvesting while the durable count says such an event exists;
             * drain-to-writer enqueue advances the epoch and closes the final
             * ready-count-to-park race. */
            if (wf_completion_ready_event_count(&wf_bridge_runtime) != 0) {
                continue;
            }
            wf_bridge_park(epoch);
        }
    }
}

uint64_t wf__completion_file_submissions(void) {
    uint64_t submissions = wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter).submissions;
#if defined(__linux__)
    if (wf_bridge_linux_ready != 0) {
        submissions += wf_linux_io_uring_statistics_snapshot(
            &wf_bridge_linux_adapter
        ).submissions;
    }
#endif
    return submissions;
}

uint64_t wf__completion_file_fallback_submissions(void) {
    return wf_bridge_file_ready == 0
        ? 0
        : wf_file_adapter_statistics_snapshot(&wf_bridge_adapter).submissions;
}

uint64_t wf__completion_file_helper_executions(void) {
    return atomic_load_explicit(
        &wf_bridge_target_executions,
        memory_order_relaxed
    );
}

uint64_t wf__completion_target_helper_count(void) {
    return wf_bridge_target_helper_count;
}

uint64_t wf__completion_target_helper_executions(void) {
    return atomic_load_explicit(
        &wf_bridge_target_executions,
        memory_order_relaxed
    );
}

uint64_t wf__completion_publications(void) {
    return wf_completion_statistics_snapshot(&wf_bridge_runtime).publications;
}

uint64_t wf__completion_linux_io_uring_submissions(void) {
#if defined(__linux__)
    return wf_bridge_linux_ready == 0
        ? 0
        : wf_linux_io_uring_statistics_snapshot(&wf_bridge_linux_adapter)
            .submissions;
#else
    return 0;
#endif
}
