#include "bridge.h"
#include "../windows_runtime.h"
#include "windows_blocking.h"
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
#define WF_WINDOWS_BRIDGE_CAPACITY WF_COMPLETION_SLOT_CAPACITY
#endif
#define WF_WINDOWS_BRIDGE_DRAIN_BUDGET 16u
#define WF_WINDOWS_BRIDGE_WINDOW_BUDGET (4u * 1024u * 1024u)

#if !defined(WF_WINDOWS_COMPILER_CAPACITY_PROBE)
_Static_assert(
    WF_WINDOWS_BRIDGE_CAPACITY == WF_WINDOWS_BLOCKING_CAPACITY,
    "a claimed production core token must imply a free blocking job"
);
_Static_assert(
    WF_WINDOWS_BRIDGE_CAPACITY == WF_WRITER_READY_CAPACITY,
    "one writer-ready cell is required for each completion slot"
);
#endif

/* writer_scheduler_windows.c supplies a completion-only default; the native
 * compute runtime overrides it with its unified work-first helper. */
extern int wf__par_help_once(void);

static wf_completion_runtime wf_windows_bridge_runtime;
static wf_completion_slot wf_windows_bridge_slots[WF_WINDOWS_BRIDGE_CAPACITY];
static wf_windows_iocp_adapter wf_windows_bridge_adapter;
static wf_windows_iocp_entry wf_windows_bridge_entries[WF_WINDOWS_BRIDGE_CAPACITY];
static wf_windows_blocking_adapter wf_windows_bridge_blocking;
static INIT_ONCE wf_windows_bridge_once = INIT_ONCE_STATIC_INIT;
static INIT_ONCE wf_windows_bridge_blocking_once = INIT_ONCE_STATIC_INIT;
static _Atomic unsigned wf_windows_bridge_ready;
static _Atomic unsigned wf_windows_bridge_blocking_ready;
static int wf_windows_bridge_error;
static int wf_windows_bridge_blocking_error;
static unsigned wf_windows_bridge_require_iocp;

static void wf_windows_bridge_shutdown_blocking(void) {
    if (atomic_load_explicit(
            &wf_windows_bridge_blocking_ready,
            memory_order_acquire
        ) != 0u
        && wf_windows_blocking_shutdown(&wf_windows_bridge_blocking) != 0) {
        abort();
    }
}

static BOOL CALLBACK wf_windows_bridge_initialize_blocking(
    PINIT_ONCE once,
    PVOID parameter,
    PVOID *context
) {
    int error;
    (void)once;
    (void)parameter;
    (void)context;
    error = wf_windows_blocking_init(
        &wf_windows_bridge_blocking,
        &wf_windows_bridge_runtime
    );
    if (error == 0 && atexit(wf_windows_bridge_shutdown_blocking) != 0) {
        error = ERROR_NOT_ENOUGH_MEMORY;
    }
    wf_windows_bridge_blocking_error = error;
    if (error == 0) {
        atomic_store_explicit(
            &wf_windows_bridge_blocking_ready,
            1u,
            memory_order_release
        );
    }
    return TRUE;
}

typedef struct wf_windows_bridge_file_entry {
    wf_windows_iocp_file file;
    uint64_t generation;
    size_t active_leases;
    unsigned descriptor_class;
    unsigned serial_busy;
    unsigned state;
} wf_windows_bridge_file_entry;

enum wf_windows_bridge_file_state {
    WF_WINDOWS_BRIDGE_FILE_EMPTY = 0,
    WF_WINDOWS_BRIDGE_FILE_LIVE = 1,
    WF_WINDOWS_BRIDGE_FILE_CLOSING = 2
};

static SRWLOCK wf_windows_bridge_files_lock = SRWLOCK_INIT;
static CONDITION_VARIABLE wf_windows_bridge_files_ready =
    CONDITION_VARIABLE_INIT;
static wf_windows_bridge_file_entry *wf_windows_bridge_files;
static size_t wf_windows_bridge_file_capacity;
static _Atomic size_t wf_windows_bridge_active_leases;

static void wf_windows_bridge_verify_required_iocp(void) {
    wf_windows_iocp_statistics statistics;
    if (wf_windows_bridge_require_iocp == 0u) {
        return;
    }
    statistics = wf_windows_iocp_statistics_snapshot(
        &wf_windows_bridge_adapter
    );
    if (statistics.submissions == 0
        || statistics.completions != statistics.submissions
        || statistics.publication_failures != 0) {
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
    wf_completion_retirement_announces_on(&wf_windows_bridge_runtime);
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
        0,
        WF_WINDOWS_IOCP_INLINE_SYNCHRONOUS_SUCCESS
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

static int wf_windows_bridge_ensure_blocking(void) {
    if (wf_windows_bridge_start() != 0) {
        abort();
    }
    if (InitOnceExecuteOnce(
            &wf_windows_bridge_blocking_once,
            wf_windows_bridge_initialize_blocking,
            NULL,
            NULL
        ) == FALSE) {
        DWORD error = GetLastError();
        return error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : (int)error;
    }
    return atomic_load_explicit(
        &wf_windows_bridge_blocking_ready,
        memory_order_acquire
    ) != 0u
        ? 0
        : (wf_windows_bridge_blocking_error == 0
               ? ERROR_GEN_FAILURE
               : wf_windows_bridge_blocking_error);
}

static int wf_windows_bridge_grow_files(size_t required) {
    wf_windows_bridge_file_entry *grown;
    size_t capacity;
    if (required <= wf_windows_bridge_file_capacity) {
        return 0;
    }
    capacity = wf_windows_bridge_file_capacity == 0
        ? 64u
        : wf_windows_bridge_file_capacity;
    while (capacity < required && capacity <= SIZE_MAX / 2u) {
        capacity *= 2u;
    }
    if (capacity < required
        || capacity > SIZE_MAX / sizeof(*wf_windows_bridge_files)) {
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
        return ERROR_NOT_ENOUGH_MEMORY;
    }
    wf_windows_bridge_files = grown;
    wf_windows_bridge_file_capacity = capacity;
    return 0;
}

static int wf_windows_bridge_register_descriptor(
    int descriptor,
    HANDLE handle,
    unsigned descriptor_class,
    void *completion_owner
) {
    wf_windows_bridge_file_entry *entry;
    int error;
    if (descriptor < 0 || handle == NULL || handle == INVALID_HANDLE_VALUE
        || descriptor_class < WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE
        || descriptor_class > WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT) {
        return ERROR_INVALID_PARAMETER;
    }
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    error = wf_windows_bridge_grow_files((size_t)descriptor + 1u);
    if (error != 0) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        return error;
    }
    entry = &wf_windows_bridge_files[descriptor];
    while (entry->state == WF_WINDOWS_BRIDGE_FILE_CLOSING) {
        if (SleepConditionVariableSRW(
                &wf_windows_bridge_files_ready,
                &wf_windows_bridge_files_lock,
                INFINITE,
                0
            ) == FALSE) {
            ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
            abort();
        }
        entry = &wf_windows_bridge_files[descriptor];
    }
    if (entry->state != WF_WINDOWS_BRIDGE_FILE_EMPTY
        || entry->active_leases != 0 || entry->serial_busy != 0) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        return ERROR_ALREADY_EXISTS;
    }
    if (entry->generation == UINT64_MAX) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry->generation += 1u;
    entry->file.handle = handle;
    entry->file.adapter = completion_owner == NULL
        ? NULL
        : &wf_windows_bridge_adapter;
    entry->descriptor_class = descriptor_class;
    entry->state = WF_WINDOWS_BRIDGE_FILE_LIVE;
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
    return 0;
}

int wf__windows_completion_register_descriptor(
    int descriptor,
    unsigned descriptor_class
) {
    intptr_t native;
    int error = wf_windows_bridge_start();
    if (error != 0) {
        return error;
    }
    native = descriptor < 0 ? -1 : _get_osfhandle(descriptor);
    if (native == -1) {
        return ERROR_INVALID_HANDLE;
    }
    return wf_windows_bridge_register_descriptor(
        descriptor,
        (HANDLE)native,
        descriptor_class,
        NULL
    );
}

/* A regular-file descriptor is associated exactly once, immediately after
 * the runtime creates it. The generation-indexed entry retains both the IOCP
 * capability and every active host-use lease until close. */
int wf__windows_completion_associate_descriptor(int descriptor) {
    intptr_t native;
    wf_windows_iocp_file associated;
    int error = wf_windows_bridge_start();
    if (error != 0) {
        return error;
    }
    native = descriptor < 0 ? -1 : _get_osfhandle(descriptor);
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
    return wf_windows_bridge_register_descriptor(
        descriptor,
        associated.handle,
        WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
        associated.adapter
    );
}

int wf__windows_completion_descriptor_lease_acquire(
    int descriptor,
    unsigned required_class,
    unsigned mode,
    wf_windows_descriptor_lease *lease
) {
    wf_windows_bridge_file_entry *entry;
    intptr_t native;
    if (lease == NULL
        || required_class > WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT
        || (mode != WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
            && mode != WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL)) {
        abort();
    }
    memset(lease, 0, sizeof(*lease));
    if (descriptor < 0 || _get_osfhandle(descriptor) == -1) {
        return 0;
    }
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    native = _get_osfhandle(descriptor);
    if (native == -1) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        return 0;
    }
    if ((size_t)descriptor >= wf_windows_bridge_file_capacity) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry = &wf_windows_bridge_files[descriptor];
    if (entry->state != WF_WINDOWS_BRIDGE_FILE_LIVE
        || entry->file.handle != (HANDLE)native
        || (required_class != WF_WINDOWS_DESCRIPTOR_CLASS_ANY
            && entry->descriptor_class != required_class)
        || entry->generation == 0
        || ((entry->descriptor_class
                 == WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE
             || entry->descriptor_class
                 == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT)
            && mode != WF_WINDOWS_DESCRIPTOR_LEASE_SHARED)
        || ((entry->descriptor_class
                 == WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE
             || entry->descriptor_class
                 == WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT)
            && mode != WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL)) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    if (entry->serial_busy != 0
        || (mode == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL
            && entry->active_leases != 0)
        || entry->active_leases == SIZE_MAX) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry->active_leases += 1u;
    if (mode == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL) {
        entry->serial_busy = 1u;
    }
    lease->descriptor = descriptor;
    lease->generation = entry->generation;
    lease->handle = entry->file.handle;
    lease->completion_owner = entry->file.adapter;
    lease->descriptor_class = entry->descriptor_class;
    lease->mode = mode;
    atomic_fetch_add_explicit(
        &wf_windows_bridge_active_leases,
        1,
        memory_order_relaxed
    );
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
    return 1;
}

void wf__windows_completion_descriptor_lease_release(
    wf_windows_descriptor_lease *lease
) {
    wf_windows_bridge_file_entry *entry;
    if (lease == NULL || lease->generation == 0 || lease->descriptor < 0) {
        abort();
    }
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    if ((size_t)lease->descriptor >= wf_windows_bridge_file_capacity) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry = &wf_windows_bridge_files[lease->descriptor];
    if (entry->state != WF_WINDOWS_BRIDGE_FILE_LIVE
        || entry->generation != lease->generation
        || entry->file.handle != lease->handle
        || entry->active_leases == 0
        || (lease->mode == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL
            && entry->serial_busy == 0)
        || (lease->mode == WF_WINDOWS_DESCRIPTOR_LEASE_SHARED
            && entry->serial_busy != 0)) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry->active_leases -= 1u;
    if (lease->mode == WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL) {
        entry->serial_busy = 0u;
    }
    if (atomic_fetch_sub_explicit(
            &wf_windows_bridge_active_leases,
            1,
            memory_order_relaxed
        ) == 0) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
    memset(lease, 0, sizeof(*lease));
}

int wf__windows_completion_descriptor_close_begin(
    int descriptor,
    wf_windows_descriptor_close_ticket *ticket
) {
    wf_windows_bridge_file_entry *entry;
    intptr_t native;
    if (ticket == NULL) {
        abort();
    }
    memset(ticket, 0, sizeof(*ticket));
    if (descriptor < 0 || _get_osfhandle(descriptor) == -1) {
        return 0;
    }
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    native = _get_osfhandle(descriptor);
    if (native == -1) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        return 0;
    }
    if ((size_t)descriptor >= wf_windows_bridge_file_capacity) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry = &wf_windows_bridge_files[descriptor];
    if (entry->state != WF_WINDOWS_BRIDGE_FILE_LIVE
        || entry->file.handle != (HANDLE)native
        || entry->generation == 0 || entry->active_leases != 0
        || entry->serial_busy != 0) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry->state = WF_WINDOWS_BRIDGE_FILE_CLOSING;
    ticket->descriptor = descriptor;
    ticket->generation = entry->generation;
    ticket->handle = entry->file.handle;
    ticket->active = 1u;
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
    return 1;
}

void wf__windows_completion_descriptor_close_finish(
    wf_windows_descriptor_close_ticket *ticket
) {
    wf_windows_bridge_file_entry *entry;
    uint64_t generation;
    if (ticket == NULL || ticket->active == 0 || ticket->descriptor < 0) {
        abort();
    }
    AcquireSRWLockExclusive(&wf_windows_bridge_files_lock);
    if ((size_t)ticket->descriptor >= wf_windows_bridge_file_capacity) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    entry = &wf_windows_bridge_files[ticket->descriptor];
    if (entry->state != WF_WINDOWS_BRIDGE_FILE_CLOSING
        || entry->generation != ticket->generation
        || entry->file.handle != ticket->handle
        || entry->active_leases != 0 || entry->serial_busy != 0) {
        ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
        abort();
    }
    generation = entry->generation;
    memset(entry, 0, sizeof(*entry));
    entry->generation = generation;
    WakeAllConditionVariable(&wf_windows_bridge_files_ready);
    ReleaseSRWLockExclusive(&wf_windows_bridge_files_lock);
    memset(ticket, 0, sizeof(*ticket));
}

size_t wf__windows_completion_active_descriptor_leases(void) {
    return atomic_load_explicit(
        &wf_windows_bridge_active_leases,
        memory_order_relaxed
    );
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

/* Harvests only the event owned by this join instead of sweeping all 64 core
 * slots on every turn. */
static void wf_windows_bridge_drain_token(const void *token_storage) {
    wf_completion_event event;
    if (token_storage == NULL) {
        abort();
    }
    (void)wf_completion_drain_token(
        &wf_windows_bridge_runtime,
        *(const wf_completion_token *)token_storage,
        &event
    );
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

static enum wf_completion_submit_verdict wf_windows_bridge_submit_blocking(
    const wf_windows_blocking_request *request,
    wf_completion_token *token,
    void *dependent_frame
) {
    enum wf_completion_submit_verdict claimed;
    enum wf_windows_blocking_submit_result submitted;
    wf_windows_blocking_request owned;
    int acquired;
    if (request == NULL || token == NULL
        || wf_windows_bridge_ensure_blocking() != 0) {
        abort();
    }
    claimed = wf_windows_bridge_claim(token);
    if (claimed == WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY) {
        return claimed;
    }
    if (claimed != WF_COMPLETION_SUBMIT_ACCEPTED) {
        abort();
    }
    owned = *request;
    switch (owned.kind) {
    case WF_WINDOWS_FILE_WRITE:
        acquired = wf__windows_completion_descriptor_lease_acquire(
            owned.operation.write.descriptor,
            WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT,
            WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL,
            &owned.lease
        );
        break;
    case WF_WINDOWS_FILE_OPEN_AT:
        acquired = wf__windows_completion_descriptor_lease_acquire(
            owned.operation.open_at.directory,
            WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT,
            WF_WINDOWS_DESCRIPTOR_LEASE_SHARED,
            &owned.lease
        );
        break;
    case WF_WINDOWS_FILE_DIRECTORY_NEXT:
        acquired = wf__windows_completion_descriptor_lease_acquire(
            owned.operation.directory_next.descriptor,
            WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE,
            WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL,
            &owned.lease
        );
        break;
    default:
        abort();
    }
    if (!acquired) {
        abort();
    }
    if (dependent_frame != NULL) {
        enum wf_completion_depend_result dependency = wf_completion_depend(
            &wf_windows_bridge_runtime,
            *token,
            WF_COMPLETION_OWNERSHIP_COMPLETE,
            dependent_frame
        );
        if (dependency != WF_COMPLETION_DEPEND_REGISTERED) {
            abort();
        }
    }
    submitted = wf_windows_blocking_submit(
        &wf_windows_bridge_blocking,
        *token,
        &owned
    );
    if (submitted != WF_WINDOWS_BLOCKING_TARGET_OWNS) {
        abort();
    }
    return WF_COMPLETION_SUBMIT_ACCEPTED;
}

static int wf_windows_bridge_path_units(
    const char *path,
    size_t *unit_count
) {
    size_t index;
    if (path == NULL || unit_count == NULL) {
        return 0;
    }
    for (index = 0; index < WF_WINDOWS_BLOCKING_PATH_UNITS; ++index) {
        uint16_t unit;
        memcpy(&unit, path + index * sizeof(unit), sizeof(unit));
        if (unit == 0) {
            *unit_count = index;
            return 1;
        }
    }
    return 0;
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

int wf__windows_completion_progress_for_retirement(void) {
    uint64_t observed;
    int progressed;
    if (atomic_load_explicit(
            &wf_windows_bridge_ready,
            memory_order_acquire
        ) == 0u) {
        return 0;
    }
    observed = wf_completion_wake_epoch(&wf_windows_bridge_runtime);
    progressed = wf_windows_bridge_drain() != 0;
    progressed |= wf__par_help_once();
    if (!progressed) {
        progressed |= wf__writer_scheduler_help_once();
    }
    if (!progressed) {
        progressed |= wf_windows_bridge_progress(observed, 10u);
    }
    return progressed;
}

static int wf_windows_bridge_submit_pread(
    int descriptor,
    void *buffer,
    uint64_t count,
    uint64_t file_offset,
    wf_completion_token *token,
    void *dependent_frame
) {
    enum wf_completion_submit_verdict claimed;
    wf_windows_file_request request;
    wf_windows_iocp_file associated;
    wf_windows_descriptor_lease lease;
    if (descriptor < 0 || (buffer == NULL && count != 0) || token == NULL) {
        abort();
    }
    if (count == 0 || count > (uint64_t)MAXDWORD
        || file_offset > (uint64_t)INT64_MAX
        || count > (uint64_t)INT64_MAX - file_offset) {
        return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
    }
    if (wf_windows_bridge_start() != 0) {
        abort();
    }
    claimed = wf_windows_bridge_claim(token);
    if (claimed == WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY) {
        return claimed;
    }
    if (claimed != WF_COMPLETION_SUBMIT_ACCEPTED) {
        abort();
    }
    if (!wf__windows_completion_descriptor_lease_acquire(
            descriptor,
            WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
            WF_WINDOWS_DESCRIPTOR_LEASE_SHARED,
            &lease
        )
        || lease.completion_owner != &wf_windows_bridge_adapter) {
        abort();
    }
    associated.handle = lease.handle;
    associated.adapter = (wf_windows_iocp_adapter *)lease.completion_owner;
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
    request.lease = lease;
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
 * current-position meaning. The bounded blocking pool preserves WriteFile's
 * exact current-position contract without stalling a scheduler lane. */
int wf__completion_file_write_submit(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage
) {
    wf_windows_blocking_request request;
    if (descriptor < 0 || (buffer == NULL && count != 0)
        || token_storage == NULL) {
        abort();
    }
    if (count == 0) {
        return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = count;
    return wf_windows_bridge_submit_blocking(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
}

int wf__completion_file_write_submit_writer(
    int descriptor,
    const void *buffer,
    uint64_t count,
    void *token_storage,
    void *frame
) {
    wf_windows_blocking_request request;
    if (descriptor < 0 || (buffer == NULL && count != 0)
        || token_storage == NULL || frame == NULL) {
        abort();
    }
    if (count == 0) {
        return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_WRITE;
    request.operation.write.descriptor = descriptor;
    request.operation.write.buffer = buffer;
    request.operation.write.count = count;
    return wf_windows_bridge_submit_blocking(
        &request,
        (wf_completion_token *)token_storage,
        frame
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
    /* The language's file read operation is positioned. Reaching this ABI is
     * compiler/runtime drift, not a permitted synchronous backend choice. */
    abort();
}

int wf__completion_file_open_at_submit(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    unsigned descriptor_class,
    void *token_storage
) {
    wf_windows_blocking_request request;
    size_t path_units;
    if (directory < 0 || path == NULL || token_storage == NULL) {
        abort();
    }
    /* An overlong path is outside the bounded job representation. The direct
     * operation preserves its precise ERROR_FILENAME_EXCED_RANGE result. */
    if (!wf_windows_bridge_path_units(path, &path_units)) {
        return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_OPEN_AT;
    request.operation.open_at.directory = directory;
    request.operation.open_at.path = path;
    request.operation.open_at.path_units = path_units;
    request.operation.open_at.flags = flags;
    request.operation.open_at.mode = mode;
    request.operation.open_at.has_mode = has_mode;
    request.operation.open_at.expected_kind = expected_kind;
    request.operation.open_at.descriptor_class = descriptor_class;
    return wf_windows_bridge_submit_blocking(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
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
    wf_windows_blocking_request request;
    if (descriptor < 0 || position == NULL
        || (buffer == NULL && count != 0) || token_storage == NULL) {
        abort();
    }
    if (count == 0) {
        return WF_COMPLETION_SUBMIT_DIRECT_ONLY;
    }
    memset(&request, 0, sizeof(request));
    request.kind = WF_WINDOWS_FILE_DIRECTORY_NEXT;
    request.operation.directory_next.descriptor = descriptor;
    request.operation.directory_next.buffer = buffer;
    request.operation.directory_next.count = count;
    request.operation.directory_next.position = position;
    return wf_windows_bridge_submit_blocking(
        &request,
        (wf_completion_token *)token_storage,
        NULL
    );
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
        if (outcome.result_size != sizeof(*result)) {
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
    if (!wf_windows_bridge_take_result(token_storage, &result)) {
        return 0;
    }
    if (result.kind != WF_WINDOWS_FILE_READ_AT
        && result.kind != WF_WINDOWS_FILE_WRITE
        && result.kind != WF_WINDOWS_FILE_DIRECTORY_NEXT) {
        abort();
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
        wf_windows_bridge_drain_token(token_storage);
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
    for (;;) {
        wf_windows_file_result result;
        uint64_t observed = wf_completion_wake_epoch(
            &wf_windows_bridge_runtime
        );
        wf_windows_bridge_drain_token(token_storage);
        if (wf_windows_bridge_take_result(token_storage, &result)) {
            if (result.kind != WF_WINDOWS_FILE_OPEN_AT
                || value == NULL || error_code == NULL
                || open_outcome == NULL) {
                abort();
            }
            *value = result.value;
            *error_code = (int)result.error_code;
            *open_outcome = result.open_outcome;
            return;
        }
        (void)wf_windows_bridge_progress(observed, INFINITE);
    }
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
        progressed |= wf__par_help_once();
        if (!progressed) {
            progressed |= wf__writer_scheduler_help_once();
        }
        if (progressed) {
            continue;
        }
        if (!wf__writer_is_done(frame)) {
            (void)wf_windows_bridge_progress(observed, INFINITE);
        }
    }
}

uint64_t wf__completion_file_submissions(void) {
    wf_windows_iocp_statistics iocp =
        wf_windows_iocp_statistics_snapshot(&wf_windows_bridge_adapter);
    wf_windows_blocking_statistics blocking =
        wf_windows_blocking_statistics_snapshot(&wf_windows_bridge_blocking);
    return iocp.submissions + blocking.submissions;
}

uint64_t wf__completion_windows_iocp_in_flight(void) {
    return (uint64_t)wf_windows_iocp_in_flight(&wf_windows_bridge_adapter);
}

uint64_t wf__completion_windows_iocp_inline_completions(void) {
    return wf_windows_iocp_statistics_snapshot(
        &wf_windows_bridge_adapter
    ).inline_completions;
}

uint64_t wf__completion_windows_iocp_dequeued_completions(void) {
    return wf_windows_iocp_statistics_snapshot(
        &wf_windows_bridge_adapter
    ).dequeued_completions;
}

uint64_t wf__completion_file_fallback_submissions(void) { return 0; }
uint64_t wf__completion_file_demoted_opens(void) { return 0; }
uint64_t wf__completion_file_helper_executions(void) {
    return wf_windows_blocking_statistics_snapshot(
        &wf_windows_bridge_blocking
    ).executions;
}
uint64_t wf__completion_target_helper_count(void) {
    return (uint64_t)wf_windows_blocking_statistics_snapshot(
        &wf_windows_bridge_blocking
    ).worker_count;
}
uint64_t wf__completion_target_helper_executions(void) {
    return wf__completion_file_helper_executions();
}
uint64_t wf__completion_publications(void) {
    return wf_completion_statistics_snapshot(
        &wf_windows_bridge_runtime
    ).publications;
}
uint64_t wf__completion_linux_io_uring_submissions(void) { return 0; }
uint64_t wf__completion_linux_io_uring_submission_enters(void) { return 0; }
uint64_t wf__completion_open_exhaustion_retries(void) {
    return wf_windows_blocking_statistics_snapshot(
               &wf_windows_bridge_blocking
           ).exhaustion_retries
        + wf__windows_direct_open_exhaustion_retries();
}
uint64_t wf__completion_open_exhaustion_waits(void) {
    return wf_completion_retirement_waits();
}

#endif
