/* Ordinary-call, current-stack compute scheduling. Derived from main
 * 9051576f; atomic ring cells and SC thief reads repair the two deque races.
 * Slots and lanes live until process exit. Only the offering thread releases
 * its slots; writer callbacks must not block on I/O or migrate stacks. */
#include "entry.h"
#include "prim.h"
#include <stdlib.h>
#include <stdio.h>

#define WF_PAR_MAX_LANES WF_SCHED_MAX_THREADS
#define WF_PAR_FRAME_BYTES WF_SCHED_FRAME_BYTES
#define WF_PAR_LANE_SLOTS WF_SCHED_LANE_SLOTS
#define WF_PAR_CACHE_LINE 128
#define WF_PAR_SPIN_ROUNDS 4096
#define WF_PAR_YIELD_ROUNDS 16
#define WF_PAR_SPLIT_OVERSUBSCRIBE 16
#define WF_PAR_SLOT_FREE 0
#define WF_PAR_SLOT_PENDING 1
#define WF_PAR_SLOT_DONE 2
struct wf__par_lane;

struct wf__par_slot {
    _Alignas(16) unsigned char frame[WF_PAR_FRAME_BYTES];
    void (*run)(void *);
    int state;
    struct wf__par_lane *waiter;
    struct wf__par_lane *home;
    int next_free;
};

struct wf__par_lane {
    _Alignas(WF_PAR_CACHE_LINE) unsigned long long top;
    _Alignas(WF_PAR_CACHE_LINE) unsigned long long bottom;
    struct wf__par_slot *buffer[WF_PAR_LANE_SLOTS];
    int free_head;
    unsigned long long seed;
    _Alignas(WF_PAR_CACHE_LINE) wf_prim_wait wait;
    int posted;
    _Alignas(WF_PAR_CACHE_LINE) uint64_t steals;
    _Alignas(WF_PAR_CACHE_LINE) struct wf__par_slot slots[WF_PAR_LANE_SLOTS];
};

static struct wf__par_lane wf__par_lanes[WF_PAR_MAX_LANES];

static int wf__par_lane_count;
static unsigned wf__par_started;

_Alignas(WF_PAR_CACHE_LINE) static unsigned long long wf__par_idle;

static _Thread_local struct wf__par_lane *wf__par_self;

static _Thread_local int wf__par_attached;

static unsigned wf__par_ready;
static wf_prim_thread wf__par_threads[WF_PAR_MAX_LANES];

extern size_t wf__floor_stack_bytes(void);

static void wf__par_signal(struct wf__par_lane *lane) {
    wf_prim_wait_lock(&lane->wait);
    lane->posted = 1;
    wf_prim_wait_signal(&lane->wait);
    wf_prim_wait_unlock(&lane->wait);
}

static void wf__par_wake_one(void) {
    unsigned long long parked = __atomic_load_n(&wf__par_idle, __ATOMIC_RELAXED);
    while (parked != 0) {
        int index = __builtin_ctzll(parked);
        unsigned long long bit = 1ull << index;

        if ((__atomic_fetch_and(&wf__par_idle, ~bit, __ATOMIC_ACQ_REL) & bit) != 0) {
            wf__par_signal(&wf__par_lanes[index]);
            return;
        }
        parked &= ~bit;
    }
}

/* The ring cell is atomic even for losing thieves delayed across wraparound.
 * Index publication orders the frame; only the successful claim permits a
 * thief to dereference it. SC thief index loads share the owner's pop order. */
static void wf__par_push(struct wf__par_lane *lane, struct wf__par_slot *slot) {
    unsigned long long bottom = lane->bottom;
    __atomic_store_n(&lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)], slot, __ATOMIC_RELAXED);
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_SEQ_CST);
}

static struct wf__par_slot *wf__par_pop(struct wf__par_lane *lane) {
    unsigned long long bottom = lane->bottom;
    unsigned long long top = __atomic_load_n(&lane->top, __ATOMIC_RELAXED);
    struct wf__par_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
    bottom -= 1;
    __atomic_store_n(&lane->bottom, bottom, __ATOMIC_SEQ_CST);
    top = __atomic_load_n(&lane->top, __ATOMIC_SEQ_CST);
    if ((long long)(bottom - top) < 0) {

        __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
        return NULL;
    }
    slot = __atomic_load_n(&lane->buffer[bottom & (WF_PAR_LANE_SLOTS - 1)], __ATOMIC_RELAXED);
    if ((long long)(bottom - top) > 0) {
        return slot;
    }

    if (!__atomic_compare_exchange_n(&lane->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
        slot = NULL;
    }
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
    return slot;
}

static struct wf__par_slot *wf__par_steal(struct wf__par_lane *victim) {
    unsigned long long top = __atomic_load_n(&victim->top, __ATOMIC_SEQ_CST);
    unsigned long long bottom = __atomic_load_n(&victim->bottom, __ATOMIC_SEQ_CST);
    struct wf__par_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        return NULL;
    }
#if defined(WF_SCHED_TEST)
    wf_sched_test_before_steal_read();
#endif
    slot = __atomic_load_n(&victim->buffer[top & (WF_PAR_LANE_SLOTS - 1)], __ATOMIC_RELAXED);
    if (!__atomic_compare_exchange_n(&victim->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
#if defined(WF_SCHED_TEST)
        wf_sched_test_after_steal(0);
#endif
        return NULL;
    }
#if defined(WF_SCHED_TEST)
    wf_sched_test_after_steal(1);
#endif
#if WF_SCHED_STATS
    uint64_t n = __atomic_load_n(&wf__par_self->steals, __ATOMIC_RELAXED);
    __atomic_store_n(&wf__par_self->steals, n + 1, __ATOMIC_RELAXED);
#endif
    return slot;
}

static struct wf__par_slot *wf__par_find(struct wf__par_lane *lane) {
    int count = __atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED);
    int offset;
    int step;
    if (count < 2) {
        return NULL;
    }
    lane->seed = lane->seed * 6364136223846793005ull + 1442695040888963407ull;
    offset = (int)((lane->seed >> 33) % (unsigned long long)count);
    for (step = 0; step < count; step += 1) {
        int index = offset + step;
        struct wf__par_lane *victim;
        struct wf__par_slot *slot;
        if (index >= count) {
            index -= count;
        }
        victim = &wf__par_lanes[index];
        if (victim == lane) {
            continue;
        }
        slot = wf__par_steal(victim);
        if (slot != NULL) {
            return slot;
        }
    }
    return NULL;
}

/* After DONE, only atomic waiter metadata in permanent slot storage may be
 * read. The owner can already reuse the frame; a late signal is harmless. */
static void wf__par_execute(struct wf__par_slot *slot) {
    struct wf__par_lane *waiter;
    slot->run(slot->frame);
    __atomic_store_n(&slot->state, WF_PAR_SLOT_DONE, __ATOMIC_SEQ_CST);
#if defined(WF_SCHED_TEST)
    wf_sched_test_after_done(slot->frame);
#endif
    waiter = __atomic_load_n(&slot->waiter, __ATOMIC_SEQ_CST);
    if (waiter != NULL) {
        wf__par_signal(waiter);
    }
}

/* Nested helping is safe for structured compute calls: no I/O continuation
 * can strand this stack. Register the waiter before the final SC DONE check;
 * signal takes the same native lock as sleep, so no wake can be lost. */
static void wf__par_wait(struct wf__par_lane *lane, struct wf__par_slot *target) {
    int rounds = 0;
    for (;;) {
        struct wf__par_slot *slot;
        if (__atomic_load_n(&target->state, __ATOMIC_ACQUIRE) == WF_PAR_SLOT_DONE) {
            return;
        }

        slot = wf__par_pop(lane);
        if (slot == NULL) {
            slot = wf__par_find(lane);
        }
        if (slot != NULL) {
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS) {
            rounds += 1;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
            rounds += 1;
            wf_prim_yield();
            continue;
        }

        wf_prim_wait_lock(&lane->wait);
        __atomic_store_n(&target->waiter, lane, __ATOMIC_SEQ_CST);
        while (__atomic_load_n(&target->state, __ATOMIC_SEQ_CST) != WF_PAR_SLOT_DONE) {
            wf_prim_wait_sleep(&lane->wait);
        }
        __atomic_store_n(&target->waiter, NULL, __ATOMIC_RELAXED);
        lane->posted = 0;
        wf_prim_wait_unlock(&lane->wait);
        return;
    }
}

static void wf__par_worker_main(void *opaque) {
    struct wf__par_lane *lane = (struct wf__par_lane *)opaque;
    int rounds = 0;
    wf__par_self = lane;
    wf__par_attached = 1;
    wf_prim_floor_attach();

    __atomic_add_fetch(&wf__par_ready, 1u, __ATOMIC_RELEASE);

    for (;;) {
        struct wf__par_slot *slot;
        slot = wf__par_find(lane);
        if (slot != NULL) {
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS) {
            rounds += 1;
            continue;
        }
        if (rounds < WF_PAR_SPIN_ROUNDS + WF_PAR_YIELD_ROUNDS) {
            rounds += 1;
            wf_prim_yield();
            continue;
        }

        /* Announce before the final scan: a publisher either sees our bit,
         * or its SC publication is visible to that scan. */
        __atomic_fetch_or(&wf__par_idle, 1ull << (lane - wf__par_lanes), __ATOMIC_SEQ_CST);
        slot = wf__par_find(lane);
        if (slot != NULL) {
            __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)),
                               __ATOMIC_ACQ_REL);
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        wf_prim_wait_lock(&lane->wait);
        if (!lane->posted) {

            wf_prim_wait_sleep(&lane->wait);
        }
        lane->posted = 0;
        wf_prim_wait_unlock(&lane->wait);
        __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)), __ATOMIC_ACQ_REL);
        rounds = 0;
    }

}

static void wf__par_prepare(struct wf__par_lane *lane, int index) {
    int slot;
    lane->top = 0;
    lane->bottom = 0;
    lane->seed = 0x9e3779b97f4a7c15ull * (unsigned long long)(index + 1);
    lane->posted = 0;
    for (slot = 0; slot < (int)WF_PAR_LANE_SLOTS; slot += 1) {
        lane->slots[slot].home = lane;
        lane->slots[slot].state = WF_PAR_SLOT_FREE;
        lane->slots[slot].waiter = NULL;
        lane->slots[slot].next_free = slot + 1;
    }
    lane->slots[WF_PAR_LANE_SLOTS - 1].next_free = -1;
    lane->free_head = 0;
}

static void wf__par_start(void) {
    int requested = wf__sched_lanes();
    int started = 0;
    if (requested < 2) return;
    /* Prepare every deque before publishing a scan bound. Initialize the
     * owner's wait before any worker exists; preserve started workers on
     * later failure, and never destroy a station a worker may reach. */
    for (int i = 0; i < requested; ++i) wf__par_prepare(&wf__par_lanes[i], i);
#if defined(WF_SCHED_TEST)
    if (!wf_sched_test_allow_owner_wait()) return;
#endif
    if (wf_prim_wait_init(&wf__par_lanes[0].wait) != 0) return;
    __atomic_store_n(&wf__par_lane_count, requested, __ATOMIC_RELAXED);
    for (int i = 1; i < requested; ++i) {
        struct wf__par_lane *lane = &wf__par_lanes[i];
        if (wf_prim_wait_init(&lane->wait) != 0) break;
#if defined(WF_SCHED_TEST)
        if (!wf_sched_test_allow_worker((unsigned)i)) {
            wf_prim_wait_destroy(&lane->wait);
            break;
        }
#endif
        if (wf_prim_thread_start(&wf__par_threads[i], wf__par_worker_main,
                lane, wf__floor_stack_bytes()) != 0) {
            wf_prim_wait_destroy(&lane->wait);
            break;
        }
        started = i;
    }
    __atomic_store_n(&wf__par_lane_count, started ? started + 1 : 0, __ATOMIC_RELAXED);
    if (!started) {
        wf_prim_wait_destroy(&wf__par_lanes[0].wait);
        return;
    }
    while (__atomic_load_n(&wf__par_ready, __ATOMIC_ACQUIRE) < (unsigned)started)
        wf_prim_yield();
}

static struct wf__par_lane *wf__par_attach(void) {
    static int taken;
    int expected = 0;
    wf__par_attached = 1;
    wf__sched_once(&wf__par_started, wf__par_start);
    if (__atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED) == 0) {
        return NULL;
    }
    if (!__atomic_compare_exchange_n(&taken, &expected, 1, 0, __ATOMIC_ACQ_REL,
                                     __ATOMIC_RELAXED)) {
        return NULL;
    }
    wf__par_self = &wf__par_lanes[0];
    return wf__par_self;
}

void *wf__par_acquire_lane(uint64_t bytes) {
    struct wf__par_lane *lane = wf__par_self;
    struct wf__par_slot *slot;
    int index;
    if (bytes > (unsigned long)WF_PAR_FRAME_BYTES) {
        return NULL;
    }
    if (lane == NULL) {
        if (wf__par_attached) {

            return NULL;
        }
        lane = wf__par_attach();
        if (lane == NULL) {
            return NULL;
        }
    }
    index = lane->free_head;
    if (index < 0) {
        return NULL;
    }
    slot = &lane->slots[index];
    lane->free_head = slot->next_free;
    return slot->frame;
}

void wf__par_publish(void *frame, void (*fn)(void *)) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    slot->run = fn;
    __atomic_store_n(&slot->state, WF_PAR_SLOT_PENDING, __ATOMIC_RELAXED);
    wf__par_push(slot->home, slot);
    if (__atomic_load_n(&wf__par_idle, __ATOMIC_SEQ_CST) != 0) {
        wf__par_wake_one();
    }
}

void wf__par_join(void *frame) {
    struct wf__par_slot *target = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = target->home;
    struct wf__par_slot *slot = wf__par_pop(lane);

    if (slot == target) {

        target->run(target->frame);
        return;
    }
    while (slot != NULL) {

        wf__par_execute(slot);
        if (__atomic_load_n(&target->state, __ATOMIC_ACQUIRE) == WF_PAR_SLOT_DONE) {
            return;
        }
        slot = wf__par_pop(lane);
        if (slot == target) {
            target->run(target->frame);
            return;
        }
    }
    wf__par_wait(lane, target);
}

void wf__par_release(void *frame) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = slot->home;
    __atomic_store_n(&slot->state, WF_PAR_SLOT_FREE, __ATOMIC_RELAXED);
    slot->next_free = lane->free_head;
    lane->free_head = (int)(slot - lane->slots);
}

int wf__par_pool_active(void) { return wf__sched_lanes() >= 2; }

uint64_t wf__par_split_budget(uint64_t span, uint64_t weight) {
    struct wf__par_lane *lane = wf__par_self;
    int lanes = wf__sched_lanes();
    uint64_t work = wf__sched_split_work();
    uint64_t want;
    uint64_t affordable;
    uint64_t chunks;
    uint64_t budget;
    if (lanes < 2 || work == 0) {
        return 0;
    }
    if (lane != NULL) {
        unsigned long long bottom = __atomic_load_n(&lane->bottom, __ATOMIC_RELAXED);
        unsigned long long top = __atomic_load_n(&lane->top, __ATOMIC_RELAXED);
        if ((long long)(bottom - top) > 0) {
            return 0;
        }
    }
    if (weight == 0) {
        weight = 1;
    }
    want = (uint64_t)lanes * WF_PAR_SPLIT_OVERSUBSCRIBE;

    if (weight >= work) {
        affordable = span;
    } else {
        affordable = span / ((work + weight - 1) / weight);
    }
    chunks = (want < affordable) ? want : affordable;

    budget = 0;
    while ((chunks >> 1) != 0) {
        chunks >>= 1;
        budget += 1;
    }
    return budget;
}

unsigned wf__sched_pool_running(void) {
    int count = __atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED);
    return count > 1 ? (unsigned)count - 1 : 0;
}
unsigned long wf__par_grants(void) {
    uint64_t sum = 0;
    for (unsigned i = 0; i < WF_PAR_MAX_LANES; ++i)
        sum += __atomic_load_n(&wf__par_lanes[i].steals, __ATOMIC_RELAXED);
    return (unsigned long)sum;
}
