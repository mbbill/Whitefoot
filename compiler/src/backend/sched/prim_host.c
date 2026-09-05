/* The host's primitives for the scheduler core (`prim.h`, items 2 to 7,
 * thread identity, and the platform layer's own three; item 1 is inline in
 * the header).
 *
 * POSIX only, and `prim_windows.c` is its twin. The switch is the hand-written
 * one
 * `research/experiments/park-on-miss-switch-cost/` measured; the park is an
 * epoch on one mutex and condition variable, which is the shape the completion
 * runtime's own park has and will be the one primitive the bridge supplies
 * once the core replaces the writer schedulers (design §7, platform item 2);
 * target progress is a weak hook the bridge defines, so a core linked without
 * a completion runtime makes no progress and drains nothing. */

#define _DARWIN_C_SOURCE 1

#include "prim.h"
#include "core.h"

#include <pthread.h>
#include <sched.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <sys/sysctl.h>
#endif

void wf_prim_fail(const char *reason) {
    (void)fprintf(stderr, "whitefoot scheduler: %s\n", reason);
    abort();
}

/* ---------------------------------------------------------- thread index */

static _Thread_local unsigned wf_prim_thread_ordinal;

unsigned wf_prim_thread_index(void) {
    return wf_prim_thread_ordinal;
}

void wf_prim_set_thread_index(unsigned index) {
    wf_prim_thread_ordinal = index;
}

/* ------------------------------------------------------------ 2. switch */

#include "switch.h"

void wf_prim_switch(void **save, void *load) {
    wf_switch_raw(save, load);
}

void *wf_prim_prepare_stack(
    void *top,
    size_t bytes,
    void (*entry)(void *),
    void *argument
) {
    /* The slot *is* the stack here, so the frame goes at its top and the size
     * is the reservation's business rather than the switch's. Windows needs
     * the size because a fiber owns a stack of its own (`prim_windows.c`). */
    (void)bytes;
    return wf_switch_prepare(top, entry, argument);
}

/* The floor's two halves. The floor exports both once the core is linked into
 * programs (design §5); until then the weak answers do nothing, so a core
 * linked without the floor still switches and `entry.c` still starts its
 * threads. This leaf is the one unit of a link that names the floor's attach,
 * and `wf_prim_floor_attach` is how `entry.c` reaches it: the Windows leaf
 * has to arm every pool fiber itself, one link admits one weak default per
 * symbol (MSVC, LNK1227), and a PE weak default satisfies only references from
 * its own unit (GNU ld), so each platform leaf carries the pair and every
 * caller above goes through it. */
__attribute__((weak)) void wf__floor_attach_thread(void) {}

__attribute__((weak)) void wf__floor_set_stack_bounds(unsigned char *low, unsigned char *high) {
    (void)low;
    (void)high;
}

void wf_prim_floor_attach(void) {
    wf__floor_attach_thread();
}

void wf_prim_set_bounds(unsigned char *low, unsigned char *high) {
    wf__floor_set_stack_bounds(low, high);
}

/* -------------------------------------------------------------- 3. park */

static pthread_mutex_t wf_prim_wake_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf_prim_wake_signal = PTHREAD_COND_INITIALIZER;
static uint64_t wf_prim_wake_epoch;
static unsigned wf_prim_sleepers;

/* Platform item 2 of design §7: one wait and wake primitive.
 *
 * The core sleeps and wakes on whatever the link's owner of the host wait set
 * supplies.  A completion link has one -- the bridge, whose park is the
 * io_uring epoll wait where a ring exists and the runtime's condition-variable
 * park elsewhere, and whose wake raises the same epoch the ring's own park
 * announces itself against -- so the bridge defines these three strongly and
 * the mapping is written down at that definition.  A core linked without a
 * completion runtime, such as the smoke build and the enumerator, gets the
 * weak answers below and keeps the epoch on this file's own mutex and
 * condition variable.  Each returns nonzero when it handled the call. */
__attribute__((weak)) int wf__sched_host_epoch(uint64_t *epoch) {
    (void)epoch;
    return 0;
}

__attribute__((weak)) int wf__sched_host_park(uint64_t observed) {
    (void)observed;
    return 0;
}

__attribute__((weak)) int wf__sched_host_wake(void) {
    return 0;
}

uint64_t wf_prim_epoch(void) {
    uint64_t supplied = 0;
    if (wf__sched_host_epoch(&supplied)) {
        return supplied;
    }
    return __atomic_load_n(&wf_prim_wake_epoch, __ATOMIC_SEQ_CST);
}

void wf_prim_park(uint64_t observed) {
    if (wf__sched_host_park(observed)) {
        return;
    }
    pthread_mutex_lock(&wf_prim_wake_lock);
    wf_prim_sleepers += 1u;
    while (__atomic_load_n(&wf_prim_wake_epoch, __ATOMIC_SEQ_CST) == observed) {
        pthread_cond_wait(&wf_prim_wake_signal, &wf_prim_wake_lock);
    }
    wf_prim_sleepers -= 1u;
    pthread_mutex_unlock(&wf_prim_wake_lock);
}

void wf_prim_wake(void) {
    if (wf__sched_host_wake()) {
        return;
    }
    pthread_mutex_lock(&wf_prim_wake_lock);
    __atomic_add_fetch(&wf_prim_wake_epoch, 1u, __ATOMIC_SEQ_CST);
    if (wf_prim_sleepers != 0u) {
        pthread_cond_broadcast(&wf_prim_wake_signal);
    }
    pthread_mutex_unlock(&wf_prim_wake_lock);
}

/* ------------------------------------------------------- 4. reservation */

static size_t wf_prim_page(void) {
    long page = sysconf(_SC_PAGESIZE);
    return page > 0 ? (size_t)page : 4096u;
}

size_t wf_prim_stack_stride(size_t bytes) {
    size_t page = wf_prim_page();
    size_t rounded = (bytes + page - 1u) / page * page;
    return page + rounded;
}

/* One mapping, carved into `count` slots of `bytes` each with a guard page
 * below every one of them.
 *
 * `MAP_NORESERVE` is what makes it a reservation rather than a commitment, and
 * it is not optional at the sizes the runtime asks for: a pool stack is the
 * floor's own 1 GiB, so a machine's worth of threads plus the spare stacks is
 * tens of gigabytes of address space, and Linux's heuristic overcommit refuses
 * a single mapping larger than memory plus swap unless it is told the mapping
 * is a reservation. The pages are committed on touch either way, so a run that
 * never descends pays for none of them. A host without the flag (it is a Linux
 * spelling) keeps today's behaviour exactly. */
unsigned char *wf_prim_reserve(unsigned count, size_t bytes) {
    size_t page = wf_prim_page();
    size_t stride = wf_prim_stack_stride(bytes);
    size_t total = stride * (size_t)count;
    unsigned char *base;
    unsigned index;
#if defined(MAP_NORESERVE)
    int flags = MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE;
#else
    int flags = MAP_PRIVATE | MAP_ANONYMOUS;
#endif
    base = mmap(NULL, total, PROT_READ | PROT_WRITE, flags, -1, 0);
    if (base == MAP_FAILED) {
        return NULL;
    }
    for (index = 0; index < count; index += 1u) {
        if (mprotect(base + (size_t)index * stride, page, PROT_NONE) != 0) {
            (void)munmap(base, total);
            return NULL;
        }
    }
    return base;
}

/* -------------------------------------------------------------- 5. lock */

static pthread_mutex_t wf_prim_core_lock = PTHREAD_MUTEX_INITIALIZER;

void wf_prim_lock(enum wf_prim_section section) {
    (void)section;
    pthread_mutex_lock(&wf_prim_core_lock);
}

void wf_prim_unlock(void) {
    pthread_mutex_unlock(&wf_prim_core_lock);
}

/* ------------------------------------------------------------- 6. yield */

void wf_prim_yield(void) {
    (void)sched_yield();
}

/* ---------------------------------------------------------- 7. progress */

__attribute__((weak)) int wf__sched_target_progress(struct wf_sched_core *core) {
    (void)core;
    return 0;
}

int wf_prim_progress(struct wf_sched_core *core) {
    return wf__sched_target_progress(core);
}

/* ------------------------------------- the platform layer's own three */

/* P1. One detached thread on a host stack this call reserves.
 *
 * A refused reservation is reported rather than silently downgraded to the
 * pthread default: no lane at all is better than a lane whose stack is a
 * fraction of the one the program's depth was sized against.
 *
 * The entry pair is the caller's record and nothing is allocated here; see
 * `prim.h` for why the runtime cannot take that pair from anywhere else. */
static void *wf_prim_thread_main(void *opaque) {
    wf_prim_thread *thread = opaque;
    thread->entry(thread->argument);
    return NULL;
}

int wf_prim_thread_start(
    wf_prim_thread *thread,
    void (*entry)(void *),
    void *argument,
    size_t stack_bytes
) {
    pthread_attr_t attributes;
    pthread_t created;
    int error;
    if (thread == NULL || entry == NULL) {
        return 1;
    }
    thread->entry = entry;
    thread->argument = argument;
    if (pthread_attr_init(&attributes) != 0) {
        return 1;
    }
    if (stack_bytes != 0
        && pthread_attr_setstacksize(&attributes, stack_bytes) != 0) {
        (void)pthread_attr_destroy(&attributes);
        return 1;
    }
    (void)pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    error = pthread_create(&created, &attributes, wf_prim_thread_main, thread);
    (void)pthread_attr_destroy(&attributes);
    return error != 0 ? 1 : 0;
}

/* P2. The host's switch needs no conversion of the thread's own stack, so
 * both halves are nothing here. Windows converts to a fiber and back. */
void wf_prim_thread_attach(void) {}

void wf_prim_thread_detach(void) {}

/* P3. The CPUs this process may be scheduled on right now.
 *
 * `hw.logicalcpu` is the Darwin spelling and reports exactly that; the
 * portable `_SC_NPROCESSORS_ONLN` answers on a host that has no such name.
 * Zero means the platform would not say, and the caller's own policy decides
 * what to do with that. */
unsigned wf_prim_online_cpus(void) {
    long online;
#if defined(__APPLE__)
    int logical = 0;
    size_t width = sizeof(logical);
    if (sysctlbyname("hw.logicalcpu", &logical, &width, NULL, 0) == 0
        && logical > 0) {
        return (unsigned)logical;
    }
#endif
    online = sysconf(_SC_NPROCESSORS_ONLN);
    return online > 0 ? (unsigned)online : 0u;
}

int wf_prim_setting_text(const char *name, char *buffer, size_t capacity) {
    const char *text;
    size_t length;
    if (name == NULL || buffer == NULL || capacity == 0) {
        return -1;
    }
    buffer[0] = '\0';
    text = getenv(name);
    if (text == NULL) {
        return 0;
    }
    length = strlen(text);
    if (length >= capacity) {
        return -1;
    }
    memcpy(buffer, text, length + 1);
    return 1;
}
