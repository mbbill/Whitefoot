#define _GNU_SOURCE
/* Matched native traversal of the ordinary linked interface. Compile-time
 * routes differ only at the view bridge, never through a function pointer.
 * Owned by ordinary-check / ordinary-bench; retire with their attribution. */
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include "workload.h"
#include "ordinary_values.h"
#include "completion/bridge.h"

#ifndef WF_BENCH_PRIVATE_BODY
#define WF_BENCH_PRIVATE_BODY 0
#endif
#if WF_BENCH_PRIVATE_BODY
#define bench_open bench_body_open
#define bench_read bench_body_read
#else
#define bench_open bench_public_open
#define bench_read bench_public_read
#endif
extern void bench_open(wf_open_result *, wf_value *, const wf_value *, const wf_view *, uint64_t, uint64_t);
extern void bench_read(wf_read_result *, wf_value *, wf_value *, wf_view *, uint64_t, uint64_t, uint64_t);

static void close_file(wf_inputs *inputs, const wf_value *file) {
    wf_close_result closed;
    wf_close_read(&closed, &inputs->handles, file);
    assert(closed.tag == 0);
}

static void check_routes(wf_inputs *inputs, wf_view *window) {
    const uint64_t credits = inputs->handles.words[0];
    char filename[] = "f00000.dat";
    wf_view name = { filename, 10 };
    wf_open_result opened;
    wf_read_result read;
    wf_value empty_factory = { { 0 } };
    bench_open(&opened, &empty_factory, &inputs->cwd, &name, 0, 10);
    assert(opened.tag == 1 && opened.error.tag == 21 && empty_factory.words[0] == 0);
    bench_open(&opened, &inputs->handles, &inputs->cwd, &name, 0, 10);
    assert(opened.tag == 0 && inputs->handles.words[0] == credits - 1);
    bench_read(&read, &inputs->handles, &opened.value, window, 0, 0, window->length);
    size_t length = wf_bench_file_length(0, 16);
    assert(read.tag == 0 && read.value == length);
    unsigned char *expected = malloc(length);
    assert(expected != NULL);
    wf_bench_file_bytes(0, expected, length);
    assert(memcmp(expected, window->data, length) == 0);
    free(expected);
    bench_read(&read, &inputs->handles, &opened.value, window, length, 0, window->length);
    assert(read.tag == 1 && read.error.tag == 0); /* EOF, not an I/O error. */
    close_file(inputs, &opened.value);
    assert(inputs->handles.words[0] == credits);
    char missing[] = "missing.dat";
    name.data = missing; name.length = sizeof(missing) - 1;
    bench_open(&opened, &inputs->handles, &inputs->cwd, &name, 0, name.length);
    assert(opened.tag == 1 && opened.error.tag == 0 && inputs->handles.words[0] == credits);
    char invalid[] = "bad/name";
    name.data = invalid; name.length = sizeof(invalid) - 1;
    bench_open(&opened, &inputs->handles, &inputs->cwd, &name, 0, name.length);
    assert(opened.tag == 1 && opened.error.tag == 9 && inputs->handles.words[0] == credits);
}

int wf__main_body(int argc, char **argv) {
    assert(argc == 3 && chdir(argv[1]) == 0);
    wf_inputs inputs;
    assert(wf__ordinary_inputs(&inputs, 0, NULL));
    const uint64_t credits = inputs.handles.words[0];
    assert(credits >= 1);
    unsigned char *data = calloc(WF_BENCH_READ_WINDOW, 1);
    assert(data != NULL);
    wf_view window = { data, WF_BENCH_READ_WINDOW };
    if (strcmp(argv[2], "check") == 0) {
        check_routes(&inputs, &window);
        puts("ordinary caller: data, EOF, refusal and credits passed");
    } else {
        const uint64_t count = strtoull(argv[2], NULL, 10);
        uint64_t sum = 0, bytes = 0;
        for (uint64_t index = 0; index < count; index++) {
            char filename[32];
            snprintf(filename, sizeof(filename), WF_BENCH_NAME_FORMAT, (unsigned long)index);
            wf_view name = { filename, WF_BENCH_NAME_BYTES };
            wf_open_result opened;
            bench_open(&opened, &inputs.handles, &inputs.cwd, &name, 0, name.length);
            if (opened.tag != 0) continue; /* Same missing-file outcome as WF. */
            wf_read_result read;
            bench_read(&read, &inputs.handles, &opened.value, &window, 0, 0, window.length);
            if (read.tag == 0) {
                assert(read.value <= window.length);
                sum += wf_bench_weighted(wf_bench_digest(data, (size_t)read.value), index);
                bytes += read.value;
            }
            close_file(&inputs, &opened.value);
        }
        printf("%020" PRIu64 " %020" PRIu64 "\n", sum, bytes);
        if (getenv("WF_BENCH_OBSERVE") != NULL) {
            fprintf(stderr, "submissions=%" PRIu64 " inline=%" PRIu64 " helper=%" PRIu64 " ring=%" PRIu64 "\n",
                wf__completion_file_submissions(), wf__completion_inline_executions(),
                wf__completion_target_helper_executions(), wf__completion_native_ring_submissions());
        }
    }
    assert(inputs.handles.words[0] == credits);
    wf_close_result closed;
    wf_close_directory(&closed, &inputs.handles, &inputs.cwd);
    assert(closed.tag == 0);
    free(data);
    return 0;
}

/* Use the same process floor as generated WF executables. */
extern int wf__floor_run(int argc, char **argv);
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
