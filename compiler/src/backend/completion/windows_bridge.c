#include "bridge.h"
#include "windows_completion.h"
#include "windows_iocp.h"

#if defined(_WIN32)

#include <fcntl.h>
#include <io.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#if defined(WF_WINDOWS_COMPILER_CAPACITY_PROBE)
/* Native CI compiles one observed executable with two slots so four source
 * reads exercise compiler-emitted status-2 recovery. Production links never
 * define this probe marker and retain the fixed 64-slot runtime. */
#define WF_WINDOWS_BRIDGE_CAPACITY 2u
#else
#define WF_WINDOWS_BRIDGE_CAPACITY 64u
#endif
#define WF_WINDOWS_BRIDGE_DRAIN_BUDGET 16u
#define WF_WINDOWS_BRIDGE_WINDOW_BUDGET (4u * 1024u * 1024u)

/* writer_scheduler_windows.c supplies a completion-only default; the native
 * compute runtime overrides it with its unified work-first helper. */
extern int wf__par_help_once(void);

static wf_completion_runtime wf_windows_bridge_runtime;
static wf_completion_slot wf_windows_bridge_slots[WF_WINDOWS_BRIDGE_CAPACITY];
static wf_windows_iocp_adapter wf_windows_bridge_adapter;
static wf_windows_iocp_entry wf_windows_bridge_entries[WF_WINDOWS_BRIDGE_CAPACITY];
static INIT_ONCE wf_windows_bridge_once = INIT_ONCE_STATIC_INIT;
static _Atomic unsigned wf_windows_bridge_ready;
static _Atomic uint64_t wf_windows_bridge_submissions;
static _Atomic uint64_t wf_windows_bridge_publications;
static int wf_windows_bridge_error;
static unsigned wf_windows_bridge_require_iocp;

typedef struct wf_windows_bridge_file_entry {
    wf_windows_iocp_file file;
    unsigned present;
} wf_windows_bridge_file_entry;

static SRWLOCK wf_windows_bridge_files_lock = SRWLOCK_INIT;
static wf_windows_bridge_file_entry *wf_windows_bridge_files;
static size_t wf_windows_bridge_file_capacity;

static void wf_windows_bridge_verify_required_iocp(void) {
    uint64_t submissions;
    uint64_t publications;
    if (wf_windows_bridge_require_iocp == 0u) {
        return;
    }
    submissions = atomic_load_explicit(
        &wf_windows_bridge_submissions,
        memory_order_relaxed
    );
    publications = atomic_load_explicit(
        &wf_windows_bridge_publications,
        memory_order_relaxed
    );
    if (submissions == 0 || publications != submissions) {
        abort();
    }
}

static BOOL CALLBACK wf_windows_bridge_initialize(
    PINIT_ONCE once,
    PVOID parameter,
    PVOID *context
) {
    int error;
    WCHAR required[2];
    DWORD required_length;
    (void)once;
    (void)parameter;
    (void)context;
    error = wf_completion_runtime_init(
        &wf_windows_bridge_runtime,
        wf_windows_bridge_slots,
        WF_WINDOWS_BRIDGE_CAPACITY
    );
    if (error != 0) {
        abort();
    }
    error = wf_completion_set_ready_callback(
        &wf_windows_bridge_runtime,
        wf__writer_scheduler_ready
    );
    if (error != 0) {
        abort();
    }
    /* With the fixed non-null storage above, the only recoverable failure
     * from iocp_init is CreateIoCompletionPort's Win32 error. */
    error = wf_windows_iocp_init(
        &wf_windows_bridge_adapter,
        &wf_windows_bridge_runtime,
        wf_windows_bridge_entries,
        WF_WINDOWS_BRIDGE_CAPACITY,
        0
    );
    if (error == 0) {
        error = wf_windows_completion_bind_iocp(
            &wf_windows_bridge_runtime,
            &wf_windows_bridge_adapter,
            wf_windows_iocp_port(&wf_windows_bridge_adapter),
            wf_windows_iocp_wake_key(&wf_windows_bridge_adapter)
        );
        /* Fresh, compiler-owned objects make every documented nonzero bind
         * result an internal invariant violation, not a host error. */
        if (error != 0) {
            abort();
        }
    }
    required_length = GetEnvironmentVariableW(
        L"WF_REQUIRE_WINDOWS_IOCP",
        required,
        (DWORD)(sizeof(required) / sizeof(required[0]))
    );
    if (error == 0 && required_length == 1u && required[0] == L'1') {
        wf_windows_bridge_require_iocp = 1u;
        if (atexit(wf_windows_bridge_verify_required_iocp) != 0) {
            error = ERROR_NOT_ENOUGH_MEMORY;
        }
    }
    wf_windows_bridge_error = error;
    if (error == 0) {
        atomic_store_explicit(
            &wf_windows_bridge_ready,
            1u,
            memory_order_release
        );
    }
    return TRUE;
}

static int wf_windows_bridge_start(void) {
    if (InitOnceExecuteOnce(
            &wf_windows_bridge_once,
            wf_windows_bridge_initialize,
            NULL,
            NULL
        ) == FALSE) {
        DWORD error = GetLastError();
        return error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : (int)error;
    }
    return atomic_load_explicit(
        &wf_windows_bridge_ready,
        memory_order_acquire
    ) != 0u
        ? 0
        : (wf_windows_bridge_error == 0
               ? ERROR_GEN_FAILURE
               : wf_windows_bridge_error);
}

/* A regular-file descriptor is associated exactly once, immediately after
 * the runtime creates it.  The descriptor-indexed entry retains the adapter's
 * minted capability for the handle lifetime, so submit never manufactures a
 * capability or repeats CreateIoCompletionPort association. */
int wf__windows_completion_associate_descriptor(int descriptor) {
    intptr_t native;
    wf_windows_iocp_file associated;
    wf_windows_bridge_file_entry *grown;
    size_t required;
    size_t capacity;
    int error = wf_windows_bridge_start();
    if (error != 0) {
        return error;
    }
    native = _get_osfhandle(descriptor);
    if (native == -1) {
        return ERROR_INVALID_HANDLE;
    }
    memset(&associated, 0, sizeof(associated));
    error = wf_windows_iocp_associate_file(
        &wf_windows_bridge_adapter,
        (HANDLE)native,
        &associated
    );
    if (error != 0) {
        return error;
    }
    required = (size_t)descriptor + 1u;
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    if (required > wf_windows_bridge_file_capacity) {
        capacity = wf_windows_bridge_file_capacity == 0
            ? 64u
            : wf_windows_bridge_file_capacity;
        while (capacity < required && capacity <= SIZE_MAX / 2u) {
            capacity *= 2u;
        }
        if (capacity < required
            || capacity > SIZE_MAX / sizeof(*wf_windows_bridge_files)) {
            ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        if (wf_windows_bridge_files == NULL) {
            grown = (wf_windows_bridge_file_entry *)HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                capacity * sizeof(*wf_windows_bridge_files)
            );
        } else {
            grown = (wf_windows_bridge_file_entry *)HeapReAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                wf_windows_bridge_files,
                capacity * sizeof(*wf_windows_bridge_files)
            );
        }
        if (grown == NULL) {
            ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        wf_windows_bridge_files = grown;
        wf_windows_bridge_file_capacity = capacity;
    }
    wf_windows_bridge_files[descriptor].file = associated;
    wf_windows_bridge_files[descriptor].present = 1u;
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
    return 0;
}

void wf__windows_completion_forget_descriptor(int descriptor) {
    if (descriptor < 0) {
        return;
    }
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    if ((size_t)descriptor < wf_windows_bridge_file_capacity) {
        memset(
            &wf_windows_bridge_files[descriptor],
            0,
            sizeof(wf_windows_bridge_files[descriptor])
        );
    }
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
}

static int wf_windows_bridge_file_for_descriptor(
    int descriptor,
    HANDLE handle,
    wf_windows_iocp_file *file
) {
    int found = 0;
    if (descriptor < 0 || file == NULL) {
        return 0;
    }
    AcquireSRWLockShared(&wf_windows_bridge_files_lock);
    if ((size_t)descriptor < wf_windows_bridge_file_capacity
        && wf_windows_bridge_files[descriptor].present != 0u
        && wf_windows_bridge_files[descriptor].file.handle == handle
        && wf_windows_bridge_files[descriptor].file.adapter
            == &wf_windows_bridge_adapter) {
        *file = wf_windows_bridge_files[descriptor].file;
        found = 1;
    }
    ReleaseSRWLockShared(&wf_windows_bridge_files_lock);
    return found;
}

static size_t wf_windows_bridge_drain(void) {
    wf_completion_event events[WF_WINDOWS_BRIDGE_DRAIN_BUDGET];
    size_t total = 0;
    for (;;) {
        size_t drained = wf_completion_drain(
            &wf_windows_bridge_runtime,
            events,
            WF_WINDOWS_BRIDGE_DRAIN_BUDGET,
            WF_WINDOWS_BRIDGE_CAPACITY
        );
        total += drained;
        if (drained < WF_WINDOWS_BRIDGE_DRAIN_BUDGET) {
            return total;
        }
    }
}

static int wf_windows_bridge_progress(
    uint64_t observed,
    DWORD timeout
) {
    enum wf_completion_park_result announced =
        wf_windows_completion_iocp_wait_begin(
            &wf_windows_bridge_runtime,
            observed
        );
    size_t published = 0;
    int error;
    if (announced == WF_COMPLETION_PARK_EPOCH_CHANGED) {
        return 1;
    }
    if (announced != WF_COMPLETION_PARK_WOKEN) {
        abort();
    }
    /* The bound adapter withdraws this announcement on its first GQCS
     * result, including a timeout.  Calling wait_end afterwards would remove
     * a peer's waiter and is therefore deliberately forbidden here. */
    error = wf_windows_iocp_progress(
        &wf_windows_bridge_adapter,
        WF_WINDOWS_BRIDGE_DRAIN_BUDGET,
        timeout,
        &published
    );
    if (error != 0 && error != WAIT_TIMEOUT) {
        abort();
    }
    if (published != 0) {
        atomic_fetch_add_explicit(
            &wf_windows_bridge_publications,
            (uint64_t)published,
            memory_order_relaxed
        );
    }
    return wf_windows_bridge_drain() != 0 || published != 0;
}

static enum wf_completion_submit_verdict wf_windows_bridge_claim(
    wf_completion_token *token
) {
    enum wf_completion_claim_result claimed;
    if (token == NULL) {
        abort();
    }
    if (wf_windows_bridge_start() != 0) {
        abort();
    }
    claimed = wf_completion_claim(&wf_windows_bridge_runtime, token);
    if (claimed == WF_COMPLETION_CLAIMED) {
        return WF_COMPLETION_SUBMIT_ACCEPTED;
    }
    if (claimed == WF_COMPLETION_CLAIM_WAIT_CAPACITY) {
        /* wf_completion_claim does not write the caller's token on this
         * result.  The source owner must consume one of its older tokens and
         * retry; waiting inside the bridge could deadlock on that same owner. */
        return WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY;
    }
    abort();
}

void wf__completion_wait_core_capacity(void) {
    if (wf_windows_bridge_start() != 0) {
        abort();
    }
    for (;;) {
        uint64_t observed = wf_completion_wake_epoch(
            &wf_windows_bridge_runtime
        );
        if (wf_completion_has_capacity(&wf_windows_bridge_runtime)) {
            return;
        }
        if (wf_windows_bridge_drain() != 0) {
            continue;
        }
        /* A native parallel runtime supplies the strong hook and already
         * prefers a ready writer before compute work. A completion-only link
         * gets the no-op default, then helps its writer queue directly. */
        if (wf__par_help_once() || wf__writer_scheduler_help_once()) {
            continue;
        }
        if (wf_completion_has_capacity(&wf_windows_bridge_runtime)) {
            return;
        }
        (void)wf_windows_bridge_progress(observed, INFINITE);
    }
}

static int wf_windows_bridge_submit_pread(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    wf_completion_token *token,
    void *dependent_frame
) {
    intptr_t native;
    enum wf_completion_submit_verdict claimed;
    wf_windows_file_request request;
    wf_windows_iocp_file associated;
    if (descriptor < 0 || (buffer == NULL && count != 0) || token == NULL) {
        abort();
    }
    if (count == 0 || count > (uint64_t)MAXDWORD
        || file_offset > (uint64_t)INT64_MAX
        || count > (uint64_t)INT64_MAX - file_offset) {
        return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
    }
    native = _get_osfhandle(descriptor);
    if (native == -1 || wf_windows_bridge_start() != 0) {
        abort();
    }
    memset(&associated, 0, sizeof(associated));
    if (!wf_windows_bridge_file_for_descriptor(
            descriptor,
            (HANDLE)native,
            &associated
        )) {
        abort();
    }
    claimed = wf_windows_bridge_claim(token);
    if (claimed == WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY) {
        return claimed;
    }
    if (claimed != WF_COMPLETION_SUBMIT_ACCEPTED) {
        abort();
    }
    if (dependent_frame != NULL
        && wf_completion_depend(
            &wf_windows_bridge_runtime,
            *token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        ) != WF_COMPLETION_DEPEND_REGISTERED) {
        abort();
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_READ_AT;
    request.file = associated;
    request.buffer.read_buffer = buffer;
    request.count = (size_t)count;
    request.offset = file_offset;
    for (;;) {
        uint64_t observed = wf_completion_wake_epoch(
            &wf_windows_bridge_runtime
        );
        enum wf_windows_iocp_submit_result submitted = wf_windows_iocp_submit(
            &wf_windows_bridge_adapter,
            *token,
            &request
        );
        if (submitted == WF_WINDOWS_IOCP_TARGET_OWNS) {
            atomic_fetch_add_explicit(
                &wf_windows_bridge_submissions,
                1,
                memory_order_relaxed
            );
            return WF_COMPLETION_SUBMIT_ACCEPTED;
        }
        if (submitted != WF_WINDOWS_IOCP_WAIT_CAPACITY) {
            abort();
        }
        (void)wf_windows_bridge_progress(observed, INFINITE);
    }
}

uint64_t wf__completion_window(
    uint64_t span,
    uint64_t slot_bytes,
    uint64_t ceiling
) {
    uint64_t window = WF_WINDOWS_BRIDGE_CAPACITY / 2u;
    if (wf_windows_bridge_start() != 0) {
        abort();
    }
    if (ceiling != 0 && ceiling < window) {
        window = ceiling;
    }
    if (span != 0 && span < window) {
        window = span;
    }
    if (slot_bytes != 0) {
        uint64_t affordable = WF_WINDOWS_BRIDGE_WINDOW_BUDGET / slot_bytes;
        if (affordable < window) {
            window = affordable;
        }
    }
    return window == 0 ? 1u : window;
}

int wf__completion_file_pread_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage
) {
    return wf_windows_bridge_submit_pread(
        descriptor,
        buffer,
        count,
        file_offset,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_pread_submit_writer(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    void *token_storage,
    void *frame
) {
    return wf_windows_bridge_submit_pread(
        descriptor,
        buffer,
        count,
        file_offset,
        (wf_completion_token *)token_storage,
        frame
    );
}

/* IOCP's positioned WRITE_AT is not `Output.write_once`: it has no stream or
 * current-position meaning.  Every non-positioned write remains on the direct
 * WriteFile implementation instead of silently changing that contract. */
int wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
) {
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)token_storage;
    return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
}

int wf__completion_file_write_submit_writer(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage,
    void *frame
) {
    (void)frame;
    return wf__completion_file_write_submit(
        descriptor,
        buffer,
        count,
        token_storage
    );
}

int wf__completion_file_read_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    void *token_storage
) {
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)token_storage;
    return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
}

int wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    void *token_storage
) {
    (void)directory;
    (void)path;
    (void)flags;
    (void)mode;
    (void)has_mode;
    (void)expected_kind;
    (void)token_storage;
    return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
}

int wf__completion_file_status_submit(int descriptor, void *token_storage) {
    (void)descriptor;
    (void)token_storage;
    return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
}

int wf__completion_file_close_submit(int descriptor, void *token_storage) {
    (void)descriptor;
    (void)token_storage;
    return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
}

int wf__completion_directory_next_submit(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position,
    void *token_storage
) {
    (void)descriptor;
    (void)buffer;
    (void)count;
    (void)position;
    (void)token_storage;
    return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
}

static int wf_windows_bridge_take_result(
    const void *token_storage,
    wf_windows_file_result *result
) {
    wf_completion_outcome outcome;
    enum wf_completion_consume_result consumed;
    if (token_storage == NULL || result == NULL) {
        abort();
    }
    memset(&outcome, 0, sizeof(outcome));
    consumed = wf_completion_consume(
        &wf_windows_bridge_runtime,
        *(const wf_completion_token *)token_storage,
        result,
        sizeof(*result),
        &outcome
    );
    if (consumed == WF_COMPLETION_CONSUMED) {
        if (outcome.result_size != sizeof(*result)
            || result->kind != WF_WINDOWS_FILE_READ_AT) {
            abort();
        }
        return 1;
    }
    if (consumed == WF_COMPLETION_CONSUME_NOT_TERMINAL
        || consumed == WF_COMPLETION_CONSUME_NOT_DRAINED) {
        return 0;
    }
    abort();
}

int wf__completion_file_take(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    wf_windows_file_result result;
    if (value == NULL || error_code == NULL) {
        abort();
    }
    (void)wf_windows_bridge_drain();
    if (!wf_windows_bridge_take_result(token_storage, &result)) {
        return 0;
    }
    *value = result.value;
    *error_code = (int)result.error_code;
    return 1;
}

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    for (;;) {
        uint64_t observed = wf_completion_wake_epoch(
            &wf_windows_bridge_runtime
        );
        if (wf__completion_file_take(token_storage, value, error_code)) {
            return;
        }
        (void)wf_windows_bridge_progress(observed, INFINITE);
    }
}

void wf__completion_file_open_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
) {
    (void)token_storage;
    (void)value;
    (void)error_code;
    (void)open_outcome;
    abort();
}

void wf__completion_file_status_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
) {
    (void)token_storage;
    (void)value;
    (void)error_code;
    (void)status;
    (void)status_capacity;
    (void)status_size;
    abort();
}

int wf__completion_file_take_status(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    void *status,
    uint64_t status_capacity,
    uint64_t *status_size
) {
    (void)token_storage;
    (void)value;
    (void)error_code;
    (void)status;
    (void)status_capacity;
    (void)status_size;
    return 0;
}

void wf__writer_scheduler_notify(void) {
    if (atomic_load_explicit(
            &wf_windows_bridge_ready,
            memory_order_acquire
        ) != 0u) {
        wf_completion_notify_compute(&wf_windows_bridge_runtime);
    }
}

void wf__writer_run_root(void *frame) {
    while (!wf__writer_is_done(frame)) {
        uint64_t observed = wf_completion_wake_epoch(
            &wf_windows_bridge_runtime
        );
        int progressed = wf_windows_bridge_drain() != 0;
        progressed |= wf__writer_scheduler_help_once();
        if (progressed) {
            continue;
        }
        if (!wf__writer_is_done(frame)) {
            (void)wf_windows_bridge_progress(observed, INFINITE);
        }
    }
}

uint64_t wf__completion_file_submissions(void) {
    return atomic_load_explicit(
        &wf_windows_bridge_submissions,
        memory_order_relaxed
    );
}

uint64_t wf__completion_file_fallback_submissions(void) { return 0; }
uint64_t wf__completion_file_demoted_opens(void) { return 0; }
uint64_t wf__completion_file_helper_executions(void) { return 0; }
uint64_t wf__completion_target_helper_count(void) { return 0; }
uint64_t wf__completion_target_helper_executions(void) { return 0; }
uint64_t wf__completion_publications(void) {
    return atomic_load_explicit(
        &wf_windows_bridge_publications,
        memory_order_relaxed
    );
}
uint64_t wf__completion_linux_io_uring_submissions(void) { return 0; }
uint64_t wf__completion_linux_io_uring_submission_enters(void) { return 0; }
uint64_t wf__completion_open_exhaustion_retries(void) { return 0; }
uint64_t wf__completion_open_exhaustion_waits(void) { return 0; }

#endif
