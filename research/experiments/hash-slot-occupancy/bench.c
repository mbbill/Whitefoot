#ifdef __APPLE__
#define _DARWIN_C_SOURCE
#else
#define _POSIX_C_SOURCE 200809L
#endif
#include <assert.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <time.h>
#ifdef __APPLE__
#include <malloc/malloc.h>
#endif

/* A scalar representation experiment, not a production hash table. */
enum { RAW, TAG, BOX, FOLDED, PROVED, VARIANTS };
static const char *names[] = {"a_raw", "b_tag", "c_box", "d_folded", "e_proved"};
static volatile uint64_t sink;
typedef struct Table Table;
typedef uint64_t (*Lookup)(const Table *, uint64_t);
typedef int (*Insert)(Table *, uint64_t);
typedef uint64_t (*Batch)(const Table *, const uint64_t *, size_t, int, uint64_t);
typedef uint64_t (*Build)(Table *, size_t);
struct Table {
    uint8_t *ctrl;
    void *slots;
    size_t cap, payload, stride;
    int variant;
    Lookup lookup;
    Insert insert;
    Batch batch;
    Build build;
};

static void *allocate(size_t bytes) {
    void *p = malloc(bytes);
    if (!p) { fputs("allocation failed\n", stderr); exit(1); }
    return p;
}
static uint64_t mix64(uint64_t x) {
    x = (x ^ (x >> 30)) * UINT64_C(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)) * UINT64_C(0x94d049bb133111eb);
    return x ^ (x >> 31);
}
static uint64_t random64(uint64_t *s) {
    *s += UINT64_C(0x9e3779b97f4a7c15);
    return mix64(*s);
}
/* The bijective mixer gives distinct keys for distinct input indices. */
static uint64_t key_at(size_t i) {
    return mix64((uint64_t)i + UINT64_C(0xa0761d6478bd642f));
}
static uint64_t word_at(uint64_t key, size_t word) {
    return (key ^ (UINT64_C(0xe7037ed1a0b428db) + word)) | 1;
}

#define TYPES(P) \
    typedef struct { uint64_t key, value[(P) / 8]; } Entry##P; \
    typedef struct { uint8_t tag; Entry##P entry; } Option##P;
TYPES(8)
TYPES(32)
TYPES(128)

/* READ declares e. The only lookup difference is the slot expression/guard. */
#define READ_RAW(P) const Entry##P *e = &((const Entry##P *)t->slots)[i]
#define READ_TAG(P) const Option##P *s = &((const Option##P *)t->slots)[i]; \
    const Entry##P *e = s->tag ? &s->entry : NULL
#define READ_BOX(P) const Entry##P *e = ((void *const *)t->slots)[i]
#define READ_PROVED(P) const Entry##P *e = &((const Option##P *)t->slots)[i].entry
#define WRITE_RAW(P) Entry##P *dst = &((Entry##P *)t->slots)[i]
#define WRITE_TAG(P) Option##P *s = &((Option##P *)t->slots)[i]; \
    s->tag = 1; Entry##P *dst = &s->entry
#define WRITE_BOX(P) Entry##P *dst = allocate(sizeof(Entry##P)); \
    ((void **)t->slots)[i] = dst

#define DEFINE(P, ID, READ, WRITE, VALID) \
    static inline uint64_t probe_##ID##_##P(const Table *t, uint64_t key) { \
        uint64_t h = mix64(key); \
        size_t i = (size_t)h & (t->cap - 1); \
        uint8_t fp = (uint8_t)((h >> 57) + 1); \
        for (;;) { \
            uint8_t c = t->ctrl[i]; \
            if (!c) return 0; \
            if (c == fp) { \
                READ(P); \
                if ((VALID) && e->key == key) return e->value[0]; \
            } \
            i = (i + 1) & (t->cap - 1); \
        } \
    } \
    static inline int put_##ID##_##P(Table *t, uint64_t key) { \
        uint64_t h = mix64(key); \
        size_t i = (size_t)h & (t->cap - 1); \
        uint8_t fp = (uint8_t)((h >> 57) + 1); \
        while (t->ctrl[i]) { \
            if (t->ctrl[i] == fp) { \
                READ(P); \
                if ((VALID) && e->key == key) return 0; \
            } \
            i = (i + 1) & (t->cap - 1); \
        } \
        WRITE(P); \
        dst->key = key; \
        for (size_t w = 0; w < (P) / 8; ++w) dst->value[w] = word_at(key, w); \
        t->ctrl[i] = fp; \
        return 1; \
    } \
    __attribute__((noinline)) uint64_t lookup_##ID##_##P( \
        const Table *t, const uint64_t *q, size_t ops, int dependent, uint64_t seed) { \
        uint64_t sum = 0; \
        size_t mask = t->cap - 1; \
        if (dependent) { \
            for (size_t j = 0; j < ops; ++j) { \
                uint64_t v = probe_##ID##_##P(t, q[seed & mask]); \
                sum += v; \
                seed = mix64(seed ^ v); \
            } \
        } else { \
            for (size_t j = 0; j < ops; ++j) \
                sum += probe_##ID##_##P(t, q[(j + seed) & mask]); \
        } \
        return sum ^ seed; \
    } \
    __attribute__((noinline)) uint64_t build_##ID##_##P(Table *t, size_t n) { \
        uint64_t sum = 0; \
        for (size_t j = 0; j < n; ++j) { \
            sum += (uint64_t)put_##ID##_##P(t, key_at(j)); \
            if ((j & 3) == 3) \
                sum += probe_##ID##_##P(t, key_at(j / 2)); \
        } \
        return sum; \
    }
#define ALL(P) \
    DEFINE(P, raw, READ_RAW, WRITE_RAW, 1) \
    DEFINE(P, tag, READ_TAG, WRITE_TAG, e != NULL) \
    DEFINE(P, box, READ_BOX, WRITE_BOX, e != NULL) \
    DEFINE(P, folded, READ_RAW, WRITE_RAW, 1) \
    DEFINE(P, proved, READ_PROVED, WRITE_TAG, 1)
ALL(8)
ALL(32)
ALL(128)

#define SELECT(P) do { \
    Lookup lookups[] = {probe_raw_##P, probe_tag_##P, probe_box_##P, \
        probe_folded_##P, probe_proved_##P}; \
    Insert inserts[] = {put_raw_##P, put_tag_##P, put_box_##P, \
        put_folded_##P, put_proved_##P}; \
    Batch batches[] = {lookup_raw_##P, lookup_tag_##P, lookup_box_##P, \
        lookup_folded_##P, lookup_proved_##P}; \
    Build builds[] = {build_raw_##P, build_tag_##P, build_box_##P, \
        build_folded_##P, build_proved_##P}; \
    t.lookup = lookups[variant]; t.insert = inserts[variant]; \
    t.batch = batches[variant]; t.build = builds[variant]; \
    t.stride = variant == BOX ? sizeof(Entry##P *) : \
        (variant == TAG || variant == PROVED ? sizeof(Option##P) : sizeof(Entry##P)); \
} while (0)

static void reset(Table *t) {
    if (t->variant == BOX) {
        void **p = t->slots;
        for (size_t i = 0; i < t->cap; ++i) { free(p[i]); p[i] = NULL; }
    } else if (t->variant == TAG || t->variant == PROVED) {
        for (size_t i = 0; i < t->cap; ++i)
            ((uint8_t *)t->slots)[i * t->stride] = 0;
    }
    memset(t->ctrl, 0, t->cap);
}
static Table create(size_t cap, size_t payload, int variant) {
    Table t = {.cap = cap, .payload = payload, .variant = variant};
    switch (payload) {
        case 8: SELECT(8); break;
        case 32: SELECT(32); break;
        case 128: SELECT(128); break;
        default: abort();
    }
    t.ctrl = allocate(cap);
    t.slots = allocate(cap * t.stride);
    if (variant == BOX) {
        void **p = t.slots;
        for (size_t i = 0; i < cap; ++i) p[i] = NULL;
    }
    reset(&t);
    return t;
}
static void destroy(Table *t) { reset(t); free(t->slots); free(t->ctrl); }
static size_t heap_bytes(const Table *t, size_t n) {
    if (t->variant != BOX) return 0;
#ifdef __APPLE__
    size_t bytes = 0;
    void *const *p = t->slots;
    for (size_t i = 0; i < t->cap; ++i) if (p[i]) bytes += malloc_size(p[i]);
    (void)n;
    return bytes;
#else
    return n * (t->payload + 8); /* Requested bytes; allocator metadata unknown. */
#endif
}
static uint64_t now_ns(void) {
    struct timespec ts;
#ifdef __APPLE__
    const clockid_t clock_id = CLOCK_MONOTONIC_RAW;
#else
    const clockid_t clock_id = CLOCK_MONOTONIC;
#endif
    if (clock_gettime(clock_id, &ts)) { perror("clock_gettime"); exit(1); }
    return (uint64_t)ts.tv_sec * UINT64_C(1000000000) + (uint64_t)ts.tv_nsec;
}
typedef struct { uint64_t ns, checksum; size_t ops, builds; long minor, major, cs; } Sample;
static void measure(Table *t, const uint64_t *q, size_t ops, int dependent,
                    uint64_t seed, size_t build_n, Sample *s) {
    struct rusage before, after;
    if (getrusage(RUSAGE_SELF, &before)) abort();
    uint64_t start = now_ns();
    uint64_t result = build_n ? t->build(t, build_n) : t->batch(t, q, ops, dependent, seed);
    uint64_t end = now_ns();
    if (getrusage(RUSAGE_SELF, &after)) abort();
    sink = result;
    s->ns += end - start;
    s->checksum += result;
    s->ops += ops;
    s->builds += build_n != 0;
    s->minor += after.ru_minflt - before.ru_minflt;
    s->major += after.ru_majflt - before.ru_majflt;
    s->cs += after.ru_nivcsw - before.ru_nivcsw;
}
static size_t capacity(int tier, size_t payload) {
    size_t cap = 1;
    if (tier == 2) {
        while (cap * (payload + 9) < (size_t)128 * 1024 * 1024) cap *= 2;
    } else {
        size_t limit = (tier ? (size_t)1024 * 1024 : 32 * 1024) / (payload + 32);
        while (cap * 2 <= limit) cap *= 2;
    }
    return cap;
}
static void queries(uint64_t *q, size_t cap, size_t n, int workload, uint64_t seed) {
    for (size_t i = 0; i < cap; ++i) {
        size_t k = (size_t)(random64(&seed) % n);
        int missing = workload == 1 || (workload == 2 && (i & 1));
        q[i] = key_at(k + (missing ? n : 0));
    }
    for (size_t i = cap - 1; i; --i) {
        size_t j = (size_t)(random64(&seed) % (i + 1));
        uint64_t tmp = q[i]; q[i] = q[j]; q[j] = tmp;
    }
}

static uint64_t entry_word(const Table *t, size_t i, size_t w) {
    const void *address = t->variant == BOX ? ((void *const *)t->slots)[i] :
        (const uint8_t *)t->slots + i * t->stride +
            ((t->variant == TAG || t->variant == PROVED) ? 8 : 0);
#define WORD(P) case P: { const Entry##P *e = address; return w == SIZE_MAX ? e->key : e->value[w]; }
    switch (t->payload) { WORD(8) WORD(32) WORD(128) default: abort(); }
#undef WORD
}

static void self_test(void) {
    const size_t sizes[] = {8, 32, 128};
    size_t checked = 0;
    for (size_t p = 0; p < 3; ++p) for (int load = 0; load < 2; ++load) {
        size_t n = load ? 224 : 128;
        uint8_t reference[256];
        uint64_t q[256], reference_sums[4][2];
        for (int v = 0; v < VARIANTS; ++v) {
            Table t = create(256, sizes[p], v);
            for (size_t j = 0; j < n; ++j) assert(t.insert(&t, key_at(j)) == 1);
            if (!v) memcpy(reference, t.ctrl, 256);
            else assert(!memcmp(reference, t.ctrl, 256));
            size_t occupied = 0, displaced = 0;
            for (size_t j = 0; j < 256; ++j) if (t.ctrl[j]) {
                ++occupied;
                uint64_t key = entry_word(&t, j, SIZE_MAX);
                displaced += ((size_t)mix64(key) & 255) != j;
                for (size_t w = 0; w < sizes[p] / 8; ++w)
                    assert(entry_word(&t, j, w) == ((key ^ (UINT64_C(0xe7037ed1a0b428db) + w)) | 1));
            }
            assert(occupied == n && displaced > 0);
            for (size_t j = 0; j < 512; ++j) {
                uint64_t key = key_at(j);
                assert(t.lookup(&t, key) == (j < n ? word_at(key, 0) : 0));
                if (j < n) assert(t.insert(&t, key) == 0);
                ++checked;
            }
            for (int w = 0; w < 4; ++w) {
                queries(q, 256, n, w, 1234 + (uint64_t)w);
                for (int d = 0; d < 2; ++d) {
                    uint64_t expected = 0, state = 7;
                    for (size_t j = 0; j < 1024; ++j) {
                        uint64_t key = q[(d ? state : j + state) & 255], value = 0;
                        for (size_t k = 0; k < n; ++k)
                            if (key_at(k) == key) { value = word_at(key, 0); break; }
                        expected += value;
                        if (d) state = mix64(state ^ value);
                    }
                    expected ^= state;
                    uint64_t actual = t.batch(&t, q, 1024, d, 7);
                    assert(actual == expected);
                    if (!v) reference_sums[w][d] = actual;
                    else assert(actual == reference_sums[w][d]);
                }
            }
            reset(&t);
            uint64_t expected_build = n;
            for (size_t j = 3; j < n; j += 4) expected_build += word_at(key_at(j / 2), 0);
            assert(t.build(&t, n) == expected_build);
            destroy(&t);
        }
    }
    fprintf(stderr, "self-test: %zu exact hit/miss checks; 30 layouts; collision, duplicate, "
        "full payload, build and independent batch oracles passed\n", checked);
}

static int compare_u64(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}
int main(int argc, char **argv) {
    if (argc > 2 || (argc == 2 && strcmp(argv[1], "--self-test"))) {
        fputs("usage: bench [--self-test]\n", stderr); return 1;
    }
    self_test();
    if (argc == 2) return 0;
    uint64_t overhead[10000];
    for (size_t i = 0; i < 10000; ++i) {
        uint64_t a = now_ns(), b = now_ns(); overhead[i] = b - a;
    }
    qsort(overhead, 10000, sizeof(*overhead), compare_u64);
    fprintf(stderr, "empty clock pair ns: min=%" PRIu64 " median=%" PRIu64
        " p75=%" PRIu64 " max=%" PRIu64 "\n", overhead[0], overhead[5000],
        overhead[7500], overhead[9999]);
    const size_t payloads[] = {8, 32, 128};
    const char *tiers[] = {"L1", "L2", "large"};
    const char *workloads[] = {"hit", "miss", "mix", "dependent", "insert"};
    puts("tier,payload,capacity,load,workload,variant,phase,rep,ops,builds,ns_per_op,"
        "checksum,minor_faults,major_faults,invol_cs,table_bytes,entry_bytes,query_bytes,seconds");
    for (int tier = 0; tier < 3; ++tier) for (int p = 0; p < 3; ++p)
    for (int load = 0; load < 2; ++load) {
        size_t cap = capacity(tier, payloads[p]), n = load ? cap * 7 / 8 : cap / 2;
        size_t ops = cap > (1u << 20) ? cap : (1u << 20);
        Table tables[VARIANTS]; size_t heaps[VARIANTS];
        uint64_t *q = allocate(cap * sizeof(*q));
        double lf = (double)n / (double)cap;
        fprintf(stderr, "configuration: %s P=%zu capacity=%zu load=%.3f\n", tiers[tier], payloads[p], cap, lf);
        for (int v = 0; v < VARIANTS; ++v) {
            tables[v] = create(cap, payloads[p], v);
            sink = tables[v].build(&tables[v], n);
            heaps[v] = heap_bytes(&tables[v], n);
        }
        for (int work = 0; work < 5; ++work) {
            if (work != 4) queries(q, cap, n, work, UINT64_C(0x12345678) + work + cap + n);
            size_t build_ops = n + n / 4;
            size_t builds = ((1u << 18) + build_ops - 1) / build_ops;
            for (int rep = -2; rep < 12; ++rep) {
                uint64_t seed = mix64((uint64_t)(rep + 3));
                uint64_t expected_checksum = 0;
                for (int order = 0; order < VARIANTS; ++order) {
                    int v = ((rep + 2) + ((rep & 1) ? VARIANTS - 1 - order : order)) % VARIANTS;
                    Table *t = &tables[v]; Sample s = {0};
                    if (work == 4) {
                        /* One untimed build warms this layout before each sample. */
                        reset(t); sink = t->build(t, n);
                        for (size_t b = 0; b < builds; ++b) {
                            reset(t);
                            measure(t, NULL, build_ops, 0, 0, n, &s);
                        }
                    } else {
                        sink = t->batch(t, q, cap, work == 3, seed);
                        measure(t, q, ops, work == 3, seed, 0, &s);
                    }
                    if (!order) expected_checksum = s.checksum;
                    else if (s.checksum != expected_checksum) {
                        fputs("cross-representation checksum mismatch\n", stderr); return 1;
                    }
                    printf("%s,%zu,%zu,%.3f,%s,%s,%s,%d,%zu,%zu,%.9f,%" PRIu64
                        ",%ld,%ld,%ld,%zu,%zu,%zu,%.9f\n", tiers[tier], payloads[p], cap, lf,
                        workloads[work], names[v], rep < 0 ? "warmup" : "measure", rep < 0 ? rep + 2 : rep,
                        s.ops, s.builds, (double)s.ns / s.ops, s.checksum, s.minor, s.major, s.cs,
                        cap * (t->stride + 1), heaps[v], work == 4 ? 0 : cap * sizeof(*q), (double)s.ns / 1e9);
                }
            }
            fflush(stdout);
            fprintf(stderr, "completed: %s P=%zu load=%.3f %s\n", tiers[tier], payloads[p], lf, workloads[work]);
        }
        for (int v = 0; v < VARIANTS; ++v) destroy(&tables[v]);
        free(q);
    }
    fprintf(stderr, "complete; sink=%" PRIu64 "\n", sink);
    return 0;
}
