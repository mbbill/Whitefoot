#ifndef WHITEFOOT_COMPLETION_PLATFORM_H
#define WHITEFOOT_COMPLETION_PLATFORM_H

#include "contract.h"

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
typedef CRITICAL_SECTION wf_io_mutex;
typedef CONDITION_VARIABLE wf_io_condition;
#else
#include <pthread.h>
typedef pthread_mutex_t wf_io_mutex;
typedef pthread_cond_t wf_io_condition;
#endif

typedef void (*wf_io_thread_fn)(void *context);

int wf_io_platform_global_init(void);
int wf_io_platform_lane_init(unsigned lane);
void wf_io_platform_wake(unsigned lane);
void wf_io_platform_park(unsigned lane);
const char *wf_io_platform_name(void);
unsigned wf_io_platform_features(void);
uint64_t wf_io_platform_poll_arms(void);

int wf_io_platform_thread_start(wf_io_thread_fn function, void *context);
int wf_io_platform_mutex_init(wf_io_mutex *mutex);
int wf_io_platform_condition_init(wf_io_condition *condition);
void wf_io_platform_mutex_lock(wf_io_mutex *mutex);
void wf_io_platform_mutex_unlock(wf_io_mutex *mutex);
void wf_io_platform_condition_wait(wf_io_condition *condition, wf_io_mutex *mutex);
void wf_io_platform_condition_signal(wf_io_condition *condition);
void wf_io_platform_pause(void);

#endif
