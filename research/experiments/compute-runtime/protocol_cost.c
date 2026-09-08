/* Owner-local acquire/publish/join/release cost with helpers held in callbacks.
 * This is not a parallel speedup benchmark or an idle-worker wake benchmark. */
#if !defined(__APPLE__)
#define _POSIX_C_SOURCE 200809L
#endif
#include "runtime.h"
#include <inttypes.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

extern int wf__floor_run(int, char **);
typedef struct {
  uint64_t input, result;
} Frame;
typedef struct {
  pthread_mutex_t lock;
  pthread_cond_t changed;
  unsigned entered;
  bool released;
} Gate;
typedef void (*Callback)(void *);
static void insist(bool condition) {
  if (!condition)
    abort();
}
static uint64_t clock_ns(clockid_t clock) {
  struct timespec t;
  insist(clock_gettime(clock, &t) == 0);
  return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
static uint64_t expected(uint64_t input) {
  return input * UINT64_C(6364136223846793005) + UINT64_C(1442695040888963407);
}
static void leaf(void *opaque) {
  Frame f;
  memcpy(&f, opaque, sizeof(f));
  f.result = expected(f.input);
  memcpy(opaque, &f, sizeof(f));
}
/* Load before timing to retain the same opaque indirect callback in both paths.
 */
static Callback volatile selected_callback = leaf;
static void hold_worker(void *opaque) {
  Gate *gate;
  memcpy(&gate, opaque, sizeof(gate));
  insist(pthread_mutex_lock(&gate->lock) == 0);
  ++gate->entered;
  insist(pthread_cond_broadcast(&gate->changed) == 0);
  while (!gate->released)
    insist(pthread_cond_wait(&gate->changed, &gate->lock) == 0);
  insist(pthread_mutex_unlock(&gate->lock) == 0);
}
static void direct_batch(size_t count, unsigned depth, uint64_t *output,
                         Callback fn) {
  Frame frames[32];
  for (size_t base = 0; base < count; base += depth) {
    for (unsigned j = 0; j < depth; ++j)
      frames[j] = (Frame){base + j + 1, ~(uint64_t)(base + j + 1)};
    for (unsigned j = depth; j--;) {
      fn(&frames[j]);
      output[base + j] = frames[j].result;
    }
  }
}
static void task_batch(size_t count, unsigned depth, uint64_t *output,
                       Callback fn) {
  void *frames[32];
  for (size_t base = 0; base < count; base += depth) {
    for (unsigned j = 0; j < depth; ++j) {
      Frame f = {base + j + 1, ~(uint64_t)(base + j + 1)};
      frames[j] = wf__par_acquire_lane(sizeof(f));
      insist(frames[j] != NULL);
      memcpy(frames[j], &f, sizeof(f));
      wf__par_publish(frames[j], fn);
    }
    for (unsigned j = depth; j--;) {
      Frame f;
      wf__par_join(frames[j]);
      memcpy(&f, frames[j], sizeof(f));
      output[base + j] = f.result;
      wf__par_release(frames[j]);
    }
  }
}
int wf__main_body(int argc, char **argv) {
  insist(argc == 4);
  unsigned width = (unsigned)strtoul(argv[1], NULL, 10),
           depth = (unsigned)strtoul(argv[2], NULL, 10);
  unsigned pass = (unsigned)strtoul(argv[3], NULL, 10);
  insist((strcmp(argv[1], "2") == 0 || strcmp(argv[1], "4") == 0) &&
         (strcmp(argv[2], "1") == 0 || strcmp(argv[2], "8") == 0 ||
          strcmp(argv[2], "32") == 0) &&
         strlen(argv[3]) == 1 && argv[3][0] >= '0' && argv[3][0] <= '4');
  insist((width == 2 || width == 4) &&
         (depth == 1 || depth == 8 || depth == 32) && pass < 5);
  insist(getenv("WF_WORKERS") &&
         strtoul(getenv("WF_WORKERS"), NULL, 10) == width);
  alarm(30);
  Gate gate = {PTHREAD_MUTEX_INITIALIZER, PTHREAD_COND_INITIALIZER, 0, false};
  void *held[3];
  for (unsigned i = 0; i < width - 1; ++i) {
    held[i] = wf__par_acquire_lane(sizeof(Gate *));
    insist(held[i] != NULL);
    insist(wf_compute_worker_count() == width);
    Gate *address = &gate;
    memcpy(held[i], &address, sizeof(address));
    wf__par_publish(held[i], hold_worker);
    insist(pthread_mutex_lock(&gate.lock) == 0);
    while (gate.entered != i + 1)
      insist(pthread_cond_wait(&gate.changed, &gate.lock) == 0);
    insist(pthread_mutex_unlock(&gate.lock) == 0);
  }
  insist(wf_compute_worker_count() == width);
  unsigned long setup_steals = wf__par_grants();
  insist(setup_steals == width - 1);
  size_t count = 262144;
  uint64_t *output = malloc(count * sizeof(*output));
  insist(output != NULL);
  Callback fn = selected_callback;
  uint64_t warm_start = clock_ns(CLOCK_MONOTONIC_RAW);
  unsigned warm_batches = 0;
  do {
    direct_batch(count, depth, output, fn);
    task_batch(count, depth, output, fn);
    ++warm_batches;
  } while (clock_ns(CLOCK_MONOTONIC_RAW) - warm_start < UINT64_C(100000000));
  uint64_t warm_ns = clock_ns(CLOCK_MONOTONIC_RAW) - warm_start;
  for (size_t i = 0; i < count; ++i)
    insist(output[i] == expected(i + 1));
  insist(wf__par_grants() == setup_steals);
  printf("# workers=%u blocked=%u depth=%u tasks=%zu pass=%u warmup_pairs=%u "
         "warmup_ns=%" PRIu64 "\n",
         width, width - 1, depth, count, pass, warm_batches, warm_ns);
  puts("sample\tphase\tmode\twall_ns\towner_cpu_ns\tsteals");
  for (unsigned sample = 0; sample < 9; ++sample) {
    for (unsigned order = 0; order < 2; ++order) {
      bool task = (order + sample + pass) % 2 != 0;
      unsigned long before = wf__par_grants();
      uint64_t cpu = clock_ns(CLOCK_THREAD_CPUTIME_ID),
               start = clock_ns(CLOCK_MONOTONIC_RAW);
      if (task)
        task_batch(count, depth, output, fn);
      else
        direct_batch(count, depth, output, fn);
      uint64_t wall = clock_ns(CLOCK_MONOTONIC_RAW) - start;
      cpu = clock_ns(CLOCK_THREAD_CPUTIME_ID) - cpu;
      unsigned long steals = wf__par_grants() - before;
      insist(steals == 0);
      for (size_t i = 0; i < count; ++i)
        insist(output[i] == expected(i + 1));
      printf("%u\t%s\t%s\t%" PRIu64 "\t%" PRIu64 "\t%lu\n", sample,
             sample ? "warm" : "first", task ? "task" : "direct", wall, cpu,
             steals);
    }
  }
  insist(pthread_mutex_lock(&gate.lock) == 0);
  gate.released = true;
  insist(pthread_cond_broadcast(&gate.changed) == 0);
  insist(pthread_mutex_unlock(&gate.lock) == 0);
  for (unsigned i = width - 1; i--;) {
    wf__par_join(held[i]);
    wf__par_release(held[i]);
  }
  insist(pthread_cond_destroy(&gate.changed) == 0);
  insist(pthread_mutex_destroy(&gate.lock) == 0);
  free(output);
  alarm(0);
  puts("# protocol PASS: samples=18 outputs=4718592");
  insist(fflush(stdout) == 0);
  return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
