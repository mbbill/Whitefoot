/* Native probe for the machine fact the idle window's admission test reads,
 * and for what the test does with it. Include the maintained core so the probe
 * reads the delivered rule rather than a copy of it.
 *
 * Two things are asserted, and neither of them is a property of the machine
 * this runs on. First, that `wf_prim_cpu_levels` answers at all: at least one
 * level, and the same count twice, because the rule reads it once at pool
 * start and a count that moved between two calls would mean the pool's
 * behaviour depended on which call it made. Second, the one-directional half
 * of the rule that can be checked anywhere: a pool that OPENED the window did
 * so on a pool that fits the CPUs and on CPUs that are alike, and the window
 * it opened is the compiled one. Nothing here asserts that this host is
 * uniform or that it is not -- an asymmetric host and a uniform host both pass
 * -- and nothing asserts the window is open, since a host with an unknown CPU
 * count legitimately has none.
 *
 * Built a second time with WF_PAR_IDLE_WINDOW_ON_ASYMMETRIC, the probe checks
 * the other direction of the knob instead: that arm withdraws the machine test
 * and nothing else, so a pool that fits must open the window whatever the
 * levels are. That is the arm the A/B twin measures with, and this is what
 * says the knob still means what its name says.
 *
 * Keep this probe while the core's window has an admission test to pass. */
#include "core.c"

#include <stdio.h>

static int fail(const char *why) {
    (void)fprintf(stderr, "cpu levels probe: %s\n", why);
    return 1;
}

int main(void) {
    unsigned levels = wf_prim_cpu_levels();
    unsigned again = wf_prim_cpu_levels();
    unsigned cpus = wf_prim_online_cpus();
    int lanes = wf__sched_lanes();
    uint64_t window;
    int fits;
    if (levels < 1u) {
        return fail("no performance level at all");
    }
    if (again != levels) {
        return fail("the level count moved between two calls");
    }
    /* Start the pool through the path an emitted program takes, then read the
     * window that start chose. It is written once, before any worker exists. */
    (void)wf__par_attach();
    window = wf__par_idle_window_us;
    fits = lanes >= 2 && cpus != 0u && (unsigned)lanes <= cpus;
    if (window != 0) {
        if (window != (uint64_t)WF_PAR_IDLE_WINDOW_US) {
            return fail("the open window is not the compiled one");
        }
        if (!fits) {
            return fail("the window opened for a pool that does not fit");
        }
#if !defined(WF_PAR_IDLE_WINDOW_ON_ASYMMETRIC)
        if (levels > 1u) {
            return fail("the window opened on a machine whose cores differ");
        }
#endif
    }
#if defined(WF_PAR_IDLE_WINDOW_ON_ASYMMETRIC)
    if (window == 0 && fits && (uint64_t)WF_PAR_IDLE_WINDOW_US != 0) {
        return fail("the twin knob withheld the window from a pool that fits");
    }
#endif
    (void)printf(
        "cpu levels probe: PASS levels=%u cpus=%u lanes=%d window_us=%llu\n",
        levels,
        cpus,
        lanes,
        (unsigned long long)window
    );
    return 0;
}
