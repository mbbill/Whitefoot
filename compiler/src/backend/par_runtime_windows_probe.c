/* Native Windows compute-pool identity observer.
 *
 * This file is test support, not part of a delivered Whitefoot executable.
 * The Windows CI gate links it beside one real `whitefootc --par --emit-llvm`
 * module and the same compiler-owned runtime units the production driver
 * carries.  At normal process exit it reads runtime-owned counters, so correct
 * program bytes cannot hide a weak sequential fallback or an owner-only run.
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <windows.h>

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

#if defined(WF_PAR_MIXED_PROBE)
void wf__mixed_observed_par_publish(void *frame, void (*fn)(void *));
void wf__mixed_observed_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
);
void wf__mixed_observed_file_open_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
);
uint64_t wf__completion_file_fallback_submissions(void);
uint64_t wf__completion_file_helper_executions(void);
uint64_t wf__completion_file_submissions(void);
uint64_t wf__completion_publications(void);
uint64_t wf__completion_windows_iocp_in_flight(void);

static volatile LONG64 wf_par_mixed_publishes;
static volatile LONG64 wf_par_mixed_overlap_publishes;
static volatile LONG64 wf_par_mixed_consumes;

/* The observed fixture has 1024 compute pairs. Each pair publishes once while
 * its source-earlier positioned read is outstanding. The completion ledger
 * also contains the one source-earlier open that the emitter can overlap; the
 * source-last open and positioned reads remain direct by construction. */
#define WF_PAR_MIXED_EXPECTED_PUBLISHES UINT64_C(1024)
#define WF_PAR_MIXED_EXPECTED_COMPLETIONS UINT64_C(1025)

void wf__par_publish(void *frame, void (*fn)(void *)) {
    (void)InterlockedIncrement64(&wf_par_mixed_publishes);
    if (wf__completion_windows_iocp_in_flight() != 0) {
        (void)InterlockedIncrement64(&wf_par_mixed_overlap_publishes);
    }
    wf__mixed_observed_par_publish(frame, fn);
}

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    wf__mixed_observed_file_join(token_storage, value, error_code);
    (void)InterlockedIncrement64(&wf_par_mixed_consumes);
}

void wf__completion_file_open_join(
    const void *token_storage,
    int64_t *value,
    int *error_code,
    unsigned *open_outcome
) {
    wf__mixed_observed_file_open_join(
        token_storage,
        value,
        error_code,
        open_outcome
    );
    (void)InterlockedIncrement64(&wf_par_mixed_consumes);
}
#endif

static void wf_par_probe_report(void) {
    unsigned long started = wf__par_started_worker_count();
    unsigned long executed = wf__par_worker_execution_count();
    unsigned long grants = wf__par_grant_count();
#if defined(WF_PAR_MIXED_PROBE)
    unsigned long long publishes =
        (unsigned long long)InterlockedCompareExchange64(
            &wf_par_mixed_publishes,
            0,
            0
        );
    unsigned long long overlap_publishes =
        (unsigned long long)InterlockedCompareExchange64(
            &wf_par_mixed_overlap_publishes,
            0,
            0
        );
    unsigned long long consumes =
        (unsigned long long)InterlockedCompareExchange64(
            &wf_par_mixed_consumes,
            0,
            0
        );
    unsigned long long submissions =
        (unsigned long long)wf__completion_file_submissions();
    unsigned long long publications =
        (unsigned long long)wf__completion_publications();
    unsigned long long fallback =
        (unsigned long long)wf__completion_file_fallback_submissions();
    unsigned long long helpers =
        (unsigned long long)wf__completion_file_helper_executions();
    int passed = started > 0 && executed > 0 && grants > 0
        && publishes == WF_PAR_MIXED_EXPECTED_PUBLISHES
        && overlap_publishes == publishes
        && submissions == WF_PAR_MIXED_EXPECTED_COMPLETIONS
        && publications == WF_PAR_MIXED_EXPECTED_COMPLETIONS
        && consumes == WF_PAR_MIXED_EXPECTED_COMPLETIONS
        && helpers == 1 && fallback == 0;

    fprintf(
        stderr,
        "windows-native-mixed-probe status=%s started=%lu executed=%lu "
        "grants=%lu publishes=%llu overlap_publishes=%llu submissions=%llu "
        "publications=%llu consumes=%llu helpers=%llu fallback=%llu\n",
        passed ? "pass" : "fail",
        started,
        executed,
        grants,
        publishes,
        overlap_publishes,
        submissions,
        publications,
        consumes,
        helpers,
        fallback
    );
#else
    int passed = started > 0 && executed > 0 && grants > 0;

    fprintf(
        stderr,
        "windows-native-par-probe status=%s started=%lu executed=%lu grants=%lu\n",
        passed ? "pass" : "fail",
        started,
        executed,
        grants
    );
#endif
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
