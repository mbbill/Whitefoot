#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

/*
 * The Windows host leaf of the file adapter: one host call per request kind,
 * and nothing else.
 *
 * `file_adapter.c` owns the queue, the helpers, the claim and the progress
 * pass, and is one implementation for every platform.  This unit is the one
 * thing that cannot be, and `file_posix.c` is its twin.  Every call it makes is
 * one of `windows_runtime.c`'s workers, so the NtCreateFile relative open, the
 * WriteFile with the stream's own current position, and the directory batch are
 * written once each and reached from here.
 *
 * What this target does *not* qualify is refused as an outcome rather than as
 * a terminated process.  The emitter emits the whole seven-entry submit ABI on
 * every target and the Windows row admits an open, a positioned read, a stream
 * write, a close and a directory batch; a shape the row does not admit reaches
 * this switch and is published as a failed operation with the host's own
 * refusal code, exactly as `wf_bridge_complete_refused` publishes an offset the
 * target ABI cannot express.
 */

#include "file_adapter.h"
#include "../windows_runtime.h"

#include <windows.h>

#include <io.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

/* The kind the request names, refused where this target has no facility. */
static wf_file_result wf_file_windows_refused(
    const wf_file_request *request,
    DWORD error_code
) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    result.head.value = -1;
    result.head.error_code = (int)error_code;
    return result;
}

/* One positioned read, made to wait here rather than on the port.
 *
 * This is the route a Windows run takes when it has no completion ring at all
 * -- `WF_IO_NO_NATIVE_RING`, and the negative control the gate runs -- and the
 * route a run *with* a ring takes for every shape that ring refuses: a count
 * above MAXDWORD, a negative offset, a descriptor of a class it does not
 * carry.  So the handle read here may well be one the port already has, and
 * this call says so rather than hoping otherwise.
 *
 * The offset travels in an `OVERLAPPED` because that is how a Windows read
 * names one.  The event beside it is this call's own, so two reads of one
 * handle cannot take each other's completion -- and its low-order bit is set,
 * which is the documented way to say that this one operation posts no packet
 * to whatever completion port the handle is bound to.  Without that bit a read
 * here on a bound handle would deliver its completion to the ring's port, and
 * the reaper would recover a "record" by subtraction from an `OVERLAPPED` that
 * is a local of this frame.  A handle value with its low bits set still names
 * the same object to every wait, so `GetOverlappedResult` below is unaffected;
 * the bit belongs to the `OVERLAPPED`, and `CloseHandle` gets the event
 * itself. */
static wf_file_result wf_file_windows_pread(const wf_file_request *request) {
    wf_file_result result;
    OVERLAPPED overlapped;
    ULARGE_INTEGER offset;
    HANDLE handle;
    HANDLE event;
    DWORD transferred = 0;
    DWORD error_code = 0;

    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    result.head.value = -1;
    if (request->operation.pread.count == 0) {
        result.head.value = 0;
        return result;
    }
    if (request->operation.pread.count > (size_t)MAXDWORD
        || request->operation.pread.offset < 0) {
        result.head.error_code = ERROR_INVALID_PARAMETER;
        return result;
    }
    handle = wf__windows_completion_descriptor_handle(
        request->operation.pread.descriptor
    );
    if (handle == INVALID_HANDLE_VALUE) {
        result.head.error_code = ERROR_INVALID_HANDLE;
        return result;
    }
    event = CreateEventW(NULL, TRUE, FALSE, NULL);
    if (event == NULL) {
        result.head.error_code = (int)GetLastError();
        return result;
    }
    memset(&overlapped, 0, sizeof(overlapped));
    offset.QuadPart = (ULONGLONG)request->operation.pread.offset;
    overlapped.Offset = offset.LowPart;
    overlapped.OffsetHigh = offset.HighPart;
    /* The low bit is the "no completion packet for this operation" flag; see
     * the header comment. */
    overlapped.hEvent = (HANDLE)((uintptr_t)event | (uintptr_t)1);
    if (ReadFile(
            handle,
            request->operation.pread.buffer,
            (DWORD)request->operation.pread.count,
            NULL,
            &overlapped
        ) == FALSE) {
        error_code = GetLastError();
        if (error_code == ERROR_IO_PENDING) {
            error_code = GetOverlappedResult(
                             handle,
                             &overlapped,
                             &transferred,
                             TRUE
                         ) != FALSE
                ? 0u
                : GetLastError();
        }
    } else if (GetOverlappedResult(handle, &overlapped, &transferred, TRUE)
               == FALSE) {
        error_code = GetLastError();
    }
    (void)CloseHandle(event);
    /* Reaching the file boundary is a successful zero-byte read in this
     * contract; Windows reports it as ERROR_HANDLE_EOF. */
    if (error_code == ERROR_HANDLE_EOF) {
        error_code = 0;
        transferred = 0;
    }
    if (error_code != 0) {
        result.head.error_code = (int)error_code;
        return result;
    }
    result.head.value = (int64_t)transferred;
    return result;
}

/* One unpositioned stream read [SYS-15], made to wait here.
 *
 * A Windows read names its position in an `OVERLAPPED`, and this operation has
 * none to name: the position it reads at is the handle's own. The two shapes a
 * standard input can be are handled by one call, and neither is a fork of any
 * logic here.
 *
 * A console handle is not opened for overlapped I/O and has no overlapped
 * form at all, so `ReadFile` with a null `OVERLAPPED` is the only thing that
 * reads it; that call advances the console's own position and blocks this
 * helper's thread until the user supplies a line, which is exactly what the
 * bounded adapter's helper thread exists for.
 *
 * A redirected pipe or file is the same call. A pipe handle the process
 * inherited is likewise not opened overlapped, so a null `OVERLAPPED` reads it
 * at its own position and returns when the writer supplies bytes or closes;
 * a redirected regular file also advances the handle's own file pointer, which
 * is what an unpositioned read means. Passing an `OVERLAPPED` here would be
 * wrong in both cases rather than faster: on a synchronous handle Windows
 * ignores it for the wait and still uses the offset it carries, which would
 * turn a stream read into a positioned read at zero and re-read the first
 * bytes forever.
 *
 * End of stream is a successful zero-byte read in this contract. A pipe whose
 * writer closed reports `ERROR_BROKEN_PIPE`, which is that end and not a
 * failure, so it is normalized to zero here exactly as a file's
 * `ERROR_HANDLE_EOF` is. */
static wf_file_result wf_file_windows_read(const wf_file_request *request) {
    wf_file_result result;
    HANDLE handle;
    DWORD transferred = 0;
    DWORD error_code = 0;

    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    result.head.value = -1;
    if (request->operation.read.count == 0) {
        result.head.value = 0;
        return result;
    }
    if (request->operation.read.count > (size_t)MAXDWORD) {
        result.head.error_code = ERROR_INVALID_PARAMETER;
        return result;
    }
    handle = wf__windows_completion_descriptor_handle(
        request->operation.read.descriptor
    );
    if (handle == INVALID_HANDLE_VALUE) {
        result.head.error_code = ERROR_INVALID_HANDLE;
        return result;
    }
    if (ReadFile(
            handle,
            request->operation.read.buffer,
            (DWORD)request->operation.read.count,
            &transferred,
            NULL
        ) == FALSE) {
        error_code = GetLastError();
    }
    if (error_code == ERROR_HANDLE_EOF || error_code == ERROR_BROKEN_PIPE) {
        error_code = 0;
        transferred = 0;
    }
    if (error_code != 0) {
        result.head.error_code = (int)error_code;
        return result;
    }
    result.head.value = (int64_t)transferred;
    return result;
}

static wf_file_result wf_file_windows_open_at(const wf_file_request *request) {
    wf_file_result result;
    HANDLE root;
    int error_code = 0;
    unsigned open_outcome = WF_FILE_OPEN_FAILED;
    int descriptor;

    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    result.head.value = -1;
    root = wf__windows_completion_descriptor_handle(
        request->operation.open_at.directory
    );
    if (root == INVALID_HANDLE_VALUE) {
        result.head.error_code = ERROR_INVALID_HANDLE;
        result.head.open_outcome = WF_FILE_OPEN_FAILED;
        return result;
    }
    descriptor = wf__windows_completion_file_open_at_worker(
        root,
        request->operation.open_at.path,
        request->operation.open_at.flags,
        request->operation.open_at.mode,
        request->operation.open_at.has_mode,
        (unsigned)request->operation.open_at.expected_kind,
        request->operation.open_at.descriptor_class,
        &error_code,
        &open_outcome
    );
    result.head.error_code = error_code;
    result.head.open_outcome = (enum wf_file_open_outcome)open_outcome;
    result.head.value = descriptor;
    return result;
}

static wf_file_result wf_file_windows_write(const wf_file_request *request) {
    wf_file_result result;
    HANDLE handle;
    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    result.head.value = -1;
    handle = wf__windows_completion_descriptor_handle(
        request->operation.write.descriptor
    );
    if (handle == INVALID_HANDLE_VALUE) {
        result.head.error_code = ERROR_INVALID_HANDLE;
        return result;
    }
    result.head.value = wf__windows_completion_file_write_worker(
        handle,
        request->operation.write.buffer,
        (uint64_t)request->operation.write.count
    );
    if (result.head.value < 0) {
        result.head.error_code = *wf__windows_error_location();
    }
    return result;
}

static wf_file_result wf_file_windows_close(const wf_file_request *request) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    /* The registry row goes before the handle does, so the descriptor number
     * the host may hand out again starts with no class and no association. */
    wf__windows_completion_forget_descriptor(
        request->operation.close.descriptor
    );
    result.head.value = _close(request->operation.close.descriptor);
    if (result.head.value < 0) {
        result.head.error_code = ERROR_INVALID_HANDLE;
    }
    return result;
}

#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
static wf_file_result wf_file_windows_directory_next(
    const wf_file_request *request
) {
    wf_file_result result;
    memset(&result, 0, sizeof(result));
    result.head.kind = request->kind;
    if (request->operation.directory_next.count == 0) {
        result.head.value = 0;
        return result;
    }
    result.head.value = wf__windows_directory_batch(
        request->operation.directory_next.descriptor,
        request->operation.directory_next.buffer,
        (uint64_t)request->operation.directory_next.count,
        request->operation.directory_next.position
    );
    if (result.head.value < 0) {
        result.head.error_code = *wf__windows_error_location();
    }
    return result;
}
#endif

wf_file_result wf_file_execute_direct(wf_file_request *request) {
    if (!wf_file_request_valid(request)) {
        wf_file_result result;
        memset(&result, 0, sizeof(result));
        result.head.kind = request == NULL ? 0 : request->kind;
        result.head.value = -1;
        result.head.error_code = ERROR_INVALID_PARAMETER;
        return result;
    }
    switch (request->kind) {
    case WF_FILE_OPEN_AT:
        return wf_file_windows_open_at(request);
    case WF_FILE_PREAD:
        return wf_file_windows_pread(request);
    case WF_FILE_WRITE:
        return wf_file_windows_write(request);
    case WF_FILE_CLOSE:
        return wf_file_windows_close(request);
    case WF_FILE_READ:
        return wf_file_windows_read(request);
#if defined(WF_FILE_HAS_DIRECTORY_NEXT)
    case WF_FILE_DIRECTORY_NEXT:
        return wf_file_windows_directory_next(request);
#endif
    case WF_FILE_PWRITE:
    case WF_FILE_STATUS:
    case WF_FILE_SOCKET_LISTEN:
    case WF_FILE_SOCKET_ACCEPT:
    case WF_FILE_SOCKET_CONNECT:
    case WF_FILE_SOCKET_RECEIVE:
    case WF_FILE_SOCKET_SEND:
    case WF_FILE_SOCKET_SHUTDOWN:
    default:
        /* The shapes the Windows target row does not qualify: a positioned
         * write (the row's write is `write_once`, which has the stream's own
         * current position), a status (the row reports no `stat` record), and
         * the six TCP kinds, whose completion-port route is slice 3 of the
         * streams-and-TCP batch
         * (`research/investigations/io-model/NETWORK.md` §7).  Each is refused
         * as an outcome the program sees, because the emitter can name any
         * submit entry on any target and a refusal is not a defect of the
         * runtime.  No Windows program reaches the six: `backend/
         * qualification.rs` maps no TCP operation on the Windows column, so a
         * Windows link is refused at qualification and this arm is the
         * runtime's own honest answer rather than the program's stop. */
        return wf_file_windows_refused(request, ERROR_NOT_SUPPORTED);
    }
}

/* The monotonic clock, which is a host call and therefore this leaf's; see
 * `file_adapter.h` for its two shared readers.  The performance counter is
 * monotonic and its frequency is fixed for the life of the system, so the
 * frequency is asked for once. */
uint64_t wf_file_monotonic_ns(void) {
    static LARGE_INTEGER frequency;
    LARGE_INTEGER now;
    if (frequency.QuadPart == 0 && QueryPerformanceFrequency(&frequency) == FALSE) {
        return 0;
    }
    if (frequency.QuadPart == 0 || QueryPerformanceCounter(&now) == FALSE) {
        return 0;
    }
    return (uint64_t)((now.QuadPart / frequency.QuadPart) * 1000000000ll)
        + (uint64_t)(
              ((now.QuadPart % frequency.QuadPart) * 1000000000ll)
              / frequency.QuadPart
          );
}
