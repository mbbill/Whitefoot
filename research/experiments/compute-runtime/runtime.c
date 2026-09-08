/* Research recovery of the compute runtime at fee335654d9dea027f4636bbad448d57a4e84d08.
 * This POSIX LP64 control uses ordinary worker stacks and nested helping.
 * The compiler's current opaque frame ABI is preserved; it links no I/O or
 * switch scheduler. See README.md for differences and qualification limits.
 * Storage and workers live until process exit; repeated calls use one owner.
 */
#include "runtime.h"

#include <pthread.h>
#include <sched.h>
#include <stdlib.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <sys/sysctl.h>
#endif

#define WF_PAR_MAX_LANES 64

#define WF_PAR_FRAME_BYTES 256

_Static_assert(sizeof(unsigned long) == 8, "compute control requires POSIX LP64");

#define WF_PAR_LANE_SLOTS 64

#define WF_PAR_CACHE_LINE 128

#define WF_PAR_SPIN_ROUNDS 4096
#define WF_PAR_YIELD_ROUNDS 16

#define WF_PAR_SPLIT_OVERSUBSCRIBE 16
#define WF_PAR_SPLIT_WORK_PER_CHUNK 1200000

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

    _Alignas(WF_PAR_CACHE_LINE) pthread_mutex_t lock;
    pthread_cond_t signal;

    int posted;

    _Alignas(WF_PAR_CACHE_LINE) struct wf__par_slot slots[WF_PAR_LANE_SLOTS];
};

static struct wf__par_lane wf__par_lanes[WF_PAR_MAX_LANES];

static int wf__par_lane_count;
static pthread_once_t wf__par_started = PTHREAD_ONCE_INIT;

_Alignas(WF_PAR_CACHE_LINE) static unsigned long long wf__par_idle;

#if WF_COMPUTE_STATS
_Alignas(WF_PAR_CACHE_LINE) static unsigned long wf_compute_steal_count;
#endif

static _Thread_local struct wf__par_lane *wf__par_self;

static _Thread_local int wf__par_attached;
#if defined(WF_COMPUTE_EVENTS)
#include "runtime_events.h"
/* Diagnostic-only, cumulative lane-local banks. Atomic reads permit snapshots
 * while background searches continue; such snapshots are not instantaneous.
 * Keep bank storage separate from the ordinary slot/deque layouts. */
struct wf_event_bank {
    _Alignas(WF_PAR_CACHE_LINE) unsigned long values[WF_EVENT_COUNT];
};
static struct wf_event_bank wf_events[WF_PAR_MAX_LANES];
static void wf_event(struct wf__par_lane *lane, unsigned event) {
    if (lane != NULL)
        __atomic_add_fetch(&wf_events[lane - wf__par_lanes].values[event], 1, __ATOMIC_RELAXED);
}
unsigned long wf_compute_event(unsigned lane, unsigned event) {
    if (lane >= WF_PAR_MAX_LANES || event >= WF_EVENT_COUNT) abort();
    return __atomic_load_n(&wf_events[lane].values[event], __ATOMIC_RELAXED);
}
#define EVENT(lane, name) wf_event((lane), WF_EVENT_##name)
#else
#define EVENT(lane, name) ((void)0)
#endif


static int wf__par_ready;
static pthread_mutex_t wf__par_ready_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t wf__par_ready_signal = PTHREAD_COND_INITIALIZER;

extern size_t wf__floor_stack_bytes(void);

static void wf__par_signal(struct wf__par_lane *lane) {
    pthread_mutex_lock(&lane->lock);
    lane->posted = 1;
    EVENT(wf__par_self, SIGNAL);
    pthread_cond_signal(&lane->signal);
    pthread_mutex_unlock(&lane->lock);
}

static void wf__par_wake_one(void) {
    unsigned long long parked = __atomic_load_n(&wf__par_idle, __ATOMIC_RELAXED);
    while (parked != 0) {
        int index = __builtin_ctzll(parked);
        unsigned long long bit = 1ull << index;

        if ((__atomic_fetch_and(&wf__par_idle, ~bit, __ATOMIC_ACQ_REL) & bit) != 0) {
            EVENT(wf__par_self, IDLE_CLAIM);
            wf__par_signal(&wf__par_lanes[index]);
            return;
        }
        parked &= ~bit;
    }
}

static void wf__par_push(struct wf__par_lane *lane, struct wf__par_slot *slot) {
    /* Pointer cells are atomic even though only the owner writes them: a
     * delayed thief may still read a cell after the ring wraps. The index
     * publication carries the frame stores; its CAS arbitrates ownership. */
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
        EVENT(lane, LOCAL_POP);
        return slot;
    }

    if (!__atomic_compare_exchange_n(&lane->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
        slot = NULL;
    }
    __atomic_store_n(&lane->bottom, bottom + 1, __ATOMIC_RELAXED);
    if (slot != NULL) EVENT(lane, LOCAL_POP);
    return slot;
}

static struct wf__par_slot *wf__par_steal(struct wf__par_lane *victim) {
    EVENT(wf__par_self, STEAL_ATTEMPT);
    unsigned long long top = __atomic_load_n(&victim->top, __ATOMIC_ACQUIRE);
    unsigned long long bottom = __atomic_load_n(&victim->bottom, __ATOMIC_ACQUIRE);
    struct wf__par_slot *slot;
    if ((long long)(bottom - top) <= 0) {
        EVENT(wf__par_self, STEAL_EMPTY);
        return NULL;
    }
#if defined(WF_COMPUTE_TEST)
    wf_compute_test_before_steal_read((unsigned)(victim - wf__par_lanes), top);
#endif
    slot = __atomic_load_n(&victim->buffer[top & (WF_PAR_LANE_SLOTS - 1)], __ATOMIC_RELAXED);
#if defined(WF_COMPUTE_TEST)
    wf_compute_test_after_steal_read(slot);
#endif
    if (!__atomic_compare_exchange_n(&victim->top, &top, top + 1, 0, __ATOMIC_SEQ_CST,
                                     __ATOMIC_RELAXED)) {
        EVENT(wf__par_self, STEAL_CAS_FAIL);
#if defined(WF_COMPUTE_TEST)
        wf_compute_test_after_steal(0);
#endif
        return NULL;
    }
#if defined(WF_COMPUTE_TEST)
    wf_compute_test_after_steal(1);
#endif
    EVENT(wf__par_self, STEAL_SUCCESS);
#if WF_COMPUTE_STATS
    __atomic_add_fetch(&wf_compute_steal_count, 1, __ATOMIC_RELAXED);
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

static void wf__par_execute(struct wf__par_slot *slot) {
    struct wf__par_lane *waiter;
    EVENT(wf__par_self, RUN_BEGIN);
    slot->run(slot->frame);
    EVENT(wf__par_self, RUN_END);
    /* After DONE the owner may read, release and reuse the frame. Only atomic
     * waiter metadata and permanent lane storage may be accessed afterward. */
    __atomic_store_n(&slot->state, WF_PAR_SLOT_DONE, __ATOMIC_SEQ_CST);
#if defined(WF_COMPUTE_TEST)
    wf_compute_test_after_done(slot->frame);
#endif
    waiter = __atomic_load_n(&slot->waiter, __ATOMIC_SEQ_CST);
    if (waiter != NULL) {
        wf__par_signal(waiter);
    }
#if defined(WF_COMPUTE_TEST)
    wf_compute_test_after_completion_tail();
#endif
}

static void wf__par_wait(struct wf__par_lane *lane, struct wf__par_slot *target) {
    /* Nonblocking, structured compute work can run nested here. No stack is
     * suspended or migrated. Empty searches eventually yield and sleep. */
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
            EVENT(lane, JOIN_YIELD);
            sched_yield();
            continue;
        }

        pthread_mutex_lock(&lane->lock);
        __atomic_store_n(&target->waiter, lane, __ATOMIC_SEQ_CST);
        while (__atomic_load_n(&target->state, __ATOMIC_SEQ_CST) != WF_PAR_SLOT_DONE) {
            EVENT(lane, JOIN_PARK);
            pthread_cond_wait(&lane->signal, &lane->lock);
            EVENT(lane, JOIN_RESUME);
        }
        __atomic_store_n(&target->waiter, NULL, __ATOMIC_RELAXED);
        lane->posted = 0;
        pthread_mutex_unlock(&lane->lock);
        return;
    }
}

extern void wf__floor_attach_thread(void);

static void *wf__par_worker_main(void *opaque) {
    struct wf__par_lane *lane = (struct wf__par_lane *)opaque;
    int rounds = 0;
    wf__par_self = lane;
    wf__par_attached = 1;
    wf__floor_attach_thread();

    pthread_mutex_lock(&wf__par_ready_lock);
    wf__par_ready += 1;
    pthread_cond_signal(&wf__par_ready_signal);
    pthread_mutex_unlock(&wf__par_ready_lock);

    for (;;) {
        struct wf__par_slot *slot = wf__par_find(lane);
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
            EVENT(lane, IDLE_YIELD);
            sched_yield();
            continue;
        }

        __atomic_fetch_or(&wf__par_idle, 1ull << (lane - wf__par_lanes), __ATOMIC_SEQ_CST);
        slot = wf__par_find(lane);
        if (slot != NULL) {
            __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)),
                               __ATOMIC_ACQ_REL);
            wf__par_execute(slot);
            rounds = 0;
            continue;
        }
        pthread_mutex_lock(&lane->lock);
        if (!lane->posted) {

            EVENT(lane, IDLE_PARK);
            pthread_cond_wait(&lane->signal, &lane->lock);
            EVENT(lane, IDLE_RESUME);
        }
        lane->posted = 0;
        pthread_mutex_unlock(&lane->lock);
        __atomic_fetch_and(&wf__par_idle, ~(1ull << (lane - wf__par_lanes)), __ATOMIC_ACQ_REL);
        rounds = 0;
    }
    return NULL;
}

static void wf__par_prepare(struct wf__par_lane *lane, int index) {
    int slot;
    lane->top = 0;
    lane->bottom = 0;
    lane->seed = 0x9e3779b97f4a7c15ull * (unsigned long long)(index + 1);
    lane->posted = 0;
    for (slot = 0; slot < WF_PAR_LANE_SLOTS; slot += 1) {
        lane->slots[slot].home = lane;
        lane->slots[slot].state = WF_PAR_SLOT_FREE;
        lane->slots[slot].waiter = NULL;
        lane->slots[slot].next_free = slot + 1;
    }
    lane->slots[WF_PAR_LANE_SLOTS - 1].next_free = -1;
    lane->free_head = 0;
}

static int wf__par_default_lanes(void) {
    long online;
#if defined(__APPLE__)
    int logical = 0;
    size_t width = sizeof(logical);
    if (sysctlbyname("hw.logicalcpu", &logical, &width, NULL, 0) == 0 && logical >= 2) {
        return (logical > WF_PAR_MAX_LANES) ? WF_PAR_MAX_LANES : logical;
    }
#endif
    online = sysconf(_SC_NPROCESSORS_ONLN);
    if (online < 2) {
        return 0;
    }
    return (online > WF_PAR_MAX_LANES) ? WF_PAR_MAX_LANES : (int)online;
}

static int wf__par_requested_lanes(void) {
    const char *setting = getenv("WF_WORKERS");
    char *end = NULL;
    long requested;
    if (setting == NULL || setting[0] == '\0') {
        return wf__par_default_lanes();
    }
    requested = strtol(setting, &end, 10);
    if (end == setting || *end != '\0' || requested < 2) {
        return 0;
    }
    if (requested > WF_PAR_MAX_LANES) {
        requested = WF_PAR_MAX_LANES;
    }
    return (int)requested;
}

static void wf__par_start(void) {
    pthread_attr_t attributes;
    int requested = wf__par_requested_lanes();
    int started = 0;
    int index;
    if (requested < 2) {
        return;
    }
    if (pthread_attr_init(&attributes) != 0) {
        return;
    }

    if (pthread_attr_setstacksize(&attributes, wf__floor_stack_bytes()) != 0) {
        pthread_attr_destroy(&attributes);
        return;
    }
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    for (index = 0; index < requested; index += 1) {
        wf__par_prepare(&wf__par_lanes[index], index);
    }

    /* No worker is created until the owner's wait station is usable. */
    if (pthread_mutex_init(&wf__par_lanes[0].lock, NULL) != 0) {
        pthread_attr_destroy(&attributes);
        return;
    }
    if (pthread_cond_init(&wf__par_lanes[0].signal, NULL) != 0) {
        pthread_mutex_destroy(&wf__par_lanes[0].lock);
        pthread_attr_destroy(&attributes);
        return;
    }
    __atomic_store_n(&wf__par_lane_count, requested, __ATOMIC_RELAXED);
    for (index = 1; index < requested; index += 1) {
        pthread_t thread;
        int created = -1;
        struct wf__par_lane *lane = &wf__par_lanes[index];
        if (pthread_mutex_init(&lane->lock, NULL) != 0) {
            break;
        }
        if (pthread_cond_init(&lane->signal, NULL) != 0) {
            pthread_mutex_destroy(&lane->lock);
            break;
        }
#if defined(WF_COMPUTE_TEST)
        if (wf_compute_test_allow_worker((unsigned)index))
#endif
        {
            created = pthread_create(&thread, &attributes, wf__par_worker_main, lane);
        }
        if (created != 0) {
            pthread_cond_destroy(&lane->signal);
            pthread_mutex_destroy(&lane->lock);
            break;
        }
        started = index;
    }
    __atomic_store_n(&wf__par_lane_count, started + 1, __ATOMIC_RELAXED);
    pthread_attr_destroy(&attributes);
    if (started == 0) {
        __atomic_store_n(&wf__par_lane_count, 0, __ATOMIC_RELAXED);
        pthread_cond_destroy(&wf__par_lanes[0].signal);
        pthread_mutex_destroy(&wf__par_lanes[0].lock);
        return;
    }

    pthread_mutex_lock(&wf__par_ready_lock);
    while (wf__par_ready < started) {
        pthread_cond_wait(&wf__par_ready_signal, &wf__par_ready_lock);
    }
    pthread_mutex_unlock(&wf__par_ready_lock);
}

static struct wf__par_lane *wf__par_attach(void) {
    static int taken;
    int expected = 0;
    wf__par_attached = 1;
    pthread_once(&wf__par_started, wf__par_start);
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

void *wf__par_acquire_lane(unsigned long bytes) {
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
        EVENT(lane, SLOT_REFUSAL);
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
    EVENT(slot->home, PUBLISH);
    wf__par_push(slot->home, slot);
    if (__atomic_load_n(&wf__par_idle, __ATOMIC_SEQ_CST) != 0) {
        wf__par_wake_one();
    }
}

void wf__par_join(void *frame) {
    struct wf__par_slot *target = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = target->home;
    EVENT(lane, JOIN);
    struct wf__par_slot *slot = wf__par_pop(lane);

    if (slot == target) {

        EVENT(lane, INLINE_RUN);
        EVENT(lane, RUN_BEGIN);
        target->run(target->frame);
        EVENT(lane, RUN_END);
        return;
    }
    while (slot != NULL) {

        wf__par_execute(slot);
        if (__atomic_load_n(&target->state, __ATOMIC_ACQUIRE) == WF_PAR_SLOT_DONE) {
            return;
        }
        slot = wf__par_pop(lane);
        if (slot == target) {
            EVENT(lane, INLINE_RUN);
        EVENT(lane, RUN_BEGIN);
        target->run(target->frame);
        EVENT(lane, RUN_END);
            return;
        }
    }
    EVENT(lane, JOIN_WAIT);
    wf__par_wait(lane, target);
}

void wf__par_release(void *frame) {
    struct wf__par_slot *slot = (struct wf__par_slot *)frame;
    struct wf__par_lane *lane = slot->home;
    __atomic_store_n(&slot->state, WF_PAR_SLOT_FREE, __ATOMIC_RELAXED);
    slot->next_free = lane->free_head;
    lane->free_head = (int)(slot - lane->slots);
}

int wf__par_pool_active(void) {
    return wf__par_requested_lanes() >= 2;
}

static int wf__par_cached_lanes = -1;

static int wf__par_lanes_once(void) {
    int lanes = __atomic_load_n(&wf__par_cached_lanes, __ATOMIC_RELAXED);
    if (lanes < 0) {
        lanes = wf__par_requested_lanes();
        __atomic_store_n(&wf__par_cached_lanes, lanes, __ATOMIC_RELAXED);
    }
    return lanes;
}

unsigned long wf__par_split_budget(unsigned long span, unsigned long weight) {
    struct wf__par_lane *lane = wf__par_self;
    int lanes = wf__par_lanes_once();
    unsigned long want;
    unsigned long affordable;
    unsigned long chunks;
    unsigned long budget;
    if (lanes < 2) {
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
    want = (unsigned long)lanes * WF_PAR_SPLIT_OVERSUBSCRIBE;

    if (weight >= (unsigned long)WF_PAR_SPLIT_WORK_PER_CHUNK) {
        affordable = span;
    } else {
        affordable = span / (((unsigned long)WF_PAR_SPLIT_WORK_PER_CHUNK + weight - 1) / weight);
    }
    chunks = (want < affordable) ? want : affordable;

    budget = 0;
    while ((chunks >> 1) != 0) {
        chunks >>= 1;
        budget += 1;
    }
    return budget;
}

#if WF_COMPUTE_STATS
unsigned long wf__par_grants(void) {
    return __atomic_load_n(&wf_compute_steal_count, __ATOMIC_RELAXED);
}
#endif

unsigned wf_compute_worker_count(void) {
    return (unsigned)__atomic_load_n(&wf__par_lane_count, __ATOMIC_RELAXED);
}

unsigned wf_compute_slot_capacity(void) {
    return WF_PAR_LANE_SLOTS;
}
