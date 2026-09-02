/* Native fault-injection proof that a Windows bridge initialization failure
 * is fatal. The CI build renames only windows_bridge.c's IOCP initializer and
 * abort calls to these symbols, leaving the other linked runtime units on
 * their production implementations. */

#if defined(WF_WINDOWS_BRIDGE_INIT_PROBE_DECLARATIONS)

/* windows_bridge.c normally obtains abort's prototype from the CRT. A
 * command-line rename can either hide that prototype or incorrectly retain
 * its dllimport attribute for the probe-owned replacement. Include the CRT
 * declaration first, then make both bridge-only symbol substitutions. */
#include <stdlib.h>

_Noreturn void wf_windows_bridge_init_probe_abort(void);

#define wf_windows_iocp_init wf_windows_bridge_init_probe_iocp_init
#define abort wf_windows_bridge_init_probe_abort

#else

#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "bridge.h"
#include "windows_completion.h"

/* Derive the injected initializer declaration from the production ABI so a
 * signature change cannot leave the cross-translation-unit probe stale. */
#define wf_windows_iocp_init wf_windows_bridge_init_probe_iocp_init
#include "windows_iocp.h"
#undef wf_windows_iocp_init

#include <stddef.h>
#include <windows.h>

int wf_windows_bridge_init_probe_iocp_init(
    wf_windows_iocp_adapter *adapter,
    wf_completion_runtime *runtime,
    wf_windows_iocp_entry *entry_storage,
    size_t entry_capacity,
    DWORD concurrency
) {
    (void)adapter;
    (void)runtime;
    (void)entry_storage;
    (void)entry_capacity;
    (void)concurrency;
    return ERROR_NOT_ENOUGH_MEMORY;
}

_Noreturn void wf_windows_bridge_init_probe_abort(void) {
    ExitProcess(86u);
}

int main(void) {
    (void)wf__completion_window(0, 0, 0);
    return 87;
}

#endif
