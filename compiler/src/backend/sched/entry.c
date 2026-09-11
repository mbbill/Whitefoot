/* Process settings and ordinary-call entry; no stack reservation. */
#include "entry.h"
#include "prim.h"
#include <stdio.h>
#include <stdlib.h>

int wf__sched_setting(
    const char *name,
    unsigned long ceiling,
    unsigned long *value
) {
    char text[WF_PRIM_SETTING_BYTES];
    int state = wf_prim_setting_text(name, text, sizeof(text));
    char *end = NULL;
    long written;
    if (state == 0 || (state == 1 && text[0] == '\0')) {
        return 0;
    }
    if (state == 1) {
        written = strtol(text, &end, 10);
        if (end != text && *end == '\0' && written >= 0
            && (unsigned long)written <= ceiling) {
            *value = (unsigned long)written;
            return 1;
        }
    }
    (void)fprintf(
        stderr,
        "whitefoot scheduler: %s must be an integer from 0 through %lu\n",
        name,
        ceiling
    );
    /* Invalid process settings are diagnosed before user code. Do not run
     * exit callbacks that could reenter this incomplete once-initializer. */
    (void)fflush(stderr);
    _Exit(1);
}

void wf__sched_once(unsigned *state, void (*body)(void)) {
    unsigned expected = 0;
    if (__atomic_load_n(state, __ATOMIC_ACQUIRE) == 2) return;
    if (__atomic_compare_exchange_n(state, &expected, 1, 0, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) {
        body();
        __atomic_store_n(state, 2, __ATOMIC_RELEASE);
    } else {
        while (__atomic_load_n(state, __ATOMIC_ACQUIRE) != 2) wf_prim_yield();
    }
}
__attribute__((weak)) size_t wf__floor_stack_bytes(void) { return 1024u * 1024u; }
__attribute__((weak)) unsigned long wf__sched_helper_ceiling(void) { return 0; }
static unsigned initialized;
static int lanes;
/* How much work one chunk must be worth before the splitter can afford
 * another, in the same unit the emitted weight operand counts: estimated IR
 * instructions of one iteration of the chunk body. `wf__par_split_budget` in
 * core.c applies it as span / ceil(work unit / weight), so this is a floor on
 * chunk size and never a ceiling on chunk count -- WF_PAR_SPLIT_OVERSUBSCRIBE
 * caps that -- and the two are combined with a minimum, so a range too small
 * to be worth splitting stays bounded by this term whatever the pool wants.
 *
 * It closes the over-split hazard: splitting a range that is not worth
 * splitting is a real regression, not a wash, and an allowance derived from
 * the lane count alone produces one on every small loop. 1,200,000 was
 * measured directly for that crossing at `f7127c03`, by sweeping the width of
 * a counted loop of estimated weight 150 over a fixed total amount of work and
 * comparing the split against the same program's sequential build:
 *
 *     width     2 000   8 000  32 000  128 000  512 000
 *     split    0.0831  0.0269  0.0116   0.0084   0.0072
 *     plain    0.0205  0.0196  0.0201   0.0201   0.0210
 *     ratio     4.1x    1.37x   0.58x    0.42x    0.34x
 *                loss    loss     win      win      win
 *
 * The crossing is near a width of 16,000, so a whole range first pays for two
 * chunks at about 16,000 x 150 = 2.4e6 instruction-equivalents and one chunk
 * is worth publishing at about half that. That measurement is a crossing and
 * not a plateau: it says where splitting starts to pay, not what a block of
 * kernels prefers once every chunk already pays for itself, which is a
 * per-block question the compute scoreboard is the instrument for. The guard
 * below is what lets that scoreboard build a runtime at another value without
 * editing this line, through the bundle's WF_RUNTIME_CONTROL_FLAGS.
 *
 * WF_SPLIT_WORK overrides it per process for diagnosis; the scoreboard's
 * harness unsets that variable so a recorded row can never be taken under one.
 * This selects how finely an admitted program is actualized in parallel; no
 * acceptance path reads it. */
#ifndef WF_PAR_SPLIT_WORK_UNIT
#define WF_PAR_SPLIT_WORK_UNIT 1200000ul
#endif
static unsigned long split_work = WF_PAR_SPLIT_WORK_UNIT;
static unsigned long report_wanted;
int wf__sched_report(char *buffer, size_t capacity) {
    if (!WF_SCHED_STATS || !report_wanted || !buffer || !capacity) return 0;
    int written = snprintf(buffer, capacity,
        "compute: threads=%d workers_started=%u steals=%lu slots_per_lane=%u",
        lanes ? lanes : 1, wf__sched_pool_running(), wf__par_grants(), WF_SCHED_LANE_SLOTS);
    return written > 0 && (size_t)written < capacity;
}
static void print_report(void) {
    char buffer[256];
    if (report_wanted == 2 && wf__sched_report(buffer, sizeof(buffer)))
        fprintf(stderr, "%s\n", buffer);
}
static void initialize(void) {
    unsigned long requested = wf_prim_online_cpus();
    unsigned long helpers;
    if (requested > WF_SCHED_MAX_THREADS) requested = WF_SCHED_MAX_THREADS;
    wf__sched_setting("WF_WORKERS", WF_SCHED_MAX_THREADS, &requested);
    lanes = requested >= 2 ? (int)requested : 0;
    wf__sched_setting("WF_SPLIT_WORK", 1000000000ul, &split_work);
    wf__sched_setting("WF_SCHED_REPORT", 2ul, &report_wanted);
    unsigned long ceiling = wf__sched_helper_ceiling();
    if (ceiling) wf__sched_setting("WF_IO_HELPERS", ceiling, &helpers);
    if (report_wanted == 2) atexit(print_report);
}
int wf__sched_lanes(void) {
    wf__sched_once(&initialized, initialize);
    return lanes;
}
uint64_t wf__sched_split_work(void) { return split_work; }
void wf__runtime_start(void) { (void)wf__sched_lanes(); }
