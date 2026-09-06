/* The stack switch, written once for the two units that switch stacks: the
 * host primitives (`prim_host.c`, primitive 2 of PARK-ON-MISS.md §7.1) and
 * the enumerator (`enumerate.c`), whose controller and simulated threads are
 * coroutines on host stacks and whose replacement primitive 2 performs this
 * same switch once the controlled scheduler has chosen to let it happen.
 *
 * `wf_switch_raw` spills the callee-saved state of the calling stack, stores
 * its stack pointer through `save`, adopts `load`, and returns into it. A
 * stack prepared by `wf_switch_prepare` runs `entry(argument)` at its first
 * switch and must never return from it. This is the switch
 * `research/experiments/park-on-miss-switch-cost/` measured. */

#ifndef WHITEFOOT_SCHED_SWITCH_H
#define WHITEFOOT_SCHED_SWITCH_H

#include <stdint.h>
#include <string.h>

#if defined(__aarch64__)

__attribute__((naked)) static void wf_switch_raw(void **save __attribute__((unused)), void *load __attribute__((unused))) {
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

__attribute__((naked)) static void wf_switch_trampoline(void) {
    __asm__ volatile(
        "mov x0, x19\n"
        "blr x20\n"
        "brk #0\n");
}

static void *wf_switch_prepare(void *top, void (*entry)(void *), void *argument) {
    uintptr_t sp = ((uintptr_t)top & ~(uintptr_t)15) - 176;
    void **frame = (void **)sp;
    memset(frame, 0, 176);
    frame[0] = argument;                                /* x19 */
    frame[1] = (void *)(uintptr_t)entry;                /* x20 */
    frame[11] = (void *)(uintptr_t)wf_switch_trampoline; /* x30 */
    return (void *)sp;
}

#elif defined(__x86_64__)

__attribute__((naked)) static void wf_switch_raw(void **save __attribute__((unused)), void *load __attribute__((unused))) {
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

__attribute__((naked)) static void wf_switch_trampoline(void) {
    __asm__ volatile(
        "movq %r12, %rdi\n"
        "callq *%r13\n"
        "ud2\n");
}

static void *wf_switch_prepare(void *top, void (*entry)(void *), void *argument) {
    uintptr_t aligned = (uintptr_t)top & ~(uintptr_t)15;
    void **frame = (void **)(aligned - 56);
    frame[0] = NULL;                                     /* r15 */
    frame[1] = NULL;                                     /* r14 */
    frame[2] = (void *)(uintptr_t)entry;                 /* r13 */
    frame[3] = argument;                                 /* r12 */
    frame[4] = NULL;                                     /* rbx */
    frame[5] = NULL;                                     /* rbp */
    frame[6] = (void *)(uintptr_t)wf_switch_trampoline;  /* return address */
    return (void *)frame;
}

#else
#error "the scheduler core has no switch for this architecture"
#endif

#endif
