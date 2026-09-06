/* Park on miss, design section 12, first measurement: the cost of one stack
 * switch, against the park-and-wake the tree measured (about 2.2 microseconds,
 * the comment above WF_PAR_SPIN_ROUNDS in compiler/src/backend/par_runtime.c).
 *
 * The switch below is the one the design names for POSIX: a hand-written save
 * of the callee-saved registers and the stack pointer, per architecture, with
 * no signal-mask syscall. Two stacks on one thread hand control back and forth;
 * each round trip is two switches. Beside it, for scale on the same host:
 * `swapcontext`, which carries the sigprocmask syscall the design refuses to
 * pay, and a condition-variable park-and-wake between two threads, which is the
 * figure the design is measured against. Numbers go to RESULTS.md beside this
 * file; nothing here is a gate. */

#define _XOPEN_SOURCE 700
#define _DARWIN_C_SOURCE 1

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/utsname.h>
#include <time.h>
#include <ucontext.h>
#include <unistd.h>

#define STACK_BYTES (256u * 1024u)
#define SWITCH_ROUNDS 20000000u
#define CONTEXT_ROUNDS 1000000u
#define WAKE_ROUNDS 100000u

typedef void (*wf_entry)(void *argument);

static uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

/* wf_switch(save, load): spill the callee-saved state onto the current stack,
 * store the resulting stack pointer through `save`, adopt `load` as the stack
 * pointer, restore the state that stack holds, and return into it. A stack
 * prepared by wf_prepare holds a frame whose return address is the trampoline,
 * so the first switch into it calls `entry(argument)`. */
#if defined(__aarch64__)

__attribute__((naked)) static void wf_switch(void **save, void *load) {
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

/* Entered by the first switch's `ret`: x19 carries the argument and x20 the
 * entry, both restored from the prepared frame. The entry never returns. */
__attribute__((naked)) static void wf_trampoline(void) {
    __asm__ volatile(
        "mov x0, x19\n"
        "blr x20\n"
        "brk #0\n");
}

static void *wf_prepare(void *top, wf_entry entry, void *argument) {
    uintptr_t sp = ((uintptr_t)top & ~(uintptr_t)15) - 176;
    void **frame = (void **)sp;
    memset(frame, 0, 176);
    frame[0] = argument;                            /* x19 */
    frame[1] = (void *)(uintptr_t)entry;            /* x20 */
    frame[11] = (void *)(uintptr_t)wf_trampoline;   /* x30 */
    return (void *)sp;
}

#elif defined(__x86_64__)

__attribute__((naked)) static void wf_switch(void **save, void *load) {
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

/* Entered by the first switch's `ret` with the stack pointer sixteen-aligned:
 * r12 carries the argument and r13 the entry. The call leaves the callee with
 * the alignment the ABI requires. The entry never returns. */
__attribute__((naked)) static void wf_trampoline(void) {
    __asm__ volatile(
        "movq %r12, %rdi\n"
        "callq *%r13\n"
        "ud2\n");
}

static void *wf_prepare(void *top, wf_entry entry, void *argument) {
    uintptr_t aligned = (uintptr_t)top & ~(uintptr_t)15;
    void **frame = (void **)(aligned - 56);
    frame[0] = NULL;                                /* r15 */
    frame[1] = NULL;                                /* r14 */
    frame[2] = (void *)(uintptr_t)entry;            /* r13 */
    frame[3] = argument;                            /* r12 */
    frame[4] = NULL;                                /* rbx */
    frame[5] = NULL;                                /* rbp */
    frame[6] = (void *)(uintptr_t)wf_trampoline;    /* return address */
    return (void *)frame;
}

#else
#error "no switch for this architecture"
#endif

/* One stack with a guard page below it, as the design gives every pool
 * stack. */
static void *stack_top(void) {
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    size_t bytes = STACK_BYTES + page;
    unsigned char *base = mmap(
        NULL,
        bytes,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0
    );
    if (base == MAP_FAILED) {
        perror("mmap");
        exit(1);
    }
    if (mprotect(base, page, PROT_NONE) != 0) {
        perror("mprotect");
        exit(1);
    }
    return base + bytes;
}

/* ---------------------------------------------------------------- switch */

static void *sp_main;
static void *sp_other;

static void pong(void *argument) {
    (void)argument;
    for (;;) {
        wf_switch(&sp_other, sp_main);
    }
}

static double measure_switch(void) {
    uint64_t started;
    uint64_t elapsed;
    unsigned round;
    sp_other = wf_prepare(stack_top(), pong, NULL);
    /* One warm round so the first entry through the trampoline is not timed. */
    wf_switch(&sp_main, sp_other);
    started = now_ns();
    for (round = 0; round < SWITCH_ROUNDS; ++round) {
        wf_switch(&sp_main, sp_other);
    }
    elapsed = now_ns() - started;
    return (double)elapsed / (2.0 * (double)SWITCH_ROUNDS);
}

/* ------------------------------------------------------------ swapcontext */

static ucontext_t context_main;
static ucontext_t context_other;

static void context_pong(void) {
    for (;;) {
        swapcontext(&context_other, &context_main);
    }
}

static double measure_swapcontext(void) {
    uint64_t started;
    uint64_t elapsed;
    unsigned round;
    unsigned char *top = stack_top();
    if (getcontext(&context_other) != 0) {
        perror("getcontext");
        exit(1);
    }
    context_other.uc_stack.ss_sp = top - STACK_BYTES;
    context_other.uc_stack.ss_size = STACK_BYTES;
    context_other.uc_link = NULL;
    makecontext(&context_other, context_pong, 0);
    swapcontext(&context_main, &context_other);
    started = now_ns();
    for (round = 0; round < CONTEXT_ROUNDS; ++round) {
        swapcontext(&context_main, &context_other);
    }
    elapsed = now_ns() - started;
    return (double)elapsed / (2.0 * (double)CONTEXT_ROUNDS);
}

/* ---------------------------------------------------------- park and wake */

static pthread_mutex_t wake_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wake_signal = PTHREAD_COND_INITIALIZER;
static unsigned wake_turn;   /* 0: the main thread's turn, 1: the other's */
static unsigned wake_rounds_left;

static void *wake_other(void *argument) {
    (void)argument;
    for (;;) {
        pthread_mutex_lock(&wake_lock);
        while (wake_turn == 0) {
            pthread_cond_wait(&wake_signal, &wake_lock);
        }
        if (wake_rounds_left == 0) {
            pthread_mutex_unlock(&wake_lock);
            return NULL;
        }
        wake_rounds_left -= 1;
        wake_turn = 0;
        pthread_cond_signal(&wake_signal);
        pthread_mutex_unlock(&wake_lock);
    }
}

/* One round trip is two park-and-wakes: the main thread wakes the other and
 * parks, the other wakes the main thread and parks. */
static double measure_park_and_wake(void) {
    pthread_t thread;
    uint64_t started;
    uint64_t elapsed;
    unsigned round;
    wake_rounds_left = WAKE_ROUNDS;
    wake_turn = 0;
    if (pthread_create(&thread, NULL, wake_other, NULL) != 0) {
        perror("pthread_create");
        exit(1);
    }
    started = now_ns();
    for (round = 0; round < WAKE_ROUNDS; ++round) {
        pthread_mutex_lock(&wake_lock);
        wake_turn = 1;
        pthread_cond_signal(&wake_signal);
        while (wake_turn == 1) {
            pthread_cond_wait(&wake_signal, &wake_lock);
        }
        pthread_mutex_unlock(&wake_lock);
    }
    elapsed = now_ns() - started;
    pthread_mutex_lock(&wake_lock);
    wake_turn = 1;
    pthread_cond_signal(&wake_signal);
    pthread_mutex_unlock(&wake_lock);
    pthread_join(thread, NULL);
    return (double)elapsed / (2.0 * (double)WAKE_ROUNDS);
}

int main(void) {
    struct utsname host;
    double switch_ns;
    double context_ns;
    double wake_ns;
    if (uname(&host) == 0) {
        printf("host: %s %s %s\n", host.sysname, host.release, host.machine);
    }
    switch_ns = measure_switch();
    context_ns = measure_swapcontext();
    wake_ns = measure_park_and_wake();
    printf("stack switch        %8.1f ns per switch   (%u round trips)\n",
           switch_ns, SWITCH_ROUNDS);
    printf("swapcontext         %8.1f ns per switch   (%u round trips)\n",
           context_ns, CONTEXT_ROUNDS);
    printf("condvar park+wake   %8.1f ns per park     (%u round trips)\n",
           wake_ns, WAKE_ROUNDS);
    printf("park+wake / switch  %8.1f x\n", wake_ns / switch_ns);
    return 0;
}
