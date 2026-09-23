/* Compile-only target geometry probes; reproduction is in STACK.md. */
typedef unsigned long long StackWord;

extern StackWord stack_external(const volatile unsigned char *, unsigned);

__attribute__((noinline)) StackWord stack_leaf(unsigned index) {
    volatile unsigned char bytes[64];
    bytes[index & 63] = 7;
    return bytes[index & 63];
}

__attribute__((noinline)) StackWord stack_large(unsigned index) {
    volatile unsigned char bytes[8192];
    bytes[index & 8191] = 9;
    return bytes[index & 8191];
}

__attribute__((noinline)) StackWord stack_parent(unsigned index) {
    volatile unsigned char bytes[64];
    bytes[index & 63] = 11;
    StackWord child = stack_leaf(index);
    return child + bytes[index & 63];
}

__attribute__((noinline)) StackWord stack_siblings(unsigned index) {
    StackWord first = stack_leaf(index);
    StackWord second = stack_large(index);
    return first + second;
}

__attribute__((noinline)) StackWord stack_tail(unsigned index) {
    return stack_leaf(index);
}

__attribute__((noinline)) StackWord stack_realign(unsigned index) {
    _Alignas(64) volatile unsigned char bytes[128];
    bytes[index & 127] = 13;
    return bytes[index & 127];
}

__attribute__((noinline)) StackWord stack_dynamic(unsigned index) {
    unsigned count = (index & 255) + 1;
    volatile unsigned char bytes[count];
    bytes[count - 1] = 17;
    return bytes[count - 1];
}

__attribute__((noinline)) StackWord stack_unknown(unsigned index) {
    volatile unsigned char bytes[64] = {0};
    return stack_external(bytes, index);
}

__attribute__((noinline)) StackWord stack_indirect(
    unsigned index, StackWord (*callee)(unsigned)) {
    return callee(index) + 1;
}

#ifdef STACK_NATIVE_OBSERVER
#include <stdio.h>
#include <string.h>

#if !defined(__aarch64__) || !defined(__APPLE__)
#error The native observer uses the arm64 Darwin ABI.
#endif

_Alignas(64) unsigned char stack_probe_buffer[4096];
extern StackWord stack_observe(StackWord offset, StackWord realign);

/* This body only closes the observer link; stack_unknown is never called. */
StackWord stack_external(const volatile unsigned char *bytes, unsigned index) {
    return bytes[index & 63];
}

/* Saved host state stays on the host stack; the probe gets the supplied SP. */
__asm__(
    ".text\n.p2align 2\n.globl _stack_observe\n_stack_observe:\n"
    "stp x29, x30, [sp, #-32]!\n"
    "stp x19, x20, [sp, #16]\n"
    "mov x19, sp\n"
    "adrp x9, _stack_probe_buffer@PAGE\n"
    "add x9, x9, _stack_probe_buffer@PAGEOFF\n"
    "add x9, x9, #4096\n"
    "sub x9, x9, x0\n"
    "mov sp, x9\n"
    "mov w0, #0\n"
    "cbnz w1, 1f\n"
    "bl _stack_leaf\n"
    "b 2f\n"
    "1: bl _stack_realign\n"
    "2: mov sp, x19\n"
    "ldp x19, x20, [sp, #16]\n"
    "ldp x29, x30, [sp], #32\n"
    "ret\n");

int main(void) {
    for (StackWord realign = 0; realign != 2; ++realign) {
        for (StackWord offset = 0; offset != 64; offset += 16) {
            memset(stack_probe_buffer, 0xa5, sizeof(stack_probe_buffer));
            StackWord result = stack_observe(offset, realign);
            size_t first = 0;
            while (first != sizeof(stack_probe_buffer)
                   && stack_probe_buffer[first] == 0xa5) {
                ++first;
            }
            size_t depth = sizeof(stack_probe_buffer) - (size_t)offset - first;
            size_t expected = realign ? 192 + ((64 - (size_t)offset) & 63) : 64;
            printf("%s offset=%llu lowest_write=%zu expected=%zu\n",
                   realign ? "realign" : "leaf", offset, depth, expected);
            if (depth != expected || result != (realign ? 13 : 7)) {
                return 1;
            }
        }
    }
    return 0;
}
#endif
