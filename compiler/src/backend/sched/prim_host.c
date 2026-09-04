/* The host's primitives for the scheduler core (`prim.h`, items 2 to 7 and
 * thread identity; item 1 is inline in the header).
 *
 * POSIX only. The switch is the hand-written one
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

void wf_prim_fail(const char *reason) {
    (void)fprintf(stderr, "whitefoot scheduler: %s\n", reason);
    abort();
}

/* ---------------------------------------------------------- thread index */

static _Thread_local unsigned wf_prim_thread;

unsigned wf_prim_thread_index(void) {
    return wf_prim_thread;
}

void wf_prim_set_thread_index(unsigned index) {
    wf_prim_thread = index;
}

/* ------------------------------------------------------------ 2. switch */

#if defined(__aarch64__)

__attribute__((naked)) void wf_prim_switch(void **save, void *load) {
    __asm__ volatile(
        "sub sp, sp, #176\n"
        "stp x19, x20, [sp, #0]\n"
        "stp x21, x22, [sp, #16]\n"
        "stp x23, x24, [sp, #32]\n"
        "stp x25, x26, [sp, #48]\n"
        "stp x27, x28, [sp, #64]\n"
        "stp x29, x30, [sp, #80]\n"
        "stp d8, d9, [sp, #96]\n"
        "stp d10, d11, [sp, #112]\n"
        "stp d12, d13, [sp, #128]\n"
        "stp d14, d15, [sp, #144]\n"
        "mov x9, sp\n"
        "str x9, [x0]\n"
        "mov sp, x1\n"
        "ldp x19, x20, [sp, #0]\n"
        "ldp x21, x22, [sp, #16]\n"
        "ldp x23, x24, [sp, #32]\n"
        "ldp x25, x26, [sp, #48]\n"
        "ldp x27, x28, [sp, #64]\n"
        "ldp x29, x30, [sp, #80]\n"
        "ldp d8, d9, [sp, #96]\n"
        "ldp d10, d11, [sp, #112]\n"
        "ldp d12, d13, [sp, #128]\n"
        "ldp d14, d15, [sp, #144]\n"
        "add sp, sp, #176\n"
        "ret\n");
}

__attribute__((naked)) static void wf_prim_trampoline(void) {
    __asm__ volatile(
        "mov x0, x19\n"
        "blr x20\n"
        "brk #0\n");
}

void *wf_prim_prepare_stack(void *top, void (*entry)(void *), void *argument) {
    uintptr_t sp = ((uintptr_t)top & ~(uintptr_t)15) - 176;
    void **frame = (void **)sp;
    memset(frame, 0, 176);
    frame[0] = argument;                                /* x19 */
    frame[1] = (void *)(uintptr_t)entry;                /* x20 */
    frame[11] = (void *)(uintptr_t)wf_prim_trampoline;  /* x30 */
    return (void *)sp;
}

#elif defined(__x86_64__)

__attribute__((naked)) void wf_prim_switch(void **save, void *load) {
    __asm__ volatile(
        "pushq %rbp\n"
        "pushq %rbx\n"
        "pushq %r12\n"
        "pushq %r13\n"
        "pushq %r14\n"
        "pushq %r15\n"
        "movq %rsp, (%rdi)\n"
        "movq %rsi, %rsp\n"
        "popq %r15\n"
        "popq %r14\n"
        "popq %r13\n"
        "popq %r12\n"
        "popq %rbx\n"
        "popq %rbp\n"
        "ret\n");
}

__attribute__((naked)) static void wf_prim_trampoline(void) {
    __asm__ volatile(
        "movq %r12, %rdi\n"
        "callq *%r13\n"
        "ud2\n");
}

void *wf_prim_prepare_stack(void *top, void (*entry)(void *), void *argument) {
    uintptr_t aligned = (uintptr_t)top & ~(uintptr_t)15;
    void **frame = (void **)(aligned - 56);
    frame[0] = NULL;                                    /* r15 */
    frame[1] = NULL;                                    /* r14 */
    frame[2] = (void *)(uintptr_t)entry;                /* r13 */
    frame[3] = argument;                                /* r12 */
    frame[4] = NULL;                                    /* rbx */
    frame[5] = NULL;                                    /* rbp */
    frame[6] = (void *)(uintptr_t)wf_prim_trampoline;   /* return address */
    return (void *)frame;
}

#else
#error "the scheduler core has no switch for this architecture"
#endif

/* The floor's per-stack half. The floor exports the setter once the core is
 * linked into programs (design §5); until then the weak answer does nothing,
 * so a core linked without the floor still switches. */
__attribute__((weak)) void wf__floor_set_stack_bounds(unsigned char *low, unsigned char *high) {
    (void)low;
    (void)high;
}

void wf_prim_set_bounds(unsigned char *low, unsigned char *high) {
    wf__floor_set_stack_bounds(low, high);
}

/* -------------------------------------------------------------- 3. park */

static pthread_mutex_t wf_prim_wake_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf_prim_wake_signal = PTHREAD_COND_INITIALIZER;
static uint64_t wf_prim_wake_epoch;
static unsigned wf_prim_sleepers;

uint64_t wf_prim_epoch(void) {
    return __atomic_load_n(&wf_prim_wake_epoch, __ATOMIC_SEQ_CST);
}

void wf_prim_park(uint64_t observed) {
    pthread_mutex_lock(&wf_prim_wake_lock);
    wf_prim_sleepers += 1u;
    while (__atomic_load_n(&wf_prim_wake_epoch, __ATOMIC_SEQ_CST) == observed) {
        pthread_cond_wait(&wf_prim_wake_signal, &wf_prim_wake_lock);
    }
    wf_prim_sleepers -= 1u;
    pthread_mutex_unlock(&wf_prim_wake_lock);
}

void wf_prim_wake(void) {
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

unsigned char *wf_prim_reserve(unsigned count, size_t bytes) {
    size_t page = wf_prim_page();
    size_t stride = wf_prim_stack_stride(bytes);
    size_t total = stride * (size_t)count;
    unsigned char *base;
    unsigned index;
    base = mmap(NULL, total, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
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

void wf_prim_lock(void) {
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
