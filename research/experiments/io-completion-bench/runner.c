/* The io-completion-bench timing harness.
 *
 * Reads a plan of labelled commands, runs each one warmup times and then
 * `rounds` recorded times, and prints one row per label with the median and
 * the observed spread of wall time plus the median child user and system CPU.
 * Every recorded run's stdout is compared with the expected checksum line, so
 * a line that publishes different bytes cannot report a time at all.
 *
 * Plan lines are tab separated:
 *     label <TAB> KEY=VALUE,KEY=VALUE <TAB> command <TAB> arg <TAB> ...
 * An empty environment field sets nothing. Lines starting with '#' and empty
 * lines are ignored. */
#define _GNU_SOURCE
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define MAX_ARGUMENTS 32
#define MAX_ROUNDS 128
#define MAX_OUTPUT 4096

struct sample {
    double wall_ms;
    double user_ms;
    double system_ms;
};

static int compare_double(const void *left, const void *right) {
    double a = *(const double *)left;
    double b = *(const double *)right;
    return (a > b) - (a < b);
}

static double median_of(double *values, size_t count) {
    qsort(values, count, sizeof *values, compare_double);
    if (count % 2 == 1) {
        return values[count / 2];
    }
    return (values[count / 2 - 1] + values[count / 2]) / 2.0;
}

static double milliseconds(struct timeval value) {
    return (double)value.tv_sec * 1000.0 + (double)value.tv_usec / 1000.0;
}

/* Runs one command once. Returns 0 on a run whose exit status was zero and
 * whose stdout matched `expected`; the caller decides what a failure means. */
static int run_once(char **argument, char *environment, const char *expected,
                    struct sample *out) {
    int channel[2];
    if (pipe(channel) != 0) {
        perror("runner: pipe");
        return -1;
    }
    struct timespec start;
    clock_gettime(CLOCK_MONOTONIC, &start);
    pid_t child = fork();
    if (child < 0) {
        perror("runner: fork");
        close(channel[0]);
        close(channel[1]);
        return -1;
    }
    if (child == 0) {
        close(channel[0]);
        if (dup2(channel[1], 1) < 0) {
            _exit(120);
        }
        close(channel[1]);
        if (environment != NULL && environment[0] != '\0') {
            char *cursor = environment;
            while (cursor != NULL && *cursor != '\0') {
                char *comma = strchr(cursor, ',');
                if (comma != NULL) {
                    *comma = '\0';
                }
                char *equals = strchr(cursor, '=');
                if (equals != NULL) {
                    *equals = '\0';
                    setenv(cursor, equals + 1, 1);
                    *equals = '=';
                }
                if (comma == NULL) {
                    break;
                }
                *comma = ',';
                cursor = comma + 1;
            }
        }
        execv(argument[0], argument);
        _exit(121);
    }
    close(channel[1]);
    char output[MAX_OUTPUT];
    size_t filled = 0;
    for (;;) {
        ssize_t moved = read(channel[0], output + filled, sizeof output - 1 - filled);
        if (moved <= 0) {
            break;
        }
        filled += (size_t)moved;
        if (filled >= sizeof output - 1) {
            break;
        }
    }
    output[filled] = '\0';
    close(channel[0]);
    int status = 0;
    struct rusage usage;
    memset(&usage, 0, sizeof usage);
    if (wait4(child, &status, 0, &usage) < 0) {
        perror("runner: wait4");
        return -1;
    }
    struct timespec end;
    clock_gettime(CLOCK_MONOTONIC, &end);
    out->wall_ms = ((double)(end.tv_sec - start.tv_sec) * 1000.0) +
                   ((double)(end.tv_nsec - start.tv_nsec) / 1000000.0);
    out->user_ms = milliseconds(usage.ru_utime);
    out->system_ms = milliseconds(usage.ru_stime);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        fprintf(stderr, "runner: %s exited with status %d\n", argument[0], status);
        return 1;
    }
    /* One trailing newline is a property of how the line was captured, not of
     * the bytes the program decided to publish, so it is trimmed on both
     * sides before the comparison. Everything else must match exactly. */
    while (filled > 0 && (output[filled - 1] == 0x0a || output[filled - 1] == 0x0d)) {
        output[--filled] = 0;
    }
    if (expected != NULL && expected[0] != 0) {
        size_t wanted = strlen(expected);
        while (wanted > 0 && (expected[wanted - 1] == 0x0a || expected[wanted - 1] == 0x0d)) {
            wanted--;
        }
        if (filled != wanted || memcmp(output, expected, wanted) != 0) {
            fprintf(stderr, "runner: %s published [%s], expected [%s]\n", argument[0], output,
                    expected);
            return 1;
        }
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 5) {
        fprintf(stderr, "usage: runner PLAN ROUNDS WARMUP EXPECTED\n");
        return 2;
    }
    FILE *plan = fopen(argv[1], "r");
    if (plan == NULL) {
        perror("runner: open plan");
        return 1;
    }
    unsigned long rounds = strtoul(argv[2], NULL, 10);
    unsigned long warmup = strtoul(argv[3], NULL, 10);
    const char *expected = argv[4];
    if (rounds == 0 || rounds > MAX_ROUNDS) {
        fprintf(stderr, "runner: ROUNDS must be 1..%d\n", MAX_ROUNDS);
        fclose(plan);
        return 2;
    }
    printf("%-34s %10s %10s %10s %10s %10s\n", "line", "median_ms", "min_ms", "max_ms", "user_ms",
           "sys_ms");
    char line[4096];
    int failures = 0;
    while (fgets(line, sizeof line, plan) != NULL) {
        size_t length = strlen(line);
        while (length > 0 && (line[length - 1] == '\n' || line[length - 1] == '\r')) {
            line[--length] = '\0';
        }
        if (length == 0 || line[0] == '#') {
            continue;
        }
        char *cursor = line;
        char *label = strsep(&cursor, "\t");
        char *environment = strsep(&cursor, "\t");
        char *argument[MAX_ARGUMENTS];
        size_t count = 0;
        while (cursor != NULL && count < MAX_ARGUMENTS - 1) {
            char *piece = strsep(&cursor, "\t");
            if (piece == NULL) {
                break;
            }
            if (piece[0] == '\0') {
                continue;
            }
            argument[count++] = piece;
        }
        argument[count] = NULL;
        if (label == NULL || count == 0) {
            fprintf(stderr, "runner: malformed plan line\n");
            failures++;
            continue;
        }
        struct sample sample;
        int broken = 0;
        for (unsigned long at = 0; at < warmup; at++) {
            if (run_once(argument, environment, expected, &sample) != 0) {
                broken = 1;
                break;
            }
        }
        if (broken) {
            fprintf(stderr, "runner: %s failed during warmup\n", label);
            failures++;
            continue;
        }
        double wall[MAX_ROUNDS];
        double user[MAX_ROUNDS];
        double system_time[MAX_ROUNDS];
        for (unsigned long at = 0; at < rounds; at++) {
            if (run_once(argument, environment, expected, &sample) != 0) {
                broken = 1;
                break;
            }
            wall[at] = sample.wall_ms;
            user[at] = sample.user_ms;
            system_time[at] = sample.system_ms;
        }
        if (broken) {
            fprintf(stderr, "runner: %s failed during measurement\n", label);
            failures++;
            continue;
        }
        double low = wall[0];
        double high = wall[0];
        for (unsigned long at = 1; at < rounds; at++) {
            if (wall[at] < low) {
                low = wall[at];
            }
            if (wall[at] > high) {
                high = wall[at];
            }
        }
        double middle = median_of(wall, rounds);
        double user_middle = median_of(user, rounds);
        double system_middle = median_of(system_time, rounds);
        printf("%-34s %10.2f %10.2f %10.2f %10.2f %10.2f\n", label, middle, low, high,
               user_middle, system_middle);
        fflush(stdout);
    }
    fclose(plan);
    return failures == 0 ? 0 : 1;
}
