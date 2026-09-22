#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#ifdef __APPLE__
#include <mach/mach.h>
#endif
extern int wf__floor_run(int, char **);
extern uint64_t wf_bench_batch(uint64_t **, uint64_t *, uint64_t);
extern unsigned long wf__par_grants(void);
static uint64_t wall_ns(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) exit(20);
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
#ifdef __APPLE__
static uint64_t time_ns(time_value_t t) {
    return (uint64_t)t.seconds * UINT64_C(1000000000) + (uint64_t)t.microseconds * 1000;
}
static uint64_t cpu_ns(void) {
    task_thread_times_info_data_t live;
    task_basic_info_data_t exited;
    mach_msg_type_number_t live_count = TASK_THREAD_TIMES_INFO_COUNT;
    mach_msg_type_number_t exited_count = TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), TASK_THREAD_TIMES_INFO, (task_info_t)&live,
                  &live_count) != KERN_SUCCESS ||
        task_info(mach_task_self(), TASK_BASIC_INFO, (task_info_t)&exited,
                  &exited_count) != KERN_SUCCESS) exit(21);
    return time_ns(live.user_time) + time_ns(live.system_time) +
           time_ns(exited.user_time) + time_ns(exited.system_time);
}
#else
static uint64_t cpu_ns(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &t)) exit(21);
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
#endif
int wf__main_body(int argc, char **argv) {
    if (argc != 5) return 22;
    const char *workers = getenv("WF_WORKERS");
    if (!workers) return 23;
    uint64_t count = strtoull(argv[3], NULL, 10);
    uint64_t rows = strtoull(argv[4], NULL, 10);
    if (count > 65536 || rows == 0 || rows > 1024) return 24;
    uint64_t *input = malloc((size_t)(count + 1) * sizeof(*input));
    uint64_t *output = calloc((size_t)rows, sizeof(*output));
    uint64_t *expected = malloc((size_t)rows * sizeof(*expected));
    if (!input || !output || !expected) return 25;
    input[0] = count;
    for (uint64_t i = 0; i < count; ++i)
        input[i + 1] = (i * UINT64_C(37)) ^ UINT64_C(0x9e3779b97f4a7c15);
    for (uint64_t row = 0; row < rows; ++row) {
        uint64_t value = row + 17;
        for (uint64_t i = 0; i < count; ++i)
            value = (value ^ input[i + 1]) * UINT64_C(6364136223846793005);
        expected[row] = value;
    }
    for (unsigned sample = 0; sample <= 5; ++sample) {
        unsigned long before_grants = wf__par_grants();
        uint64_t cpu_before = cpu_ns(), wall_before = wall_ns();
        uint64_t actual = wf_bench_batch(&input, output, rows);
        uint64_t wall = wall_ns() - wall_before, cpu = cpu_ns() - cpu_before;
        unsigned long grants = wf__par_grants() - before_grants;
        if (actual != rows) return 26;
        for (uint64_t row = 0; row < rows; ++row)
            if (output[row] != expected[row]) return 27;
        if (input[0] != count) return 28;
        for (uint64_t i = 0; i < count; ++i)
            if (input[i + 1] != ((i * UINT64_C(37)) ^ UINT64_C(0x9e3779b97f4a7c15)))
                return 29;
        printf("%s\t%s\t%s\t%llu\t%llu\t%u\t%llu\t%llu\t%lu\n",
               argv[1], workers, argv[2], (unsigned long long)count,
               (unsigned long long)rows, sample, (unsigned long long)wall,
               (unsigned long long)cpu, grants);
    }
    free(expected);
    free(output);
    free(input);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
