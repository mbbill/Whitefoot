#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef struct { uint8_t *p; int64_t n; } Buf;
typedef struct {
  uint64_t lines, words, bytes, first_space, last_space;
} Summary;

extern void summarize(Summary *out, Buf b);
extern void combine(Summary *out, const Summary *a, const Summary *b);

typedef struct { Buf b; Summary out; } Job;

static void *worker(void *opaque) {
  Job *job = opaque;
  summarize(&job->out, job->b);
  return NULL;
}

static uint64_t now_ns(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

static Buf slurp(const char *path) {
  FILE *f = fopen(path, "rb");
  if (!f) { perror(path); exit(1); }
  if (fseek(f, 0, SEEK_END) != 0) { perror("fseek"); exit(1); }
  long size = ftell(f);
  if (size < 0 || fseek(f, 0, SEEK_SET) != 0) { perror("ftell/fseek"); exit(1); }
  Buf b = {malloc(size ? (size_t)size : 1), size};
  if (!b.p) { perror("malloc"); exit(1); }
  if (size && fread(b.p, 1, (size_t)size, f) != (size_t)size) {
    perror("fread"); exit(1);
  }
  fclose(f);
  return b;
}

static Summary run_once(Buf input, int threads) {
  if (threads == 1) {
    Summary out;
    summarize(&out, input);
    return out;
  }
  Job *jobs = calloc((size_t)threads, sizeof(*jobs));
  pthread_t *ids = calloc((size_t)threads, sizeof(*ids));
  if (!jobs || !ids) { perror("calloc"); exit(1); }
  for (int i = 0; i < threads; i++) {
    int64_t begin = input.n * i / threads;
    int64_t end = input.n * (i + 1) / threads;
    jobs[i].b = (Buf){input.p + begin, end - begin};
    int rc = pthread_create(&ids[i], NULL, worker, &jobs[i]);
    if (rc) { errno = rc; perror("pthread_create"); exit(1); }
  }
  for (int i = 0; i < threads; i++) {
    int rc = pthread_join(ids[i], NULL);
    if (rc) { errno = rc; perror("pthread_join"); exit(1); }
  }
  Summary out = {0, 0, 0, 1, 1};
  for (int i = 0; i < threads; i++) {
    Summary next;
    combine(&next, &out, &jobs[i].out);
    out = next;
  }
  free(ids);
  free(jobs);
  return out;
}

int main(int argc, char **argv) {
  if (argc < 2 || argc > 4) {
    fprintf(stderr, "usage: %s FILE [THREADS] [REPEATS]\n", argv[0]);
    return 2;
  }
  int threads = argc >= 3 ? atoi(argv[2]) : 1;
  int repeats = argc >= 4 ? atoi(argv[3]) : 1;
  if (threads < 1 || threads > 64 || repeats < 1) return 2;
  Buf input = slurp(argv[1]);
  Summary out = {0};
  uint64_t best = UINT64_MAX;
  for (int i = 0; i < repeats; i++) {
    uint64_t start = now_ns();
    out = run_once(input, threads);
    uint64_t elapsed = now_ns() - start;
    if (elapsed < best) best = elapsed;
  }
  printf("%llu %llu %llu threads=%d best_ns=%llu\n",
         (unsigned long long)out.lines, (unsigned long long)out.words,
         (unsigned long long)out.bytes, threads, (unsigned long long)best);
  free(input.p);
  return 0;
}
