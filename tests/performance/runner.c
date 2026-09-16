/* Linux paired regression process. Only call() is timed; preparation,
 * checking, final release, printing and shutdown are outside the interval.
 * The explicit slowdown control is used only to calibrate the instrument. */
#define _POSIX_C_SOURCE 200809L
#include "../programs/compute/oracle.h"
#include <inttypes.h>
#include <stdint.h>
#include <string.h>
#include <time.h>

extern int wf__floor_run(int, char **);

static uint64_t clock_ns(clockid_t clock) {
    struct timespec value;
    if (clock_gettime(clock, &value)) wf_oracle_fail("performance clock failed");
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
}

int wf__main_body(int argc, char **argv) {
    if (argc == 2 && !strcmp(argv[1], "verify")) {
        printf("%s oracle PASS: compared=%zu\n", wf_oracle_name, wf_oracle_verify());
        return 0;
    }
    if (argc != 5 && argc != 6)
        wf_oracle_fail("usage: image measure baseline|candidate WIDTH PASS [slow]");
    if (strcmp(argv[1], "measure") ||
        (strcmp(argv[2], "baseline") && strcmp(argv[2], "candidate")) ||
        (strcmp(argv[3], "1") && strcmp(argv[3], "2") && strcmp(argv[3], "4")) ||
        strlen(argv[4]) != 1 || argv[4][0] < '0' || argv[4][0] > '4' ||
        (argc == 6 && strcmp(argv[5], "slow")))
        wf_oracle_fail("invalid performance process identity");
    const char *workers = getenv("WF_WORKERS");
    if (!workers || strcmp(workers, argv[3])) wf_oracle_fail("worker identity mismatch");
    fprintf(stderr, "%s: %s; wall=CLOCK_MONOTONIC cpu=CLOCK_PROCESS_CPUTIME_ID\n",
            wf_oracle_name, wf_oracle_fixture);
    wf_oracle_prepare();
    for (unsigned sample = 0; sample <= 5; ++sample) {
        uint64_t cpu_start = clock_ns(CLOCK_PROCESS_CPUTIME_ID);
        uint64_t wall_start = clock_ns(CLOCK_MONOTONIC);
        size_t count = wf_oracle_call();
        if (argc == 6) {
            /* Deliberately repeat real WF work and its intermediate cleanup.
             * This is a visible instrument control, never a compiler option. */
            if (wf_oracle_check() != count) wf_oracle_fail("slow control output extent");
            if (wf_oracle_call() != count) wf_oracle_fail("slow control call extent");
        }
        uint64_t wall = clock_ns(CLOCK_MONOTONIC) - wall_start;
        uint64_t cpu = clock_ns(CLOCK_PROCESS_CPUTIME_ID) - cpu_start;
        if (!count || wf_oracle_check() != count) wf_oracle_fail("output comparison extent");
        printf("%s\t%s\t%s\t%s\t%u\t%" PRIu64 "\t%" PRIu64 "\t%zu\n",
               wf_oracle_name, argv[2], argv[3], argv[4], sample, wall, cpu, count);
    }
    wf_oracle_finish();
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
