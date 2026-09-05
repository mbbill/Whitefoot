#if !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif
/* F_NOCACHE, the Darwin half of the WF_IO_NOCACHE target policy, is a BSD
 * extension the POSIX macro above hides. */
#if defined(__APPLE__) && !defined(_DARWIN_C_SOURCE)
#define _DARWIN_C_SOURCE 1
#endif

/*
 * The POSIX host leaf of the file adapter: one host call per request kind, and
 * nothing else.
 *
 * `file_adapter.c` owns the queue, the helpers, the claim and the progress
 * pass, and is one implementation for every platform.  This unit is the one
 * thing that cannot be: the switch below is exactly the set of host calls a
 * POSIX target makes, and `file_windows.c` is its twin.  Nothing here knows
 * that a queue exists.
 */

#include "file_adapter.h"
#include "file_posix.h"

#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <poll.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#if !defined(WF_COMPLETION_POLL)
#define WF_COMPLETION_POLL poll
#else
extern int WF_COMPLETION_POLL(struct pollfd *, nfds_t, int);
#endif

#if !defined(WF_COMPLETION_PREAD)
#define WF_COMPLETION_PREAD pread
#else
extern ssize_t WF_COMPLETION_PREAD(int, void *, size_t, off_t);
#endif

/* The one host call an adapter open makes, named so a build may observe it.
 * The observing build is expected to perform the same host call; nothing else
 * about the open changes, and the default build calls `openat` directly. */
#if !defined(WF_FILE_OPENAT)
#define WF_FILE_OPENAT openat
#else
extern int WF_FILE_OPENAT(int, const char *, int, ...);
#endif

#if defined(__APPLE__)
/* libSystem exports the exact facility used by the qualified Darwin target;
 * it is intentionally not replaced with an opendir/readdir loop. */
#if !defined(WF_COMPLETION_GETDIRENTRIES64)
#define WF_COMPLETION_GETDIRENTRIES64 __getdirentries64
#endif
extern ssize_t WF_COMPLETION_GETDIRENTRIES64(
    int,
    void *,
    size_t,
    int64_t *
);
#elif defined(__linux__)
/* glibc exports the exact facility used by the qualified Linux target.  It is
 * declared here rather than reached through <dirent.h> for the same reason
 * the Darwin entry above is: the declaration is behind _GNU_SOURCE, which
 * this unit does not ask for, and the prototype is fixed by the ABI.  It is
 * intentionally not replaced with an opendir/readdir loop, which is a scan
 * built out of other operations [QUAL-2, QUAL-3]. */
#if !defined(WF_COMPLETION_GETDENTS64)
#define WF_COMPLETION_GETDENTS64 getdents64
#endif
extern ssize_t WF_COMPLETION_GETDENTS64(int, void *, size_t);
#endif

_Static_assert(
    sizeof(struct stat) <= WF_FILE_STATUS_CAPACITY,
    "WF_FILE_STATUS_CAPACITY must hold the host stat record"
);

static wf_file_result wf_file_execute_once(const wf_file_request *request) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    result.head.value = -1;

    switch (request->kind) {
    case WF_FILE_READ:
        if (request->operation.read.count > (size_t)SSIZE_MAX) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_WRITE:
        if (request->operation.write.count > (size_t)SSIZE_MAX) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PREAD:
        /* An empty transfer has no external action.  This must be decided
         * before descriptor and offset validation reaches the host: callers
         * rely on zero length completing without touching
         * either the destination or the file facility. */
        if (request->operation.pread.count == 0) {
            result.head.value = 0;
            return result;
        }
        if (request->operation.pread.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pread.offset
                != request->operation.pread.offset) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
    case WF_FILE_PWRITE:
        if (request->operation.pwrite.count > (size_t)SSIZE_MAX
            || (int64_t)(off_t)request->operation.pwrite.offset
                != request->operation.pwrite.offset) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        if (request->operation.directory_next.count > (size_t)SSIZE_MAX) {
            result.head.error_code = EINVAL;
            return result;
        }
        break;
#endif
    default:
        break;
    }

    /* Each pass is one qualified host attempt. No-progress interruption and
     * readiness refusal are absorbed by wf_file_execute_direct below and
     * never become writer-visible outcomes. */
    switch (request->kind) {
    case WF_FILE_OPEN_AT:
        if (request->operation.open_at.expected_kind
            > WF_FILE_EXPECT_DIRECTORY) {
            result.head.error_code = EINVAL;
            result.head.open_outcome = WF_FILE_OPEN_FAILED;
            return result;
        }
        if (request->operation.open_at.has_mode != 0) {
            result.head.value = WF_FILE_OPENAT(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      ),
                (mode_t)request->operation.open_at.mode
            );
        } else {
            result.head.value = WF_FILE_OPENAT(
                request->operation.open_at.directory,
                request->operation.open_at.path,
                request->operation.open_at.flags
                    | wf_file_open_kind_flags(
                          request->operation.open_at.expected_kind
                      )
            );
        }
        if (result.head.value < 0) {
            result.head.open_outcome = WF_FILE_OPEN_FAILED;
            break;
        }
        if (request->operation.open_at.expected_kind != WF_FILE_EXPECT_ANY) {
            struct stat status;
            int descriptor = (int)result.head.value;
            if (fstat(descriptor, &status) != 0) {
                int status_error = errno;
                (void)close(descriptor);
                result.head.error_code = status_error;
                result.head.open_outcome = WF_FILE_OPEN_STATUS_FAILED;
                return result;
            }
            result.head.open_outcome = wf_file_kind_outcome(
                request->operation.open_at.expected_kind,
                (unsigned int)status.st_mode
            );
            if (result.head.open_outcome != WF_FILE_OPEN_SUCCEEDED) {
                (void)close(descriptor);
                return result;
            }
        }
        /* The descriptor is the one this open hands back: the kind check has
         * either passed or was not asked for. WF_IO_NOCACHE is applied here,
         * once, so a refused descriptor never receives it. */
        wf_file_apply_uncached_reads((int)result.head.value);
        break;
    case WF_FILE_READ:
        result.head.value = read(
            request->operation.read.descriptor,
            request->operation.read.buffer,
            request->operation.read.count
        );
        break;
    case WF_FILE_WRITE:
        result.head.value = write(
            request->operation.write.descriptor,
            request->operation.write.buffer,
            request->operation.write.count
        );
        break;
    case WF_FILE_PREAD:
        result.head.value = WF_COMPLETION_PREAD(
            request->operation.pread.descriptor,
            request->operation.pread.buffer,
            request->operation.pread.count,
            (off_t)request->operation.pread.offset
        );
        break;
    case WF_FILE_PWRITE:
        result.head.value = pwrite(
            request->operation.pwrite.descriptor,
            request->operation.pwrite.buffer,
            request->operation.pwrite.count,
            (off_t)request->operation.pwrite.offset
        );
        break;
    case WF_FILE_STATUS: {
        struct stat status;
        result.head.value = fstat(request->operation.status.descriptor, &status);
        if (result.head.value == 0) {
            result.status_size = sizeof(status);
            memcpy(result.status, &status, sizeof(status));
        }
        break;
    }
    case WF_FILE_CLOSE:
        result.head.value = close(request->operation.close.descriptor);
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
#if defined(__APPLE__)
        result.head.value = WF_COMPLETION_GETDIRENTRIES64(
            request->operation.directory_next.descriptor,
            request->operation.directory_next.buffer,
            request->operation.directory_next.count,
            request->operation.directory_next.position
        );
#else
        /* Linux keeps the whole enumeration cursor in the descriptor, so the
         * facility takes no base-position argument and this cell is left as
         * the caller gave it. */
        result.head.value = WF_COMPLETION_GETDENTS64(
            request->operation.directory_next.descriptor,
            request->operation.directory_next.buffer,
            request->operation.directory_next.count
        );
#endif
        break;
#endif
    default:
        result.head.error_code = EINVAL;
        return result;
    }
    if (result.head.value < 0) {
        result.head.error_code = errno;
    }
    return result;
}

static int wf_file_wait_ready(const wf_file_request *request) {
    struct pollfd descriptor;
    int waited;
    memset(&descriptor, 0, sizeof(descriptor));
    switch (request->kind) {
    case WF_FILE_READ:
        descriptor.fd = request->operation.read.descriptor;
        descriptor.events = POLLIN;
        break;
    case WF_FILE_PREAD:
        descriptor.fd = request->operation.pread.descriptor;
        descriptor.events = POLLIN;
        break;
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        descriptor.fd = request->operation.directory_next.descriptor;
        descriptor.events = POLLIN;
        break;
#endif
    case WF_FILE_WRITE:
        descriptor.fd = request->operation.write.descriptor;
        descriptor.events = POLLOUT;
        break;
    case WF_FILE_PWRITE:
        descriptor.fd = request->operation.pwrite.descriptor;
        descriptor.events = POLLOUT;
        break;
    default:
        return EINVAL;
    }
    do {
        waited = WF_COMPLETION_POLL(&descriptor, 1, -1);
    } while (waited < 0 && errno == EINTR);
    return waited > 0 ? 0 : errno;
}

wf_file_result wf_file_execute_direct(const wf_file_request *request) {
    wf_file_result result;
    if (!wf_file_request_valid(request)) {
        memset(&result, 0, sizeof(result));
        result.head.kind = request == NULL ? 0 : request->kind;
        result.head.value = -1;
        result.head.error_code = EINVAL;
        return result;
    }
    for (;;) {
        int readiness_error;
        result = wf_file_execute_once(request);
        if (result.head.value >= 0 || result.head.error_code == 0) {
            return result;
        }
        switch (request->kind) {
        case WF_FILE_READ:
        case WF_FILE_WRITE:
        case WF_FILE_PREAD:
        case WF_FILE_PWRITE:
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
        case WF_FILE_DIRECTORY_NEXT:
#endif
            break;
        default:
            return result;
        }
        if (result.head.error_code == EINTR) {
            continue;
        }
        if (result.head.error_code != EAGAIN
#if defined(EWOULDBLOCK) && EWOULDBLOCK != EAGAIN
            && result.head.error_code != EWOULDBLOCK
#endif
        ) {
            return result;
        }
        readiness_error = wf_file_wait_ready(request);
        if (readiness_error != 0) {
            result.head.error_code = readiness_error;
            return result;
        }
    }
}


/* The monotonic clock, which is a host call and therefore this leaf's.
 *
 * Two readers: the helper policy's measurement of what one host call cost, and
 * the bridge's bounded look at its own record before it announces sleep. Both
 * are shared code, so the clock is declared in `file_adapter.h` beside the
 * execution leaf and answered here.
 *
 * WF_FILE_MONOTONIC_NS is a named seam of the same class as WF_COMPLETION_PREAD
 * and WF_COMPLETION_POLL: a build may answer it with a scripted clock so that
 * the growth rule can be tested for what it decides rather than for how fast
 * the machine running the test happened to be.  The shipped build reads the
 * host monotonic clock and nothing else. */
#if !defined(WF_FILE_MONOTONIC_NS)
static uint64_t wf_file_monotonic_ns_host(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return 0;
    }
    return (uint64_t)now.tv_sec * 1000000000u + (uint64_t)now.tv_nsec;
}
#define WF_FILE_MONOTONIC_NS wf_file_monotonic_ns_host
#else
extern uint64_t WF_FILE_MONOTONIC_NS(void);
#endif

uint64_t wf_file_monotonic_ns(void) {
    return WF_FILE_MONOTONIC_NS();
}
