/* Native Windows proof that a full compiler completion window asks its source
 * owner to release capacity without modifying the rejected request's token. */

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
    volatile LONG verdict;
} wf_windows_bridge_probe_submit;

static DWORD WINAPI wf_windows_bridge_probe_submit_full(void *opaque) {
    wf_windows_bridge_probe_submit *submit =
        (wf_windows_bridge_probe_submit *)opaque;
    int verdict = wf__completion_file_pread_submit(
        submit->descriptor,
        submit->buffer,
        1,
        0,
        submit->token
    );
    InterlockedExchange(&submit->verdict, (LONG)verdict);
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
    unsigned char direct_buffer = 0;
    wf_completion_token local_token;
    wf_completion_token local_untouched;
    wf_completion_token tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY + 1u];
    wf_completion_token untouched;
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
    if (wf__completion_file_pread_direct(
            descriptor,
            &direct_buffer,
            1,
            0
        ) != 1
        || direct_buffer != (unsigned char)'x') {
        return wf_windows_bridge_probe_fail(
            "direct pread on the associated handle did not complete exactly"
        );
    }
    local_token.slot = UINT32_C(0x5a5aa5a5);
    local_token.generation = UINT64_C(0x5a5aa5a50ff00ff0);
    local_untouched = local_token;
    if (wf__completion_file_pread_submit(
            descriptor,
            NULL,
            0,
            0,
            &local_token
        ) != WF_COMPLETION_SUBMIT_DIRECT_ONLY
        || wf__completion_file_pread_submit(
            descriptor,
            &direct_buffer,
            UINT64_C(0x100000000),
            0,
            &local_token
        ) != WF_COMPLETION_SUBMIT_DIRECT_ONLY) {
        return wf_windows_bridge_probe_fail(
            "a locally ineligible request did not remain direct-only"
        );
    }
    if (local_token.slot != local_untouched.slot
        || local_token.generation != local_untouched.generation) {
        return wf_windows_bridge_probe_fail(
            "a direct-only verdict modified the caller's token"
        );
    }

    memset(tokens, 0, sizeof(tokens));
    memset(buffers, 0, sizeof(buffers));
    for (index = 0; index < WF_WINDOWS_BRIDGE_PROBE_CAPACITY; ++index) {
        if (wf__completion_file_pread_submit(
                descriptor,
                &buffers[index],
                1,
                0,
                &tokens[index]
            ) != WF_COMPLETION_SUBMIT_ACCEPTED) {
            return wf_windows_bridge_probe_fail(
                "a slot below the core capacity was not accepted"
            );
        }
    }

    tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY].slot = UINT32_C(0xa5a55a5a);
    tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY].generation =
        UINT64_C(0xa5a55a5af00ff00f);
    untouched = tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY];
    memset(&full, 0, sizeof(full));
    full.descriptor = descriptor;
    full.buffer = &buffers[WF_WINDOWS_BRIDGE_PROBE_CAPACITY];
    full.token = &tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY];
    full.verdict = -1;
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
    if (InterlockedCompareExchange(&full.verdict, 0, 0)
        != WF_COMPLETION_SUBMIT_WAIT_CORE_CAPACITY) {
        return wf_windows_bridge_probe_fail(
            "the operation beyond core capacity did not return WAIT"
        );
    }
    if (tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY].slot != untouched.slot
        || tokens[WF_WINDOWS_BRIDGE_PROBE_CAPACITY].generation
            != untouched.generation) {
        return wf_windows_bridge_probe_fail(
            "the WAIT verdict modified the caller's token"
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
    if (wf__completion_file_pread_submit(
            full.descriptor,
            full.buffer,
            1,
            0,
            full.token
        ) != WF_COMPLETION_SUBMIT_ACCEPTED) {
        return wf_windows_bridge_probe_fail(
            "the exact request was not accepted after capacity was released"
        );
    }

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
    if (wf__completion_file_close_direct(descriptor) != 0) {
        return wf_windows_bridge_probe_fail("could not close the descriptor");
    }
    if (DeleteFileA(argv[1]) == FALSE) {
        return wf_windows_bridge_probe_fail("could not remove the fixture");
    }
    return 0;
}
