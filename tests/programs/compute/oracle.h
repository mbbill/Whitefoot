/* Shared entry for complete-result compute oracles. The performance runner
 * defines WF_ORACLE_NO_MAIN and calls the explicit prepare/call/check API. */
#ifndef WHITEFOOT_COMPUTE_ORACLE_H
#define WHITEFOOT_COMPUTE_ORACLE_H
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
extern const char *const wf_oracle_name;
extern const char *const wf_oracle_fixture;
size_t wf_oracle_verify(void);
#ifdef WF_ORACLE_PERFORMANCE
void wf_oracle_prepare(void);
size_t wf_oracle_call(void);
size_t wf_oracle_check(void);
void wf_oracle_finish(void);
#endif
static _Noreturn void wf_oracle_fail(const char *message) {
    fprintf(stderr, "%s\n", message);
    exit(1);
}
#ifndef WF_ORACLE_NO_MAIN
extern int wf__floor_run(int, char **);
#ifdef WFB_ORACLE_PARALLEL
extern int wf__par_pool_active(void);
extern unsigned long wf__par_grants(void);
#endif
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    size_t compared = wf_oracle_verify();
#ifdef WFB_ORACLE_PARALLEL
    const char *workers = getenv("WF_WORKERS");
    if (workers && atoi(workers) > 1 &&
        (!wf__par_pool_active() || wf__par_grants() == 0))
        wf_oracle_fail("oracle did not exercise a worker");
    if (workers && atoi(workers) == 1 && wf__par_grants() != 0)
        wf_oracle_fail("pool-off oracle handed out work");
#endif
    printf("%s oracle PASS: compared=%zu\n", wf_oracle_name, compared);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
#endif
#endif
