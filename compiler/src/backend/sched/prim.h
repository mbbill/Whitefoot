#ifndef WHITEFOOT_SCHED_PRIM_H
#define WHITEFOOT_SCHED_PRIM_H
/* Native thread, wait, yield and spin primitives. Deque atomics remain
 * inline in the shared core, with the same ordering on every target. */
#include <stddef.h>
#include <stdint.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef struct { SRWLOCK lock; CONDITION_VARIABLE signal; } wf_prim_wait;
#else
#include <pthread.h>
typedef struct { pthread_mutex_t lock; pthread_cond_t signal; } wf_prim_wait;
#endif
typedef struct { void (*entry)(void *); void *argument; } wf_prim_thread;
/* Keep the Windows historical spin hint distinct from an OS thread yield.
 * Other platforms retain their current polling behavior. */
static inline void wf_prim_spin_hint(void) {
#if defined(_WIN32)
    YieldProcessor();
#endif
}
int wf_prim_wait_init(wf_prim_wait *wait);
void wf_prim_wait_destroy(wf_prim_wait *wait);
void wf_prim_wait_lock(wf_prim_wait *wait);
void wf_prim_wait_unlock(wf_prim_wait *wait);
void wf_prim_wait_sleep(wf_prim_wait *wait);
void wf_prim_wait_signal(wf_prim_wait *wait);
int wf_prim_thread_start(wf_prim_thread *thread, void (*entry)(void *), void *argument, size_t stack_bytes);
void wf_prim_floor_attach(void);
void wf_prim_yield(void);
unsigned wf_prim_online_cpus(void);
/* Monotonic microseconds from an unspecified fixed origin, so only
 * differences between two readings mean anything. Zero is reserved: it is
 * what a host with no usable monotonic clock answers, and a caller that reads
 * zero must fall back to behaviour that needs no clock rather than treat the
 * difference as elapsed time. */
uint64_t wf_prim_monotonic_us(void);
#define WF_PRIM_SETTING_BYTES 64u
int wf_prim_setting_text(const char *name, char *buffer, size_t capacity);
#endif
