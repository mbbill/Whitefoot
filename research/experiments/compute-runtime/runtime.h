#ifndef WHITEFOOT_COMPUTE_CONTROL_H
#define WHITEFOOT_COMPUTE_CONTROL_H

/* Current POSIX LP64 emitted-module protocol. The runtime owns opaque frame
 * storage until release; join returns before the caller reads the result. */
void *wf__par_acquire_lane(unsigned long bytes);
void wf__par_publish(void *frame, void (*run)(void *));
void wf__par_join(void *frame);
void wf__par_release(void *frame);
int wf__par_pool_active(void);
unsigned long wf__par_split_budget(unsigned long span, unsigned long weight);
unsigned long wf__par_grants(void);

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
