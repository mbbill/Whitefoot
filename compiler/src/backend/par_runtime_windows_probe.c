/* Native Windows compute-pool identity observer.
 *
 * This file is test support, not part of a delivered Whitefoot executable.
 * The Windows CI gate links it beside one real `whitefootc --par --emit-llvm`
 * module and the same compiler-owned runtime units the production driver
 * carries.  At normal process exit it reads runtime-owned counters, so correct
 * program bytes cannot hide a weak sequential fallback or an owner-only run.
 */

#include <stdio.h>
#include <stdlib.h>

#if !defined(_WIN32)
#error "par_runtime_windows_probe.c requires a Windows target"
#endif
#if !defined(WF_PAR_IDENTITY_PROBE)
#error "par_runtime_windows_probe.c requires WF_PAR_IDENTITY_PROBE"
#endif

#define WF_PAR_PROBE_FAILURE_STATUS 86

unsigned long wf__par_grant_count(void);
unsigned long wf__par_started_worker_count(void);
unsigned long wf__par_worker_execution_count(void);

static void wf_par_probe_report(void) {
    unsigned long started = wf__par_started_worker_count();
    unsigned long executed = wf__par_worker_execution_count();
    unsigned long grants = wf__par_grant_count();
    int passed = started > 0 && executed > 0 && grants > 0;

    fprintf(
        stderr,
        "windows-native-par-probe status=%s started=%lu executed=%lu grants=%lu\n",
        passed ? "pass" : "fail",
        started,
        executed,
        grants
    );
    if (!passed) {
        fflush(stderr);
        _Exit(WF_PAR_PROBE_FAILURE_STATUS);
    }
}

/* Clang lowers this constructor into the MSVC CRT initializer table.  It only
 * registers the observer: the counters are read after wmain and every source
 * join have returned, never while a task may still be in flight. */
__attribute__((constructor)) static void wf_par_probe_register(void) {
    if (atexit(wf_par_probe_report) != 0) {
        fputs(
            "windows-native-par-probe status=fail reason=registration\n",
            stderr
        );
        fflush(stderr);
        _Exit(WF_PAR_PROBE_FAILURE_STATUS);
    }
}
