/* Native Windows proof that a submit beyond this target's core capacity waits
 * inside the bridge and settles once another owner joins.
 *
 * It used to prove the shape of a verdict the emitted code interpreted: a
 * DIRECT_ONLY answer that selected the inline arm and left the caller's token
 * untouched, and a WAIT answer the emitted fork retried.  Both are retired by
 * the owner's ruling recorded in `research/investigations/io-model/
 * PARK-ON-MISS.md` section 8 -- one lowering per operation, no inline arm and
 * no verdict fork -- so a submit answers nothing and does not return until the
 * record is the target's.  That is what is proved here instead, and the
 * assertions of the retired shape are gone rather than weakened.  The shapes
 * this target still refuses outright (an empty transfer, an offset the native
 * type cannot express) stop the process until slice 3's Windows record port
 * lets the bridge publish them, so they cannot be probed in process. */

#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "bridge.h"
#include "windows_completion.h"
#include "windows_runtime.h"

#include <windows.h>

#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WF_WINDOWS_BRIDGE_PROBE_CAPACITY 64u
#define WF_WINDOWS_BRIDGE_PROBE_TIMEOUT_MS 5000u

_Static_assert(WF_COMPLETION_SUBMIT_DIRECT_ONLY == 0, "stable direct verdict");
_Static_assert(WF_COMPLETION_SUBMIT_ACCEPTED == 1, "stable accepted verdict");
_Static_assert(
    WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY == 2,
    "stable core-capacity verdict"
);

typedef struct wf_windows_bridge_probe_submit {
    int descriptor;
    unsigned char *buffer;
    wf_completion_token *token;
    volatile LONG returned;
} wf_windows_bridge_probe_submit;

static DWORD WINAPI wf_windows_bridge_probe_submit_full(void *opaque) {
    wf_windows_bridge_probe_submit *submit =
        (wf_windows_bridge_probe_submit *)opaque;
    wf__completion_file_pread_submit(
        submit->descriptor,
        submit->buffer,
        1,
        0,
        submit->token
    );
    InterlockedExchange(&submit->returned, 1);
    return 0;
}

static int wf_windows_bridge_probe_fail(const char *message) {
    fprintf(stderr, "windows bridge capacity probe: %s\n", message);
    return 1;
}

int main(int argc, char **argv) {
    HANDLE fixture;
    DWORD written = 0;
    HANDLE native;
    int descriptor;
    wf_completion_token tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY + 1u];
    unsigned char buffers[WF_WINDOWS_BRIDGE_PROBE_CAPACITY + 1u];
    wf_windows_bridge_probe_submit full;
    HANDLE submit_thread;
    DWORD waited;
    size_t index;

    if (argc != 2) {
        return wf_windows_bridge_probe_fail("expected one fixture path");
    }
    fixture = CreateFileA(
        argv[1],
        GENERIC_WRITE,
        0,
        NULL,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );
    if (fixture == INVALID_HANDLE_VALUE) {
        return wf_windows_bridge_probe_fail("could not create the fixture");
    }
    if (WriteFile(fixture, "x", 1, &written, NULL) == FALSE
        || written != 1 || CloseHandle(fixture) == FALSE) {
        return wf_windows_bridge_probe_fail("could not publish the fixture");
    }

    native = CreateFileA(
        argv[1],
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        NULL
    );
    if (native == INVALID_HANDLE_VALUE) {
        return wf_windows_bridge_probe_fail("could not open an overlapped file");
    }
    descriptor = _open_osfhandle((intptr_t)native, _O_RDONLY | _O_BINARY);
    if (descriptor < 0) {
        (void)CloseHandle(native);
        return wf_windows_bridge_probe_fail("could not adopt the file handle");
    }
    if (wf__windows_completion_associate_descriptor(descriptor) != 0) {
        (void)_close(descriptor);
        return wf_windows_bridge_probe_fail("could not associate the file");
    }
    /* The direct pread that used to warm this handle went with the direct
     * family (design section 8); the submitted reads below are the only route
     * an associated handle has now. */
    memset(tokens, 0, sizeof(tokens));
    memset(buffers, 0, sizeof(buffers));
    for (index = 0; index < WF_WINDOWS_BRIDGE_PROBE_CAPACITY; ++index) {
        wf__completion_file_pread_submit(
            descriptor,
            &buffers[index],
            1,
            0,
            &tokens[index]
        );
    }

    memset(&full, 0, sizeof(full));
    full.descriptor = descriptor;
    full.buffer = &buffers[WF_WINDOWS_BRIDGE_PROBE_CAPACITY];
    full.token = &tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY];
    full.returned = 0;
    submit_thread = CreateThread(
        NULL,
        0,
        wf_windows_bridge_probe_submit_full,
        &full,
        0,
        NULL
    );
    if (submit_thread == NULL) {
        return wf_windows_bridge_probe_fail("could not start the boundary submit");
    }
    /* The core is full, so the boundary submit is inside the bridge's own
     * capacity wait.  It has no verdict to hand back and must not return
     * until a join elsewhere retires a slot. */
    Sleep(50);
    if (InterlockedCompareExchange(&full.returned, 0, 0) != 0) {
        return wf_windows_bridge_probe_fail(
            "the submit beyond core capacity returned before a slot was free"
        );
    }

    {
        int64_t value = -1;
        int error_code = -1;
        wf__completion_file_join(&tokens[0], &value, &error_code);
        if (value != 1 || error_code != 0
            || buffers[0] != (unsigned char)'x') {
            return wf_windows_bridge_probe_fail(
                "the capacity-releasing operation did not complete exactly"
            );
        }
    }
    waited = WaitForSingleObject(
        submit_thread,
        WF_WINDOWS_BRIDGE_PROBE_TIMEOUT_MS
    );
    if (waited != WAIT_OBJECT_0) {
        fputs(
            "windows bridge capacity probe: full submit did not return\n",
            stderr
        );
        ExitProcess(1);
    }
    (void)CloseHandle(submit_thread);

    for (index = 1; index <= WF_WINDOWS_BRIDGE_PROBE_CAPACITY; ++index) {
        int64_t value = -1;
        int error_code = -1;
        wf__completion_file_join(&tokens[index], &value, &error_code);
        if (value != 1 || error_code != 0
            || buffers[index] != (unsigned char)'x') {
            return wf_windows_bridge_probe_fail(
                "an accepted operation did not complete exactly"
            );
        }
    }
    if (wf__completion_file_submissions()
            != WF_WINDOWS_BRIDGE_PROBE_CAPACITY + 1u
        || wf__completion_publications()
            != WF_WINDOWS_BRIDGE_PROBE_CAPACITY + 1u) {
        return wf_windows_bridge_probe_fail(
            "submission and publication counts do not close"
        );
    }
    if (wf__completion_file_fallback_submissions() != 0) {
        return wf_windows_bridge_probe_fail(
            "an IOCP-eligible request used a direct fallback"
        );
    }
    {
        wf_completion_token close_token;
        int64_t value = -1;
        int error_code = -1;
        memset(&close_token, 0, sizeof(close_token));
        wf__completion_file_close_submit(descriptor, &close_token);
        wf__completion_file_join(&close_token, &value, &error_code);
        if (value != 0 || error_code != 0) {
            return wf_windows_bridge_probe_fail(
                "could not close the descriptor"
            );
        }
    }
    if (DeleteFileA(argv[1]) == FALSE) {
        return wf_windows_bridge_probe_fail("could not remove the fixture");
    }
    return 0;
}
