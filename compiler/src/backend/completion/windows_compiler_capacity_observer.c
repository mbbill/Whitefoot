/* Test-only observer for compiler-emitted Windows core-capacity recovery.
 *
 * The native CI gate compiles this beside an emitted Whitefoot module.  It
 * renames the core's implementation of wf_completion_claim and the bridge's
 * implementation of wf__completion_file_join, leaving these wrappers at the
 * emitted ABI.  Thus this records executed status-2 claims and joins rather
 * than inferring either fact from LLVM text or a standalone C bridge probe.
 */

#define WIN32_LEAN_AND_MEAN

#include "bridge.h"
#include "windows_completion.h"

#include <stdio.h>
#include <stdlib.h>
#include <windows.h>

#if !defined(_WIN32)
#error "windows_compiler_capacity_observer.c requires a Windows target"
#endif

#define WF_WINDOWS_CAPACITY_OBSERVER_FAILURE_STATUS 87

enum wf_completion_claim_result
wf__capacity_observed_completion_claim(
    wf_completion_runtime *runtime,
    wf_completion_token *token
);

void wf__capacity_observed_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
);

static volatile LONG64 wf_capacity_waits;
static volatile LONG64 wf_capacity_consumes;

enum wf_completion_claim_result wf_completion_claim(
    wf_completion_runtime *runtime,
    wf_completion_token *token
) {
    enum wf_completion_claim_result result =
        wf__capacity_observed_completion_claim(runtime, token);
    if (result == WF_COMPLETION_CLAIM_WAIT_CAPACITY) {
        (void)InterlockedIncrement64(&wf_capacity_waits);
    }
    return result;
}

void wf__completion_file_join(
    const void *token_storage,
    int64_t *value,
    int *error_code
) {
    wf__capacity_observed_file_join(token_storage, value, error_code);
    (void)InterlockedIncrement64(&wf_capacity_consumes);
}

static void wf_capacity_observer_report(void) {
    unsigned long long waits = (unsigned long long)InterlockedCompareExchange64(
        &wf_capacity_waits,
        0,
        0
    );
    unsigned long long consumes =
        (unsigned long long)InterlockedCompareExchange64(
            &wf_capacity_consumes,
            0,
            0
        );
    unsigned long long accepted =
        (unsigned long long)wf__completion_file_submissions();
    unsigned long long publications =
        (unsigned long long)wf__completion_publications();
    unsigned long long fallback =
        (unsigned long long)wf__completion_file_fallback_submissions();
    int passed = waits > 0 && accepted == 3u && publications == accepted
        && consumes == accepted && fallback == 0u;

    fprintf(
        stderr,
        "windows-native-completion-capacity status=%s accepted=%llu "
        "publications=%llu consumes=%llu capacity_waits=%llu fallback=%llu\n",
        passed ? "pass" : "fail",
        accepted,
        publications,
        consumes,
        waits,
        fallback
    );
    if (!passed) {
        fflush(stderr);
        _Exit(WF_WINDOWS_CAPACITY_OBSERVER_FAILURE_STATUS);
    }
}

/* The report runs after command main returns and after all source-level joins
 * have completed, so every accepted target operation must have a matching
 * terminal publication and successful consume. */
__attribute__((constructor)) static void wf_capacity_observer_register(void) {
    if (atexit(wf_capacity_observer_report) != 0) {
        fputs(
            "windows-native-completion-capacity status=fail reason=registration\n",
            stderr
        );
        fflush(stderr);
        _Exit(WF_WINDOWS_CAPACITY_OBSERVER_FAILURE_STATUS);
    }
}
