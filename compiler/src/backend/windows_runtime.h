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

#define WF_WINDOWS_DESCRIPTOR_CLASS_ANY 0u
#define WF_WINDOWS_DESCRIPTOR_CLASS_READ_FILE 1u
#define WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT 2u
#define WF_WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE 3u
#define WF_WINDOWS_DESCRIPTOR_CLASS_OUTPUT 4u

/* What the runtime remembers about one descriptor it produced.
 *
 * All that is left of the descriptor ledger the deleted bounded pool needed.
 * Two facts, and each has exactly one reader that cannot do without it:
 *
 *   the resource class, because the ring is offered only a descriptor this
 *   runtime opened as a regular file for reading, and a directory handle would
 *   otherwise pass the port's own "is it a disk file" test;
 *
 *   whether the completion port has already taken this handle, because
 *   `CreateIoCompletionPort` associates a handle exactly once and there is no
 *   host call that asks whether it already has.
 *
 * The generation, the shared and serial lease modes, the active-lease count,
 * the close ticket and the condition variable a close waited on are all gone
 * with the pool they protected: the record is the submitting frame's block,
 * the emitter joins every outstanding operation before any terminator, and a
 * close is itself a submitted operation the program orders against its reads
 * -- which is the same argument the POSIX bridge has always run on, with no
 * ledger at all (design section 7). */

/* Registers a descriptor this runtime produced.  Returns 0, or a Win32 error
 * for a descriptor with no native handle. */
int wf__windows_completion_register_descriptor(
    int descriptor,
    unsigned descriptor_class
);

/* What this runtime remembers about `descriptor`: answers 1 when it is one
 * this runtime produced, writing the class it was registered under and whether
 * the completion port has taken its handle. */
int wf__windows_completion_descriptor_state(
    int descriptor,
    unsigned *descriptor_class,
    unsigned *port_associated
);

/* Records that the completion port has taken this descriptor's handle,
 * registering it under `descriptor_class` if it was not registered. */
void wf__windows_completion_note_port_association(
    int descriptor,
    unsigned descriptor_class
);

/* Forgets a descriptor whose close has been made, so the next descriptor to
 * reuse the number starts with no association and no class. */
void wf__windows_completion_forget_descriptor(int descriptor);

/* The native handle behind a descriptor, or INVALID_HANDLE_VALUE. */
HANDLE wf__windows_completion_descriptor_handle(int descriptor);

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

int wf__windows_completion_file_open_at_worker(
    HANDLE root,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    unsigned descriptor_class,
    int *error_code,
    unsigned *open_outcome
);
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


#if defined(__cplusplus)
}
#endif

#endif
