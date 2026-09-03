#ifndef WHITEFOOT_WINDOWS_RUNTIME_H
#define WHITEFOOT_WINDOWS_RUNTIME_H

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32)
#include <windows.h>
#endif

#if defined(__cplusplus)
extern "C" {
#endif

#define WF_WINDOWS_OPEN_RESOURCE_NATIVE_HANDLE (1u << 0)
#define WF_WINDOWS_OPEN_RESOURCE_CRT_DESCRIPTOR (1u << 1)
#define WF_WINDOWS_OPEN_REFUSED_NONE 0u
#define WF_WINDOWS_OPEN_REFUSED_NATIVE_HANDLE 1u
#define WF_WINDOWS_OPEN_REFUSED_CRT_DESCRIPTOR 2u
#define WF_WINDOWS_OPEN_REFUSED_GENERAL_RESOURCE 3u

#define WF_WINDOWS_DESCRIPTOR_CLASS_ANY 0u
#define WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE 1u
#define WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT 2u
#define WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE 3u
#define WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT 4u

#define WF_WINDOWS_DESCRIPTOR_LEASE_SHARED 1u
#define WF_WINDOWS_DESCRIPTOR_LEASE_SERIAL 2u

typedef struct wf_windows_descriptor_lease {
    int descriptor;
    uint64_t generation;
    HANDLE handle;
    void *completion_owner;
    unsigned descriptor_class;
    unsigned mode;
} wf_windows_descriptor_lease;

typedef struct wf_windows_descriptor_close_ticket {
    int descriptor;
    uint64_t generation;
    HANDLE handle;
    unsigned active;
} wf_windows_descriptor_close_ticket;

uint64_t wf__windows_wcslen(const uint16_t *text);
int wf__windows_utf8_measure(
    const uint16_t *text,
    uint64_t unit_count,
    uint64_t *encoded_length
);
int wf__windows_utf8_copy(
    const uint16_t *text,
    uint64_t unit_count,
    uint8_t *destination,
    uint64_t destination_capacity
);
int wf__windows_relative_path_valid(
    const uint16_t *text,
    uint64_t unit_count
);

int *wf__windows_error_location(void);
int wf__windows_open_cwd(const void *unused_path, int flags, ...);
int wf__windows_stdout_descriptor(void);
int wf__windows_stderr_descriptor(void);
int64_t wf__windows_diagnostic_write(const void *bytes, uint64_t length);

int64_t wf__completion_file_pread_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t file_offset
);
int64_t wf__completion_file_write_direct(
    int descriptor,
    const void *buffer,
    uint64_t count
);
int wf__completion_file_open_at_direct(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    unsigned descriptor_class,
    int *error_code,
    unsigned *open_outcome
);
int wf__windows_completion_file_open_at_worker(
    HANDLE root,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    unsigned descriptor_class,
    int *error_code,
    unsigned *open_outcome,
    unsigned *took_resources,
    unsigned *returned_resources,
    unsigned *refused_resource,
    unsigned awarded_resource,
    int on_an_award
);
uint64_t wf__windows_open_order_reserve(void);
int wf__windows_open_order_is_serving(uint64_t ticket);
void wf__windows_open_order_enter(uint64_t ticket);
void wf__windows_open_order_leave(uint64_t ticket);
void wf__windows_open_resource_attempt_enter(void);
void wf__windows_open_resource_attempt_leave(void);
int wf__windows_completion_register_descriptor(
    int descriptor,
    unsigned descriptor_class
);
int wf__windows_completion_descriptor_lease_acquire(
    int descriptor,
    unsigned required_class,
    unsigned mode,
    wf_windows_descriptor_lease *lease
);
void wf__windows_completion_descriptor_lease_release(
    wf_windows_descriptor_lease *lease
);
int wf__windows_completion_descriptor_close_begin(
    int descriptor,
    wf_windows_descriptor_close_ticket *ticket
);
void wf__windows_completion_descriptor_close_finish(
    wf_windows_descriptor_close_ticket *ticket
);
size_t wf__windows_completion_active_descriptor_leases(void);
int64_t wf__windows_completion_file_write_worker(
    HANDLE handle,
    const void *buffer,
    uint64_t count
);
int64_t wf__windows_completion_directory_next_worker(
    HANDLE directory,
    void *buffer,
    uint64_t count,
    int64_t *position
);
int wf__completion_file_status_direct(
    int descriptor,
    void *status,
    uint64_t status_capacity
);
int wf__completion_file_close_direct(int descriptor);
int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
);
int64_t wf__windows_directory_batch(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
);

/* The Windows completion bridge owns this symbol in every ordinary Windows
 * link. A successfully classified regular read descriptor is associated before
 * it becomes writer-visible. The return is zero or a Win32 error code. */
int wf__windows_completion_associate_descriptor(int descriptor);

/* Direct open exhaustion can coexist with already-owned IOCP/blocking work.
 * This bounded progress turn keeps the direct caller from sleeping while it
 * is the only scheduler able to reap that work. */
int wf__windows_completion_progress_for_retirement(void);
uint64_t wf__windows_direct_open_exhaustion_retries(void);

#if defined(__cplusplus)
}
#endif

#endif
