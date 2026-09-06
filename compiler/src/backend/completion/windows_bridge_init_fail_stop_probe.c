/* Native fault-injection proof that a bridge with no engine is fatal.
 *
 * A bridge that cannot initialize leaves no engine to run an operation and no
 * drain to publish it, so it is a trusted-computing-base failure and
 * terminates deterministically where the floor does (design
 * `research/investigations/io-model/PARK-ON-MISS.md` section 7, "Every submit
 * path ends in a published record").  This probe is what makes that a fact
 * about the shipped bridge rather than a sentence.
 *
 * Two injections, and the second is why: a *refused ring* is no longer a
 * failure on this platform.  `WF_IO_NO_NATIVE_RING` deliberately runs a Windows
 * process on the bounded adapter, and the gate runs it, so a completion port
 * that cannot be created degrades to that same supported route rather than
 * stopping the process.  What has to stop the process is a bridge with neither
 * engine, and reaching it takes both the ring's initializer and the adapter's.
 *
 * The CI build renames only `bridge.c`'s two initializers and its abort to
 * these symbols, leaving every other linked runtime unit on its production
 * implementation.  The exit status is 86, stdout is empty, and stderr carries
 * exactly the one line the production fail-stop writes before it ends the
 * process (`bridge.c`, `wf_bridge_fail`); a bridge that returned from the
 * submit instead would be visible as the 87 below.  The abort is what is
 * renamed rather than that fail-stop, so the line under test is the shipped
 * one: only the last instruction of the shipped path is replaced.
 */

#if defined(WF_WINDOWS_BRIDGE_INIT_PROBE_DECLARATIONS)

/* `bridge.c` normally obtains abort's prototype from the CRT. A command-line
 * rename can either hide that prototype or incorrectly retain its dllimport
 * attribute for the probe-owned replacement. Include the CRT declaration
 * first, then make the bridge-only symbol substitutions. */
#include <stdlib.h>

_Noreturn void wf_windows_bridge_init_probe_abort(void);

#define wf_windows_iocp_init wf_windows_bridge_init_probe_iocp_init
#define wf_file_adapter_init wf_windows_bridge_init_probe_adapter_init
#define abort wf_windows_bridge_init_probe_abort

#else

#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "bridge.h"
#include "contract.h"

/* Derive both injected declarations from the production ABI so a signature
 * change cannot leave the cross-translation-unit probe stale. */
#define wf_windows_iocp_init wf_windows_bridge_init_probe_iocp_init
#include "windows_iocp.h"
#undef wf_windows_iocp_init

#define wf_file_adapter_init wf_windows_bridge_init_probe_adapter_init
#include "file_adapter.h"
#undef wf_file_adapter_init

#include <stdalign.h>
#include <stddef.h>
#include <windows.h>

int wf_windows_bridge_init_probe_iocp_init(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    DWORD concurrency
) {
    (void)adapter;
    (void)runtime;
    (void)concurrency;
    return ERROR_NOT_ENOUGH_MEMORY;
}

int wf_windows_bridge_init_probe_adapter_init(
    wf_file_adapter *adapter,
    wf_completion_runtime *runtime,
    size_t helper_capacity,
    size_t helper_count
) {
    (void)adapter;
    (void)runtime;
    (void)helper_capacity;
    (void)helper_count;
    return ERROR_NOT_ENOUGH_MEMORY;
}

_Noreturn void wf_windows_bridge_init_probe_abort(void) {
    ExitProcess(86u);
}

int main(void) {
    _Alignas(WF_COMPLETION_RECORD_ALIGN)
        unsigned char record[WF_COMPLETION_RECORD_BYTES];
    /* A submit is where the bridge is required, and it must not return. */
    wf__completion_file_close_submit(-1, record);
    return 87;
}

#endif
