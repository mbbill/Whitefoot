/* Link the unchanged FIR oracle with main and its four native entry references
 * renamed to these adapters. The actual native object keeps its ordinary
 * symbols and internal tail calls, and is identical to the timing object. */
#include "fir_static.h"
#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

extern int fir_qualified_oracle_main(void);

static FirStatic *pool;
#ifdef FIR_STATIC_TEST
static unsigned allowed_helpers = 3;
static pthread_t owner, helpers[4];
static atomic_uint_fast64_t kernel_calls[4], kernel_samples[4];
static pthread_mutex_t hold_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t hold_condition = PTHREAD_COND_INITIALIZER;
static int hold_enabled, held, caller_finished, owner_waiting, release_worker, returned;

int fir_static_test_create(unsigned index, pthread_t *thread,
                           void *(*entry)(void *), void *argument) {
    if (index > allowed_helpers) return EAGAIN;
    int result = pthread_create(thread, NULL, entry, argument);
    if (result == 0) helpers[index] = *thread;
    return result;
}

void fir_static_test_before_kernel(unsigned index, size_t count) {
    assert(index < 4 && count > 0);
    assert(pthread_equal(pthread_self(), index ? helpers[index] : owner));
    atomic_fetch_add(&kernel_calls[index], 1);
    atomic_fetch_add(&kernel_samples[index], count);
    assert(pthread_mutex_lock(&hold_lock) == 0);
    if (hold_enabled && index == 1) {
        held = 1;
        assert(pthread_cond_broadcast(&hold_condition) == 0);
        while (!release_worker)
            assert(pthread_cond_wait(&hold_condition, &hold_lock) == 0);
    }
    assert(pthread_mutex_unlock(&hold_lock) == 0);
}

void fir_static_test_after_kernel(unsigned index, size_t count) {
    (void)count;
    assert(pthread_mutex_lock(&hold_lock) == 0);
    if (hold_enabled && index == 0) {
        caller_finished = 1;
        assert(pthread_cond_broadcast(&hold_condition) == 0);
    }
    assert(pthread_mutex_unlock(&hold_lock) == 0);
}

int fir_static_test_wait(pthread_cond_t *condition, pthread_mutex_t *lock) {
    assert(pthread_mutex_lock(&hold_lock) == 0);
    if (hold_enabled) {
        owner_waiting = 1;
        assert(pthread_cond_broadcast(&hold_condition) == 0);
    }
    assert(pthread_mutex_unlock(&hold_lock) == 0);
    return pthread_cond_wait(condition, lock);
}

#endif

#define ADAPTER(name) \
void fir_static_check_##name(const double *prefix, const double *taps, size_t k, \
                        size_t count, double *output) { \
    fir_static_run(pool, fir_native_##name, prefix, taps, k, count, output); \
}
ADAPTER(direct)
ADAPTER(lanes4)
ADAPTER(lanes8)
ADAPTER(lanes16)
#undef ADAPTER

#ifdef FIR_STATIC_TEST
static void *release_held_worker(void *unused) {
    (void)unused;
    assert(pthread_mutex_lock(&hold_lock) == 0);
    while (!held || !caller_finished || !owner_waiting)
        assert(pthread_cond_wait(&hold_condition, &hold_lock) == 0);
    assert(!returned);
    release_worker = 1;
    assert(pthread_cond_broadcast(&hold_condition) == 0);
    assert(pthread_mutex_unlock(&hold_lock) == 0);
    return NULL;
}

static void check_held_borrow(unsigned lanes) {
    if (lanes < 2) return;
    double prefix[129], output[129], taps[1] = {1.0};
    for (unsigned i = 0; i < 129; ++i) prefix[i] = (double)i / 7.0;
    hold_enabled = 1;
    pthread_t release_thread;
    assert(pthread_create(&release_thread, NULL, release_held_worker, NULL) == 0);
    fir_static_run(pool, fir_native_lanes16, prefix, taps, 1, 129, output);
    assert(pthread_mutex_lock(&hold_lock) == 0);
    returned = 1;
    assert(held && caller_finished && owner_waiting && release_worker);
    hold_enabled = 0;
    assert(pthread_mutex_unlock(&hold_lock) == 0);
    assert(pthread_join(release_thread, NULL) == 0);
    assert(memcmp(prefix, output, sizeof(output)) == 0);
    /* Both stack buffers are immediately reused after the synchronous call. */
    memset(prefix, 0xa5, sizeof(prefix));
    memset(output, 0x5a, sizeof(output));
}

static void check_recreate(void) {
    const size_t lengths[] = {0, 1, 2, 3, 4, 7, 129};
    unsigned cycles = 0;
    for (unsigned repeat = 0; repeat < 4; ++repeat) {
        for (unsigned requested = 1; requested <= 4; requested *= 2) {
            FirStaticInfo info;
            pool = fir_static_create(requested, &info);
            assert(pool != NULL);
            unsigned expected = requested - 1 < allowed_helpers ? requested : allowed_helpers + 1;
            assert(info.actual_lanes == expected);
            assert(info.creation_error == (expected < requested ? EAGAIN : 0));
            double prefix[129], output[131], taps[1] = {1.0};
            for (unsigned i = 0; i < 129; ++i) prefix[i] = (double)(i + repeat) / 7.0;
            for (size_t i = 0; i < sizeof(lengths) / sizeof(lengths[0]); ++i) {
                for (unsigned j = 0; j < 131; ++j) output[j] = -918.0;
                fir_static_run(pool, fir_native_lanes16, prefix, taps, 1, lengths[i], output + 1);
                assert(memcmp(prefix, output + 1, lengths[i] * sizeof(double)) == 0);
                assert(output[0] == -918.0 && output[lengths[i] + 1] == -918.0);
            }
            fir_static_destroy(pool);
            pool = NULL;
            ++cycles;
        }
    }
    printf("FIR static recreate PASS: cycles=%u runs=%u\n", cycles, cycles * 7);
}

int main(int argc, char **argv) {
    assert(argc == 3 || (argc == 4 && strcmp(argv[3], "held") == 0));
    alarm(120); /* Failure watchdog for qualification, never a timing policy. */
    unsigned requested = (unsigned)strtoul(argv[1], NULL, 10);
    allowed_helpers = (unsigned)strtoul(argv[2], NULL, 10);
    owner = pthread_self();
    FirStaticInfo info;
    errno = 0;
    assert(fir_static_create(3, &info) == NULL && errno == EINVAL);
    pool = fir_static_create(requested, &info);
    assert(pool != NULL);
    unsigned expected = requested - 1 < allowed_helpers ? requested : allowed_helpers + 1;
    assert(info.requested_lanes == requested && info.actual_lanes == expected);
    assert(info.creation_error == (expected < requested ? EAGAIN : 0));
    if (argc == 3) assert(fir_qualified_oracle_main() == 0);
    check_held_borrow(info.actual_lanes);
    fir_static_destroy(pool);
    pool = NULL;
    uint64_t sum = 0;
    for (unsigned i = 0; i < 4; ++i) {
        uint64_t calls = atomic_load(&kernel_calls[i]);
        uint64_t samples = atomic_load(&kernel_samples[i]);
        assert(i < expected ? calls > 0 : calls == 0);
        sum += samples;
        printf("FIR static lane: lane=%u calls=%llu samples=%llu\n", i,
               (unsigned long long)calls, (unsigned long long)samples);
    }
    assert(sum == (argc == 3 ? UINT64_C(1294336) : 0) + (expected > 1 ? 129 : 0));
    printf("FIR static qualification PASS: requested=%u actual=%u helpers=%u creation_error=%d idle=condvar spin=0\n",
           requested, info.actual_lanes, info.actual_lanes - 1, info.creation_error);
    check_recreate();
    alarm(0);
    return 0;
}
#else
int main(int argc, char **argv) {
    assert(argc == 2);
    alarm(120);
    unsigned requested = (unsigned)strtoul(argv[1], NULL, 10);
    FirStaticInfo info;
    pool = fir_static_create(requested, &info);
    assert(pool != NULL && info.actual_lanes == requested && info.creation_error == 0);
    assert(fir_qualified_oracle_main() == 0);
    fir_static_destroy(pool);
    pool = NULL;
    printf("FIR static ordinary PASS: requested=%u actual=%u helpers=%u idle=condvar spin=0\n",
           requested, info.actual_lanes, info.actual_lanes - 1);
    alarm(0);
    return 0;
}
#endif
