#ifndef WF_COMPUTE_EVENTS_H
#define WF_COMPUTE_EVENTS_H
/* Observer schema wf-1. Retire with the compute scheduler diagnostic.
 * Only published/popped/stolen/completed jobs are stable after their joins.
 * Wait and signal events count API calls, not OS context switches or wakes. */
enum {
  WF_EVENT_PUBLISH,
  WF_EVENT_LOCAL_POP,
  WF_EVENT_STEAL_ATTEMPT,
  WF_EVENT_STEAL_EMPTY,
  WF_EVENT_STEAL_CAS_FAIL,
  WF_EVENT_STEAL_SUCCESS,
  WF_EVENT_RUN_BEGIN,
  WF_EVENT_RUN_END,
  WF_EVENT_INLINE_RUN,
  WF_EVENT_JOIN,
  WF_EVENT_JOIN_WAIT,
  WF_EVENT_JOIN_YIELD,
  WF_EVENT_IDLE_YIELD,
  WF_EVENT_JOIN_PARK,
  WF_EVENT_JOIN_RESUME,
  WF_EVENT_IDLE_PARK,
  WF_EVENT_IDLE_RESUME,
  WF_EVENT_SIGNAL,
  WF_EVENT_IDLE_CLAIM,
  WF_EVENT_SLOT_REFUSAL,
  WF_EVENT_COUNT
};
#ifdef __cplusplus
extern "C" {
#endif
unsigned long wf_compute_event(unsigned lane, unsigned event);
#ifdef __cplusplus
}
#endif
#endif
