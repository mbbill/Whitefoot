#ifndef WHITEFOOT_COMPUTE_CONTROL_H
#define WHITEFOOT_COMPUTE_CONTROL_H

/* Diagnostic statistics are optional and do not participate in scheduling.
 * Existing observers retain them by default; pure timing builds opt out. */
#ifndef WF_COMPUTE_STATS
#define WF_COMPUTE_STATS 1
#endif
#if WF_COMPUTE_STATS != 0 && WF_COMPUTE_STATS != 1
#error "WF_COMPUTE_STATS must be zero or one"
#endif
/* Current POSIX LP64 emitted-module protocol. The runtime owns opaque frame
 * storage until release; join returns before the caller reads the result. */
void *wf__par_acquire_lane(unsigned long bytes);
void wf__par_publish(void *frame, void (*run)(void *));
void wf__par_join(void *frame);
void wf__par_release(void *frame);
int wf__par_pool_active(void);
unsigned long wf__par_split_budget(unsigned long span, unsigned long weight);
#if WF_COMPUTE_STATS
unsigned long wf__par_grants(void);
#endif

/* Read-only control qualification; not part of the emitted-module ABI. */
unsigned wf_compute_worker_count(void);
unsigned wf_compute_slot_capacity(void);

#if defined(WF_COMPUTE_TEST)
void wf_compute_test_before_steal_read(unsigned victim, unsigned long long top);
void wf_compute_test_after_steal_read(void *observed);
void wf_compute_test_after_steal(int claimed);
void wf_compute_test_after_done(void *frame);
void wf_compute_test_after_completion_tail(void);
int wf_compute_test_allow_worker(unsigned index);
#endif

#endif
