#ifndef WHITEFOOT_WINDOWS_RUNTIME_H
#define WHITEFOOT_WINDOWS_RUNTIME_H

#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

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
    int *error_code,
    unsigned *open_outcome
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
void wf__windows_completion_forget_descriptor(int descriptor);

#if defined(__cplusplus)
}
#endif

#endif
