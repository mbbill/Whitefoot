/* Native probe for the recursive-component budget the emitted entry queries.
 * Include the maintained core so the probe reads the delivered definition
 * rather than a copy of its arithmetic. The budget is process policy: it
 * selects how much of an already admitted program is actualized in parallel,
 * so the probe asserts the answer for a configured pool width and nothing
 * about acceptance. Keep this test while the core owns that answer. */
#include "core.c"

#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    char *end = NULL;
    unsigned long expected;
    uint64_t budget;
    uint64_t again;
    if (argc != 2) {
        (void)fprintf(stderr, "recursion budget probe: expected one argument\n");
        return 1;
    }
    expected = strtoul(argv[1], &end, 10);
    if (end == argv[1] || *end != '\0') {
        (void)fprintf(stderr, "recursion budget probe: unreadable expectation\n");
        return 1;
    }
    budget = wf__par_recursion_budget();
    if (budget != (uint64_t)expected) {
        (void)fprintf(
            stderr,
            "recursion budget probe: budget=%llu expected=%lu lanes=%d\n",
            (unsigned long long)budget,
            expected,
            wf__sched_lanes()
        );
        return 1;
    }
    /* One process answers one budget: every component entry of one run must
     * cut at the same depth, and a second call may not start a second pool. */
    again = wf__par_recursion_budget();
    if (again != budget) {
        (void)fprintf(stderr, "recursion budget probe: answer moved\n");
        return 1;
    }
    (void)printf("recursion budget probe: PASS\n");
    return 0;
}
