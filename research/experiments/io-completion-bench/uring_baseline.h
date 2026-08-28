/* Raw io_uring read pipelines for the Linux N line.
 *
 * Written against the kernel ABI directly rather than liburing so the
 * baseline builds inside a bare container. It keeps DEPTH IORING_OP_READ
 * operations in flight over per-slot buffers and does openat/close on the
 * submitting thread, which is exactly the split the Whitefoot Linux adapter
 * uses: its ring carries reads and writes only.
 *
 * Two pipelines share the ring plumbing below. wf_bench_run_uring is the
 * many-files one: it opens a file per unit of work on the submitting thread.
 * wf_bench_run_read_uring is the read-heavy one: the descriptors are opened
 * once by the caller and every operation in flight is a positioned read, so
 * nothing but reads reaches the ring. */
#ifndef WF_BENCH_URING_BASELINE_H
#define WF_BENCH_URING_BASELINE_H

#include <errno.h>
#include <fcntl.h>
#include <linux/io_uring.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <unistd.h>

#include "workload.h"

struct wf_bench_ring {
    int descriptor;
    void *submission_map;
    size_t submission_bytes;
    struct io_uring_sqe *entries;
    size_t entry_bytes;
    unsigned *submission_head;
    unsigned *submission_tail;
    unsigned *submission_mask;
    unsigned *submission_array;
    unsigned *completion_head;
    unsigned *completion_tail;
    unsigned *completion_mask;
    struct io_uring_cqe *completions;
};

static inline int wf_bench_ring_setup(struct wf_bench_ring *ring, unsigned entries) {
    struct io_uring_params parameters;
    memset(&parameters, 0, sizeof parameters);
    long created = syscall(__NR_io_uring_setup, entries, &parameters);
    if (created < 0) {
        return 1;
    }
    ring->descriptor = (int)created;
    if ((parameters.features & IORING_FEAT_SINGLE_MMAP) == 0) {
        close(ring->descriptor);
        return 1;
    }
    size_t submission_bytes = parameters.sq_off.array + parameters.sq_entries * sizeof(unsigned);
    size_t completion_bytes =
        parameters.cq_off.cqes + parameters.cq_entries * sizeof(struct io_uring_cqe);
    if (completion_bytes > submission_bytes) {
        submission_bytes = completion_bytes;
    }
    void *map = mmap(NULL, submission_bytes, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE,
                     ring->descriptor, IORING_OFF_SQ_RING);
    if (map == MAP_FAILED) {
        close(ring->descriptor);
        return 1;
    }
    ring->submission_map = map;
    ring->submission_bytes = submission_bytes;
    ring->entry_bytes = parameters.sq_entries * sizeof(struct io_uring_sqe);
    ring->entries = mmap(NULL, ring->entry_bytes, PROT_READ | PROT_WRITE,
                         MAP_SHARED | MAP_POPULATE, ring->descriptor, IORING_OFF_SQES);
    if (ring->entries == MAP_FAILED) {
        munmap(map, submission_bytes);
        close(ring->descriptor);
        return 1;
    }
    unsigned char *base = map;
    ring->submission_head = (unsigned *)(base + parameters.sq_off.head);
    ring->submission_tail = (unsigned *)(base + parameters.sq_off.tail);
    ring->submission_mask = (unsigned *)(base + parameters.sq_off.ring_mask);
    ring->submission_array = (unsigned *)(base + parameters.sq_off.array);
    ring->completion_head = (unsigned *)(base + parameters.cq_off.head);
    ring->completion_tail = (unsigned *)(base + parameters.cq_off.tail);
    ring->completion_mask = (unsigned *)(base + parameters.cq_off.ring_mask);
    ring->completions = (struct io_uring_cqe *)(base + parameters.cq_off.cqes);
    return 0;
}

static inline void wf_bench_ring_teardown(struct wf_bench_ring *ring) {
    munmap(ring->entries, ring->entry_bytes);
    munmap(ring->submission_map, ring->submission_bytes);
    close(ring->descriptor);
}

struct wf_bench_slot {
    unsigned char *window;
    int descriptor;
    uint64_t index;
};

static inline int wf_bench_submit_read(struct wf_bench_ring *ring, unsigned handle,
                                      int descriptor, void *buffer, uint64_t offset,
                                      unsigned length) {
    unsigned tail = atomic_load_explicit((_Atomic unsigned *)ring->submission_tail,
                                         memory_order_relaxed);
    unsigned head =
        atomic_load_explicit((_Atomic unsigned *)ring->submission_head, memory_order_acquire);
    unsigned mask = *ring->submission_mask;
    if ((tail - head) > mask) {
        return 1;
    }
    unsigned position = tail & mask;
    struct io_uring_sqe *entry = &ring->entries[position];
    memset(entry, 0, sizeof *entry);
    entry->opcode = IORING_OP_READ;
    entry->fd = descriptor;
    entry->off = offset;
    entry->addr = (unsigned long long)(uintptr_t)buffer;
    entry->len = length;
    entry->user_data = handle;
    ring->submission_array[position] = position;
    atomic_store_explicit((_Atomic unsigned *)ring->submission_tail, tail + 1,
                          memory_order_release);
    return 0;
}

static inline int wf_bench_run_uring(int directory, uint64_t count, unsigned long depth, uint64_t *sum,
                              uint64_t *bytes) {
    unsigned entries = 8;
    while (entries < depth * 2) {
        entries *= 2;
    }
    struct wf_bench_ring ring;
    if (wf_bench_ring_setup(&ring, entries) != 0) {
        fprintf(stderr, "baseline: io_uring_setup failed: %s\n", strerror(errno));
        return 1;
    }
    struct wf_bench_slot *slots = calloc(depth, sizeof *slots);
    if (slots == NULL) {
        wf_bench_ring_teardown(&ring);
        return 1;
    }
    unsigned char *windows = malloc((size_t)depth * WF_BENCH_READ_WINDOW);
    if (windows == NULL) {
        free(slots);
        wf_bench_ring_teardown(&ring);
        return 1;
    }
    for (unsigned long at = 0; at < depth; at++) {
        slots[at].window = windows + (size_t)at * WF_BENCH_READ_WINDOW;
        slots[at].descriptor = -1;
    }
    unsigned long *free_slots = malloc(depth * sizeof *free_slots);
    if (free_slots == NULL) {
        free(windows);
        free(slots);
        wf_bench_ring_teardown(&ring);
        return 1;
    }
    unsigned long free_count = depth;
    for (unsigned long at = 0; at < depth; at++) {
        free_slots[at] = depth - 1 - at;
    }
    uint64_t next = 0;
    unsigned long in_flight = 0;
    char name[32];
    while (next < count || in_flight > 0) {
        unsigned pending = 0;
        while (next < count && free_count > 0) {
            unsigned long handle = free_slots[--free_count];
            snprintf(name, sizeof name, WF_BENCH_NAME_FORMAT, (unsigned long)next);
            int descriptor = openat(directory, name, O_RDONLY);
            if (descriptor < 0) {
                free_slots[free_count++] = handle;
                next++;
                continue;
            }
            slots[handle].descriptor = descriptor;
            slots[handle].index = next;
            if (wf_bench_submit_read(&ring, (unsigned)handle, descriptor, slots[handle].window,
                                     0, WF_BENCH_READ_WINDOW)
                != 0) {
                close(descriptor);
                slots[handle].descriptor = -1;
                free_slots[free_count++] = handle;
                break;
            }
            next++;
            in_flight++;
            pending++;
        }
        unsigned wait_for = in_flight > 0 ? 1 : 0;
        if (pending > 0 || wait_for > 0) {
            long entered = syscall(__NR_io_uring_enter, ring.descriptor, pending, wait_for,
                                   wait_for > 0 ? IORING_ENTER_GETEVENTS : 0, NULL, 0);
            if (entered < 0 && errno != EINTR) {
                fprintf(stderr, "baseline: io_uring_enter failed: %s\n", strerror(errno));
                free(free_slots);
                free(windows);
                free(slots);
                wf_bench_ring_teardown(&ring);
                return 1;
            }
        }
        unsigned head =
            atomic_load_explicit((_Atomic unsigned *)ring.completion_head, memory_order_relaxed);
        unsigned tail =
            atomic_load_explicit((_Atomic unsigned *)ring.completion_tail, memory_order_acquire);
        while (head != tail) {
            struct io_uring_cqe *completion = &ring.completions[head & *ring.completion_mask];
            unsigned long handle = (unsigned long)completion->user_data;
            int result = completion->res;
            struct wf_bench_slot *slot = &slots[handle];
            if (result > 0) {
                *bytes += (uint64_t)result;
                *sum += wf_bench_weighted(wf_bench_digest(slot->window, (size_t)result),
                                          slot->index);
            }
            close(slot->descriptor);
            slot->descriptor = -1;
            free_slots[free_count++] = handle;
            in_flight--;
            head++;
        }
        atomic_store_explicit((_Atomic unsigned *)ring.completion_head, head,
                              memory_order_release);
    }
    free(free_slots);
    free(windows);
    free(slots);
    wf_bench_ring_teardown(&ring);
    return 0;
}


/* The read-heavy pipeline: DEPTH positioned reads in flight over descriptors
 * the caller opened once, so nothing but reads ever reaches the ring. Each
 * slot carries the read position it was submitted for, which is both the
 * checksum's weight and the schedule's key. */
static inline int wf_bench_run_read_uring(const int *descriptors, uint64_t reads, uint64_t window,
                                          uint64_t blocks, unsigned long depth, uint64_t *sum,
                                          uint64_t *bytes) {
    unsigned entries = 8;
    while (entries < depth * 2) {
        entries *= 2;
    }
    struct wf_bench_ring ring;
    if (wf_bench_ring_setup(&ring, entries) != 0) {
        fprintf(stderr, "read_baseline: io_uring_setup failed: %s\n", strerror(errno));
        return 1;
    }
    struct wf_bench_slot *slots = calloc(depth, sizeof *slots);
    unsigned char *windows = malloc((size_t)depth * (size_t)window);
    unsigned long *free_slots = malloc(depth * sizeof *free_slots);
    if (slots == NULL || windows == NULL || free_slots == NULL) {
        free(free_slots);
        free(windows);
        free(slots);
        wf_bench_ring_teardown(&ring);
        return 1;
    }
    for (unsigned long at = 0; at < depth; at++) {
        slots[at].window = windows + (size_t)at * (size_t)window;
        slots[at].descriptor = -1;
        free_slots[at] = depth - 1 - at;
    }
    unsigned long free_count = depth;
    uint64_t next = 0;
    unsigned long in_flight = 0;
    while (next < reads || in_flight > 0) {
        unsigned pending = 0;
        while (next < reads && free_count > 0) {
            unsigned long handle = free_slots[--free_count];
            uint64_t offset = wf_bench_read_block(next, blocks) * window;
            slots[handle].descriptor = descriptors[wf_bench_read_file(next)];
            slots[handle].index = next;
            if (wf_bench_submit_read(&ring, (unsigned)handle, slots[handle].descriptor,
                                     slots[handle].window, offset, (unsigned)window)
                != 0) {
                free_slots[free_count++] = handle;
                break;
            }
            next++;
            in_flight++;
            pending++;
        }
        unsigned wait_for = in_flight > 0 ? 1 : 0;
        if (pending > 0 || wait_for > 0) {
            long entered = syscall(__NR_io_uring_enter, ring.descriptor, pending, wait_for,
                                   wait_for > 0 ? IORING_ENTER_GETEVENTS : 0, NULL, 0);
            if (entered < 0 && errno != EINTR) {
                fprintf(stderr, "read_baseline: io_uring_enter failed: %s\n", strerror(errno));
                free(free_slots);
                free(windows);
                free(slots);
                wf_bench_ring_teardown(&ring);
                return 1;
            }
        }
        unsigned head =
            atomic_load_explicit((_Atomic unsigned *)ring.completion_head, memory_order_relaxed);
        unsigned tail =
            atomic_load_explicit((_Atomic unsigned *)ring.completion_tail, memory_order_acquire);
        while (head != tail) {
            struct io_uring_cqe *completion = &ring.completions[head & *ring.completion_mask];
            unsigned long handle = (unsigned long)completion->user_data;
            int result = completion->res;
            struct wf_bench_slot *slot = &slots[handle];
            if (result > 0) {
                *bytes += (uint64_t)result;
                *sum += wf_bench_weighted(
                    wf_bench_digest(slot->window, wf_bench_check_length(window, (uint64_t)result)),
                    slot->index);
            }
            slot->descriptor = -1;
            free_slots[free_count++] = handle;
            in_flight--;
            head++;
        }
        atomic_store_explicit((_Atomic unsigned *)ring.completion_head, head,
                              memory_order_release);
    }
    free(free_slots);
    free(windows);
    free(slots);
    wf_bench_ring_teardown(&ring);
    return 0;
}

#endif
