/* The io-completion-bench timing harness.
 *
 * Reads a plan of labelled commands and runs the whole plan as a pass, over
 * and over: `warmup` unrecorded passes and then `rounds` recorded ones,
 * reversing the plan's order on every other pass. It then prints one row per
 * label with the median across passes and the observed spread of wall time,
 * plus the median child user and system CPU. Every recorded run's stdout is
 * compared with the expected checksum line, so a line that publishes
 * different bytes cannot report a time at all.
 *
 * Passes, rather than all of one line's runs and then all of the next's,
 * because the hosts these tables are taken on drift. A shared runner's disk,
 * its neighbours, and a laptop's thermal state all change over the minutes a
 * table takes, and a grouped schedule turns that drift into a difference
 * between lines: the line that ran first is measured against a different
 * machine from the line that ran last. Interleaving spreads the drift across
 * every line instead, and reversing alternate passes cancels the residue of
 * position within a pass -- the line that runs last in a forward pass runs
 * first in the next one, so a systematic first-in-pass or last-in-pass cost
 * lands on every line equally.
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
#define MAX_LINES 128
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

/* One plan line, with the samples every recorded pass leaves in it. */
struct plan_line {
    char *storage;
    char *label;
    char *environment;
    char *argument[MAX_ARGUMENTS];
    size_t count;
    double wall[MAX_ROUNDS];
    double user[MAX_ROUNDS];
    double system_time[MAX_ROUNDS];
    size_t recorded;
    int broken;
};

/* Splits one plan line in place. The caller owns `storage` until the run is
 * over, because every pointer below points into it. */
static int parse_plan_line(struct plan_line *line, char *storage) {
    line->storage = storage;
    line->count = 0;
    line->recorded = 0;
    line->broken = 0;
    char *cursor = storage;
    line->label = strsep(&cursor, "\t");
    line->environment = strsep(&cursor, "\t");
    while (cursor != NULL && line->count < MAX_ARGUMENTS - 1) {
        char *piece = strsep(&cursor, "\t");
        if (piece == NULL) {
            break;
        }
        if (piece[0] == '\0') {
            continue;
        }
        line->argument[line->count++] = piece;
    }
    line->argument[line->count] = NULL;
    return line->label != NULL && line->count != 0;
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
    struct plan_line *lines = calloc(MAX_LINES, sizeof *lines);
    if (lines == NULL) {
        fprintf(stderr, "runner: out of memory\n");
        fclose(plan);
        return 1;
    }
    size_t count = 0;
    char text[4096];
    int failures = 0;
    while (fgets(text, sizeof text, plan) != NULL) {
        size_t length = strlen(text);
        while (length > 0 && (text[length - 1] == '\n' || text[length - 1] == '\r')) {
            text[--length] = '\0';
        }
        if (length == 0 || text[0] == '#') {
            continue;
        }
        if (count == MAX_LINES) {
            fprintf(stderr, "runner: a plan may hold at most %d lines\n", MAX_LINES);
            fclose(plan);
            free(lines);
            return 2;
        }
        char *storage = strdup(text);
        if (storage == NULL || !parse_plan_line(&lines[count], storage)) {
            fprintf(stderr, "runner: malformed plan line\n");
            free(storage);
            fclose(plan);
            free(lines);
            return 2;
        }
        count++;
    }
    fclose(plan);
    if (count == 0) {
        fprintf(stderr, "runner: the plan is empty\n");
        free(lines);
        return 2;
    }

    /* Passes, alternating direction. The pass index decides the direction, so
     * the warm-up passes alternate too and the first recorded pass follows
     * the last warm-up one rather than repeating it. */
    unsigned long passes = warmup + rounds;
    for (unsigned long pass = 0; pass < passes; pass++) {
        int forward = pass % 2 == 0;
        int recording = pass >= warmup;
        fprintf(stderr, "runner: pass %lu of %lu (%s%s)\n", pass + 1, passes,
                forward ? "plan order" : "reversed", recording ? "" : ", warm-up");
        for (size_t step = 0; step < count; step++) {
            struct plan_line *line = &lines[forward ? step : count - 1 - step];
            if (line->broken) {
                continue;
            }
            struct sample sample;
            if (run_once(line->argument, line->environment, expected, &sample) != 0) {
                fprintf(stderr, "runner: %s failed on pass %lu\n", line->label, pass + 1);
                line->broken = 1;
                failures++;
                continue;
            }
            if (!recording) {
                continue;
            }
            line->wall[line->recorded] = sample.wall_ms;
            line->user[line->recorded] = sample.user_ms;
            line->system_time[line->recorded] = sample.system_ms;
            line->recorded++;
        }
    }

    printf("%-34s %10s %10s %10s %10s %10s\n", "line", "median_ms", "min_ms", "max_ms", "user_ms",
           "sys_ms");
    for (size_t at = 0; at < count; at++) {
        struct plan_line *line = &lines[at];
        if (line->broken || line->recorded == 0) {
            continue;
        }
        double low = line->wall[0];
        double high = line->wall[0];
        for (size_t pass = 1; pass < line->recorded; pass++) {
            if (line->wall[pass] < low) {
                low = line->wall[pass];
            }
            if (line->wall[pass] > high) {
                high = line->wall[pass];
            }
        }
        double middle = median_of(line->wall, line->recorded);
        double user_middle = median_of(line->user, line->recorded);
        double system_middle = median_of(line->system_time, line->recorded);
        printf("%-34s %10.2f %10.2f %10.2f %10.2f %10.2f\n", line->label, middle, low, high,
               user_middle, system_middle);
    }
    fflush(stdout);
    for (size_t at = 0; at < count; at++) {
        free(lines[at].storage);
    }
    free(lines);
    return failures == 0 ? 0 : 1;
}
